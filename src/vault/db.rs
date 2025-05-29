// Copyright (c) Roc Streaming authors
// Licensed under MPL-2.0
use crate::dto::Uid;
use crate::vault::error::VaultError;

use redb::{Database, TableDefinition, TableHandle};
use serde::Serialize;
use serde::de::DeserializeOwned;
use std::collections::HashSet;
use std::fmt::Debug;
use std::result;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Mutex};
use tokio::task;

pub type Result<T> = result::Result<T, VaultError>;

pub type Table = TableDefinition<
    // Liftetime of table name. Out TableDefinitions are static
    // constants and their names are static literals.
    'static,
    // Table key (UID).
    // This lifetime doesn't affect lifetimes of actual keys in table.
    // It has to be static because our TableDefinitions are static constants.
    &'static str,
    // Table value (serialized struct).
    // This lifetime doesn't affect lifetimes of actual values in table.
    // It has to be static because our TableDefinitions are static constants.
    &'static [u8],
>;

#[derive(Debug)]
pub struct Db {
    handle: Database,

    // Reusable buffer for messagepack serialization.
    // Since write operations are serialized in time by the upper layer (Storage),
    // it's fine to use the same buffer. Mutex is used just to make compiller happy.
    buffer: Mutex<Vec<u8>>,

    // Metrics.
    read_ops: AtomicU64,
    write_ops: AtomicU64,
}

#[derive(PartialEq, Debug)]
pub struct DbMetrics {
    /// Accumulative counter of read transactions.
    pub read_ops: u64,
    /// Accumulative counter of write transactions.
    pub write_ops: u64,
}

/// Thread-safe key-value vault.
///
/// Uses `redb` for storage and `rmp_serde` (messagepack) for serialization.
/// Implements async interface on top of `redb`, invokes blocking I/O operations
/// using tokio::task::spawn_blocking().
///
/// Thread-safe, allows N concurrent reads and up to 1 concurrent write at the same time.
/// Crash-safe, 1 write is mapped to 1 transaction.
impl Db {
    /// Open DB instance.
    pub async fn open(db_path: &str) -> Result<Arc<Self>> {
        assert!(!db_path.is_empty());

        // run blocking open on background thread
        let task_result = task::spawn_blocking({
            let db_path = db_path.to_string();

            move || -> Result<Database> { Ok(Database::create(db_path)?) }
        })
        .await
        .unwrap(); // panic if tokio failed to run task

        Ok(Arc::new(Db {
            handle: task_result?,
            buffer: Mutex::new(Vec::with_capacity(4096)),
            read_ops: AtomicU64::new(0),
            write_ops: AtomicU64::new(0),
        }))
    }

    /// Get metrics.
    pub fn metrics(&self) -> DbMetrics {
        DbMetrics {
            read_ops: self.read_ops.load(Ordering::SeqCst),
            write_ops: self.write_ops.load(Ordering::SeqCst),
        }
    }

    /// Obtain list of table keys (UIDs).
    pub async fn list_entries(
        self: &Arc<Self>, table_definition: Table,
    ) -> Result<Arc<HashSet<Uid>>> {
        // run blocking read on background thread
        let task_result = task::spawn_blocking({
            let self_clone = Arc::clone(self);

            move || -> Result<Arc<HashSet<Uid>>> {
                let transaction = self_clone.handle.begin_read()?;

                let table = match transaction.open_table(table_definition) {
                    Ok(table) => table,
                    Err(redb::TableError::TableDoesNotExist(_)) => {
                        return Ok(Arc::new(HashSet::new()));
                    },
                    Err(err) => return Err(VaultError::from(err)),
                };

                let iter =
                    table.range::<&str>(..).map_err(|err| VaultError::ReadError(err))?;

                let uids = iter
                    .map(|elem| -> Result<Uid> {
                        let (key_guard, _value_guard) =
                            elem.map_err(|err| VaultError::ReadError(err))?;
                        let uid = Uid::parse(key_guard.value())
                            .map_err(|err| VaultError::ValidationError(err))?;
                        Ok(uid)
                    })
                    .collect::<Result<HashSet<Uid>>>()?;

                tracing::debug!(
                    "table '{}': list: found {} uid(s)",
                    TableHandle::name(&table_definition),
                    uids.len()
                );
                self_clone.read_ops.fetch_add(1, Ordering::SeqCst);

                Ok(Arc::new(uids))
            }
        })
        .await
        .unwrap(); // panic if tokio failed to run task

        // get uids from db or propagate error
        let uids: Arc<HashSet<Uid>> = task_result?;

        Ok(uids)
    }

