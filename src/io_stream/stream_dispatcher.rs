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
        vec![self.get_stream("11-22-33").await]
    }

    pub async fn get_stream(&self, uid: &str) -> StreamSpec {
        StreamSpec { stream_uid: uid.into(), sources: vec![], destinations: vec![] }
    }
}
