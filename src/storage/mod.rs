// Copyright (c) Roc Streaming authors
// Licensed under MPL-2.0
mod db;
mod error;
#[cfg(test)]
mod tests;

use crate::models::Device;
use crate::storage::db::{Db, Table};
pub use crate::storage::error::StorageError;

use derive_builder::Builder;
use quick_cache::sync::Cache;
use serde::Serialize;
use serde::de::DeserializeOwned;
use std::collections::HashSet;
use std::fmt::Debug;
use std::result;
use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};
use tokio::sync::{Mutex, RwLock};
use validator::Validate;

pub type Result<T> = result::Result<T, StorageError>;

/// Persistent storage config.
#[derive(Builder, Default, Validate, Debug)]
#[builder(setter(into))]
pub struct StorageConfig {
    /// DB file path.
    /// Directory should exist, file is auto-created.
    #[validate(length(min = 1))]
    pub db_path: String,

    /// How much entries to cache in memory.
    #[builder(default = 1000)]
    #[validate(range(min = 1))]
    pub cache_size: usize,
}

/// Persistent storage metrics.
#[derive(PartialEq, Debug)]
pub struct StorageMetrics {
    /// Total count of entries cached in memory currently.
    pub cache_size: usize,

    /// Cummulative counters of cache hits.
    pub cache_hits: u64,
    /// Cummulative counters of cache misses.
    pub cache_misses: u64,
    /// Cummulative counters of drops caused by conflicts.
    pub cache_drops: u64,
    /// Cummulative counters of copy-on-writes.
    pub cache_cows: u64,

    /// Cummulative DB read transactions.
    pub db_reads: u64,
    /// Cummulative DB write transactions.
    pub db_writes: u64,
}

/// Persistent storage.
/// Thread-safe, async.
///
/// Combines persistent DB (`redb`) + in-memory LRU cache (`quick-cache`).
/// Allows N concurrent reads and up to 1 concurrent write.
///
/// Returns ARCs with immutable caches owned by storage. Storage will do
/// copy-on-write if it needs to update cache, but it's not the unique owner.
#[derive(Debug)]
pub struct Storage {
    db: Arc<Db>,
    write_lock: Mutex<()>,
    device_cache: RwLock<MemCache<Device>>,
    // metrics
    cache_hits: AtomicU64,
    cache_misses: AtomicU64,
    cache_drops: AtomicU64,
    cache_cows: AtomicU64,
}

#[derive(Debug)]
struct MemCache<T> {
    /// LRU cache with the most used subset of entries present in DB.
    /// quick-cache will automatically remove least used entries when
    /// cache size exceeds configured limit.
    kvmap: Cache<String, Arc<T>>,

    /// Lazy-initialized list of all keys present in DB.
    /// Empty until the first list_imp() call.
    /// After it becomes not-empty, write_imp() and remove_imp(), maintain it up-to-date.
    /// Lazy initialization allows faster startup time.
    kset: Option<Arc<HashSet<String>>>,
}

impl Storage {
    /// Create instance.
    pub async fn open(config: &StorageConfig) -> Result<Self> {
        config.validate()?;

        let db = Db::open(config.db_path.as_str()).await?;

        Ok(Storage {
            db,
            write_lock: Mutex::new(()),
            device_cache: RwLock::new(MemCache {
                kvmap: Cache::new(config.cache_size),
                kset: None,
            }),
            cache_hits: AtomicU64::new(0),
            cache_misses: AtomicU64::new(0),
            cache_drops: AtomicU64::new(0),
            cache_cows: AtomicU64::new(0),
        })
    }

    /// Get metrics.
    pub async fn metrics(&self) -> StorageMetrics {
        let db_metrics = self.db.metrics();

        let mut cache_size = 0;
        for cache in [&self.device_cache] {
            let rlocked_cache = cache.read().await;
            cache_size += rlocked_cache.kvmap.len();
        }

        StorageMetrics {
            cache_size,
            cache_hits: self.cache_hits.load(Ordering::SeqCst),
            cache_misses: self.cache_misses.load(Ordering::SeqCst),
            cache_drops: self.cache_drops.load(Ordering::SeqCst),
            cache_cows: self.cache_cows.load(Ordering::SeqCst),
            db_reads: db_metrics.read_ops,
            db_writes: db_metrics.write_ops,
        }
    }

    /// List all device UIDs.
    /// First call will read the list from DB, subsequent calls will
    /// return value from memory.
    pub async fn list_devices(&self) -> Result<Arc<HashSet<String>>> {
        self.list_imp(db::DEVICE_TABLE, &self.device_cache).await
    }

    /// Read device by UID.
    /// Returns value from in-memory cache if present, otherwise
    /// reads from DB and updates cache.
    pub async fn read_device(&self, uid: &str) -> Result<Arc<Device>> {
        if uid.is_empty() {
            return Err(StorageError::InvalidArgument("empty uid"));
        }
        self.read_imp(db::DEVICE_TABLE, &self.device_cache, uid).await
    }

    /// Write device.
    /// Updates both in-memory cache and DB.
    /// Blocks until DB transaction is completed.
    pub async fn write_device(&self, device: &Arc<Device>) -> Result<()> {
        if device.uid.is_empty() {
            return Err(StorageError::InvalidArgument("empty device.uid"));
        }
        device.validate()?;
        self.write_imp(db::DEVICE_TABLE, &self.device_cache, &device.uid, device).await
    }

    /// Remove device.
    /// Updates both in-memory cache and DB.
    /// Blocks until DB transaction is completed.
    pub async fn remove_device(&self, uid: &str) -> Result<()> {
        if uid.is_empty() {
            return Err(StorageError::InvalidArgument("empty uid"));
        }
        self.remove_imp(db::DEVICE_TABLE, &self.device_cache, uid).await
    }

