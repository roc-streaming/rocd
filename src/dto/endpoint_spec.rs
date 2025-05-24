// Copyright (c) Roc Streaming authors
// Licensed under MPL-2.0
use salvo::oapi::ToSchema;
use serde::{Deserialize, Serialize};
use validator::Validate;

#[derive(Clone, PartialEq, Debug, Validate, ToSchema, Serialize, Deserialize)]
pub struct EndpointSpec {
    /// Globally unique endpoint identifier.
    #[validate(length(min = 1))]
    pub endpoint_uuid: String,

    /// What stands behind this endpoint - physical device, virtual device, etc.
    pub endpoint_type: EndpointType,

    /// Can it be stream input or output?
    pub stream_direction: EndpointDir,

    /// Which driver provides this endpoint - pipewire, coreaudio, etc.
    pub driver: EndpointDriver,

    /// Human-readable name.
    #[validate(length(min = 1))]
    pub display_name: String,

    /// OS name (if any).
    #[validate(length(min = 1))]
    pub system_name: String,
}

#[derive(Clone, PartialEq, Debug, ToSchema, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EndpointType {
    /// Audio device managed by OS.
    SystemDevice,
    /// Special virtual audio device managed by rocd.
    StreamingDevice,
}

#[derive(Clone, PartialEq, Debug, ToSchema, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EndpointDir {
    /// Endpoint can be stream source.
    Input,
    /// Endpoint can be stream destination.
    Output,
    /// Endpoint can be use both as source and destination.
    Duplex,
}

#[derive(Clone, PartialEq, Debug, ToSchema, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EndpointDriver {
    Pipewire,
}
