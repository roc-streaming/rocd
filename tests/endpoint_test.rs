// Copyright (c) Roc Streaming authors
// Licensed under MPL-2.0
mod test_client;
mod test_server;

use crate::test_client::Client;
use crate::test_client::types::*;
use crate::test_server::Server;

use reqwest::StatusCode;

#[tokio::test]
async fn test_endpoint_list() {
    let server = Server::new();
    let client = Client::new(server.url());

    {
        // GET /peers/{peer_uid}/endpoints
        let resp = client.list_endpoints("111111-222222-333333").await.unwrap();

        assert_eq!(resp.status(), StatusCode::OK);
        assert_eq!(
            resp.into_inner(),
            vec![EndpointSpec {
                endpoint_uri: "/peers/111111-222222-333333/endpoints/444444-555555-666666"
                    .into(),
                peer_uid: "111111-222222-333333".into(),
                endpoint_uid: "444444-555555-666666".into(),
                endpoint_type: EndpointType::SystemDevice,
                stream_direction: EndpointDir::Output,
                driver: EndpointDriver::Pipewire,
                display_name: "Display Name".into(),
                system_name: "system_name".into(),
            }],
        );
    }
}
