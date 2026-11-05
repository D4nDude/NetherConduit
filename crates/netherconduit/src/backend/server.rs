use netherconduit_core::packet::{
    RawPacket,
    builder::RawPacketBuilder,
    primitives::GameProfile,
    types::{
        HandshakePacket, MinecraftPacket,
        login::{LoginSetCompressionPacket, LoginStartPacket, LoginSuccessPacket},
    },
};
use tokio::net::TcpStream;
use uuid::Uuid;

use crate::{
    backend::{ProxyBackend, ProxyBackendError},
    connection::TcpConnection,
};

#[allow(unused)]
pub struct ProxyServerBackend {
    address: String,
    port: u16,
    connection: Option<TcpConnection>,
    session_id: Option<Uuid>,
}

impl ProxyServerBackend {
    pub(crate) fn new(address: &str, port: u16) -> Self {
        ProxyServerBackend {
            address: address.to_string(),
            port,
            connection: None,
            session_id: None,
        }
    }
}

impl ProxyBackend for ProxyServerBackend {
    async fn init(&mut self, player_profile: &GameProfile) -> Result<(), ProxyBackendError> {
        log::info!("Connecting to server...");
        let tcp_stream = match TcpStream::connect(format!("{}:{}", self.address, self.port)).await {
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
        // tcp_stream.set_nonblocking(true)?;
        let mut conn = TcpConnection::new(tcp_stream);
        let handshake_info = HandshakePacket::new(
            netherconduit_core::server::protocol_version::ConnectionProtocolVersion::MC776,
            "localhost",
            25565,
            netherconduit_core::packet::types::handshake::HandshakeIntent::Login,
        );
        log::debug!("Sending Handshake: {:?}", handshake_info);
        conn.send(handshake_info.into_raw()?).await;
        self.connection = Some(conn);
        log::info!("Connected to backend!");
        // Login!

        let login_start_packet = LoginStartPacket::new(player_profile);
        self.send_packet(login_start_packet.into_raw()?).await?;
        let server_ack = self.read_packet().await?;
        if server_ack.id()?.value() == 3 {
            let set_compression_packet = LoginSetCompressionPacket::from_raw(&server_ack)?;
            log::info!("Server sent: {:?}", set_compression_packet,);
            if let Some(ref mut conn) = self.connection {
                conn.set_compression(Some(set_compression_packet.threshold.value_usize()?));
            }
        }
        let server_ack = self.read_packet().await?;
        if server_ack.id()?.value() != 2 {
            return Err(ProxyBackendError::FailedToInit(format!(
                "Server did not accept Login, got: {server_ack}"
            )));
        }

        log::debug!("Server_ack: {}", server_ack);
        let server_ack_packet = LoginSuccessPacket::from_raw(&server_ack)?;
        self.session_id = Some(server_ack_packet.session_id);

        self.send_packet(RawPacketBuilder::new(3)?.build()).await?;

        Ok(())
    }

    async fn send_packet(&mut self, packet: RawPacket) -> Result<(), ProxyBackendError> {
        if self.connection.is_none() {
            log::warn!("sending packet without initing server!");
            return Err(ProxyBackendError::InvalidState(
                "Connection not started".to_string(),
            ));
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

    async fn read_packet(&mut self) -> Result<RawPacket, ProxyBackendError> {
        match &mut self.connection {
            Some(handle) => Ok(handle.read().await?),
            None => {
                log::error!("Handle terminated....");
                Err(ProxyBackendError::InvalidState(
                    "Handle Terminated".to_string(),
                ))
            }
        }
    }

    async fn shutdown(&mut self) -> Result<(), ProxyBackendError> {
        if let Some(mut conn) = self.connection.take() {
            conn.shutdown().await
        }
        Ok(())
    }
}
