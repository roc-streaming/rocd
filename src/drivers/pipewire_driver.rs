// Copyright (c) Roc Streaming authors
// Licensed under MPL-2.0
use crate::drivers::driver::*;
use crate::dto::DriverId;

//use pipewire::main_loop::MainLoop;
use async_trait::async_trait;
use std::sync::Arc;
//use std::thread;

pub struct PipewireDriver {
    // TODO
}

impl PipewireDriver {}

#[async_trait]
impl Driver for PipewireDriver {
    async fn open() -> DriverResult<Arc<dyn Driver>> {
        tracing::debug!("opening pipewire driver");

        Ok(Arc::new(PipewireDriver {}))
    }

    async fn close(&self) {
        tracing::debug!("closing pipewire driver");

        // TODO
    }

    fn id(&self) -> DriverId {
        DriverId::Pipewire
    }
}
