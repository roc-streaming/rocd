// Copyright (c) Roc Streaming authors
// Licensed under MPL-2.0
use rocd::drivers::*;
use rocd::dto::DriverId;

use async_trait::async_trait;
use std::sync::Arc;

pub struct MockDriver {}

#[async_trait]
impl Driver for MockDriver {
    async fn open() -> DriverResult<Arc<dyn Driver>> {
        tracing::debug!("opening mock driver");

        Ok(Arc::new(MockDriver {}))
    }

    async fn close(&self) {
        tracing::debug!("closing mock driver");
    }

    fn id(&self) -> DriverId {
        DriverId::Unspecified
    }
}
