use std::num::TryFromIntError;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum DecodeError {
    #[error("IO Error: {0}")]
    IOError(#[from] std::io::Error),
    #[error("Decode Error: Incomplete input")]
    Incomplete,
    #[error("Data is too long")]
    TooLong,
    #[error("Decode Error: Invalid value for data type: {0}")]
    Invalid(String),
}

impl From<TryFromIntError> for DecodeError {
    fn from(value: TryFromIntError) -> Self {
        DecodeError::Invalid(value.to_string())
    }
}

#[derive(Debug, Error)]
pub enum EncodeError {
    #[error("IO Error: {0}")]
    IOError(#[from] std::io::Error),
    #[error("Encode Error: Invalid value for data type: {0}")]
    Invalid(String),
    #[error("Encode Error: Value is too long: {0}")]
    TooLong(String),
    #[error("Encode Error: Decode Error: {0}")]
    DecodeError(#[from] DecodeError),
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
