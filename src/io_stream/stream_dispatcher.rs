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

    pub async fn get_all(&self, _network_uid: &str) -> Vec<StreamSpec> {
        vec![self.get_stream("11-22-33", "12-34-56").await]
    }

    pub async fn get_stream(&self, network_uid: &str, stream_uid: &str) -> StreamSpec {
        StreamSpec {
            stream_uri: format!("/networks/{network_uid}/streams/{stream_uid}"),
            //
            network_uid: network_uid.into(),
            stream_uid: stream_uid.into(),
            //
            source: ConnectionSpec::Endpoint {
                connection_type: ConnectionType::Endpoint,
                endpoint_uri: "/networks/11-22-33/peers/44-55-66/endpoints/77-88-99".into(),
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
