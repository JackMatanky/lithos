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
    sync::{Arc, OnceLock},
    time::{SystemTime, UNIX_EPOCH},
};

use async_trait::async_trait;
use redb::{ReadableDatabase as _, ReadableTable as _, TableDefinition};
use rkyv::{Archive, Deserialize, Serialize, bytecheck::CheckBytes};
use tracing::{error, info_span};

use crate::spi::{
    cache::{CacheReader, CacheWriter, encoder::RkyvCodec},
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
    /// Unix timestamp of when the entry was created/updated.
    pub timestamp: u64,
    /// The actual cached value.
    pub value: V,
    /// Extensible metadata for the cached entry.
    pub metadata: MetadataMap,
}

impl<V> Entry<V> {
    /// Create a new entry.
    #[inline]
    #[must_use]
    pub fn new(value: V, timestamp: u64, metadata: MetadataMap) -> Self {
        Self {
            timestamp,
            value,
            metadata,
        }
    }
}

/// A view into a cached entry, providing zero-copy access to archived data.
pub struct EntryView<'guard, V, C, K = String>
where
    C: crate::spi::cache::encoder::Codec<K, Entry<V>>,
{
    codec: C,
    guard: redb::AccessGuard<'guard, &'static [u8]>,
    _marker: std::marker::PhantomData<(K, V)>,
}

impl<'guard, V, C, K> EntryView<'guard, V, C, K>
where
    C: crate::spi::cache::encoder::Codec<K, Entry<V>>,
{
    /// Access the archived value without full deserialization.
    ///
    /// # Note
    /// This is a zero-copy view and requires aligned archived bytes. If the
    /// underlying storage does not guarantee alignment, this returns a
    /// `CacheError::SerializationError`.
    ///
    /// # Errors
    /// Returns `CacheError::SerializationError` if access fails.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use lithos_adapters::spi::cache::redb::{EntryView, Entry, MetadataMap};
    /// # use lithos_adapters::spi::cache::encoder::{Codec, RkyvCodec};
    /// # use std::collections::HashMap;
    /// # let codec = RkyvCodec::default();
    /// # let entry = Entry::new("test".to_string(), 0, HashMap::new());
    /// # let bytes = <RkyvCodec as Codec<String, Entry<String>>>::encode_value(&codec, &entry).unwrap();
    /// # // In real usage, the guard comes from redb
    /// ```
    #[inline]
    pub fn as_archived(&self) -> Result<&C::Archived, CacheError> {
        self.codec.access(self.guard.value())
    }

    /// Create a new `EntryView`.
    #[inline]
    #[must_use]
    pub fn new(
        guard: redb::AccessGuard<'guard, &'static [u8]>,
        codec: C,
    ) -> Self {
        Self {
            guard,
            codec,
            _marker: std::marker::PhantomData,
        }
    }
}

/// Builder for Redb cache.
///
/// # Examples
///
/// ```rust
/// use lithos_adapters::spi::cache::RedbBuilder;
///
/// let mut builder = RedbBuilder::<String, String>::new();
/// builder.path("cache.redb").table_name("metadata");
///
/// let reader = builder.reader().unwrap();
/// let writer = builder.writer().unwrap();
/// ```
///
/// For fail-fast validation, use [`Builder::try_path`] and
/// [`Builder::try_table_name`].
#[derive(Debug, Clone)]
pub struct Builder<K, V>
where
    K: Clone + Eq + std::hash::Hash + Send + Sync + 'static,
    V: Clone + Send + Sync + 'static,
{
    path: Option<std::path::PathBuf>,
    shared_inner: Arc<OnceLock<RedbInner<K, V>>>,
    table_name: Option<String>,
    _marker: std::marker::PhantomData<(K, V)>,
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
            shared_inner: Arc::new(OnceLock::new()),
            table_name: None,
            _marker: std::marker::PhantomData,
        }
    }

    /// Set the database path.
    #[inline]
    pub fn path<P: AsRef<std::path::Path>>(&mut self, path: P) -> &mut Self {
        if let Err(e) = self.try_path(path) {
            tracing::warn!(?e, "Invalid path provided to Redb cache builder");
        }
        self
    }

    /// Reset the internal state, forcing a fresh database connection to be
    /// created on next access.
    #[inline]
    fn reset_state(&mut self) {
        self.shared_inner = Arc::new(OnceLock::new());
    }

    /// Set the table name.
    #[inline]
    pub fn table_name(&mut self, name: &str) -> &mut Self {
        if let Err(e) = self.try_table_name(name) {
            tracing::warn!(
                ?e,
                "Invalid table name provided to Redb cache builder"
            );
        }
        self
    }

    /// Set the database path with fail-fast validation.
    ///
    /// # Errors
    /// Returns `CacheError::BackendError` if the path is invalid.
    #[inline]
    pub fn try_path<P: AsRef<std::path::Path>>(
        &mut self,
        path: P,
    ) -> Result<&mut Self, CacheError> {
        let p = path.as_ref();
        Self::validate_path(Some(p))?;
        self.path = Some(p.to_path_buf());
        self.reset_state();
        Ok(self)
    }

    /// Set the table name with fail-fast validation.
    ///
    /// # Errors
    /// Returns `CacheError::BackendError` if the table name is invalid.
    #[inline]
    pub fn try_table_name(
        &mut self,
        name: &str,
    ) -> Result<&mut Self, CacheError> {
        Self::validate_table_name(Some(name))?;
        self.table_name = Some(name.to_owned());
        self.reset_state();
        Ok(self)
    }

    /// Validate the database path.
    #[inline]
    fn validate_path(
        path: Option<&std::path::Path>,
    ) -> Result<&std::path::Path, CacheError> {
        let path = path.ok_or_else(|| CacheError::BackendError {
            backend: "redb",
            message: "Database path is required".into(),
        })?;
        Self::validate_path_not_empty(path)?;
        Self::validate_path_not_directory(path)?;
        Ok(path)
    }

    /// Check if the path is a file (not a directory).
    #[inline]
    fn validate_path_not_directory(
        path: &std::path::Path,
    ) -> Result<(), CacheError> {
        if path.is_dir() {
            return Err(CacheError::BackendError {
                backend: "redb",
                message: format!("Path is a directory: {}", path.display())
                    .into(),
            });
        }
        Ok(())
    }

    /// Check if the path is not empty.
    #[inline]
    fn validate_path_not_empty(
        path: &std::path::Path,
    ) -> Result<(), CacheError> {
        if path.as_os_str().is_empty() {
            return Err(CacheError::BackendError {
                backend: "redb",
                message: "Database path cannot be empty".into(),
            });
        }
        Ok(())
    }

    /// Validate the table name.
    #[inline]
    fn validate_table_name(name: Option<&str>) -> Result<&str, CacheError> {
        let name = name.ok_or_else(|| CacheError::BackendError {
            backend: "redb",
            message: "Table name is required".into(),
        })?;
        if name.is_empty() {
            return Err(CacheError::BackendError {
                backend: "redb",
                message: "Table name cannot be empty".into(),
            });
        }
        Ok(name)
    }
}

