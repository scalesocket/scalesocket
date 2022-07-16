use crate::{
    error::AppError,
    types::{FromProcessRx, ToProcessTx},
};
use warp::ws::WebSocket;

pub async fn handle(
    ws: WebSocket,
    rx_proc: FromProcessRx,
    tx_proc: ToProcessTx,
) -> Result<(), AppError> {
    // TODO handle connection

    Ok(())
}