    /// Generic list implementation.
    async fn list_imp<T>(
        &self, table: Table, cache: &RwLock<MemCache<T>>,
    ) -> Result<Arc<HashSet<String>>> {
        // Fast path: keyset already initialized.
        {
            let rlocked_cache = cache.read().await;
            if let Some(kset_ptr) = rlocked_cache.kset.as_ref() {
                self.cache_hits.fetch_add(1, Ordering::SeqCst);
                return Ok(Arc::clone(kset_ptr));
            }
        }

        // Slow path: read keyset from db.
        let kset: Arc<HashSet<String>> = self.db.list_entries(table).await?;

        {
            let mut wlocked_cache = cache.write().await;

            // Concurrent list_imp() already initialized keyset, we have nothing to do.
            if let Some(kset_ptr) = wlocked_cache.kset.as_ref() {
                self.cache_drops.fetch_add(1, Ordering::SeqCst);
                return Ok(Arc::clone(kset_ptr));
            }
            _ = wlocked_cache.kset.insert(Arc::clone(&kset));
        }

        self.cache_misses.fetch_add(1, Ordering::SeqCst);
        Ok(kset)
    }

    /// Generic read implementation for type T.
    async fn read_imp<T>(
        &self, table: Table, cache: &RwLock<MemCache<T>>, uid: &str,
    ) -> Result<Arc<T>>
    where
        T: DeserializeOwned + Sync + Send + Debug + 'static,
    {
        // Fast path: read value from memory cache.
        {
            let rlocked_cache = cache.read().await;
            if let Some(value) = rlocked_cache.kvmap.get(uid) {
                self.cache_hits.fetch_add(1, Ordering::SeqCst);
                return Ok(value);
            }
        }

        // Slow path: read value from db.
        let value: Arc<T> = self.db.read_entry(table, uid).await?;

        // Save value from db to memory cache.
        {
            let wlocked_cache = cache.write().await;
            if let Some(other_value) = wlocked_cache.kvmap.get(uid) {
                // There are two possibilities how we can enter here:
                //
                //  - Concurrent write_imp() updated cache while we were reading from db.
                //    In this case, the value from cache takes priority over the value we
                //    got from db, because write_imp() updates cache before updating db.
                //
                //  - Concurrent read_imp() updated cache while we were reading from db.
                //    In this case, the value we got from db is equal to value in cache,
                //    and there is no difference which one to use.
                //
                // Therefore, we can safely drop the value that we've read from db and
                // instead return the value we've found in cache. This will give us correct
                // result in both cases.
                self.cache_drops.fetch_add(1, Ordering::SeqCst);
                return Ok(other_value);
            }
            wlocked_cache.kvmap.insert(uid.to_string(), Arc::clone(&value));
        }

        self.cache_misses.fetch_add(1, Ordering::SeqCst);
        Ok(value)
    }

    /// Generic write implementation for type T.
    async fn write_imp<T>(
        &self, table: Table, cache: &RwLock<MemCache<T>>, uid: &str, value: &Arc<T>,
    ) -> Result<()>
    where
        T: Serialize + Sync + Send + Debug + 'static,
    {
        // Serialize writes to ensure that cache and db updates from concurrent writes
        // won't overlap and create a mess. Db backend anyway supports only one
        // concurrent write, so we don't make it worse.
        let _guard = self.write_lock.lock();

        // Write to memory cache.
        {
            let mut wlocked_cache = cache.write().await;
            wlocked_cache.kvmap.insert(uid.to_string(), Arc::clone(value));

            // If keyset is already lazy-initialized, update it.
            if let Some(kset_ptr) = wlocked_cache.kset.as_mut() {
                // Copy-on-write: make_mut() will clone hashset if someone outside holds
                // another pointer to it. Hence we can safely modify the keyset, while
                // ARCs returned outside will remain immutable.
                let kset: &mut HashSet<String> = match Arc::get_mut(kset_ptr) {
                    Some(kset) => kset,
                    None => {
                        self.cache_cows.fetch_add(1, Ordering::SeqCst);
                        Arc::make_mut(kset_ptr)
                    },
                };
                kset.insert(uid.to_string());
            }
        }

        // Write to db.
        self.db.write_entry(table, uid, value).await?;

        Ok(())
    }

    /// Generic delete implementation.
    async fn remove_imp<T>(
        &self, table: Table, cache: &RwLock<MemCache<T>>, uid: &str,
    ) -> Result<()>
    where
        T: Sync + Send + 'static,
    {
        // Serialize with writes.
        let _guard = self.write_lock.lock();

        // First remove from db.
        let result = self.db.remove_entry(table, uid).await;

        // Then remove from memory cache.
        // Should be done after removing from db, because otherwise concurrent
        // read_imp() could read it from db and re-add it to cache.
        // Note: remove from cache even if removing from DB failed.
        {
            let mut wlocked_cache = cache.write().await;
            wlocked_cache.kvmap.remove(uid);

            // If keyset is already lazy-initialized, update it.
            if let Some(kset_ptr) = wlocked_cache.kset.as_mut() {
                // Copy-on-write, see comment in write_imp().
                let kset: &mut HashSet<String> = match Arc::get_mut(kset_ptr) {
                    Some(kset) => kset,
                    None => {
                        self.cache_cows.fetch_add(1, Ordering::SeqCst);
                        Arc::make_mut(kset_ptr)
                    },
                };
                kset.remove(uid);
            }
        }

        result
    }
}
