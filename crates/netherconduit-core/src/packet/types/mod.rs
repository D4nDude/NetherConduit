pub mod handshake;
mod status;

pub use handshake::HandshakePacket;
pub use status::PingRequestPacket;

use crate::packet::{RawPacket, stream::{DecodeError, EncodeError}};

pub trait MinecraftPacket: Sized {
    fn from_raw(raw_packet: &RawPacket) -> Result<Self, DecodeError>;
    fn into_raw(self) -> Result<RawPacket, EncodeError>;
    fn packet_type() -> MinecraftPacketType;
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum MinecraftPacketType {
    Handshake,
    PingRequest,
}

pub enum Packet {
    Handshake(HandshakePacket),
}
