use crate::{
    cli::Config,
    connection,
    error::AppResult,
    process::{self, Process},
    types::{
        ConnID, Event, EventRx, EventTx, FromProcessTx, PortID, RoomID, ShutdownTx, ToProcessTx,
    },
    utils::new_conn_id,
};

use {
    futures::{FutureExt, TryFutureExt},
    id_pool::IdPool as PortPool,
    std::collections::{HashMap, HashSet},
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

pub async fn handle(mut rx: EventRx, tx: EventTx, config: Config) -> AppResult<()> {
    let mut state = State {
        conns: HashMap::new(),
        procs: HashMap::new(),
        ports: PortPool::new_ranged(config.tcpports.clone()),
        cfg: config,
    };

    while let Some(event) = rx.recv().await {
        match event {
            Event::Connect { room, ws } if state.procs.contains_key(&room) => {
                attach(room, ws, &tx, &mut state);
            }
            Event::Connect { room, ws } => {
                spawn(&room, &tx, &mut state);
                // TODO handle spawn error if spawn.is_ok() else tracing::error
                attach(room, ws, &tx, &mut state);
            }
            Event::Disconnect { room, conn } => {
                disconnect(room, conn, &mut state);
            }
            Event::ProcessExit { room, code, port } => {
                exit(room, code, port, &mut state);
            }
            Event::Shutdown => {
                break;
            }
        }
    }
    Ok(())
}

#[instrument(name = "connection", skip(ws, tx, state))]
fn attach(room: RoomID, ws: Box<WebSocket>, tx: &EventTx, state: &mut State) {
    let conn = new_conn_id();

    // Get process handles from map
    let (proc_tx_broadcast, proc_tx, _) = state.procs.get(&room).expect("room not in process map");
    let proc_rx = proc_tx_broadcast.subscribe();

    let mut on_connect = || {
        // Successfully spawned, store connection handle in map
        let is_inserted = state
            .conns
            .entry(room.to_string())
            .or_default()
            .insert(conn);

        if is_inserted {
            tracing::info! { id = conn, "client connected" };

            // Inform child
            if let Some(ref join_msg_template) = state.cfg.joinmsg {
                let join_msg = join_msg_template.replace("%ID", &conn.to_string());
                let _ = proc_tx.send(join_msg);
            }
        }
    };

    let on_disconnect = || {
        let tx = tx.clone();
        let room = room.clone();

        // Return callback for connection::handle
        async move |_| {
            tracing::debug! { id=conn, "client disconnecting" };
            let _ = tx.send(Event::Disconnect { room, conn });
        }
    };

    tokio::spawn(
        connection::handle(*ws, proc_rx, proc_tx.clone())
            .then({
                // NOTE: we invoke on_connect closure immediately...
                on_connect();
                // NOTE: ...and then invoke a closure returning the async callback closure
                on_disconnect()
            })
            .in_current_span(),
    );
}

#[instrument(name = "process", skip(tx, state))]
fn spawn(room: &RoomID, tx: &EventTx, state: &mut State) {
    let port = match state.cfg.tcp {
        true => state.ports.request_id(),
        false => None,
    };

    if let Some(port) = port {
        tracing::debug!("reserved port {}", port);
    }

    let mut proc = Process::new(&state.cfg, port);
    let proc_tx_broadcast = proc.broadcast_tx.clone();
    let proc_tx = proc.tx.clone();
    let kill_tx = proc.kill_tx.take().unwrap();

    let on_spawn = || {
        // Successfully spawned, store handles in map
        state
            .procs
            .insert(room.to_string(), (proc_tx_broadcast, proc_tx, kill_tx));
    };

    let on_kill = || {
        let tx = tx.clone();
        let room = room.to_string();

        // Return callback for process::handle
        move |code: Option<i32>| {
            tx.send(Event::ProcessExit { room, code, port })
                .expect("Failed to send ProcessExit event");
            Ok(())
        }
    };

    tokio::spawn(
        process::handle(proc)
            .map_ok_or_else(
                move |e| {
                    tracing::error!("{}", e);
                    Err(e)
                },
                {
                    // NOTE: we invoke on_spawn closure immediately...
                    on_spawn();
                    // NOTE: ...and then invoke a closure returning the callback closure
                    on_kill()
                },
            )
            .in_current_span(),
    );
}

#[instrument(name = "connection", skip(conn, state))]
fn disconnect(room: RoomID, conn: ConnID, state: &mut State) {
    let room_conns = state.conns.entry(room.clone()).or_default();

    // Get process handles from map
    let (_, proc_tx, _) = state.procs.get(&room).expect("room not in process map");

    let is_removed = room_conns.remove(&conn);

    if is_removed {
        tracing::info! { id = conn, "client disconnected" };

        // Inform child
        if let Some(ref leave_msg_template) = state.cfg.leavemsg {
            let leave_msg = leave_msg_template.replace("%ID", &conn.to_string());
            let _ = proc_tx.send(leave_msg);
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

#[instrument(name = "process", skip(code, port, state))]
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

#[cfg(test)]
mod tests {

    use crate::cli::Config;
    use clap::Parser;
    use std::collections::{HashMap, HashSet};
    use tokio::sync::{broadcast, mpsc, oneshot};

    use super::{disconnect, FromProcessTx, ShutdownTx, State, ToProcessTx, PortPool};

    fn create_config(args: &'static str) -> Config {
        Config::parse_from(args.split_whitespace())
    }

    fn create_process_handle() -> (FromProcessTx, ToProcessTx, ShutdownTx) {
        let (tx, _) = mpsc::unbounded_channel::<String>();
        let (broadcast_tx, _) = broadcast::channel::<String>(16);
        let (kill_tx, _) = oneshot::channel();
        (broadcast_tx, tx, kill_tx)
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
            ports: PortPool::new()
        };

        disconnect("room1".to_string(), 1, &mut state);

        assert!(state.conns.get("room1").unwrap().is_empty());
        assert!(!state.conns.get("room2").unwrap().is_empty());
    }
}
