use {tokio::sync::mpsc, warp::ws::WebSocket};

pub type RoomID = String;
pub type UserID = usize;

#[derive(Debug)]
pub enum Event {
    Connect { room: RoomID, ws: Box<WebSocket> },
}

pub type EventTx = mpsc::UnboundedSender<Event>;
pub type EventRx = mpsc::UnboundedReceiver<Event>;
