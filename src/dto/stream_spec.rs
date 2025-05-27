// Copyright (c) Roc Streaming authors
// Licensed under MPL-2.0
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(Clone, PartialEq, Debug, Serialize, Deserialize, ToSchema)]
#[schema(title = "StreamSpec")]
pub struct StreamSpec {
    pub stream_uid: String,

    pub sources: Vec<AnchorSpec>,
    pub destinations: Vec<AnchorSpec>,
}

#[derive(Clone, PartialEq, Debug, Serialize, Deserialize, ToSchema)]
#[schema(title = "AnchorSpec")]
#[serde(rename_all = "snake_case")]
pub enum AnchorSpec {
    Endpoint(EndpointAnchorSpec),
    Address(AddressAnchorSpec),
}

#[derive(Clone, PartialEq, Debug, Serialize, Deserialize, ToSchema)]
#[schema(title = "AnchorType")]
#[serde(rename_all = "snake_case")]
pub enum AnchorType {
    Endpoint,
    Address,
}

#[derive(Clone, PartialEq, Debug, Serialize, Deserialize, ToSchema)]
#[schema(title = "EndpointAnchorSpec")]
pub struct EndpointAnchorSpec {
    #[serde(rename = "type")]
    pub anchor_type: AnchorType,

    pub peer_uid: String,
    pub endpoint_uid: String,
}

#[derive(Clone, PartialEq, Debug, Serialize, Deserialize, ToSchema)]
#[schema(title = "AddressAnchorSpec")]
pub struct AddressAnchorSpec {
    #[serde(rename = "type")]
    pub anchor_type: AnchorType,

    pub source_uri: String,
    pub repair_uri: String,
    pub control_uri: String,
}
