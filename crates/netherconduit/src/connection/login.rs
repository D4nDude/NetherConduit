use netherconduit_core::packet::types::{
    MinecraftPacket,
    login::{LoginStartPacket, LoginSuccessPacket},
};
use uuid::Uuid;

use crate::connection::{
    TcpConnection,
    player_connection::{PlayContext, PlayerConnectionError},
};

#[derive(Debug)]
pub(super) struct LoginContext {}

impl LoginContext {
    pub fn new() -> Self {
        LoginContext {}
    }
}

pub(super) async fn handle_login(
    connection: &mut TcpConnection,
    _context: LoginContext,
) -> Result<PlayContext, PlayerConnectionError> {
    let packet = connection.read().await?;
    if packet.id()?.value() != 0 {
        return Err(PlayerConnectionError::LoginSequenceIncorrect("Id not 0"));
    };
    let login_start_packet = LoginStartPacket::from_raw(&packet)?;

    let player_profile = login_start_packet.into_game_profile();
    let session_id = Uuid::new_v4();

    // TODO: Encryption and Compression

    let login_success = LoginSuccessPacket::new(player_profile.clone(), session_id);
    connection.send(login_success.into_raw()?).await;

    let packet = connection.read().await?;
    if packet.id()?.value() != 3 {
        return Err(PlayerConnectionError::LoginSequenceIncorrect("Id not 3"));
    };

    Ok(PlayContext::new(player_profile, session_id))
}
