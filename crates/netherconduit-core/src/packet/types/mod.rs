pub mod handshake;
pub mod login;
mod status;

pub use handshake::HandshakePacket;
pub use status::PingRequestPacket;

use crate::{
    packet::{
        RawPacket,
        stream::{DecodeError, EncodeError},
    },
    server::protocol_version::ConnectionProtocolVersion,
};

pub trait MinecraftPacket: Sized {
    fn from_raw(raw_packet: &RawPacket) -> Result<Self, DecodeError>;
    fn into_raw(self) -> Result<RawPacket, EncodeError>;
    // fn packet_type() -> MinecraftPacketType;
    fn get_id(protocol_version: ConnectionProtocolVersion) -> u32;
}

#[cfg(test)]
mod test {
    use bytes::BytesMut;
    use tokio_util::codec::{Decoder, Encoder};

    use crate::packet::{
        RawPacket,
        stream::{decoder::MinecraftPacketDecoder, encoder::MinecraftPacketEncoder},
    };

    #[macro_export]
    macro_rules! all_round_trip_tests {
        ( $name:ident, $generator:path ) => {
            paste! {
                #[test_log::test]
                fn [<$name:snake _decode_then_encode>]() {
                    let packet = $generator();

                    assert_packet_round_trip(packet);
                }

                #[test_log::test]
                fn [<$name:snake _framed_no_compression>]() {
                    let packet = $generator();

                    assert_packet_frame_unframe(packet, None);
                }

                #[test_log::test]
                fn [<$name:snake _framed_under_compression_threshold>]() {
                    let packet = $generator();

                    assert_packet_frame_unframe(packet, Some(usize::MAX));
                }

                #[test_log::test]
                fn [<$name:snake _framed_over_compression_threshold>]() {
                    let packet = $generator();

                    assert_packet_frame_unframe(packet, Some(0));
                }
            }
        };
    }

    pub(super) fn assert_packet_round_trip<P>(packet: P) -> P
    where
        P: super::MinecraftPacket + PartialEq + std::fmt::Debug + Clone,
    {
        let expected = packet.clone();

        let raw = packet.into_raw().expect("packet encoding should succeed");

        let decoded = P::from_raw(&raw).expect("packet decoding should succeed");

        assert_eq!(decoded, expected);

        decoded
    }

    pub(super) fn assert_packet_frame_unframe<P>(packet: P, compresion_threshold: Option<usize>)
    where
        P: super::MinecraftPacket + PartialEq + std::fmt::Debug + Clone,
    {
        let expected = packet.clone();

        let raw = packet.into_raw().expect("packet encoding should succeed");

        let uncompressed_raw = encode_decode(raw, compresion_threshold);

        let decoded = P::from_raw(&uncompressed_raw).expect("packet decoding should succeed");

        assert_eq!(
            decoded, expected,
            "Packet not Framed Correctly, Compression: {compresion_threshold:?}"
        );
    }

    fn encode_decode(packet: RawPacket, compresion_threshold: Option<usize>) -> RawPacket {
        let mut encoder = MinecraftPacketEncoder::new(compresion_threshold);
        let mut decoder = MinecraftPacketDecoder::new(compresion_threshold);

        let mut buffer = BytesMut::new();

        match encoder.encode(packet, &mut buffer) {
            Ok(_) => (),
            Err(e) => panic!("Error while encoding: {e}"),
        };

        let output_packet = match decoder.decode(&mut buffer) {
            Ok(x) => match x {
                Some(value) => value,
                None => {
                    panic!("Decoder returned nothing on {buffer:?}")
                }
            },
            Err(e) => panic!("Error while encoding: {e}"),
        };
        assert!(buffer.is_empty(), "Decoder did not consume entire buffer");
        output_packet
    }
}
