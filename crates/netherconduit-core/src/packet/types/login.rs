use std::fmt::Display;

use uuid::Uuid;

use super::MinecraftPacket;
use crate::packet::builder::RawPacketBuilder;
use crate::packet::primitives::GameProfile;
use crate::packet::stream::DecodeError;
use crate::packet::{RawPacket, decoder::RawPacketDecoder};
use crate::registry::packet::{PacketType, get_packet_id};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LoginStartPacket {
    pub name: String,
    pub uuid: Uuid,
}

impl MinecraftPacket for LoginStartPacket {
    fn from_raw(raw_packet: &RawPacket) -> Result<Self, DecodeError> {
        let mut parser: RawPacketDecoder = RawPacketDecoder::new(raw_packet.payload()?);
        let player_name = parser.string()?;
        if player_name.len() > 16 {
            return Err(DecodeError::Invalid(format!(
                "Player Name can only be 16 characters, found: {} (name: {player_name}",
                player_name.len()
            )));
        }
        let player_uuid: Uuid = parser.uuid()?;
        Ok(Self {
            name: player_name,
            uuid: player_uuid,
        })
    }

    fn into_raw(self) -> Result<RawPacket, crate::packet::stream::EncodeError> {
        Ok(RawPacketBuilder::new(0)?
            .string(self.name)?
            .bytes(self.uuid.as_bytes())?
            .build())
    }

    fn get_id(protocol_version: crate::server::protocol_version::ConnectionProtocolVersion) -> u32 {
        get_packet_id(protocol_version, PacketType::LoginStart)
    }
}

impl Display for LoginStartPacket {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "(Login Start Packet: {}, \"{}\")", self.name, self.uuid)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LoginSuccessPacket {
    pub profile: GameProfile,
    pub session_id: Uuid,
}

impl LoginSuccessPacket {
    pub fn new(profile: GameProfile, session_id: Uuid) -> Self {
        LoginSuccessPacket {
            profile,
            session_id,
        }
    }
}

impl MinecraftPacket for LoginSuccessPacket {
    fn from_raw(raw_packet: &RawPacket) -> Result<Self, DecodeError> {
        let mut parser: RawPacketDecoder = RawPacketDecoder::new(raw_packet.payload()?);
        let profile = parser.game_profile()?;
        let session_id: Uuid = parser.uuid()?;
        Ok(Self {
            profile,
            session_id,
        })
    }

    fn into_raw(self) -> Result<RawPacket, crate::packet::stream::EncodeError> {
        Ok(RawPacketBuilder::new(0)?
            .game_profile(self.profile)?
            .bytes(self.session_id.as_bytes())?
            .build())
    }

    fn get_id(protocol_version: crate::server::protocol_version::ConnectionProtocolVersion) -> u32 {
        get_packet_id(protocol_version, PacketType::LoginSuccess)
    }
}

impl Display for LoginSuccessPacket {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "(Login Success Packet: {}, \"{}\")",
            self.profile, self.session_id
        )
    }
}
