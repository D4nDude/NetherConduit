use crate::packet::{
    primitives::{VarInt, var_int},
    stream::{DecodeError, EncodeError, RawPacketDecodable, RawPacketEncodable},
};

use log::error;

impl RawPacketDecodable for String {
    fn decode(buffer: &[u8]) -> Result<(Self, usize), DecodeError> {
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
                    return Err(DecodeError::Invalid(
                        "Failed to decode string, not valid UTF-8".to_string(),
                    ));
                }
            },
            packet_length,
        ))
    }
}

impl RawPacketEncodable for &str {
    fn encode(&self, buffer: &mut bytes::BytesMut) -> Result<usize, EncodeError> {
        if self.len() < 32767 {
            let string_length = VarInt::new(self.len().try_into()?);
            string_length.encode(buffer)?;
            buffer.extend_from_slice(self.as_bytes());
            Ok(self.len() + string_length.get_encoded_length())
        } else {
            Err(EncodeError::Invalid(format!("String is too long: {self}")))
        }
    }

    fn get_encoded_length(&self) -> usize {
        self.len() + var_int::get_var_int_encoded_length(self.len().try_into().unwrap())
    }
}

impl RawPacketEncodable for String {
    fn encode(&self, buffer: &mut bytes::BytesMut) -> Result<usize, EncodeError> {
        self.as_str().encode(buffer)
    }

    fn get_encoded_length(&self) -> usize {
        self.as_str().get_encoded_length()
    }
}
