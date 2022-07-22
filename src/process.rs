use crate::{
    cli::Config,
    error::{AppError, AppResult},
    types::{
        FromProcessTx, ShutdownRx, ShutdownRxStream, ShutdownTx, ToProcessRx, ToProcessRxStream,
        ToProcessTx,
    },
    utils::{exit_code, run},
};
use {
    futures::{FutureExt, StreamExt},
    tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader},
    tokio::process::{Child, Command},
    tokio::sync::{broadcast, mpsc, oneshot},
    tokio_stream::wrappers::{LinesStream, UnboundedReceiverStream},
};

pub async fn handle(mut process: Process) -> AppResult<()> {
    let mut proc = spawn(&mut process).await?;
    let mut child = proc.child.take().unwrap();

    tracing::debug! { "process handler listening to child" };
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
            _ = proc.kill_rx.next() => {
                // TODO propagate signal to child
                break;
            }
            status = child.wait() => {
                tracing::error! { code=exit_code(status), "childprocess exited" };
                break;
            },
        }
    }
    tracing::debug! { "process handler done" };
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
            let kill_rx = process.kill_rx.take().unwrap().into_stream();

            Ok(RunningProcess {
                child: Some(child),
                rx_sock,
                rx_proc,
                tx_proc: Box::new(stdin),
                kill_rx,
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
    pub kill_rx: Option<ShutdownRx>,
    pub kill_tx: Option<ShutdownTx>,
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
    kill_rx: ShutdownRxStream,
}

type FromProcessTxAny = Box<dyn tokio::io::AsyncWrite + Unpin + Send>;
type FromProcessRxAny =
    Box<dyn futures::Stream<Item = Result<String, std::io::Error>> + Unpin + Send>;

impl Process {
    pub fn new(config: &Config) -> Self {
        let (tx, rx) = mpsc::unbounded_channel::<String>();
        let (broadcast_tx, _) = broadcast::channel::<String>(16);
        let (kill_tx, kill_rx) = oneshot::channel();

        let cmd = run(&config.cmd, &config.args);
        let source = Some(Source::Stdio(cmd));

        Self {
            source,
            tx,
            rx: Some(rx),
            broadcast_tx,
            kill_tx: Some(kill_tx),
            kill_rx: Some(kill_rx),
        }
    }
}

#[cfg(test)]
mod tests {

    use clap::Parser;

    use super::{handle, spawn, Process};
    use crate::cli::Config;

    fn create_process(args: &'static str) -> Process {
        let config = Config::parse_from(args.split_whitespace());
        Process::new(&config)
    }

    #[tokio::test]
    async fn test_spawn_process() {
        let mut process = create_process("scalesocket echo");

        let mut proc = spawn(&mut process).await.unwrap();
        let mut child = proc.child.take().unwrap();

        assert_eq!(child.wait().await.ok().unwrap().code(), Some(0));
    }

    #[tokio::test]
    async fn test_handle_process_output() {
        let process = create_process("scalesocket echo -- foo");
        let mut proc_rx = process.broadcast_tx.subscribe();

        handle(process).await.ok();
        let output = proc_rx.recv().await.ok();

        assert_eq!(output, Some("foo".to_string()));
    }

    #[tokio::test]
    async fn test_handle_process_input() {
        let process = create_process("scalesocket head -- -n 1");
        let mut proc_rx = process.broadcast_tx.subscribe();
        let sock_tx = process.tx.clone();

        let send = async {
            sock_tx.send("foo\n".to_string()).ok();
            Ok(())
        };
        let handle = handle(process);

        tokio::try_join!(handle, send).ok();
        let output = proc_rx.recv().await.ok();

        assert_eq!(output, Some("foo".to_string()));
    }
}
