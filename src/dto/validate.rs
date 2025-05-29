// Copyright (c) Roc Streaming authors
// Licensed under MPL-2.0
use crate::dto::uri::UriKind;

use std::str::Utf8Error;

#[derive(thiserror::Error, Debug)]
pub enum ValidationError {
    #[error("when {key} is '{value}', allowed fields are: {allow_fields}")]
    EnumTypeError { key: &'static str, value: &'static str, allow_fields: &'static str },

    #[error("{key} must be {expected} URI, not {actual} URI")]
    UriTypeError { key: &'static str, expected: UriKind, actual: UriKind },

    #[error("invalid UID '{0}'")]
    UidFormatError(String),

    #[error("invalid URI '{0}'")]
    UriFormatError(String),

    #[error("invalid URL '{0}': {1}")]
    UrlFormatError(String, #[source] url::ParseError),

    #[error("illformed UTF-8 string: {0}")]
    Utf8Error(#[from] Utf8Error),
}

pub type ValidationResult = Result<(), ValidationError>;

pub trait Validate {
    fn validate(&self) -> ValidationResult;
}
