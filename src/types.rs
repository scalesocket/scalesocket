use crate::envvars::Env;
use {
    bytes::Bytes,
    tokio::sync::{broadcast, mpsc, oneshot},
    tokio_stream::wrappers::UnboundedReceiverStream,
    warp::ws::{Message, WebSocket},
};

pub type RoomID = String;
pub type ConnID = usize;
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

#[derive(Debug, clap::ValueEnum, Clone, Copy)]
pub enum Framing {
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
pub type FromProcessTx = broadcast::Sender<Message>;
pub type FromProcessRx = broadcast::Receiver<Message>;
