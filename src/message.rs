use {
    bytes::Bytes, num_derive::FromPrimitive, num_traits::FromPrimitive,
    sender_sink::wrappers::SinkError, serde_json::Value, warp::ws::Message,
};

use crate::types::{ConnID, Frame, Header};

/// An extension trait for `Message`s that provides routing helpers
pub trait Address<T> {
    fn header(self, header: Header) -> (Header, T);
    #[cfg(test)]
    fn to(self, to: ConnID) -> (Header, T);
    #[cfg(test)]
    fn broadcast(self) -> (Header, T);
}

impl Address<Message> for Message {
    fn header(self, header: Header) -> (Header, Message) {
        (header, self)
    }

    #[cfg(test)]
    fn to(self, to: ConnID) -> (Header, Message) {
        (Header::to(to), self)
    }

    #[cfg(test)]
    fn broadcast(self) -> (Header, Message) {
        (Header::broadcast(), self)
    }
}

/// Deserialize message coming from process
pub fn deserialize(msg: &Bytes, frame: Option<Frame>) -> Result<(Header, &[u8]), &'static str> {
    match frame {
        Some(f) => match f {
            Frame::GWSocket => {
                let (header, msg_type, length, payload) = parse_binary_header(msg);
                let effective_len = payload.len();
                let header_len = length as usize;

                assert_eq!(
                    effective_len, header_len,
                    "Message length {} does not match {}: Chunked payloads are not supported",
                    effective_len, header_len
                );

                if msg_type.is_none() {
                    return Err("Unknown message type");
                }

                Ok((header, payload))
            }
            Frame::JSON => Ok(parse_json_header(msg)),
        },
        None => Ok((Header::broadcast(), msg)),
    }
}

/// Serialize message going to process
pub fn serialize(msg: Message, conn: ConnID, frame: Option<Frame>) -> Result<Message, SinkError> {
    match frame {
        Some(f) => match f {
            Frame::GWSocket => {
                unimplemented!("Client side binary framing has not been implemented")
            }
            Frame::JSON => match serde_json::from_slice::<Value>(msg.as_bytes()) {
                Ok(mut v) if v.is_object() => {
                    v["_from"] = Value::from(conn);
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

pub(crate) fn parse_json_header(msg: &Bytes) -> (Header, &[u8]) {
    (
        serde_json::from_slice::<Header>(msg).unwrap_or_default(),
        msg,
    )
}

/// Parse fixed-length 12 byte header consisting of three u32 values in network byte order.
///
/// The header consists of the routing ID, message type and payload length.
/// Message type is 1 for text and 2 for binary.
pub(crate) fn parse_binary_header(data: &[u8]) -> (Header, Option<Type>, u32, &[u8]) {
    let mut id_data = [0; 4];
    id_data.copy_from_slice(&data[0..4]);
    let id = u32::from_le_bytes(id_data);

    let mut msg_type_data = [0; 4];
    msg_type_data.copy_from_slice(&data[4..8]);
    let msg_type = u32::from_le_bytes(msg_type_data);
    let msg_type: Option<Type> = FromPrimitive::from_u32(msg_type);

    let mut msg_len_data = [0; 4];
    msg_len_data.copy_from_slice(&data[8..12]);
    let msg_len = u32::from_le_bytes(msg_len_data);

    let header = match id {
        // message is broadcast
        0 => Header::broadcast(),
        // message is routed
        id => Header::to(id),
    };

    (header, msg_type, msg_len, &data[12..])
}

#[cfg(test)]
mod tests {

    use super::{parse_binary_header, Header, Type};

    #[test]
    fn test_parse_id() {
        let payload = [
            (123 as u32).to_le_bytes(),
            (0 as u32).to_le_bytes(),
            (0 as u32).to_le_bytes(),
        ]
        .concat();
        let (result, _, _, _) = parse_binary_header(&payload);
        assert_eq!(result, Header::to(123));
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
        assert_eq!(result, Header::broadcast());
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
