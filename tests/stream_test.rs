// Copyright (c) Roc Streaming authors
// Licensed under MPL-2.0
mod test_client;
mod test_server;

use crate::test_client::Client;
use crate::test_client::types::*;
use crate::test_server::Server;

use reqwest::StatusCode;

#[tokio::test]
async fn test_list_streams() {
    let server = Server::new();
    let client = Client::new(server.url());

    {
        // GET /streams
        let resp = client.list_streams().await.unwrap();

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
}

#[tokio::test]
async fn test_read_stream() {
    let server = Server::new();
    let client = Client::new(server.url());

    {
        // GET /streams/{stream_uid}
        let resp = client.read_stream("777777-888888-999999").await.unwrap();

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
}
