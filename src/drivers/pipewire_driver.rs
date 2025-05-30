// Copyright (c) Roc Streaming authors
// Licensed under MPL-2.0
use crate::drivers::driver::*;
use crate::dto::DriverId;

use std::sync::Arc;

pub struct PipewireDriver {
    // TODO
}

impl PipewireDriver {
    pub fn open() -> DriverResult<Arc<dyn Driver>> {
        tracing::debug!("opening pipewire driver");

        Ok(Arc::new(PipewireDriver {}))
    }
}

impl Driver for PipewireDriver {
    fn id(&self) -> DriverId {
        DriverId::Pipewire
    }
}
