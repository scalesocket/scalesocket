use crate::{
    cli::Config,
    process::{self, Process},
    types::{ConnID, Event, EventRx, EventTx, FromProcessTx, RoomID, ToProcessTx},
    utils::new_conn_id,
};

use {
    futures::FutureExt,
    std::collections::{HashMap, HashSet},
    warp::ws::WebSocket,
};

type ConnectionMap = HashMap<RoomID, HashSet<ConnID>>;
type ProcessMap = HashMap<RoomID, (FromProcessTx, ToProcessTx)>;

pub async fn handle(mut rx: EventRx, tx: EventTx, config: Config) {
    let mut conns: ConnectionMap = HashMap::new();
    let mut procs: ProcessMap = HashMap::new();

    let handle_attach =
        |ws: Box<WebSocket>, room: RoomID, procs: &mut ProcessMap, conns: &mut ConnectionMap| {
            // Get process handles from map
            let (proc_tx_broadcast, proc_tx) = procs.get(&room).expect("room not in process map");
            let conn = new_conn_id();

            // TODO handle connection

            conns.entry(room).or_default().insert(conn);
        };

    let handle_spawn = |room: &RoomID, procs: &mut ProcessMap| {
        let proc = Process::new(&config);
        let proc_tx = proc.tx.clone();
        let proc_tx_broadcast = proc.broadcast_tx.clone();

        tokio::spawn(process::handle(proc).then({
            // Successfully spawned, store handles in map
            procs.insert(room.to_string(), (proc_tx_broadcast, proc_tx.clone()));

            let tx = tx.clone();
            let room = room.to_string();
            async move |_| {
                tx.send(Event::ProcessExit { room })
                    .expect("Failed to send ProcessExit event")
            }
        }));
    };

    while let Some(event) = rx.recv().await {
        match event {
            Event::Connect { ws, room } => {
                handle_spawn(&room, &mut procs);
                handle_attach(ws, room, &mut procs, &mut conns);
            }
            Event::ProcessExit { room } => {
                // TODO
            }
        }
    }
}
