use std::fmt::Display;

use bytes::{Bytes, BytesMut};

use crate::packet::{
    primitives::VarInt,
    stream::{DecodeError, RawPacketDecodable, RawPacketEncodable},
};

pub mod builder;
pub mod decoder;
pub mod primitives;
pub mod stream;
pub mod types;

#[derive(Debug, Clone, PartialEq, Eq, Default, Hash)]
pub struct RawPacket {
    raw_data: Bytes,
    packet_length_offset: usize,
}

impl RawPacket {
    pub fn new(raw_data: Bytes, packet_length_offset: usize) -> RawPacket {
        RawPacket {
            raw_data,
            packet_length_offset,
        }
    }

    pub fn from_data(data: Bytes) -> RawPacket {
        let packet_length = VarInt::new(data.len().try_into().unwrap());
        let mut dst = BytesMut::new();
        packet_length.encode(&mut dst).unwrap();
        dst.extend(data);
        RawPacket {
            raw_data: dst.freeze(),
            packet_length_offset: packet_length.get_encoded_length(),
        }
    }

    pub fn construct(id: impl Into<VarInt>, payload: Bytes) -> RawPacket {
        let id = id.into();
        let mut dst = BytesMut::new();
        id.encode(&mut dst).unwrap();
        dst.extend(payload);
        RawPacket::from_data(dst.freeze())
    }

    pub fn data_len(&self) -> usize {
        self.raw_data.len() - self.packet_length_offset
    }

    pub fn is_empty(&self) -> bool {
        self.raw_data.len() == (self.id().unwrap().get_encoded_length() + self.packet_length_offset)
    }

    pub fn raw_data(&self) -> &Bytes {
        &self.raw_data
    }

    pub fn data(&self) -> Bytes {
        self.raw_data.slice(self.packet_length_offset..)
    }

    pub fn id(&self) -> Result<VarInt, DecodeError> {
        let (vint, _) = VarInt::decode(&self.data())?;
        Ok(vint)
    }

    pub fn payload(&self) -> Result<Bytes, DecodeError> {
        let (_, id_len) = VarInt::decode(&self.data())?;
        Ok(self.raw_data.slice((self.packet_length_offset + id_len)..))
    }

    pub fn split(&self) -> Result<(VarInt, Bytes), DecodeError> {
        let (id, id_len) = VarInt::decode(&self.data())?;
        Ok((
            id,
            self.raw_data.slice((self.packet_length_offset + id_len)..),
        ))
    }
}

impl Display for RawPacket {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.is_empty() {
            write!(f, "(id:{},None)", self.id().unwrap())
        } else {
            write!(
                f,
                "(id:{},0x{:x})",
                self.id().unwrap(),
                self.payload().unwrap()
            )
        }
    }
}
