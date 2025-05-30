// Copyright (c) Roc Streaming authors
// Licensed under MPL-2.0
use crate::dto::uid::*;
use crate::dto::uri::*;
use crate::dto::validate::*;

use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(Clone, PartialEq, Debug, Serialize, Deserialize, ToSchema)]
pub struct EndpointSpec {
    #[schema(value_type = String)]
    pub endpoint_uri: Uri,

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
        if self.endpoint_uri.kind() != UriKind::Endpoint {
            return Err(ValidationError::LayoutError("unexpected endpoint_uri format".into()));
        }

        if self.endpoint_uri.endpoint_uid().unwrap() != self.endpoint_uid {
            return Err(ValidationError::LayoutError(
                "UID mismatch in endpoint_uri and endpoint_uid fields".into(),
            ));
        }

        Ok(())
    }
}

#[derive(Copy, Clone, PartialEq, Debug, strum::Display, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "snake_case")]
#[strum(serialize_all = "snake_case")]
pub enum EndpointType {
    SystemDevice,
    StreamingDevice,
}

#[derive(Copy, Clone, PartialEq, Debug, strum::Display, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "snake_case")]
#[strum(serialize_all = "snake_case")]
pub enum EndpointDir {
    Input,
    Output,
    Duplex,
}

#[derive(Copy, Clone, PartialEq, Debug, strum::Display, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "snake_case")]
#[strum(serialize_all = "snake_case")]
pub enum EndpointDriver {
    Pipewire,
}

#[cfg(test)]
mod tests {
    use super::*;
    use assertables::*;

    #[test]
    fn test_validate() {
        let peer_uid = Uid::generate_random();
        let endpoint_uid = Uid::generate_random();

        let good_spec = EndpointSpec {
            endpoint_uri: Uri::from_endpoint(&peer_uid, &endpoint_uid),
            endpoint_uid,
            endpoint_type: EndpointType::SystemDevice,
            stream_direction: EndpointDir::Duplex,
            driver: EndpointDriver::Pipewire,
            display_name: "test".into(),
            system_name: "test".into(),
        };

        assert_ok!(good_spec.validate());

        let bad_specs = vec![
            // invalid endpoint_uri type
            {
                let mut spec = good_spec.clone();
                spec.endpoint_uri = Uri::from_peer(&peer_uid);
                spec
            },
            // UID mismatch in endpoint_uri and endpoint_uid
            {
                let mut spec = good_spec.clone();
                spec.endpoint_uid = Uid::generate_random();
                spec
            },
        ];

        for spec in &bad_specs {
            assert_matches!(spec.validate(), Err(ValidationError::LayoutError(_)));
        }
    }
}
