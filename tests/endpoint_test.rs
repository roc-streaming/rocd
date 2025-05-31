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

struct EndpointTestContext {
    driver: Arc<dyn Driver>,
    server: Server,
    client: Client,
}

impl AsyncTestContext for EndpointTestContext {
    async fn setup() -> EndpointTestContext {
        let driver = MockDriver::open().await.unwrap();
        let server = Server::start(&driver).await;
        let client = Client::new(server.url());

        EndpointTestContext { driver, server, client }
    }

    async fn teardown(self) {
        self.server.shutdown().await;
        self.driver.close().await;
    }
}

#[test_context(EndpointTestContext)]
#[tokio::test]
#[traced_test]
async fn test_list_endpoints(ctx: &mut EndpointTestContext) {
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

#[test_context(EndpointTestContext)]
#[tokio::test]
#[traced_test]
async fn test_read_endpoint(ctx: &mut EndpointTestContext) {
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
