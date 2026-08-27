use std::net::TcpStream;
use tokio::sync::watch;

use crate::{
    backend::{ProxyBackend, ProxyBackendError, ProxyBackendHandle},
    connection::Connection,
};

pub struct ProxyServerBackend {
    address: String,
    port: u16,
    connection: Option<Connection>,
    active_trigger: watch::Sender<bool>,
    active_sink: watch::Receiver<bool>,
}

impl ProxyServerBackend {
    pub(crate) fn new(address: &str, port: u16) -> Self {
        let (active_trigger, active_sink) = watch::channel(true);
        ProxyServerBackend {
            address: address.to_string(),
            port,
            connection: None,
            active_trigger,
            active_sink,
        }
    }
}

impl ProxyServerBackend {
    async fn handle_reading(self) {}
}

impl ProxyBackend for ProxyServerBackend {
    fn init(&mut self) -> Result<ProxyBackendHandle, ProxyBackendError> {
        let tcp_stream = match TcpStream::connect(format!("{}:{}", self.address, self.port)) {
            Ok(conn) => conn,
            Err(e) => {
                log::error!(
                    "Unable to connect to backed server: {}, reason: {:?}",
                    self.address,
                    e
                );
                return Err(ProxyBackendError::FailedToInit(
                    "Could not connect to server".to_string(),
                ));
            }
        };
        tcp_stream.set_nonblocking(true)?;
        let (mut conn, handle) = Connection::new(tokio::net::TcpStream::from_std(tcp_stream)?);
        conn.dispatch();
        self.connection = Some(conn);
        Ok(handle.into())
    }

    // fn send_packet(&self, packet: RawPacket) -> Result<(), ProxyBackendError> {
    //     match &self.connection_handler {
    //         Some(handle) => match handle.try_send(packet) {
    //             Ok(()) => Ok(()),
    //             Err(TrySendError::Closed(_)) => {
    //                 log::error!("Handle Closed....");
    //                 Err(InvalidState("Handle Terminated".to_string()))
    //             }
    //             Err(TrySendError::Full(_)) => {
    //                 log::error!("Queue is full");
    //                 Err(InvalidState("Queue Full".to_string()))
    //             }
    //         },
    //         None => {
    //             log::error!("Handle terminated....");
    //             Err(InvalidState("Handle Terminated".to_string()))
    //         }
    //     }
    // }

    // fn pull_packet(&mut self) -> Result<Option<RawPacket>, ProxyBackendError> {
    //     match &mut self.connection_handler {
    //         Some(handle) => match handle.try_recv() {
    //             Ok(packet) => Ok(Some(packet)),
    //             Err(TryRecvError::Empty) => Ok(None),
    //             Err(TryRecvError::Disconnected) => {
    //                 log::error!("Handle Terminated....");
    //                 Err(InvalidState("Handle Terminated".to_string()))
    //             }
    //         },
    //         None => {
    //             log::error!("Handle terminated....");
    //             Err(InvalidState("Handle Terminated".to_string()))
    //         }
    //     }
    // }

    // fn wait_for_packet(&mut self) -> impl Future<Output = Result<RawPacket, ProxyBackendError>> {
    //     async {
    //         match &mut self.connection_handler {
    //             Some(handle) => match handle.recv().await {
    //                 Some(packet) => Ok(packet),
    //                 None => Err(InvalidState("Channel has been closed".to_string())),
    //             },
    //             None => {
    //                 log::error!("Handle terminated....");
    //                 Err(InvalidState("Handle Terminated".to_string()))
    //             }
    //         }
    //     }
    // }

    fn shutdown(&mut self) -> Result<(), ProxyBackendError> {
        self.active_trigger
            .send(false)
            .expect("Active watch should be valid for lifetime of Backend");
        todo!()
    }
}
