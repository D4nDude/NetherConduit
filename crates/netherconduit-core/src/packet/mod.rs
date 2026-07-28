use std::fmt::Display;

use bytes::BytesMut;

use crate::packet::{primitives::VarInt, stream::Encode};

pub mod primitives;
pub mod stream;

#[derive(Debug, PartialEq, Eq)]
pub struct RawPacket {
    pub id: VarInt,
    pub payload: BytesMut,
}

impl RawPacket {
    pub fn new(id: impl Into<VarInt>, payload: BytesMut) -> RawPacket {
        RawPacket {
            id: id.into(),
            payload,
        }
    }

    pub fn len(&self) -> usize {
        self.id.get_encoded_length() + self.payload.len()
    }
}

impl Display for RawPacket {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "(id:{},0x{:x})", self.id, self.payload)
    }
}
