use log::info;
use std::{
    env::{
        self,
        VarError::{NotPresent, NotUnicode},
    },
    net::SocketAddr,
};
use tokio::{
    net::TcpListener,
    sync::watch::{self, Receiver, Ref, Sender},
};

use crate::connection::player_connection::PlayerConnectionManager;

#[derive(Debug)]
pub(crate) struct ProxyConfig {
    address: &'static str,
    port: u16,
    pub default_server: String,
    pub default_server_port: u16,
    _max_players: u32,
    pub description: String,
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
        let port: u16 = match env::var("DEFAULT_SERVER_PORT") {
            Ok(value) => value.parse::<u16>().expect("Port should be a u16 number"),
            Err(e) => match e {
                NotPresent => 25566,
                NotUnicode(v) => {
                    log::error!("DEFAULT_SERVER is not valid unicode, got: {:?}", v);
                    25566
                }
            },
        };
        ProxyConfig {
            address: "0.0.0.0",
            port: 25565,
            default_server: server,
            default_server_port: port,
            _max_players: 20,
            description: "{\"text\": \"Netherconduit Proxy\"}".to_string(),
        }
    }
}

struct Proxy {
    _configuration_pusher: Sender<ProxyConfig>,
    configuration_watcher: Receiver<ProxyConfig>,
}

impl Proxy {
    fn new(initial_config: ProxyConfig) -> Self {
        let (sender, reciever) = watch::channel(initial_config);
        Proxy {
            _configuration_pusher: sender,
            configuration_watcher: reciever,
        }
    }

    fn borrow_config<'a>(&'a self) -> Ref<'a, ProxyConfig> {
        self.configuration_watcher.borrow()
    }

    fn configuration_receiver(&self) -> Receiver<ProxyConfig> {
        self.configuration_watcher.clone()
    }
}

pub(crate) async fn start_proxy(config: ProxyConfig) {
    info!("Starting Proxy with Config:\n{:?}", config);

    let proxy = Proxy::new(config);

    let addr = SocketAddr::new(
        proxy.borrow_config().address.parse().unwrap(),
        proxy.borrow_config().port,
    );
    let listener = TcpListener::bind(addr).await.unwrap();

    while let Ok((stream, socket)) = listener.accept().await {
        info!("New Client Connection from: {:#?}", socket);
        let connection_handler =
            PlayerConnectionManager::new(stream, proxy.configuration_receiver()).await;
        let _joinhandle = tokio::spawn(connection_handler.handle());
    }
}
