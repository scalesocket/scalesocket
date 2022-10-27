use crate::{
    error::{AppError, AppResult},
    message::encode,
    types::{ConnID, Framing, FromProcessRx, ToProcessTx},
};

use {
    futures::{future::ready, FutureExt, StreamExt, TryFutureExt, TryStreamExt},
    sender_sink::wrappers::{SinkError, UnboundedSenderSink},
    serde_json::Value,
    std::sync::Arc,
    tokio::sync::Barrier,
    tokio::try_join,
    tokio_stream::wrappers::BroadcastStream,
    tracing::instrument,
    warp::ws::{Message, WebSocket},
};

#[instrument(parent = None, name = "connection", skip_all)]
pub async fn handle(
    ws: WebSocket,
    id: ConnID,
    framing: Option<Framing>,
    proc_rx: FromProcessRx,
    proc_tx: ToProcessTx,
    barrier: Option<Arc<Barrier>>,
) -> AppResult<()> {
    let proc_rx = BroadcastStream::new(proc_rx);
    let (sock_tx, sock_rx) = ws.split();
    tracing::debug! { "connection handler listening to client" };

    // forward process to socket
    let proc_to_sock = proc_rx
        .filter_map(|line| ready(line.ok().map(Ok)))
        .forward(sock_tx);

    // forward socket to process, until closed
    let sock_to_proc = {
        let proc_tx_sink = UnboundedSenderSink::from(proc_tx.clone());
        async move {
            // forward until close message from client
            let result = sock_rx
                .try_take_while(|msg| ready(Ok(!msg.is_close())))
                .filter_map(|line| ready(line.ok()))
                .map(|msg| encode(msg, id, framing))
                .forward(proc_tx_sink)
                .await;

            Err::<(), AppError>(match result {
                Ok(_) => AppError::StreamClosed("client"),
                Err(_) => AppError::StreamError("socket to process"),
            })
        }
    };

    // exit in case receiver is dropped (process::handle exited)
    let proc_exit = proc_tx.closed().map(|_| Err::<(), ()>(()));

    // await barrier to let process::handle spawn child
    let proc_ready = async {
        match barrier {
            Some(barrier) => {
                barrier.wait().await;
                tracing::debug!("waited for process");
                Ok(())
            }
            None => Ok::<(), ()>(()),
        }
    };

    if let Err(e) = try_join!(
        sock_to_proc,
        proc_to_sock.map_err(|_| AppError::StreamError("process to socket")),
        proc_exit.map_err(|_| AppError::ChannelError("process to socket")),
        proc_ready.map_err(|_| AppError::StreamError("due to spawn failure")),
    ) {
        match e {
            AppError::StreamClosed(_) => {}
            AppError::StreamError(_) if proc_tx.is_closed() => {}
            AppError::StreamError(e) => tracing::debug!("failed to stream {}", e),
            AppError::ChannelError(e) => tracing::error!("channel from {} closed", e),
            _ => unreachable!(),
        }
    }
    tracing::debug! { id = id, "connection handler done" };

    Ok(())
}