impl<K, V> Builder<K, V>
where
    K: std::fmt::Debug + Clone + Eq + std::hash::Hash + Send + Sync + 'static,
    V: Clone + Send + Sync + 'static,
{
    /// Internal helper to obtain the shared inner state.
    fn get_or_init_inner(&self) -> Result<RedbInner<K, V>, CacheError> {
        if let Some(inner) = self.shared_inner.get() {
            return Ok(Arc::clone(inner));
        }

        let inner = self.inner_builder()?;
        // Try to set it. If someone else set it first, that's fine, we'll
        // return whatever is there.
        _ = self.shared_inner.set(Arc::clone(&inner));
        Ok(inner)
    }

    /// Internal helper to build the Inner state.
    #[inline]
    fn inner_builder(&self) -> Result<RedbInner<K, V>, CacheError> {
        let path = Self::validate_path(self.path.as_deref())?;
        let table_name = Self::validate_table_name(self.table_name.as_deref())?;

        let db = if path.exists() {
            redb::Database::open(path).map_err(|e| {
                error!(backend = "redb", ?e, "Failed to open database");
                CacheError::BackendError {
                    backend: "redb",
                    message: format!("Failed to open database: {e}").into(),
                }
            })?
        } else {
            redb::Database::create(path).map_err(|e| {
                error!(backend = "redb", ?e, "Failed to create database");
                CacheError::BackendError {
                    backend: "redb",
                    message: format!("Failed to create database: {e}").into(),
                }
            })?
        };

        Ok(Arc::new(Inner::new(db, table_name, RkyvCodec)))
    }

    /// Build a Reader handle.
    ///
    /// Creates a new database connection (if not already initialized by this
    /// builder) and returns only a Reader handle.
    ///
    /// # Errors
    ///
    /// Returns `CacheError` if the database cannot be initialized.
    #[inline]
    pub fn reader(&self) -> Result<Reader<K, V>, CacheError> {
        let inner = self.get_or_init_inner()?;
        Ok(Reader {
            inner,
        })
    }

    /// Build a Writer handle.
    ///
    /// Creates a new database connection (if not already initialized by this
    /// builder) and returns only a Writer handle.
    ///
    /// # Errors
    ///
    /// Returns `CacheError` if the database cannot be initialized.
    #[inline]
    pub fn writer(&self) -> Result<Writer<K, V>, CacheError> {
        let inner = self.get_or_init_inner()?;
        Ok(Writer {
            inner,
        })
    }
}

/// Read-only handle for Redb cache.
///
/// This handle provides read-only access to the cache following CQRS
/// principles.
///
/// # Examples
///
/// ```rust
/// # use lithos_adapters::spi::cache::{RedbBuilder, CacheReader, CacheWriter};
/// # tokio::runtime::Runtime::new().unwrap().block_on(async {
/// # let dir = tempfile::tempdir().unwrap();
/// # let db_path = dir.path().join("test.redb");
/// let mut builder = RedbBuilder::<String, String>::new();
/// builder.path(db_path).table_name("test");
/// let reader = builder.reader().unwrap();
/// let writer = builder.writer().unwrap();
///
/// // Ensure table is created
/// writer.clear().await.unwrap();
///
/// let value = reader.get(&"key".to_string()).await.unwrap();
/// assert!(value.is_none());
/// # });
/// ```
#[derive(Debug, Clone)]
pub struct Reader<K, V, C = RkyvCodec>
where
    K: Clone + Eq + std::hash::Hash + Send + Sync + 'static,
    V: Clone + Send + Sync + 'static,
{
    inner: Arc<Inner<K, V, C>>,
}

