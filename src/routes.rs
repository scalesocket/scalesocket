use crate::{
    cli::Config,
    types::{CGIEnv, Event, EventTx, RoomID, ShutdownRx},
    utils::warpext,
};

use {
    futures::FutureExt,
    prometheus_client::encoding::text::encode,
    prometheus_client::registry::Registry,
    serde_json::json,
    std::path::PathBuf,
    warp::ws::Ws,
    warp::{self, http::Response, Filter, Rejection, Reply},
};

pub fn handle(
    tx: EventTx,
    config: Config,
    shutdown_rx: ShutdownRx,
    registry: Option<Registry>,
) -> impl futures::Future<Output = ()> {
    let shutdown_rx = shutdown_rx.map(|_| ());

    warp::serve(
        socket(tx)
            .or(health())
            .or(metrics(registry))
            .or(files(config.staticdir.clone())),
    )
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

pub fn metrics(
    registry: Option<Registry>,
) -> impl Filter<Extract = impl Reply, Error = Rejection> + Clone {
    let registry = std::sync::Arc::new(registry);
    warpext::enable_if(registry.is_some())
        .and(warp::path("metrics"))
        .and(warp::path::end())
        .and(warp::get())
        .map(move || {
            // Encode metrics
            let mut buffer = Vec::new();
            let res = match *registry {
                Some(ref registry) => encode(&mut buffer, &registry),
                // Unreachable, since registry.is_some()
                None => unreachable!(),
            };

            let encoded = match res.is_ok() {
                true => String::from_utf8(buffer).ok(),
                false => None,
            };

            let builder = Response::builder().header(
                "content-type",
                "application/openmetrics-text; version=1.0.0; charset=utf-8",
            );
            match encoded {
                Some(data) => builder.body(data),
                None => builder.status(500).body(String::default()),
            }
        })
}

pub fn files(
    path: Option<PathBuf>,
) -> impl Filter<Extract = impl Reply, Error = Rejection> + Clone {
    warpext::enable_if(path.is_some()).and(warp::fs::dir(path.unwrap_or_default()))
}

#[cfg(test)]
mod tests {
    use prometheus_client::metrics::counter::Counter;
    use prometheus_client::metrics::family::Family;
    use warp::http::StatusCode;
    use warp::test::request;

    use super::*;

    #[tokio::test]
    async fn health_returns_ok() {
        let api = health();

        let resp = request().method("GET").path("/health").reply(&api).await;

        assert_eq!(resp.status(), StatusCode::OK);
        assert_eq!(resp.body(), "{\"status\":\"ok\"}");
    }

    #[tokio::test]
    async fn metrics_returns_metrics() {
        let mut registry = <Registry>::default();
        registry.register(
            "example_metric",
            "Example description",
            Box::new(Family::<(), Counter>::default()),
        );
        let api = metrics(Some(registry));

        let resp = request().method("GET").path("/metrics").reply(&api).await;

        assert_eq!(resp.status(), StatusCode::OK);
        assert_eq!(
            resp.body(),
            "# HELP example_metric Example description.\n# TYPE example_metric counter\n# EOF\n"
        );
    }
}
