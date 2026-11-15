use std::error::Error;

use log::error;
use netherconduit_core::{
    connection::ConnectionState,
    packet::{
        RawPacket,
        builder::RawPacketBuilder,
        primitives::GameProfile,
        stream::{DecodeError, EncodeError},
        types::{MinecraftPacket, login::LoginDisconnectPacket},
    },
    server::status::ServerStatus,
};
use thiserror::Error;
use tokio::{net::TcpStream, sync::watch::Receiver};
use uuid::Uuid;

use crate::{
    backend::{ProxyBackend, ProxyServerBackend},
    connection::{
        TcpConnection, TcpConnectionError,
        handshake::{HandshakeResult, handle_handshake},
        login::handle_login,
        status::handle_status,
    },
    core::proxy::ProxyConfig,
};

#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub(crate) enum PacketAction {
    Forward,
    ForwardAnd(Box<PacketAction>),
    Return(RawPacket),
    ReturnAnd(RawPacket, Box<PacketAction>),
    UpdateState(ConnectionState),
    Disconnect,
}

pub(crate) struct PlayerConnectionManager {
    state: ConnectionState,
    player_connection: TcpConnection,
    proxy_configuration_reciever: Receiver<ProxyConfig>,
}

#[derive(Debug, Error)]
pub(crate) enum PlayerConnectionError {
    // #[error("Packet not recieved before timeout exceeded.")]
    // Timeout,
    #[error("Error with player Tcp Connection: {0}.")]
    TcpConnectionError(#[from] TcpConnectionError),
    #[error("Error decoding player packet: {0}")]
    PacketDecodeError(#[from] DecodeError),
    #[error("Error Encoding server packet: {0}")]
    PacketEncodeError(#[from] EncodeError),
    #[error("Login Packet Sequence was incorrect: {0}.")]
    LoginSequenceIncorrect(&'static str),
}

#[derive(Debug)]
pub(super) struct PlayContext {
    player_profile: GameProfile,
    _session_id: Uuid,
}

impl PlayContext {
    pub fn new(player_profile: GameProfile, session_id: Uuid) -> Self {
        PlayContext {
            player_profile,
            _session_id: session_id,
        }
    }
}

impl PlayerConnectionManager {
    pub(crate) async fn new(
        player_stream: TcpStream,
        proxy_configuration_reciever: Receiver<ProxyConfig>,
    ) -> PlayerConnectionManager {
        let player_connection = TcpConnection::new(player_stream);

        PlayerConnectionManager {
            state: ConnectionState::Handshake,
            player_connection,
            proxy_configuration_reciever,
        }
    }

    pub(crate) async fn handle(mut self) {
        // self.connection_backend.init().unwrap();

        // First the handshake
        log::info!("Handshaking");
        let handshake_result = match handle_handshake(&mut self.player_connection).await {
            Ok(result) => result,
            Err(e) => {
                log::error!("Handshake TCP Error: {e}");
                self.state = ConnectionState::LoginDisconnect;
                self.shutdown().await;
                return;
            }
        };

        // Status

        let play_context: PlayContext = match handshake_result {
            HandshakeResult::Status(mut status_ctx) => {
                log::info!("Status check!");
                let server_status = ServerStatus::new(
                    status_ctx.protocol_version,
                    None,
                    self.proxy_configuration_reciever
                        .borrow()
                        .description
                        .as_str(),
                );
                match handle_status(&mut self.player_connection, &mut status_ctx, server_status)
                    .await
                {
                    Ok(_) => (),
                    Err(e) => {
                        log::error!("Handshake TCP Error: {e}");
                    }
                };
                self.state = ConnectionState::LoginDisconnect;
                self.shutdown().await;
                return;
            }
            HandshakeResult::Login(context) => {
                match handle_login(&mut self.player_connection, context).await {
                    Ok(result) => result,
                    Err(e) => {
                        error!("Error in Login: {e}");
                        let disconnect_packet = LoginDisconnectPacket::new(
                            format!("{{\"text\": \"Error in Login: {e}\"}}")
                                .to_string()
                                .as_ref(),
                        );
                        self.send_packet_to_client(disconnect_packet.into_raw().unwrap())
                            .await;
                        self.state = ConnectionState::Disconnecting;
                        self.shutdown().await;
                        return;
                    }
                }
            }
        };

        log::info!(
            "Client Login Success! Profile: {:?}",
            play_context.player_profile
        );

        self.state = ConnectionState::Login;

        // TODO: Temprary setup of default backend. need to setup based on default
        let mut connection_backend = ProxyServerBackend::new(
            &self.proxy_configuration_reciever.borrow().default_server,
            self.proxy_configuration_reciever
                .borrow()
                .default_server_port,
        );

        // TODO: Better Init logic
        match connection_backend.init(&play_context.player_profile).await {
            Ok(_) => {}
            Err(e) => {
                error!("Error in Connecting to backend: {e}, {:?}", e.source());
                self.send_packet_to_client(
                    RawPacketBuilder::new(2)
                        .unwrap()
                        .string(e.to_string())
                        .unwrap()
                        .build(),
                )
                .await;
                self.state = ConnectionState::Disconnecting;
                self.shutdown().await;
                return;
            }
        };
        self.state = ConnectionState::Play;

        while self.state != ConnectionState::Disconnecting {
            if self.state != ConnectionState::Play {
                log::trace!("In State: {:#?}", self.state);
            }
            if self.player_connection.is_shutdown() {
                log::warn!("Player connection has shutdown! Setting to disconnecting state");
                self.state = ConnectionState::Disconnecting;
            }
            self.state = match self.state {
                ConnectionState::Status => {
                    panic!("Unexpected Status state?!");
                }
                ConnectionState::Login => {
                    panic!("unexpected Login State?!");
                }
                ConnectionState::Closed => {
                    error!("Closed Early.");
                    panic!("Closed Early.");
                }
                ConnectionState::Play => {
                    tokio::select! {
                        packet = self.next_player_packet() => {
                            match packet {
                                Ok(packet) => { send_packet_to_server(&mut connection_backend, packet).await; ConnectionState::Play },
                                Err(e) => {log::warn!("Player Connection Disconnected: {e}");ConnectionState::Disconnecting},
                            }
                        }
                        packet = connection_backend.read_packet() => {
                            match packet {
                                Ok(packet) => { self.send_packet_to_client(packet).await; ConnectionState::Play },
                                Err(e) => {log::warn!("Backend Connection Disconnected: {e}");ConnectionState::Disconnecting},
                            }
                        }
                    }
                }
                state => todo!("Status not yet implemented: {state}"),
            };
        }

        if self.state == ConnectionState::Disconnecting {
            self.shutdown().await;
        }
    }

    async fn shutdown(&mut self) {
        self.player_connection.shutdown().await;
        self.state = ConnectionState::Closed;
        log::info!("Connection Terminated.");
    }

    async fn next_player_packet(&mut self) -> Result<RawPacket, TcpConnectionError> {
        self.player_connection.read().await
    }

    async fn send_packet_to_client(&mut self, packet: RawPacket) {
        #[cfg(feature = "trace-packets")]
        log::trace!("Sending to Client: {}", packet);
        self.player_connection.send(packet).await
    }

    #[allow(dead_code)]
    async fn handle_action<T: ProxyBackend>(
        &mut self,
        backend: &mut T,
        action: PacketAction,
        original_packet: Option<RawPacket>,
    ) -> Option<ConnectionState> {
        match action {
            PacketAction::Forward => {
                send_packet_to_server(
                    backend,
                    original_packet
                        .expect("Should not forward withot passing packet")
                        .clone(),
                )
                .await;
                None
            }
            PacketAction::ForwardAnd(next_action) => {
                log::trace!(
                    "Forwarding And doing Packet: {:#?}; next_action: {:#?}",
                    original_packet,
                    next_action
                );
                send_packet_to_server(
                    backend,
                    original_packet
                        .clone()
                        .expect("Should not forward withot passing packet"),
                )
                .await;
                Box::pin(self.handle_action(backend, *next_action, original_packet)).await
            }
            PacketAction::Return(packet) => {
                log::trace!("Returning Packet: {:#?}", packet);
                self.send_packet_to_client(packet).await;
                None
            }
            PacketAction::ReturnAnd(packet, next_action) => {
                log::trace!(
                    "Returning And doing Packet: {:#?}; next_action: {:#?}",
                    packet,
                    next_action
                );
                self.send_packet_to_client(packet).await;
                Box::pin(self.handle_action(backend, *next_action, original_packet)).await
            }
            PacketAction::UpdateState(new_state) => {
                log::debug!("Switching state to {new_state}");
                Some(new_state)
            }
            PacketAction::Disconnect => Some(ConnectionState::Disconnecting),
        }
    }
}

async fn send_packet_to_server<T: ProxyBackend>(connection_backend: &mut T, packet: RawPacket) {
    #[cfg(feature = "trace-packets")]
    log::trace!("Sending to Server: {}", packet);
    connection_backend
        .send_packet(packet)
        .await
        .expect("Could not send to server")
}
