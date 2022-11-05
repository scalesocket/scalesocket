use crate::{
    cli::Config,
    envvars::CGIEnv,
    error::{AppError, AppResult},
    message::{deserialize, Address},
    types::{Framing, FromProcessTx, PortID, ShutdownRx, ShutdownTx, ToProcessRx, ToProcessTx},
    utils::run,
};
use {
    bytes::Bytes,
    std::net::{SocketAddr, SocketAddrV4},
    tokio::process::{Child, Command as ProcessCommand},
    tokio::sync::{broadcast, mpsc, oneshot},
    warp::ws::Message,
};

#[derive(Debug)]
pub struct Channel {
    pub source: Option<Source>,
    pub is_binary: bool,
    pub attach_delay: Option<u64>,
    pub framing: Option<Framing>,
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

impl Channel {
    pub fn new(config: &Config, port: Option<PortID>, env: CGIEnv) -> Self {
        let (tx, rx) = mpsc::unbounded_channel::<Message>();
        let (cast_tx, _) = broadcast::channel(16);
        let (kill_tx, kill_rx) = oneshot::channel();

        let cmd = run(&config.cmd, &config.args, port, env.into(), &config.passenv);
        let source = match &config.tcp {
            true => {
                let addr = SocketAddrV4::new("127.0.0.1".parse().unwrap(), port.unwrap()).into();
                Some(Source::Tcp(Command::new(cmd), addr))
            }
            false => Some(Source::Stdio(Command::new(cmd))),
        };

        Self {
            source,
            is_binary: config.binary,
            attach_delay: config.cmd_attach_delay,
            framing: config.frame,
            tx,
            rx: Some(rx),
            cast_tx,
            kill_tx: Some(kill_tx),
            kill_rx: Some(kill_rx),
        }
    }

    pub fn take_senders(&mut self) -> (FromProcessTx, ToProcessTx, ShutdownTx) {
        let proc_tx_broadcast = self.cast_tx.clone();
        let proc_tx = self.tx.clone();
        let kill_tx = self.kill_tx.take().unwrap();
        (proc_tx_broadcast, proc_tx, kill_tx)
    }

    pub fn write_sock(&mut self, msg: Bytes) {
        if let Ok((id, payload)) = deserialize(&msg, self.framing) {
            if self.is_binary {
                let _ = self.cast_tx.send(Message::binary(payload).to_some(id));
            } else {
                let msg = std::str::from_utf8(payload).unwrap_or_default();
                let _ = self.cast_tx.send(Message::text(msg).to_some(id));
            };
        }
    }
}

#[derive(Debug)]
pub struct Command(ProcessCommand);

impl Command {
    pub fn new(cmd: ProcessCommand) -> Self {
        Self(cmd)
    }

    pub fn spawn(mut self) -> AppResult<Child> {
        self.0
            .spawn()
            .map_err(|e| AppError::ProcessSpawnError(e.to_string()))
    }
}
