use std::fmt::Display;

use serde::ser::{Serialize, SerializeStruct};

use crate::packet::{primitives::VarInt, stream::DecodeError};

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone, Copy, Hash, Default)]
pub enum ConnectionProtocolVersion {
    #[default]
    MC776 = 776,
    MC772 = 772,
}

impl ConnectionProtocolVersion {
    pub fn from_var_int(version: VarInt) -> Result<Self, DecodeError> {
        Ok(match version.value() {
            776 => ConnectionProtocolVersion::MC776,
            772 => ConnectionProtocolVersion::MC772,
            x => {
                return Err(DecodeError::Invalid(format!(
                    "Invalid protocol versino: {x}"
                )));
            }
        })
    }

    pub fn name(self) -> &'static str {
        match self {
            ConnectionProtocolVersion::MC776 => "26.2",
            ConnectionProtocolVersion::MC772 => "1.21.8",
        }
    }

    pub fn protocol(self) -> u32 {
        self as u32
    }
}

impl Display for ConnectionProtocolVersion {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl Serialize for ConnectionProtocolVersion {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let mut state = serializer.serialize_struct("version", 2)?;
        state.serialize_field("name", &self.name())?;
        state.serialize_field("protocol", &self.protocol())?;
        state.end()
    }
}
