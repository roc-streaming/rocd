// Copyright (c) Roc Streaming authors
// Licensed under MPL-2.0
mod cache;

use crate::models::Device;
use crate::storage::cache::MemCache;
use derive_builder::Builder;
use std::result;
use std::sync::Arc;
use thiserror::Error;
use validator::{Validate, ValidationErrors};

#[derive(Default, Builder, Debug)]
#[builder(setter(into))]
pub struct StorageConfig {
    #[builder(default = 1000)]
    pub cache_size: usize,
}

#[derive(Error, Debug)]
pub enum StorageError {
    #[error("bad device: {0}")]
    BadDevice(ValidationErrors),
}

pub type Result<T> = result::Result<T, StorageError>;

pub struct Storage {
    mem_cache: MemCache,
}

impl Storage {
    pub fn new(config: StorageConfig) -> Self {
        Storage { mem_cache: MemCache::new(config.cache_size) }
    }

    pub fn read_device(&self, uid: &str) -> Option<Arc<Device>> {
        self.mem_cache.read_device(uid)
    }

    pub fn write_device(&self, device: Device) -> Result<()> {
        if let Err(err) = device.validate() {
            return Err(StorageError::BadDevice(err));
        }

        self.mem_cache.write_device(device);

        Ok(())
    }
}
