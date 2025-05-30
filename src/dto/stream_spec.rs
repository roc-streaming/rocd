// Copyright (c) Roc Streaming authors
// Licensed under MPL-2.0
use crate::dto::uid::*;
use crate::dto::uri::*;
use crate::dto::validate::*;

use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(Clone, PartialEq, Debug, Serialize, Deserialize, ToSchema)]
pub struct StreamSpec {
    #[schema(value_type = String)]
    pub stream_uri: Uri,

    #[schema(value_type = String)]
    pub stream_uid: Uid,

    pub source: ConnectionSpec,
    pub destination: ConnectionSpec,
}

impl Validate for StreamSpec {
    fn validate(&self) -> ValidationResult {
        if self.stream_uri.kind() != UriKind::Stream {
            return Err(ValidationError::LayoutError("unexpected stream_uri format".into()));
        }

        if self.stream_uri.stream_uid().unwrap() != self.stream_uid {
            return Err(ValidationError::LayoutError(
                "UID mismatch in stream_uri and stream_uid fields".into(),
            ));
        }

        self.source.validate()?;
        self.destination.validate()?;

        Ok(())
    }
}

#[derive(Copy, Clone, PartialEq, Debug, strum::Display, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "snake_case")]
#[strum(serialize_all = "snake_case")]
pub enum ConnectionType {
    Endpoint,
    External,
}

#[derive(Clone, PartialEq, Debug, Serialize, Deserialize, ToSchema)]
#[serde(untagged, rename_all = "snake_case")]
pub enum ConnectionSpec {
    #[schema(title = "EndpointConnection")]
    Endpoint {
        // connection_type must be "endpoint"
        connection_type: ConnectionType,

        #[schema(value_type = String)]
        endpoint_uri: Uri,
    },
    #[schema(title = "ExternalConnection")]
    External {
        // connection_type must be "external"
        connection_type: ConnectionType,

        #[schema(value_type = String)]
        media_uri: Uri,
        #[schema(value_type = String)]
        repair_uri: Uri,
        #[schema(value_type = String)]
        control_uri: Uri,
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
            return Err(ValidationError::LayoutError(format!(
                "when connection_type is '{}', allowed fields are: {}",
                specified_type,
                match specified_type {
                    ConnectionType::Endpoint => "endpoint_uri",
                    ConnectionType::External => "media_uri, repair_uri, control_uri",
                }
            )));
        }

        match self {
            ConnectionSpec::Endpoint { endpoint_uri, .. } => {
                if endpoint_uri.kind() != UriKind::Endpoint {
                    return Err(ValidationError::LayoutError(
                        "unexpected endpoint_uri format".into(),
                    ));
                }
            },
            ConnectionSpec::External { media_uri, repair_uri, control_uri, .. } => {
                if media_uri.kind() != UriKind::External {
                    return Err(ValidationError::LayoutError(
                        "unexpected media_uri format, must be external URI".into(),
                    ));
                }
                if repair_uri.kind() != UriKind::External {
                    return Err(ValidationError::LayoutError(
                        "unexpected repair_uri format, must be external URI".into(),
                    ));
                }
                if control_uri.kind() != UriKind::External {
                    return Err(ValidationError::LayoutError(
                        "unexpected control_uri format, must be external URI".into(),
                    ));
                }
            },
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
        let peer_uid = Uid::generate_random();
        let endpoint_uid = Uid::generate_random();
        let stream_uid = Uid::generate_random();

        let media_uri = Uri::parse("rtp+rs8m://192.168.0.101:10000").unwrap();
        let repair_uri = Uri::parse("rs8m://192.168.0.101:10001").unwrap();
        let control_uri = Uri::parse("rtcp://192.168.0.101:10002").unwrap();

        let good_spec = StreamSpec {
            stream_uri: Uri::from_stream(&stream_uid),
            stream_uid: stream_uid,
            source: ConnectionSpec::Endpoint {
                connection_type: ConnectionType::Endpoint,
                endpoint_uri: Uri::from_endpoint(&peer_uid, &endpoint_uid),
            },
            destination: ConnectionSpec::External {
                connection_type: ConnectionType::External,
                media_uri: media_uri.clone(),
                repair_uri: repair_uri.clone(),
                control_uri: control_uri.clone(),
            },
        };

        assert_ok!(good_spec.validate());

        let bad_specs = vec![
            // invalid stream_uri type
            {
                let mut spec = good_spec.clone();
                spec.stream_uri = Uri::from_peer(&stream_uid);
                spec
            },
            // UID mismatch in stream_uri and stream_uid
            {
                let mut spec = good_spec.clone();
                spec.stream_uid = Uid::generate_random();
                spec
            },
            // source.connection_type inconsistent with other fields
            {
                let mut spec = good_spec.clone();
                spec.source = ConnectionSpec::Endpoint {
                    connection_type: ConnectionType::External,
                    endpoint_uri: Uri::from_endpoint(&peer_uid, &endpoint_uid),
                };
                spec
            },
            // destination.connection_type inconsistent with other fields
            {
                let mut spec = good_spec.clone();
                spec.destination = ConnectionSpec::External {
                    connection_type: ConnectionType::Endpoint,
                    media_uri: media_uri.clone(),
                    repair_uri: repair_uri.clone(),
                    control_uri: control_uri.clone(),
                };
                spec
            },
            // invalid endpoint_uri type
            {
                let mut spec = good_spec.clone();
                spec.source = ConnectionSpec::Endpoint {
                    connection_type: ConnectionType::Endpoint,
                    endpoint_uri: Uri::from_peer(&peer_uid),
                };
                spec
            },
            // invalid media_uri type
            {
                let mut spec = good_spec.clone();
                spec.destination = ConnectionSpec::External {
                    connection_type: ConnectionType::Endpoint,
                    media_uri: Uri::from_peer(&peer_uid),
                    repair_uri: repair_uri.clone(),
                    control_uri: control_uri.clone(),
                };
                spec
            },
            // invalid repair_uri type
            {
                let mut spec = good_spec.clone();
                spec.destination = ConnectionSpec::External {
                    connection_type: ConnectionType::Endpoint,
                    media_uri: media_uri.clone(),
                    repair_uri: Uri::from_peer(&peer_uid),
                    control_uri: control_uri.clone(),
                };
                spec
            },
            // invalid control_uri type
            {
                let mut spec = good_spec.clone();
                spec.destination = ConnectionSpec::External {
                    connection_type: ConnectionType::Endpoint,
                    media_uri: media_uri.clone(),
                    repair_uri: repair_uri.clone(),
                    control_uri: Uri::from_peer(&peer_uid),
                };
                spec
            },
        ];

        for spec in &bad_specs {
            assert_matches!(spec.validate(), Err(ValidationError::LayoutError(_)));
        }
    }
}
