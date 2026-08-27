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
        let packet_length = VarInt::new(i32::try_from(item.data.len()).unwrap());
        packet_length.encode(dst);
        dst.extend(item.data);
        Ok(())
    }
}

#[cfg(test)]
mod test {
    use super::MinecraftPacketEncoder;
    use crate::packet::RawPacket;
    use bytes::{BufMut, Bytes, BytesMut};
    use tokio_util::codec::Encoder;

    #[test]
    fn zero_length_packet() {
        let mut encoder = MinecraftPacketEncoder::new();

        let test_data: RawPacket = RawPacket::construct(0, Bytes::new());

        let mut test_output = BytesMut::new();
        encoder.encode(test_data, &mut test_output).unwrap();
        assert_eq!(test_output, vec![0x01, 0x0]);
    }

    #[test]
    fn well_sized_packet() {
        let mut encoder = MinecraftPacketEncoder::new();

        let mut payload = BytesMut::new();
        payload.put_bytes(0x10, 15);
        let test_data: RawPacket = RawPacket::construct(16, payload.freeze());

        let mut test_output = BytesMut::new();
        encoder.encode(test_data, &mut test_output).unwrap();
        assert_eq!(test_output, vec![0x10; 17]);
    }

    #[test]
    fn repeated_well_sized_packets() {
        let mut encoder = MinecraftPacketEncoder::new();

        let mut payload = BytesMut::new();
        payload.put_bytes(0x10, 15);
        let test_data: RawPacket = RawPacket::construct(16, payload.freeze());

        let mut test_output = BytesMut::new();
        encoder.encode(test_data, &mut test_output).unwrap();

        let mut payload = BytesMut::new();
        payload.put_bytes(0x11, 16);
        let test_data: RawPacket = RawPacket::construct(17, payload.freeze());

        encoder.encode(test_data, &mut test_output).unwrap();

        let mut expected_result = vec![0x10; 17];
        expected_result.extend(vec![0x11; 18]);
        assert_eq!(test_output, expected_result);
    }

    // vec![0x008606096c6f63616c686f73741ec601]
}
