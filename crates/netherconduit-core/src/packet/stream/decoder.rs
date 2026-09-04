use bytes::BytesMut;
use tokio_util::codec::Decoder;

use crate::packet::RawPacket;
use crate::packet::primitives::VarInt;
use crate::packet::stream::DecodeError;

const MAX_PACKET_SIZE: usize = (2 ^ 21) - 1;

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
        let (packet_length, int_size) = match VarInt::decode_bounded::<3>(src) {
            Ok(value) => value,
            Err(DecodeError::Incomplete) => return Ok(None), // not enough data for a varint
            Err(DecodeError::Invalid(error)) => {
                return Err(std::io::Error::new(
                    std::io::ErrorKind::InvalidData,
                    format!("Packet Length is invalid value: {error}"),
                ));
            }
        };
        
        // check length is not negative
        let packet_length = match usize::try_from(packet_length) {
            Ok(size) => size,
            Err(error) => {
                return Err(std::io::Error::new(
                    std::io::ErrorKind::InvalidData,
                    format!("Packet length could not be converted to usize: {error}"),
                ));
            }
        };

        // check packet is not too long
        if packet_length > MAX_PACKET_SIZE {
            return Err(
                std::io::Error::new(std::io::ErrorKind::InvalidData,
                format!("Packet Length is defined too long: {packet_length}"),
            ))
        }

        

        let full_packet_length = packet_length + int_size;

        // incomplete packet
        if src.len() < full_packet_length {
            return Ok(None);
        };

        let data = src.split_to(full_packet_length);

        Ok(Some(RawPacket::new(data.freeze(), int_size)))
    }
}

#[cfg(test)]
mod test {
    use super::MinecraftPacketDecoder;
    use crate::packet::primitives::VarInt;
    use bytes::{BufMut, Bytes, BytesMut};
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
        assert_eq!(output_packet.id().unwrap(), VarInt::new(16));
        assert_eq!(
            output_packet.payload().unwrap(),
            Bytes::from(&[0x10; 15][..])
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
        assert_eq!(output_packet.id().unwrap(), VarInt::new(16));
        assert_eq!(
            output_packet.payload().unwrap(),
            Bytes::from(&[0x10; 15][..])
        );

        test_data.put_bytes(0x11, 18);
        let output_packet = decoder.decode(&mut test_data).unwrap().unwrap();
        assert_eq!(output_packet.id().unwrap(), VarInt::new(17));
        assert_eq!(
            output_packet.payload().unwrap(),
            Bytes::from(&[0x11; 16][..])
        );
    }

    #[test]
    fn double_well_sized_packets() {
        let mut decoder = MinecraftPacketDecoder::new();

        let mut test_data: BytesMut = BytesMut::from(&[0x10; 17][..]);
        test_data.put_bytes(0x11, 18);

        let output_packet = decoder.decode(&mut test_data).unwrap().unwrap();
        assert_eq!(output_packet.id().unwrap(), VarInt::new(16));
        assert_eq!(
            output_packet.payload().unwrap(),
            Bytes::from(&[0x10; 15][..])
        );
        let output_packet = decoder.decode(&mut test_data).unwrap().unwrap();
        assert_eq!(output_packet.id().unwrap(), VarInt::new(17));
        assert_eq!(
            output_packet.payload().unwrap(),
            Bytes::from(&[0x11; 16][..])
        );
    }

    // vec![0x008606096c6f63616c686f73741ec601]
}
