use log::info;

mod backend;
mod connection;
mod core;

#[tokio::main]
async fn main() {
    simple_logger::init().unwrap();
    console_subscriber::ConsoleLayer::builder()
        .with_default_env()
        .server_addr(([0, 0, 0, 0], 6669))
        .init();

    let pid = std::process::id();

    info!("Starting Proxy! pid: {pid}");
    let proxy_config = core::proxy::ProxyConfig::new();
    tokio::spawn(core::proxy::start_proxy(proxy_config))
        .await
        .unwrap()
}
