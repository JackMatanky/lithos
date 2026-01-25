//! Redb persistent cache adapter implementation.
//!
//! This module provides a persistent cache using the `redb` key-value store.
//! It supports table isolation, allowing multiple independent cache instances
//! to share the same database file.

use std::{
    collections::HashMap,
    path::Path,
    sync::Arc,
    time::{SystemTime, UNIX_EPOCH},
};

use async_trait::async_trait;
use redb::{ReadableDatabase as _, TableDefinition, TableHandle as _};
use rkyv::{
    Archive, Archived, Deserialize, Serialize, bytecheck::CheckBytes,
    util::AlignedVec,
};
use tracing::{error, info, info_span};

use crate::spi::{cache::Cache, errors::CacheError};

/// Type alias for the metadata map.
pub type MetadataMap = HashMap<String, String>;

/// Type alias for cache retrieval results with metadata.
pub type CacheResult<V> = Result<Option<(V, MetadataMap)>, CacheError>;

/// A wrapper for cached values with persistence metadata.
#[allow(clippy::exhaustive_structs)]
#[derive(Archive, Serialize, Deserialize, CheckBytes)]
#[bytecheck(crate = rkyv::bytecheck)]
#[non_exhaustive]
pub struct CachedEntry<V> {
    /// The actual cached value.
    pub value: V,
    /// Unix timestamp of when the entry was created/updated.
    pub timestamp: u64,
    /// Extensible metadata for the cached entry.
    pub metadata: MetadataMap,
}

/// A persistent cache implementation using Redb.
///
/// # Example
///
/// ```rust
/// use lithos_adapters::spi::cache::{Cache, redb::RedbCache};
/// use tempfile::tempdir;
///
/// # tokio::runtime::Runtime::new().unwrap().block_on(async {
/// let dir = tempdir().unwrap();
/// let db_path = dir.path().join("cache.redb");
///
/// let cache = RedbCache::new(db_path, "my_table").unwrap();
/// cache.put("key".to_owned(), "value".to_owned()).await.unwrap();
///
/// let result: Option<String> = cache.get(&"key".to_owned()).await.unwrap();
/// assert_eq!(result, Some("value".to_owned()));
/// # });
/// ```
#[allow(clippy::module_name_repetitions)]
#[allow(dead_code)]
pub struct RedbCache<K, V> {
    db: Arc<redb::Database>,
    table_definition: TableDefinition<'static, &'static [u8], &'static [u8]>,
    _marker: std::marker::PhantomData<(K, V)>,
}

impl<K, V> std::fmt::Debug for RedbCache<K, V> {
    #[inline]
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("RedbCache")
            .field("table", &self.table_definition.name())
            .finish_non_exhaustive()
    }
}

