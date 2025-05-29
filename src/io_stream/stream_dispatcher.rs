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
        vec![self.get_stream(&Uid::parse("777777-888888-999999").unwrap()).await]
    }

    pub async fn get_stream(&self, stream_uid: &Uid) -> StreamSpec {
        StreamSpec {
            stream_uri: format!("/streams/{stream_uid}"),
            stream_uid: *stream_uid,
            source: ConnectionSpec::Endpoint {
                connection_type: ConnectionType::Endpoint,
                endpoint_uri: "/peers/111111-222222-333333/endpoints/444444-555555-666666"
                    .into(),
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
