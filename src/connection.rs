use crate::{
    error::{AppError, AppResult},
    types::{FromProcessRx, ToProcessTx},
};
use {
    futures::{FutureExt, StreamExt, TryFutureExt},
    sender_sink::wrappers::UnboundedSenderSink,
    std::convert::Infallible,
    tokio::try_join,
    tokio_stream::wrappers::BroadcastStream,
    warp::ws::{Message, WebSocket},
};

pub async fn handle(ws: WebSocket, rx_proc: FromProcessRx, tx_proc: ToProcessTx) -> AppResult<()> {
    let rx_proc = BroadcastStream::new(rx_proc);
    let tx_proc_sink = UnboundedSenderSink::from(tx_proc.clone());
    let (tx_sock, rx_sock) = ws.split();

    // forward process to socket
    let proc_to_sock = rx_proc
        .filter_map(|line| async { line.ok().map(|t| Ok(Message::text(t))).or(None) })
        .forward(tx_sock);

    // forward socket to process
    let sock_to_proc = rx_sock
        .filter_map(|msg| async { msg.ok().map(|m| Ok(m.to_str().unwrap_or("").to_string())) })
        .forward(tx_proc_sink);

    // exit in case receiver is dropped (process::handle exited)
    let rx_proc_closed = tx_proc.closed().map(|_| Ok::<(), Infallible>(()));

    if let Err(e) = try_join!(
        proc_to_sock.map_err(|_| AppError::StreamError("process to socket")),
        sock_to_proc.map_err(|_| AppError::StreamError("socket to process")),
        rx_proc_closed.map_err(AppError::from)
    ) {
        match e {
            AppError::StreamError(_) if tx_proc.is_closed() => {}
            AppError::StreamError(e) => tracing::error!("Failed to stream {}", e),
            _ => unreachable!(),
        }
    }
    tracing::debug! { "connection handler done" };

    Ok(())
}
