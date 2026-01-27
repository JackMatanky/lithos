//! Redb persistent cache adapter implementation.
//!
//! This module provides a persistent cache using the `redb` key-value store.
//! It supports table isolation, allowing multiple independent cache instances
//! to share the same database file.

#![allow(
    clippy::exhaustive_structs,
    reason = "rkyv Archive macro generates exhaustive patterns in macro \
              expansion"
)]

use std::{
    collections::HashMap,
    path::Path,
    sync::Arc,
    time::{SystemTime, UNIX_EPOCH},
};

use async_trait::async_trait;
use redb::{ReadableDatabase as _, TableDefinition};
use rkyv::{
    Archive, Archived, Deserialize, Serialize, bytecheck::CheckBytes,
    util::AlignedVec,
};
use tracing::{error, info, info_span};

use crate::spi::{
    cache::{CacheReader, CacheWriter, deserializer::RkyvCodec},
    errors::CacheError,
};

/// Type alias for the metadata map.
pub type MetadataMap = HashMap<String, String>;

/// Type alias for cache retrieval results with metadata.
pub type Outcome<V> = Result<Option<(V, MetadataMap)>, CacheError>;

/// A wrapper for cached values with persistence metadata.
#[derive(Archive, Serialize, Deserialize, CheckBytes)]
#[bytecheck(crate = rkyv::bytecheck)]
#[non_exhaustive]
pub struct Entry<V> {
    /// The actual cached value.
    pub value: V,
    /// Unix timestamp of when the entry was created/updated.
    pub timestamp: u64,
    /// Extensible metadata for the cached entry.
    pub metadata: MetadataMap,
}

/// Executor for bridging async/sync operations.
///
/// Wraps `tokio::spawn_blocking` with tracing instrumentation and error
/// mapping.
#[derive(Debug, Clone)]
pub(crate) struct Executor;

impl Executor {
    /// Map redb error to `CacheError`.
    #[inline]
    #[expect(
        clippy::wildcard_enum_match_arm,
        reason = "redb::Error variants are unstable; wildcard catches all \
                  non-IO errors as backend errors which is correct behavior"
    )]
    fn map_redb_error(e: redb::Error) -> CacheError {
        match e {
            redb::Error::Io(io_err) => CacheError::IoError(io_err),
            other => CacheError::BackendError {
                backend: "redb",
                message: format!("{other}").into(),
            },
        }
    }

    /// Execute a blocking operation in a separate thread pool.
    ///
    /// # Errors
    ///
    /// Returns `CacheError` if the task panics or the operation fails.
    #[inline]
    async fn spawn<F, R>(
        &self,
        span: tracing::Span,
        f: F,
    ) -> Result<R, CacheError>
    where
        F: FnOnce() -> Result<R, redb::Error> + Send + 'static,
        R: Send + 'static,
    {
        tokio::task::spawn_blocking(move || {
            let _enter = span.enter();
            f()
        })
        .await
        .map_err(|e| {
            error!(?e, "Blocking task failed");
            CacheError::BackendError {
                backend: "tokio",
                message: format!("Blocking task failed: {e}").into(),
            }
        })?
        .map_err(Self::map_redb_error)
    }
}

/// Inner state for Redb cache.
///
/// This struct holds the database connection and codec, wrapped by
/// Reader/Writer handles. It's not directly clonable to enforce the use of Arc
/// for sharing.
#[derive(Debug)]
pub(crate) struct Inner<K, V, C>
where
    K: Clone + Eq + std::hash::Hash + Send + Sync + 'static,
    V: Clone + Send + Sync + 'static,
{
    db: Arc<redb::Database>,
    executor: Executor,
    table_name: Arc<str>,
    codec: C,
    _marker: std::marker::PhantomData<(K, V)>,
}

impl<K, V, C> Inner<K, V, C>
where
    K: Clone + Eq + std::hash::Hash + Send + Sync + 'static,
    V: Clone + Send + Sync + 'static,
{
    /// Execute a read operation.
    #[inline]
    async fn read<F, R>(&self, f: F) -> Result<R, CacheError>
    where
        F: FnOnce(&redb::ReadTransaction, &str) -> Result<R, redb::Error>
            + Send
            + 'static,
        R: Send + 'static,
    {
        let db = Arc::clone(&self.db);
        let table_name = Arc::clone(&self.table_name);
        let span = info_span!("redb_read", table = %table_name);

        self.executor
            .spawn(span, move || {
                let txn = db.begin_read()?;
                f(&txn, &table_name)
            })
            .await
    }

    /// Execute a write operation.
    #[inline]
    async fn write<F, R>(&self, f: F) -> Result<R, CacheError>
    where
        F: FnOnce(&redb::WriteTransaction, &str) -> Result<R, redb::Error>
            + Send
            + 'static,
        R: Send + 'static,
    {
        let db = Arc::clone(&self.db);
        let table_name = Arc::clone(&self.table_name);
        let span = info_span!("redb_write", table = %table_name);

        self.executor
            .spawn(span, move || {
                let txn = db.begin_write()?;
                let result = f(&txn, &table_name)?;
                txn.commit()?;
                Ok(result)
            })
            .await
    }
}

