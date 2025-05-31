// Copyright (c) Roc Peering authors
// Licensed under MPL-2.0
mod test_client;
mod test_driver;
mod test_server;

use crate::test_client::Client;
use crate::test_client::types::*;
use crate::test_driver::MockDriver;
use crate::test_server::Server;
use rocd::drivers::Driver;

use reqwest::StatusCode;
use std::sync::Arc;
use test_context::{AsyncTestContext, test_context};
use tracing_test::traced_test;

struct PeerTestContext {
    driver: Arc<dyn Driver>,
    server: Server,
    client: Client,
}

impl AsyncTestContext for PeerTestContext {
    async fn setup() -> PeerTestContext {
        let driver = MockDriver::open().await.unwrap();
        let server = Server::start(&driver).await;
        let client = Client::new(server.url());

        PeerTestContext { driver, server, client }
    }

    async fn teardown(self) {
        self.server.shutdown().await;
        self.driver.close().await;
    }
}

#[test_context(PeerTestContext)]
#[tokio::test]
#[traced_test]
async fn test_list_peers(ctx: &mut PeerTestContext) {
    // GET /peers
    let resp = ctx.client.list_peers().await.unwrap();

    assert_eq!(resp.status(), StatusCode::OK);
    assert_eq!(
        resp.into_inner(),
        vec![PeerSpec {
            peer_uri: "/peers/777777-888888-999999".into(),
            peer_uid: "777777-888888-999999".into(),
        }],
    );
}

#[test_context(PeerTestContext)]
#[tokio::test]
#[traced_test]
async fn test_read_peer(ctx: &mut PeerTestContext) {
    for peer in ["777777-888888-999999", "self"] {
        // GET /peers/{peer_uid}
        let resp = ctx.client.read_peer(peer).await.unwrap();

        assert_eq!(resp.status(), StatusCode::OK);
        assert_eq!(
            resp.into_inner(),
            PeerSpec {
                peer_uri: "/peers/777777-888888-999999".into(),
                peer_uid: "777777-888888-999999".into(),
            },
        );
    }
}
