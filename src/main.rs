#![feature(async_closure)]
mod cli;
mod error;
mod events;
mod logging;
mod process;
mod types;
mod utils;

use crate::{
    cli::Config,
    logging::setup_logging,
    types::{Event, EventTx, RoomID},
};

use {clap::Parser, futures::FutureExt, tokio::sync::mpsc, tokio::try_join, warp::Filter};

#[tokio::main]
async fn main() {
    let config = Config::parse();

    setup_logging(&config);

    let (tx, rx) = mpsc::unbounded_channel::<Event>();

    let handle_routes = warp::serve(routes::socket(tx.clone()).or(routes::health()))
        .run(config.addr)
        .unit_error();
    let handle_events = events::handle(rx, tx, config).unit_error();

    tracing::info! { "listening" };

    let _ = try_join!(handle_events, handle_routes);
}

mod routes {
    use crate::types::{Event, EventTx, RoomID};
    use {
        serde_json::json,
        warp::ws::Ws,
        warp::{self, Filter, Rejection, Reply},
    };

    pub fn socket(
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

    pub fn health() -> impl Filter<Extract = impl Reply, Error = Rejection> + Clone {
        warp::path("health")
            .and(warp::path::end())
            .and(warp::get())
            .map(|| warp::reply::json(&json!({"status" : "ok"})))
    }
}

#[cfg(test)]
mod tests {
    use warp::http::StatusCode;
    use warp::test::request;

    use super::routes;

    #[tokio::test]
    async fn test_health() {
        let api = routes::health();

        let resp = request().method("GET").path("/health").reply(&api).await;

        assert_eq!(resp.status(), StatusCode::OK);
    }
}
