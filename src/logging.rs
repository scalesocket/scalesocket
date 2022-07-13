use crate::cli::Config;
use tracing_subscriber::{filter::LevelFilter, fmt::layer, prelude::*, Registry};

pub fn setup_logging(config: &Config) {
    let level = match config.verbosity {
        0 => LevelFilter::INFO,
        1 => LevelFilter::DEBUG,
        2 | _ => LevelFilter::TRACE,
    };
    let subscriber = Registry::default();

    // Tracing can only disable layers during runtime using Option<Layer>
    let (json_log, plain_log) = if config.json {
        let layer = layer().without_time().json();
        (Some(layer), None)
    } else {
        let layer = layer().pretty();
        (None, Some(layer))
    };

    let subscriber = subscriber
        .with(json_log.with_filter(level))
        .with(plain_log.with_filter(level));

    tracing::subscriber::set_global_default(subscriber).expect("Tracing initilized multiple times");
}
