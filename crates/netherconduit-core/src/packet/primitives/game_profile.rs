use std::fmt::Display;

use uuid::Uuid;

use crate::packet::stream::{DecodeError, EncodeError, RawPacketDecodable, RawPacketEncodable};

#[derive(Debug, Clone, PartialEq, Eq, Hash, Default)]
pub struct GameProfile {
    pub uuid: Uuid,
    pub username: String,
    pub properties: Vec<Property>,
}

impl GameProfile {
    pub fn new(uuid: Uuid, username: &str, properties: &[Property]) -> Self {
        GameProfile {
            uuid,
            username: username.to_string(),
            properties: Vec::from(properties),
        }
    }
}

impl RawPacketDecodable for GameProfile {
    fn decode(buffer: &[u8]) -> Result<(Self, usize), DecodeError> {
        let (uuid, cursor) = Uuid::decode(buffer)?;
        let (username, username_size) = String::decode(
            buffer
                .get(cursor..)
                .ok_or(DecodeError::Invalid("Username string invalid".to_string()))?,
        )?;
        let cursor = cursor + username_size;
        let (props, props_len) = Vec::<Property>::decode(
            buffer
                .get(cursor..)
                .ok_or(DecodeError::Invalid("Invalid properties".to_string()))?,
        )?;
        Ok((
            GameProfile {
                uuid,
                username,
                properties: props,
            },
            cursor + props_len,
        ))
    }
}

impl RawPacketEncodable for GameProfile {
    fn encode(&self, buffer: &mut bytes::BytesMut) -> Result<usize, EncodeError> {
        let uuid_len = self.uuid.encode(buffer)?;
        if self.username.len() > 16 {
            return Err(EncodeError::TooLong(self.username.clone()));
        }
        let username_len = self.username.encode(buffer)?;
        if self.properties.len() > 16 {
            return Err(EncodeError::TooLong(format!("{:?}", self.properties)));
        }
        let props_len = self.properties.encode(buffer)?;
        Ok(uuid_len + username_len + props_len)
    }

    fn get_encoded_length(&self) -> usize {
        self.uuid.get_encoded_length()
            + self.username.get_encoded_length()
            + self.properties.get_encoded_length()
    }
}

impl Display for GameProfile {
    fn fmt(&self, _f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        todo!()
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Default)]
pub struct Property {
    name: String,
    value: String,
    signature: Option<String>,
}

impl Property {
    pub fn new(name: &str, value: &str, signature: Option<&str>) -> Self {
        Property {
            name: name.to_string(),
            value: value.to_string(),
            signature: signature.map(|x| x.to_string()),
        }
    }
}

impl RawPacketDecodable for Property {
    fn decode(buffer: &[u8]) -> Result<(Self, usize), DecodeError> {
        let (name, name_size) = String::decode(buffer)?;
        if name_size > 64 {
            return Err(DecodeError::Invalid("String too long".to_string()));
        }
        let (value, value_size) = String::decode(buffer)?;
        if value_size > 32767 {
            return Err(DecodeError::Invalid("String too long".to_string()));
        }
        let (signature, signature_size) = Option::<String>::decode(buffer)?;
        if signature_size > 1025 {
            return Err(DecodeError::Invalid("String too long".to_string()));
        }
        Ok((
            Property::new(&name, &value, signature.as_deref()),
            name_size + value_size + signature_size,
        ))
    }
}

impl RawPacketEncodable for Property {
    fn encode(&self, buffer: &mut bytes::BytesMut) -> Result<usize, EncodeError> {
        let mut buffer_size = self.name.encode(buffer)?;
        buffer_size += self.value.encode(buffer)?;
        buffer_size += self.signature.encode(buffer)?;
        Ok(buffer_size)
    }

    fn get_encoded_length(&self) -> usize {
        self.name.get_encoded_length()
            + self.value.get_encoded_length()
            + self.signature.get_encoded_length()
    }
}
