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

pub struct Server {
    server: Arc<RestServer>,
    address: SocketAddr,
    url: String,
}

impl Server {
    pub async fn start(driver: &Arc<dyn Driver>) -> Self {
        let peer_dispatcher = Arc::new(PeerDispatcher::new());
        let endpoint_dispatch = Arc::new(EndpointDispatcher::new(driver));
        let stream_dispatch = Arc::new(StreamDispatcher::new(driver));

        let server =
            Arc::new(RestServer::new(&peer_dispatcher, &endpoint_dispatch, &stream_dispatch));

        let address =
            server.start(SocketAddr::from_str("127.0.0.1:0").unwrap()).await.unwrap();

        let url = format!("http://{}", address);

        Server { server, address, url }
    }

    pub async fn shutdown(&self) {
        self.server.stop().await;
        self.server.wait().await.unwrap();
    }

    pub fn address(&self) -> SocketAddr {
        self.address
    }

    pub fn url(&self) -> &str {
        &self.url
    }
}
