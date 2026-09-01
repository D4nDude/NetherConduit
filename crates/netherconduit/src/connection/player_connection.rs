use netherconduit_core::{
    connection::ConnectionState,
    packet::{
        RawPacket,
        builder::RawPacketBuilder,
        types::{HandshakePacket, MinecraftPacket, PingRequestPacket, handshake::HandshakeIntent},
    },
    server::{protocol_version::ConnectionProtocolVersion, status::ServerStatus},
};
use tokio::{net::TcpStream, sync::watch::Receiver};

use crate::{
    backend::{ProxyBackend, ProxyServerBackend},
    connection::Connection,
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
    player_connection: Connection,
    protocol_version: ConnectionProtocolVersion,
    connection_backend: Box<dyn ProxyBackend>,
    proxy_configuration_reciever: Receiver<ProxyConfig>,
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
        let mut player_connection = Connection::new(player_stream);

        // TODO: Malicious parties can hang and not send a handshake, need timeout
        log::info!("Handshaking");
        let (initial_state, protocol_version) = match player_connection.read().await {
            Some(packet) => handle_handshake(&packet),
            None => {
                log::error!("Invalid/no packet recieved. Disconnecting");
                (
                    ConnectionState::LoginDisconnect,
                    ConnectionProtocolVersion::default(),
                )
            }
        };

        // TODO: Temprary setup of default backend. Will be moved to after login
        let server_backend = ProxyServerBackend::new(
            &proxy_configuration_reciever.borrow().default_server,
            proxy_configuration_reciever.borrow().default_server_port,
        );

        PlayerConnectionManager {
            state: initial_state,
            player_connection,
            protocol_version,
            connection_backend: Box::new(server_backend),
            proxy_configuration_reciever,
        }
    }

    pub(crate) async fn handle(mut self) {
        // self.connection_backend.init().unwrap();

        while self.state != ConnectionState::Closed {
            if self.state != ConnectionState::Play {
                log::trace!("In State: {:#?}", self.state);
            }
            if self.player_connection.is_shutdown() {
                self.state = ConnectionState::Closed;
            }
            self.state = match self.state {
                ConnectionState::Status => {
                    log::info!("Status check!");
                    let (action, original_packet) = match self.player_connection.read().await {
                        Some(packet) => {
                            let server_status = ServerStatus::new(
                                self.protocol_version,
                                None,
                                self.proxy_configuration_reciever
                                    .borrow()
                                    .description
                                    .as_str(),
                            );
                            (handle_status(server_status, &packet), Some(packet))
                        }
                        None => {
                            log::error!("Invalid/no packet recieved. Disconnecting");
                            (PacketAction::Disconnect, None)
                        }
                    };
                    self.handle_action(action, original_packet)
                        .await
                        .unwrap_or(self.state)
                }
                ConnectionState::Play => {
                    tokio::select! {
                        packet = self.player_connection.read() => {
                            match packet {
                                Some(packet) => { self.send_packet_to_server(packet).await; ConnectionState::Play },
                                None => ConnectionState::Closed,
                            }
                        }
                        packet = self.connection_backend.read_packet() => {
                            match packet.expect("Backend shouldnt close first") {
                                Some(packet) => { self.send_packet_to_client(packet).await; ConnectionState::Play },
                                None => ConnectionState::Closed,
                            }
                        }
                    }
                }
                ConnectionState::Login => {
                    self.connection_backend.init().await.unwrap();
                    ConnectionState::Play
                }
                state => todo!("Status not yet implemented: {state}"),
            };
        }

        self.connection_backend.shutdown().await.unwrap();
        self.player_connection.shutdown().await;
        log::info!("Connection Terminated.");
    }

    async fn send_packet_to_server(&mut self, packet: RawPacket) {
        // log::debug!("Sending to Server: {:?}", packet);
        self.connection_backend
            .as_mut()
            .send_packet(packet)
            .await
            .expect("Could not send to server")
    }

    async fn send_packet_to_client(&mut self, packet: RawPacket) {
        // log::debug!("Sending to Client: {:?}", packet);
        self.player_connection.send(packet).await
    }

    async fn handle_action(
        &mut self,
        action: PacketAction,
        original_packet: Option<RawPacket>,
    ) -> Option<ConnectionState> {
        match action {
            PacketAction::Forward => {
                self.send_packet_to_server(
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
                self.send_packet_to_server(
                    original_packet
                        .clone()
                        .expect("Should not forward withot passing packet"),
                )
                .await;
                Box::pin(self.handle_action(*next_action, original_packet)).await
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
                Box::pin(self.handle_action(*next_action, original_packet)).await
            }
            PacketAction::UpdateState(new_state) => {
                log::debug!("Switching state to {new_state}");
                Some(new_state)
            }
            PacketAction::Disconnect => Some(ConnectionState::Closed),
        }
    }
}

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

fn handle_status(server_status: ServerStatus, packet: &RawPacket) -> PacketAction {
    log::trace!("Incoming Status: {packet}");
    match packet.id().unwrap().value() {
        0 => {
            log::debug!("Status Check");
            let return_packet = RawPacketBuilder::new(0)
                .string(
                    server_status
                        .to_json()
                        .expect("Should be valid status encoding"),
                )
                .unwrap()
                .build();
            PacketAction::Return(return_packet)
            // PacketAction::ReturnAnd(return_packet, Box::new(PacketAction::Disconnect))
        }
        1 => {
            let ping_request_packet = match PingRequestPacket::from_raw(packet) {
                Ok(value) => value,
                Err(e) => {
                    log::error!("Decode Error: {:?}", e);
                    return PacketAction::Disconnect;
                }
            };
            log::debug!("Ping Request: {:#?}", ping_request_packet);
            let return_packet = RawPacketBuilder::new(1)
                .long(ping_request_packet.payload)
                .unwrap()
                .build();
            PacketAction::ReturnAnd(return_packet, Box::new(PacketAction::Disconnect))
        }
        value => {
            log::warn!("Invalid status packet ID: {value}");
            PacketAction::Disconnect
        }
    }
}
