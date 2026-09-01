use async_trait::async_trait;
use netherconduit_core::packet::{RawPacket, builder::RawPacketBuilder};
use std::net::TcpStream;

use crate::{
    backend::{ProxyBackend, ProxyBackendError},
    connection::Connection,
};

#[allow(unused)]
pub struct ProxyServerBackend {
    address: String,
    port: u16,
    connection: Option<Connection>,
}

impl ProxyServerBackend {
    pub(crate) fn new(address: &str, port: u16) -> Self {
        ProxyServerBackend {
            address: address.to_string(),
            port,
            connection: None,
        }
    }
}

#[async_trait]
impl ProxyBackend for ProxyServerBackend {
    async fn init(&mut self) -> Result<(), ProxyBackendError> {
        log::info!("Connecting to server...");
        let tcp_stream = match TcpStream::connect(format!("{}:{}", self.address, self.port)) {
            Ok(conn) => conn,
            Err(e) => {
                log::error!(
                    "Unable to connect to backed server: {}, reason: {:?}",
                    self.address,
                    e
                );
                return Err(ProxyBackendError::FailedToInit(
                    "Could not connect to server".to_string(),
                ));
            }
        };
        tcp_stream.set_nonblocking(true)?;
        let mut conn = Connection::new(tokio::net::TcpStream::from_std(tcp_stream)?);
        conn.send(RawPacketBuilder::new(0).var_int(776.into()).unwrap().string_slice("localhost").unwrap().unsigned_short(25565).unwrap().var_int(2.into()).unwrap().build()).await;
        self.connection = Some(conn);
        log::info!("Connected to backend!");
        Ok(())
    }

    async fn send_packet(&mut self, packet: RawPacket) -> Result<(), ProxyBackendError> {
        if self.connection.is_none() {
            log::warn!("sending packet without initing server!");
            self.init().await?;
        }
        match &mut self.connection {
            Some(handle) => {
                handle.send(packet).await;
                Ok(())
            }
            None => {
                log::error!("Handle terminated....");
                Err(ProxyBackendError::InvalidState(
                    "Handle Terminated".to_string(),
                ))
            }
        }
    }

    async fn read_packet(&mut self) -> Result<Option<RawPacket>, ProxyBackendError> {
        match &mut self.connection {
            Some(handle) => Ok(handle.read().await),
            None => {
                log::error!("Handle terminated....");
                Err(ProxyBackendError::InvalidState(
                    "Handle Terminated".to_string(),
                ))
            }
        }
    }

    async fn shutdown(&mut self) -> Result<(), ProxyBackendError> {
        if let Some(conn) = self.connection.take() {
            conn.shutdown().await
        }
        Ok(())
    }
}