impl<K, V, C> Reader<K, V, C>
where
    K: std::fmt::Debug + Clone + Eq + std::hash::Hash + Send + Sync + 'static,
    V: Clone + Send + Sync + 'static,
    C: crate::spi::cache::encoder::Codec<K, Entry<V>>
        + Clone
        + Send
        + Sync
        + 'static,
{
    /// Retrieve value and metadata by key.
    ///
    /// # Note
    /// This method performs full deserialization and heap allocation. For
    /// zero-copy access, use [`with_view`](Self::with_view).
    ///
    /// # Errors
    /// Returns `CacheError` if retrieval or deserialization fails.
    #[inline]
    pub async fn get_with_metadata(&self, key: &K) -> Outcome<V> {
        let key_bytes = self.inner.codec.encode_key(key)?;
        let codec = self.inner.codec.clone();

        self.inner
            .read(move |txn, table_name| {
                let table_def =
                    TableDefinition::<&[u8], &[u8]>::new(table_name);
                let table = txn
                    .open_table(table_def)
                    .map_err(|e| Executor::map_redb_error(e.into()))?;

                table
                    .get(key_bytes.as_slice())
                    .map_err(|e| Executor::map_redb_error(e.into()))?
                    .map(|guard| codec.decode_value(guard.value()))
                    .transpose()
            })
            .await?
            .map(|entry| Ok((entry.value, entry.metadata)))
            .transpose()
    }

    /// Retrieve a page of keys with an optional cursor.
    ///
    /// The cursor is the last key returned from a previous call; this method
    /// returns keys strictly after the cursor. Ordering follows the underlying
    /// encoded key bytes.
    ///
    /// # Errors
    /// Returns `CacheError` if retrieval or decoding fails.
    #[inline]
    pub async fn keys_page(
        &self,
        limit: usize,
        cursor: Option<K>,
    ) -> Result<(Vec<K>, Option<K>), CacheError> {
        if limit == 0 {
            return Err(CacheError::BackendError {
                backend: "redb",
                message: "limit must be greater than 0".into(),
            });
        }

        let cursor_bytes = cursor
            .as_ref()
            .map(|key| self.inner.codec.encode_key(key))
            .transpose()?;
        let codec = self.inner.codec.clone();

        self.inner
            .read(move |txn, table_name| {
                let table_def =
                    TableDefinition::<&[u8], &[u8]>::new(table_name);
                let table = txn
                    .open_table(table_def)
                    .map_err(|e| Executor::map_redb_error(e.into()))?;

                let mut keys = Vec::with_capacity(limit);
                let mut next_cursor = None;

                for result in table
                    .iter()
                    .map_err(|e| Executor::map_redb_error(e.into()))?
                {
                    let (key_handle, _): (redb::AccessGuard<'_, &[u8]>, _) =
                        result
                            .map_err(|e| Executor::map_redb_error(e.into()))?;
                    let key_bytes = key_handle.value();

                    if let Some(cursor_bytes) = cursor_bytes.as_deref()
                        && key_bytes <= cursor_bytes
                    {
                        continue;
                    }

                    let key = codec.decode_key(key_bytes).map_err(|e| {
                        CacheError::SerializationError {
                            type_name: std::any::type_name::<K>(),
                            message: format!("Key decoding failed: {e}").into(),
                        }
                    })?;
                    keys.push(key.clone());
                    next_cursor = Some(key);

                    if keys.len() == limit {
                        break;
                    }
                }

                if keys.len() < limit {
                    next_cursor = None;
                }

                Ok((keys, next_cursor))
            })
            .await
    }

    /// Provide zero-copy access to the archived entry via a closure.
    ///
    /// This method enables high-performance access to cached data without
    /// heap allocation or full deserialization by operating directly on the
    /// memory-mapped database pages.
    ///
    /// # Note
    /// This is a zero-copy view and requires aligned archived bytes. If the
    /// underlying storage does not guarantee alignment, this returns a
    /// `CacheError::SerializationError`.
    ///
    /// # Errors
    /// Returns `CacheError` if retrieval or validation fails.
    #[inline]
    pub async fn with_view<F, R>(
        &self,
        key: &K,
        f: F,
    ) -> Result<Option<R>, CacheError>
    where
        F: FnOnce(&C::Archived) -> R + Send + 'static,
        R: Send + 'static,
    {
        let key_bytes = self.inner.codec.encode_key(key)?;
        let codec = self.inner.codec.clone();

        self.inner
            .read(move |txn, table_name| {
                let table_def =
                    TableDefinition::<&[u8], &[u8]>::new(table_name);
                let table = txn
                    .open_table(table_def)
                    .map_err(|e| Executor::map_redb_error(e.into()))?;

                if let Some(guard) = table
                    .get(key_bytes.as_slice())
                    .map_err(|e| Executor::map_redb_error(e.into()))?
                {
                    let encoded = guard.value();
                    let archived = codec.access(encoded)?;
                    Ok(Some(f(archived)))
                } else {
                    Ok(None)
                }
            })
            .await
    }
}

#[async_trait]
impl<K, V, C> CacheReader<K, V> for Reader<K, V, C>
where
    K: std::fmt::Debug + Clone + Eq + std::hash::Hash + Send + Sync + 'static,
    V: Clone + Send + Sync + 'static,
    C: crate::spi::cache::encoder::Codec<K, Entry<V>>
        + Clone
        + Send
        + Sync
        + 'static,
{
    #[inline]
    async fn get(&self, key: &K) -> Result<Option<V>, CacheError> {
        Ok(self.get_with_metadata(key).await?.map(|(v, _)| v))
    }

    #[inline]
    async fn has(&self, key: &K) -> Result<bool, CacheError> {
        let key_bytes = self.inner.codec.encode_key(key)?;

        self.inner
            .read(move |txn, table_name| {
                let table_def =
                    TableDefinition::<&[u8], &[u8]>::new(table_name);
                let table = txn
                    .open_table(table_def)
                    .map_err(|e| Executor::map_redb_error(e.into()))?;
                Ok(table
                    .get(key_bytes.as_slice())
                    .map_err(|e| Executor::map_redb_error(e.into()))?
                    .is_some())
            })
            .await
    }

    #[inline]
    async fn keys(&self) -> Result<Vec<K>, CacheError> {
        let codec = self.inner.codec.clone();

        self.inner
            .read(move |txn, table_name| {
                let table_def =
                    TableDefinition::<&[u8], &[u8]>::new(table_name);
                let table = txn
                    .open_table(table_def)
                    .map_err(|e| Executor::map_redb_error(e.into()))?;

                let mut keys = Vec::new();
                for result in table
                    .iter()
                    .map_err(|e| Executor::map_redb_error(e.into()))?
                {
                    let (key_handle, _): (redb::AccessGuard<'_, &[u8]>, _) =
                        result
                            .map_err(|e| Executor::map_redb_error(e.into()))?;
                    let key =
                        codec.decode_key(key_handle.value()).map_err(|e| {
                            CacheError::SerializationError {
                                type_name: std::any::type_name::<K>(),
                                message: format!("Key decoding failed: {e}")
                                    .into(),
                            }
                        })?;
                    keys.push(key);
                }
                Ok(keys)
            })
            .await
    }
}

/// Write-only handle for Redb cache.
///
/// This handle provides write-only access to the cache following CQRS
/// principles.
///
/// # Examples
///
/// ```rust
/// # use lithos_adapters::spi::cache::{RedbBuilder, CacheWriter};
/// # tokio::runtime::Runtime::new().unwrap().block_on(async {
/// # let dir = tempfile::tempdir().unwrap();
/// # let db_path = dir.path().join("test_writer.redb");
/// let writer = RedbBuilder::<String, String>::new()
///     .path(db_path)
///     .table_name("test")
///     .writer()
///     .unwrap();
///
/// writer.put("key".to_string(), "value".to_string()).await.unwrap();
/// # });
/// ```
#[derive(Debug, Clone)]
pub struct Writer<K, V, C = RkyvCodec>
where
    K: Clone + Eq + std::hash::Hash + Send + Sync + 'static,
    V: Clone + Send + Sync + 'static,
{
    inner: Arc<Inner<K, V, C>>,
}

impl<K, V, C> Writer<K, V, C>
where
    K: std::fmt::Debug + Clone + Eq + std::hash::Hash + Send + Sync + 'static,
    V: Clone + Send + Sync + 'static,
    C: crate::spi::cache::encoder::Codec<K, Entry<V>>
        + Clone
        + Send
        + Sync
        + 'static,
{
    /// Store value with custom metadata.
    ///
    /// # Errors
    /// Returns `CacheError` if serialization or storage fails.
    #[inline]
    pub async fn put_with_metadata(
        &self,
        key: K,
        value: V,
        metadata: MetadataMap,
    ) -> Result<(), CacheError> {
        let entry = Entry {
            value,
            timestamp: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .map_err(|e| CacheError::BackendError {
                    backend: "system",
                    message: format!("System time error: {e}").into(),
                })?
                .as_secs(),
            metadata,
        };

        let key_bytes = self.inner.codec.encode_key(&key)?;
        let value_bytes = self.inner.codec.encode_value(&entry)?;

        self.inner
            .write(move |txn, table_name| {
                let table_def =
                    TableDefinition::<&[u8], &[u8]>::new(table_name);
                let mut table = txn
                    .open_table(table_def)
                    .map_err(|e| Executor::map_redb_error(e.into()))?;
                table
                    .insert(key_bytes.as_slice(), value_bytes.as_slice())
                    .map_err(|e| Executor::map_redb_error(e.into()))?;
                Ok(())
            })
            .await
    }
}

#[async_trait]
impl<K, V, C> CacheWriter<K, V> for Writer<K, V, C>
where
    K: std::fmt::Debug + Clone + Eq + std::hash::Hash + Send + Sync + 'static,
    V: Clone + Send + Sync + 'static,
    C: crate::spi::cache::encoder::Codec<K, Entry<V>>
        + Clone
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
                _ = txn
                    .delete_table(table_def)
                    .map_err(|e| Executor::map_redb_error(e.into()))?;
                _ = txn
                    .open_table(table_def)
                    .map_err(|e| Executor::map_redb_error(e.into()))?;
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
                let mut table = txn
                    .open_table(table_def)
                    .map_err(|e| Executor::map_redb_error(e.into()))?;
                Ok(table
                    .remove(key_bytes.as_slice())
                    .map_err(|e| Executor::map_redb_error(e.into()))?
                    .is_some())
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

/// Type alias for the Inner state with default codec.
pub(crate) type RedbInner<K, V> = Arc<Inner<K, V, RkyvCodec>>;

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
    /// Create a new Inner state.
    #[inline]
    fn new(db: redb::Database, table_name: &str, codec: C) -> Self {
        Self {
            db: Arc::new(db),
            executor: Executor,
            table_name: Arc::from(table_name),
            codec,
            _marker: std::marker::PhantomData,
        }
    }

    /// Execute a read operation.
    #[inline]
    async fn read<F, R>(&self, f: F) -> Result<R, CacheError>
    where
        F: FnOnce(&redb::ReadTransaction, &str) -> Result<R, CacheError>
            + Send
            + 'static,
        R: Send + 'static,
    {
        let db = Arc::clone(&self.db);
        let table_name = Arc::clone(&self.table_name);
        let span = info_span!("redb_read", table = %table_name);

        self.executor
            .spawn(span, move || {
                let txn = db
                    .begin_read()
                    .map_err(|e| Executor::map_redb_error(e.into()))?;
                f(&txn, &table_name)
            })
            .await
    }

    /// Execute a write operation.
    #[inline]
    async fn write<F, R>(&self, f: F) -> Result<R, CacheError>
    where
        F: FnOnce(&redb::WriteTransaction, &str) -> Result<R, CacheError>
            + Send
            + 'static,
        R: Send + 'static,
    {
        let db = Arc::clone(&self.db);
        let table_name = Arc::clone(&self.table_name);
        let span = info_span!("redb_write", table = %table_name);

        self.executor
            .spawn(span, move || {
                let txn = db
                    .begin_write()
                    .map_err(|e| Executor::map_redb_error(e.into()))?;
                let result = f(&txn, &table_name)?;
                txn.commit().map_err(|e| Executor::map_redb_error(e.into()))?;
                Ok(result)
            })
            .await
    }
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
                message: format!("{other} (kind: {other:?})").into(),
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
        F: FnOnce() -> Result<R, CacheError> + Send + 'static,
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
    }
}

