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

    pub(super) fn assert_packet_frame_unframe<P>(packet: P) -> P
    where
        P: super::MinecraftPacket + PartialEq + std::fmt::Debug + Clone,
    {
        let expected = packet.clone();

        let raw = packet.into_raw().expect("packet encoding should succeed");

        log::info!("Testing encode/decode with compression unset.");
        let uncompressed_raw = encode_decode(raw, None);

        let decoded = P::from_raw(&uncompressed_raw).expect("packet decoding should succeed");

        assert_eq!(
            decoded, expected,
            "Packet not Framed Correctly, Compression: None"
        );

        log::info!("Testing encode/decode with compression set to max threshold.");
        let not_compressed_raw = encode_decode(uncompressed_raw, Some(usize::MAX));

        let decoded = P::from_raw(&not_compressed_raw).expect("packet decoding should succeed");

        assert_eq!(
            decoded, expected,
            "Packet not Framed Correctly, Compression: Some(inf)"
        );

        log::info!("Testing encode/decode with compression set to min threshold.");
        let compressed_raw = encode_decode(not_compressed_raw, Some(1));

        let decoded = P::from_raw(&compressed_raw).expect("packet decoding should succeed");

        assert_eq!(
            decoded, expected,
            "Packet not Framed Correctly, Compression: Some(1)"
        );

        decoded
    }

    fn encode_decode(packet: RawPacket, compresion_threshold: Option<usize>) -> RawPacket {
        let mut encoder = MinecraftPacketEncoder::new(compresion_threshold);
        let mut decoder = MinecraftPacketDecoder::new(compresion_threshold);

        let mut buffer = BytesMut::new();

        match encoder.encode(packet, &mut buffer) {
            Ok(_) => (),
            Err(e) => panic!("Error while encoding: {e}"),
        };

        match decoder.decode(&mut buffer) {
            Ok(x) => match x {
                Some(value) => value,
                None => {
                    panic!("Decoder returned nothing on {buffer:?}")
                }
            },
            Err(e) => panic!("Error while encoding: {e}"),
        }
    }
}
