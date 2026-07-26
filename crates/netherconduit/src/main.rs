use log::info;

mod core;

#[tokio::main]
async fn main() {
    simple_logger::init().unwrap();

    info!("Starting Proxy!");
    let proxy_config = core::proxy::ProxyConfig::new();
    core::proxy::start_proxy(proxy_config).await;
}
