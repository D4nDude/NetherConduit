use crate::packet::{
    primitives::VarInt,
    stream::{Decode, decoder::DecodeError},
};

use log::error;

impl Decode for String {
    fn decode(buffer: &[u8]) -> Result<(Self, usize), crate::packet::stream::decoder::DecodeError> {
        let (string_length, length_bytes) = VarInt::decode(buffer)?;
        let packet_length = usize::try_from(string_length)? + length_bytes;
        let output_string = match buffer.get(length_bytes..packet_length) {
            Some(value) => value,
            None => {
                error!("Failed to decode String, length was too short");
                return Err(DecodeError::Incomplete);
            }
        };
        Ok((
            match String::from_utf8(output_string.to_vec()) {
                Ok(value) => value,
                Err(_) => {
                    error!("Failed to decode string, not valid UTF-8");
                    return Err(DecodeError::Invalid);
                }
            },
            packet_length,
        ))
    }
}
