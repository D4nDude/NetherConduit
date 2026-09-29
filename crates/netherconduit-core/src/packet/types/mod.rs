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
