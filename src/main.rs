mod cli;
mod logging;

use {
    clap::Parser,
    cli::Config,
    futures::FutureExt,
    serde_json::json,
    tokio::try_join,
    warp::ws::Ws,
    warp::{self, Filter, Rejection, Reply},
};

#[tokio::main]
async fn main() {
    let config = Config::parse();

    let handle_routes = warp::serve(socket_route().or(health_route()))
        .run(config.addr)
        .unit_error();

    tracing::info! { "listening" };

    let _ = try_join!(handle_routes);
}

pub fn socket_route() -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
    warp::path!(String)
        .and(warp::ws())
        .map(move |_, websocket: Ws| {
            websocket.on_upgrade(move |ws| {
                async { () }
            })
        })
}

pub fn health_route() -> impl Filter<Extract = impl Reply, Error = Rejection> + Clone {
    warp::path("health")
        .and(warp::path::end())
        .and(warp::get())
        .map(|| warp::reply::json(&json!({"status" : "ok"})))
}
