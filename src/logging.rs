use tracing_subscriber::{filter::LevelFilter, fmt::layer, prelude::*, Registry};

use crate::cli::Config;

pub fn setup_logging(config: &Config) {
    let level = match config.verbosity {
        0 => LevelFilter::INFO,
        1 => LevelFilter::DEBUG,
        _ => LevelFilter::TRACE,
    };
    let subscriber = Registry::default();

    // Tracing can only disable layers during runtime using Option<Layer>
    let (json_log, plain_log) = if config.json {
        let layer = layer()
            .compact()
            .without_time()
            .with_target(false)
            .json()
            .with_span_list(false)
            .flatten_event(true);
        (Some(layer), None)
    } else {
        let layer = layer().compact().without_time().with_target(false);
        (None, Some(layer))
    };

    let subscriber = subscriber
        .with(json_log.with_filter(level))
        .with(plain_log.with_filter(level));

    tracing::subscriber::set_global_default(subscriber).expect("Tracing initilized multiple times");
}
