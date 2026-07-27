use tokio::net::TcpStream;

use crate::connection::{ConnectionHandler, ConnectionSide};

pub(crate) struct PlayerConnection {
    player_handler: ConnectionHandler,
    server_handler: ConnectionHandler,
}

impl PlayerConnection {
    pub(crate) async fn new(stream: TcpStream) -> PlayerConnection {
        let (player_read_side, player_write_side) = stream.into_split();
        let server_connection = TcpStream::connect("localhost:25566").await.unwrap();
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
        PlayerConnection {
            player_handler,
            server_handler,
        }
    }

    pub(crate) async fn dispatch(self) {
        let PlayerConnection {
            player_handler,
            server_handler,
        } = self;

        tokio::join!(player_handler.handle(), server_handler.handle());
    }
}
