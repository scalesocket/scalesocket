use {
    bytes::Bytes,
    std::collections::HashMap,
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

#[derive(Clone, Debug, Default)]
pub struct Env {
    pub cgi: CGIEnv,
    pub query: HashMap<String, String>,
}

impl From<Env> for HashMap<String, String> {
    /// Performs conversion and turns query keys to uppercase
    fn from(env: Env) -> Self {
        let mut result: HashMap<String, String> = env.cgi.into();
        let query_uppercase: HashMap<String, String> = env
            .query
            .into_iter()
            .map(|(k, v)| (k.to_uppercase(), v))
            .collect();

        result.extend(query_uppercase);
        result
    }
}

#[derive(Clone, Debug, Default)]
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

impl From<CGIEnv> for HashMap<String, String> {
    fn from(env: CGIEnv) -> Self {
        HashMap::from([
            // NOTE: implicit uppercase
            ("QUERY_STRING".to_string(), env.query_string),
            ("REMOTE_ADDR".to_string(), env.remote_addr),
        ])
    }
}
