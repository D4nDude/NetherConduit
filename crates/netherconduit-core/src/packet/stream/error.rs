use std::num::TryFromIntError;

#[derive(Debug, PartialEq, Eq, Hash, Clone, Copy)]
pub enum DecodeError {
    Incomplete,
    Invalid,
}

impl From<TryFromIntError> for DecodeError {
    fn from(_value: TryFromIntError) -> Self {
        DecodeError::Invalid
    }
}

impl From<DecodeError> for std::io::Error {
    fn from(value: DecodeError) -> Self {
        match value {
            DecodeError::Incomplete => std::io::Error::new(
                std::io::ErrorKind::UnexpectedEof,
                "Not enough data to create value",
            ),
            DecodeError::Invalid => std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                "Invalid data format for value",
            ),
        }
    }
}