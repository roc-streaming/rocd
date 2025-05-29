// Copyright (c) Roc Streaming authors
// Licensed under MPL-2.0
use crate::dto::uid::*;
use crate::dto::validate::*;

use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(Clone, PartialEq, Debug, Serialize, Deserialize, ToSchema)]
pub struct EndpointSpec {
    pub endpoint_uri: String,

    #[schema(value_type = String)]
    pub peer_uid: Uid,
    #[schema(value_type = String)]
    pub endpoint_uid: Uid,

    pub endpoint_type: EndpointType,
    pub stream_direction: EndpointDir,
    pub driver: EndpointDriver,

    pub display_name: String,
    pub system_name: String,
}

impl Validate for EndpointSpec {
    fn validate(&self) -> ValidationResult {
        Ok(())
    }
}

#[derive(Copy, Clone, PartialEq, Debug, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum EndpointType {
    SystemDevice,
    StreamingDevice,
}

#[derive(Copy, Clone, PartialEq, Debug, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum EndpointDir {
    Input,
    Output,
    Duplex,
}

#[derive(Copy, Clone, PartialEq, Debug, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum EndpointDriver {
    Pipewire,
}

#[cfg(test)]
mod tests {
    use super::*;
    use assertables::*;

    #[test]
    fn test_validate() {
        let good_spec = EndpointSpec {
            endpoint_uri: "test".into(),
            peer_uid: Uid::generate_random(),
            endpoint_uid: Uid::generate_random(),
            endpoint_type: EndpointType::SystemDevice,
            stream_direction: EndpointDir::Duplex,
            driver: EndpointDriver::Pipewire,
            display_name: "test".into(),
            system_name: "test".into(),
        };

        assert_ok!(good_spec.validate());
    }
}
