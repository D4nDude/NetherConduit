use std::fmt::Display;

use uuid::Uuid;

use super::MinecraftPacket;
use crate::packet::builder::RawPacketBuilder;
use crate::packet::primitives::{GameProfile, VarInt};
use crate::packet::stream::DecodeError;
use crate::packet::{RawPacket, decoder::RawPacketDecoder};
use crate::registry::packet::{PacketType, get_packet_id};

//------------------------------------------------------------------------------
//         ServerBound
//------------------------------------------------------------------------------

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

impl LoginStartPacket {
    pub fn new(game_profile: &GameProfile) -> Self {
        LoginStartPacket {
            name: game_profile.username.clone(),
            uuid: game_profile.uuid,
        }
    }

    pub fn deconstruct(self) -> (String, Uuid) {
        (self.name, self.uuid)
    }

    pub fn into_game_profile(self) -> GameProfile {
        GameProfile::new(self.uuid, &self.name, &[])
    }
}

//------------------------------------------------------------------------------
//         ClientBound
//------------------------------------------------------------------------------

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LoginDisconnectPacket {
    pub reason: String, //TODO: JSONTextComponent
}

impl LoginDisconnectPacket {
    pub fn new(reason: &str) -> Self {
        LoginDisconnectPacket {
            reason: reason.to_string(),
        }
    }
}

impl MinecraftPacket for LoginDisconnectPacket {
    fn from_raw(raw_packet: &RawPacket) -> Result<Self, DecodeError> {
        let mut parser: RawPacketDecoder = RawPacketDecoder::new(raw_packet.payload()?);
        let reason = parser.string()?;
        Ok(Self { reason })
    }

    fn into_raw(self) -> Result<RawPacket, crate::packet::stream::EncodeError> {
        Ok(RawPacketBuilder::new(0)?.string(self.reason)?.build())
    }

    fn get_id(protocol_version: crate::server::protocol_version::ConnectionProtocolVersion) -> u32 {
        get_packet_id(protocol_version, PacketType::LoginSuccess)
    }
}

impl Display for LoginDisconnectPacket {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "(Login Disconnect Packet: \"{}\")", self.reason,)
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
        Ok(RawPacketBuilder::new(2)?
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

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LoginSetCompressionPacket {
    pub threshold: VarInt,
}

impl LoginSetCompressionPacket {
    pub fn new(threshold: VarInt) -> Self {
        LoginSetCompressionPacket { threshold }
    }
}

impl MinecraftPacket for LoginSetCompressionPacket {
    fn from_raw(raw_packet: &RawPacket) -> Result<Self, DecodeError> {
        let mut parser: RawPacketDecoder = RawPacketDecoder::new(raw_packet.payload()?);
        let threshold = parser.var_int()?;
        Ok(Self { threshold })
    }

    fn into_raw(self) -> Result<RawPacket, crate::packet::stream::EncodeError> {
        Ok(RawPacketBuilder::new(2)?.var_int(self.threshold)?.build())
    }

    fn get_id(protocol_version: crate::server::protocol_version::ConnectionProtocolVersion) -> u32 {
        get_packet_id(protocol_version, PacketType::LoginSuccess)
    }
}

impl Display for LoginSetCompressionPacket {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "(Login Set Compression Packet: {})", self.threshold)
    }
}

#[cfg(test)]
mod test {

    use uuid::Uuid;

    use super::super::test::assert_packet_frame_unframe;
    use super::super::test::assert_packet_round_trip;
    use crate::packet::primitives::VarInt;

    #[test_log::test]
    fn login_start_packet_round_trip() {
        use super::LoginStartPacket;
        use crate::packet::primitives::GameProfile;

        let profile = GameProfile::new(Uuid::new_v4(), "TestUsername", &[]);

        let packet = LoginStartPacket::new(&profile);

        let new_packet = assert_packet_round_trip(packet);

        assert_eq!(profile, new_packet.into_game_profile())
    }

    #[test_log::test]
    fn login_start_packet_framed_round_trip() {
        use super::LoginStartPacket;
        use crate::packet::primitives::GameProfile;

        let profile = GameProfile::new(Uuid::new_v4(), "TestUsername", &[]);

        let packet = LoginStartPacket::new(&profile);

        assert_packet_frame_unframe(packet);
    }

    #[test]
    fn login_disconnect_packet_round_trip() {
        use super::LoginDisconnectPacket;

        let packet = LoginDisconnectPacket::new("Could not connect to Server.");

        assert_packet_round_trip(packet);
    }

    #[test]
    fn login_set_compression_packet_round_trip() {
        use super::LoginSetCompressionPacket;

        let packet = LoginSetCompressionPacket::new(256.into());

        let new_packet = assert_packet_round_trip(packet);

        assert_eq!(VarInt::new(256), new_packet.threshold);
    }

    #[test]
    fn login_success_packet_round_trip() {
        use super::LoginSuccessPacket;
        use crate::packet::primitives::GameProfile;

        let profile = GameProfile::new(Uuid::new_v4(), "TestUsername", &[]);

        let packet = LoginSuccessPacket::new(profile, Uuid::new_v4());

        assert_packet_round_trip(packet);
    }
}
