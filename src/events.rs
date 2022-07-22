use crate::{
    cli::Config,
    connection,
    process::{self, Process},
    types::{ConnID, Event, EventRx, EventTx, FromProcessTx, RoomID, ShutdownTx, ToProcessTx},
    utils::new_conn_id,
};

use {
    futures::FutureExt,
    std::collections::{HashMap, HashSet},
    warp::ws::WebSocket,
};

type ConnectionMap = HashMap<RoomID, HashSet<ConnID>>;
type ProcessMap = HashMap<RoomID, (FromProcessTx, ToProcessTx, ShutdownTx)>;

pub async fn handle(mut rx: EventRx, tx: EventTx, config: Config) {
    let mut conns: ConnectionMap = HashMap::new();
    let mut procs: ProcessMap = HashMap::new();

    let handle_attach =
        |room: RoomID, ws: Box<WebSocket>, procs: &mut ProcessMap, conns: &mut ConnectionMap| {
            let conn = new_conn_id();
            // Get process handles from map
            let (proc_tx_broadcast, proc_tx, _) =
                procs.get(&room).expect("room not in process map");
            let proc_tx = proc_tx.clone();
            let proc_rx = proc_tx_broadcast.subscribe();

            tokio::spawn(connection::handle(*ws, proc_rx, proc_tx).then({
                conns.entry(room.to_string()).or_default().insert(conn);

                let tx = tx.clone();
                async move |_| {
                    let _ = tx.send(Event::Disconnect { room, conn });
                }
            }));
        };

    let handle_spawn = |room: &RoomID, procs: &mut ProcessMap| {
        let mut proc = Process::new(&config);
        let proc_tx = proc.tx.clone();
        let proc_tx_broadcast = proc.broadcast_tx.clone();
        let kill_tx = proc.kill_tx.take().unwrap();

        tokio::spawn(process::handle(proc).then({
            // Successfully spawned, store handles in map
            procs.insert(room.to_string(), (proc_tx_broadcast, proc_tx, kill_tx));

            let tx = tx.clone();
            let room = room.to_string();
            async move |_| {
                tx.send(Event::ProcessExit { room })
                    .expect("Failed to send ProcessExit event")
            }
        }));
    };

    let handle_disconnect =
        |room: RoomID, conn: ConnID, procs: &mut ProcessMap, conns: &mut ConnectionMap| {
            let room_conns = conns.entry(room.clone()).or_default();

            if room_conns.remove(&conn) {
                tracing::info! { id = conn, "client disconnected" };
                // TODO inform clients
            }

            if room_conns.is_empty() {
                tracing::info! { "client was last in room, killing process" };
                let kill_tx = procs.remove(&room).unwrap().2;
                kill_tx.send(()).expect("Failed to send kill signal");
            }
        };

    while let Some(event) = rx.recv().await {
        match event {
            Event::Connect { ws, room } => {
                handle_spawn(&room, &mut procs);
                handle_attach(room, ws, &mut procs, &mut conns);
            }
            Event::Disconnect { room, conn } => {
                handle_disconnect(room, conn, &mut procs, &mut conns);
            }
            Event::ProcessExit { room } => {
                // TODO
            }
            Event::Shutdown => {
                break;
            }
        }
    }
}
