use netherconduit_core::packet::RawPacket;
use tokio::sync::mpsc::{Receiver, Sender};

mod server;
pub use server::ProxyServerBackend;

use crate::connection::ConnectionHandle;

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

pub struct ProxyBackendHandle {
    pub incoming: Receiver<RawPacket>,
    pub outgoing: Sender<RawPacket>,
}

impl From<ConnectionHandle> for ProxyBackendHandle {
    fn from(value: ConnectionHandle) -> Self {
        let ConnectionHandle { incoming, outgoing } = value;
        ProxyBackendHandle { incoming, outgoing }
    }
}

pub(crate) trait ProxyBackend: Send + Sync {
    fn init(&mut self) -> Result<ProxyBackendHandle, ProxyBackendError>;
    // fn send_packet(&self, packet: RawPacket) -> Result<(), ProxyBackendError>;
    // fn pull_packet(&mut self) -> Result<Option<RawPacket>, ProxyBackendError>;
    fn shutdown(&mut self) -> Result<(), ProxyBackendError>;
}
