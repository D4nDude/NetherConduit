use std::ptr::read;

// Connection handler
use futures::sink::SinkExt;
use log::{debug, error, info, warn};
use netherconduit_core::packet::stream::{
    decoder::MinecraftPacketDecoder, encoder::MinecraftPacketEncoder,
};
use netherconduit_core::packet::types::{
    HandshakePacket, MinecraftPacket, handshake::HandshakeIntent,
};
use netherconduit_core::packet::{ConnectionState, RawPacket};
use tokio::net::tcp::{OwnedReadHalf, OwnedWriteHalf};
use tokio::net::{TcpStream, tcp};
use tokio::sync::mpsc::{self, Receiver, Sender};
use tokio_stream::StreamExt;
use tokio_util::codec::{FramedRead, FramedWrite};

pub(crate) mod player_connection;

#[derive(Debug, PartialEq, Eq)]
pub(crate) enum ConnectionSide {
    PlayerToServer,
    ServerToPlayer,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub(crate) enum PacketAction {
    #[default]
    Forward,
    UpdateState(ConnectionState),
    Disconnect,
}

pub struct ConnectionHandler {
    state: ConnectionState, // Current connection state
    side: ConnectionSide,
    connection_send_queue_reciever: Option<Receiver<RawPacket>>, // internal reciever for sending packets
    connection_recieve_forward_queue: Option<Sender<RawPacket>>, // queue to send recived packets onward
    tcp_stream: Option<TcpStream>,
    // exposed queue to allow external writing to packet out
    connection_send_queue: Sender<RawPacket>, // queue endpoint to send packets through the connection
}

impl ConnectionHandler {
    pub(crate) fn new(
        tcp_stream: TcpStream,
        side: ConnectionSide,
        connection_recieve_forward_queue: Option<Sender<RawPacket>>,
    ) -> ConnectionHandler {
        // Create the queue four our sender
        let (connection_send_queue, output_queue) = mpsc::channel(64);
        ConnectionHandler {
            state: ConnectionState::Handshake,
            side,
            connection_recieve_forward_queue,
            connection_send_queue_reciever: Some(output_queue),
            tcp_stream: Some(tcp_stream),
            connection_send_queue,
        }
    }

    pub(crate) fn set_connection_recieve_forward_queue(&mut self, queue: Sender<RawPacket>) {
        self.connection_recieve_forward_queue = Some(queue);
    }

    pub(crate) fn get_connection_send_queue(&self) -> Sender<RawPacket> {
        self.connection_send_queue.clone()
    }

    pub(crate) async fn handle(mut self) {
        // split the tcp stream
        let (read_side, write_side) = self
            .tcp_stream
            .take()
            .expect("Stream should be available to split")
            .into_split();

        let write_task = tokio::spawn(write_handler(
            write_side,
            self.connection_send_queue_reciever
                .take()
                .expect("We need to move the mpsc pipe to the write stream"),
        ));
        tokio::join!(self.read_handler(read_side), write_task);
    }

    fn handle_handshake(packet: &RawPacket) -> PacketAction {
        let packet = match HandshakePacket::from_raw(&packet) {
            Ok(value) => value,
            Err(e) => {
                error!("Decode Error: {:?}", e);
                return PacketAction::Disconnect;
            }
        };
        match packet.intent {
            HandshakeIntent::Status => PacketAction::UpdateState(ConnectionState::Status),
            HandshakeIntent::Login => PacketAction::UpdateState(ConnectionState::Login),
            HandshakeIntent::Transfer => PacketAction::UpdateState(ConnectionState::Login),
        }
    }

    async fn read_handler(mut self, read_side: OwnedReadHalf) {
        // Craete the packet framer
        let decoder = MinecraftPacketDecoder::new();
        let mut reader = FramedRead::new(read_side, decoder);

        let connection_forward_queue = self
            .connection_recieve_forward_queue
            .expect("missing queue to send packets onward");

        // iterate until state is closed
        while self.state != ConnectionState::Closed {
            // pull next packet from framer
            let potential_next_packet = reader.next().await;

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

            // process packet based on state
            let action = match self.state {
                ConnectionState::Handshake => Self::handle_handshake(&next_packet),
                _ => {
                    error!("Connection State not implemented: {:?}", self.state);
                    PacketAction::Disconnect
                }
            };

            // pass the packet on or affect connection state
            match action {
                PacketAction::Forward => {
                    connection_forward_queue.send(next_packet);
                }
                PacketAction::UpdateState(new_state) => {
                    self.state = new_state;
                }
                PacketAction::Disconnect => {
                    error!("I dont know how to disconnect...");
                    panic!("Force Disconnect");
                }
            }
        }
    }
}

async fn write_handler(write_side: OwnedWriteHalf, mut connection_send_queue: Receiver<RawPacket>) {
    // Create output stream decoder
    let encoder = MinecraftPacketEncoder::new();
    let mut writer = FramedWrite::new(write_side, encoder);

    while !connection_send_queue.is_closed() {
        let next_packet = match connection_send_queue.recv().await {
            Some(packet) => packet,
            None => {
                log::debug!("Conneciton output stream closed");
                break;
            }
        };
        writer.send(next_packet);
    }
}
