// Copyright (c) Roc Streaming authors
// Licensed under MPL-2.0
use crate::drivers::error::*;
use crate::dto::DriverId;

pub type DriverResult<T> = std::result::Result<T, DriverError>;

pub trait Driver: Send + Sync {
    fn id(&self) -> DriverId;
}
