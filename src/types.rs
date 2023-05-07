use crate::{cli::Config, envvars::Env};
use {
    tokio::sync::{broadcast, mpsc, oneshot},
    tokio_stream::wrappers::UnboundedReceiverStream,
    warp::ws::{Message, WebSocket},
};

pub type RoomID = String;
pub type ConnID = u32;
pub type PortID = u16;

#[derive(Debug)]
pub enum Event {
    Connect {
        room: RoomID,
        ws: Box<WebSocket>,
        env: Env,
    },
    Disconnect {
        room: RoomID,
        conn: ConnID,
        env: Env,
    },
    ProcessExit {
        room: RoomID,
        code: Option<i32>,
        port: Option<PortID>,
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

// Channel for passing data to from child process
pub type FromProcessTx = broadcast::Sender<(Option<ConnID>, Message)>;
pub type FromProcessRx = broadcast::Receiver<(Option<ConnID>, Message)>;

pub type ProcessSenders = (FromProcessTx, ToProcessTx, ShutdownTx);
