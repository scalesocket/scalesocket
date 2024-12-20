use {
    futures::{FutureExt, TryFutureExt},
    id_pool::IdPool as PortPool,
    std::collections::{HashMap, HashSet},
    std::sync::atomic::{AtomicU32, Ordering},
    std::sync::Arc,
    std::sync::Mutex,
    tokio::sync::Barrier,
    tracing::{instrument, Instrument},
    warp::ws::{Message, WebSocket},
};

use crate::{
    channel::Channel,
    cli::Config,
    connection,
    envvars::{replace_template_env, Env},
    error::AppResult,
    metrics::Metrics,
    process,
    types::{CacheBuffer, ConnID, Event, EventRx, EventTx, PortID, ProcessSenders, RoomID},
};

type ConnectionMap = HashMap<RoomID, HashSet<ConnID>>;
type ProcessMap = HashMap<RoomID, ProcessSenders>;
type ProcessCacheMap = HashMap<RoomID, Arc<Mutex<CacheBuffer>>>;

struct State {
    pub conns_next_id: AtomicU32,
    pub conns: ConnectionMap,
    pub cache: ProcessCacheMap,
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
) -> Result<(), ()> {
    let is_oneshot = config.oneshot;
    let mut state = State::new(config);

    while let Some(event) = rx.recv().await {
        match event {
            Event::Connect { room, ws, .. } if state.procs.contains_key(&room) && is_oneshot => {
                let _ = ws.close().await;
            }
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

                if is_oneshot {
                    break;
                }
            }
            Event::ProcessExit { room, code, port } => {
                metrics.clear(&room);
                exit(room, code, port, &mut state);

                if is_oneshot {
                    break;
                }
            }
            Event::ProcessMeta { room, value } => {
                metrics.set_metadata(&room, value);
            }
            Event::Shutdown => {
                break;
            }
        }
    }

    shutdown(state);

    // Stop upon event handler termination
    Err(())
}

impl State {
    pub fn new(cfg: Config) -> Self {
        Self {
            conns_next_id: AtomicU32::new(1),
            conns: HashMap::new(),
            procs: HashMap::new(),
            ports: PortPool::new_ranged(cfg.tcpports.clone()),
            cache: HashMap::new(),
            cfg,
        }
    }