    /// Read entry of type T from given DB table.
    /// DB holds raw bytes, entry is deserialized using messagepack.
    pub async fn read_entry<T>(
        self: &Arc<Self>, table_definition: Table, uid: &Uid,
    ) -> Result<Arc<T>>
    where
        T: DeserializeOwned + Sync + Send + Debug + 'static,
    {
        // run blocking read on background thread
        let task_result = task::spawn_blocking({
            let self_clone = Arc::clone(self);
            let uid = *uid;

            move || -> Result<Arc<T>> {
                let transaction = self_clone.handle.begin_read()?;

                let table = match transaction.open_table(table_definition) {
                    Ok(table) => table,
                    Err(redb::TableError::TableDoesNotExist(_)) => {
                        return Err(VaultError::UidNotFound(uid));
                    },
                    Err(err) => return Err(VaultError::from(err)),
                };

                // read bytes from db
                let db_value = match table.get(uid.as_str()) {
                    Ok(Some(value)) => value,
                    Ok(None) => return Err(VaultError::UidNotFound(uid)),
                    Err(err) => return Err(VaultError::ReadError(err)),
                };

                // deserialize from bytes with messagepack
                let value = rmp_serde::from_slice(db_value.value())?;

                tracing::debug!(
                    "table '{}': read: {:?}",
                    TableHandle::name(&table_definition),
                    &value
                );
                self_clone.read_ops.fetch_add(1, Ordering::SeqCst);

                Ok(Arc::new(value))
            }
        })
        .await
        .unwrap(); // panic if tokio failed to run task

        // get value from db or propagate error
        let value: Arc<T> = task_result?;

        Ok(value)
    }

    /// Write entry of type T to given DB table.
    /// DB holds raw bytes, entry is serialized using messagepack.
    pub async fn write_entry<T>(
        self: &Arc<Self>, table_definition: Table, uid: &Uid, value: &Arc<T>,
    ) -> Result<()>
    where
        T: Serialize + Sync + Send + Debug + 'static,
    {
        // run blocking read on background thwrite
        let task_result = task::spawn_blocking({
            let self_clone = Arc::clone(self);
            let value = Arc::clone(value);
            let uid = *uid;

            move || -> Result<()> {
                let transaction = self_clone.handle.begin_write()?;

                {
                    let mut table = transaction.open_table(table_definition)?;

                    // serialize to bytes with messagepack
                    let mut buffer = self_clone.buffer.lock().unwrap();
                    buffer.clear();
                    let mut buffer_writer =
                        rmp_serde::Serializer::<&mut Vec<u8>>::new(buffer.as_mut());
                    value.serialize(&mut buffer_writer)?;

                    // write bytes to db
                    table
                        .insert(uid.as_str(), &buffer[..])
                        .map_err(|err| VaultError::WriteError(err))?;
                }

                transaction.commit()?;

                tracing::debug!(
                    "table '{}': write: {:?}",
                    TableHandle::name(&table_definition),
                    &value
                );
                self_clone.write_ops.fetch_add(1, Ordering::SeqCst);

                Ok(())
            }
        })
        .await
        .unwrap(); // panic if tokio failed to run task

        task_result
    }

    /// Remove entry from given DB table.
    pub async fn remove_entry(
        self: &Arc<Self>, table_definition: Table, uid: &Uid,
    ) -> Result<()> {
        // run blocking read on background thwrite
        let task_result = task::spawn_blocking({
            let self_clone = Arc::clone(self);
            let uid = *uid;

            move || -> Result<()> {
                let transaction = self_clone.handle.begin_write()?;

                {
                    let mut table = transaction.open_table(table_definition)?;

                    // delete value from db
                    match table.remove(uid.as_str()) {
                        Ok(Some(_)) => (),
                        Ok(None) => return Err(VaultError::UidNotFound(uid)),
                        Err(err) => return Err(VaultError::WriteError(err)),
                    };
                }

                transaction.commit()?;

                tracing::debug!(
                    "table '{}': remove: {:?}",
                    TableHandle::name(&table_definition),
                    uid
                );
                self_clone.write_ops.fetch_add(1, Ordering::SeqCst);

                Ok(())
            }
        })
        .await
        .unwrap(); // panic if tokio failed to run task

        task_result
    }
}
