use futures::FutureExt;
use tokio::signal::unix::{signal, SignalKind};

use crate::types::{Event, EventTx, ShutdownTx};

pub async fn handle(routes_shutdown_tx: ShutdownTx, events_shutdown_tx: EventTx) {
    let mut interrupt = signal(SignalKind::interrupt()).expect("failed to create signal");
    let mut terminate = signal(SignalKind::terminate()).expect("failed to create signal");
    let signals = futures::future::select(interrupt.recv().boxed(), terminate.recv().boxed());

    signals.await;

    tracing::info!("received signal, shutting down");
    let _ = routes_shutdown_tx.send(());
    let _ = events_shutdown_tx.send(Event::Shutdown);
}
