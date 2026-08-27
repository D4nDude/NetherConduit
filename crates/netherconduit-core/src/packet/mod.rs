use std::fmt::Display;

use bytes::{Bytes, BytesMut};

use crate::packet::{
    primitives::VarInt,
    stream::{Decode, Encode, decoder::DecodeError},
};

pub mod factory;
pub mod primitives;
pub mod stream;
pub mod types;

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct RawPacket {
    pub data: Bytes,
}

impl RawPacket {
    pub fn new(data: Bytes) -> RawPacket {
        RawPacket { data }
    }

    pub fn construct(id: impl Into<VarInt>, payload: Bytes) -> RawPacket {
        let id = id.into();
        let mut dst = BytesMut::new();
        id.encode(&mut dst);
        dst.extend(payload);
        RawPacket {
            data: dst.freeze()
        }
    }

    pub fn len(&self) -> usize {
        self.data.len()
    }

    pub fn is_empty(&self) -> bool {
        self.data.len() == self.id().unwrap().get_encoded_length()
    }

    pub fn id(&self) -> Result<VarInt, DecodeError> {
        let (vint, _) = VarInt::decode(&self.data)?;
        Ok(vint)
    }

    pub fn payload(&self) -> Result<Bytes, DecodeError> {
        let (_, id_len) = VarInt::decode(&self.data)?;
        Ok(self.data.slice(id_len..))
    }
}

impl Display for RawPacket {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "(id:{},0x{:x})",
            self.id().unwrap(),
            self.payload().unwrap()
        )
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ConnectionState {
    Handshake,
    Status,
    Login,
    Configuration,
    Play,
    Closed,
}
