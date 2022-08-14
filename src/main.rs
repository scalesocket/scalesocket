#![feature(async_closure, bool_to_option)]
mod cli;
mod connection;
mod error;
mod events;
mod events_utils;
mod logging;
mod metrics;
mod process;
mod routes;
mod types;
mod utils;

use crate::{cli::Config, logging::setup_logging, metrics::Metrics, types::Event};

use {
    clap::Parser, futures::FutureExt, prometheus_client::registry::Registry, tokio::sync,
    tokio::try_join,
};

#[tokio::main]
async fn main() {
    let config = Config::parse();

    setup_logging(&config);

    let (tx, rx) = sync::mpsc::unbounded_channel::<Event>();
    let (routes_shutdown_tx, routes_shutdown_rx) = sync::oneshot::channel();
    let events_shutdown_tx = tx.clone();

    let mut registry = config.metrics.then_some(<Registry>::default());
    let mtr = Metrics::new(&mut registry);

    tracing::info! { "listening at {}", config.addr };

    let handle_events = events::handle(rx, tx.clone(), config.clone(), mtr.clone()).unit_error();
    let handle_routes = routes::handle(tx, config, routes_shutdown_rx, mtr, registry).unit_error();
    let handle_signal = signal::handle(routes_shutdown_tx, events_shutdown_tx).unit_error();

    let _ = try_join!(handle_events, handle_routes, handle_signal);
}

mod signal {
    use crate::types::{Event, EventTx, ShutdownTx};
    use futures::FutureExt;
    use tokio::signal::unix::{signal, SignalKind};

    pub async fn handle(routes_shutdown_tx: ShutdownTx, events_shutdown_tx: EventTx) {
        let mut interrupt = signal(SignalKind::interrupt()).expect("failed to create signal");
        let mut terminate = signal(SignalKind::terminate()).expect("failed to create signal");
        let signals = futures::future::select(interrupt.recv().boxed(), terminate.recv().boxed());

        signals.await;

        tracing::info! { "received signal, shutting down" };
        let _ = routes_shutdown_tx.send(());
        let _ = events_shutdown_tx.send(Event::Shutdown);
    }
}

#[cfg(test)]
mod tests {
    use clap::Parser;
    use futures::{FutureExt, StreamExt};
    use prometheus_client::registry::Registry;
    use tokio::time::{sleep, Duration};
    use warp::test::WsClient;

    use super::routes;
    use crate::cli::Config;
    use crate::events;
    use crate::metrics::Metrics;
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

    fn create_metrics() -> Metrics {
        Metrics::new(&mut Some(<Registry>::default()))
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
    async fn stdio_e2e_echo() {
        let (tx, rx) = tokio::sync::mpsc::unbounded_channel::<Event>();
        let config = create_config("scalesocket echo -- hello");
        let metrics = create_metrics();
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
        let handle = events::handle(rx, tx.clone(), config, metrics);

        let _ = tokio::try_join!(handle, shutdown, inspect);
        assert_eq!(received_messages, vec!["hello"]);
    }
}
