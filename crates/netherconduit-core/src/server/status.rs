use serde_json::Error;

use crate::server::protocol_version::ConnectionProtocolVersion;

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct ServerStatus {
    version: ConnectionProtocolVersion,
    players: Option<String>,
    description: String,
    favicon: Option<String>,
}

impl ServerStatus {
    pub fn new(
        protocol_version: ConnectionProtocolVersion,
        players: Option<&str>,
        description: &str,
    ) -> Self {
        let players = players.map(|string_slice| string_slice.to_string());
        Self {
            version: protocol_version,
            players,
            description: description.to_string(),
            favicon: None,
        }
    }

    pub fn to_json(&self) -> Result<String, Error> {
        if let Some(players_string) = &self.players {
            Ok(format!(
                "{{\"version\":{},\"players\":{},\"description\":{}}}",
                serde_json::to_string(&self.version)?,
                players_string,
                self.description
            ))
        } else {
            Ok(format!(
                "{{\"version\":{},\"description\":{}}}",
                serde_json::to_string(&self.version)?,
                self.description
            ))
        }
    }
}

#[cfg(test)]
mod test {
    use crate::server::status::ServerStatus;

    #[test]
    fn basic_serialization_test() {
        let test_status = ServerStatus::new(
            crate::server::protocol_version::ConnectionProtocolVersion::MC776,
            Some("{\"max\":20,\"online\":0}"),
            "{\"text\":\"Hello, world!\"}",
        );

        assert_eq!(
            test_status.to_json().unwrap(),
            "{\"version\":{\"name\":\"26.2\",\"protocol\":776},\"players\":{\"max\":20,\"online\":0},\"description\":{\"text\":\"Hello, world!\"}}"
        )
    }

    #[test]
    fn basic_serialization_test_no_players() {
        let test_status = ServerStatus::new(
            crate::server::protocol_version::ConnectionProtocolVersion::MC776,
            None,
            "{\"text\":\"Hello, world!\"}",
        );

        assert_eq!(
            test_status.to_json().unwrap(),
            "{\"version\":{\"name\":\"26.2\",\"protocol\":776},\"description\":{\"text\":\"Hello, world!\"}}"
        )
    }
}
