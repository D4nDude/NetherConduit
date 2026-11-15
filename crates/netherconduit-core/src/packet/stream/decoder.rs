use std::num::TryFromIntError;

use bytes::BytesMut;
use thiserror::Error;
use tokio_util::codec::Decoder;

use crate::packet::RawPacket;
use crate::packet::primitives::VarInt;
use crate::packet::stream::DecodeError;
use crate::packet::stream::decoder::MinecraftPacketDecoderError::{
    PacketLengthInvalid, PacketLengthNegative, PacketLengthTooLong,
};

const MAX_PACKET_SIZE: usize = (1 << 21) - 1; // 2097151 bytes

#[derive(Debug, Error)]
pub enum MinecraftPacketDecoderError {
    #[error("IO Error: {0}")]
    IOError(#[from] std::io::Error),
    #[error("Packet Decode Error: {0}")]
    DecodeError(#[from] DecodeError),
    #[error("Packet Length is invalid value: {0}")]
    InvalidPacketLength(String),
    #[error("Packet Length Negative: {0}")]
    PacketLengthNegative(i32),
    #[error("Packet length could not be converted to usize: {0}")]
    PacketLengthInvalid(TryFromIntError),
    #[error("Packet Length is defined too long: {{{0} > {MAX_PACKET_SIZE}}}")]
    PacketLengthTooLong(usize),
}

#[derive(Debug, Default, PartialEq, Eq)]
pub struct MinecraftPacketDecoder {
    compression_threshold: Option<usize>,
}

impl MinecraftPacketDecoder {
    pub fn new(compression_threshold: Option<usize>) -> MinecraftPacketDecoder {
        MinecraftPacketDecoder {
            compression_threshold,
        }
    }

    pub fn set_compression(&mut self, compression_threshold: Option<usize>) {
        self.compression_threshold = compression_threshold;
    }
}

impl Decoder for MinecraftPacketDecoder {
    type Item = RawPacket;
    type Error = MinecraftPacketDecoderError;

    fn decode(&mut self, src: &mut BytesMut) -> Result<Option<Self::Item>, Self::Error> {
        let (packet_length, int_size) = match VarInt::decode_bounded::<3>(src) {
            Ok(value) => value,
            Err(DecodeError::Incomplete) => {
                #[cfg(feature = "trace-decoder")]
                log::trace!("Not enough data for a VarInt decode");
                return Ok(None);
            } // not enough data for a varint
            Err(DecodeError::Invalid(error)) => {
                return Err(MinecraftPacketDecoderError::InvalidPacketLength(error));
            }
            Err(DecodeError::TooLong) => {
                return Err(MinecraftPacketDecoderError::InvalidPacketLength(
                    "Too long".to_string(),
                ));
            }
            Err(DecodeError::IOError(e)) => return Err(MinecraftPacketDecoderError::IOError(e)),
        };

        if packet_length.value() < 0 {
            return Err(PacketLengthNegative(packet_length.value()));
        }

        // check length is not negative
        let packet_length = match usize::try_from(packet_length) {
            Ok(size) => size,
            Err(error) => {
                return Err(PacketLengthInvalid(error));
            }
        };

        // check packet is not too long
        if packet_length > MAX_PACKET_SIZE {
            return Err(PacketLengthTooLong(packet_length));
        }

        let full_packet_length = packet_length + int_size;

        // incomplete packet
        if src.len() < full_packet_length {
            #[cfg(feature = "trace-decoder")]
            log::trace!(
                "Buffer not long enough to decode packet: {} < {full_packet_length}",
                src.len()
            );
            src.reserve(full_packet_length);
            return Ok(None);
        };

        let data = src.split_to(full_packet_length).split_off(int_size);

        if self.compression_threshold.is_some() {
            Ok(Some(RawPacket::new_compressed(data.freeze())))
        } else {
            Ok(Some(RawPacket::new(data.freeze())))
        }
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
        let mut decoder = MinecraftPacketDecoder::new(None);
        let mut test_data: BytesMut = BytesMut::new();
        let output = decoder.decode(&mut test_data).unwrap();
        assert_matches!(output, None);
    }

    #[test]
    fn well_sized_packet() {
        let mut decoder = MinecraftPacketDecoder::new(None);
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
        let mut decoder = MinecraftPacketDecoder::new(None);
        let mut test_data: BytesMut = BytesMut::from(&[0x10; 16][..]);
        let output = decoder.decode(&mut test_data).unwrap();
        assert_matches!(output, None);
    }

    #[test]
    fn repeated_well_sized_packets() {
        let mut decoder = MinecraftPacketDecoder::new(None);

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
        let mut decoder = MinecraftPacketDecoder::new(None);

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
