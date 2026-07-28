// Connection handler
use futures::sink::SinkExt;
use log::{debug, error, info, warn};
use netherconduit_core::packet::stream::{
    decoder::MinecraftPacketDecoder, encoder::MinecraftPacketEncoder,
};
use tokio::{
    io::AsyncWriteExt,
    net::tcp::{OwnedReadHalf, OwnedWriteHalf},
};
use tokio_stream::StreamExt;
use tokio_util::codec::{FramedRead, FramedWrite};

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

    pub(crate) async fn handle(self) {
        let decoder = MinecraftPacketDecoder::new();
        let mut reader = FramedRead::new(self.read_stream, decoder);
        let encoder = MinecraftPacketEncoder::new();
        let mut writer = FramedWrite::new(self.write_stream, encoder);

        while let Some(result) = reader.next().await {
            let packet = match result {
                Ok(packet) => packet,
                Err(error) => {
                    error!("Failed to decode packet: {error}");
                    break;
                }
            };
            info!("{:#?} Packet Recieved: {}", self.side, packet);
            writer.send(packet).await.unwrap();
        }
        warn!("Disconnected");
    }
}
