use std::{error::Error, fmt::Display, num::TryFromIntError};

#[derive(Debug, PartialEq, Eq, Hash, Clone)]
pub enum DecodeError {
    Incomplete,
    Invalid(String),
}

impl From<TryFromIntError> for DecodeError {
    fn from(value: TryFromIntError) -> Self {
        DecodeError::Invalid(value.to_string())
    }
}

impl From<DecodeError> for std::io::Error {
    fn from(value: DecodeError) -> Self {
        match value {
            DecodeError::Incomplete => std::io::Error::new(
                std::io::ErrorKind::UnexpectedEof,
                "Not enough data to create value",
            ),
            DecodeError::Invalid(cause) => std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                format!("Invalid data format for value: {cause}"),
            ),
        }
    }
}

impl Display for DecodeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match &self {
            DecodeError::Incomplete => write!(f, "Decode Error: Incomplete input"),
            DecodeError::Invalid(value) => {
                write!(f, "Decode Error: Invalid value for data type: {}", value)
            }
        }
    }
}

impl Error for DecodeError {}

#[derive(Debug, PartialEq, Eq, Hash, Clone)]
pub enum EncodeError {
    Invalid(String),
}

impl From<TryFromIntError> for EncodeError {
    fn from(value: TryFromIntError) -> Self {
        EncodeError::Invalid(format!("TryFromIntError: {value}"))
    }
}

impl From<serde_json::Error> for EncodeError {
    fn from(value: serde_json::Error) -> Self {
        EncodeError::Invalid(format!("JSON Serialisation Error: {value}"))
    }
}
