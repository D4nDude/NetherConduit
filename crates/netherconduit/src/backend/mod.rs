use netherconduit_core::packet::RawPacket;
use tokio::sync::mpsc::{Receiver, Sender};

mod server;
pub use server::ProxyServerBackend;
use tokio_util::sync::CancellationToken;

use crate::connection::ConnectionHandle;

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

pub struct ProxyBackendHandle {
    pub _incoming: Receiver<RawPacket>,
    pub outgoing: Sender<RawPacket>,
    _shutdown_token: CancellationToken,
}

impl From<ConnectionHandle> for ProxyBackendHandle {
    fn from(value: ConnectionHandle) -> Self {
        let ConnectionHandle {
            incoming,
            outgoing,
            shutdown_token,
        } = value;
        ProxyBackendHandle {
            _incoming: incoming,
            outgoing,
            _shutdown_token: shutdown_token,
        }
    }
}

#[allow(unused)]
pub(crate) trait ProxyBackend: Send + Sync {
    fn init(&mut self) -> Result<ProxyBackendHandle, ProxyBackendError>;
    // fn send_packet(&self, packet: RawPacket) -> Result<(), ProxyBackendError>;
    // fn pull_packet(&mut self) -> Result<Option<RawPacket>, ProxyBackendError>;
    fn shutdown(&mut self) -> Result<(), ProxyBackendError>;
}
