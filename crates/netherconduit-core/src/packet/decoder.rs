use bytes::Bytes;

use crate::packet::{
    primitives::VarInt,
    stream::{DecodeError, RawPacketDecodable},
};

pub struct RawPacketDecoder {
    data: Bytes,
    cursor: usize,
}

impl RawPacketDecoder {
    pub fn new(data: Bytes) -> RawPacketDecoder {
        RawPacketDecoder { data, cursor: 0 }
    }

    pub fn boolean(&mut self) -> Result<bool, DecodeError> {
        let (bool, bool_len) = bool::decode(&self.data.slice(self.cursor..))?;
        self.cursor += bool_len;
        Ok(bool)
    }

    pub fn byte(&mut self) -> Result<i8, DecodeError> {
        let (byte, byte_len) = i8::decode(&self.data.slice(self.cursor..))?;
        self.cursor += byte_len;
        Ok(byte)
    }

    pub fn unsigned_byte(&mut self) -> Result<u8, DecodeError> {
        let (unsigned_byte, unsigned_byte_len) = u8::decode(&self.data.slice(self.cursor..))?;
        self.cursor += unsigned_byte_len;
        Ok(unsigned_byte)
    }

    pub fn short(&mut self) -> Result<i16, DecodeError> {
        let (short, short_len) = i16::decode(&self.data.slice(self.cursor..))?;
        self.cursor += short_len;
        Ok(short)
    }

    pub fn unsigned_short(&mut self) -> Result<u16, DecodeError> {
        let (unsigned_short, short_len) = u16::decode(&self.data.slice(self.cursor..))?;
        self.cursor += short_len;
        Ok(unsigned_short)
    }

    pub fn int(&mut self) -> Result<i32, DecodeError> {
        let (int, int_len) = i32::decode(&self.data.slice(self.cursor..))?;
        self.cursor += int_len;
        Ok(int)
    }

    pub fn long(&mut self) -> Result<i64, DecodeError> {
        let (long, long_len) = i64::decode(&self.data.slice(self.cursor..))?;
        self.cursor += long_len;
        Ok(long)
    }

    pub fn float(&mut self) -> Result<f32, DecodeError> {
        let (float, float_len) = f32::decode(&self.data.slice(self.cursor..))?;
        self.cursor += float_len;
        Ok(float)
    }

    pub fn double(&mut self) -> Result<f64, DecodeError> {
        let (double, double_len) = f64::decode(&self.data.slice(self.cursor..))?;
        self.cursor += double_len;
        Ok(double)
    }

    pub fn string(&mut self) -> Result<String, DecodeError> {
        let (string, str_len) = String::decode(&self.data.slice(self.cursor..))?;
        self.cursor += str_len;
        Ok(string)
    }

    pub fn var_int(&mut self) -> Result<VarInt, DecodeError> {
        let (var_int, int_len) = VarInt::decode(&self.data.slice(self.cursor..))?;
        self.cursor += int_len;
        Ok(var_int)
    }
}
