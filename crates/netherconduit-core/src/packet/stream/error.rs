use std::num::TryFromIntError;

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

#[derive(Debug, PartialEq, Eq, Hash, Clone, Copy)]
pub enum EncodeError {
    Invalid,
}

impl From<TryFromIntError> for EncodeError {
    fn from(_value: TryFromIntError) -> Self {
        EncodeError::Invalid
    }
}
