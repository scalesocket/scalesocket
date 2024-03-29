use {
    prometheus_client::encoding::text::encode,
    prometheus_client::registry::Registry,
    serde_json::json,
    std::path::PathBuf,
    warp::ws::Ws,
    warp::{self, http::Response, Filter, Rejection, Reply},
};

use crate::{
    cli::Config,
    envvars::Env,
    metrics::Metrics,
    types::{Event, EventTx, RoomID, ShutdownRx},
    utils::warpext::{self, handle_rejection},
};

const RESERVED_ROOMS: &[&str] = &[
    "api",
    "metrics",
    "health",
    "static",
    "upload",
    "robots.txt",
    "favicon.ico",
];

pub fn handle(
    tx: EventTx,
    config: Config,
    shutdown_rx: ShutdownRx,
    metrics: Metrics,
    registry: Option<Registry>,
) -> impl futures::Future<Output = ()> {
    let shutdown_rx = async {
        shutdown_rx.await.ok();
    };

    warp::serve(
        socket(tx)
            .or(health())
            .or(openmetrics(registry, config.metrics))
            .or(rooms_api(metrics.clone(), config.api))
            .or(metadata_api(metrics, config.api))
            .or(files(config.staticdir.clone()))
            .recover(handle_rejection),
    )
    .bind_with_graceful_shutdown(config.addr, shutdown_rx)
    .1
}

pub fn socket(tx: EventTx) -> impl Filter<Extract = (impl Reply,), Error = Rejection> + Clone {
    warpext::path::param_not::<RoomID>(RESERVED_ROOMS)
        .and(warp::path::end())
        .and(warp::ws())
        .and(warpext::env())
        .map(move |room: RoomID, websocket: Ws, mut env: Env| {
            let tx = tx.clone();
            env.set_room(&room);
            websocket.on_upgrade(move |ws| {
                let ws = Box::new(ws);
                let event = Event::Connect { env, room, ws };
                tx.send(event).expect("Failed to send Connect event");
                futures::future::ready(())
            })
        })
}

pub fn health() -> impl Filter<Extract = (impl Reply,), Error = Rejection> + Clone {
    warp::path("health")
        .and(warp::path::end())
        .and(warp::get())
        .map(|| warp::reply::json(&json!({"status" : "ok"})))
}

