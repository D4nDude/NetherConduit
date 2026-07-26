use log::info;
use std::net::SocketAddr;
use tokio::net::TcpListener;

#[derive(Debug)]
pub(crate) struct ProxyConfig {
    address: &'static str,
    port: u16,
}

impl ProxyConfig {
    pub(crate) fn new() -> ProxyConfig {
        ProxyConfig {
            address: "127.0.0.1",
            port: 25565,
        }
    }
}

pub(crate) async fn start_proxy(config: ProxyConfig) {
    info!("Starting Proxy with Config:\n{:?}", config);

    let addr = SocketAddr::new(config.address.parse().unwrap(), config.port);
    let listener = TcpListener::bind(addr).await.unwrap();

    while let Ok((_stream, socket)) = listener.accept().await {
        info!("New Client Connection from: {:#?}", socket);
    }
}
