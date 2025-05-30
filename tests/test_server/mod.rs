// Copyright (c) Roc Streaming authors
// Licensed under MPL-2.0
use rocd::drivers::Driver;
use rocd::io_endpoints::EndpointDispatcher;
use rocd::io_streams::StreamDispatcher;
use rocd::p2p::PeerDispatcher;
use rocd::rest_api::RestServer;

use std::net::SocketAddr;
use std::str::FromStr;
use std::sync::Arc;
use std::thread::{self, JoinHandle};
use tokio::runtime;

/// Runs rocd::rest_api::RestServer on separate thread with
/// separate tokio runtime.
pub struct Server {
    server: Arc<RestServer>,
    tcp_addr: SocketAddr,
    http_url: String,
    thread_handle: Option<JoinHandle<()>>,
    thread_stopper: Option<tokio::sync::oneshot::Sender<()>>,
}

impl Server {
    /// Create and start server.
    pub fn new(driver: &Arc<dyn Driver>) -> Self {
        let peer_dispatcher = Arc::new(PeerDispatcher::new());
        let endpoint_dispatch = Arc::new(EndpointDispatcher::new(driver));
        let stream_dispatch = Arc::new(StreamDispatcher::new(driver));

        let server =
            Arc::new(RestServer::new(&peer_dispatcher, &endpoint_dispatch, &stream_dispatch));

        let (tx_addr, rx_addr) = std::sync::mpsc::channel();
        let (tx_stop, rx_stop) = tokio::sync::oneshot::channel();

        // Run server on separate run-time.
        // This gives us two benefits:
        //  - tested server doesn't interfer with test code
        //  - we can't block on server shutdown in drop()
        let thread_handle = thread::spawn({
            let server = Arc::clone(&server);

            move || {
                let tokio_runtime =
                    runtime::Builder::new_current_thread().enable_all().build().unwrap();

                tokio_runtime.block_on(async {
                    let address = server
                        .start(SocketAddr::from_str("127.0.0.1:0").unwrap())
                        .await
                        .unwrap();
                    tx_addr.send(address).unwrap();

                    _ = rx_stop.await;

                    server.stop().await;
                    _ = server.wait().await;
                });
            }
        });

        // wait until address is reported from thread
        let address = rx_addr.recv().unwrap();
        let url = format!("http://{}", address);

        Server {
            server,
            tcp_addr: address,
            http_url: url,
            thread_handle: Some(thread_handle),
            thread_stopper: Some(tx_stop),
        }
    }

    /// Get server address.
    pub fn address(&self) -> SocketAddr {
        self.tcp_addr
    }

    /// Get server URL.
    pub fn url(&self) -> &str {
        &self.http_url
    }
}

impl Drop for Server {
    /// Stop and wait server.
    fn drop(&mut self) {
        let stopper = self.thread_stopper.take().unwrap();
        stopper.send(()).unwrap();

        let handle = self.thread_handle.take().unwrap();
        handle.join().unwrap();
    }
}
