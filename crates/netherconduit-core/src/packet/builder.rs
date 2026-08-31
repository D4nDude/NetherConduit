use bytes::BytesMut;
use serde::Serialize;

use crate::packet::{
    RawPacket,
    primitives::VarInt,
    stream::{EncodeError, RawPacketEncodable},
};

pub struct RawPacketBuilder {
    data: BytesMut,
}

impl RawPacketBuilder {
    pub fn new(id: impl Into<VarInt>) -> Self {
        let id = id.into();

        let mut data = BytesMut::new();
        id.encode(&mut data).unwrap();

        Self { data }
    }

    pub fn build(self) -> RawPacket {
        RawPacket {
            data: self.data.freeze(),
        }
    }

    pub fn put(mut self, data: impl RawPacketEncodable) -> Result<Self, EncodeError> {
        log::warn!("Generic put in builder, use concrete for better readability");
        data.encode(&mut self.data)?;
        Ok(self)
    }

    pub fn bool(mut self, bool: bool) -> Result<Self, EncodeError> {
        bool.encode(&mut self.data)?;
        Ok(self)
    }

    pub fn byte(mut self, byte: i8) -> Result<Self, EncodeError> {
        byte.encode(&mut self.data)?;
        Ok(self)
    }

    pub fn unsigned_byte(mut self, unsigned_byte: u8) -> Result<Self, EncodeError> {
        unsigned_byte.encode(&mut self.data)?;
        Ok(self)
    }

    pub fn short(mut self, short: i16) -> Result<Self, EncodeError> {
        short.encode(&mut self.data)?;
        Ok(self)
    }

    pub fn unsigned_short(mut self, unsigned_short: u16) -> Result<Self, EncodeError> {
        unsigned_short.encode(&mut self.data)?;
        Ok(self)
    }

    pub fn int(mut self, int: i32) -> Result<Self, EncodeError> {
        int.encode(&mut self.data)?;
        Ok(self)
    }

    pub fn long(mut self, long: i64) -> Result<Self, EncodeError> {
        long.encode(&mut self.data)?;
        Ok(self)
    }

    pub fn float(mut self, float: f32) -> Result<Self, EncodeError> {
        float.encode(&mut self.data)?;
        Ok(self)
    }

    pub fn double(mut self, double: f64) -> Result<Self, EncodeError> {
        double.encode(&mut self.data)?;
        Ok(self)
    }

    pub fn string_slice(mut self, string_slice: &str) -> Result<Self, EncodeError> {
        string_slice.encode(&mut self.data)?;
        Ok(self)
    }

    pub fn string(mut self, string: String) -> Result<Self, EncodeError> {
        string.encode(&mut self.data)?;
        Ok(self)
    }

    pub fn json<T: Serialize + ?Sized>(mut self, jsonable: &T) -> Result<Self, EncodeError> {
        serde_json::to_string(jsonable)?.encode(&mut self.data)?;
        Ok(self)
    }

    pub fn var_int(mut self, var_int: VarInt) -> Result<Self, EncodeError> {
        var_int.encode(&mut self.data)?;
        Ok(self)
    }
}

impl From<RawPacketBuilder> for RawPacket {
    fn from(builder: RawPacketBuilder) -> Self {
        RawPacket {
            data: builder.data.freeze(),
        }
    }
}
