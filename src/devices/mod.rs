use salvo::oapi::ToSchema;
use serde::{Deserialize, Serialize};

#[derive(ToSchema, Serialize, Deserialize)]
pub struct Device {
    // immutable fields
    //
    uid: String,
    system_name: String,
    display_name: String,

    #[serde(rename = "type")]
    type_: DeviceType,
    driver: DeviceDriver,
    is_hardware: bool,
    is_stream: bool,

    // mutable fields (updated via HTTP)
    //
    status: DeviceStatus,
    is_muted: bool,
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

// enum to String casting; TODO: uncomment if needed or remove
//impl From<DeviceType> for String {
//    fn from(tp: DeviceType) -> String {
//        match tp {
//            DeviceType::Sink => "sink".to_owned(),
//            DeviceType::Source => "source".to_owned(),
//        }
//    }
//}

pub async fn get_all() -> Vec<Device> {
    return vec![Device {
        uid: "uid".into(),
        system_name: "sname".into(),
        display_name: "dname".into(),
        type_: DeviceType::Sink,
        driver: DeviceDriver::Pulseaudio,
        is_hardware: true,
        is_stream: true,
        status: DeviceStatus::Disabled,
        is_muted: false,
    }];
}
