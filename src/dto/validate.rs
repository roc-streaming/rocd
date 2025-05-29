// Copyright (c) Roc Streaming authors
// Licensed under MPL-2.0
use std::str::Utf8Error;

#[derive(thiserror::Error, Debug)]
pub enum ValidationError {
    #[error("invalid layout: {0}")]
    LayoutError(&'static str),

    #[error("invalid uid '{0}', expected 'xxxxxx-xxxxxx-xxxxxx' where 'x' is [a-z0-9]")]
    UidError(String),

    #[error("invalid utf-8 string: {0}")]
    UtfError(#[from] Utf8Error),
}

pub type ValidationResult = Result<(), ValidationError>;

pub trait Validate {
    fn validate(&self) -> ValidationResult;
}