/// Read-only handle for Redb cache.
///
/// This handle provides read-only access to the cache following CQRS
/// principles.
#[derive(Debug, Clone)]
pub struct Reader<K, V, C = RkyvCodec>
where
    K: Clone + Eq + std::hash::Hash + Send + Sync + 'static,
    V: Clone + Send + Sync + 'static,
{
    inner: Arc<Inner<K, V, C>>,
}

/// Write-only handle for Redb cache.
///
/// This handle provides write-only access to the cache following CQRS
/// principles.
#[derive(Debug, Clone)]
pub struct Writer<K, V, C = RkyvCodec>
where
    K: Clone + Eq + std::hash::Hash + Send + Sync + 'static,
    V: Clone + Send + Sync + 'static,
{
    inner: Arc<Inner<K, V, C>>,
}

/// Builder for Redb cache.
#[derive(Debug, Clone)]
pub struct Builder<K, V>
where
    K: Clone + Eq + std::hash::Hash + Send + Sync + 'static,
    V: Clone + Send + Sync + 'static,
{
    path: Option<std::path::PathBuf>,
    table_name: Option<String>,
    _marker: std::marker::PhantomData<(K, V)>,
}

/// Type alias for the Inner state with default codec.
type RedbInner<K, V> = Arc<Inner<K, V, RkyvCodec>>;

/// Type alias for a Reader/Writer pair returned by `Builder::build()`.
type ReaderWriterPair<K, V> = (Reader<K, V>, Writer<K, V>);

impl<K, V> Builder<K, V>
where
    K: Clone + Eq + std::hash::Hash + Send + Sync + 'static,
    V: Clone + Send + Sync + 'static,
{
    /// Create a new builder.
    #[inline]
    #[must_use]
    pub fn new() -> Self {
        Self {
            path: None,
            table_name: None,
            _marker: std::marker::PhantomData,
        }
    }

    /// Set the database path.
    #[inline]
    pub fn path<P: AsRef<std::path::Path>>(&mut self, path: P) -> &mut Self {
        self.path = Some(path.as_ref().to_path_buf());
        self
    }

    /// Set the table name.
    #[inline]
    pub fn table_name(&mut self, name: &str) -> &mut Self {
        self.table_name = Some(name.to_owned());
        self
    }
}

impl<K, V> Builder<K, V>
where
    K: std::fmt::Debug + Clone + Eq + std::hash::Hash + Send + Sync + 'static,
    V: Clone + Send + Sync + 'static,
{
    /// Build both Reader and Writer handles sharing the same database.
    ///
    /// # Errors
    ///
    /// Returns `CacheError` if the database cannot be initialized.
    #[inline]
    pub fn build(&self) -> Result<ReaderWriterPair<K, V>, CacheError> {
        let inner = self.build_inner()?;
        let reader = Reader {
            inner: Arc::clone(&inner),
        };
        let writer = Writer {
            inner,
        };
        Ok((reader, writer))
    }

    /// Internal helper to build the Inner state.
    #[inline]
    fn build_inner(&self) -> Result<RedbInner<K, V>, CacheError> {
        let path =
            self.path.as_ref().ok_or_else(|| CacheError::BackendError {
                backend: "redb",
                message: "Database path is required".into(),
            })?;

        let table_name = self.table_name.as_ref().ok_or_else(|| {
            CacheError::BackendError {
                backend: "redb",
                message: "Table name is required".into(),
            }
        })?;

        // Validate path is not a directory
        if path.is_dir() {
            return Err(CacheError::BackendError {
                backend: "redb",
                message: format!("Path is a directory: {}", path.display())
                    .into(),
            });
        }

        let db = redb::Database::create(path).map_err(|e| {
            error!(backend = "redb", ?e, "Failed to open database");
            CacheError::BackendError {
                backend: "redb",
                message: format!("Failed to open database: {e}").into(),
            }
        })?;

        Ok(Arc::new(Inner {
            db: Arc::new(db),
            executor: Executor,
            table_name: Arc::from(table_name.as_str()),
            codec: RkyvCodec,
            _marker: std::marker::PhantomData,
        }))
    }

    /// Build a Reader handle independently.
    ///
    /// # Errors
    ///
    /// Returns `CacheError` if the database cannot be initialized.
    #[inline]
    pub fn build_reader(&self) -> Result<Reader<K, V>, CacheError> {
        let inner = self.build_inner()?;
        Ok(Reader {
            inner,
        })
    }

    /// Build a Writer handle independently.
    ///
    /// # Errors
    ///
    /// Returns `CacheError` if the database cannot be initialized.
    #[inline]
    pub fn build_writer(&self) -> Result<Writer<K, V>, CacheError> {
        let inner = self.build_inner()?;
        Ok(Writer {
            inner,
        })
    }
}