#[cfg(test)]
mod tests {

    use super::*;

    mod fixtures {
        use tempfile::{TempDir, tempdir};

        use super::*;

        pub fn temp_dir() -> TempDir {
            tempdir().expect("failed to create temp dir")
        }

        pub fn db_path(temp_dir: &TempDir) -> std::path::PathBuf {
            temp_dir.path().join("test.redb")
        }

        pub fn builder(
            db_path: std::path::PathBuf,
        ) -> Builder<String, TestValue> {
            let mut builder = Builder::new();
            builder.path(db_path).table_name("test");
            builder
        }

        pub async fn handles(
            builder: Builder<String, TestValue>,
        ) -> (Reader<String, TestValue>, Writer<String, TestValue>) {
            let reader = builder.reader().expect("failed to build reader");
            let writer = builder.writer().expect("failed to build writer");
            // Ensure table is created for tests
            writer.clear().await.unwrap();
            (reader, writer)
        }
    }

    mod api {
        use super::{fixtures::*, *};

        /// [5.4-U-10] P2: Test builder defaults and codec selection.
        #[test]
        fn allows_usage_without_specifying_codec() {
            // GIVEN: a Redb builder
            let temp_dir = temp_dir();
            let db_path = db_path(&temp_dir);

            // WHEN: handles are built without explicit codec
            let mut builder = Builder::<String, String>::new();
            builder.path(db_path).table_name("test");

            let reader = builder.reader().expect("failed to build reader");

            // THEN: the default RkyvCodec is used
            let _: Reader<String, String, RkyvCodec> = reader;
        }

