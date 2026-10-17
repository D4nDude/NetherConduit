use crate::server::protocol_version::ConnectionProtocolVersion;

#[derive()]
pub enum PacketType {
    // Handshake
    Handshake,
    // Clientbound Status
    StatusResponse,
    PongResponse,
    // ServerBound Status
    StatusRequest,
    PingRequest,
    // Clientbound Login
    DisconnectLogin,
    EncryptionRequest,
    LoginSuccess,
    SetCompression,
    LoginPluginRequest,
    CookieRequest,
    // Serverbound Login
    LoginStart,
    EncryptionResponse,
    LoginPluginResponse,
    LoginAcknowledged,
    CookieResponse,
}

pub fn get_packet_id(protocol_version: ConnectionProtocolVersion, packet_type: PacketType) -> u32 {
    match protocol_version {
        ConnectionProtocolVersion::MC776 => match packet_type {
            PacketType::Handshake => 0,
            PacketType::StatusResponse => 0,
            PacketType::PongResponse => 1,
            PacketType::StatusRequest => 0,
            PacketType::PingRequest => 1,
            PacketType::DisconnectLogin => 0,
            PacketType::EncryptionRequest => 1,
            PacketType::LoginSuccess => 2,
            PacketType::SetCompression => 3,
            PacketType::LoginPluginRequest => 4,
            PacketType::CookieRequest => 5,
            PacketType::LoginStart => 0,
            PacketType::EncryptionResponse => 1,
            PacketType::LoginPluginResponse => 2,
            PacketType::LoginAcknowledged => 3,
            PacketType::CookieResponse => 4,
        },
        _ => todo!("Protocol not implemented"),
    }
}
