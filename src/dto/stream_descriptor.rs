// Copyright (c) Roc Streaming authors
// Licensed under MPL-2.0
use salvo::oapi::ToSchema;
use serde::{Deserialize, Serialize};
use validator::Validate;

#[derive(Clone, PartialEq, Debug, Validate, ToSchema, Serialize, Deserialize)]
pub struct StreamDescriptor {
    #[validate(length(min = 1))]
    pub uid: String,

    #[validate(length(min = 1))]
    pub sources: Vec<StreamEndpointDescriptor>,

    #[validate(length(min = 1))]
    pub destinations: Vec<StreamEndpointDescriptor>,
}

#[derive(Clone, PartialEq, Debug, ToSchema, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum StreamEndpointType {
    Port,
    Addr,
}

#[derive(Clone, PartialEq, Debug, ToSchema, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum StreamEndpointDescriptor {
    Port(PortEndpointDescriptor),
    Addr(AddrEndpointDescriptor),
}

#[derive(Clone, PartialEq, Debug, ToSchema, Serialize, Deserialize)]
pub struct PortEndpointDescriptor {
    #[serde(rename = "type")]
    pub endpoint_type: StreamEndpointType,

    pub peer_uid: String,
    pub port_uid: String,
}

#[derive(Clone, PartialEq, Debug, ToSchema, Serialize, Deserialize)]
pub struct AddrEndpointDescriptor {
    #[serde(rename = "type")]
    pub endpoint_type: StreamEndpointType,

    pub audio_source: String,
    pub audio_repair: String,
    pub audio_control: String,
}
