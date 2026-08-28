// Connection handler
use futures::sink::SinkExt;
use log::error;
use netherconduit_core::packet::RawPacket;
use netherconduit_core::packet::stream::{
    decoder::MinecraftPacketDecoder, encoder::MinecraftPacketEncoder,
};
use tokio::io::AsyncWriteExt;
use tokio::net::TcpStream;
use tokio::net::tcp::{OwnedReadHalf, OwnedWriteHalf};
use tokio_stream::StreamExt;
use tokio_util::codec::{FramedRead, FramedWrite};
use tokio_util::sync::CancellationToken;

pub(crate) mod player_connection;

#[derive(Debug)]
pub struct Connection {
    reader: FramedRead<OwnedReadHalf, MinecraftPacketDecoder>,
    writer: FramedWrite<OwnedWriteHalf, MinecraftPacketEncoder>,

    shutdown_token: CancellationToken,
}

impl Connection {
    pub(crate) fn new(tcp_stream: TcpStream) -> Connection {
        let (read_side, write_side) = tcp_stream.into_split();

        let decoder = MinecraftPacketDecoder::new();
        let reader = FramedRead::new(read_side, decoder);

        let encoder = MinecraftPacketEncoder::new();
        let writer = FramedWrite::new(write_side, encoder);

        Connection {
            reader,
            writer,
            shutdown_token: CancellationToken::new(),
        }
    }

    pub async fn read(&mut self) -> Option<RawPacket> {
        tokio::select! {
            _ = self.shutdown_token.cancelled() => {
                log::error!("Connection already shutdown");
                None
            }

            potential_next_packet = self.reader.next() => {

                // unwrap to see if a packet was consumed
                let next_packet_result = match potential_next_packet {
                    Some(next_packet_result) => next_packet_result,
                    None => {
                        log::debug!("Read Stream Disconnected");
                        self.shutdown_token.cancel();
                        return None
                    },
                };

                // check if packet framer encountered error
                match next_packet_result {
                    Ok(packet) => Some(packet),
                    Err(error) => {
                        error!("Failed to decode packet: {error}");
                        None
                    }
                }
            }
        }
    }

    pub async fn send(&mut self, packet: RawPacket) {
        // log::trace!("Write task writing packet: {packet}");
        tokio::select! {
            _ = self.shutdown_token.cancelled() => {
                log::error!("Connection already shutdown");
            }

            result = self.writer.send(packet) => {
                match result {
                    Ok(()) => (),
                    Err(error) => log::error!("Failed to send packet: {error}"),
                }
            }
        }
    }

    pub async fn flush(&mut self) {
        log::debug!("Flushing Write Queue");
        if let Err(error) = self.writer.flush().await {
            log::error!("Failed to flush connection: {error}");
        }
    }

    pub async fn shutdown(mut self) {
        if self.shutdown_token.is_cancelled() {
            log::debug!("Connection already terminated");
            return;
        }
        self.shutdown_token.cancel();
        self.flush().await;
        log::debug!("Shutting down TCP connection");
        if let Err(error) = self.writer.into_inner().shutdown().await {
            log::error!("Failed to shutdown TCP Connection: {error}")
        }
    }

    pub fn is_shutdown(&self) -> bool {
        self.shutdown_token.is_cancelled()
    }
}
