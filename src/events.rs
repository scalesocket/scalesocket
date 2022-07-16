use crate::{
    cli::Config,
    process::{self, Process},
    types::{ConnID, Event, EventRx, EventTx, RoomID},
    utils::new_conn_id,
};

use {
    futures::FutureExt,
    std::collections::{HashMap, HashSet},
    warp::ws::WebSocket,
};

type ConnectionMap = HashMap<RoomID, HashSet<ConnID>>;

pub async fn handle(mut rx: EventRx, tx: EventTx, config: Config) {
    let mut conns: ConnectionMap = HashMap::new();

    let handle_attach = |ws: Box<WebSocket>, room: RoomID, conns: &mut ConnectionMap| {
        let conn = new_conn_id();

        conns.entry(room).or_default().insert(conn);
    };

    let handle_spawn = |room: &RoomID| {
        let proc = Process::new(&config);

        tokio::spawn(process::handle(proc).then({
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
                handle_spawn(&room);
                handle_attach(ws, room, &mut conns);
            }
            Event::ProcessExit { room } => {
                // TODO
            }
        }
    }
}
