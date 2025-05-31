// Copyright (c) Roc Streaming authors
// Licensed under MPL-2.0
use crate::dto::DriverId;

#[derive(thiserror::Error, Debug)]
pub enum DriverError {
    #[error("no supported drivers found")]
    NoDriversError,

    #[error("driver not supported: {0}")]
    UnsupportedError(DriverId),

    #[error("can't open driver: {0}")]
    OpenError(String),

    #[error("lost connection to driver")]
    ConnectionError,
}
