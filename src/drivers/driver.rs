// Copyright (c) Roc Streaming authors
// Licensed under MPL-2.0
use crate::drivers::error::*;
use crate::dto::DriverId;

use async_trait::async_trait;
use std::sync::Arc;

pub type DriverResult<T> = std::result::Result<T, DriverError>;

#[async_trait]
pub trait Driver: Send + Sync {
    /// Open driver.
    async fn open() -> DriverResult<Arc<dyn Driver>>
    where
        Self: Sized;

    /// Close driver.
    async fn close(self: Arc<Self>);

    /// Get driver ID.
    fn id(&self) -> DriverId;
}
