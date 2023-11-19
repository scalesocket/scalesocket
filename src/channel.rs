use {
    bytes::Bytes,
    std::net::{SocketAddr, SocketAddrV4},
    std::sync::{Arc, Mutex},
    tokio::process::{Child, Command as ProcessCommand},
    tokio::sync::{broadcast, mpsc, oneshot},
    warp::ws::Message,
};

use crate::{
    cli::Config,
    envvars::CGIEnv,
    error::{AppError, AppResult},
    message::{deserialize, Address},
    types::{
        CacheBuffer, Caching, Event, EventTx, Framing, FromProcessTx, PortID, ProcessSenders,
        RoomID, ShutdownRx, ShutdownTx, ToProcessRx, ToProcessTx,
    },
    utils::run,
};

#[derive(Debug)]
pub struct Channel {
    pub source: Option<Source>,
    pub room: RoomID,
    pub is_binary: bool,
    pub attach_delay: Option<u64>,
    pub framing: Framing,
    pub caching: Caching,
    pub tx: ToProcessTx,
    pub rx: Option<ToProcessRx>,
    pub cast_tx: FromProcessTx,
    pub kill_rx: Option<ShutdownRx>,
    pub kill_tx: Option<ShutdownTx>,
    pub event_tx: Option<EventTx>,
    pub cache: Option<Arc<Mutex<CacheBuffer>>>,
}

#[derive(Debug)]
pub enum Source {
    Stdio(Command),
    Tcp(Command, SocketAddr),
}

impl Channel {
    pub fn new(
        config: &Config,
        port: Option<PortID>,
        room: &str,
        env: CGIEnv,
        cache: Option<Arc<Mutex<CacheBuffer>>>,
    ) -> Self {
        let (tx, rx) = mpsc::unbounded_channel::<Message>();
        let cast_tx = broadcast::Sender::new(16);
        let (kill_tx, kill_rx) = oneshot::channel();

        let cmd = run(
            &config.cmd,
            config.args.clone(),
            port,
            env.into(),
            &config.passenv,
        );
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
            room: room.to_string(),
            attach_delay: config.delay,
            framing: config.into(),
            caching: config.into(),
            tx,
            rx: Some(rx),
            cast_tx,
            kill_tx: Some(kill_tx),
            kill_rx: Some(kill_rx),
            event_tx: None,
            cache,
        }
    }

    pub fn take_senders(&mut self) -> ProcessSenders {
        let proc_tx_broadcast = self.cast_tx.clone();
        let proc_tx = self.tx.clone();
        let kill_tx = self.kill_tx.take().unwrap();
        (proc_tx_broadcast, proc_tx, kill_tx)
    }

    pub fn give_sender(&mut self, event_tx: EventTx) {
        self.event_tx = Some(event_tx);
    }

    /// Send a message to the socket clients (or event bus)
    pub fn write_sock(&mut self, msg: Bytes) {
        let write_metadata = |event_tx: Option<&EventTx>, room: &str, value: serde_json::Value| {
            let _ = event_tx
                .expect("event_tx to be passed")
                .send(Event::ProcessMeta {
                    room: room.to_string(),
                    value,
                });
        };
        let write_cache = |cache: Option<&Arc<Mutex<CacheBuffer>>>, msg: Message| {
            if let Some(cache) = cache {
                cache.lock().expect("poisoned lock").write(msg);
            }
        };

        match deserialize(&msg, self.framing.process_to_socket()) {
            Ok((h, _)) if h.is_meta && self.is_binary => {
                tracing::warn!("binary metadata is not supported");
            }
            Ok((h, msg)) if h.is_meta => {
                let value = serde_json::from_slice(msg).unwrap_or_default();
                write_metadata(self.event_tx.as_ref(), &self.room, value);
            }
            Ok((h, msg)) => {
                let msg = match self.is_binary {
                    true => Message::binary(msg),
                    false => Message::text(std::str::from_utf8(msg).unwrap_or_default()),
                };

                if self.caching.matches(&h) {
                    write_cache(self.cache.as_ref(), msg.clone());
                }

                let _ = self.cast_tx.send(msg.header(h));
            }
            Err(_) => {
                tracing::warn!(room = self.room, "error deserializing message from process")
            }
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