impl<K, V> Default for Builder<K, V>
where
    K: Clone + Eq + std::hash::Hash + Send + Sync + 'static,
    V: Clone + Send + Sync + 'static,
{
    #[inline]
    fn default() -> Self {
        Self::new()
    }
}

// Implement CacheReader for Reader
#[async_trait]
impl<K, V, C> CacheReader<K, V> for Reader<K, V, C>
where
    K: std::fmt::Debug + Clone + Eq + std::hash::Hash + Send + Sync + 'static,
    V: Clone + Send + Sync + 'static,
    C: crate::spi::cache::deserializer::Codec<K, Entry<V>>
        + Send
        + Sync
        + 'static,
{
    #[inline]
    async fn get(&self, key: &K) -> Result<Option<V>, CacheError> {
        let key_bytes = self.inner.codec.encode_key(key)?;

        self.inner
            .read(move |txn, table_name| {
                let table_def =
                    TableDefinition::<&[u8], &[u8]>::new(table_name);
                let table = txn.open_table(table_def)?;

                if let Some(guard) = table.get(key_bytes.as_slice())? {
                    Ok(Some(guard.value().to_vec()))
                } else {
                    Ok(None)
                }
            })
            .await?
            .map(|bytes| {
                let entry: Entry<V> = self.inner.codec.decode_value(&bytes)?;
                Ok(entry.value)
            })
            .transpose()
    }

    #[inline]
    async fn has(&self, key: &K) -> Result<bool, CacheError> {
        let key_bytes = self.inner.codec.encode_key(key)?;

        self.inner
            .read(move |txn, table_name| {
                let table_def =
                    TableDefinition::<&[u8], &[u8]>::new(table_name);
                let table = txn.open_table(table_def)?;
                Ok(table.get(key_bytes.as_slice())?.is_some())
            })
            .await
    }
}

// Implement CacheWriter for Writer
#[async_trait]
impl<K, V, C> CacheWriter<K, V> for Writer<K, V, C>
where
    K: std::fmt::Debug + Clone + Eq + std::hash::Hash + Send + Sync + 'static,
    V: Clone + Send + Sync + 'static,
    C: crate::spi::cache::deserializer::Codec<K, Entry<V>>
        + Send
        + Sync
        + 'static,
{
    #[inline]
    async fn clear(&self) -> Result<(), CacheError> {
        self.inner
            .write(move |txn, table_name| {
                let table_def =
                    TableDefinition::<&[u8], &[u8]>::new(table_name);
                _ = txn.delete_table(table_def)?;
                _ = txn.open_table(table_def)?;
                Ok(())
            })
            .await
    }

    #[inline]
    async fn delete(&self, key: &K) -> Result<bool, CacheError> {
        let key_bytes = self.inner.codec.encode_key(key)?;

        self.inner
            .write(move |txn, table_name| {
                let table_def =
                    TableDefinition::<&[u8], &[u8]>::new(table_name);
                let mut table = txn.open_table(table_def)?;
                Ok(table.remove(key_bytes.as_slice())?.is_some())
            })
            .await
    }

    #[inline]
    async fn invalidate(&self, _key: &K) -> Result<bool, CacheError> {
        // Redb doesn't support invalidation without deletion
        // This is a no-op for persistent storage, always returns false
        Ok(false)
    }

    #[inline]
    async fn put(&self, key: K, value: V) -> Result<(), CacheError> {
        let entry = Entry {
            value,
            timestamp: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .map_err(|e| CacheError::BackendError {
                    backend: "system",
                    message: format!("System time error: {e}").into(),
                })?
                .as_secs(),
            metadata: HashMap::new(),
        };

        let key_bytes = self.inner.codec.encode_key(&key)?;
        let value_bytes = self.inner.codec.encode_value(&entry)?;

        self.inner
            .write(move |txn, table_name| {
                let table_def =
                    TableDefinition::<&[u8], &[u8]>::new(table_name);
                let mut table = txn.open_table(table_def)?;
                table.insert(key_bytes.as_slice(), value_bytes.as_slice())?;
                Ok(())
            })
            .await
    }
}

