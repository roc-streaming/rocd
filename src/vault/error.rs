// Copyright (c) Roc Streaming authors
// Licensed under MPL-2.0
use crate::dto::{Uid, ValidationError};

#[derive(thiserror::Error, Debug)]
pub enum VaultError {
    #[error("can't open db: {0}")]
    DatabaseError(#[from] Box<redb::DatabaseError>),

    #[error("can't open transaction: {0}")]
    TransactionError(#[from] Box<redb::TransactionError>),

    #[error("can't open table: {0}")]
    TableError(#[from] Box<redb::TableError>),

    #[error("can't read value: {0}")]
    ReadError(#[source] Box<redb::StorageError>),

    #[error("can't write value: {0}")]
    WriteError(#[source] Box<redb::StorageError>),

    #[error("can't commit transaction: {0}")]
    CommitError(#[from] Box<redb::CommitError>),

    #[error("can't decode value: {0}")]
    DecodeError(#[from] rmp_serde::decode::Error),

    #[error("can't encode value: {0}")]
    EncodeError(#[from] rmp_serde::encode::Error),

    #[error("invalid value: {0}")]
    ValidationError(#[from] ValidationError),

    #[error("invalid argument: {0}")]
    InvalidArgument(&'static str),

    #[error("uid not found: {0}")]
    UidNotFound(Box<Uid>),
}

impl From<redb::DatabaseError> for VaultError {
    fn from(err: redb::DatabaseError) -> Self {
        VaultError::from(Box::new(err))
    }
}

impl From<redb::TransactionError> for VaultError {
    fn from(err: redb::TransactionError) -> Self {
        VaultError::from(Box::new(err))
    }
}

impl From<redb::TableError> for VaultError {
    fn from(err: redb::TableError) -> Self {
        VaultError::from(Box::new(err))
    }
}

impl From<redb::CommitError> for VaultError {
    fn from(err: redb::CommitError) -> Self {
        VaultError::from(Box::new(err))
    }
}
