use byteorder::ByteOrder;

use crate::packet::stream::{Decode, decoder::DecodeError};

impl Decode for u16 {
    fn decode(buffer: &[u8]) -> Result<(Self, usize), crate::packet::stream::decoder::DecodeError> {
        if buffer.len() < 2 {
            return Err(DecodeError::Incomplete);
        }
        Ok((byteorder::BigEndian::read_u16(buffer), 2))
    }
}
