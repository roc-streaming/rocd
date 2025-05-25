// Copyright (c) Roc Streaming authors
// Licensed under MPL-2.0
use salvo::oapi::ToSchema;
use serde::{Deserialize, Serialize};
use validator::Validate;

#[derive(Clone, PartialEq, Debug, Validate, Serialize, Deserialize, ToSchema)]
#[salvo(schema(name = "StreamSpec"))]
pub struct StreamSpec {
    /// Globally unique stream identifier.
    #[validate(length(min = 1))]
    pub stream_uuid: String,

    /// From where this stream reads audio.
    #[validate(length(min = 1))]
    pub sources: Vec<AnchorSpec>,

    /// To where this stream writes audio.
    #[validate(length(min = 1))]
    pub destinations: Vec<AnchorSpec>,
}

#[derive(Clone, PartialEq, Debug, Serialize, Deserialize, ToSchema)]
#[salvo(schema(name = "AnchorSpec"))]
#[serde(rename_all = "snake_case")]
pub enum AnchorSpec {
    Endpoint(EndpointAnchorSpec),
    Address(AddressAnchorSpec),
}

#[derive(Clone, PartialEq, Debug, Serialize, Deserialize, ToSchema)]
#[salvo(schema(name = "AnchorType"))]
#[serde(rename_all = "snake_case")]
pub enum AnchorType {
    Endpoint,
    Address,
}

#[derive(Clone, PartialEq, Debug, Validate, Serialize, Deserialize, ToSchema)]
#[salvo(schema(name = "EndpointAnchorSpec"))]
pub struct EndpointAnchorSpec {
    #[serde(rename = "type")]
    pub anchor_type: AnchorType,

    #[validate(length(min = 1))]
    pub peer_uuid: String,

    #[validate(length(min = 1))]
    pub endpoint_uuid: String,
}

#[derive(Clone, PartialEq, Debug, Validate, Serialize, Deserialize, ToSchema)]
#[salvo(schema(name = "AddressAnchorSpec"))]
pub struct AddressAnchorSpec {
    #[serde(rename = "type")]
    pub anchor_type: AnchorType,

    pub source_uri: String,
    pub repair_uri: String,
    pub control_uri: String,
}
