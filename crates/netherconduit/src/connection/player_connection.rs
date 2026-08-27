use netherconduit_core::{
    connection::ConnectionState,
    packet::{
        RawPacket,
        builder::RawPacketBuilder,
        types::{HandshakePacket, MinecraftPacket, PingRequestPacket, handshake::HandshakeIntent},
    },
};
use tokio::net::TcpStream;

use crate::{
    backend::{ProxyBackend, ProxyBackendHandle, ProxyServerBackend},
    connection::{Connection, ConnectionHandle},
};

#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub(crate) enum PacketAction {
    Forward(RawPacket),
    Return(RawPacket),
    ReturnAnd(RawPacket, Box<PacketAction>),
    UpdateState(ConnectionState),
    Disconnect,
}

pub(crate) struct PlayerConnectionManager {
    state: ConnectionState,
    player_connection: Connection,
    player_handler: ConnectionHandle,
    _connection_backend: Box<dyn ProxyBackend>,
    connection_backend_handle: Option<ProxyBackendHandle>,
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
        target_server: &str,
        target_port: u16,
    ) -> PlayerConnectionManager {
        let (player_connection, player_handler) = Connection::new(player_stream);
        let server_backend = ProxyServerBackend::new(target_server, target_port);
        PlayerConnectionManager {
            state: ConnectionState::Handshake,
            player_connection,
            player_handler,
            _connection_backend: Box::new(server_backend),
            connection_backend_handle: None,
        }
    }

    pub(crate) async fn handle(mut self) {
        self.player_connection.dispatch();
        // self.connection_backend_handle = Some(self.connection_backend.init().unwrap()); // TODO: better match

        while self.state != ConnectionState::Closed {
            log::trace!("In State: {:#?}", self.state);
            self.state = match self.state {
                ConnectionState::Handshake => {
                    log::info!("Handshaking");
                    let action: PacketAction = match self.player_handler.recv().await {
                        Some(packet) => handle_handshake(&packet),
                        None => {
                            log::error!("Invalid/no packet recieved. Disconnecting");
                            PacketAction::Disconnect
                        }
                    };
                    self.handle_action(action).await.unwrap_or(self.state)
                }
                ConnectionState::Status => {
                    log::info!("Status check!");
                    let action: PacketAction = match self.player_handler.recv().await {
                        Some(packet) => handle_status(&packet),
                        None => {
                            log::error!("Invalid/no packet recieved. Disconnecting");
                            PacketAction::Disconnect
                        }
                    };
                    self.handle_action(action).await.unwrap_or(self.state)
                }
                state => todo!("Status not yet implemented: {state}"),
            };
        }

        self.player_connection.shutdown().await;
        log::info!("Connection Terminated.");
    }

    async fn send_packet_to_server(&mut self, packet: RawPacket) {
        // log::debug!("Sending to Server: {:?}", packet);
        self.connection_backend_handle
            .as_ref()
            .unwrap()
            .outgoing
            .send(packet)
            .await
            .expect("Could not send to server handler pipe")
    }

    async fn send_packet_to_client(&self, packet: RawPacket) {
        // log::debug!("Sending to Client: {:?}", packet);
        self.player_handler
            .send(packet)
            .await
            .expect("Could not send to player handler pipe")
    }

    async fn handle_action(&mut self, action: PacketAction) -> Option<ConnectionState> {
        match action {
            PacketAction::Forward(packet) => {
                self.send_packet_to_server(packet).await;
                None
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
                Box::pin(self.handle_action(*next_action)).await
            }
            PacketAction::UpdateState(new_state) => {
                log::debug!("Switching state to {new_state}");
                Some(new_state)
            }
            PacketAction::Disconnect => Some(ConnectionState::Closed),
        }
    }
}

fn handle_handshake(packet: &RawPacket) -> PacketAction {
    let packet = match HandshakePacket::from_raw(packet) {
        Ok(value) => value,
        Err(e) => {
            log::error!("Decode Error: {:?}", e);
            return PacketAction::Disconnect;
        }
    };
    log::trace!("Incoming Handshake: {packet}");
    match packet.intent {
        HandshakeIntent::Status => PacketAction::UpdateState(ConnectionState::Status),
        HandshakeIntent::Login => PacketAction::UpdateState(ConnectionState::Login),
        HandshakeIntent::Transfer => PacketAction::UpdateState(ConnectionState::Login),
    }
}

fn handle_status(packet: &RawPacket) -> PacketAction {
    log::trace!("Incoming Status: {packet}");
    match packet.id().unwrap().value() {
        0 => {
            log::debug!("Status Check");
            let return_packet = RawPacketBuilder::new(0)
                .string_slice(
                    "{\"version\": {\"name\": \"26.2\",\"protocol\": 776},\"players\": {\"max\": 20,\"online\": 0},\"description\": {\"text\": \"Hello, world!\"}}",
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
