use async_trait::async_trait;
use netherconduit_core::packet::RawPacket;

mod server;
pub use server::ProxyServerBackend;

#[allow(dead_code)]
#[derive(Debug)]
pub(crate) enum ProxyBackendError {
    FailedToInit(String),
    IOError(std::io::Error),
    InvalidState(String),
}

impl From<std::io::Error> for ProxyBackendError {
    fn from(value: std::io::Error) -> Self {
        ProxyBackendError::IOError(value)
    }
}

#[allow(unused)]
#[async_trait]
pub(crate) trait ProxyBackend: Send + Sync {
    fn init(&mut self) -> Result<(), ProxyBackendError>;
    async fn send_packet(&mut self, packet: RawPacket) -> Result<(), ProxyBackendError>;
    async fn read_packet(&mut self) -> Result<Option<RawPacket>, ProxyBackendError>;
    async fn shutdown(&mut self) -> Result<(), ProxyBackendError>;
}
