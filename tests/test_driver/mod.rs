// Copyright (c) Roc Streaming authors
// Licensed under MPL-2.0
use rocd::drivers::*;
use rocd::dto::DriverId;

use std::sync::Arc;

pub struct MockDriver {
    // TODO
}

impl MockDriver {
    pub fn open() -> Arc<dyn Driver> {
        Arc::new(MockDriver {})
    }
}

impl Driver for MockDriver {
    fn id(&self) -> DriverId {
        DriverId::Unspecified
    }
}
