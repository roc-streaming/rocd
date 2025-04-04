// Copyright (c) Roc Streaming authors
// Licensed under MPL-2.0
use crate::models::Device;
use quick_cache::sync::Cache;
use std::sync::Arc;

pub(super) struct MemCache {
    devices: Cache<String, Arc<Device>>,
}

impl MemCache {
    pub(super) fn new(size: usize) -> Self {
        MemCache { devices: Cache::new(size) }
    }

    pub(super) fn read_device(&self, uid: &str) -> Option<Arc<Device>> {
        self.devices.get(uid)
    }

    pub(super) fn write_device(&self, device: Device) {
        assert!(!device.uid.is_empty());
        self.devices.insert(device.uid.clone(), Arc::new(device))
    }
}
