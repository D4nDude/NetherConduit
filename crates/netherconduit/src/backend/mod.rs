use std::num::TryFromIntError;

use netherconduit_core::packet::{
    RawPacket,
    primitives::GameProfile,
    stream::{DecodeError, EncodeError},
};
use thiserror::Error;

// mod login;
mod server;
use crate::connection::TcpConnectionError;
pub use server::ProxyServerBackend;

#[allow(dead_code)]
#[derive(Debug, Error)]
pub(crate) enum ProxyBackendError {
    #[error("Failed to init backend: {0}")]
    FailedToInit(String),
    #[error("IO Error: {0}")]
    IOError(#[from] std::io::Error),
    #[error("Invalid state: {0}")]
    InvalidState(String),
    #[error("Packet Encode Error: {0}")]
    EncodeError(#[from] EncodeError),
    #[error("Packet Decode Error: {0}")]
    DecodeError(#[from] DecodeError),
    #[error("Tcp Error: {0}")]
    TcpError(#[from] TcpConnectionError),
    #[error("Try From Int Error Error: {0}")]
    TryFromIntError(#[from] TryFromIntError),
}

#[allow(unused)]
pub(crate) trait ProxyBackend: Send + Sync {
    async fn init(&mut self, player_profile: &GameProfile) -> Result<(), ProxyBackendError>;
    /// Send a packet to the Backend
    async fn send_packet(&mut self, packet: RawPacket) -> Result<(), ProxyBackendError>;
    /// Recieve a packet from the backend to send to the client
    async fn read_packet(&mut self) -> Result<RawPacket, ProxyBackendError>;
    async fn shutdown(&mut self) -> Result<(), ProxyBackendError>;
}
