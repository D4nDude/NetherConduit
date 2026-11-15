// while self.state == ConnectionState::Status {
//             log::info!("Status check!");
//             self.state = match self.next_player_packet().await {
//                 Ok(packet) => {
//                     let server_status = ServerStatus::new(
//                         self.protocol_version.unwrap(),
//                         None,
//                         self.proxy_configuration_reciever
//                             .borrow()
//                             .description
//                             .as_str(),
//                     );
//                     handle_status(&mut self.player_connection, server_status, &packet).await
//                 }
//                 Err(e) => {
//                     log::error!("Status TCP Error: {e}");
//                     ConnectionState::Disconnecting
//                 }
//             };
//         }

use netherconduit_core::{
    packet::{
        builder::RawPacketBuilder,
        types::{MinecraftPacket, PingRequestPacket},
    },
    server::{protocol_version::ConnectionProtocolVersion, status::ServerStatus},
};

use crate::connection::{TcpConnection, player_connection::PlayerConnectionError};

#[derive(Debug)]
pub(super) struct StatusContext {
    pub protocol_version: ConnectionProtocolVersion,
}

impl StatusContext {
    pub fn new(protocol_version: ConnectionProtocolVersion) -> Self {
        StatusContext { protocol_version }
    }
}

pub(super) async fn handle_status(
    connection: &mut TcpConnection,
    _context: &mut StatusContext, //TODO: Figure out whats needed
    server_status: ServerStatus,
) -> Result<(), PlayerConnectionError> {
    let packet = connection.read().await?;

    log::trace!("Incoming Status: {packet}");
    if packet.id()?.value() == 0 {
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
    };

    let packet = connection.read().await?;

    if packet.id()?.value() == 1 {
        let ping_request_packet = PingRequestPacket::from_raw(&packet)?;
        log::debug!("Ping Request: {:#?}", ping_request_packet);
        let return_packet = RawPacketBuilder::new(1)
            .unwrap()
            .long(ping_request_packet.payload)
            .unwrap()
            .build();
        connection.send(return_packet).await;
    }

    Ok(())
}
