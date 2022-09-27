use crate::{
    channel::Channel,
    cli::Config,
    connection,
    envvars::{replace_template_env, Env},
    error::AppResult,
    metrics::Metrics,
    process,
    types::{
        ConnID, Event, EventRx, EventTx, FromProcessTx, PortID, RoomID, ShutdownTx, ToProcessTx,
    },
    utils::new_conn_id,
};

use {
    futures::{FutureExt, TryFutureExt},
    id_pool::IdPool as PortPool,
    std::collections::{HashMap, HashSet},
    std::sync::Arc,
    tokio::sync::Barrier,
    tracing::{instrument, Instrument},
    warp::ws::WebSocket,
};

type ConnectionMap = HashMap<RoomID, HashSet<ConnID>>;
type ProcessMap = HashMap<RoomID, (FromProcessTx, ToProcessTx, ShutdownTx)>;

struct State {
    pub conns: ConnectionMap,
    pub procs: ProcessMap,
    pub ports: PortPool,
    pub cfg: Config,
}

#[instrument(name = "event", skip_all)]
pub async fn handle(
    tx: EventTx,
    mut rx: EventRx,
    config: Config,
    metrics: Metrics,
) -> AppResult<()> {
    let mut state = State::new(config);

    while let Some(event) = rx.recv().await {
        match event {
            Event::Connect { room, ws, env } if state.procs.contains_key(&room) => {
                metrics.inc_ws_connections(&room);
                attach(room, env, ws, &tx, &mut state, None);
            }
            Event::Connect { room, ws, env } => {
                metrics.inc_ws_connections(&room);
                let spawn_barrier = Some(Arc::new(Barrier::new(2)));
                let attach_barrier = spawn_barrier.clone();

                spawn(&room, &env, &tx, &mut state, spawn_barrier).ok();
                attach(room, env, ws, &tx, &mut state, attach_barrier);
            }
            Event::Disconnect { room, conn, env } => {
                metrics.dec_ws_connections(&room);
                disconnect(room, env, conn, &mut state);
            }
            Event::ProcessExit { room, code, port } => {
                exit(room, code, port, &mut state);
            }
            Event::Shutdown => {
                shutdown(state);
                break;
            }
        }
    }
    Ok(())
}

impl State {
    pub fn new(cfg: Config) -> Self {
        Self {
            conns: HashMap::new(),
            procs: HashMap::new(),
            ports: PortPool::new_ranged(cfg.tcpports.clone()),
            cfg,
        }
    }
}

#[instrument(name = "attach", skip(env, ws, tx, state, barrier))]
fn attach(
    room: RoomID,
    env: Env,
    ws: Box<WebSocket>,
    tx: &EventTx,
    state: &mut State,
    barrier: Option<Arc<Barrier>>,
) {
    let conn = new_conn_id();

    // Get process handles from map
    let (proc_tx_broadcast, proc_tx, _) = state.procs.get(&room).expect("room not in process map");
    let proc_rx = proc_tx_broadcast.subscribe();

    let mut on_init = || {
        // Store connection handle in map
        let is_inserted = state
            .conns
            .entry(room.to_string())
            .or_default()
            .insert(conn);

        if is_inserted {
            tracing::info! { id = conn, "client connected" };

            // Inform child
            if let Some(ref join_msg_template) = state.cfg.joinmsg {
                let join_msg = replace_template_env(join_msg_template, conn, &env);
                let _ = proc_tx.send(join_msg.into());
            }
        }
    };

    let on_disconnect = || {
        let tx = tx.clone();
        let room = room.clone();
        let env = env.clone();

        // Return callback for connection::handle
        async move |_| {
            tracing::debug! { id=conn, "client disconnecting" };
            let _ = tx.send(Event::Disconnect { room, conn, env });
        }
    };

    tokio::spawn(
        connection::handle(*ws, proc_rx, proc_tx.clone(), barrier)
            .then({
                // NOTE: we invoke on_init closure immediately...
                on_init();
                // NOTE: ...and then invoke a closure returning the async callback closure
                on_disconnect()
            })
            .in_current_span(),
    );
}

#[instrument(name = "spawn", skip(env, tx, state, barrier))]
fn spawn(
    room: &RoomID,
    env: &Env,
    tx: &EventTx,
    state: &mut State,
    barrier: Option<Arc<Barrier>>,
) -> AppResult<()> {
    let port = state.ports.request_id();

    if let Some(port) = port {
        tracing::debug!("reserved port {}", port);
    }

    let mut proc = Channel::new(&state.cfg, port, env.cgi.clone());
    let senders = proc.take_senders();

    let on_init = || {
        // Store sender handles in map
        state.procs.insert(room.to_string(), senders);
    };

    let on_kill = || {
        let tx = tx.clone();
        let room = room.to_string();

        // Return callback for process::handle
        move |code: Option<i32>| {
            // if sending fails, the events::handle has already been torn down
            let _ = tx.send(Event::ProcessExit { room, code, port });
            Ok(())
        }
    };

    tokio::spawn(
        process::handle(proc, barrier)
            .map_ok_or_else(
                move |e| {
                    tracing::error!("{}", e);
                    Err(e)
                },
                {
                    // NOTE: we invoke on_init closure immediately...
                    on_init();
                    // NOTE: ...and then invoke a closure returning the callback closure
                    on_kill()
                },
            )
            .in_current_span(),
    );

    Ok(())
}

