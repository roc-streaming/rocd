// Copyright (c) Roc Streaming authors
// Licensed under MPL-2.0
use rocd::drivers::*;
use rocd::dto::DriverId;

use test_context::{AsyncTestContext, test_context};
use tracing_test::traced_test;

struct DriverTestContext {
    driver_registry: DriverRegistry,
}

impl AsyncTestContext for DriverTestContext {
    async fn setup() -> DriverTestContext {
        DriverTestContext { driver_registry: DriverRegistry::new() }
    }
}

impl DriverTestContext {
    async fn each_driver<F: AsyncFn(DriverId)>(&self, func: F) {
        for driver_id in self.driver_registry.supported_drivers().iter() {
            func(*driver_id).await
        }
    }
}

#[cfg_attr(not(feature = "driver-tests"), ignore = "driver-tests")]
#[test_context(DriverTestContext)]
#[tokio::test]
#[traced_test]
async fn test_supported_drivers(ctx: &mut DriverTestContext) {
    let supported_drivers = ctx.driver_registry.supported_drivers();

    assert!(!supported_drivers.is_empty());

    for driver_id in supported_drivers {
        assert_ne!(DriverId::Unspecified, driver_id);
    }
}

#[cfg_attr(not(feature = "driver-tests"), ignore = "driver-tests")]
#[test_context(DriverTestContext)]
#[tokio::test]
#[traced_test]
async fn test_open_default_driver(ctx: &mut DriverTestContext) {
    let driver = ctx.driver_registry.open_default_driver().await.unwrap();

    assert_ne!(DriverId::Unspecified, driver.id())
}

#[cfg_attr(not(feature = "driver-tests"), ignore = "driver-tests")]
#[test_context(DriverTestContext)]
#[tokio::test]
#[traced_test]
async fn test_open_driver(ctx: &mut DriverTestContext) {
    ctx.each_driver(async |driver_id| {
        let driver = ctx
            .driver_registry
            .open_driver(driver_id)
            .await
            .expect(format!("can't open {driver_id} driver").as_str());

        assert_eq!(driver_id, driver.id())
    })
    .await
}
