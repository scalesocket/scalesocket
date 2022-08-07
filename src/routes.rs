use crate::{
    cli::Config,
    types::{CGIEnv, Event, EventTx, RoomID, ShutdownRx},
    utils::warpext,
};
use {
    futures::FutureExt,
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
        .and(warpext::cgi_env())
        .map(move |room: RoomID, websocket: Ws, env: CGIEnv| {
            let tx = tx.clone();
            websocket.on_upgrade(move |ws| {
                let event = Event::Connect {
                    ws: Box::new(ws),
                    room,
                    env,
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
    warpext::enable_if(path.is_some()).and(warp::fs::dir(path.unwrap_or_default()))
}

#[cfg(test)]
mod tests {
    use warp::http::StatusCode;
    use warp::test::request;

    use super::*;

    #[tokio::test]
    async fn health_returns_ok() {
        let api = health();

        let resp = request().method("GET").path("/health").reply(&api).await;

        assert_eq!(resp.status(), StatusCode::OK);
    }
}
