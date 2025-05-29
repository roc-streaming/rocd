// Copyright (c) Roc Streaming authors
// Licensed under MPL-2.0
use std::str::Utf8Error;

#[derive(thiserror::Error, Debug)]
pub enum ValidationError {
    #[error("illformed uid '{0}', expected 'xxxxxx-xxxxxx-xxxxxx' where 'x' is [a-z0-9]")]
    UidError(String),

    #[error("illformed utf-8 string: {0}")]
    UtfError(#[from] Utf8Error),
}
