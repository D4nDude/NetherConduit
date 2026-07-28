use bytes::BytesMut;
use tokio_util::codec::Decoder;

use crate::packet::RawPacket;
use crate::packet::primitives::peak_varint;

pub struct MinecraftPacketDecoder {}

impl MinecraftPacketDecoder {
    pub fn new() -> MinecraftPacketDecoder {
        MinecraftPacketDecoder {}
    }
}

impl Default for MinecraftPacketDecoder {
    fn default() -> Self {
        Self::new()
    }
}

impl Decoder for MinecraftPacketDecoder {
    type Item = RawPacket;
    type Error = std::io::Error;

    fn decode(&mut self, src: &mut BytesMut) -> Result<Option<Self::Item>, Self::Error> {
        let (length, int_size) = match peak_varint(src) {
            Some(value) => value,
            None => return Ok(None), // not enough data for a varint
        };

        let packet_length = usize::try_from(length).unwrap() + int_size;

        // incomplete packet
        if src.len() < packet_length {
            return Ok(None);
        };

        let data = src.split_to(packet_length);
        Ok(Some(RawPacket::new(data)))
    }
}

#[cfg(test)]
mod test {
    use super::MinecraftPacketDecoder;
    use crate::packet::RawPacket;
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
            RawPacket::new(BytesMut::from(&[0x10; 17][..]))
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
            RawPacket::new(BytesMut::from(&[0x10; 17][..]))
        );

        test_data.put_bytes(0x11, 18);
        let output_packet = decoder.decode(&mut test_data).unwrap().unwrap();
        assert_eq!(
            output_packet,
            RawPacket::new(BytesMut::from(&[0x11; 18][..]))
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
            RawPacket::new(BytesMut::from(&[0x10; 17][..]))
        );
        let output_packet = decoder.decode(&mut test_data).unwrap().unwrap();
        assert_eq!(
            output_packet,
            RawPacket::new(BytesMut::from(&[0x11; 18][..]))
        );
    }

    // vec![0x008606096c6f63616c686f73741ec601]
}
