// Connection handler
use futures::sink::SinkExt;
use log::error;
use netherconduit_core::packet::RawPacket;
use netherconduit_core::packet::stream::{
    decoder::MinecraftPacketDecoder, encoder::MinecraftPacketEncoder,
};
use tokio::net::TcpStream;
use tokio::net::tcp::{OwnedReadHalf, OwnedWriteHalf};
use tokio::sync::mpsc::error::{SendError, TryRecvError, TrySendError};
use tokio::sync::mpsc::{self, Receiver, Sender};
use tokio::task::JoinHandle;
use tokio_stream::StreamExt;
use tokio_util::codec::{FramedRead, FramedWrite};

pub(crate) mod player_connection;

#[derive(Debug)]
pub struct Connection {
    connection_outgoing_queue_receiver: Option<Receiver<RawPacket>>, // internal reciever for sending packets
    connection_incoming_queue_sender: Sender<RawPacket>, // queue to send recived packets onward
    tcp_stream: Option<TcpStream>,
    write_task: Option<JoinHandle<OwnedWriteHalf>>,
    read_task: Option<JoinHandle<OwnedReadHalf>>,
}

#[derive(Debug)]
pub struct ConnectionHandle {
    pub incoming: Receiver<RawPacket>,
    pub outgoing: Sender<RawPacket>,
}

impl ConnectionHandle {
    pub async fn send(&self, packet: RawPacket) -> Result<(), SendError<RawPacket>> {
        self.outgoing.send(packet).await
    }

    pub fn try_send(&self, packet: RawPacket) -> Result<(), TrySendError<RawPacket>> {
        self.outgoing.try_send(packet)
    }

    pub async fn recv(&mut self) -> Option<RawPacket> {
        self.incoming.recv().await
    }

    pub fn try_recv(&mut self) -> Result<RawPacket, TryRecvError> {
        self.incoming.try_recv()
    }
}

impl Connection {
    pub(crate) fn new(tcp_stream: TcpStream) -> (Connection, ConnectionHandle) {
        // Create the queue four our sender
        let (connection_incoming_queue_sender, connection_incomming_queue_receiver) =
            mpsc::channel(64);
        let (connection_outgoing_queue_sender, connection_outgoing_queue_receiver) =
            mpsc::channel(64);
        (
            Connection {
                connection_outgoing_queue_receiver: Some(connection_outgoing_queue_receiver),
                connection_incoming_queue_sender,
                tcp_stream: Some(tcp_stream),
                write_task: None,
                read_task: None,
            },
            ConnectionHandle {
                incoming: connection_incomming_queue_receiver,
                outgoing: connection_outgoing_queue_sender,
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
        )));

        self.read_task = Some(tokio::spawn(read_handler(
            read_side,
            self.connection_incoming_queue_sender.clone(),
        )));
    }

    pub(crate) fn collect(&mut self) {}
}

async fn read_handler(
    read_side: OwnedReadHalf,
    connection_incoming_queue_sender: Sender<RawPacket>,
) -> OwnedReadHalf {
    // Craete the packet framer
    let decoder = MinecraftPacketDecoder::new();
    let mut reader = FramedRead::new(read_side, decoder);

    // iterate until state is closed
    while !connection_incoming_queue_sender.is_closed() {
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

        // send to incoming queue
        connection_incoming_queue_sender
            .send(next_packet)
            .await
            .expect("Queue should not be closed");
    }

    reader.into_inner()
}

async fn write_handler(
    write_side: OwnedWriteHalf,
    mut connection_outgoing_queue_receiver: Receiver<RawPacket>,
) -> OwnedWriteHalf {
    // Create output stream decoder
    let encoder = MinecraftPacketEncoder::new();
    let mut writer = FramedWrite::new(write_side, encoder);

    while !connection_outgoing_queue_receiver.is_closed() {
        let next_packet = match connection_outgoing_queue_receiver.recv().await {
            Some(packet) => packet,
            None => {
                log::debug!("Conneciton output stream closed");
                break;
            }
        };
        writer
            .send(next_packet)
            .await
            .expect("Queue should not be closed");
    }

    writer.into_inner()
}
