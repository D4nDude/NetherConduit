use std::io::Write;

use bytes::{BufMut, BytesMut};
use flate2::Compression;
use flate2::write::ZlibEncoder;
use tokio_util::codec::Encoder;

use crate::packet::RawPacket;
use crate::packet::primitives::VarInt;
use crate::packet::stream::{EncodeError, RawPacketEncodable};

#[derive(Debug, Default, PartialEq, Eq)]
pub struct MinecraftPacketEncoder {
    compression_threshold: Option<usize>,
}

impl MinecraftPacketEncoder {
    pub fn new(compression_threshold: Option<usize>) -> MinecraftPacketEncoder {
        MinecraftPacketEncoder {
            compression_threshold,
        }
    }

    pub fn set_compression(&mut self, compression_threshold: Option<usize>) {
        self.compression_threshold = compression_threshold;
    }
}

impl Encoder<RawPacket> for MinecraftPacketEncoder {
    type Error = EncodeError;

    fn encode(&mut self, item: RawPacket, dst: &mut BytesMut) -> Result<(), Self::Error> {
        match self.compression_threshold {
            None => {
                VarInt::from_usize(item.len())?.encode(dst)?;
                dst.extend_from_slice(item.raw_data());
                Ok(())
            }
            Some(threshold) => {
                let data = item.data()?;
                if data.len() < threshold {
                    // UnCompressed packet length
                    VarInt::from_usize(1 + item.len())?.encode(dst)?;
                    dst.put_u8(0u8);
                    dst.extend_from_slice(data.as_ref());
                    Ok(())
                } else {
                    let mut e = ZlibEncoder::new(Vec::new(), Compression::default());
                    e.write_all(data.as_ref())?;
                    let compressed_data = e.finish()?;
                    let compressed_data_len = VarInt::from_usize(compressed_data.len())?;

                    // Compressed packet length
                    VarInt::from_usize(
                        compressed_data_len.get_encoded_length() + compressed_data.len(),
                    )?
                    .encode(dst)?;
                    // Length of uncompressed packet
                    VarInt::from_usize(item.len())?.encode(dst)?;
                    // Compressed packet
                    dst.extend_from_slice(&compressed_data);
                    Ok(())
                }
            }
        }
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
        let mut encoder = MinecraftPacketEncoder::new(None);

        let test_data: RawPacket = RawPacket::construct(0, Bytes::new());

        let mut test_output = BytesMut::new();
        encoder.encode(test_data, &mut test_output).unwrap();
        assert_eq!(test_output, vec![0x01, 0x0]);
    }

    #[test]
    fn well_sized_packet() {
        let mut encoder = MinecraftPacketEncoder::new(None);

        let mut payload = BytesMut::new();
        payload.put_bytes(0x10, 15);
        let test_data: RawPacket = RawPacket::construct(16, payload.freeze());

        let mut test_output = BytesMut::new();
        encoder.encode(test_data, &mut test_output).unwrap();
        assert_eq!(test_output, vec![0x10; 17]);
    }

    #[test]
    fn repeated_well_sized_packets() {
        let mut encoder = MinecraftPacketEncoder::new(None);

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
}
