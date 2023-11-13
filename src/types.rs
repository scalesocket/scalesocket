use {
    bytes::Bytes,
    serde::Deserialize,
    std::io::Result as IOResult,
    tokio::sync::{broadcast, mpsc, oneshot},
    tokio_stream::wrappers::UnboundedReceiverStream,
    warp::ws::{Message, WebSocket},
};

use crate::{cli::Config, envvars::Env};

pub type RoomID = String;
pub type ConnID = u32;
pub type PortID = u16;

#[derive(Debug, Clone, Default, PartialEq, Deserialize)]
pub struct Header {
    #[serde(rename = "_to")]
    pub to: Option<ConnID>,
    #[serde(rename = "_meta", default = "bool::default")]
    pub is_meta: bool,
}

impl Header {
    pub fn to(to: ConnID) -> Self {
        Header {
            to: Some(to),
            is_meta: false,
        }
    }

    pub fn broadcast() -> Self {
        Header {
            to: None,
            is_meta: false,
        }
    }
}

#[derive(Debug)]
pub enum Event {
    Connect {
        env: Env,
        room: RoomID,
        ws: Box<WebSocket>,
    },
    Disconnect {
        env: Env,
        room: RoomID,
        conn: ConnID,
    },
    ProcessExit {
        room: RoomID,
        code: Option<i32>,
        port: Option<PortID>,
    },
    ProcessMeta {
        room: RoomID,
        value: serde_json::Value,
    },
    Shutdown,
}

/// Composite type for incoming and outgoing framing
#[derive(Debug, Clone, Copy)]
pub enum Framing {
    None,
    /// Common framing for incoming and outgoing messages
    Symmetric(Frame),
    /// Independent framing for incoming and outgoing messages
    Asymmetric(Option<Frame>, Option<Frame>),
}

impl Framing {
    pub fn socket_to_process(&self) -> Option<Frame> {
        match self {
            Framing::None => None,
            Framing::Symmetric(f) => Some(*f),
            Framing::Asymmetric(f, _) => *f,
        }
    }

    pub fn process_to_socket(&self) -> Option<Frame> {
        match self {
            Framing::None => None,
            Framing::Symmetric(f) => Some(*f),
            Framing::Asymmetric(_, f) => *f,
        }
    }
}

impl From<&Config> for Framing {
    fn from(cfg: &Config) -> Self {
        if cfg.server_frame.is_some() || cfg.client_frame.is_some() {
            return Self::Asymmetric(cfg.client_frame, cfg.server_frame);
        }
        if let Some(frame) = cfg.frame {
            return Self::Symmetric(frame);
        }
        Self::None
    }
}

#[derive(Debug, clap::ValueEnum, Clone, Copy)]
#[allow(clippy::upper_case_acronyms)]
pub enum Frame {
    JSON,
    Binary,
}

#[derive(Debug, clap::ValueEnum, Clone, Copy)]
#[allow(clippy::upper_case_acronyms)]
pub enum Log {
    JSON,
    Text,
}

// Channel for app events
pub type EventTx = mpsc::UnboundedSender<Event>;
pub type EventRx = mpsc::UnboundedReceiver<Event>;

// Channel for passing data to child process
pub type ToProcessTx = mpsc::UnboundedSender<Message>;
pub type ToProcessRx = mpsc::UnboundedReceiver<Message>;
pub type ToProcessRxStream = UnboundedReceiverStream<Message>;

// Channel for triggering shutdown of child process
pub type ShutdownTx = oneshot::Sender<()>;
pub type ShutdownRx = oneshot::Receiver<()>;
pub type ShutdownRxStream = futures::future::IntoStream<ShutdownRx>;

// Channel for passing data from child process
pub type FromProcessTx = broadcast::Sender<(Header, Message)>;
pub type FromProcessRx = broadcast::Receiver<(Header, Message)>;
pub type FromProcessTxAny = Box<dyn tokio::io::AsyncWrite + Unpin + Send>;
pub type FromProcessRxAny = Box<dyn futures::Stream<Item = IOResult<Bytes>> + Unpin + Send>;

pub type ProcessSenders = (FromProcessTx, ToProcessTx, ShutdownTx);
