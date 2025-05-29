// Copyright (c) Roc Streaming authors
// Licensed under MPL-2.0
mod db;
mod error;
#[cfg(test)]
mod tests;

use crate::dto::*;
use crate::vault::db::{Db, Table};
pub use crate::vault::error::VaultError;

use derive_builder::Builder;
use serde::Serialize;
use serde::de::DeserializeOwned;
use std::collections::HashSet;
use std::fmt::Debug;
use std::result;
use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};
use tokio::sync::{Mutex, RwLock};

pub type Result<T> = result::Result<T, VaultError>;

/// Persistent storage config.
#[derive(Builder, Default, Debug)]
#[builder(setter(into))]
pub struct VaultConfig {
    /// DB file path.
    /// Directory should exist, file is auto-created.
    pub db_path: String,

    /// How much entries to cache in memory.
    #[builder(default = 1000)]
    pub cache_size: usize,
}

/// Persistent storage metrics.
#[derive(PartialEq, Debug)]
pub struct VaultMetrics {
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

/// Persistent storage for run-time state.
/// Thread-safe, async.
///
/// Combines persistent DB (`redb`) + in-memory LRU cache (`quick-cache`).
/// Allows N concurrent reads and up to 1 concurrent write.
///
/// Returns ARCs with immutable caches owned by vault. Vault will do
/// copy-on-write if it needs to update cache, but it's not the unique owner.
#[derive(Debug)]
pub struct Vault {
    backend: Backend,
}

impl Vault {
    /// Create instance.
    pub async fn open(config: &VaultConfig) -> Result<Self> {
        let db = Db::open(config.db_path.as_str()).await?;

        Ok(Vault { backend: Backend::new(config, db) })
    }

    /// Get metrics.
    pub async fn metrics(&self) -> VaultMetrics {
        self.backend.metrics().await
    }

    /// List all endpoint UIDs.
    pub async fn list_endpoints(&self) -> Result<Arc<HashSet<Uid>>> {
        self.backend.list_entities(Backend::ENDPOINT_TABLE, &self.backend.endpoint_cache).await
    }

    /// Read endpoint by UID.
    pub async fn read_endpoint(&self, uid: &Uid) -> Result<Arc<EndpointSpec>> {
        self.backend
            .read_entity(Backend::ENDPOINT_TABLE, &self.backend.endpoint_cache, uid)
            .await
    }

    /// Write endpoint.
    pub async fn write_endpoint(&self, endpoint: &Arc<EndpointSpec>) -> Result<()> {
        self.backend
            .write_entity(
                Backend::ENDPOINT_TABLE,
                &self.backend.endpoint_cache,
                &endpoint.endpoint_uid,
                endpoint,
            )
            .await
    }

    /// Remove endpoint.
    pub async fn remove_endpoint(&self, uid: &Uid) -> Result<()> {
        self.backend
            .remove_entity(Backend::ENDPOINT_TABLE, &self.backend.endpoint_cache, uid)
            .await
    }

    /// List all stream UIDs.
    pub async fn list_streams(&self) -> Result<Arc<HashSet<Uid>>> {
        self.backend.list_entities(Backend::STREAM_TABLE, &self.backend.stream_cache).await
    }

    /// Read stream by UID.
    pub async fn read_stream(&self, uid: &Uid) -> Result<Arc<StreamSpec>> {
        self.backend.read_entity(Backend::STREAM_TABLE, &self.backend.stream_cache, uid).await
    }

    /// Write stream.
    pub async fn write_stream(&self, stream: &Arc<StreamSpec>) -> Result<()> {
        self.backend
            .write_entity(
                Backend::STREAM_TABLE,
                &self.backend.stream_cache,
                &stream.stream_uid,
                stream,
            )
            .await
    }

    /// Remove stream.
    pub async fn remove_stream(&self, uid: &Uid) -> Result<()> {
        self.backend
            .remove_entity(Backend::STREAM_TABLE, &self.backend.stream_cache, uid)
            .await
    }
}

#[derive(Debug)]
struct Cache<T> {
    /// LRU cache with the most used subset of entries present in DB.
    /// quick-cache will automatically remove least used entries when
    /// cache size exceeds configured limit.
    kvmap: quick_cache::sync::Cache<Uid, Arc<T>>,

    /// Lazy-initialized list of all keys present in DB.
    /// Empty until the first list_entities() call.
    /// After it becomes not-empty, write_entity() and remove_entity(), maintain it up-to-date.
    /// Lazy initialization allows faster startup time.
    kset: Option<Arc<HashSet<Uid>>>,
}

impl<T> Cache<T> {
    fn new(config: &VaultConfig) -> Self {
        Cache { kvmap: quick_cache::sync::Cache::new(config.cache_size), kset: None }
    }
}

#[derive(Debug)]
struct Backend {
    write_lock: Mutex<()>,

    // disk storage
    db: Arc<Db>,

    // memory cache
    endpoint_cache: RwLock<Cache<EndpointSpec>>,
    stream_cache: RwLock<Cache<StreamSpec>>,

    // metrics
    cache_hits: AtomicU64,
    cache_misses: AtomicU64,
    cache_drops: AtomicU64,
    cache_cows: AtomicU64,
}

impl Backend {
    const ENDPOINT_TABLE: Table = Table::new("endpoints");
    const STREAM_TABLE: Table = Table::new("streams");

