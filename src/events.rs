use crate::{
    cli::Config,
    types::{ConnID, Event, EventRx, EventTx, RoomID},
    utils::new_conn_id,
};

use {
    std::collections::{HashMap, HashSet},
    warp::ws::WebSocket,
};

type ConnectionMap = HashMap<RoomID, HashSet<ConnID>>;

pub async fn handle(mut rx: EventRx, tx: EventTx, config: Config) {
    let mut conns: ConnectionMap = HashMap::new();

    let handle_attach = |ws: Box<WebSocket>, room: RoomID, conns: &mut ConnectionMap| {
        let conn = new_conn_id();

        // TODO spawn process

        conns.entry(room).or_default().insert(conn);
    };

    while let Some(event) = rx.recv().await {
        match event {
            Event::Connect { ws, room } => {
                handle_attach(ws, room, &mut conns);
            }
        }
    }
}
