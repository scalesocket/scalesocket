use crate::{
    cli::Config,
    error::{AppError, AppResult},
    types::{FromProcessTx, ToProcessRx, ToProcessRxStream, ToProcessTx},
    utils::{exit_code, run},
};
use {
    futures::StreamExt,
    tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader},
    tokio::process::{Child, Command},
    tokio::sync::{broadcast, mpsc},
    tokio_stream::wrappers::{LinesStream, UnboundedReceiverStream},
};

pub async fn handle(mut process: Process) -> AppResult<()> {
    let mut proc = spawn(&mut process).await?;
    let mut child = proc.child.take().unwrap();

    tracing::debug! { "listening childprocess" };
    loop {
        tokio::select! {
            Some(v) = proc.rx_sock.next() => {
                proc.tx_proc.write_all(&[v.as_bytes(), b"\n"].concat()).await?;
            }
            Some(v) = proc.rx_proc.next() => {
                if let Ok(msg) = v {
                    let _ = process.broadcast_tx.send(msg);
                }
            }
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
            let stdin = child
                .stdin
                .take()
                .ok_or(AppError::ProcessStdIOError("stdin"))?;
            let stdout = child
                .stdout
                .take()
                .ok_or(AppError::ProcessStdIOError("stdout"))?;

            let rx_sock = UnboundedReceiverStream::new(process.rx.take().unwrap());
            let rx_proc = Box::new(LinesStream::new(BufReader::new(stdout).lines()));

            Ok(RunningProcess {
                child: Some(child),
                rx_sock,
                rx_proc,
                tx_proc: Box::new(stdin),
            })
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
    rx_sock: ToProcessRxStream,
    rx_proc: FromProcessRxAny,
    tx_proc: FromProcessTxAny,
}

type FromProcessTxAny = Box<dyn tokio::io::AsyncWrite + Unpin + Send>;
type FromProcessRxAny =
    Box<dyn futures::Stream<Item = Result<String, std::io::Error>> + Unpin + Send>;

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
