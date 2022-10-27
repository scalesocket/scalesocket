use crate::{
    channel::{Channel, Source},
    error::{AppError, AppResult},
    types::{ShutdownRxStream, ToProcessRxStream},
    utils::exit_code,
};
use {
    bytes::Bytes,
    futures::TryStreamExt,
    futures::{FutureExt, StreamExt},
    std::io::Result as IOResult,
    std::sync::Arc,
    tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader},
    tokio::net::TcpStream,
    tokio::process::Child,
    tokio::sync::Barrier,
    tokio::time::{sleep, Duration},
    tokio_stream::wrappers::{LinesStream, UnboundedReceiverStream},
    tokio_util::codec::{BytesCodec, FramedRead},
    tracing::instrument,
    warp::ws::Message,
};

#[instrument(parent = None, name = "process", skip_all)]
pub async fn handle(mut channel: Channel, barrier: Option<Arc<Barrier>>) -> AppResult<Option<i32>> {
    if let Some(barrier) = barrier.clone() {
        barrier.wait().await;
        tracing::debug!("waited for connection");
    }
    let mut proc = spawn(&mut channel).await?;
    let mut child = proc.child.take().unwrap();

    tracing::debug! { "process handler listening to child" };

    let exit_code = loop {
        tokio::select! {
            Some(v) = proc.sock_rx.next() => {
                proc.write_child(v, channel.is_binary).await?;
            }
            Some(Ok(msg)) = proc.proc_rx.next() => {
                channel.write_sock(msg);
            },
            _ = proc.kill_rx.next() => {
                // TODO send SIGINT and wait
                let _ = child.kill().await;
                break None;
            }
            status = child.wait() => {
                break exit_code(status);
            },
        }
    };

    // Stream remaining messages
    while let Some(Ok(msg)) = proc.proc_rx.next().await {
        channel.write_sock(msg);
    }

    tracing::debug! { "process handler done" };
    Ok(exit_code)
}

async fn spawn(channel: &mut Channel) -> AppResult<RunningProcess> {
    let kill_rx = channel.kill_rx.take().unwrap().into_stream();
    let sock_rx = UnboundedReceiverStream::new(channel.rx.take().unwrap());

    match channel.source.take().unwrap() {
        Source::Stdio(cmd) => {
            let mut child = cmd.spawn()?;

            if let Some(pid) = child.id() {
                tracing::debug!("spawned childprocess with pid {}", pid);
            } else {
                tracing::debug!("spawned childprocess with unknown pid");
            }

            if let Some(attach_delay) = channel.attach_delay {
                tracing::debug!("delaying stdin attach for {} seconds", attach_delay);
                sleep(Duration::from_secs(attach_delay)).await;
            }

            let stdin = child
                .stdin
                .take()
                .ok_or(AppError::ProcessStdIOError("stdin"))?;
            let stdout = child
                .stdout
                .take()
                .ok_or(AppError::ProcessStdIOError("stdout"))?;

            let proc_rx: Box<dyn futures::Stream<Item = IOResult<Bytes>> + Unpin + Send> =
                match channel.is_binary {
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
        Source::Tcp(cmd, addr) => {
            let child = cmd.spawn()?;

            if let Some(attach_delay) = channel.attach_delay {
                tracing::debug!("delaying tcp connect for {} seconds", attach_delay);
                sleep(Duration::from_secs(attach_delay)).await;
            }

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
                match channel.is_binary {
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

struct RunningProcess {
    child: Option<Child>,
    sock_rx: ToProcessRxStream,
    proc_rx: FromProcessRxAny,
    proc_tx: FromProcessTxAny,
    kill_rx: ShutdownRxStream,
}

type FromProcessTxAny = Box<dyn tokio::io::AsyncWrite + Unpin + Send>;
type FromProcessRxAny = Box<dyn futures::Stream<Item = IOResult<Bytes>> + Unpin + Send>;

impl RunningProcess {
    pub async fn write_child(&mut self, msg: Message, is_binary: bool) -> IOResult<()> {
        if is_binary {
            self.proc_tx.write_all(&msg.as_bytes()).await?;
        } else {
            self.proc_tx
                .write_all(&[&msg.as_bytes()[..], b"\n"].concat())
                .await?;
        };
        Ok(())
    }
}

#[cfg(test)]
mod tests {

    use clap::Parser;
    use futures::StreamExt;
    use tokio_stream::wrappers::BroadcastStream;
    use warp::ws::Message;

    use super::{handle, spawn};
    use crate::{channel::Channel, cli::Config, envvars::CGIEnv};

    fn create_process(args: &'static str) -> Channel {
        let config = Config::parse_from(args.split_whitespace());
        Channel::new(&config, None, CGIEnv::default())
    }

    #[tokio::test]
    async fn test_spawn_process() {
        let mut process = create_process("scalesocket echo");

        let mut proc = spawn(&mut process).await.unwrap();
        let mut child = proc.child.take().unwrap();

        assert_eq!(child.wait().await.ok().unwrap().code(), Some(0));
    }

    #[tokio::test]
    async fn test_spawn_passes_cgi_env() {
        let process = create_process("scalesocket --passenv= printenv");
        let proc_rx = process.cast_tx.subscribe();

        handle(process, None).await.ok();
        let output = BroadcastStream::new(proc_rx)
            .filter_map(|d| async { d.ok() })
            .take(2)
            .collect::<Vec<_>>()
            .await;

        assert_eq!(
            output,
            vec![
                Message::text("QUERY_STRING="),
                Message::text("REMOTE_ADDR=")
            ]
        );
    }

    #[tokio::test]
    async fn test_handle_process_output_lines() {
        let process = create_process("scalesocket echo -- foo");
        let mut proc_rx = process.cast_tx.subscribe();

        handle(process, None).await.ok();
        let output = proc_rx.recv().await.ok();

        assert_eq!(output, Some(Message::text("foo")));
    }

    #[tokio::test]
    async fn test_handle_process_output_binary() {
        let process = create_process("scalesocket --binary echo");
        let mut proc_rx = process.cast_tx.subscribe();

        handle(process, None).await.ok();
        let output = proc_rx.recv().await.ok();

        assert_eq!(output, Some(Message::binary([10])));
    }

    #[tokio::test]
    async fn test_handle_process_input() {
        let process = create_process("scalesocket head -- -n 1");
        let mut proc_rx = process.cast_tx.subscribe();
        let sock_tx = process.tx.clone();

        let send = async {
            sock_tx.send(Message::text("foo\n")).ok();
            Ok(())
        };
        let handle = handle(process, None);

        tokio::try_join!(handle, send).ok();
        let output = proc_rx.recv().await.ok();

        assert_eq!(output, Some(Message::text("foo")));
    }
}
