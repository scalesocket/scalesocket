use {
    bytes::Bytes,
    std::net::SocketAddr,
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
    },
    Disconnect {
        room: RoomID,
        conn: ConnID,
    },
    ProcessExit {
        room: RoomID,
        code: Option<i32>,
        port: Option<PortID>,
    },
    Shutdown,
}

// Channel for app events
pub type EventTx = mpsc::UnboundedSender<Event>;
pub type EventRx = mpsc::UnboundedReceiver<Event>;

// Channel for passing data to child process
pub type ToProcessTx = mpsc::UnboundedSender<Bytes>;
pub type ToProcessRx = mpsc::UnboundedReceiver<Bytes>;
pub type ToProcessRxStream = UnboundedReceiverStream<Bytes>;

// Channel for triggering shutdown of child process
pub type ShutdownTx = oneshot::Sender<()>;
pub type ShutdownRx = oneshot::Receiver<()>;
pub type ShutdownRxStream = futures::future::IntoStream<ShutdownRx>;

// Channel for passing data to from child process
pub type FromProcessTx = broadcast::Sender<Message>;
pub type FromProcessRx = broadcast::Receiver<Message>;

#[derive(Debug, Default)]
pub struct CGIEnv {
    /// URL-encoded search or parameter string
    query_string: String,
    /// network address of the client sending the request
    remote_addr: String,
}

impl CGIEnv {
    pub fn from_filter(query_string: Option<String>, remote_addr: Option<SocketAddr>) -> Self {
        let query_string = query_string.unwrap_or_default();
        let remote_addr = remote_addr
            .map(|a| a.to_string())
            .unwrap_or_else(|| "".to_string());

        Self {
            query_string,
            remote_addr,
        }
    }
}
