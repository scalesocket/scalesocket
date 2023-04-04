use serde_json::Value;

use crate::types::{ConnID, Framing};
use num_derive::FromPrimitive;
use num_traits::FromPrimitive;
use {bytes::Bytes, sender_sink::wrappers::SinkError, warp::ws::Message};

/// An extension trait for `Message`s that provides routing helpers
pub trait Address<T> {
    fn to(self, to: ConnID) -> (Option<ConnID>, T);
    fn to_some(self, to: Option<ConnID>) -> (Option<ConnID>, Message);
    fn broadcast(self) -> (Option<ConnID>, T);
}

impl Address<Message> for Message {
    fn to(self, to: ConnID) -> (Option<ConnID>, Message) {
        (Some(to), self)
    }

    fn to_some(self, to: Option<ConnID>) -> (Option<ConnID>, Message) {
        (to, self)
    }

    fn broadcast(self) -> (Option<ConnID>, Message) {
        (None, self)
    }
}

pub fn deserialize(
    msg: &Bytes,
    framing: Option<Framing>,
) -> Result<(Option<ConnID>, &[u8]), &'static str> {
    match framing {
        Some(mode) => match mode {
            Framing::Binary => {
                let (id, msg_type, length, payload) = parse_binary_header(msg);
                let effective_len = payload.len();
                let header_len = length as usize;

                assert_eq!(
                    effective_len, header_len,
                    "Message length {} does not match {}: Chunked payloads are not supported",
                    effective_len, header_len
                );

                match msg_type {
                    Some(Type::Binary) => Ok((id, payload)),
                    Some(Type::Text) => Ok((id, payload)),
                    None => Err("Unknown message type"),
                }
            }
            Framing::JSON => {
                let (id, msg) = parse_json_header(msg);
                Ok((id, msg))
            }
        },
        None => Ok((None, msg)),
    }
}

pub fn serialize(
    msg: Message,
    conn: ConnID,
    framing: Option<Framing>,
) -> Result<Message, SinkError> {
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

#[derive(FromPrimitive, ToPrimitive, Debug, PartialEq)]
pub enum Type {
    Text = 1,
    Binary = 2,
}

pub fn parse_json_header(msg: &Bytes) -> (Option<ConnID>, &[u8]) {
    let id = match serde_json::from_slice::<Value>(msg) {
        Ok(ref payload) => payload
            .get("id")
            .and_then(|v: &Value| v.as_u64())
            .and_then(|v: u64| u32::try_from(v).ok()),
        _ => None,
    };
    (id, msg)
}

/// Parse fixed-length 12 byte header consisting of three u32 values in network byte order.
///
/// The header consists of the routing ID, message type and payload length.
/// Message type is 1 for text and 2 for binary.
pub fn parse_binary_header(data: &[u8]) -> (Option<ConnID>, Option<Type>, u32, &[u8]) {
    let mut id_data = [0; 4];
    id_data.copy_from_slice(&data[0..4]);

    let id = u32::from_le_bytes(id_data);
    let id = match id {
        // message is broadcast
        0 => None,
        // message is routed
        id => Some(id),
    };

    let mut msg_type_data = [0; 4];
    msg_type_data.copy_from_slice(&data[4..8]);
    let msg_type = u32::from_le_bytes(msg_type_data);
    let msg_type: Option<Type> = FromPrimitive::from_u32(msg_type);

    let mut msg_len_data = [0; 4];
    msg_len_data.copy_from_slice(&data[8..12]);
    let msg_len = u32::from_le_bytes(msg_len_data);

    (id, msg_type, msg_len, &data[12..])
}

#[cfg(test)]
mod tests {
    use super::{parse_binary_header, Type};

    #[test]
    fn test_parse_id() {
        let payload = [
            (123 as u32).to_le_bytes(),
            (0 as u32).to_le_bytes(),
            (0 as u32).to_le_bytes(),
        ]
        .concat();
        let (result, _, _, _) = parse_binary_header(&payload);
        assert_eq!(result, Some(123));
    }

    #[test]
    fn test_parse_id_0_is_broadcast() {
        let payload = [
            (0 as u32).to_le_bytes(),
            (0 as u32).to_le_bytes(),
            (0 as u32).to_le_bytes(),
        ]
        .concat();
        let (result, _, _, _) = parse_binary_header(&payload);
        assert_eq!(result, None);
    }

    #[test]
    fn test_parse_type() {
        let payload = [
            (0 as u32).to_le_bytes(),
            (2 as u32).to_le_bytes(),
            (0 as u32).to_le_bytes(),
        ]
        .concat();
        let (_, result, _, _) = parse_binary_header(&payload);
        assert_eq!(result, Some(Type::Binary));
    }

    #[test]
    fn test_parse_length() {
        let payload = [
            (0 as u32).to_le_bytes(),
            (0 as u32).to_le_bytes(),
            (123 as u32).to_le_bytes(),
        ]
        .concat();
        let (_, _, result, _) = parse_binary_header(&payload);
        assert_eq!(result, 123);
    }
}
