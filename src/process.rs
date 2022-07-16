use crate::{
    cli::Config,
    error::AppResult,
    types::{FromProcessTx, ToProcessRx, ToProcessTx},
    utils::{exit_code, run},
};
use {
    tokio::process::{Child, Command},
    tokio::sync::{broadcast, mpsc},
};

pub async fn handle(mut process: Process) -> AppResult<()> {
    let mut proc = spawn(&mut process).await?;
    let mut child = proc.child.take().unwrap();

    tracing::debug! { "listening childprocess" };
    loop {
        tokio::select! {
            status = child.wait() => {
                tracing::error! { code=exit_code(status), "childprocess exited" };
                break;
            },
        }
    }
    tracing::debug! { "stopped listening childprocess" };
    Ok(())
}

async fn spawn(process: &mut Process) -> AppResult<RunningProcess> {
    match process.source.take().unwrap() {
        Source::Stdio(mut cmd) => {
            let mut child = cmd.spawn()?;
            tracing::debug!("spawned childprocess");

            Ok(RunningProcess { child: Some(child) })
        }
    }
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

struct RunningProcess {
    child: Option<Child>,
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