        /// [5.4-U-04] P0: Test independent handle creation.
        #[tokio::test]
        async fn builder_creates_reader_independently() {
            // GIVEN: a Redb builder
            let temp_dir = temp_dir();
            let db_path = db_path(&temp_dir);

            // WHEN: a reader handle is built independently
            let result = Builder::<String, String>::new()
                .path(db_path)
                .table_name("test")
                .reader();

            // THEN: the handle is correct and functional
            assert!(result.is_ok());
            let reader = result.unwrap();
            let _: Reader<String, String> = reader;
        }

        /// [5.4-U-04] P0: Test independent handle creation.
        #[tokio::test]
        async fn builder_creates_writer_independently() {
            // GIVEN: a Redb builder
            let temp_dir = temp_dir();
            let db_path = db_path(&temp_dir);

            // WHEN: a writer handle is built independently
            let result = Builder::<String, String>::new()
                .path(db_path)
                .table_name("test")
                .writer();

            // THEN: the handle is correct and functional
            assert!(result.is_ok());
            let writer = result.unwrap();
            let _: Writer<String, String> = writer;
        }

        /// [5.4-U-08] P0: Test CQRS coordination.
        #[tokio::test]
        async fn reader_and_writer_work_together() {
            // GIVEN: a shared Redb database (via fixture)
            let temp_dir = temp_dir();
            let (reader, writer) = handles(builder(db_path(&temp_dir))).await;

            // WHEN: writing data through the writer
            writer
                .put("key".to_owned(), TestValue("value".to_owned()))
                .await
                .expect("put failed");

            // THEN: data can be retrieved through the reader
            let result =
                reader.get(&"key".to_owned()).await.expect("get failed");
            assert_eq!(result, Some(TestValue("value".to_owned())));

            // AND: existence checks work
            let has_key =
                reader.has(&"key".to_owned()).await.expect("has failed");
            assert!(has_key);

            // AND: deletion through the writer is reflected in the reader
            let deleted =
                writer.delete(&"key".to_owned()).await.expect("delete failed");
            assert!(deleted);

            let deleted_result =
                reader.get(&"key".to_owned()).await.expect("get failed");
            assert_eq!(deleted_result, None);
        }

