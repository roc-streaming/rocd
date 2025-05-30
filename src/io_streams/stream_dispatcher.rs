// Copyright (c) Roc Streaming authors
// Licensed under MPL-2.0
use crate::drivers::*;
use crate::dto::*;

use std::sync::Arc;

pub struct StreamDispatcher {
    driver: Arc<dyn Driver>,
}

impl StreamDispatcher {
    pub fn new(driver: &Arc<dyn Driver>) -> Self {
        StreamDispatcher { driver: Arc::clone(driver) }
    }

    pub async fn get_all(&self) -> Vec<StreamSpec> {
        vec![self.get_stream(&Uid::parse("777777-888888-999999").unwrap()).await]
    }

    pub async fn get_stream(&self, stream_uid: &Uid) -> StreamSpec {
        StreamSpec {
            stream_uri: Uri::from_stream(stream_uid),
            stream_uid: *stream_uid,
            source: ConnectionSpec::Endpoint {
                connection_type: ConnectionType::Endpoint,
                endpoint_uri: Uri::from_endpoint(
                    &Uid::parse("111111-222222-333333").unwrap(),
                    &Uid::parse("444444-555555-666666").unwrap(),
                ),
            },
            destination: ConnectionSpec::External {
                connection_type: ConnectionType::External,
                media_uri: Uri::parse("rtp+rs8m://192.168.0.101:10000").unwrap(),
                repair_uri: Uri::parse("rs8m://192.168.0.101:10001").unwrap(),
                control_uri: Uri::parse("rtcp://192.168.0.101:10002").unwrap(),
            },
        }
    }
}
