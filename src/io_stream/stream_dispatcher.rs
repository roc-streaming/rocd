// Copyright (c) Roc Streaming authors
// Licensed under MPL-2.0
use crate::dto::*;

pub struct StreamDispatcher {
    // TODO
}

impl StreamDispatcher {
    pub fn new() -> Self {
        StreamDispatcher {}
    }

    pub async fn get_all(&self) -> Vec<StreamSpec> {
        vec![self.get_stream("77-88-99").await]
    }

    pub async fn get_stream(&self, stream_uid: &str) -> StreamSpec {
        StreamSpec {
            stream_uri: format!("/streams/{stream_uid}"),
            stream_uid: stream_uid.into(),
            source: ConnectionSpec::Endpoint {
                connection_type: ConnectionType::Endpoint,
                endpoint_uri: "/peers/11-22-33/endpoints/44-55-66".into(),
            },
            destination: ConnectionSpec::External {
                connection_type: ConnectionType::External,
                media_uri: "rtp+rs8m://192.168.0.101:10000".into(),
                repair_uri: "rs8m://192.168.0.101:10001".into(),
                control_uri: "rtcp://192.168.0.101:10002".into(),
            },
        }
    }
}
