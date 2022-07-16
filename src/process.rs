use crate::{
    cli::Config,
    error::AppResult,
    types::{FromProcessTx, ToProcessRx, ToProcessTx},
    utils::run,
};
use {
    tokio::process::{Child, Command},
    tokio::sync::{broadcast, mpsc},
};

pub async fn handle(mut process: Process) -> AppResult<()> {
    // TODO spawn and handle process
    Ok(())
}

#[derive(Debug)]
pub struct Process {
    source: Option<Source>,
    pub tx: ToProcessTx,
    pub rx: Option<ToProcessRx>,
    pub broadcast_tx: FromProcessTx,
}

#[derive(Debug)]
pub enum Source {
    Stdio(Command),
}

impl Process {
    pub fn new(config: &Config) -> Self {
        let (tx, rx) = mpsc::unbounded_channel::<String>();
        let (broadcast_tx, _) = broadcast::channel::<String>(16);

        let cmd = run(&config.cmd, &config.args);
        let source = Some(Source::Stdio(cmd));

        Self {
            source,
            tx,
            rx: Some(rx),
            broadcast_tx,
        }
    }
}
