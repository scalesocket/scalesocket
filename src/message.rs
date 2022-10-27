use serde_json::Value;

use crate::types::{ConnID, Framing};
use {sender_sink::wrappers::SinkError, warp::ws::Message};

pub trait Address<T> {
    fn to(self, to: usize) -> (Option<usize>, T);
    fn to_some(self, to: Option<usize>) -> (Option<usize>, Message);
    fn broadcast(self) -> (Option<usize>, T);
}

impl Address<Message> for Message {
    fn to(self, to: usize) -> (Option<usize>, Message) {
        (Some(to), self)
    }

    fn to_some(self, to: Option<usize>) -> (Option<usize>, Message) {
        (to, self)
    }

    fn broadcast(self) -> (Option<usize>, Message) {
        (None, self)
    }
}

pub fn encode(msg: Message, conn: ConnID, framing: Option<Framing>) -> Result<Message, SinkError> {
    match framing {
        Some(mode) => match mode {
            Framing::Binary => todo!(),
            Framing::JSON => match serde_json::from_slice::<Value>(msg.as_bytes()) {
                Ok(mut v) if v.is_object() => {
                    v["id"] = Value::from(conn);
                    Ok(Message::text(v.to_string()))
                }
                Ok(_) => {
                    tracing::error!("bad data: message is not a JSON object");
                    Err(SinkError::SendFailed)
                }
                Err(_) => {
                    tracing::error!("bad data: message is not valid JSON");
                    Err(SinkError::SendFailed)
                }
            },
        },
        None => Ok(msg),
    }
}
