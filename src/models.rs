use salvo::oapi::ToSchema;
use serde::{Deserialize, Serialize};

#[derive(ToSchema, Serialize, Deserialize)]
pub struct Device {
    // immutable fields
    //
    pub uid: String,
    pub system_name: String,
    pub display_name: String,

    #[serde(rename = "type")]
    pub type_: DeviceType,
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

#[derive(ToSchema, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DeviceType {
    Sink,
    Source,
}

#[derive(ToSchema, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DeviceStatus {
    Disabled,
    Enabled,
    Unavailable,
}

#[derive(ToSchema, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DeviceDriver {
    Pipewire,
    Pulseaudio,
}
