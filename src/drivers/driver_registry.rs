// Copyright (c) Roc Streaming authors
// Licensed under MPL-2.0
use crate::drivers::driver::*;
use crate::drivers::error::*;
use crate::dto::DriverId;

#[cfg(feature = "pipewire")]
use crate::drivers::pipewire_driver::PipewireDriver;

use std::collections::HashMap;
use std::pin::Pin;
use std::sync::Arc;
use strum::IntoEnumIterator;

/// Future returned by async methods of Driver trait.
/// This kind of type is generated by #[async_trait].
type DriverFuture<Output> = Pin<Box<dyn Future<Output = Output> + Send>>;

/// Pointer to Driver::open function.
type DriverOpenFn = fn() -> DriverFuture<DriverResult<Arc<dyn Driver>>>;

/// Registry of all supported drivers.
/// Whether a driver is supported is defined at compile-time.
pub struct DriverRegistry {
    driver_map: HashMap<DriverId, DriverOpenFn>,
}

impl DriverRegistry {
    /// Construct driver registry.
    pub fn new() -> Self {
        #[allow(unused_mut)]
        let mut driver_map: HashMap<DriverId, DriverOpenFn> = HashMap::new();

        #[cfg(feature = "pipewire")]
        driver_map.insert(DriverId::Pipewire, PipewireDriver::open);

        DriverRegistry { driver_map }
    }

    /// Detect first supported driver and open it.
    pub async fn open_driver(&self) -> DriverResult<Arc<dyn Driver>> {
        tracing::debug!("iterating suported drivers: {:?}", self.driver_map.keys());

        let mut result = Err(DriverError::NoDriversError);

        // iterate drivers in order as they are defined in DriverId enum
        for driver_id in DriverId::iter() {
            if let Some(driver_fn) = self.driver_map.get(&driver_id) {
                result = driver_fn().await;

                if let Ok(driver) = result {
                    tracing::debug!("successfully opened driver: {:?}", driver.id());
                    return Ok(driver);
                }
            }
        }

        result
    }

    /// Open specific driver.
    pub async fn open_driver_by_id(
        &self, driver_id: DriverId,
    ) -> DriverResult<Arc<dyn Driver>> {
        tracing::debug!("trying to open driver: {driver_id:?}");

        let driver_fn =
            self.driver_map.get(&driver_id).ok_or(DriverError::UnsupportedError(driver_id))?;

        let result = driver_fn().await;

        if let Ok(driver) = result {
            tracing::debug!("successfully opened driver: {:?}", driver.id());
            return Ok(driver);
        }

        result
    }
}
