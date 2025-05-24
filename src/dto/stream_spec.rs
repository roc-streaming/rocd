// Copyright (c) Roc Streaming authors
// Licensed under MPL-2.0
use salvo::oapi::ToSchema;
use serde::{Deserialize, Serialize};
use validator::Validate;

#[derive(Clone, PartialEq, Debug, Validate, ToSchema, Serialize, Deserialize)]
pub struct StreamSpec {
    /// Globally unique stream identifier.
    #[validate(length(min = 1))]
    pub stream_uuid: String,

    /// From where this stream reads audio.
    #[validate(length(min = 1))]
    pub sources: Vec<StreamAnchorSpec>,

    /// To where this stream writes audio.
    #[validate(length(min = 1))]
    pub destinations: Vec<StreamAnchorSpec>,
}

#[derive(Clone, PartialEq, Debug, ToSchema, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum StreamAnchorType {
    Endpoint,
    Address,
}

#[derive(Clone, PartialEq, Debug, ToSchema, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum StreamAnchorSpec {
    Endpoint(EndpointAnchorSpec),
    Address(AddressAnchorSpec),
}

#[derive(Clone, PartialEq, Debug, ToSchema, Serialize, Deserialize)]
pub struct EndpointAnchorSpec {
    #[serde(rename = "type")]
    pub anchor_type: StreamAnchorType,

    pub peer_uuid: String,
    pub endpoint_uuid: String,
}

#[derive(Clone, PartialEq, Debug, ToSchema, Serialize, Deserialize)]
pub struct AddressAnchorSpec {
    #[serde(rename = "type")]
    pub anchor_type: StreamAnchorType,

    pub source_uri: String,
    pub repair_uri: String,
    pub control_uri: String,
}
