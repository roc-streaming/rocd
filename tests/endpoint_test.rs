// Copyright (c) Roc Streaming authors
// Licensed under MPL-2.0
mod test_client;
mod test_server;

use crate::test_client::Client;
use crate::test_client::types::*;
use crate::test_server::Server;

use reqwest::StatusCode;

#[tokio::test]
async fn test_list() {
    let server = Server::new();
    let client = Client::new(server.url());

    {
        // GET /peer/self/endpoints
        let resp = client.list_endpoints("11-22-33".into()).await.unwrap();

        assert_eq!(resp.status(), StatusCode::OK);
        assert_eq!(
            resp.into_inner(),
            vec![EndpointSpec {
                peer_uid: "11-22-33".into(),
                endpoint_uid: "44-55-66".into(),
                endpoint_type: EndpointType::SystemDevice.into(),
                stream_direction: EndpointDir::Output,
                driver: EndpointDriver::Pipewire,
                display_name: "Display Name".into(),
                system_name: "system_name".into(),
            }],
        );
    }
}
