use std::fmt::Display;

use uuid::Uuid;

use crate::packet::stream::{DecodeError, EncodeError, RawPacketDecodable, RawPacketEncodable};

#[derive(Debug, Clone, PartialEq, Eq, Hash, Default)]
pub struct GameProfile {
    pub uuid: Uuid,
    username: String,
}

impl GameProfile {
    pub fn new(uuid: Uuid, username: &str) -> Self {
        GameProfile {
            uuid,
            username: username.to_string(),
        }
    }
}

impl RawPacketDecodable for GameProfile {
    fn decode(buffer: &[u8]) -> Result<(Self, usize), DecodeError> {
        todo!()
    }
}

impl RawPacketEncodable for GameProfile {
    fn encode(&self, buffer: &mut bytes::BytesMut) -> Result<usize, EncodeError> {
        todo!()
    }

    fn get_encoded_length(&self) -> usize {
        todo!()
    }
}

impl Display for GameProfile {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        todo!()
    }
}
