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
        stream: TcpStream,
        target: &str,
    ) -> Result<PlayerConnection, PlayerConnectionError> {
        let (player_read_side, player_write_side) = stream.into_split();
        let server_connection = match TcpStream::connect(target).await {
            Ok(conn) => conn,
            Err(e) => {
                log::error!("Cannot Connect to backend server: {}", e.kind());
                return Err(PlayerConnectionError::new(e));
            }
        };
        let (server_read_side, server_write_side) = server_connection.into_split();

        let player_handler = ConnectionHandler::new(
            player_read_side,
            server_write_side,
            ConnectionSide::PlayerToServer,
        );
        let server_handler = ConnectionHandler::new(
            server_read_side,
            player_write_side,
            ConnectionSide::ServerToPlayer,
        );
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
