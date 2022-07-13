mod cli;
mod logging;

use {
    clap::Parser,
    cli::Config,
    futures::FutureExt,
    serde_json::json,
    warp::{self, Filter, Rejection, Reply},
};

fn main() {
    let config = Config::parse();

    let handle_routes = warp::serve(health_route()).run(config.addr).unit_error();

    tracing::info! { "listening" };
}

pub fn health_route() -> impl Filter<Extract = impl Reply, Error = Rejection> + Clone {
    warp::path("health")
        .and(warp::path::end())
        .and(warp::get())
        .map(|| warp::reply::json(&json!({"status" : "ok"})))
}
