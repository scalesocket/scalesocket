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
};

#[tokio::main]
async fn main() {
    let config = Config::parse();

    setup_logging(&config);

    let (tx, rx) = sync::mpsc::unbounded_channel::<Event>();
    let (routes_shutdown_tx, routes_shutdown_rx) = sync::oneshot::channel();
    let events_shutdown_tx = tx.clone();

    tracing::info! { "listening at {}", config.addr };

    let handle_events = events::handle(rx, tx.clone(), config.clone()).unit_error();
    let handle_routes = routes::handle(tx, config, routes_shutdown_rx).unit_error();
    let handle_signal = async {
        let mut interrupt = signal(SignalKind::interrupt()).expect("failed to create signal");
        let mut terminate = signal(SignalKind::terminate()).expect("failed to create signal");
        let signals = futures::future::select(interrupt.recv().boxed(), terminate.recv().boxed());

        signals.await;

        tracing::info! { "received signal, shutting down" };
        routes_shutdown_tx.send(()).ok();
        events_shutdown_tx.send(Event::Shutdown).ok();
    }
    .unit_error();

    let _ = try_join!(handle_events, handle_routes, handle_signal);
}

mod routes {

    use futures::FutureExt;

    use crate::{
        cli::Config,
        types::{Event, EventTx, RoomID, ShutdownRx},
    };
    use {
        serde_json::json,
        std::path::PathBuf,
        warp::ws::Ws,
        warp::{self, Filter, Rejection, Reply},
    };

    pub fn handle(
        tx: EventTx,
        config: Config,
        shutdown_rx: ShutdownRx,
    ) -> impl futures::Future<Output = ()> {
        let shutdown_rx = shutdown_rx.map(|_| ());

        warp::serve(socket(tx).or(health()).or(files(config.staticdir.clone())))
            .bind_with_graceful_shutdown(config.addr, shutdown_rx)
            .1
    }

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
    use clap::Parser;
    use futures::{FutureExt, StreamExt};
    use tokio::time::{sleep, Duration};
    use warp::test::request;
    use warp::{http::StatusCode, test::WsClient};

    use super::routes;
    use crate::cli::Config;
    use crate::events;
    use crate::types::{Event, EventTx};

    struct Client {
        inner: WsClient,
    }

    impl Client {
        pub async fn connect(path: &'static str, tx: EventTx) -> Self {
            let api = routes::socket(tx);
            let client = warp::test::ws()
                .path(path)
                .handshake(api)
                .await
                .expect("handshake");

            Self { inner: client }
        }

        pub async fn send(&mut self, text: &'static str) -> &Self {
            self.inner.send_text(text).await;
            self
        }

        pub async fn recv(&mut self) -> Result<String, ()> {
            let res = self.inner.recv().await;
            match res {
                Ok(msg) => Ok(msg.to_str().unwrap_or_default().to_owned()),
                Err(_) => Err(()),
            }
        }
    }

    fn create_config(args: &'static str) -> Config {
        Config::parse_from(args.split_whitespace())
    }

    #[tokio::test]
    async fn health_returns_ok() {
        let api = routes::health();

        let resp = request().method("GET").path("/health").reply(&api).await;

        assert_eq!(resp.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn socket_connect_event() {
        let (tx, mut rx) = tokio::sync::mpsc::unbounded_channel::<Event>();
        Client::connect("/example", tx).await.send("hello").await;

        let received_event = rx.recv().await.unwrap();
        let room = match received_event {
            Event::Connect { room, .. } => Some(room),
            _ => None,
        };

        assert_eq!(Some("example".to_string()), room);
    }

    #[tokio::test]
    async fn socket_e2e_echo() {
        let (tx, rx) = tokio::sync::mpsc::unbounded_channel::<Event>();
        let config = create_config("scalesocket echo -- hello");
        let mut received_messages: Vec<String> = Vec::new();
        let mut client = Client::connect("/example", tx.clone()).await;

        let inspect = async {
            // TODO figure out easier way to inspect stream
            let mut stream = Box::pin(client.recv().into_stream());
            while let Some(msg) = stream.next().await {
                received_messages.push(msg.unwrap_or_default());
            }
            Ok(())
        };
        let shutdown = async {
            sleep(Duration::from_millis(250)).await;
            tx.send(Event::Shutdown).ok();
            Ok(())
        };
        let handle = events::handle(rx, tx.clone(), config);

        let _ = tokio::try_join!(handle, shutdown, inspect);
        assert_eq!(received_messages, vec!["hello"]);
    }
}