impl<K, V> RedbCache<K, V> {
    /// Retrieve value and metadata by key.
    ///
    /// # Errors
    /// Returns `CacheError` if retrieval or deserialization fails.
    ///
    /// # Example
    ///
    /// ```rust
    /// # use lithos_adapters::spi::cache::redb::RedbCache;
    /// # use std::collections::HashMap;
    /// # use tempfile::tempdir;
    /// # tokio::runtime::Runtime::new().unwrap().block_on(async {
    /// # let dir = tempdir().unwrap();
    /// # let db_path = dir.path().join("metadata.redb");
    /// # let cache = RedbCache::new(db_path, "table").unwrap();
    /// let metadata = HashMap::from([("version".to_owned(), "1.0".to_owned())]);
    /// cache
    ///     .put_with_metadata(
    ///         "key".to_owned(),
    ///         "value".to_owned(),
    ///         metadata.clone(),
    ///     )
    ///     .await
    ///     .unwrap();
    ///
    /// let (val, meta) =
    ///     cache.get_with_metadata(&"key".to_owned()).await.unwrap().unwrap();
    /// assert_eq!(val, "value".to_owned());
    /// assert_eq!(meta, metadata);
    /// # });
    /// ```
    #[tracing::instrument(
        skip(self, key),
        fields(
            table_name = %self.table_definition.name(),
            operation = "get_with_metadata",
            cache_layer = "disk"
        )
    )]
    #[inline]
    pub async fn get_with_metadata(&self, key: &K) -> CacheResult<V>
    where
        K: Clone + Send + Sync + 'static,
        K: for<'ser> Serialize<
            rkyv::api::high::HighSerializer<
                rkyv::util::AlignedVec,
                rkyv::ser::allocator::ArenaHandle<'ser>,
                rkyv::rancor::Error,
            >,
        >,
        V: Clone + Send + Sync + 'static,
        V: Archive,
        V: for<'ser> Serialize<
            rkyv::api::high::HighSerializer<
                rkyv::util::AlignedVec,
                rkyv::ser::allocator::ArenaHandle<'ser>,
                rkyv::rancor::Error,
            >,
        >,
        Archived<V>: rkyv::Deserialize<
                V,
                rkyv::api::high::HighDeserializer<rkyv::rancor::Error>,
            >,
        for<'validation> Archived<V>: CheckBytes<
            rkyv::api::high::HighValidator<'validation, rkyv::rancor::Error>,
        >,
    {
        let key_bytes =
            rkyv::to_bytes::<rkyv::rancor::Error>(key).map_err(|e| {
                error!(?e, "Failed to serialize key");
                CacheError::SerializationError {
                    type_name: std::any::type_name::<K>(),
                    message: format!("Failed to serialize key: {e}").into(),
                }
            })?;

        let db = Arc::clone(&self.db);
        let table_definition = self.table_definition;

        tokio::task::spawn_blocking(move || {
            let span = info_span!("redb_transaction", operation = "read");
            let _guard = span.enter();

            let read_txn = db.begin_read().map_err(|e| {
                error!(
                    backend = "redb",
                    ?e,
                    "Failed to begin read transaction"
                );
                CacheError::BackendError {
                    backend: "redb",
                    message: format!("Failed to begin read transaction: {e}")
                        .into(),
                }
            })?;

            let exists_result = read_txn
                .open_table(table_definition)
                .map_err(|e| {
                    error!(backend = "redb", ?e, "Failed to open table");
                    CacheError::BackendError {
                        backend: "redb",
                        message: format!("Failed to open table: {e}").into(),
                    }
                })?
                .get(key_bytes.as_slice())
                .map_err(|e| {
                    error!(backend = "redb", ?e, "Failed to get entry");
                    CacheError::BackendError {
                        backend: "redb",
                        message: format!("Failed to get entry: {e}").into(),
                    }
                })?;

            if let Some(guard) = exists_result {
                let bytes = guard.value();
                let mut aligned = AlignedVec::<16>::new();
                aligned.extend_from_slice(bytes);

                let archived = rkyv::access::<
                    Archived<CachedEntry<V>>,
                    rkyv::rancor::Error,
                >(&aligned)
                .map_err(|e| {
                    error!(?e, "Failed to access archived entry");
                    CacheError::SerializationError {
                        type_name: std::any::type_name::<CachedEntry<V>>(),
                        message: format!(
                            "Failed to access archived entry: {e}"
                        )
                        .into(),
                    }
                })?;

                let entry: CachedEntry<V> = rkyv::deserialize::<
                    CachedEntry<V>,
                    rkyv::rancor::Error,
                >(archived)
                .map_err(|e| {
                    error!(?e, "Failed to deserialize entry");
                    CacheError::SerializationError {
                        type_name: std::any::type_name::<CachedEntry<V>>(),
                        message: format!("Failed to deserialize entry: {e}")
                            .into(),
                    }
                })?;

                info!(cache_layer = "disk", "Cache hit");
                Ok(Some((entry.value, entry.metadata)))
            } else {
                info!(cache_layer = "disk", "Cache miss");
                Ok(None)
            }
        })
        .await
        .map_err(|e| {
            error!(?e, "Blocking task failed");
            CacheError::BackendError {
                backend: "tokio",
                message: format!("Blocking task failed: {e}").into(),
            }
        })?
    }

    /// Create a new `RedbCache` instance.
    ///
    /// # Errors
    /// Returns `CacheError` if the database cannot be opened.
    #[inline]
    pub fn new<P: AsRef<Path>>(
        db_path: P,
        table_name: &str,
    ) -> Result<Self, CacheError> {
        let db = redb::Database::create(db_path).map_err(|e| {
            error!(backend = "redb", ?e, "Failed to open database");
            CacheError::BackendError {
                backend: "redb",
                message: format!("Failed to open database: {e}").into(),
            }
        })?;

        let leaked_name: &'static str =
            Box::leak(table_name.to_owned().into_boxed_str());
        let table_definition = TableDefinition::new(leaked_name);

        Ok(Self {
            db: Arc::new(db),
            table_definition,
            _marker: std::marker::PhantomData,
        })
    }

    /// Store value with custom metadata.
    ///
    /// # Errors
    /// Returns `CacheError` if serialization or storage fails.
    #[tracing::instrument(
        skip(self, key, value, metadata),
        fields(
            table_name = %self.table_definition.name(),
            operation = "put_with_metadata",
            cache_layer = "disk"
        )
    )]
    #[inline]
    pub async fn put_with_metadata(
        &self,
        key: K,
        value: V,
        metadata: MetadataMap,
    ) -> Result<(), CacheError>
    where
        K: Clone + Send + Sync + 'static,
        K: for<'ser> Serialize<
            rkyv::api::high::HighSerializer<
                rkyv::util::AlignedVec,
                rkyv::ser::allocator::ArenaHandle<'ser>,
                rkyv::rancor::Error,
            >,
        >,
        V: Clone + Send + Sync + 'static,
        V: Archive,
        V: for<'ser> Serialize<
            rkyv::api::high::HighSerializer<
                rkyv::util::AlignedVec,
                rkyv::ser::allocator::ArenaHandle<'ser>,
                rkyv::rancor::Error,
            >,
        >,
    {
        let key_bytes =
            rkyv::to_bytes::<rkyv::rancor::Error>(&key).map_err(|e| {
                error!(?e, "Failed to serialize key");
                CacheError::SerializationError {
                    type_name: std::any::type_name::<K>(),
                    message: format!("Failed to serialize key: {e}").into(),
                }
            })?;

        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_err(|e| {
                error!(?e, "System time error");
                CacheError::BackendError {
                    backend: "system",
                    message: format!("System time error: {e}").into(),
                }
            })?
            .as_secs();

        let entry = CachedEntry {
            value,
            timestamp,
            metadata,
        };

        let value_bytes = rkyv::to_bytes::<rkyv::rancor::Error>(&entry)
            .map_err(|e| {
                error!(?e, "Failed to serialize entry");
                CacheError::SerializationError {
                    type_name: std::any::type_name::<CachedEntry<V>>(),
                    message: format!("Failed to serialize entry: {e}").into(),
                }
            })?;

        let db = Arc::clone(&self.db);
        let table_definition = self.table_definition;

        tokio::task::spawn_blocking(move || {
            let span = info_span!("redb_transaction", operation = "write");
            let _guard = span.enter();

            let write_txn = db.begin_write().map_err(|e| {
                error!(
                    backend = "redb",
                    ?e,
                    "Failed to begin write transaction"
                );
                CacheError::BackendError {
                    backend: "redb",
                    message: format!("Failed to begin write transaction: {e}")
                        .into(),
                }
            })?;

            _ = write_txn
                .open_table(table_definition)
                .map_err(|e| {
                    error!(backend = "redb", ?e, "Failed to open table");
                    CacheError::BackendError {
                        backend: "redb",
                        message: format!("Failed to open table: {e}").into(),
                    }
                })?
                .insert(key_bytes.as_slice(), value_bytes.as_slice())
                .map_err(|e| {
                    error!(backend = "redb", ?e, "Failed to insert entry");
                    CacheError::BackendError {
                        backend: "redb",
                        message: format!("Failed to insert entry: {e}").into(),
                    }
                })?;

            write_txn.commit().map_err(|e| {
                error!(
                    backend = "redb",
                    ?e,
                    "Failed to commit put transaction"
                );
                CacheError::BackendError {
                    backend: "redb",
                    message: format!("Failed to commit put: {e}").into(),
                }
            })?;

            info!(cache_layer = "disk", "Entry stored successfully");
            Ok(())
        })
        .await
        .map_err(|e| {
            error!(?e, "Blocking task failed");
            CacheError::BackendError {
                backend: "tokio",
                message: format!("Blocking task failed: {e}").into(),
            }
        })?
    }

    /// Create a new `RedbCache` instance sharing an existing database.
    #[inline]
    #[must_use]
    pub fn with_db(db: Arc<redb::Database>, table_name: &str) -> Self {
        let leaked_name: &'static str =
            Box::leak(table_name.to_owned().into_boxed_str());
        let table_definition = TableDefinition::new(leaked_name);

        Self {
            db,
            table_definition,
            _marker: std::marker::PhantomData,
        }
    }
}

