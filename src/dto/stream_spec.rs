// Copyright (c) Roc Streaming authors
// Licensed under MPL-2.0
use crate::dto::uid::*;
use crate::dto::validate::*;

use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(Clone, PartialEq, Debug, Serialize, Deserialize, ToSchema)]
pub struct StreamSpec {
    pub stream_uri: String,

    #[schema(value_type = String)]
    pub stream_uid: Uid,

    pub source: ConnectionSpec,
    pub destination: ConnectionSpec,
}

impl Validate for StreamSpec {
    fn validate(&self) -> ValidationResult {
        self.source.validate()?;
        self.destination.validate()?;

        Ok(())
    }
}

#[derive(Copy, Clone, PartialEq, Debug, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum ConnectionType {
    Endpoint,
    External,
}

#[derive(Clone, PartialEq, Debug, Serialize, Deserialize, ToSchema)]
#[serde(untagged, rename_all = "snake_case")]
pub enum ConnectionSpec {
    Endpoint {
        connection_type: ConnectionType,
        endpoint_uri: String,
    },
    External {
        connection_type: ConnectionType,
        media_uri: String,
        repair_uri: String,
        control_uri: String,
    },
}

impl Validate for ConnectionSpec {
    fn validate(&self) -> ValidationResult {
        let (specified_type, layout_type) = match self {
            ConnectionSpec::Endpoint { connection_type, .. } => {
                (*connection_type, ConnectionType::Endpoint)
            },
            ConnectionSpec::External { connection_type, .. } => {
                (*connection_type, ConnectionType::External)
            },
        };

        if specified_type != layout_type {
            match specified_type {
                ConnectionType::Endpoint => {
                    return Err(ValidationError::LayoutError(
                        "connection_type 'endpoint' requires endpoint_uri",
                    ));
                },
                ConnectionType::External => {
                    return Err(ValidationError::LayoutError(
                        "connection_type 'endpoint' requires media_uri/repair_uri/control_uri",
                    ));
                },
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use assertables::*;

    #[test]
    fn test_validate() {
        let good_spec = StreamSpec {
            stream_uri: "test".into(),
            stream_uid: Uid::generate_random(),
            source: ConnectionSpec::Endpoint {
                connection_type: ConnectionType::Endpoint,
                endpoint_uri: "test".into(),
            },
            destination: ConnectionSpec::External {
                connection_type: ConnectionType::External,
                media_uri: "test".into(),
                repair_uri: "test".into(),
                control_uri: "test".into(),
            },
        };

        assert_ok!(good_spec.validate());

        let bad_specs = vec![
            // inconsistent connection_type in source
            {
                let mut spec = good_spec.clone();
                spec.source = ConnectionSpec::Endpoint {
                    connection_type: ConnectionType::External,
                    endpoint_uri: "test".into(),
                };
                spec
            },
            // inconsistent connection_type in destination
            {
                let mut spec = good_spec.clone();
                spec.destination = ConnectionSpec::External {
                    connection_type: ConnectionType::Endpoint,
                    media_uri: "test".into(),
                    repair_uri: "test".into(),
                    control_uri: "test".into(),
                };
                spec
            },
        ];

        for spec in &bad_specs {
            assert_err!(spec.validate());
        }
    }
}
