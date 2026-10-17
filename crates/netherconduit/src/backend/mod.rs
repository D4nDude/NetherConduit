use async_trait::async_trait;
use netherconduit_core::packet::{
    RawPacket,
    stream::{DecodeError, EncodeError},
};

mod login;
mod server;
pub use server::ProxyServerBackend;

#[allow(dead_code)]
#[derive(Debug)]
pub(crate) enum ProxyBackendError {
    FailedToInit(String),
    IOError(std::io::Error),
    InvalidState(String),
    EncodeError(EncodeError),
    DecodeError(DecodeError),
}

impl From<std::io::Error> for ProxyBackendError {
    fn from(value: std::io::Error) -> Self {
        ProxyBackendError::IOError(value)
    }
}

impl From<EncodeError> for ProxyBackendError {
    fn from(value: EncodeError) -> Self {
        ProxyBackendError::EncodeError(value)
    }
}

#[allow(unused)]
#[async_trait]
pub(crate) trait ProxyBackend: Send + Sync {
    async fn init(&mut self) -> Result<(), ProxyBackendError>;
    /// Send a packet to the Backend
    async fn send_packet(&mut self, packet: RawPacket) -> Result<(), ProxyBackendError>;
    /// Recieve a packet from the backend to send to the client
    async fn read_packet(&mut self) -> Result<Option<RawPacket>, ProxyBackendError>;
    async fn shutdown(&mut self) -> Result<(), ProxyBackendError>;
}
