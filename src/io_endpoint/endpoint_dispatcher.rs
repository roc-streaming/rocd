// Copyright (c) Roc Streaming authors
// Licensed under MPL-2.0
use crate::dto::*;

pub struct EndpointDispatcher {
    // TODO
}

impl EndpointDispatcher {
    pub fn new() -> Self {
        EndpointDispatcher {}
    }

    pub async fn get_all(&self) -> Vec<EndpointSpec> {
        vec![self.get_endpoint("11-22-33").await]
    }

    pub async fn get_endpoint(&self, uid: &str) -> EndpointSpec {
        EndpointSpec {
            endpoint_uuid: uid.into(),
            endpoint_type: EndpointType::SystemDevice,
            stream_direction: EndpointDir::Output,
            driver: EndpointDriver::Pipewire,
            display_name: "Display Name".into(),
            system_name: "system_name".into(),
        }
    }
}