    pub fn new_conn_id(&self) -> ConnID {
        self.conns_next_id.fetch_add(1, Ordering::Relaxed)
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
    let conn = state.new_conn_id();
    let framing = (&state.cfg).into();

    // Get process senders from map
    let (proc_tx_broadcast, proc_tx, _) = state.procs.get(&room).expect("room not in process map");
    let proc_rx = proc_tx_broadcast.subscribe();

    // Clone process cache from map for minimal mutex contention
    let cache = match state.cache.get(&room) {
        Some(shared) => shared.lock().expect("poisoned lock").to_vec(),
        None => Vec::new(),
    };

    let mut on_init = || {
        // Store connection handle in map
        let is_inserted = state
            .conns
            .entry(room.to_string())
            .or_default()
            .insert(conn);

        if is_inserted {
            tracing::info!(id = conn, "client connected");

            // Inform child
            if let Some(ref join_msg_template) = state.cfg.joinmsg {
                let join_msg = replace_template_env(join_msg_template, conn, &env);
                let _ = proc_tx.send(Message::text(join_msg));
            }
        }
    };

    let on_disconnect = || {
        let tx = tx.clone();
        let room = room.clone();
        let env = env.clone();

        // Return callback for connection::handle
        move |_| {
            tracing::debug!(id = conn, "client disconnecting");
            let _ = tx.send(Event::Disconnect { room, conn, env });
            futures::future::ready(())
        }
    };

    tokio::spawn(
        connection::handle(*ws, conn, framing, proc_rx, proc_tx.clone(), barrier, cache)
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
    room: &str,
    env: &Env,
    tx: &EventTx,
    state: &mut State,
    barrier: Option<Arc<Barrier>>,
) -> AppResult<()> {
    let port = state.ports.request_id();

    if let Some(port) = port {
        tracing::debug!("reserved port {}", port);
    }

    let cache = match state.cfg.cache {
        Some(ref c) => {
            state
                .cache
                .entry(room.to_string())
                .or_insert_with(|| Arc::new(Mutex::new(CacheBuffer::new(c))));
            state.cache.get(room).cloned()
        }
        None => None,
    };

    let mut proc = Channel::new(&state.cfg, port, room, env.cgi.clone(), cache);
    let senders = proc.take_senders();
    proc.give_sender(tx.clone());

    let on_init = || {
        // Store senders in map
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
        tracing::info!(id = conn, "client disconnected");

        // Inform child
        if let Some(ref leave_msg_template) = state.cfg.leavemsg {
            let leave_msg = replace_template_env(leave_msg_template, conn, &env);
            let _ = proc_tx.send(Message::text(leave_msg));
        }
    }

    if room_conns.is_empty() {
        if let Some((_, _, kill_tx)) = state.procs.remove(&room) {
            if kill_tx.send(()).is_ok() {
                // Only log if kill was sent
                tracing::info!("all clients disconnected, killing process");
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

    if !state.cfg.cache_persist {
        state.cache.remove(&room);
    }

    if state.procs.contains_key(&room) {
        tracing::error!(room, code, "process exited");
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

    use std::{
        collections::{HashMap, HashSet},
        sync::{atomic::AtomicU32, Arc, Mutex},
    };

    use clap::Parser;
    use tokio::sync::{
        self, broadcast,
        mpsc::{self},
        oneshot,
    };
    use warp::{filters::ws::Message, Filter};

    use super::{attach, disconnect, Env, Event, PortPool, State};
    use crate::{
        cli::Config,
        types::{Cache, CacheBuffer, ProcessSenders, ToProcessRx},
    };

    fn create_config(args: &'static str) -> Config {
        Config::parse_from(args.split_whitespace())
    }

    fn create_process_senders() -> ProcessSenders {
        create_process().1
    }

    fn create_process() -> (ToProcessRx, ProcessSenders) {
        let (proc_tx, proc_rx) = mpsc::unbounded_channel();
        let broadcast_tx = broadcast::Sender::new(16);
        let (kill_tx, _) = oneshot::channel();
        (proc_rx, (broadcast_tx, proc_tx, kill_tx))
    }

    fn create_process_with_cache() -> (ToProcessRx, ProcessSenders, CacheBuffer) {
        let (proc_tx, proc_rx) = mpsc::unbounded_channel();
        let broadcast_tx = broadcast::Sender::new(16);
        let (kill_tx, _) = oneshot::channel();
        let cache = CacheBuffer::new(&Cache::All(8));
        (proc_rx, (broadcast_tx, proc_tx, kill_tx), cache)
    }

    async fn create_ws() -> (warp::ws::WebSocket, warp::test::WsClient) {
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

        let wsc = warp::test::ws().handshake(route).await.expect("handshake");
        let ws = rx.recv().await.unwrap();

        (ws, wsc)
    }

    #[tokio::test]
    async fn test_attach() {
        let (mut proc_rx, senders) = create_process();
        let mut state = State {
            conns_next_id: AtomicU32::new(1),
            conns: HashMap::new(),
            procs: HashMap::from([("room1".to_string(), senders)]),
            cfg: create_config("scalesocket cat --joinmsg=foo"),
            ports: PortPool::new(),
            cache: HashMap::new(),
        };
        let (tx, _) = sync::mpsc::unbounded_channel::<Event>();
        let (ws, _) = create_ws().await;

        attach(
            "room1".to_string(),
            Env::default(),
            Box::new(ws),
            &tx,
            &mut state,
            None,
        );

        let _ = proc_rx.recv().await;
    }

    #[tokio::test]
    async fn test_attach_sends_joinmsg() {
        let (mut proc_rx, senders) = create_process();
        let mut state = State {
            conns_next_id: AtomicU32::new(1),
            conns: HashMap::new(),
            procs: HashMap::from([("room1".to_string(), senders)]),
            cfg: create_config("scalesocket cat --joinmsg=foo"),
            ports: PortPool::new(),
            cache: HashMap::new(),
        };
        let (tx, _) = sync::mpsc::unbounded_channel::<Event>();
        let (ws, _) = create_ws().await;

        attach(
            "room1".to_string(),
            Env::default(),
            Box::new(ws),
            &tx,
            &mut state,
            None,
        );

        let received_msg = proc_rx.recv().await.unwrap();
        let received_msg = std::str::from_utf8(&received_msg.as_bytes()).unwrap();
        assert_eq!("foo", received_msg);
    }

    #[tokio::test]
    async fn test_attach_sends_cache() {
        let (_proc_rx, senders, mut cache) = create_process_with_cache();

        cache.write(Message::text("foo"));
        cache.write(Message::text("bar"));

        let mut state = State {
            conns_next_id: AtomicU32::new(1),
            conns: HashMap::new(),
            procs: HashMap::from([("room1".to_string(), senders)]),
            cfg: create_config("scalesocket --cache=all:64 --joinmsg=baz cat"),
            ports: PortPool::new(),
            cache: HashMap::from([("room1".to_string(), Arc::new(Mutex::new(cache)))]),
        };
        let (tx, _) = sync::mpsc::unbounded_channel::<Event>();
        let (ws, mut wsc) = create_ws().await;

        attach(
            "room1".to_string(),
            Env::default(),
            Box::new(ws),
            &tx,
            &mut state,
            None,
        );

        assert_eq!(wsc.recv().await.unwrap(), Message::text("foo"));
        assert_eq!(wsc.recv().await.unwrap(), Message::text("bar"));
    }

    #[tokio::test]
    async fn test_disconnect() {
        let mut state = State {
            conns_next_id: AtomicU32::new(1),
            conns: HashMap::from([
                ("room1".to_string(), HashSet::from([1])),
                ("room2".to_string(), HashSet::from([2])),
            ]),
            procs: HashMap::from([("room1".to_string(), create_process_senders())]),
            cfg: create_config("scalesocket cat"),
            ports: PortPool::new(),
            cache: HashMap::new(),
        };

        disconnect("room1".to_string(), Env::default(), 1, &mut state);

        assert!(state.conns.get("room1").unwrap().is_empty());
        assert!(!state.conns.get("room2").unwrap().is_empty());
    }
}