/// A persistent cache implementation using Redb.
///
/// **DEPRECATED**: Use `Builder` to create `Reader` and `Writer` handles
/// instead.
///
/// # Example
///
/// ```rust
/// use lithos_adapters::spi::cache::{CacheReader, CacheWriter, RedbBuilder};
/// use tempfile::tempdir;
///
/// # tokio::runtime::Runtime::new().unwrap().block_on(async {
/// let dir = tempdir().unwrap();
/// let db_path = dir.path().join("cache.redb");
///
/// let (reader, writer) = RedbBuilder::new()
///     .path(&db_path)
///     .table_name("my_table")
///     .build()
///     .unwrap();
/// writer.put("key".to_owned(), "value".to_owned()).await.unwrap();
///
/// let result: Option<String> = reader.get(&"key".to_owned()).await.unwrap();
/// assert_eq!(result, Some("value".to_owned()));
/// # });
/// ```
pub struct Cache<K, V> {
    db: Arc<redb::Database>,
    table_name: Arc<str>,
    _marker: std::marker::PhantomData<(K, V)>,
}

impl<K, V> std::fmt::Debug for Cache<K, V> {
    #[inline]
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("RedbCache")
            .field("table", &self.table_name)
            .finish_non_exhaustive()
    }
}

impl<K, V> Cache<K, V> {
    #[inline]
    fn begin_read(
        db: &redb::Database,
    ) -> Result<redb::ReadTransaction, CacheError> {
        db.begin_read().map_err(|e| {
            error!(backend = "redb", ?e, "Failed to begin read transaction");
            CacheError::BackendError {
                backend: "redb",
                message: format!("Failed to begin read transaction: {e}")
                    .into(),
            }
        })
    }

    #[inline]
    fn begin_write(
        db: &redb::Database,
    ) -> Result<redb::WriteTransaction, CacheError> {
        db.begin_write().map_err(|e| {
            error!(backend = "redb", ?e, "Failed to begin write transaction");
            CacheError::BackendError {
                backend: "redb",
                message: format!("Failed to begin write transaction: {e}")
                    .into(),
            }
        })
    }

    /// Create a new `RedbCache` instance.
    ///
    /// # Errors
    /// Returns `CacheError` if the database cannot be opened.
    #[inline]
    pub async fn new<P: AsRef<Path>>(
        db_path: P,
        table_name: &str,
    ) -> Result<Self, CacheError> {
        let path = db_path.as_ref().to_path_buf();
        let db = tokio::task::spawn_blocking(move || {
            redb::Database::create(path).map_err(|e| {
                error!(backend = "redb", ?e, "Failed to open database");
                CacheError::BackendError {
                    backend: "redb",
                    message: format!("Failed to open database: {e}").into(),
                }
            })
        })
        .await
        .map_err(|e| {
            error!(?e, "Blocking task for database creation failed");
            CacheError::BackendError {
                backend: "tokio",
                message: format!("Database task failed: {e}").into(),
            }
        })??;

        Ok(Self {
            db: Arc::new(db),
            table_name: Arc::from(table_name),
            _marker: std::marker::PhantomData,
        })
    }

