use crate::{
    cli::Config,
    connection,
    error::AppResult,
    process::{self, Process},
    types::{ConnID, Event, EventRx, EventTx, FromProcessTx, RoomID, ShutdownTx, ToProcessTx},
    utils::new_conn_id,
};

use {
    futures::FutureExt,
    std::collections::{HashMap, HashSet},
    tracing::{instrument, Instrument},
    warp::ws::WebSocket,
};

type ConnectionMap = HashMap<RoomID, HashSet<ConnID>>;
type ProcessMap = HashMap<RoomID, (FromProcessTx, ToProcessTx, ShutdownTx)>;

struct State {
    pub conns: ConnectionMap,
    pub procs: ProcessMap,
    pub cfg: Config,
}

pub async fn handle(mut rx: EventRx, tx: EventTx, config: Config) -> AppResult<()> {
    let mut state = State {
        conns: HashMap::new(),
        procs: HashMap::new(),
        cfg: config,
    };

    while let Some(event) = rx.recv().await {
        match event {
            Event::Connect { room, ws } if state.procs.contains_key(&room) => {
                attach(room, ws, &tx, &mut state);
            }
            Event::Connect { room, ws } => {
                spawn(&room, &tx, &mut state);
                attach(room, ws, &tx, &mut state);
            }
            Event::Disconnect { room, conn } => {
                disconnect(room, conn, &mut state);
            }
            Event::ProcessExit { room, code } => {
                if state.procs.contains_key(&room) {
                    tracing::error! { room=room, code=code, "process exited" };
                    // TODO inform clients
                }
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
    tracing::info! { id=conn, "client connected" };

    // Get process handles from map
    let (proc_tx_broadcast, proc_tx, _) = state.procs.get(&room).expect("room not in process map");
    let proc_tx = proc_tx.clone();
    let proc_rx = proc_tx_broadcast.subscribe();

    tokio::spawn(
        connection::handle(*ws, proc_rx, proc_tx)
            .then({
                // Successfully spawned, store connection handle in map
                state
                    .conns
                    .entry(room.to_string())
                    .or_default()
                    .insert(conn);

                let tx = tx.clone();

                // Return callback for connection::handle
                async move |_| {
                    tracing::debug! { id=conn, "client disconnecting" };
                    let _ = tx.send(Event::Disconnect { room, conn });
                }
            })
            .in_current_span(),
    );
}

#[instrument(name = "process", skip(tx, state))]
fn spawn(room: &RoomID, tx: &EventTx, state: &mut State) {
    let mut proc = Process::new(&state.cfg);
    let proc_tx_broadcast = proc.broadcast_tx.clone();
    let proc_tx = proc.tx.clone();
    let kill_tx = proc.kill_tx.take().unwrap();

    tokio::spawn(
        process::handle(proc)
            .then({
                // Successfully spawned, store handles in map
                state
                    .procs
                    .insert(room.to_string(), (proc_tx_broadcast, proc_tx, kill_tx));

                let tx = tx.clone();
                let room = room.to_string();

                // Return callback for process::handle
                async move |result| {
                    tx.send(Event::ProcessExit {
                        room,
                        code: result.unwrap_or(None),
                    })
                    .expect("Failed to send ProcessExit event")
                }
            })
            .in_current_span(),
    );
}

#[instrument(name = "connection", skip(conn, state))]
fn disconnect(room: RoomID, conn: ConnID, state: &mut State) {
    let room_conns = state.conns.entry(room.clone()).or_default();
    let proc = state.procs.get(&room);

    if room_conns.remove(&conn) {
        tracing::info! { id = conn, "client disconnected" };
        // TODO inform clients
    }

    if room_conns.is_empty() {
        if let Some((_, _, kill_tx)) = state.procs.remove(&room) {
            if let Ok(_) = kill_tx.send(()) {
                // Only log if kill was sent
                tracing::info! { "all clients disconnected, killing process" };
            }
        }
    }
}

#[cfg(test)]
mod tests {

    use std::collections::{HashMap, HashSet};

    use super::{disconnect, State};

    #[tokio::test]
    async fn test_disconnect() {
        let mut state = State {
            conns: HashMap::from([
                ("room1".to_string(), HashSet::from([1])),
                ("room2".to_string(), HashSet::from([2])),
            ]),
            procs: HashMap::new(),
        };

        disconnect("room1".to_string(), 1, &mut state);

        assert!(state.conns.get("room1").unwrap().is_empty());
        assert!(!state.conns.get("room2").unwrap().is_empty());
    }
}