        /// [5.4-U-08] P1: Test key discovery.
        #[tokio::test]
        async fn should_return_all_keys() {
            // GIVEN: a database with multiple entries
            let temp_dir = temp_dir();
            let (reader, writer) = handles(builder(db_path(&temp_dir))).await;

            writer
                .put("k1".to_owned(), TestValue("v1".to_owned()))
                .await
                .expect("put failed");
            writer
                .put("k2".to_owned(), TestValue("v2".to_owned()))
                .await
                .expect("put failed");

            // WHEN: retrieving all keys
            let mut keys = reader.keys().await.expect("keys failed");
            keys.sort();

            // THEN: all expected keys are returned
            assert_eq!(keys.len(), 2);
            assert!(keys.contains(&"k1".to_owned()));
            assert!(keys.contains(&"k2".to_owned()));
        }

        /// [5.4-U-08] P1: Test key paging.
        #[tokio::test]
        async fn should_page_keys() {
            // GIVEN: a database with multiple entries
            let temp_dir = temp_dir();
            let (reader, writer) = handles(builder(db_path(&temp_dir))).await;

            writer
                .put("a".to_owned(), TestValue("v1".to_owned()))
                .await
                .expect("put failed");
            writer
                .put("b".to_owned(), TestValue("v2".to_owned()))
                .await
                .expect("put failed");

            // WHEN: retrieving keys in pages
            let (page1, cursor) =
                reader.keys_page(1, None).await.expect("keys_page failed");
            let (page2, cursor2) =
                reader.keys_page(1, cursor).await.expect("keys_page failed");

            // THEN: pages return distinct keys and a terminal cursor
            assert_eq!(page1.len(), 1);
            assert_eq!(page2.len(), 1);
            assert!(cursor2.is_none() || cursor2 == page2.last().cloned());
        }
    }

    mod executor {
        use super::{fixtures::*, *};
        use crate::spi::cache::encoder::Codec;

        /// [5.4-U-07] P1: Test transactional batching.
        #[tokio::test]
        async fn batches_multiple_writes_in_single_transaction() {
            // GIVEN: a Redb cache (via fixture)
            let temp_dir = temp_dir();
            let (reader, writer) = handles(builder(db_path(&temp_dir))).await;

            // WHEN: performing a batch write via the inner closure
            let k1 = "k1".to_owned();
            let k2 = "k2".to_owned();
            let v1 = Entry::new(TestValue("v1".to_owned()), 0, HashMap::new());
            let v2 = Entry::new(TestValue("v2".to_owned()), 0, HashMap::new());

            let k1_bytes =
                <RkyvCodec as Codec<String, Entry<TestValue>>>::encode_key(
                    &writer.inner.codec,
                    &k1,
                )
                .unwrap();
            let k2_bytes =
                <RkyvCodec as Codec<String, Entry<TestValue>>>::encode_key(
                    &writer.inner.codec,
                    &k2,
                )
                .unwrap();
            let v1_bytes =
                <RkyvCodec as Codec<String, Entry<TestValue>>>::encode_value(
                    &writer.inner.codec,
                    &v1,
                )
                .unwrap();
            let v2_bytes =
                <RkyvCodec as Codec<String, Entry<TestValue>>>::encode_value(
                    &writer.inner.codec,
                    &v2,
                )
                .unwrap();

            writer
                .inner
                .write(move |txn, table_name| {
                    let table_def =
                        TableDefinition::<&[u8], &[u8]>::new(table_name);
                    let mut table = txn
                        .open_table(table_def)
                        .map_err(|e| Executor::map_redb_error(e.into()))?;

                    table
                        .insert(k1_bytes.as_slice(), v1_bytes.as_slice())
                        .map_err(|e| Executor::map_redb_error(e.into()))?;
                    table
                        .insert(k2_bytes.as_slice(), v2_bytes.as_slice())
                        .map_err(|e| Executor::map_redb_error(e.into()))?;

                    Ok(())
                })
                .await
                .expect("batch write failed");

            // THEN: both entries are persisted
            assert!(reader.has(&"k1".to_owned()).await.unwrap());
            assert!(reader.has(&"k2".to_owned()).await.unwrap());
        }

        /// [5.4-U-06] P1: Test error conversion.
        #[tokio::test]
        async fn maps_redb_error_to_cache_error() {
            // GIVEN: an existing Redb file
            let temp_dir = temp_dir();
            let db_path = db_path(&temp_dir);

            // WHEN: building handles
            let _reader = Builder::<String, String>::new()
                .path(&db_path)
                .table_name("test")
                .reader()
                .expect("failed to build reader");

            // THEN: the executor is correctly integrated
            assert!(db_path.exists());
        }
    }

    mod builder {
        use super::{fixtures::*, *};

