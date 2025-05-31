// Copyright (c) Roc Streaming authors
// Licensed under MPL-2.0
use crate::drivers::error::*;
use crate::dto::DriverId;

use async_trait::async_trait;
use std::sync::Arc;

pub type DriverResult<T> = std::result::Result<T, DriverError>;

#[async_trait]
pub trait Driver: Send + Sync {
    async fn open() -> DriverResult<Arc<dyn Driver>>
    where
        Self: Sized;

    async fn close(&self);

    fn id(&self) -> DriverId;
}
