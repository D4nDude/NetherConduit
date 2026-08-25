use log::info;
use std::{
    env::{
        self,
        VarError::{NotPresent, NotUnicode},
    },
    net::SocketAddr,
};
use tokio::net::TcpListener;

use crate::connection::player_connection::PlayerConnectionManager;

#[derive(Debug)]
pub(crate) struct ProxyConfig {
    address: &'static str,
    port: u16,
    default_server: String,
}

impl ProxyConfig {
    pub(crate) fn new() -> ProxyConfig {
        let server = match env::var("DEFAULT_SERVER") {
            Ok(value) => value,
            Err(e) => match e {
                NotPresent => "localhost",
                NotUnicode(v) => {
                    log::error!("DEFAULT_SERVER is not valid unicode, got: {:?}", v);
                    "localhost"
                }
            }
            .to_string(),
        };
        let port = match env::var("DEFAULT_SERVER_PORT") {
            Ok(value) => value,
            Err(e) => match e {
                NotPresent => "25566",
                NotUnicode(v) => {
                    log::error!("DEFAULT_SERVER is not valid unicode, got: {:?}", v);
                    "25566"
                }
            }
            .to_string(),
        };
        ProxyConfig {
            address: "0.0.0.0",
            port: 25565,
            default_server: format!("{server}:{port}"),
        }
    }
}

pub(crate) async fn start_proxy(config: ProxyConfig) {
    info!("Starting Proxy with Config:\n{:?}", config);

    let addr = SocketAddr::new(config.address.parse().unwrap(), config.port);
    let listener = TcpListener::bind(addr).await.unwrap();

    while let Ok((stream, socket)) = listener.accept().await {
        info!("New Client Connection from: {:#?}", socket);
        let _joinhandle = match PlayerConnectionManager::new(stream, &config.default_server).await {
            Ok(connection_handler) => tokio::spawn(connection_handler.handle()),
            Err(e) => {
                log::error!("Could not Establish connection: {:?}", e.error);
                continue;
            }
        };
    }
}
