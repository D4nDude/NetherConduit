use log::info;

mod backend;
mod connection;
mod core;

#[tokio::main]
async fn main() {
    simple_logger::init().unwrap();

    let pid = std::process::id();

    info!("Starting Proxy! pid: {pid}");
    let proxy_config = core::proxy::ProxyConfig::new();
    core::proxy::start_proxy(proxy_config).await;
}
