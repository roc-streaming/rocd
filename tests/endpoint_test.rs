// Copyright (c) Roc Streaming authors
// Licensed under MPL-2.0
mod rest_client;

use rocd::io_endpoint::EndpointDispatcher;
use rocd::io_stream::StreamDispatcher;
use rocd::rest_api::RestServer;

use std::sync::Arc;

fn make_server() -> RestServer {
    let endpoint_dispatcher = Arc::new(EndpointDispatcher::new());
    let stream_dispatcher = Arc::new(StreamDispatcher::new());

    RestServer::new(endpoint_dispatcher, stream_dispatcher)
}

#[tokio::test]
async fn test_todo() {
    let _server = make_server();

    // TODO:
    //  - run server in separate thread
    //  - get server address
    //  - create client with that address
}
