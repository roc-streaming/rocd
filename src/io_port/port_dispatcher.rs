// Copyright (c) Roc Streaming authors
// Licensed under MPL-2.0
use crate::dto::*;

pub struct PortDispatcher {
    // TODO
}

impl PortDispatcher {
    pub fn new() -> Self {
        PortDispatcher {}
    }

    pub async fn get_all(&self) -> Vec<PortDescriptor> {
        vec![self.get_port("11-22-33").await]
    }

    pub async fn get_port(&self, uid: &str) -> PortDescriptor {
        PortDescriptor {
            port_type: PortType::SystemDevice,
            port_direction: PortDirection::Output,
            port_driver: PortDriver::Pipewire,
            uid: uid.into(),
            display_name: "Display Name".into(),
            system_name: "system_name".into(),
        }
    }
}