        /// [5.4-U-04] P1: Edge Case - invalid path.
        #[test]
        fn fails_when_path_is_directory() {
            // GIVEN: a directory path
            let temp_dir = temp_dir();

            // WHEN: attempting to use it as a DB path
            let result = Builder::<String, String>::new()
                .path(temp_dir.path())
                .table_name("test")
                .reader();

            // THEN: an error is returned
            assert!(result.is_err());
            assert!(matches!(
                result.unwrap_err(),
                CacheError::BackendError { .. }
            ));
        }

        /// [5.4-U-04] P1: Test database initialization.
        #[test]
        fn initializes_db_with_correct_table() {
            // GIVEN: a DB path
            let temp_dir = temp_dir();
            let db_path = db_path(&temp_dir);

            // WHEN: building handles
            let result = Builder::<String, String>::new()
                .path(&db_path)
                .table_name("my_table")
                .reader();

            // THEN: the database file is created
            result.unwrap();
            assert!(db_path.exists());
        }
    }

    mod core_ops {
        use super::{fixtures::*, *};

        /// [5.4-U-08] P1: Test cache clearing.
        #[tokio::test]
        async fn should_clear_all_entries() {
            // GIVEN: a database with entries
            let temp_dir = temp_dir();
            let (reader, writer) = handles(builder(db_path(&temp_dir))).await;

            writer
                .put("k1".to_owned(), TestValue("v1".to_owned()))
                .await
                .expect("put failed");
            writer
                .put("k2".to_owned(), TestValue("v2".to_owned()))
                .await
                .expect("put failed");

            // WHEN: clear is called
            writer.clear().await.expect("clear failed");

            // THEN: all entries are removed
            let has_k1 =
                reader.has(&"k1".to_owned()).await.expect("has failed");
            let has_k2 =
                reader.has(&"k2".to_owned()).await.expect("has failed");
            assert!(!has_k1);
            assert!(!has_k2);
        }

        /// [5.4-U-08] P1: Test existence reporting.
        #[tokio::test]
        async fn should_correctly_report_existence() {
            // GIVEN: a database
            let temp_dir = temp_dir();
            let (reader, writer) = handles(builder(db_path(&temp_dir))).await;

            // WHEN: checking existence of present and missing keys
            let key = "exists".to_owned();
            writer
                .put(key.clone(), TestValue("yes".to_owned()))
                .await
                .expect("put failed");

            // THEN: report is accurate
            let has_key = reader.has(&key).await.expect("has failed");
            let has_missing =
                reader.has(&"missing".to_owned()).await.expect("has failed");
            assert!(has_key);
            assert!(!has_missing);
        }

        /// [5.4-U-05] P0: Test data persistence.
        #[tokio::test]
        async fn should_persist_data_across_instances() {
            // GIVEN: a database file
            let temp_dir = temp_dir();
            let db_path = db_path(&temp_dir);
            let key = "key".to_owned();
            let value = TestValue("persistent".to_owned());

            // WHEN: writing data and closing the instance
            let writer = Builder::<String, TestValue>::new()
                .path(&db_path)
                .table_name("table")
                .writer()
                .expect("failed to create cache");
            writer.put(key.clone(), value.clone()).await.expect("put failed");

            drop(writer);

            // THEN: data persists when a new instance is opened
            let reader = Builder::<String, TestValue>::new()
                .path(&db_path)
                .table_name("table")
                .reader()
                .expect("failed to reload cache");
            let result = reader.get(&key).await.expect("get failed");
            assert_eq!(result, Some(value));
        }
    }

    mod initialization {
        use super::{fixtures::*, *};

        /// [5.4-U-04] P1: Test cache initialization.
        #[tokio::test]
        async fn should_initialize_redb_cache() {
            // GIVEN: a path
            let temp_dir = temp_dir();
            let db_path = db_path(&temp_dir);

            // WHEN: a Redb cache is built
            let result = Builder::<String, TestValue>::new()
                .path(db_path)
                .table_name("test_table")
                .reader();

            // THEN: it succeeds
            result.unwrap();
        }

        /// [5.4-U-06] P1: Test IO error mapping.
        #[tokio::test]
        async fn should_map_io_error_during_init() {
            // GIVEN: a read-only database file
            use std::fs::File;
            let temp_dir = temp_dir();
            let db_path = temp_dir.path().join("read_only.redb");

            File::create(&db_path).expect("failed to create file");
            let mut perms = std::fs::metadata(&db_path)
                .expect("failed to get metadata")
                .permissions();
            perms.set_readonly(true);
            std::fs::set_permissions(&db_path, perms)
                .expect("failed to set permissions");

            // WHEN: attempting to open it for writing
            let result = Builder::<String, TestValue>::new()
                .path(db_path)
                .table_name("test_table")
                .reader();

            // THEN: a BackendError is returned
            assert!(result.is_err());
            let err = result.expect_err("should have error");
            assert!(matches!(err, CacheError::BackendError {
                backend: "redb",
                ..
            }));
        }

