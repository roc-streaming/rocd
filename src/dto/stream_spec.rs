// Copyright (c) Roc Streaming authors
// Licensed under MPL-2.0
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(Clone, PartialEq, Debug, Serialize, Deserialize, ToSchema)]
#[schema(title = "StreamSpec")]
pub struct StreamSpec {
    pub stream_uri: String,

    pub stream_uid: String,

    pub source: ConnectionSpec,
    pub destination: ConnectionSpec,
}

#[derive(Clone, PartialEq, Debug, Serialize, Deserialize, ToSchema)]
#[schema(title = "ConnectionType")]
#[serde(rename_all = "snake_case")]
pub enum ConnectionType {
    Endpoint,
    External,
}

#[derive(Clone, PartialEq, Debug, Serialize, Deserialize, ToSchema)]
#[schema(title = "ConnectionSpec")]
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
