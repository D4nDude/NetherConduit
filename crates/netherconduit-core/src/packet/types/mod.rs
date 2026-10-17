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

    pub(super) fn assert_packet_round_trip<P>(packet: P)
    where
        P: super::MinecraftPacket + PartialEq + std::fmt::Debug + Clone,
    {
        let expected = packet.clone();

        let raw = packet.into_raw().expect("packet encoding should succeed");

        let decoded = P::from_raw(&raw).expect("packet decoding should succeed");

        assert_eq!(decoded, expected);
    }
}
