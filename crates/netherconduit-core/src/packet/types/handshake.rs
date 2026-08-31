use std::fmt::Display;

use super::{MinecraftPacket, MinecraftPacketType};
use crate::packet::stream::DecodeError;
use crate::packet::{RawPacket, decoder::RawPacketDecoder};
use crate::server::protocol_version::ConnectionProtocolVersion;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum HandshakeIntent {
    Status,
    Login,
    Transfer,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HandshakePacket {
    pub protocol_version: ConnectionProtocolVersion,
    pub server_address: String,
    pub server_port: u16,
    pub intent: HandshakeIntent,
}

impl MinecraftPacket for HandshakePacket {
    fn from_raw(raw_packet: &RawPacket) -> Result<Self, DecodeError> {
        let mut parser: RawPacketDecoder = RawPacketDecoder::new(raw_packet.payload()?);
        let protocol_version = parser.var_int()?;
        let server_address = parser.string()?;
        let server_port = parser.unsigned_short()?;
        let intent_number = parser.var_int()?;
        let intent = match intent_number.value() {
            1 => HandshakeIntent::Status,
            2 => HandshakeIntent::Login,
            3 => HandshakeIntent::Transfer,
            intent => {
                return Err(DecodeError::Invalid(format!(
                    "Invalid handshake intent: {intent}"
                )));
            }
        };
        Ok(Self {
            protocol_version: ConnectionProtocolVersion::from_var_int(protocol_version)?,
            server_address,
            server_port,
            intent,
        })
    }

    fn packet_type() -> MinecraftPacketType {
        MinecraftPacketType::Handshake
    }
}

impl Display for HandshakePacket {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "(Handshake Packet: {}, \"{}\", {}, {:?})",
            self.protocol_version, self.server_address, self.server_port, self.intent
        )
    }
}
