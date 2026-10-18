use log::error;
use netherconduit_core::{
    connection::ConnectionState,
    packet::{
        RawPacket,
        builder::RawPacketBuilder,
        primitives::GameProfile,
        types::{HandshakePacket, MinecraftPacket, PingRequestPacket, handshake::HandshakeIntent},
    },
    server::{protocol_version::ConnectionProtocolVersion, status::ServerStatus},
};
use thiserror::Error;
use tokio::{net::TcpStream, sync::watch::Receiver};

use crate::{
    backend::{ProxyBackend, ProxyServerBackend},
    connection::{
        TcpConnection, TcpConnectionError,
        player_connection::PlayerConnectionError::LoginSequenceIncorrect,
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
    protocol_version: Option<ConnectionProtocolVersion>,
    proxy_configuration_reciever: Receiver<ProxyConfig>,
}

#[derive(Debug, Error)]
enum PlayerConnectionError {
    #[error("Packet not recieved before timeout exceeded.")]
    Timeout,
    #[error("Error with player Tcp Connection: {0}.")]
    TcpConnectionError(#[from] TcpConnectionError),
    #[error("Login Packet Sequence was incorrect: {0}.")]
    LoginSequenceIncorrect(&'static str),
}

// #[derive(Debug)]
// pub(crate) struct PlayerConnectionError {
//     pub(crate) error: Error,
// }

// impl PlayerConnectionError {
//     pub fn new(error: Error) -> Self {
//         PlayerConnectionError { error }
//     }
// }

impl PlayerConnectionManager {
    pub(crate) async fn new(
        player_stream: TcpStream,
        proxy_configuration_reciever: Receiver<ProxyConfig>,
    ) -> PlayerConnectionManager {
        let player_connection = TcpConnection::new(player_stream);

        PlayerConnectionManager {
            state: ConnectionState::Handshake,
            player_connection,
            protocol_version: None,
            proxy_configuration_reciever,
        }
    }

    pub(crate) async fn handle(mut self) {
        // self.connection_backend.init().unwrap();

        // First the handshake
        log::info!("Handshaking");
        let (initial_state, protocol_version) = match self.next_player_packet().await {
            Ok(packet) => handle_handshake(&packet),
            Err(e) => {
                log::error!("Handshake TCP Error: {e}");
                self.state = ConnectionState::LoginDisconnect;
                self.shutdown().await;
                self.state = ConnectionState::Closed;
                return;
            }
        };
        self.state = initial_state;
        self.protocol_version = Some(protocol_version);

        // Status Check
        while self.state == ConnectionState::Status {
            log::info!("Status check!");
            self.state = match self.next_player_packet().await {
                Ok(packet) => {
                    let server_status = ServerStatus::new(
                        self.protocol_version.unwrap(),
                        None,
                        self.proxy_configuration_reciever
                            .borrow()
                            .description
                            .as_str(),
                    );
                    handle_status(&mut self.player_connection, server_status, &packet).await
                }
                Err(e) => {
                    log::error!("Status TCP Error: {e}");
                    ConnectionState::Disconnecting
                }
            };
        }

        while self.state == ConnectionState::Login {
            self.state = ConnectionState::Play;
        }

        // TODO: Temprary setup of default backend. need to setup based on default
        let mut connection_backend = ProxyServerBackend::new(
            &self.proxy_configuration_reciever.borrow().default_server,
            self.proxy_configuration_reciever
                .borrow()
                .default_server_port,
        );
        connection_backend.init().await.unwrap();

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
                            match packet.expect("Backend shouldnt close first") {
                                Some(packet) => { self.send_packet_to_client(packet).await; ConnectionState::Play },
                                None => {log::warn!("Backend Connection Disconnected");ConnectionState::Disconnecting},
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
        // log::debug!("Sending to Client: {:?}", packet);
        self.player_connection.send(packet).await
    }

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

    async fn handle_login(&mut self) -> Result<GameProfile, PlayerConnectionError> {
        let packet = self.next_player_packet().await?;
        if packet.id().unwrap().value() != 0 {
            return Err(LoginSequenceIncorrect("Id not 0"));
        };

        Ok(GameProfile::default())
    }
}

async fn send_packet_to_server<T: ProxyBackend>(connection_backend: &mut T, packet: RawPacket) {
    // log::debug!("Sending to Server: {:?}", packet);
    connection_backend
        .send_packet(packet)
        .await
        .expect("Could not send to server")
}

// Helper function to handle handshake flow
fn handle_handshake(raw_packet: &RawPacket) -> (ConnectionState, ConnectionProtocolVersion) {
    let packet = match HandshakePacket::from_raw(raw_packet) {
        Ok(value) => value,
        Err(e) => {
            log::error!("Decode Error: {:?}", e);
            return (
                ConnectionState::LoginDisconnect,
                ConnectionProtocolVersion::default(),
            );
        }
    };
    log::trace!("Incoming Handshake: {packet}");
    (
        match packet.intent {
            HandshakeIntent::Status => ConnectionState::Status,
            HandshakeIntent::Login => ConnectionState::Login,
            HandshakeIntent::Transfer => ConnectionState::Login,
        },
        packet.protocol_version,
    )
}

// Helper function to handle status flow
async fn handle_status(
    connection: &mut TcpConnection,
    server_status: ServerStatus,
    packet: &RawPacket,
) -> ConnectionState {
    log::trace!("Incoming Status: {packet}");
    match packet.id().unwrap().value() {
        0 => {
            log::debug!("Status Check");
            let return_packet = RawPacketBuilder::new(0)
                .unwrap()
                .string(
                    server_status
                        .to_json()
                        .expect("Should be valid status encoding"),
                )
                .unwrap()
                .build();
            connection.send(return_packet).await;
            ConnectionState::Status
            // PacketAction::ReturnAnd(return_packet, Box::new(PacketAction::Disconnect))
        }
        1 => {
            let ping_request_packet = match PingRequestPacket::from_raw(packet) {
                Ok(value) => value,
                Err(e) => {
                    log::error!("Decode Error: {:?}", e);
                    return ConnectionState::Disconnecting;
                }
            };
            log::debug!("Ping Request: {:#?}", ping_request_packet);
            let return_packet = RawPacketBuilder::new(1)
                .unwrap()
                .long(ping_request_packet.payload)
                .unwrap()
                .build();
            connection.send(return_packet).await;
            ConnectionState::Disconnecting
        }
        value => {
            log::warn!("Invalid status packet ID: {value}");
            ConnectionState::Disconnecting
        }
    }
}
