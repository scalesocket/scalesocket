#![feature(async_closure)]
mod cli;
mod connection;
mod error;
mod events;
mod logging;
mod process;
mod types;
mod utils;

use crate::{cli::Config, logging::setup_logging, types::Event};

use {
    clap::Parser,
    futures::FutureExt,
    tokio::signal::unix::{signal, SignalKind},
    tokio::sync,
    tokio::try_join,
    warp::Filter,
};

#[tokio::main]
async fn main() {
    let config = Config::parse();

    setup_logging(&config);

    let (tx, rx) = sync::mpsc::unbounded_channel::<Event>();
    let (tx_routes_shutdown, rx_routes_shutdown) = sync::oneshot::channel();
    let rx_routes_shutdown = rx_routes_shutdown.map(|_| ());
    let tx_events_shutdown = tx.clone();

    let handle_routes = warp::serve(
        routes::socket(tx.clone())
            .or(routes::health())
            .or(routes::files(config.staticdir.clone())),
    )
    .bind_with_graceful_shutdown(config.addr, rx_routes_shutdown)
    .1
    .unit_error();

    let handle_events = events::handle(rx, tx, config).unit_error();

    let handle_signal = async {
        let mut interrupt = signal(SignalKind::interrupt()).expect("failed to create signal");
        let mut terminate = signal(SignalKind::terminate()).expect("failed to create signal");
        let signals = futures::future::select(interrupt.recv().boxed(), terminate.recv().boxed());

        signals.await;

        tracing::info! { "received signal, shutting down" };
        tx_routes_shutdown.send(()).ok();
        tx_events_shutdown.send(Event::Shutdown).ok();
    }
    .unit_error();

    tracing::info! { "listening" };

    let _ = try_join!(handle_events, handle_routes, handle_signal);
}

mod routes {

    use crate::types::{Event, EventTx, RoomID};
    use {
        serde_json::json,
        std::path::PathBuf,
        warp::ws::Ws,
        warp::{self, Filter, Rejection, Reply},
    };

    pub fn socket(tx: EventTx) -> impl Filter<Extract = impl Reply, Error = Rejection> + Clone {
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
                    async {}
                })
            })
    }

    pub fn health() -> impl Filter<Extract = impl Reply, Error = Rejection> + Clone {
        warp::path("health")
            .and(warp::path::end())
            .and(warp::get())
            .map(|| warp::reply::json(&json!({"status" : "ok"})))
    }

    pub fn files(
        path: Option<PathBuf>,
    ) -> impl Filter<Extract = impl Reply, Error = Rejection> + Clone {
        enable_if(path.is_some()).and(warp::fs::dir(path.unwrap_or_default()))
    }

    fn enable_if(condition: bool) -> impl Filter<Extract = (), Error = Rejection> + Copy {
        warp::any()
            .and_then(async move || {
                if condition {
                    Ok(())
                } else {
                    Err(warp::reject::not_found())
                }
            })
            // deal with Ok(())
            .untuple_one()
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