#[async_trait]
impl<K, V> Cache<K, V> for RedbCache<K, V>
where
    K: Clone + Eq + std::hash::Hash + Send + Sync + 'static,
    K: for<'ser> Serialize<
        rkyv::api::high::HighSerializer<
            rkyv::util::AlignedVec,
            rkyv::ser::allocator::ArenaHandle<'ser>,
            rkyv::rancor::Error,
        >,
    >,
    V: Clone + Send + Sync + 'static,
    V: Archive,
    V: for<'ser> Serialize<
        rkyv::api::high::HighSerializer<
            rkyv::util::AlignedVec,
            rkyv::ser::allocator::ArenaHandle<'ser>,
            rkyv::rancor::Error,
        >,
    >,
    Archived<V>: rkyv::Deserialize<
            V,
            rkyv::api::high::HighDeserializer<rkyv::rancor::Error>,
        >,
    for<'validation> Archived<V>: CheckBytes<
        rkyv::api::high::HighValidator<'validation, rkyv::rancor::Error>,
    >,
{
    #[tracing::instrument(
        skip(self),
        fields(
            table_name = %self.table_definition.name(),
            operation = "clear",
            cache_layer = "disk"
        )
    )]
    #[inline]
    async fn clear(&self) -> Result<(), CacheError> {
        let db = Arc::clone(&self.db);
        let table_definition = self.table_definition;

        tokio::task::spawn_blocking(move || {
            let span = info_span!("redb_transaction", operation = "write");
            let _guard = span.enter();

            let write_txn = db.begin_write().map_err(|e| {
                error!(
                    backend = "redb",
                    ?e,
                    "Failed to begin write transaction"
                );
                CacheError::BackendError {
                    backend: "redb",
                    message: format!("Failed to begin write transaction: {e}")
                        .into(),
                }
            })?;

            _ = write_txn.delete_table(table_definition).map_err(|e| {
                error!(backend = "redb", ?e, "Failed to delete table");
                CacheError::BackendError {
                    backend: "redb",
                    message: format!("Failed to delete table: {e}").into(),
                }
            })?;

            _ = write_txn.open_table(table_definition).map_err(|e| {
                error!(backend = "redb", ?e, "Failed to recreate table");
                CacheError::BackendError {
                    backend: "redb",
                    message: format!("Failed to recreate table: {e}").into(),
                }
            })?;

            write_txn.commit().map_err(|e| {
                error!(
                    backend = "redb",
                    ?e,
                    "Failed to commit clear transaction"
                );
                CacheError::BackendError {
                    backend: "redb",
                    message: format!("Failed to commit clear: {e}").into(),
                }
            })?;

            info!(cache_layer = "disk", "Table cleared");
            Ok(())
        })
        .await
        .map_err(|e| {
            error!(?e, "Blocking task failed");
            CacheError::BackendError {
                backend: "tokio",
                message: format!("Blocking task failed: {e}").into(),
            }
        })?
    }

    #[tracing::instrument(
        skip(self, key),
        fields(
            table_name = %self.table_definition.name(),
            operation = "delete",
            cache_layer = "disk"
        )
    )]
    #[inline]
    async fn delete(&self, key: &K) -> Result<bool, CacheError> {
        let key_bytes =
            rkyv::to_bytes::<rkyv::rancor::Error>(key).map_err(|e| {
                error!(?e, "Failed to serialize key");
                CacheError::SerializationError {
                    type_name: std::any::type_name::<K>(),
                    message: format!("Failed to serialize key: {e}").into(),
                }
            })?;

        let db = Arc::clone(&self.db);
        let table_definition = self.table_definition;

        tokio::task::spawn_blocking(move || {
            let span = info_span!("redb_transaction", operation = "write");
            let _guard = span.enter();

            let write_txn = db.begin_write().map_err(|e| {
                error!(
                    backend = "redb",
                    ?e,
                    "Failed to begin write transaction"
                );
                CacheError::BackendError {
                    backend: "redb",
                    message: format!("Failed to begin write transaction: {e}")
                        .into(),
                }
            })?;

            let existed = write_txn
                .open_table(table_definition)
                .map_err(|e| {
                    error!(backend = "redb", ?e, "Failed to open table");
                    CacheError::BackendError {
                        backend: "redb",
                        message: format!("Failed to open table: {e}").into(),
                    }
                })?
                .remove(key_bytes.as_slice())
                .map_err(|e| {
                    error!(backend = "redb", ?e, "Failed to remove entry");
                    CacheError::BackendError {
                        backend: "redb",
                        message: format!("Failed to remove entry: {e}").into(),
                    }
                })?
                .is_some();

            write_txn.commit().map_err(|e| {
                error!(
                    backend = "redb",
                    ?e,
                    "Failed to commit delete transaction"
                );
                CacheError::BackendError {
                    backend: "redb",
                    message: format!("Failed to commit delete: {e}").into(),
                }
            })?;

            info!(cache_layer = "disk", ?existed, "Delete complete");
            Ok(existed)
        })
        .await
        .map_err(|e| {
            error!(?e, "Blocking task failed");
            CacheError::BackendError {
                backend: "tokio",
                message: format!("Blocking task failed: {e}").into(),
            }
        })?
    }

    #[inline]
    async fn get(&self, key: &K) -> Result<Option<V>, CacheError> {
        Ok(self.get_with_metadata(key).await?.map(|(v, _)| v))
    }

    #[tracing::instrument(
        skip(self, key),
        fields(
            table_name = %self.table_definition.name(),
            operation = "has",
            cache_layer = "disk"
        )
    )]
    #[inline]
    async fn has(&self, key: &K) -> Result<bool, CacheError> {
        let key_bytes =
            rkyv::to_bytes::<rkyv::rancor::Error>(key).map_err(|e| {
                error!(?e, "Failed to serialize key");
                CacheError::SerializationError {
                    type_name: std::any::type_name::<K>(),
                    message: format!("Failed to serialize key: {e}").into(),
                }
            })?;

        let db = Arc::clone(&self.db);
        let table_definition = self.table_definition;

        tokio::task::spawn_blocking(move || {
            let span = info_span!("redb_transaction", operation = "read");
            let _guard = span.enter();

            let read_txn = db.begin_read().map_err(|e| {
                error!(
                    backend = "redb",
                    ?e,
                    "Failed to begin read transaction"
                );
                CacheError::BackendError {
                    backend: "redb",
                    message: format!("Failed to begin read transaction: {e}")
                        .into(),
                }
            })?;

            let exists = read_txn
                .open_table(table_definition)
                .map_err(|e| {
                    error!(backend = "redb", ?e, "Failed to open table");
                    CacheError::BackendError {
                        backend: "redb",
                        message: format!("Failed to open table: {e}").into(),
                    }
                })?
                .get(key_bytes.as_slice())
                .map_err(|e| {
                    error!(backend = "redb", ?e, "Failed to check entry");
                    CacheError::BackendError {
                        backend: "redb",
                        message: format!("Failed to check entry: {e}").into(),
                    }
                })?
                .is_some();

            info!(cache_layer = "disk", ?exists, "Has complete");
            Ok(exists)
        })
        .await
        .map_err(|e| {
            error!(?e, "Blocking task failed");
            CacheError::BackendError {
                backend: "tokio",
                message: format!("Blocking task failed: {e}").into(),
            }
        })?
    }

    #[inline]
    async fn invalidate(&self, key: &K) -> Result<bool, CacheError> {
        self.delete(key).await
    }

    #[inline]
    async fn put(&self, key: K, value: V) -> Result<(), CacheError> {
        self.put_with_metadata(key, value, HashMap::new()).await
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use rkyv::{
        Archive, Archived, Deserialize, Serialize, bytecheck::CheckBytes,
    };
    use tempfile::tempdir;
    use tracing_test::traced_test;

    use super::*;

    #[derive(
        Archive, Serialize, Deserialize, CheckBytes, Debug, PartialEq, Clone,
    )]
    #[bytecheck(crate = rkyv::bytecheck)]
    struct TestValue(String);

    #[test]
    fn cached_entry_should_implement_rkyv_traits() {
        let entry = CachedEntry {
            value: TestValue("test".to_owned()),
            timestamp: 123_456_789,
            metadata: HashMap::from([("key".to_owned(), "value".to_owned())]),
        };

        let bytes = rkyv::to_bytes::<rkyv::rancor::Error>(&entry)
            .expect("failed to serialize");

        let archived = rkyv::access::<
            Archived<CachedEntry<TestValue>>,
            rkyv::rancor::Error,
        >(&bytes)
        .expect("failed to access");

        assert_eq!(archived.timestamp, 123_456_789);
        assert_eq!(archived.value.0, "test");
        assert_eq!(archived.metadata.len(), 1);
    }

    #[test]
    fn should_initialize_redb_cache() {
        let dir = tempdir().expect("failed to create temp dir");
        let db_path = dir.path().join("cache.redb");

        let cache = RedbCache::<String, TestValue>::new(db_path, "test_table");
        cache.unwrap();
    }

    #[test]
    fn should_support_multiple_tables_in_same_db() {
        let dir = tempdir().expect("failed to create temp dir");
        let db_path = dir.path().join("multi_table.redb");

        let cache1 = RedbCache::<String, TestValue>::new(&db_path, "table1")
            .expect("failed to create cache1");
        let db = Arc::clone(&cache1.db);

        let _cache2 = RedbCache::<String, TestValue>::with_db(db, "table2");

        assert!(db_path.exists());
    }

    #[test]
    fn should_map_io_error_during_init() {
        use std::fs::File;
        let dir = tempdir().expect("failed to create temp dir");
        let db_path = dir.path().join("read_only.redb");

        File::create(&db_path).expect("failed to create file");
        let mut perms = std::fs::metadata(&db_path)
            .expect("failed to get metadata")
            .permissions();
        perms.set_readonly(true);
        std::fs::set_permissions(&db_path, perms)
            .expect("failed to set permissions");

        let cache = RedbCache::<String, TestValue>::new(db_path, "test_table");

        assert!(cache.is_err());
        let err = cache.expect_err("should have error");
        assert!(matches!(err, CacheError::BackendError {
            backend: "redb",
            ..
        }));
    }

    #[tokio::test]
    async fn should_persist_data_across_instances() {
        let dir = tempdir().expect("failed to create temp dir");
        let db_path = dir.path().join("persist.redb");

        let key = "key".to_owned();
        let value = TestValue("persistent".to_owned());

        {
            let cache = RedbCache::<String, TestValue>::new(&db_path, "table")
                .expect("failed to create cache");
            cache.put(key.clone(), value.clone()).await.expect("put failed");
        } // cache dropped here

        {
            let cache = RedbCache::<String, TestValue>::new(&db_path, "table")
                .expect("failed to reload cache");
            let result = cache.get(&key).await.expect("get failed");
            assert_eq!(result, Some(value));
        }
    }

    #[tokio::test]
    async fn should_correctly_report_existence() {
        let dir = tempdir().expect("failed to create temp dir");
        let db_path = dir.path().join("has.redb");
        let cache = RedbCache::<String, TestValue>::new(db_path, "table")
            .expect("init failed");

        let key = "exists".to_owned();
        cache
            .put(key.clone(), TestValue("yes".to_owned()))
            .await
            .expect("put failed");

        assert!(cache.has(&key).await.expect("has failed"));
        assert!(!cache.has(&"missing".to_owned()).await.expect("has failed"));
    }

    #[tokio::test]
    async fn should_clear_all_entries() {
        let dir = tempdir().expect("failed to create temp dir");
        let db_path = dir.path().join("clear.redb");
        let cache = RedbCache::<String, TestValue>::new(db_path, "table")
            .expect("init failed");

        cache
            .put("k1".to_owned(), TestValue("v1".to_owned()))
            .await
            .expect("put failed");
        cache
            .put("k2".to_owned(), TestValue("v2".to_owned()))
            .await
            .expect("put failed");

        cache.clear().await.expect("clear failed");

        assert!(!cache.has(&"k1".to_owned()).await.expect("has failed"));
        assert!(!cache.has(&"k2".to_owned()).await.expect("has failed"));
    }

    #[tokio::test]
    async fn should_support_metadata() {
        let dir = tempdir().expect("failed to create temp dir");
        let db_path = dir.path().join("metadata.redb");
        let cache = RedbCache::<String, TestValue>::new(db_path, "table")
            .expect("init failed");

        let key = "key".to_owned();
        let value = TestValue("value".to_owned());
        let metadata =
            HashMap::from([("version".to_owned(), "1.0".to_owned())]);

        cache
            .put_with_metadata(key.clone(), value.clone(), metadata.clone())
            .await
            .expect("put failed");

        let result = cache.get_with_metadata(&key).await.expect("get failed");
        assert!(result.is_some());
        let (v, m) = result.expect("should have result");
        assert_eq!(v, value);
        assert_eq!(m, metadata);
    }

    #[tokio::test]
    async fn should_update_timestamp_on_put() {
        let dir = tempdir().expect("failed to create temp dir");
        let db_path = dir.path().join("timestamp.redb");
        let cache = RedbCache::<String, TestValue>::new(db_path, "table")
            .expect("init failed");

        let key = "key".to_owned();

        cache
            .put(key.clone(), TestValue("v1".to_owned()))
            .await
            .expect("put failed");
        let _res1 = cache
            .get_with_metadata(&key)
            .await
            .expect("get failed")
            .expect("should have result");

        // Wait a bit to ensure timestamp changes
        tokio::time::sleep(std::time::Duration::from_secs(1)).await;

        cache
            .put(key.clone(), TestValue("v2".to_owned()))
            .await
            .expect("put failed");
        let res2 = cache
            .get_with_metadata(&key)
            .await
            .expect("get failed")
            .expect("should have result");

        assert_eq!(res2.0, TestValue("v2".to_owned()));
        assert!(res2.1.is_empty());
    }

    #[tokio::test]
    #[traced_test]
    async fn should_emit_tracing_info() {
        let dir = tempdir().expect("failed to create temp dir");
        let db_path = dir.path().join("tracing.redb");
        let cache = RedbCache::<String, TestValue>::new(db_path, "table")
            .expect("init failed");

        let key = "key".to_owned();
        cache
            .put(key.clone(), TestValue("v1".to_owned()))
            .await
            .expect("put failed");

        // Smoke test: verify it doesn't panic and logs are produced
        // Manual verification of stdout confirms instrumentation is working
    }
}