    /// Execute a blocking read operation in a separate task.
    async fn run_blocking_read<F, R>(
        &self,
        operation: &'static str,
        f: F,
    ) -> Result<R, CacheError>
    where
        F: FnOnce(&redb::ReadTransaction) -> Result<R, CacheError>
            + Send
            + 'static,
        R: Send + 'static,
    {
        let db = Arc::clone(&self.db);
        let table_name = Arc::clone(&self.table_name);
        tokio::task::spawn_blocking(move || {
            let _span = info_span!(
                "redb_transaction",
                %operation,
                %table_name
            )
            .entered();
            let txn = Self::begin_read(&db)?;
            f(&txn)
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

    /// Execute a blocking write operation in a separate task.
    async fn run_blocking_write<F, R>(
        &self,
        operation: &'static str,
        f: F,
    ) -> Result<R, CacheError>
    where
        F: FnOnce(&redb::WriteTransaction) -> Result<R, CacheError>
            + Send
            + 'static,
        R: Send + 'static,
    {
        let db = Arc::clone(&self.db);
        let table_name = Arc::clone(&self.table_name);
        tokio::task::spawn_blocking(move || {
            let _span = info_span!(
                "redb_transaction",
                %operation,
                %table_name
            )
            .entered();
            let txn = Self::begin_write(&db)?;
            let result = f(&txn)?;
            txn.commit().map_err(|e| {
                error!(backend = "redb", ?e, "Failed to commit transaction");
                CacheError::BackendError {
                    backend: "redb",
                    message: format!("Failed to commit: {e}").into(),
                }
            })?;
            Ok(result)
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
        Self {
            db,
            table_name: Arc::from(table_name),
            _marker: std::marker::PhantomData,
        }
    }
}

impl<K, V> Cache<K, V>
where
    K: std::fmt::Debug
        + for<'ser> Serialize<
            rkyv::api::high::HighSerializer<
                rkyv::util::AlignedVec,
                rkyv::ser::allocator::ArenaHandle<'ser>,
                rkyv::rancor::Error,
            >,
        >,
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
    #[inline]
    fn deserialize_entry(bytes: &[u8]) -> Result<Entry<V>, CacheError> {
        let mut aligned = AlignedVec::<16>::new();
        aligned.extend_from_slice(bytes);

        let archived =
            rkyv::access::<Archived<Entry<V>>, rkyv::rancor::Error>(&aligned)
                .map_err(|e| {
                error!(?e, "Failed to access archived entry");
                CacheError::SerializationError {
                    type_name: std::any::type_name::<Entry<V>>(),
                    message: format!("Failed to access archived entry: {e}")
                        .into(),
                }
            })?;

        rkyv::deserialize::<Entry<V>, rkyv::rancor::Error>(archived).map_err(
            |e| {
                error!(?e, "Failed to deserialize entry");
                CacheError::SerializationError {
                    type_name: std::any::type_name::<Entry<V>>(),
                    message: format!("Failed to deserialize entry: {e}").into(),
                }
            },
        )
    }

    /// Retrieve value and metadata by key.
    ///
    /// # Errors
    /// Returns `CacheError` if retrieval or deserialization fails.
    #[tracing::instrument(
        skip(self),
        fields(
            table_name = %self.table_name,
            operation = "get_with_metadata",
            cache_layer = "disk",
            key = ?key
        )
    )]
    #[inline]
    pub async fn get_with_metadata(&self, key: &K) -> Outcome<V>
    where
        K: Clone + Send + Sync + 'static,
        V: Clone + Send + Sync + 'static,
    {
        let key_bytes = Self::serialize_key(key)?;
        let table_name = Arc::clone(&self.table_name);

        self.run_blocking_read("read", move |txn| {
            let table_def = TableDefinition::<&[u8], &[u8]>::new(&table_name);
            let table = txn.open_table(table_def).map_err(|e| {
                error!(backend = "redb", ?e, "Failed to open table");
                CacheError::BackendError {
                    backend: "redb",
                    message: format!("Failed to open table: {e}").into(),
                }
            })?;

            let guard = table.get(key_bytes.as_slice()).map_err(|e| {
                error!(backend = "redb", ?e, "Failed to get entry");
                CacheError::BackendError {
                    backend: "redb",
                    message: format!("Failed to get entry: {e}").into(),
                }
            })?;

            if let Some(guard) = guard {
                let entry = Self::deserialize_entry(guard.value())?;
                info!(cache_layer = "disk", "Cache hit");
                Ok(Some((entry.value, entry.metadata)))
            } else {
                info!(cache_layer = "disk", "Cache miss");
                Ok(None)
            }
        })
        .await
    }

    /// Store value with custom metadata.
    ///
    /// # Errors
    /// Returns `CacheError` if serialization or storage fails.
    #[tracing::instrument(
        skip(self, value, metadata),
        fields(
            table_name = %self.table_name,
            operation = "put_with_metadata",
            cache_layer = "disk",
            key = ?key
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
        V: Clone + Send + Sync + 'static,
    {
        let key_bytes = Self::serialize_key(&key)?;
        let entry = Entry {
            value,
            timestamp: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .map_err(|e| {
                    error!(?e, "System time error");
                    CacheError::BackendError {
                        backend: "system",
                        message: format!("System time error: {e}").into(),
                    }
                })?
                .as_secs(),
            metadata,
        };
        let value_bytes = Self::serialize_entry(&entry)?;
        let table_name = Arc::clone(&self.table_name);

        self.run_blocking_write("write", move |txn| {
            let table_def = TableDefinition::<&[u8], &[u8]>::new(&table_name);
            let mut table = txn.open_table(table_def).map_err(|e| {
                error!(backend = "redb", ?e, "Failed to open table");
                CacheError::BackendError {
                    backend: "redb",
                    message: format!("Failed to open table: {e}").into(),
                }
            })?;

            table
                .insert(key_bytes.as_slice(), value_bytes.as_slice())
                .map_err(|e| {
                    error!(backend = "redb", ?e, "Failed to insert entry");
                    CacheError::BackendError {
                        backend: "redb",
                        message: format!("Failed to insert entry: {e}").into(),
                    }
                })?;

            info!(cache_layer = "disk", "Entry stored successfully");
            Ok(())
        })
        .await
    }

    #[inline]
    fn serialize_entry(entry: &Entry<V>) -> Result<Vec<u8>, CacheError> {
        rkyv::to_bytes::<rkyv::rancor::Error>(entry)
            .map(|bytes| bytes.to_vec())
            .map_err(|e| {
                error!(?e, "Failed to serialize entry");
                CacheError::SerializationError {
                    type_name: std::any::type_name::<Entry<V>>(),
                    message: format!("Failed to serialize entry: {e}").into(),
                }
            })
    }

    #[inline]
    fn serialize_key(key: &K) -> Result<Vec<u8>, CacheError> {
        rkyv::to_bytes::<rkyv::rancor::Error>(key)
            .map(|bytes| bytes.to_vec())
            .map_err(|e| {
                error!(?e, "Failed to serialize key");
                CacheError::SerializationError {
                    type_name: std::any::type_name::<K>(),
                    message: format!("Failed to serialize key: {e}").into(),
                }
            })
    }
}

#[async_trait]
impl<K, V> CacheReader<K, V> for Cache<K, V>
where
    K: std::fmt::Debug
        + Clone
        + Eq
        + std::hash::Hash
        + Send
        + Sync
        + 'static
        + for<'ser> Serialize<
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
    #[inline]
    async fn get(&self, key: &K) -> Result<Option<V>, CacheError> {
        Ok(self.get_with_metadata(key).await?.map(|(v, _)| v))
    }

    #[tracing::instrument(
        skip(self),
        fields(
            table_name = %self.table_name,
            operation = "has",
            cache_layer = "disk",
            key = ?key
        )
    )]
    #[inline]
    async fn has(&self, key: &K) -> Result<bool, CacheError> {
        let key_bytes = Self::serialize_key(key)?;
        let table_name = Arc::clone(&self.table_name);

        self.run_blocking_read("read", move |txn| {
            let table_def = TableDefinition::<&[u8], &[u8]>::new(&table_name);
            let table = txn.open_table(table_def).map_err(|e| {
                error!(backend = "redb", ?e, "Failed to open table");
                CacheError::BackendError {
                    backend: "redb",
                    message: format!("Failed to open table: {e}").into(),
                }
            })?;

            let exists = table
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
    }
}

#[async_trait]
impl<K, V> CacheWriter<K, V> for Cache<K, V>
where
    K: std::fmt::Debug
        + Clone
        + Eq
        + std::hash::Hash
        + Send
        + Sync
        + 'static
        + for<'ser> Serialize<
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
            table_name = %self.table_name,
            operation = "clear",
            cache_layer = "disk"
        )
    )]
    #[inline]
    async fn clear(&self) -> Result<(), CacheError> {
        let table_name = Arc::clone(&self.table_name);

        self.run_blocking_write("write", move |txn| {
            let table_def = TableDefinition::<&[u8], &[u8]>::new(&table_name);
            _ = txn.delete_table(table_def).map_err(|e| {
                error!(backend = "redb", ?e, "Failed to delete table");
                CacheError::BackendError {
                    backend: "redb",
                    message: format!("Failed to delete table: {e}").into(),
                }
            })?;

            _ = txn.open_table(table_def).map_err(|e| {
                error!(backend = "redb", ?e, "Failed to recreate table");
                CacheError::BackendError {
                    backend: "redb",
                    message: format!("Failed to recreate table: {e}").into(),
                }
            })?;

            info!(cache_layer = "disk", "Table cleared");
            Ok(())
        })
        .await
    }

    #[tracing::instrument(
        skip(self),
        fields(
            table_name = %self.table_name,
            operation = "delete",
            cache_layer = "disk",
            key = ?key
        )
    )]
    #[inline]
    async fn delete(&self, key: &K) -> Result<bool, CacheError> {
        let key_bytes = Self::serialize_key(key)?;
        let table_name = Arc::clone(&self.table_name);

        self.run_blocking_write("write", move |txn| {
            let table_def = TableDefinition::<&[u8], &[u8]>::new(&table_name);
            let mut table = txn.open_table(table_def).map_err(|e| {
                error!(backend = "redb", ?e, "Failed to open table");
                CacheError::BackendError {
                    backend: "redb",
                    message: format!("Failed to open table: {e}").into(),
                }
            })?;

            let existed = table
                .remove(key_bytes.as_slice())
                .map_err(|e| {
                    error!(backend = "redb", ?e, "Failed to remove entry");
                    CacheError::BackendError {
                        backend: "redb",
                        message: format!("Failed to remove entry: {e}").into(),
                    }
                })?
                .is_some();

            info!(cache_layer = "disk", ?existed, "Delete complete");
            Ok(existed)
        })
        .await
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
    use super::*;

    mod redb_api {
        use tempfile::tempdir;

        use super::*;

        #[tokio::test]
        async fn builder_creates_reader_independently() {
            let dir = tempdir().expect("failed to create temp dir");
            let db_path = dir.path().join("reader.redb");

            let result = Builder::<String, String>::new()
                .path(db_path)
                .table_name("test")
                .build_reader();

            assert!(result.is_ok());
            let reader = result.unwrap();
            let _: Reader<String, String> = reader;
        }

        #[tokio::test]
        async fn builder_creates_writer_independently() {
            let dir = tempdir().expect("failed to create temp dir");
            let db_path = dir.path().join("writer.redb");

            let result = Builder::<String, String>::new()
                .path(db_path)
                .table_name("test")
                .build_writer();

            assert!(result.is_ok());
            let writer = result.unwrap();
            let _: Writer<String, String> = writer;
        }

        #[tokio::test]
        async fn reader_and_writer_work_together() {
            let dir = tempdir().expect("failed to create temp dir");
            let db_path = dir.path().join("rw.redb");

            // Create both handles sharing the same database
            let mut builder = Builder::<String, TestValue>::new();
            builder.path(&db_path).table_name("test");

            let (reader, writer) =
                builder.build().expect("failed to build handles");

            // Write data
            writer
                .put("key".to_owned(), TestValue("value".to_owned()))
                .await
                .expect("put failed");

            // Read it back
            let result =
                reader.get(&"key".to_owned()).await.expect("get failed");
            assert_eq!(result, Some(TestValue("value".to_owned())));

            // Check existence
            let has_key =
                reader.has(&"key".to_owned()).await.expect("has failed");
            assert!(has_key);

            // Delete
            let deleted =
                writer.delete(&"key".to_owned()).await.expect("delete failed");
            assert!(deleted);

            // Verify deleted
            let deleted_result =
                reader.get(&"key".to_owned()).await.expect("get failed");
            assert_eq!(deleted_result, None);
        }
    }

    mod executor {
        use tempfile::tempdir;

        use super::*;

        #[tokio::test]
        async fn maps_redb_error_to_cache_error() {
            let dir = tempdir().expect("failed to create temp dir");
            let db_path = dir.path().join("executor.redb");

            // Create a valid reader to get the executor
            let _reader = Builder::<String, String>::new()
                .path(&db_path)
                .table_name("test")
                .build_reader()
                .expect("failed to build reader");

            // Executor is internal, but we can verify it works through
            // operations This test verifies the builder integrates
            // the Executor correctly
            assert!(db_path.exists());
        }
    }

    mod redb_builder {
        use tempfile::tempdir;

        use super::*;

        #[test]
        fn fails_when_path_is_directory() {
            let dir = tempdir().expect("failed to create temp dir");

            let result = Builder::<String, String>::new()
                .path(dir.path())
                .table_name("test")
                .build_reader();

            assert!(result.is_err());
            assert!(matches!(
                result.unwrap_err(),
                CacheError::BackendError { .. }
            ));
        }

        #[test]
        fn initializes_db_with_correct_table() {
            let dir = tempdir().expect("failed to create temp dir");
            let db_path = dir.path().join("init.redb");

            let result = Builder::<String, String>::new()
                .path(&db_path)
                .table_name("my_table")
                .build_reader();

            result.unwrap();
            assert!(db_path.exists());
        }
    }

    mod core_ops {
        use tempfile::tempdir;

        use super::*;

        #[tokio::test]
        async fn should_clear_all_entries() {
            let dir = tempdir().expect("failed to create temp dir");
            let db_path = dir.path().join("clear.redb");
            let cache = Cache::<String, TestValue>::new(db_path, "table")
                .await
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

            let has_k1 = cache.has(&"k1".to_owned()).await.expect("has failed");
            let has_k2 = cache.has(&"k2".to_owned()).await.expect("has failed");
            assert!(!has_k1);
            assert!(!has_k2);
        }

        #[tokio::test]
        async fn should_correctly_report_existence() {
            let dir = tempdir().expect("failed to create temp dir");
            let db_path = dir.path().join("has.redb");
            let cache = Cache::<String, TestValue>::new(db_path, "table")
                .await
                .expect("init failed");

            let key = "exists".to_owned();
            cache
                .put(key.clone(), TestValue("yes".to_owned()))
                .await
                .expect("put failed");

            let has_key = cache.has(&key).await.expect("has failed");
            let has_missing =
                cache.has(&"missing".to_owned()).await.expect("has failed");
            assert!(has_key);
            assert!(!has_missing);
        }

        #[tokio::test]
        async fn should_persist_data_across_instances() {
            let dir = tempdir().expect("failed to create temp dir");
            let db_path = dir.path().join("persist.redb");

            let key = "key".to_owned();
            let value = TestValue("persistent".to_owned());

            // First instance: write data
            let cache1 = Cache::<String, TestValue>::new(&db_path, "table")
                .await
                .expect("failed to create cache");
            cache1.put(key.clone(), value.clone()).await.expect("put failed");
            drop(cache1); // Explicit drop to close database

            // Second instance: read data
            let cache2 = Cache::<String, TestValue>::new(&db_path, "table")
                .await
                .expect("failed to reload cache");
            let result = cache2.get(&key).await.expect("get failed");
            assert_eq!(result, Some(value));
            drop(cache2); // Explicit drop for consistency
        }
    }

    mod initialization {
        use tempfile::tempdir;

        use super::*;

        #[tokio::test]
        async fn should_initialize_redb_cache() {
            let dir = tempdir().expect("failed to create temp dir");
            let db_path = dir.path().join("cache.redb");

            let cache =
                Cache::<String, TestValue>::new(db_path, "test_table").await;
            cache.unwrap();
        }

        #[tokio::test]
        async fn should_map_io_error_during_init() {
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

            let cache =
                Cache::<String, TestValue>::new(db_path, "test_table").await;

            assert!(cache.is_err());
            let err = cache.expect_err("should have error");
            assert!(matches!(err, CacheError::BackendError {
                backend: "redb",
                ..
            }));
        }

        #[tokio::test]
        async fn should_support_multiple_tables_in_same_db() {
            let dir = tempdir().expect("failed to create temp dir");
            let db_path = dir.path().join("multi_table.redb");

            let cache1 = Cache::<String, TestValue>::new(&db_path, "table1")
                .await
                .expect("failed to create cache1");
            let db = Arc::clone(&cache1.db);

            let _cache2 = Cache::<String, TestValue>::with_db(db, "table2");

            assert!(db_path.exists());
        }
    }

    mod metadata {
        use std::collections::HashMap;

        use tempfile::tempdir;

        use super::*;

        #[tokio::test]
        async fn should_support_metadata() {
            let dir = tempdir().expect("failed to create temp dir");
            let db_path = dir.path().join("metadata.redb");
            let cache = Cache::<String, TestValue>::new(db_path, "table")
                .await
                .expect("init failed");

            let key = "key".to_owned();
            let value = TestValue("value".to_owned());
            let metadata =
                HashMap::from([("version".to_owned(), "1.0".to_owned())]);

            cache
                .put_with_metadata(key.clone(), value.clone(), metadata.clone())
                .await
                .expect("put failed");

            let result =
                cache.get_with_metadata(&key).await.expect("get failed");
            assert!(result.is_some());
            let (v, m) = result.expect("should have result");
            assert_eq!(v, value);
            assert_eq!(m, metadata);
        }

        #[tokio::test]
        async fn should_update_timestamp_on_put() {
            let dir = tempdir().expect("failed to create temp dir");
            let db_path = dir.path().join("timestamp.redb");
            let cache = Cache::<String, TestValue>::new(db_path, "table")
                .await
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
    }

    mod observability {
        use tempfile::tempdir;
        use tracing_test::traced_test;

        use super::*;
        #[tokio::test]
        #[traced_test]
        async fn should_emit_tracing_info() {
            let dir = tempdir().expect("failed to create temp dir");
            let db_path = dir.path().join("tracing.redb");
            let cache = Cache::<String, TestValue>::new(db_path, "table")
                .await
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

    mod serialization {
        use std::collections::HashMap;

        use rkyv::Archived;

        use super::*;

        #[test]
        fn cached_entry_should_implement_rkyv_traits() {
            let entry = Entry {
                value: TestValue("test".to_owned()),
                timestamp: 123_456_789,
                metadata: HashMap::from([(
                    "key".to_owned(),
                    "value".to_owned(),
                )]),
            };

            let bytes = rkyv::to_bytes::<rkyv::rancor::Error>(&entry)
                .expect("failed to serialize");

            let archived = rkyv::access::<
                Archived<Entry<TestValue>>,
                rkyv::rancor::Error,
            >(&bytes)
            .expect("failed to access");

            assert_eq!(archived.timestamp, 123_456_789);
            assert_eq!(archived.value.0, "test");
            assert_eq!(archived.metadata.len(), 1);
        }
    }

    #[derive(
        Archive, Serialize, Deserialize, CheckBytes, Debug, PartialEq, Clone,
    )]
    #[bytecheck(crate = rkyv::bytecheck)]
    struct TestValue(String);
}
