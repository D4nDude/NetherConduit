use std::fmt::Display;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ConnectionState {
    Status,
    Login,
    Configuration,
    Play,
    LoginDisconnect,
    Closed,
}

impl Display for ConnectionState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ConnectionState::Status => write!(f, "Status"),
            ConnectionState::Login => write!(f, "Login"),
            ConnectionState::Configuration => write!(f, "Configuration"),
            ConnectionState::Play => write!(f, "Play"),
            ConnectionState::LoginDisconnect => write!(f, "LoginDisconnect"),
            ConnectionState::Closed => write!(f, "Closed"),
        }
    }
}
