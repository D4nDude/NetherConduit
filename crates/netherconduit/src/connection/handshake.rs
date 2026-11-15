use netherconduit_core::packet::types::{
    HandshakePacket, MinecraftPacket, handshake::HandshakeIntent,
};

use crate::connection::{
    TcpConnection, login::LoginContext, player_connection::PlayerConnectionError,
    status::StatusContext,
};

#[derive(Debug)]
pub(super) enum HandshakeResult {
    Status(StatusContext),
    Login(LoginContext), //TODO: Login Context
}

// Helper function to handle handshake flow
pub(super) async fn handle_handshake(
    connection: &mut TcpConnection,
) -> Result<HandshakeResult, PlayerConnectionError> {
    let raw_packet = connection.read().await?;
    let packet = HandshakePacket::from_raw(&raw_packet)?;
    log::trace!("Incoming Handshake: {packet}");
    match packet.intent {
        HandshakeIntent::Status => {
            let status_ctx = StatusContext::new(packet.protocol_version);
            Ok(HandshakeResult::Status(status_ctx))
        }
        HandshakeIntent::Login => Ok(HandshakeResult::Login(LoginContext::new())),
        HandshakeIntent::Transfer => Ok(HandshakeResult::Login(LoginContext::new())),
    }
}
