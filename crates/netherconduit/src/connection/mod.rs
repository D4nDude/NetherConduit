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
use tokio::sync::mpsc::error::SendError;
use tokio::sync::mpsc::{self, Receiver, Sender};
use tokio::task::JoinHandle;
use tokio_stream::StreamExt;
use tokio_util::codec::{FramedRead, FramedWrite};
use tokio_util::sync::CancellationToken;

pub(crate) mod player_connection;

#[derive(Debug)]
pub struct Connection {
    connection_outgoing_queue_receiver: Option<Receiver<RawPacket>>, // internal reciever for sending packets
    connection_incoming_queue_sender: Sender<RawPacket>, // queue to send recived packets onward

    tcp_stream: Option<TcpStream>,
    write_task: Option<JoinHandle<()>>,
    read_task: Option<JoinHandle<()>>,

    shutdown_token: CancellationToken,
}

#[derive(Debug)]
pub struct ConnectionHandle {
    pub incoming: Receiver<RawPacket>,
    pub outgoing: Sender<RawPacket>,
    pub shutdown_token: CancellationToken,
}

impl ConnectionHandle {
    pub async fn send(&self, packet: RawPacket) -> Result<(), SendError<RawPacket>> {
        tokio::select! {
            result = self.outgoing.send(packet.clone()) => result,

            _ = self.shutdown_token.cancelled() => {
                Err(SendError(packet))
            }
        }
    }

    pub async fn recv(&mut self) -> Option<RawPacket> {
        self.incoming.recv().await
    }
}

impl Connection {
    pub(crate) fn new(tcp_stream: TcpStream) -> (Connection, ConnectionHandle) {
        // Create the queue four our sender
        let (connection_incoming_queue_sender, connection_incomming_queue_receiver) =
            mpsc::channel(64);
        let (connection_outgoing_queue_sender, connection_outgoing_queue_receiver) =
            mpsc::channel(64);
        let shutdown_token = CancellationToken::new();
        (
            Connection {
                connection_outgoing_queue_receiver: Some(connection_outgoing_queue_receiver),
                connection_incoming_queue_sender,
                tcp_stream: Some(tcp_stream),
                write_task: None,
                read_task: None,
                shutdown_token: shutdown_token.clone(),
            },
            ConnectionHandle {
                incoming: connection_incomming_queue_receiver,
                outgoing: connection_outgoing_queue_sender,
                shutdown_token,
            },
        )
    }

    pub(crate) fn dispatch(&mut self) {
        // split the tcp stream
        let (read_side, write_side) = self
            .tcp_stream
            .take()
            .expect("TcpStream already split")
            .into_split();

        self.write_task = Some(tokio::spawn(write_handler(
            write_side,
            self.connection_outgoing_queue_receiver
                .take()
                .expect("Reciever already given to a task"),
            self.shutdown_token.clone(),
        )));

        self.read_task = Some(tokio::spawn(read_handler(
            read_side,
            self.connection_incoming_queue_sender.clone(),
            self.shutdown_token.clone(),
        )));
    }

    pub async fn shutdown(&mut self) {
        self.shutdown_token.cancel();

        if let Some(task) = self.write_task.take()
            && let Err(error) = task.await
        {
            log::warn!("write_task join error: {error}")
        }

        if let Some(task) = self.read_task.take()
            && let Err(error) = task.await
        {
            log::warn!("read_task join error: {error}")
        }
    }
}

async fn read_handler(
    read_side: OwnedReadHalf,
    connection_incoming_queue_sender: Sender<RawPacket>,
    shutdown_token: CancellationToken,
) {
    // Craete the packet framer
    let decoder = MinecraftPacketDecoder::new();
    let mut reader = FramedRead::new(read_side, decoder);

    // iterate until state is closed
    while !connection_incoming_queue_sender.is_closed() {
        // pull next packet from framer
        let potential_next_packet = tokio::select! {
            _ = shutdown_token.cancelled() => {
                log::debug!("Read handler shutting down");
                break;
            }

            packet = reader.next() => packet,
        };

        // unwrap to see if a packet was consumed
        let next_packet_result = match potential_next_packet {
            Some(next_packet_result) => next_packet_result,
            None => break,
        };

        // check if packet framer encountered error
        let next_packet = match next_packet_result {
            Ok(packet) => packet,
            Err(error) => {
                error!("Failed to decode packet: {error}");
                break;
            }
        };

        // send to incoming queue
        if connection_incoming_queue_sender
            .send(next_packet)
            .await
            .is_err()
        {
            break;
        }
    }
}

async fn write_handler(
    write_side: OwnedWriteHalf,
    mut connection_outgoing_queue_receiver: Receiver<RawPacket>,
    shutdown_token: CancellationToken,
) {
    // Create output stream decoder
    let encoder = MinecraftPacketEncoder::new();
    let mut writer = FramedWrite::new(write_side, encoder);

    loop {
        tokio::select! {
            biased;

            _ = shutdown_token.cancelled() => {
                log::debug!("Write handler shutting down");

                // Flush the queue
                while let Ok(packet) = connection_outgoing_queue_receiver.try_recv() {
                    log::trace!("Write task writing packet: {packet}");
                    if let Err(error) = writer.send(packet).await {
                        log::error!("Failed to send packet during shutdown: {error}");
                        return;
                    }
                }

                break;
            }

            next_packet = connection_outgoing_queue_receiver.recv() => {
                match next_packet {
                    Some(packet) => {
                        log::trace!("Write task writing packet: {packet}");
                        if let Err(error) = writer.send(packet).await {
                            log::error!("Failed to send packet: {error}");
                            break;
                        }
                    }
                    None => {
                        log::debug!("Connection output stream closed");
                        break;
                    }
                }
            }
        }
    }

    log::debug!("Flushing Write Queue");
    if let Err(error) = writer.flush().await {
        log::error!("Failed to flush connection: {error}");
    }

    log::debug!("Shutting down TCP connection");
    if let Err(error) = writer.into_inner().shutdown().await {
        log::error!("Failed to shutdown TCP Connection: {error}")
    }
}
