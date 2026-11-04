use crate::packet::stream::{DecodeError, EncodeError, RawPacketDecodable, RawPacketEncodable};

impl<T> RawPacketDecodable for Option<T> {
    fn decode(_buffer: &[u8]) -> Result<(Self, usize), DecodeError> {
        todo!()
    }
}

impl<T> RawPacketEncodable for Option<T> {
    fn encode(&self, _buffer: &mut bytes::BytesMut) -> Result<usize, EncodeError> {
        todo!()
    }

    fn get_encoded_length(&self) -> usize {
        todo!()
    }
}
