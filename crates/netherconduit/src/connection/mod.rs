// Connection handler
use log::{error, info};
use netherconduit_core::packet::stream::decoder::MinecraftPacketDecoder;
use tokio::{
    io::AsyncWriteExt,
    net::tcp::{OwnedReadHalf, OwnedWriteHalf},
};
use tokio_stream::StreamExt;
use tokio_util::codec::FramedRead;

pub(crate) mod player_connection;

#[derive(Debug, PartialEq, Eq)]
#[allow(dead_code)]
enum ConnectionState {
    Handshake,
    Active,
    Closed,
}

#[derive(Debug, PartialEq, Eq)]
pub(crate) enum ConnectionSide {
    PlayerToServer,
    ServerToPlayer,
}

pub struct ConnectionHandler {
    // state: ConnectionState,
    side: ConnectionSide,
    read_stream: OwnedReadHalf,
    write_stream: OwnedWriteHalf,
}

impl ConnectionHandler {
    pub(crate) fn new(
        read_side: OwnedReadHalf,
        write_side: OwnedWriteHalf,
        side: ConnectionSide,
    ) -> ConnectionHandler {
        ConnectionHandler {
            // state: ConnectionState::Handshake,
            side,
            read_stream: read_side,
            write_stream: write_side,
        }
    }

    pub(crate) async fn handle(mut self) {
        let decoder = MinecraftPacketDecoder::new();
        let mut reader = FramedRead::new(self.read_stream, decoder);

        while let Some(result) = reader.next().await {
            let packet = match result {
                Ok(packet) => packet,
                Err(error) => {
                    error!("Failed to decode packet: {error}");
                    break;
                }
            };
            info!("{:#?} Packet Recieved: {:#?}", self.side, packet);
            self.write_stream
                .write_all(packet.get_data().as_ref())
                .await
                .unwrap();
        }
    }
}