    /// Constructor.
    fn new(config: &VaultConfig, db: Arc<Db>) -> Self {
        Backend {
            db,
            write_lock: Mutex::new(()),
            endpoint_cache: RwLock::new(Cache::new(config)),
            stream_cache: RwLock::new(Cache::new(config)),
            cache_hits: AtomicU64::new(0),
            cache_misses: AtomicU64::new(0),
            cache_drops: AtomicU64::new(0),
            cache_cows: AtomicU64::new(0),
        }
    }

    /// Get metrics.
    pub async fn metrics(&self) -> VaultMetrics {
        let db_metrics = self.db.metrics();

        let mut cache_size = 0;
        for cache in [&self.endpoint_cache] {
            let rlocked_cache = cache.read().await;
            cache_size += rlocked_cache.kvmap.len();
        }

        VaultMetrics {
            cache_size,
            cache_hits: self.cache_hits.load(Ordering::SeqCst),
            cache_misses: self.cache_misses.load(Ordering::SeqCst),
            cache_drops: self.cache_drops.load(Ordering::SeqCst),
            cache_cows: self.cache_cows.load(Ordering::SeqCst),
            db_reads: db_metrics.read_ops,
            db_writes: db_metrics.write_ops,
        }
    }

    /// Generic list implementation.
    /// First call will read the list from DB, subsequent calls will
    /// return value from memory.
    async fn list_entities<T>(
        &self, table: Table, cache: &RwLock<Cache<T>>,
    ) -> Result<Arc<HashSet<Uid>>> {
        // Fast path: keyset already initialized.
        {
            let rlocked_cache = cache.read().await;
            if let Some(kset_ptr) = rlocked_cache.kset.as_ref() {
                self.cache_hits.fetch_add(1, Ordering::SeqCst);
                return Ok(Arc::clone(kset_ptr));
            }
        }

        // Slow path: read keyset from db.
        let kset: Arc<HashSet<Uid>> = self.db.list_entry(table).await?;

        {
            let mut wlocked_cache = cache.write().await;

            // Concurrent list_entities() already initialized keyset, we have nothing to do.
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
    /// Returns value from in-memory cache if present, otherwise
    /// reads from DB and updates cache.
    async fn read_entity<T>(
        &self, table: Table, cache: &RwLock<Cache<T>>, uid: &Uid,
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
                //  - Concurrent write_entity() updated cache while we were reading from db.
                //    In this case, the value from cache takes priority over the value we
                //    got from db, because write_entity() updates cache before updating db.
                //
                //  - Concurrent read_entity() updated cache while we were reading from db.
                //    In this case, the value we got from db is equal to value in cache,
                //    and there is no difference which one to use.
                //
                // Therefore, we can safely drop the value that we've read from db and
                // instead return the value we've found in cache. This will give us correct
                // result in both cases.
                self.cache_drops.fetch_add(1, Ordering::SeqCst);
                return Ok(other_value);
            }
            wlocked_cache.kvmap.insert(*uid, Arc::clone(&value));
        }

        self.cache_misses.fetch_add(1, Ordering::SeqCst);
        Ok(value)
    }

    /// Generic write implementation for type T.
    /// Updates both in-memory cache and DB.
    /// Blocks until DB transaction is completed.
    async fn write_entity<T>(
        &self, table: Table, cache: &RwLock<Cache<T>>, uid: &Uid, value: &Arc<T>,
    ) -> Result<()>
    where
        T: Serialize + Validate + Sync + Send + Debug + 'static,
    {
        // Refuse to save invalid values.
        value.validate()?;

        // Serialize writes to ensure that cache and db updates from concurrent writes
        // won't overlap and create a mess. Db backend anyway supports only one
        // concurrent write, so we don't make it worse.
        let _guard = self.write_lock.lock();

        // Write to memory cache.
        {
            let mut wlocked_cache = cache.write().await;
            wlocked_cache.kvmap.insert(*uid, Arc::clone(value));

            // If keyset is already lazy-initialized, update it.
            if let Some(kset_ptr) = wlocked_cache.kset.as_mut() {
                // Copy-on-write: make_mut() will clone hashset if someone outside holds
                // another pointer to it. Hence we can safely modify the keyset, while
                // ARCs returned outside will remain immutable.
                let kset: &mut HashSet<Uid> = match Arc::get_mut(kset_ptr) {
                    Some(kset) => kset,
                    None => {
                        self.cache_cows.fetch_add(1, Ordering::SeqCst);
                        Arc::make_mut(kset_ptr)
                    },
                };
                kset.insert(*uid);
            }
        }

        // Write to db.
        self.db.write_entry(table, uid, value).await?;

        Ok(())
    }

    /// Generic delete implementation.
    /// Updates both in-memory cache and DB.
    /// Blocks until DB transaction is completed.
    async fn remove_entity<T>(
        &self, table: Table, cache: &RwLock<Cache<T>>, uid: &Uid,
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
        // read_entity() could read it from db and re-add it to cache.
        // Note: remove from cache even if removing from DB failed.
        {
            let mut wlocked_cache = cache.write().await;
            wlocked_cache.kvmap.remove(uid);

            // If keyset is already lazy-initialized, update it.
            if let Some(kset_ptr) = wlocked_cache.kset.as_mut() {
                // Copy-on-write, see comment in write_entity().
                let kset: &mut HashSet<Uid> = match Arc::get_mut(kset_ptr) {
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
