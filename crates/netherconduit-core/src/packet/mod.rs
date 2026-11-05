use std::{fmt::Display, io::Write};

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
    data: Bytes,
    compressed: bool,
}

impl RawPacket {
    pub fn new(data: Bytes) -> RawPacket {
        RawPacket {
            data,
            compressed: false,
        }
    }

    pub fn new_compressed(data: Bytes) -> RawPacket {
        RawPacket {
            data,
            compressed: true,
        }
    }

    pub fn construct(id: impl Into<VarInt>, payload: Bytes) -> RawPacket {
        let id = id.into();
        let mut dst = BytesMut::new();
        id.encode(&mut dst).unwrap();
        dst.extend(payload);
        RawPacket::new(dst.freeze())
    }

    pub fn len(&self) -> usize {
        self.data.len()
    }

    pub fn is_empty(&self) -> bool {
        self.data.is_empty()
    }

    pub fn raw_data(&self) -> &Bytes {
        &self.data
    }

    pub fn data(&self) -> Result<Bytes, DecodeError> {
        if self.compressed {
            decompress_data(self.data.clone())
        } else {
            Ok(self.data.clone())
        }
    }

    pub fn id(&self) -> Result<VarInt, DecodeError> {
        let (vint, _) = VarInt::decode(&self.data()?)?;
        Ok(vint)
    }

    pub fn payload(&self) -> Result<Bytes, DecodeError> {
        let (_, id_len) = VarInt::decode(&self.data()?)?;
        Ok(self.data()?.slice(id_len..))
    }

    pub fn split(&self) -> Result<(VarInt, Bytes), DecodeError> {
        let (id, id_len) = VarInt::decode(&self.data()?)?;
        Ok((
            id,
            self.data()?.slice(id_len..),
        ))
    }
}

impl Display for RawPacket {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.is_empty() {
            write!(f, "(id:{},None)", self.id().unwrap())
        } else if self.compressed {
            write!(
                f,
                "(id:{},compressed:0x{:x},0x{:x})",
                self.id().unwrap(),
                self.payload().unwrap(),
                self.data().unwrap()
            )
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

fn decompress_data(data: Bytes) -> Result<Bytes, DecodeError> {
    use flate2::write::ZlibDecoder;

    let (uncompressed_len, len_size) = VarInt::decode(&data)?;
    if uncompressed_len.value() == 0 {
        return Ok(data.slice(1..)); //TODO: May panic
    }

    let mut decoder = ZlibDecoder::new(Vec::with_capacity(
        uncompressed_len.value_u32()?.try_into()?,
    ));
    decoder.write_all(&data.slice(len_size..))?; //TODO: May panic
    Ok(decoder.finish()?.into())
}
