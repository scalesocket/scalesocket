use {
    tokio::sync::{broadcast, mpsc, oneshot},
    tokio_stream::wrappers::{LinesStream, UnboundedReceiverStream},
    warp::ws::WebSocket,
};

pub type RoomID = String;
pub type ConnID = usize;

#[derive(Debug)]
pub enum Event {
    Connect { room: RoomID, ws: Box<WebSocket> },
    Disconnect { room: RoomID, conn: ConnID },
    ProcessExit { room: RoomID },
    Shutdown,
}

// Channel for app events
pub type EventTx = mpsc::UnboundedSender<Event>;
pub type EventRx = mpsc::UnboundedReceiver<Event>;

// Channel for passing data to child process
pub type ToProcessTx = mpsc::UnboundedSender<String>;
pub type ToProcessRx = mpsc::UnboundedReceiver<String>;
pub type ToProcessRxStream = UnboundedReceiverStream<String>;

// Channel for triggering shutdown of child process
pub type ShutdownTx = oneshot::Sender<()>;
pub type ShutdownRx = oneshot::Receiver<()>;
pub type ShutdownRxStream = futures::future::IntoStream<ShutdownRx>;

// Channel for passing data to from child process
pub type FromProcessTx = broadcast::Sender<String>;
pub type FromProcessRx = broadcast::Receiver<String>;
