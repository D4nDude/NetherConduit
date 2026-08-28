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
    backend::{ProxyBackend, ProxyServerBackend},
    connection::Connection,
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
    connection_backend: Box<dyn ProxyBackend>,
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
        let player_connection = Connection::new(player_stream);
        let server_backend = ProxyServerBackend::new(target_server, target_port);
        PlayerConnectionManager {
            state: ConnectionState::Handshake,
            player_connection,
            connection_backend: Box::new(server_backend),
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
                ConnectionState::Handshake => {
                    log::info!("Handshaking");
                    let (action, original_packet) = match self.player_connection.read().await {
                        Some(packet) => (handle_handshake(&packet), Some(packet)),
                        None => {
                            log::error!("Invalid/no packet recieved. Disconnecting");
                            (PacketAction::Disconnect, None)
                        }
                    };
                    self.handle_action(action, original_packet)
                        .await
                        .unwrap_or(self.state)
                }
                ConnectionState::Status => {
                    log::info!("Status check!");
                    let (action, original_packet) = match self.player_connection.read().await {
                        Some(packet) => (handle_status(&packet), Some(packet)),
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

fn handle_handshake(raw_packet: &RawPacket) -> PacketAction {
    let packet = match HandshakePacket::from_raw(raw_packet) {
        Ok(value) => value,
        Err(e) => {
            log::error!("Decode Error: {:?}", e);
            return PacketAction::Disconnect;
        }
    };
    log::trace!("Incoming Handshake: {packet}");
    match packet.intent {
        HandshakeIntent::Status => PacketAction::UpdateState(ConnectionState::Status),
        HandshakeIntent::Login => {
            PacketAction::ForwardAnd(Box::new(PacketAction::UpdateState(ConnectionState::Play)))
        }
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