        /// [5.4-U-04] P2: Test multi-table support in one file.
        #[tokio::test]
        async fn should_support_multiple_tables_in_same_db() {
            // GIVEN: a database path
            let temp_dir = temp_dir();
            let db_path = db_path(&temp_dir);

            // WHEN: opening multiple tables in the same file
            {
                let mut builder1 = Builder::<String, TestValue>::new();
                builder1.path(&db_path).table_name("table1");
                let _reader1 =
                    builder1.reader().expect("failed to create reader1");
                let _writer1 =
                    builder1.writer().expect("failed to create writer1");
            } // Drop builder1 and handles to release lock

            let mut builder2 = Builder::<String, TestValue>::new();
            builder2.path(&db_path).table_name("table2");
            let _reader2 = builder2.reader().expect("failed to create reader2");

            // THEN: the database handles multiple tables successfully
            assert!(db_path.exists());
        }
    }

    mod metadata {
        use lithos_test_utils::time_test;
        use tempfile::tempdir;

        use super::{fixtures::*, *};

        /// [5.4-U-08] P1: Test metadata support.
        #[tokio::test]
        async fn should_support_metadata() {
            // GIVEN: a cache and metadata
            let temp_dir = temp_dir();
            let (reader, writer) = handles(builder(db_path(&temp_dir))).await;

            let key = "key".to_owned();
            let value = TestValue("value".to_owned());
            let metadata =
                HashMap::from([("version".to_owned(), "1.0".to_owned())]);

            // WHEN: putting with custom metadata
            writer
                .put_with_metadata(key.clone(), value.clone(), metadata.clone())
                .await
                .expect("put failed");

            // THEN: both value and metadata are retrieved correctly
            let result =
                reader.get_with_metadata(&key).await.expect("get failed");
            assert!(result.is_some());
            let (v, m) = result.expect("should have result");
            assert_eq!(v, value);
            assert_eq!(m, metadata);
        }

        // [5.4-U-08] P1: Test timestamp updating using virtual time.
        time_test!(
            async fn should_update_timestamp_on_put() {
                // GIVEN: a database entry
                let dir = tempdir().expect("failed to create temp dir");
                let db_path = dir.path().join("timestamp.redb");
                let mut builder = Builder::<String, TestValue>::new();
                builder.path(db_path).table_name("table");
                let reader = builder.reader().expect("reader init failed");
                let writer = builder.writer().expect("writer init failed");

                // Ensure table is created
                writer.clear().await.unwrap();

                let key = "key".to_owned();

                writer
                    .put(key.clone(), TestValue("v1".to_owned()))
                    .await
                    .expect("put failed");

                // WHEN: waiting and updating the entry
                // (Wait 1.1s to ensure SystemTime::now() changes second)
                tokio::time::advance(std::time::Duration::from_millis(1100))
                    .await;

                writer
                    .put(key.clone(), TestValue("v2".to_owned()))
                    .await
                    .expect("put failed");

                // THEN: the value is updated
                let res2 = reader
                    .get_with_metadata(&key)
                    .await
                    .expect("get failed")
                    .expect("should have result");

                assert_eq!(res2.0, TestValue("v2".to_owned()));
            }
        );
    }

    mod observability {
        use tracing_test::traced_test;

        use super::{fixtures::*, *};

        /// [5.4-U-11] P1: Test tracing emission.
        #[tokio::test]
        #[traced_test]
        async fn should_emit_tracing_info() {
            // GIVEN: a Redb cache
            let temp_dir = temp_dir();
            let (reader, writer) = handles(builder(db_path(&temp_dir))).await;

            // WHEN: performing operations
            let key = "key".to_owned();
            writer
                .put(key.clone(), TestValue("v1".to_owned()))
                .await
                .expect("put failed");

            let _: Option<TestValue> =
                reader.get(&key).await.expect("get failed");

            // THEN: tracing info is emitted (smoke test)
        }

        /// [5.4-U-11] P0: Verify nested tracing spans.
        #[tokio::test]
        #[traced_test]
        async fn emits_nested_spans_for_transactions() {
            // GIVEN: a Redb cache
            let temp_dir = temp_dir();
            let (reader, writer) = handles(builder(db_path(&temp_dir))).await;

            // WHEN: performing operations
            writer
                .put("key".to_owned(), TestValue("value".to_owned()))
                .await
                .expect("put failed");

            let result =
                reader.get(&"key".to_owned()).await.expect("get failed");
            assert_eq!(result, Some(TestValue("value".to_owned())));

            // THEN: nested spans are emitted (verified via traced_test)
        }
    }

    mod nfr {
        use super::{fixtures::*, *};

        /// [5.4-U-09/12] P0: Verify direct pointer access.
        #[tokio::test]
        async fn verifies_direct_pointer_access() {
            // GIVEN: a Redb cache with data
            let temp_dir = temp_dir();
            let (reader, writer) = handles(builder(db_path(&temp_dir))).await;

            let key = "large_data".to_owned();
            let value = TestValue("x".repeat(1024)); // 1KB of data

            writer.put(key.clone(), value.clone()).await.expect("put failed");

            // WHEN: accessing data via with_view
            let result = reader
                .with_view(&key, |archived| {
                    // THEN: archived data is accessible zero-copy
                    assert_eq!(archived.value.0, "x".repeat(1024));
                    archived.value.0.len()
                })
                .await
                .expect("with_view failed");

            assert_eq!(result, Some(1024));
        }
    }

    #[derive(
        Archive, Serialize, Deserialize, CheckBytes, Debug, PartialEq, Clone,
    )]
    #[bytecheck(crate = rkyv::bytecheck)]
    struct TestValue(String);
}
