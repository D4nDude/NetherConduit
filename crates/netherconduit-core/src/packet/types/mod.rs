use log::error;

pub mod handshake;
mod status;

pub use handshake::HandshakePacket;
pub use status::PingRequestPacket;

use crate::connection::ConnectionState;
use crate::packet::{RawPacket, stream::DecodeError};

pub trait MinecraftPacket: Sized {
    fn from_raw(raw_packet: &RawPacket) -> Result<Self, DecodeError>;
    fn packet_type() -> MinecraftPacketType;
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum MinecraftPacketType {
    Handshake,
    PingRequest,
}

pub enum Packet {
    Handshake(HandshakePacket),
}

pub fn decode_packet<T>(
    raw_packet: &RawPacket,
    connection_state: ConnectionState,
) -> Result<Packet, DecodeError> {
    match connection_state {
        ConnectionState::Handshake => handle_handshake(raw_packet, connection_state),
        unknown_state => {
            error!("Packet {:?} not yet implemented", unknown_state);
            Err(DecodeError::Invalid(format!(
                "Packet {:?} not yet implemented",
                unknown_state
            )))
        }
    }
}

fn handle_handshake(
    _raw_packet: &RawPacket,
    _connection_state: ConnectionState,
) -> Result<Packet, DecodeError> {
    todo!()
    // if raw_packet.id()?.value() == 0 {
    //     Ok(Packet::Handshake(HandshakePacket::from_raw(raw_packet)?))
    // } else {
    //     error!("Handshake packet ID: {} Invalid", raw_packet.id);
    //     Err(DecodeError::Invalid)
    // }
}