pub fn openmetrics(
    registry: Option<Registry>,
    enabled: bool,
) -> impl Filter<Extract = (impl Reply,), Error = Rejection> + Clone {
    let registry = std::sync::Arc::new(registry);

    warpext::enable_if(enabled)
        .and(warp::path("metrics"))
        .and(warp::path::end())
        .and(warp::get())
        .map(move || {
            // Encode metrics
            let mut buffer = String::new();
            let res = match *registry {
                Some(ref registry) => encode(&mut buffer, registry),
                None => unreachable!("Registry should always be Some"),
            };

            let encoded = match res.is_ok() {
                true => Some(buffer),
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

pub fn rooms_api(
    metrics: Metrics,
    enabled: bool,
) -> impl Filter<Extract = (impl Reply,), Error = Rejection> + Clone {
    warpext::enable_if(enabled)
        .and(warp::path!("api" / "rooms"))
        .and(warp::path::end())
        .and(warp::get())
        .map(move || warp::reply::json(&metrics.get_rooms()))
}

pub fn metadata_api(
    metrics: Metrics,
    enabled: bool,
) -> impl Filter<Extract = (impl Reply,), Error = Rejection> + Clone {
    let metric = warp::path::param::<String>()
        .map(Some)
        .or_else(|_| async { Ok::<(Option<String>,), std::convert::Infallible>((None,)) });

    warpext::enable_if(enabled)
        .and(warp::path!("api" / RoomID / ..))
        .and(metric)
        .and(warp::path::end())
        .and(warp::get())
        .map(
            move |room: RoomID, metric: Option<String>| match metric.as_deref() {
                Some("connections") => warp::reply::json(&metrics.get_room_connections(room)),
                Some("metadata") => warp::reply::json(&metrics.get_room_metadata(&room)),
                _ => warp::reply::json(&metrics.get_room(room)),
            },
        )
}

pub fn files(
    path: Option<PathBuf>,
) -> impl Filter<Extract = (impl Reply,), Error = Rejection> + Clone {
    warpext::enable_if(path.is_some()).and(warp::fs::dir(path.unwrap_or_default()))
}

#[cfg(test)]
mod tests {
    use prometheus_client::metrics::counter::Counter;
    use prometheus_client::metrics::family::Family;
    use serde_json::{self, Value};
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
    async fn socket_rejects_invalid_room() {
        let (tx, _) = tokio::sync::mpsc::unbounded_channel::<Event>();
        let api = socket(tx).recover(handle_rejection);

        let ws = |path: &'static str| {
            request()
                .method("GET")
                .header("Connection", "Upgrade")
                .header("Upgrade", "websocket")
                .header("Sec-WebSocket-Key", "SGVsbG8sIHdvcmxkIQ==")
                .header("Sec-WebSocket-Version", 13)
                .path(path)
                .reply(&api)
        };

        assert_eq!(ws("/ok").await.status(), StatusCode::SWITCHING_PROTOCOLS);
        assert_eq!(ws("/api").await.status(), StatusCode::BAD_REQUEST);
        assert_eq!(ws("/api/rooms").await.status(), StatusCode::BAD_REQUEST);
        assert_eq!(ws("/metrics").await.status(), StatusCode::BAD_REQUEST);
        assert_eq!(ws("/health").await.status(), StatusCode::BAD_REQUEST);
    }

    #[tokio::test]
    async fn metrics_returns_metrics() {
        let mut registry = <Registry>::default();
        registry.register(
            "example_metric",
            "Example description",
            Family::<(), Counter>::default(),
        );
        let api = openmetrics(Some(registry), true);

        let resp = request().method("GET").path("/metrics").reply(&api).await;

        assert!(resp.status().is_success());
        assert_eq!(
            resp.body(),
            "# HELP example_metric Example description.\n# TYPE example_metric counter\n# EOF\n"
        );
    }

    #[tokio::test]
    async fn room_api_returns_all_rooms() {
        let metrics = Metrics::new(&mut None, true);
        metrics.inc_ws_connections("foo");
        metrics.inc_ws_connections("bar");
        metrics.set_metadata("foo", json!({"_meta": true, "bar": 123}));

        let api = rooms_api(metrics, true);

        let resp = request()
            .method("GET")
            .path("/api/rooms/")
            .reply(&api)
            .await;

        assert!(resp.status().is_success());
        let body: Vec<Value> = serde_json::from_slice(resp.body()).unwrap();
        assert!(body
            .contains(&json!({"name": "foo", "connections": 1, "metadata": json!({"bar": 123})})));
        assert!(body.contains(&json!({"name": "bar", "connections": 1, "metadata": null})));
    }

    #[tokio::test]
    async fn metadata_api_returns_room_metadata() {
        let metrics = Metrics::new(&mut None, true);
        metrics.set_metadata("foo", json!({"_meta": true, "bar": 123}));

        let api = metadata_api(metrics, true);

        let resp = request().method("GET").path("/api/foo/").reply(&api).await;

        assert!(resp.status().is_success());
        let body: Value = serde_json::from_slice(resp.body()).unwrap();
        assert_eq!(body["metadata"], json!({"bar": 123}));
    }

    #[tokio::test]
    async fn metadata_api_returns_room_metric() {
        let metrics = Metrics::new(&mut None, true);
        metrics.inc_ws_connections("foo");
        metrics.inc_ws_connections("bar");

        let api = metadata_api(metrics, true);

        let resp = request()
            .method("GET")
            .path("/api/foo/connections")
            .reply(&api)
            .await;

        assert!(resp.status().is_success());
        assert_eq!(resp.body(), "1");
    }
}
