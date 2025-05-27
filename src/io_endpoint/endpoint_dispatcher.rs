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

    pub async fn get_all(&self, _network_uid: &str, _peer_uid: &str) -> Vec<EndpointSpec> {
        vec![self.get_endpoint("11-22-33", "44-55-66", "77-88-99").await]
    }

    pub async fn get_endpoint(
        &self, network_uid: &str, peer_uid: &str, endpoint_uid: &str,
    ) -> EndpointSpec {
        EndpointSpec {
            endpoint_uri: format!(
                "/networks/{network_uid}/peers/{peer_uid}/endpoints/{endpoint_uid}"
            ),
            //
            network_uid: network_uid.into(),
            peer_uid: peer_uid.into(),
            endpoint_uid: endpoint_uid.into(),
            //
            endpoint_type: EndpointType::SystemDevice,
            stream_direction: EndpointDir::Output,
            driver: EndpointDriver::Pipewire,
            //
            display_name: "Display Name".into(),
            system_name: "system_name".into(),
        }
    }
}
