use std::fmt::Display;

use super::{MinecraftPacket, MinecraftPacketType};
use crate::packet::builder::RawPacketBuilder;
use crate::packet::stream::DecodeError;
use crate::packet::{RawPacket, decoder::RawPacketDecoder};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PingRequestPacket {
    pub payload: i64,
}

impl MinecraftPacket for PingRequestPacket {
    fn from_raw(raw_packet: &RawPacket) -> Result<Self, DecodeError> {
        let mut parser: RawPacketDecoder = RawPacketDecoder::new(raw_packet.payload()?);
        let payload = parser.long()?;
        Ok(Self { payload })
    }

    fn into_raw(self) -> Result<RawPacket, crate::packet::stream::EncodeError> {
        Ok(RawPacketBuilder::new(0)?.long(self.payload)?.build())
    }

    fn packet_type() -> MinecraftPacketType {
        MinecraftPacketType::PingRequest
    }
}

impl Display for PingRequestPacket {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "(PingRequestPacket Packet: {})", self.payload)
    }
}
