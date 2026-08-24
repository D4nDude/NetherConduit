use std::io::Error;

use tokio::net::TcpStream;

use crate::connection::{ConnectionHandler, ConnectionSide};

pub(crate) struct PlayerConnection {
    player_handler: ConnectionHandler,
    server_handler: ConnectionHandler,
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

impl PlayerConnection {
    pub(crate) async fn new(
        player_stream: TcpStream,
        target: &str,
    ) -> Result<PlayerConnection, PlayerConnectionError> {
        let server_connection = match TcpStream::connect(target).await {
            Ok(conn) => conn,
            Err(e) => {
                log::error!("Cannot Connect to backend server: {}", e.kind());
                return Err(PlayerConnectionError::new(e));
            }
        };

        let mut player_handler =
            ConnectionHandler::new(player_stream, ConnectionSide::PlayerToServer, None);
        let server_handler = ConnectionHandler::new(
            server_connection,
            ConnectionSide::ServerToPlayer,
            Some(player_handler.get_connection_send_queue()),
        );
        player_handler
            .set_connection_recieve_forward_queue(server_handler.get_connection_send_queue());
        Ok(PlayerConnection {
            player_handler,
            server_handler,
        })
    }

    pub(crate) async fn dispatch(self) {
        let PlayerConnection {
            player_handler,
            server_handler,
        } = self;

        tokio::join!(player_handler.handle(), server_handler.handle());
    }
}
