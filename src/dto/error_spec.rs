// Copyright (c) Roc Streaming authors
// Licensed under MPL-2.0
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(Clone, PartialEq, Debug, Serialize, Deserialize, ToSchema)]
pub struct ErrorSpec {
    pub error_code: ErrorCode,
    pub error_text: String,
}

#[derive(Clone, PartialEq, Debug, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum ErrorCode {
    ValidationFailed,
}
