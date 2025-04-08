// Copyright (c) Roc Streaming authors
// Licensed under MPL-2.0
use salvo::oapi::ToSchema;
use serde::{Deserialize, Serialize};
use validator::Validate;

#[derive(Clone, PartialEq, Debug, Validate, ToSchema, Serialize, Deserialize)]
pub struct Device {
    // immutable fields
    //
    #[validate(length(min = 1))]
    pub uid: String,
    #[validate(length(min = 1))]
    pub system_name: String,
    #[validate(length(min = 1))]
    pub display_name: String,

    #[serde(rename = "type")]
    pub dev_type: DeviceType,
    pub driver: DeviceDriver,
    pub is_hardware: bool,
    pub is_stream: bool,

    // mutable fields (updated via HTTP)
    //
    pub status: DeviceStatus,
    pub is_muted: bool,
    // TODO

    // to_address: AddressList,
    // from_address: AddressList,

    // internal fields
    //
    // node_id: String,
    // module_id: String,
}

#[derive(Clone, PartialEq, Debug, ToSchema, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DeviceType {
    Sink,
    Source,
}

#[derive(Clone, PartialEq, Debug, ToSchema, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DeviceStatus {
    Disabled,
    Enabled,
    Unavailable,
}

#[derive(Clone, PartialEq, Debug, ToSchema, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DeviceDriver {
    Pipewire,
}
