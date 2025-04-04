// Copyright (c) Roc Streaming authors
// Licensed under MPL-2.0
use crate::models::*;

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
    vec![get_device("uid").await]
}

pub async fn get_device(uid: &str) -> Device {
    Device {
        uid: uid.into(),
        system_name: "sname".into(),
        display_name: "dname".into(),
        dev_type: DeviceType::Sink,
        driver: DeviceDriver::Pipewire,
        is_hardware: true,
        is_stream: true,
        status: DeviceStatus::Disabled,
        is_muted: false,
    }
}
