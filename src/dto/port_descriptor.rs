// Copyright (c) Roc Streaming authors
// Licensed under MPL-2.0
use salvo::oapi::ToSchema;
use serde::{Deserialize, Serialize};
use validator::Validate;

#[derive(Clone, PartialEq, Debug, Validate, ToSchema, Serialize, Deserialize)]
pub struct PortDescriptor {
    pub port_type: PortType,
    pub port_direction: PortDirection,
    pub port_driver: PortDriver,

    #[validate(length(min = 1))]
    pub uid: String,

    #[validate(length(min = 1))]
    pub display_name: String,

    #[validate(length(min = 1))]
    pub system_name: String,
}

#[derive(Clone, PartialEq, Debug, ToSchema, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PortType {
    SystemDevice,
    StreamingDevice,
}

#[derive(Clone, PartialEq, Debug, ToSchema, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PortDirection {
    Input,
    Output,
    Duplex,
}

#[derive(Clone, PartialEq, Debug, ToSchema, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PortDriver {
    Pipewire,
}
