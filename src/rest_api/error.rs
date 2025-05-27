// Copyright (c) Roc Streaming authors
// Licensed under MPL-2.0
use std::io;
use thiserror::Error;
use tokio::task::JoinError;

#[derive(Error, Debug)]
pub enum RestError {
    #[error("bad state")]
    StateError,

    #[error("bind failed: {0}")]
    BindError(#[source] io::Error),

    #[error("serve failed: {0}")]
    ServeError(#[source] io::Error),

    #[error("task failed: {0}")]
    TokioError(#[source] JoinError),
}
