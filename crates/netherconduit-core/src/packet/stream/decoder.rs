use bytes::{Buf, BytesMut};
use tokio_util::codec::Decoder;

use crate::packet::RawPacket;
use crate::packet::primitives::VarInt;
use crate::packet::stream::Decode;

#[derive(Debug, PartialEq, Eq, Hash, Clone, Copy)]
pub enum DecodeError {
    Incomplete,
    Invalid,
}

impl From<DecodeError> for std::io::Error {
    fn from(value: DecodeError) -> Self {
        match value {
            DecodeError::Incomplete => std::io::Error::new(
                    std::io::ErrorKind::UnexpectedEof,
                    "Not enough data to create value",
                ),
            DecodeError::Invalid => std::io::Error::new(
                    std::io::ErrorKind::InvalidData,
                    "Invalid data format for value",
                ),
        }
    }
}

#[derive(Debug, Default, PartialEq, Eq)]
pub struct MinecraftPacketDecoder {}

impl MinecraftPacketDecoder {
    pub fn new() -> MinecraftPacketDecoder {
        MinecraftPacketDecoder {}
    }
}

impl Decoder for MinecraftPacketDecoder {
    type Item = RawPacket;
    type Error = std::io::Error;

    fn decode(&mut self, src: &mut BytesMut) -> Result<Option<Self::Item>, Self::Error> {
        let (packet_length, int_size) = match VarInt::decode(src) {
            Ok(value) => value,
            Err(DecodeError::Incomplete) => return Ok(None), // not enough data for a varint
            Err(DecodeError::Invalid) => {
                return Err(std::io::Error::new(
                    std::io::ErrorKind::InvalidData,
                    "Packet Length is invalid value",
                ));
            }
        };

        let full_packet_length = usize::try_from(packet_length).unwrap() + int_size;

        // incomplete packet
        if src.len() < full_packet_length {
            return Ok(None);
        };

        src.advance(int_size);

        let mut data = src.split_to(usize::try_from(packet_length).unwrap());

        // We should be guaranteed to get a valid VarInt from here
        let (packet_id, _) = VarInt::decode_consuming_buffer(&mut data).unwrap();

        Ok(Some(RawPacket::new(packet_id, data)))
    }
}

#[cfg(test)]
mod test {
    use super::MinecraftPacketDecoder;
    use crate::packet::RawPacket;
    use crate::packet::primitives::VarInt;
    use bytes::{BufMut, BytesMut};
    use std::assert_matches;
    use tokio_util::codec::Decoder;

    #[test]
    fn no_packet() {
        let mut decoder = MinecraftPacketDecoder::new();
        let mut test_data: BytesMut = BytesMut::new();
        let output = decoder.decode(&mut test_data).unwrap();
        assert_matches!(output, None);
    }

    #[test]
    fn well_sized_packet() {
        let mut decoder = MinecraftPacketDecoder::new();
        let mut test_data: BytesMut = BytesMut::from(&[0x10; 17][..]);
        let output_packet = decoder.decode(&mut test_data).unwrap().unwrap();
        assert_eq!(
            output_packet,
            RawPacket::new(VarInt::new(16), BytesMut::from(&[0x10; 15][..]))
        );
    }

    #[test]
    fn too_short_packet() {
        let mut decoder = MinecraftPacketDecoder::new();
        let mut test_data: BytesMut = BytesMut::from(&[0x10; 16][..]);
        let output = decoder.decode(&mut test_data).unwrap();
        assert_matches!(output, None);
    }

    #[test]
    fn repeated_well_sized_packets() {
        let mut decoder = MinecraftPacketDecoder::new();

        let mut test_data: BytesMut = BytesMut::from(&[0x10; 17][..]);
        let output_packet = decoder.decode(&mut test_data).unwrap().unwrap();
        assert_eq!(
            output_packet,
            RawPacket::new(VarInt::new(16), BytesMut::from(&[0x10; 15][..]))
        );

        test_data.put_bytes(0x11, 18);
        let output_packet = decoder.decode(&mut test_data).unwrap().unwrap();
        assert_eq!(
            output_packet,
            RawPacket::new(VarInt::new(17), BytesMut::from(&[0x11; 16][..]))
        );
    }

    #[test]
    fn double_well_sized_packets() {
        let mut decoder = MinecraftPacketDecoder::new();

        let mut test_data: BytesMut = BytesMut::from(&[0x10; 17][..]);
        test_data.put_bytes(0x11, 18);

        let output_packet = decoder.decode(&mut test_data).unwrap().unwrap();
        assert_eq!(
            output_packet,
            RawPacket::new(VarInt::new(16), BytesMut::from(&[0x10; 15][..]))
        );
        let output_packet = decoder.decode(&mut test_data).unwrap().unwrap();
        assert_eq!(
            output_packet,
            RawPacket::new(VarInt::new(17), BytesMut::from(&[0x11; 16][..]))
        );
    }

    // vec![0x008606096c6f63616c686f73741ec601]
}
