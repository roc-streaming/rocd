// Copyright (c) Roc Streaming authors
// Licensed under MPL-2.0
use crate::dto::uid::*;

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

#[derive(Clone, PartialEq, Debug, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum EndpointType {
    SystemDevice,
    StreamingDevice,
}

#[derive(Clone, PartialEq, Debug, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum EndpointDir {
    Input,
    Output,
    Duplex,
}

#[derive(Clone, PartialEq, Debug, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum EndpointDriver {
    Pipewire,
}
