use crate::{
    cli::Config,
    error::{AppError, AppResult},
    types::{
        FromProcessTx, PortID, ShutdownRx, ShutdownRxStream, ShutdownTx, ToProcessRx,
        ToProcessRxStream, ToProcessTx,
    },
    utils::{exit_code, run},
};
use {
    bytes::Bytes,
    futures::TryStreamExt,
    futures::{FutureExt, StreamExt},
    std::io::Result as IOResult,
    std::net::SocketAddr,
    std::net::SocketAddrV4,
    tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader},
    tokio::net::TcpStream,
    tokio::process::{Child, Command},
    tokio::sync::{broadcast, mpsc, oneshot},
    tokio::time::{sleep, Duration},
    tokio_stream::wrappers::{LinesStream, UnboundedReceiverStream},
    tokio_util::codec::{BytesCodec, FramedRead},
    warp::ws::Message,
};

pub async fn handle(mut process: Process) -> AppResult<Option<i32>> {
    let mut proc = spawn(&mut process).await?;
    let mut child = proc.child.take().unwrap();

    tracing::debug! { "process handler listening to child" };
    let exit_code = loop {
        tokio::select! {
            Some(v) = proc.sock_rx.next() => {
                if process.is_binary {
                    proc.proc_tx.write_all(&v).await?;
                } else {
                    proc.proc_tx.write_all(&[&v[..], b"\n"].concat()).await?;
                };
            }
            Some(v) = proc.proc_rx.next() => {
                if let Ok(msg) = v {
                    if process.is_binary {
                        let _ = process.cast_tx.send(Message::binary(msg));
                    } else {
                        let msg = std::str::from_utf8(&msg).unwrap_or_default();
                        let _ = process.cast_tx.send(Message::text(msg));
                    };
                }
            }
            _ = proc.kill_rx.next() => {
                // TODO propagate signal to child
                break None;
            }
            status = child.wait() => {
                break exit_code(status);
            },
        }
    };
    tracing::debug! { "process handler done" };
    Ok(exit_code)
}

async fn spawn(process: &mut Process) -> AppResult<RunningProcess> {
    let kill_rx = process.kill_rx.take().unwrap().into_stream();
    let sock_rx = UnboundedReceiverStream::new(process.rx.take().unwrap());

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

            let proc_rx: Box<dyn futures::Stream<Item = IOResult<Bytes>> + Unpin + Send> =
                match process.is_binary {
                    true => {
                        let buffer = BufReader::new(stdout);
                        let stream = FramedRead::new(buffer, BytesCodec::new());
                        // let stream = ReaderStream::new(buffer);
                        Box::new(stream.map_ok(Bytes::from))
                    }
                    false => {
                        let buffer = BufReader::new(stdout);
                        let stream = LinesStream::new(buffer.lines());
                        Box::new(stream.map_ok(Bytes::from))
                    }
                };

            Ok(RunningProcess {
                child: Some(child),
                sock_rx,
                proc_rx,
                proc_tx: Box::new(stdin),
                kill_rx,
            })
        }
        Source::Tcp(mut cmd, addr) => {
            let child = cmd.spawn()?;
            sleep(Duration::from_secs(1)).await;

            let stream = match TcpStream::connect(addr).await {
                Ok(s) => s,
                Err(e) => {
                    return Err(AppError::NetworkError(
                        addr.to_string(),
                        e.kind().to_string(),
                    ))
                }
            };

            tracing::debug!("connected to childprocess at {}", addr);

            let (rx, tx) = stream.into_split();
            let proc_rx: Box<dyn futures::Stream<Item = IOResult<Bytes>> + Unpin + Send> =
                match process.is_binary {
                    true => {
                        let buffer = BufReader::new(rx);
                        let stream = FramedRead::new(buffer, BytesCodec::new());
                        Box::new(stream.map_ok(Bytes::from))
                    }
                    false => {
                        let buffer = BufReader::new(rx);
                        let stream = LinesStream::new(buffer.lines());
                        Box::new(stream.map_ok(Bytes::from))
                    }
                };

            Ok(RunningProcess {
                child: Some(child),
                sock_rx,
                proc_tx: Box::new(tx),
                proc_rx,
                kill_rx,
            })
        }
    }
}

#[derive(Debug)]
pub struct Process {
    source: Option<Source>,
    is_binary: bool,
    pub tx: ToProcessTx,
    pub rx: Option<ToProcessRx>,
    pub cast_tx: FromProcessTx,
    pub kill_rx: Option<ShutdownRx>,
    pub kill_tx: Option<ShutdownTx>,
}

#[derive(Debug)]
pub enum Source {
    Stdio(Command),
    Tcp(Command, SocketAddr),
}

struct RunningProcess {
    child: Option<Child>,
    sock_rx: ToProcessRxStream,
    proc_rx: FromProcessRxAny,
    proc_tx: FromProcessTxAny,
    kill_rx: ShutdownRxStream,
}

type FromProcessTxAny = Box<dyn tokio::io::AsyncWrite + Unpin + Send>;
type FromProcessRxAny =
    Box<dyn futures::Stream<Item = Result<Bytes, std::io::Error>> + Unpin + Send>;

impl Process {
    pub fn new(config: &Config, port: Option<PortID>) -> Self {
        let (tx, rx) = mpsc::unbounded_channel::<Bytes>();
        let (cast_tx, _) = broadcast::channel(16);
        let (kill_tx, kill_rx) = oneshot::channel();

        let cmd = run(&config.cmd, &config.args, port);
        let source = match &config.tcp {
            true => {
                let addr = SocketAddrV4::new("127.0.0.1".parse().unwrap(), port.unwrap()).into();
                Some(Source::Tcp(cmd, addr))
            }
            false => Some(Source::Stdio(cmd)),
        };

        Self {
            source,
            is_binary: config.binary,
            tx,
            rx: Some(rx),
            cast_tx,
            kill_tx: Some(kill_tx),
            kill_rx: Some(kill_rx),
        }
    }
}

#[cfg(test)]
mod tests {

    use clap::Parser;

    use super::{handle, spawn, Message, Process};
    use crate::cli::Config;

    fn create_process(args: &'static str) -> Process {
        let config = Config::parse_from(args.split_whitespace());
        Process::new(&config, None)
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
        let mut proc_rx = process.cast_tx.subscribe();

        handle(process).await.ok();
        let output = proc_rx.recv().await.ok();

        assert_eq!(output, Some(Message::text("foo")));
    }

    #[tokio::test]
    async fn test_handle_process_input() {
        let process = create_process("scalesocket head -- -n 1");
        let mut proc_rx = process.cast_tx.subscribe();
        let sock_tx = process.tx.clone();

        let send = async {
            sock_tx.send("foo\n".into()).ok();
            Ok(())
        };
        let handle = handle(process);

        tokio::try_join!(handle, send).ok();
        let output = proc_rx.recv().await.ok();

        assert_eq!(output, Some(Message::text("foo")));
    }
}
