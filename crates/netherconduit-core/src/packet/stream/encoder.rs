use bytes::BytesMut;
use tokio_util::codec::Encoder;

use crate::packet::RawPacket;
use crate::packet::primitives::VarInt;
use crate::packet::stream::Encode;

#[derive(Debug, Default, PartialEq, Eq)]
pub struct MinecraftPacketEncoder {}

impl MinecraftPacketEncoder {
    pub fn new() -> MinecraftPacketEncoder {
        MinecraftPacketEncoder {}
    }
}

impl Encoder<RawPacket> for MinecraftPacketEncoder {
    type Error = std::io::Error;

    fn encode(&mut self, item: RawPacket, dst: &mut BytesMut) -> Result<(), Self::Error> {
        let id_byte_length = item.id.get_encoded_length();
        let packet_length =
            VarInt::new(i32::try_from(id_byte_length + item.payload.len()).unwrap());
        packet_length.encode(dst);
        item.id.encode(dst);
        dst.extend(item.payload);
        Ok(())
    }
}

mod test {
    use super::MinecraftPacketEncoder;
    use crate::packet::RawPacket;
    use crate::packet::primitives::VarInt;
    use bytes::{BufMut, Bytes, BytesMut};
    use std::assert_matches;
    use tokio_util::codec::Encoder;

    #[test]
    fn zero_length_packet() {
        let mut encoder = MinecraftPacketEncoder::new();

        let test_data: RawPacket = RawPacket::new(0, BytesMut::new());

        let mut test_output = BytesMut::new();
        encoder.encode(test_data, &mut test_output).unwrap();
        assert_eq!(test_output, vec![0x01, 0x0]);
    }

    #[test]
    fn well_sized_packet() {
        let mut encoder = MinecraftPacketEncoder::new();

        let mut payload = BytesMut::new();
        payload.put_bytes(0x10, 15);
        let test_data: RawPacket = RawPacket::new(16, payload);

        let mut test_output = BytesMut::new();
        encoder.encode(test_data, &mut test_output).unwrap();
        assert_eq!(test_output, vec![0x10; 17]);
    }

    #[test]
    fn repeated_well_sized_packets() {
        let mut encoder = MinecraftPacketEncoder::new();

        let mut payload = BytesMut::new();
        payload.put_bytes(0x10, 15);
        let test_data: RawPacket = RawPacket::new(16, payload);

        let mut test_output = BytesMut::new();
        encoder.encode(test_data, &mut test_output).unwrap();

        let mut payload = BytesMut::new();
        payload.put_bytes(0x11, 16);
        let test_data: RawPacket = RawPacket::new(17, payload);

        encoder.encode(test_data, &mut test_output).unwrap();

        let mut expected_result = vec![0x10; 17];
        expected_result.extend(vec![0x11; 18]);
        assert_eq!(test_output, expected_result);
    }

    // vec![0x008606096c6f63616c686f73741ec601]
}
