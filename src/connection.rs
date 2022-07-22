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

pub async fn handle(ws: WebSocket, proc_rx: FromProcessRx, proc_tx: ToProcessTx) -> AppResult<()> {
    let proc_rx = BroadcastStream::new(proc_rx);
    let proc_tx_sink = UnboundedSenderSink::from(proc_tx.clone());
    let (sock_tx, sock_rx) = ws.split();

    // forward process to socket
    let proc_to_sock = proc_rx
        .filter_map(|line| async { line.ok().map(|t| Ok(Message::text(t))).or(None) })
        .forward(sock_tx);

    // forward socket to process
    let sock_to_proc = sock_rx
        .filter_map(|msg| async { msg.ok().map(|m| Ok(m.to_str().unwrap_or("").to_string())) })
        .forward(proc_tx_sink);

    // exit in case receiver is dropped (process::handle exited)
    let proc_tx_closed = proc_tx.closed().map(|_| Err::<(), ()>(()));

    if let Err(e) = try_join!(
        proc_to_sock.map_err(|_| AppError::StreamError("process to socket")),
        sock_to_proc.map_err(|_| AppError::StreamError("socket to process")),
        proc_tx_closed.map_err(|_| AppError::ChannelError("process to socket"))
    ) {
        match e {
            AppError::StreamError(_) if proc_tx.is_closed() => {}
            AppError::StreamError(e) => tracing::debug!("Failed to stream {}", e),
            AppError::ChannelError(e) => tracing::error!("Channel from {} closed", e),
            _ => unreachable!(),
        }
    }
    tracing::debug! { "connection handler done" };

    Ok(())
}
