// Copyright (c) Roc Streaming authors
// Licensed under MPL-2.0
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(
    Copy,
    Clone,
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
    Hash,
    Debug,
    strum::Display,
    strum::EnumIter,
    clap::ValueEnum,
    Serialize,
    Deserialize,
    ToSchema,
)]
#[serde(rename_all = "snake_case")]
#[strum(serialize_all = "snake_case")]
pub enum DriverId {
    #[value(skip)]
    Unspecified,

    // Drivers are probed in the order defined here. When multiple drivers are supported,
    // and the user didn't select a driver explicitly, the first working one is used.
    Pipewire,
}
