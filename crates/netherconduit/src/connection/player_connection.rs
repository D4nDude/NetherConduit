use std::io::Error;

use netherconduit_core::packet::{ConnectionState, RawPacket};
use tokio::net::TcpStream;

use crate::connection::{Connection, ConnectionHandle};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub(crate) enum PacketAction {
    #[default]
    Forward,
    UpdateState(ConnectionState),
    Disconnect,
}

pub(crate) struct PlayerConnectionManager {
    state: ConnectionState,
    player_connection: Connection,
    player_handler: ConnectionHandle,
    server_connection: Connection,
    server_handler: ConnectionHandle,
}

#[derive(Debug)]
pub(crate) struct PlayerConnectionError {
    pub(crate) error: Error,
}

impl PlayerConnectionError {
    pub fn new(error: Error) -> Self {
        PlayerConnectionError { error }
    }
}

impl PlayerConnectionManager {
    pub(crate) async fn new(
        player_stream: TcpStream,
        target: &str,
    ) -> Result<PlayerConnectionManager, PlayerConnectionError> {
        let server_connection = match TcpStream::connect(target).await {
            Ok(conn) => conn,
            Err(e) => {
                log::error!("Cannot Connect to backend server: {}", e.kind());
                return Err(PlayerConnectionError::new(e));
            }
        };

        let (player_connection, player_handler) = Connection::new(player_stream);
        let (server_connection, server_handler) = Connection::new(server_connection);
        Ok(PlayerConnectionManager {
            state: ConnectionState::Handshake,
            player_connection,
            player_handler,
            server_connection,
            server_handler,
        })
    }

    pub(crate) async fn handle(mut self) {
        self.player_connection.dispatch();
        self.server_connection.dispatch();

        while self.state != ConnectionState::Closed {
            tokio::select! { Some(packet) = self.player_handler.recv() => {
                    self.handle_client_packet(packet).await;
                }

            Some(packet) = self.server_handler.recv() => {
                self.handle_server_packet(packet).await;
            }};

            // process packet based on state
            // let action = match self.state {
            //     ConnectionState::Handshake => Self::handle_handshake(&next_packet),
            //     _ => {
            //         error!("Connection State not implemented: {:?}", self.state);
            //         PacketAction::Disconnect
            //     }
            // };

            // // pass the packet on or affect connection state
            // match action {
            //     PacketAction::Forward => {
            //         connection_forward_queue.send(next_packet);
            //     }
            //     PacketAction::UpdateState(new_state) => {
            //         self.state = new_state;
            //     }
            //     PacketAction::Disconnect => {
            //         error!("I dont know how to disconnect...");
            //         panic!("Force Disconnect");
            //     }
            // }
        }

        //tokio::join!(player_handler.handle(), server_handler.handle());
        log::info!("Connection Terminated.");
    }

    async fn handle_client_packet(&self, packet: RawPacket) {
        log::debug!("Sending to Server: {:?}", packet);
        self.server_handler
            .send(packet)
            .await
            .expect("Could not send to server handler pipe")
    }

    async fn handle_server_packet(&self, packet: RawPacket) {
        log::debug!("Sending to Client: {:?}", packet);
        self.player_handler
            .send(packet)
            .await
            .expect("Could not send to player handler pipe")
    }
}

// fn handle_handshake(packet: &RawPacket) -> PacketAction {
//     let packet = match HandshakePacket::from_raw(&packet) {
//         Ok(value) => value,
//         Err(e) => {
//             error!("Decode Error: {:?}", e);
//             return PacketAction::Disconnect;
//         }
//     };
//     match packet.intent {
//         HandshakeIntent::Status => PacketAction::UpdateState(ConnectionState::Status),
//         HandshakeIntent::Login => PacketAction::UpdateState(ConnectionState::Login),
//         HandshakeIntent::Transfer => PacketAction::UpdateState(ConnectionState::Login),
//     }
// }
