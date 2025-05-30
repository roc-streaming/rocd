// Copyright (c) Roc Peering authors
// Licensed under MPL-2.0
mod test_client;
mod test_driver;
mod test_server;

use crate::test_client::Client;
use crate::test_client::types::*;
use crate::test_driver::MockDriver;
use crate::test_server::Server;

use reqwest::StatusCode;

#[tokio::test]
async fn test_list_peers() {
    let driver = MockDriver::open();
    let server = Server::new(&driver);
    let client = Client::new(server.url());

    {
        // GET /peers
        let resp = client.list_peers().await.unwrap();

        assert_eq!(resp.status(), StatusCode::OK);
        assert_eq!(
            resp.into_inner(),
            vec![PeerSpec {
                peer_uri: "/peers/777777-888888-999999".into(),
                peer_uid: "777777-888888-999999".into(),
            }],
        );
    }
}

#[tokio::test]
async fn test_read_peer() {
    let driver = MockDriver::open();
    let server = Server::new(&driver);
    let client = Client::new(server.url());

    for peer in ["777777-888888-999999", "self"] {
        // GET /peers/{peer_uid}
        let resp = client.read_peer(peer).await.unwrap();

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