#[instrument(name = "disconnect", skip(env, conn, state))]
fn disconnect(room: RoomID, env: Env, conn: ConnID, state: &mut State) {
    let room_conns = state.conns.entry(room.clone()).or_default();

    // Get process handles from map
    // TODO bug this will prevent leaving room after process has quit
    let (_, proc_tx, _) = state.procs.get(&room).expect("room not in process map");

    let is_removed = room_conns.remove(&conn);

    if is_removed {
        tracing::info! { id = conn, "client disconnected" };

        // Inform child
        if let Some(ref leave_msg_template) = state.cfg.leavemsg {
            let leave_msg = replace_template_env(leave_msg_template, conn, &env);
            let _ = proc_tx.send(leave_msg.into());
        }
    }

    if room_conns.is_empty() {
        if let Some((_, _, kill_tx)) = state.procs.remove(&room) {
            if kill_tx.send(()).is_ok() {
                // Only log if kill was sent
                tracing::info! { "all clients disconnected, killing process" };
            }
        }
    }
}

#[instrument(name = "exit", skip(code, port, state))]
fn exit(room: RoomID, code: Option<i32>, port: Option<PortID>, state: &mut State) {
    if let Some(port) = port {
        let _ = state.ports.return_id(port);
        tracing::debug!("released port {}", port);
    }

    if state.procs.contains_key(&room) {
        tracing::error! { room, code, "process exited" };
        // TODO inform clients
    }
}

#[instrument(name = "shutdown", skip_all)]
fn shutdown(state: State) {
    tracing::debug!("killing processes");

    let procs = state.procs.into_values();
    for (_, _, kill_tx) in procs {
        let _ = kill_tx.send(());
    }
}

#[cfg(test)]
mod tests {

    use crate::cli::Config;
    use bytes::Bytes;
    use clap::Parser;
    use std::collections::{HashMap, HashSet};
    use tokio::sync::{
        self, broadcast,
        mpsc::{self, UnboundedReceiver},
        oneshot,
    };
    use warp::Filter;

    use super::{
        attach, disconnect, Env, Event, FromProcessTx, PortPool, ShutdownTx, State, ToProcessTx,
    };

    fn create_config(args: &'static str) -> Config {
        Config::parse_from(args.split_whitespace())
    }

    fn create_process_handle() -> (FromProcessTx, ToProcessTx, ShutdownTx) {
        create_process().1
    }

    fn create_process() -> (
        UnboundedReceiver<Bytes>,
        (FromProcessTx, ToProcessTx, ShutdownTx),
    ) {
        let (tx, rx) = mpsc::unbounded_channel();
        let (broadcast_tx, _) = broadcast::channel(16);
        let (kill_tx, _) = oneshot::channel();
        (rx, (broadcast_tx, tx, kill_tx))
    }

    async fn create_ws() -> warp::ws::WebSocket {
        // Use channel to move Websocket out of closure (but could use Arc<Mutex>>)
        let (tx, mut rx) = sync::mpsc::unbounded_channel();

        // Dummy route that completes after hanshake
        let route = warp::ws().map(move |websocket: warp::ws::Ws| {
            let tx = tx.clone();
            websocket.on_upgrade(move |ws| {
                tx.send(ws).ok();
                futures::future::ready(())
            })
        });

        let _ = warp::test::ws().handshake(route).await.expect("handshake");
        let ws = rx.recv().await.unwrap();

        ws
    }

    #[tokio::test]
    async fn test_attach() {
        let (mut rx, handle) = create_process();
        let mut state = State {
            conns: HashMap::new(),
            procs: HashMap::from([("room1".to_string(), handle)]),
            cfg: create_config("scalesocket cat --joinmsg=foo"),
            ports: PortPool::new(),
        };
        let (tx, _) = sync::mpsc::unbounded_channel::<Event>();
        let ws = create_ws().await;

        attach(
            "room1".to_string(),
            Env::default(),
            Box::new(ws),
            &tx,
            &mut state,
            None,
        );

        let _ = rx.recv().await;
    }

    #[tokio::test]
    async fn test_attach_sends_joinmsg() {
        let (mut rx, handle) = create_process();
        let mut state = State {
            conns: HashMap::new(),
            procs: HashMap::from([("room1".to_string(), handle)]),
            cfg: create_config("scalesocket cat --joinmsg=foo"),
            ports: PortPool::new(),
        };
        let (tx, _) = sync::mpsc::unbounded_channel::<Event>();
        let ws = create_ws().await;

        attach(
            "room1".to_string(),
            Env::default(),
            Box::new(ws),
            &tx,
            &mut state,
            None,
        );

        let received_event = rx.recv().await.unwrap();
        let received_msg = std::str::from_utf8(&received_event).unwrap();
        assert_eq!("foo", received_msg);
    }

    #[tokio::test]
    async fn test_disconnect() {
        let mut state = State {
            conns: HashMap::from([
                ("room1".to_string(), HashSet::from([1])),
                ("room2".to_string(), HashSet::from([2])),
            ]),
            procs: HashMap::from([("room1".to_string(), create_process_handle())]),
            cfg: create_config("scalesocket cat"),
            ports: PortPool::new(),
        };

        disconnect("room1".to_string(), Env::default(), 1, &mut state);

        assert!(state.conns.get("room1").unwrap().is_empty());
        assert!(!state.conns.get("room2").unwrap().is_empty());
    }
}
