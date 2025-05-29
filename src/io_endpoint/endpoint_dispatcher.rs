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

    pub async fn get_all(&self, _peer_uid: &Uid) -> Vec<EndpointSpec> {
        vec![
            self.get_endpoint(
                &Uid::parse("111111-222222-333333").unwrap(),
                &Uid::parse("444444-555555-666666").unwrap(),
            )
            .await,
        ]
    }

    pub async fn get_endpoint(&self, peer_uid: &Uid, endpoint_uid: &Uid) -> EndpointSpec {
        EndpointSpec {
            endpoint_uri: format!("/peers/{peer_uid}/endpoints/{endpoint_uid}"),
            peer_uid: *peer_uid,
            endpoint_uid: *endpoint_uid,
            endpoint_type: EndpointType::SystemDevice,
            stream_direction: EndpointDir::Output,
            driver: EndpointDriver::Pipewire,
            display_name: "Display Name".into(),
            system_name: "system_name".into(),
        }
    }
}
