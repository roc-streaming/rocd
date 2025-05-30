// Copyright (c) Roc Streaming authors
// Licensed under MPL-2.0
use crate::dto::*;

use axum::extract::Json;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use std::io;
use tokio::task::JoinError;

/// Errors returned from server methods.
#[derive(thiserror::Error, Debug)]
pub enum ServerError {
    #[error("bad state")]
    StateError,

    #[error("bind failed: {0}")]
    BindError(#[source] io::Error),

    #[error("serve failed: {0}")]
    ServeError(#[source] io::Error),

    #[error("task failed: {0}")]
    TokioError(#[source] JoinError),
}

/// Errors returned from API handlers.
/// Axum will automatically translate it to HTTP response using IntoResponse.
#[derive(thiserror::Error, Debug)]
pub enum HandlerError {
    #[error("{0}")]
    ValidationError(#[from] ValidationError),
}

impl IntoResponse for HandlerError {
    fn into_response(self) -> Response {
        let error_text = self.to_string();

        let (status_code, error_code) = match &self {
            Self::ValidationError(_) => (StatusCode::BAD_REQUEST, ErrorCode::InvalidArgs),
        };

        let response_body = ErrorSpec { error_code, error_text };

        (status_code, Json(response_body)).into_response()
    }
}
