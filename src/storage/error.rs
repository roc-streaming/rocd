// Copyright (c) Roc Streaming authors
// Licensed under MPL-2.0
use thiserror::Error;

#[derive(Error, Debug)]
pub enum StorageError {
    #[error("can't open db: {0}")]
    DatabaseError(#[from] redb::DatabaseError),

    #[error("can't open transaction: {0}")]
    TransactionError(#[from] redb::TransactionError),

    #[error("can't open table: {0}")]
    TableError(#[from] redb::TableError),

    #[error("can't read value: {0}")]
    ReadError(#[source] redb::StorageError),

    #[error("can't write value: {0}")]
    WriteError(#[source] redb::StorageError),

    #[error("can't commit transaction: {0}")]
    CommitError(#[from] redb::CommitError),

    #[error("can't decode value: {0}")]
    DecodeError(#[from] rmp_serde::decode::Error),

    #[error("can't encode value: {0}")]
    EncodeError(#[from] rmp_serde::encode::Error),

    #[error("invalid argument: {0}")]
    InvalidArgument(&'static str),

    #[error("invalid argument: {0}")]
    InvalidEntry(#[from] validator::ValidationErrors),

    #[error("uid not found: {0}")]
    UidNotFound(String),
}
