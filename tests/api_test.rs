// Copyright (c) Roc Streaming authors
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

struct ApiTestContext {
    driver: Arc<dyn Driver>,
    server: Server,
    client: Client,
}

impl AsyncTestContext for ApiTestContext {
    async fn setup() -> ApiTestContext {
        let driver = MockDriver::open().await.unwrap();
        let server = Server::start(&driver).await;
        let client = Client::new(server.url());

        ApiTestContext { driver, server, client }
    }

    async fn teardown(self) {
        self.server.shutdown().await;
        self.driver.close().await;
    }
}

#[test_context(ApiTestContext)]
#[tokio::test]
#[traced_test]
async fn test_list_peers(ctx: &mut ApiTestContext) {
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

#[test_context(ApiTestContext)]
#[tokio::test]
#[traced_test]
async fn test_read_peer(ctx: &mut ApiTestContext) {
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

#[test_context(ApiTestContext)]
#[tokio::test]
#[traced_test]
async fn test_list_endpoints(ctx: &mut ApiTestContext) {
    for peer in ["111111-222222-333333", "self"] {
        // GET /peers/{peer_uid}/endpoints
        let resp = ctx.client.list_endpoints(peer).await.unwrap();

        assert_eq!(resp.status(), StatusCode::OK);
        assert_eq!(
            resp.into_inner(),
            vec![EndpointSpec {
                endpoint_uri: "/peers/111111-222222-333333/endpoints/444444-555555-666666"
                    .into(),
                endpoint_uid: "444444-555555-666666".into(),
                endpoint_type: EndpointType::SystemDevice,
                stream_direction: EndpointDir::Output,
                driver: DriverId::Pipewire,
                display_name: "Display Name".into(),
                system_name: "system_name".into(),
            }],
        );
    }
}

#[test_context(ApiTestContext)]
#[tokio::test]
#[traced_test]
async fn test_read_endpoint(ctx: &mut ApiTestContext) {
    for peer in ["777777-888888-999999", "self"] {
        // GET /peers/{peer_uid}/endpoints/{endpoint_uid}
        let resp = ctx.client.read_endpoint(peer, "444444-555555-666666").await.unwrap();

        assert_eq!(resp.status(), StatusCode::OK);
        assert_eq!(
            resp.into_inner(),
            EndpointSpec {
                endpoint_uri: "/peers/777777-888888-999999/endpoints/444444-555555-666666"
                    .into(),
                endpoint_uid: "444444-555555-666666".into(),
                endpoint_type: EndpointType::SystemDevice,
                stream_direction: EndpointDir::Output,
                driver: DriverId::Pipewire,
                display_name: "Display Name".into(),
                system_name: "system_name".into(),
            },
        );
    }
}

#[test_context(ApiTestContext)]
#[tokio::test]
#[traced_test]
async fn test_list_streams(ctx: &mut ApiTestContext) {
    // GET /streams
    let resp = ctx.client.list_streams().await.unwrap();

    assert_eq!(resp.status(), StatusCode::OK);
    assert_eq!(
        resp.into_inner(),
        vec![StreamSpec {
            stream_uri: "/streams/777777-888888-999999".into(),
            stream_uid: "777777-888888-999999".into(),
            source: ConnectionSpec::EndpointConnection {
                connection_type: ConnectionType::Endpoint,
                endpoint_uri: "/peers/111111-222222-333333/endpoints/444444-555555-666666"
                    .into(),
            },
            destination: ConnectionSpec::ExternalConnection {
                connection_type: ConnectionType::External,
                media_uri: "rtp+rs8m://192.168.0.101:10000".into(),
                repair_uri: "rs8m://192.168.0.101:10001".into(),
                control_uri: "rtcp://192.168.0.101:10002".into(),
            },
        }],
    );
}

#[test_context(ApiTestContext)]
#[tokio::test]
#[traced_test]
async fn test_read_stream(ctx: &mut ApiTestContext) {
    // GET /streams/{stream_uid}
    let resp = ctx.client.read_stream("777777-888888-999999").await.unwrap();

    assert_eq!(resp.status(), StatusCode::OK);
    assert_eq!(
        resp.into_inner(),
        StreamSpec {
            stream_uri: "/streams/777777-888888-999999".into(),
            stream_uid: "777777-888888-999999".into(),
            source: ConnectionSpec::EndpointConnection {
                connection_type: ConnectionType::Endpoint,
                endpoint_uri: "/peers/111111-222222-333333/endpoints/444444-555555-666666"
                    .into(),
            },
            destination: ConnectionSpec::ExternalConnection {
                connection_type: ConnectionType::External,
                media_uri: "rtp+rs8m://192.168.0.101:10000".into(),
                repair_uri: "rs8m://192.168.0.101:10001".into(),
                control_uri: "rtcp://192.168.0.101:10002".into(),
            },
        },
    );
}
