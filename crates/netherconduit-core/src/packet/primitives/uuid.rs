use crate::packet::stream::{DecodeError, EncodeError, RawPacketDecodable, RawPacketEncodable};
use uuid::Uuid;

impl RawPacketDecodable for Uuid {
    fn decode(buffer: &[u8]) -> Result<(Self, usize), DecodeError> {
        let (num, size) = u128::decode(buffer)?;
        Ok((Uuid::from_u128(num), size))
    }
}

impl RawPacketEncodable for Uuid {
    fn encode(&self, buffer: &mut bytes::BytesMut) -> Result<usize, EncodeError> {
        self.as_u128().encode(buffer)
    }

    fn get_encoded_length(&self) -> usize {
        16
    }
}
