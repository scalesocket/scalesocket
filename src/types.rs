use {
    tokio::sync::{broadcast, mpsc},
    warp::ws::WebSocket,
};

pub type RoomID = String;
pub type ConnID = usize;

#[derive(Debug)]
pub enum Event {
    Connect { room: RoomID, ws: Box<WebSocket> },
    ProcessExit { room: RoomID },
}

// Channel for app events
pub type EventTx = mpsc::UnboundedSender<Event>;
pub type EventRx = mpsc::UnboundedReceiver<Event>;

// Channel for passing data to child process
pub type ToProcessTx = mpsc::UnboundedSender<String>;
pub type ToProcessRx = mpsc::UnboundedReceiver<String>;

// Channel for passing data to from child process
pub type FromProcessTx = broadcast::Sender<String>;
pub type FromProcessRx = broadcast::Receiver<String>;
