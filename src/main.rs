mod cli;
mod logging;
mod types;

use crate::{
    cli::Config,
    types::{Event, EventTx, RoomID},
};

use {
    clap::Parser,
    futures::FutureExt,
    serde_json::json,
    tokio::sync::mpsc,
    tokio::try_join,
    warp::ws::Ws,
    warp::{self, Filter, Rejection, Reply},
};

#[tokio::main]
async fn main() {
    let config = Config::parse();
    let (tx, rx) = mpsc::unbounded_channel::<Event>();

    let handle_routes = warp::serve(socket_route(tx).or(health_route()))
        .run(config.addr)
        .unit_error();

    tracing::info! { "listening" };

    let _ = try_join!(handle_routes);
}

pub fn socket_route(
    tx: EventTx,
) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
    warp::path!(String)
        .and(warp::path::end())
        .and(warp::ws())
        .map(move |room: RoomID, websocket: Ws| {
            let tx = tx.clone();
            websocket.on_upgrade(move |ws| {
                let event = Event::Connect {
                    ws: Box::new(ws),
                    room,
                };
                tx.send(event).expect("Failed to send Connect event");
                async { () }
            })
        })
}

pub fn health_route() -> impl Filter<Extract = impl Reply, Error = Rejection> + Clone {
    warp::path("health")
        .and(warp::path::end())
        .and(warp::get())
        .map(|| warp::reply::json(&json!({"status" : "ok"})))
}
