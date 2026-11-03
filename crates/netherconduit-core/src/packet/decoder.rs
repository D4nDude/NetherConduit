use bytes::Bytes;
#[cfg(feature = "trace-parser")]
use log::trace;
use uuid::Uuid;

use crate::packet::{
    primitives::{GameProfile, VarInt},
    stream::{DecodeError, RawPacketDecodable},
};

#[derive(Debug)]
pub struct RawPacketDecoder {
    data: Bytes,
    cursor: usize,
}

impl RawPacketDecoder {
    pub fn new(data: Bytes) -> RawPacketDecoder {
        #[cfg(feature = "trace-parser")]
        trace!("New Parser Initialised: {data:?}");
        RawPacketDecoder { data, cursor: 0 }
    }

    pub fn boolean(&mut self) -> Result<bool, DecodeError> {
        let (bool, bool_len) = bool::decode(&self.data.slice(self.cursor..))?;
        self.cursor += bool_len;
        #[cfg(feature = "trace-parser")]
        trace!("Parser bool: {bool}");
        Ok(bool)
    }

    pub fn byte(&mut self) -> Result<i8, DecodeError> {
        let (byte, byte_len) = i8::decode(&self.data.slice(self.cursor..))?;
        self.cursor += byte_len;
        #[cfg(feature = "trace-parser")]
        trace!("Parser byte: {byte}");
        Ok(byte)
    }

    pub fn unsigned_byte(&mut self) -> Result<u8, DecodeError> {
        let (unsigned_byte, unsigned_byte_len) = u8::decode(&self.data.slice(self.cursor..))?;
        self.cursor += unsigned_byte_len;
        #[cfg(feature = "trace-parser")]
        trace!("Parser unsigned_byte: {unsigned_byte}");
        Ok(unsigned_byte)
    }

    pub fn short(&mut self) -> Result<i16, DecodeError> {
        let (short, short_len) = i16::decode(&self.data.slice(self.cursor..))?;
        self.cursor += short_len;
        #[cfg(feature = "trace-parser")]
        trace!("Parser short: {short}");
        Ok(short)
    }

    pub fn unsigned_short(&mut self) -> Result<u16, DecodeError> {
        let (unsigned_short, short_len) = u16::decode(&self.data.slice(self.cursor..))?;
        self.cursor += short_len;
        #[cfg(feature = "trace-parser")]
        trace!("Parser unsigned_short: {unsigned_short}");
        Ok(unsigned_short)
    }

    pub fn int(&mut self) -> Result<i32, DecodeError> {
        let (int, int_len) = i32::decode(&self.data.slice(self.cursor..))?;
        self.cursor += int_len;
        #[cfg(feature = "trace-parser")]
        trace!("Parser int: {int}");
        Ok(int)
    }

    pub fn long(&mut self) -> Result<i64, DecodeError> {
        let (long, long_len) = i64::decode(&self.data.slice(self.cursor..))?;
        self.cursor += long_len;
        #[cfg(feature = "trace-parser")]
        trace!("Parser long: {long}");
        Ok(long)
    }

    pub fn float(&mut self) -> Result<f32, DecodeError> {
        let (float, float_len) = f32::decode(&self.data.slice(self.cursor..))?;
        self.cursor += float_len;
        #[cfg(feature = "trace-parser")]
        trace!("Parser float: {float}");
        Ok(float)
    }

    pub fn double(&mut self) -> Result<f64, DecodeError> {
        let (double, double_len) = f64::decode(&self.data.slice(self.cursor..))?;
        self.cursor += double_len;
        #[cfg(feature = "trace-parser")]
        trace!("Parser double: {double}");
        Ok(double)
    }

    pub fn string(&mut self) -> Result<String, DecodeError> {
        let (string, str_len) = String::decode(&self.data.slice(self.cursor..))?;
        self.cursor += str_len;
        #[cfg(feature = "trace-parser")]
        trace!("Parser string: \"{string}\"");
        Ok(string)
    }

    pub fn var_int(&mut self) -> Result<VarInt, DecodeError> {
        let (var_int, int_len) = VarInt::decode(&self.data.slice(self.cursor..))?;
        self.cursor += int_len;
        #[cfg(feature = "trace-parser")]
        trace!("Parser var_int: {var_int}");
        Ok(var_int)
    }

    pub fn uuid(&mut self) -> Result<Uuid, DecodeError> {
        if (self.cursor + 16) > self.data.len() {
            return Err(DecodeError::Incomplete);
        }
        let uuid = *Uuid::from_bytes_ref(
            self.data
                .slice(self.cursor..(self.cursor + 16))
                .as_array::<16>()
                .expect("Should be exactly 16 Bytes"),
        );
        self.cursor += 16;
        #[cfg(feature = "trace-parser")]
        trace!("Parser uuid: {uuid:?}");
        Ok(uuid)
    }

    pub fn game_profile(&mut self) -> Result<GameProfile, DecodeError> {
        let (game_profile, profile_len) = GameProfile::decode(&self.data.slice(self.cursor..))?;
        self.cursor += profile_len;
        #[cfg(feature = "trace-parser")]
        trace!("Parser game_profile: {game_profile:?}");
        Ok(game_profile)
    }
}
