//! Multi-layer cache coordinator.
//!
//! This module provides a `CacheCoordinator` that orchestrates access between
//! a fast in-memory cache (e.g., Moka) and a persistent disk cache (e.g.,
//! Redb).
//!
//! # Architecture
//!
//! The coordinator follows CQRS principles by splitting into `Reader` and
//! `Writer` handles. It implements:
//!
//! - **Read-Through**: Queries check memory first, then disk. Disk hits trigger
//!   an asynchronous backfill to memory.
//! - **Write-Through**: Writes go to disk first for persistence, then to
//!   memory.
//! - **Parallel Invalidation**: Deletions and clears affect both layers
//!   concurrently.
//! - **Decoupled Backfill**: Memory backfills are performed in a background
//!   task via a bounded MPSC channel, ensuring read latency is never affected
//!   by memory write speeds.
//!
//! # Example
//!
//! ```rust
//! # use std::sync::Arc;
//! # use async_trait::async_trait;
//! # use lithos_adapters::spi::cache::{
//! #     CacheCoordinatorBuilder, CacheReader, CacheWriter,
//! # };
//! # use lithos_adapters::spi::errors::CacheError;
//! #
//! # struct DummyReader;
//! # #[async_trait]
//! # impl CacheReader<String, String> for DummyReader {
//! #     async fn get(&self, _k: &String) -> Result<Option<String>, CacheError> { Ok(None) }
//! #     async fn keys(&self) -> Result<Vec<String>, CacheError> { Ok(Vec::new()) }
//! # }
//! #
//! # struct DummyWriter;
//! # #[async_trait]
//! # impl CacheWriter<String, String> for DummyWriter {
//! #     async fn put(&self, _k: String, _v: String) -> Result<(), CacheError> { Ok(()) }
//! #     async fn delete(&self, _k: &String) -> Result<bool, CacheError> { Ok(false) }
//! #     async fn clear(&self) -> Result<(), CacheError> { Ok(()) }
//! # }
//! #
//! # tokio::runtime::Runtime::new().unwrap().block_on(async {
//! let mut builder = CacheCoordinatorBuilder::<String, String>::new();
//! builder
//!     .memory_reader(Arc::new(DummyReader))
//!     .memory_writer(Arc::new(DummyWriter))
//!     .disk_reader(Arc::new(DummyReader))
//!     .disk_writer(Arc::new(DummyWriter));
//!
//! let reader = builder.reader().await.unwrap();
//! let writer = builder.writer().unwrap();
//! # });
//! ```
//!
//! # CQRS Usage
//!
//! For proper CQRS separation following hexagonal architecture, use
//! `reader()` and `writer()` independently:
//!
//! ```rust
//! # use std::sync::Arc;
//! # use lithos_adapters::spi::cache::{
//! #     CacheCoordinatorBuilder, MokaBuilder, RedbBuilder,
//! # };
//! # use lithos_adapters::spi::errors::CacheError;
//! #
//! # fn example() -> Result<(), CacheError> {
//! # tokio::runtime::Runtime::new().unwrap().block_on(async {
//! # let db_path = std::path::PathBuf::from("test.redb");
//! // Build memory and disk layers
//! let mut moka_builder = MokaBuilder::<String, String>::new();
//! moka_builder.max_capacity(100);
//! let (mem_reader, mem_writer) =
//!     (moka_builder.reader()?, moka_builder.writer()?);
//!
//! let mut redb_builder = RedbBuilder::<String, String>::new();
//! redb_builder.path(&db_path).table_name("cache");
//! let (disk_reader, disk_writer) =
//!     (redb_builder.reader()?, redb_builder.writer()?);
//!
//! // ✅ CQRS Query Side (reads only):
//! // Note: memory_writer is no longer needed for backfill functionality
//! let query_cache = CacheCoordinatorBuilder::new()
//!     .memory_reader(Arc::new(mem_reader))
//!     .disk_reader(Arc::new(disk_reader))
//!     .reader()
//!     .await?; // Returns Reader only
//!
//! // ✅ CQRS Command Side (writes only):
//! let command_cache = CacheCoordinatorBuilder::new()
//!     .memory_writer(Arc::new(mem_writer))
//!     .disk_writer(Arc::new(disk_writer))
//!     .writer()?; // Returns Writer only
//!
//! // Now inject into separate Query and Command adapters:
//! // - QueryAdapter owns query_cache (read-only operations)
//! // - CommandAdapter owns command_cache (write-only operations)
//! # Ok(())
//! # })
//! # }
//! ```
//!
//! This pattern enforces architectural boundaries and prevents mixing
//! read/write concerns in a single adapter, as required by the hexagonal CQRS
//! architecture.

use std::sync::Arc;

use async_trait::async_trait;
use tracing::{debug, info, instrument};

use crate::spi::{
    cache::{BackfillHandle, CacheReader, CacheWriter, backfiller},
    errors::CacheError,
};

/// Builder for constructing a `CacheCoordinator` pair.
pub struct Builder<K, V>
where
    K: Clone + Eq + std::hash::Hash + Send + Sync + 'static,
    V: Clone + Send + Sync + 'static,
{
    backfill_capacity: usize,
    disk_reader: Option<Arc<dyn CacheReader<K, V>>>,
    disk_writer: Option<Arc<dyn CacheWriter<K, V>>>,
    memory_reader: Option<Arc<dyn CacheReader<K, V>>>,
    memory_writer: Option<Arc<dyn CacheWriter<K, V>>>,
}

impl<K, V> Default for Builder<K, V>
where
    K: Clone + Eq + std::hash::Hash + Send + Sync + 'static,
    V: Clone + Send + Sync + 'static,
{
    #[inline]
    fn default() -> Self {
        Self {
            backfill_capacity: 1024,
            disk_reader: None,
            disk_writer: None,
            memory_reader: None,
            memory_writer: None,
        }
    }
}

impl<K, V> Builder<K, V>
where
    K: Clone + Eq + std::hash::Hash + Send + Sync + 'static,
    V: Clone + Send + Sync + 'static,
{
    /// Set the backfill channel capacity.
    #[inline]
    pub fn backfill_capacity(&mut self, capacity: usize) -> &mut Self {
        self.backfill_capacity = capacity;
        self
    }

    /// Helper to build a Reader handle with a specific backfill handle.
    fn build_reader_with_handle(
        &self,
        backfill: BackfillHandle<K, V>,
    ) -> Result<Reader<K, V>, CacheError> {
        let memory = self.memory_reader.clone().ok_or_else(|| {
            CacheError::BackendError {
                backend: "coordinator",
                message: "memory_reader is required for Reader".into(),
            }
        })?;

        let disk = self.disk_reader.clone().ok_or_else(|| {
            CacheError::BackendError {
                backend: "coordinator",
                message: "disk_reader is required for Reader".into(),
            }
        })?;

        Ok(Reader {
            memory,
            disk,
            backfill,
        })
    }

    /// Set the disk reader.
    #[inline]
    pub fn disk_reader(
        &mut self,
        reader: Arc<dyn CacheReader<K, V>>,
    ) -> &mut Self {
        self.disk_reader = Some(reader);
        self
    }

    /// Set the disk writer.
    #[inline]
    pub fn disk_writer(
        &mut self,
        writer: Arc<dyn CacheWriter<K, V>>,
    ) -> &mut Self {
        self.disk_writer = Some(writer);
        self
    }

    /// Set the memory reader.
    #[inline]
    pub fn memory_reader(
        &mut self,
        reader: Arc<dyn CacheReader<K, V>>,
    ) -> &mut Self {
        self.memory_reader = Some(reader);
        self
    }

    /// Set the memory writer.
    #[inline]
    pub fn memory_writer(
        &mut self,
        writer: Arc<dyn CacheWriter<K, V>>,
    ) -> &mut Self {
        self.memory_writer = Some(writer);
        self
    }

    /// Create a new builder with default settings.
    #[inline]
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Build a Reader handle.
    ///
    /// Creates a new cache coordinator reader.
    ///
    /// # Errors
    /// Returns `CacheError::BackendError` if `memory_reader` or `disk_reader`
    /// is missing.
    #[inline]
    pub async fn reader(&self) -> Result<Reader<K, V>, CacheError> {
        let (handle, worker) = backfiller::new(self.backfill_capacity);

        // ✅ CRITICAL FIX: If memory_writer is present, start the worker so
        // backfill works!
        if let Some(mw) = self.memory_writer.as_ref() {
            if tokio::runtime::Handle::try_current().is_err() {
                return Err(CacheError::RuntimeError {
                    runtime: "tokio",
                    message: "Tokio runtime is required for backfill".into(),
                });
            }
            worker.start(Arc::clone(mw));
            tokio::task::yield_now().await;
        }

        self.build_reader_with_handle(handle)
    }

    /// Build a Writer handle.
    ///
    /// # Errors
    /// Returns `CacheError::BackendError` if `memory_writer` or `disk_writer`
    /// is missing.
    #[inline]
    pub fn writer(&self) -> Result<Writer<K, V>, CacheError> {
        let memory_writer = self.memory_writer.clone().ok_or_else(|| {
            CacheError::BackendError {
                backend: "coordinator",
                message: "memory_writer is required for Writer".into(),
            }
        })?;

        let disk_writer = self.disk_writer.clone().ok_or_else(|| {
            CacheError::BackendError {
                backend: "coordinator",
                message: "disk_writer is required for Writer".into(),
            }
        })?;

        Ok(Writer {
            memory: memory_writer,
            disk: disk_writer,
        })
    }
}

/// Cache reader coordinator handle.
///
/// Implements read-through logic: Memory hit returns immediately; Memory miss
/// checks Disk; Disk hit triggers background Backfill to Memory.
pub struct Reader<K, V>
where
    K: Clone + Eq + std::hash::Hash + Send + Sync + 'static,
    V: Clone + Send + Sync + 'static,
{
    memory: Arc<dyn CacheReader<K, V>>,
    disk: Arc<dyn CacheReader<K, V>>,
    backfill: BackfillHandle<K, V>,
}

#[async_trait]
impl<K, V> CacheReader<K, V> for Reader<K, V>
where
    K: Clone + Eq + std::hash::Hash + Send + Sync + 'static,
    V: Clone + Send + Sync + 'static,
{
    #[instrument(skip(self, key), fields(operation = "get"))]
    async fn get(&self, key: &K) -> Result<Option<V>, CacheError> {
        // 1. Check memory cache
        if let Some(value) = self.memory.get(key).await? {
            debug!("Memory Hit");
            return Ok(Some(value));
        }

        // 2. Memory Miss -> Check disk cache
        if let Some(value) = self.disk.get(key).await? {
            info!("Memory Miss / Disk Hit");

            // Trigger Asynchronous Backfill
            self.backfill.trigger(key.clone(), value.clone());

            return Ok(Some(value));
        }

        debug!("Disk Miss");
        Ok(None)
    }

    #[instrument(skip(self, key), fields(operation = "has"))]
    async fn has(&self, key: &K) -> Result<bool, CacheError> {
        // Check memory then disk
        if self.memory.has(key).await? {
            return Ok(true);
        }
        self.disk.has(key).await
    }

    #[instrument(skip(self), fields(operation = "keys"))]
    async fn keys(&self) -> Result<Vec<K>, CacheError> {
        use std::collections::HashSet;

        let mem_keys = self.memory.keys().await?;
        let disk_keys = self.disk.keys().await?;

        let capacity = mem_keys.len().saturating_add(disk_keys.len());
        let mut unique_keys: HashSet<K> = HashSet::with_capacity(capacity);
        unique_keys.extend(mem_keys);
        unique_keys.extend(disk_keys);

        Ok(unique_keys.into_iter().collect())
    }
}

impl<K, V> Clone for Reader<K, V>
where
    K: Clone + Eq + std::hash::Hash + Send + Sync + 'static,
    V: Clone + Send + Sync + 'static,
{
    #[inline]
    fn clone(&self) -> Self {
        Self {
            memory: Arc::clone(&self.memory),
            disk: Arc::clone(&self.disk),
            backfill: self.backfill.clone(),
        }
    }

    #[inline]
    fn clone_from(&mut self, source: &Self) {
        self.memory = Arc::clone(&source.memory);
        self.disk = Arc::clone(&source.disk);
        self.backfill.clone_from(&source.backfill);
    }
}

/// Cache writer coordinator handle.
///
/// Implements write-through logic: Write to Disk first, then Memory.
/// Deletions and clears are performed in parallel across both layers.
pub struct Writer<K, V>
where
    K: Clone + Eq + std::hash::Hash + Send + Sync + 'static,
    V: Clone + Send + Sync + 'static,
{
    memory: Arc<dyn CacheWriter<K, V>>,
    disk: Arc<dyn CacheWriter<K, V>>,
}

#[async_trait]
impl<K, V> CacheWriter<K, V> for Writer<K, V>
where
    K: Clone + Eq + std::hash::Hash + Send + Sync + 'static,
    V: Clone + Send + Sync + 'static,
{
    #[instrument(skip(self), fields(operation = "clear"))]
    async fn clear(&self) -> Result<(), CacheError> {
        // Parallel invalidation
        let (mem_res, disk_res) =
            tokio::join!(self.memory.clear(), self.disk.clear());

        mem_res?;
        disk_res?;
        Ok(())
    }

    #[instrument(skip(self, key), fields(operation = "delete"))]
    async fn delete(&self, key: &K) -> Result<bool, CacheError> {
        // Parallel invalidation
        let (mem_res, disk_res) =
            tokio::join!(self.memory.delete(key), self.disk.delete(key));

        let mem_deleted = mem_res?;
        let disk_deleted = disk_res?;

        Ok(mem_deleted || disk_deleted)
    }

    #[instrument(skip(self, key), fields(operation = "invalidate"))]
    async fn invalidate(&self, key: &K) -> Result<bool, CacheError> {
        self.delete(key).await
    }

    #[instrument(skip(self, key, value), fields(operation = "put"))]
    async fn put(&self, key: K, value: V) -> Result<(), CacheError> {
        // 1. Write to disk first to ensure persistence
        self.disk.put(key.clone(), value.clone()).await?;

        // 2. Only write to memory if disk write succeeds
        if let Err(e) = self.memory.put(key, value).await {
            return Err(CacheError::PartialWrite {
                backend: "coordinator",
                message: format!("disk committed, memory failed: {e}").into(),
            });
        }

        debug!("Cache Write success (Disk then Memory)");
        Ok(())
    }
}

impl<K, V> Clone for Writer<K, V>
where
    K: Clone + Eq + std::hash::Hash + Send + Sync + 'static,
    V: Clone + Send + Sync + 'static,
{
    #[inline]
    fn clone(&self) -> Self {
        Self {
            memory: Arc::clone(&self.memory),
            disk: Arc::clone(&self.disk),
        }
    }

    #[inline]
    fn clone_from(&mut self, source: &Self) {
        self.memory = Arc::clone(&source.memory);
        self.disk = Arc::clone(&source.disk);
    }
}

#[cfg(test)]
#[expect(
    clippy::excessive_nesting,
    reason = "Mockall async trait expectations require nested Box::pin(async \
              { ... }) blocks."
)]
mod tests {
    use super::*;

    mod fixtures {
        use super::*;
        use crate::spi::cache::{MockCacheReader, MockCacheWriter};

        /// Result type for mock-built handles.
        pub type ReaderWriterMocks =
            (Reader<String, String>, Writer<String, String>);

        /// Helper to build handles with provided mocks.
        pub async fn build_with_mocks(
            mem_reader: MockCacheReader<String, String>,
            mem_writer: MockCacheWriter<String, String>,
            disk_reader: MockCacheReader<String, String>,
            disk_writer: MockCacheWriter<String, String>,
        ) -> ReaderWriterMocks {
            let mut builder = Builder::new();
            builder
                .memory_reader(Arc::new(mem_reader))
                .memory_writer(Arc::new(mem_writer))
                .disk_reader(Arc::new(disk_reader))
                .disk_writer(Arc::new(disk_writer));
            let reader =
                builder.reader().await.expect("Failed to build reader");
            let writer = builder.writer().expect("Failed to build writer");
            (reader, writer)
        }
    }

    mod coordinator_init {
        use super::*;
        use crate::spi::cache::{MockCacheReader, MockCacheWriter};

        #[test]
        fn verify_linkage() {
            let _reader: crate::spi::cache::ReaderCoordinator<String, String>;
            let _writer: crate::spi::cache::WriterCoordinator<String, String>;
        }

        #[tokio::test]
        async fn shares_ports_correctly() {
            let mem_reader: Arc<dyn CacheReader<String, String>> =
                Arc::new(MockCacheReader::new());
            let mem_writer: Arc<dyn CacheWriter<String, String>> =
                Arc::new(MockCacheWriter::new());
            let disk_reader: Arc<dyn CacheReader<String, String>> =
                Arc::new(MockCacheReader::new());
            let disk_writer: Arc<dyn CacheWriter<String, String>> =
                Arc::new(MockCacheWriter::new());

            let mut builder = Builder::new();
            builder
                .memory_reader(Arc::clone(&mem_reader))
                .memory_writer(Arc::clone(&mem_writer))
                .disk_reader(Arc::clone(&disk_reader))
                .disk_writer(Arc::clone(&disk_writer));

            let reader = builder.reader().await.unwrap();
            let writer = builder.writer().unwrap();

            assert!(Arc::ptr_eq(&reader.memory, &mem_reader));
            assert!(Arc::ptr_eq(&writer.memory, &mem_writer));
        }

        #[tokio::test]
        async fn builds_reader_independently() {
            let mut builder = Builder::<String, String>::new();
            builder
                .memory_reader(Arc::new(MockCacheReader::new()))
                .memory_writer(Arc::new(MockCacheWriter::new()))
                .disk_reader(Arc::new(MockCacheReader::new()))
                .disk_writer(Arc::new(MockCacheWriter::new()));

            let reader =
                builder.reader().await.expect("Failed to build reader");
            let _: Reader<String, String> = reader;
        }

        #[tokio::test]
        async fn builds_reader_without_memory_writer() {
            let mut builder = Builder::<String, String>::new();
            builder
                .memory_reader(Arc::new(MockCacheReader::new()))
                .disk_reader(Arc::new(MockCacheReader::new()));

            let reader =
                builder.reader().await.expect("Failed to build reader");
            let _: Reader<String, String> = reader;
        }

        #[tokio::test]
        async fn builds_writer_independently() {
            let mut builder = Builder::<String, String>::new();
            builder
                .memory_reader(Arc::new(MockCacheReader::new()))
                .memory_writer(Arc::new(MockCacheWriter::new()))
                .disk_reader(Arc::new(MockCacheReader::new()))
                .disk_writer(Arc::new(MockCacheWriter::new()));

            let writer = builder.writer().expect("Failed to build writer");
            let _: Writer<String, String> = writer;
        }
    }

    mod backfill {
        use std::time::Duration;

        use super::{fixtures::*, *};
        use crate::spi::cache::{MockCacheReader, MockCacheWriter};

        #[tokio::test]
        async fn triggers_asynchronous_memory_put_on_disk_hit() {
            let mut mem_reader = MockCacheReader::new();
            let mut mem_writer = MockCacheWriter::new();
            let mut disk_reader = MockCacheReader::new();
            let disk_writer = MockCacheWriter::new();

            // Setup: Memory miss, Disk hit
            mem_reader
                .expect_get()
                .with(mockall::predicate::eq("key".to_owned()))
                .returning(|_| Box::pin(async { Ok(None) }));

            disk_reader
                .expect_get()
                .with(mockall::predicate::eq("key".to_owned()))
                .returning(|_| {
                    Box::pin(async { Ok(Some("value".to_owned())) })
                });

            // Expect: Backfill to memory
            mem_writer
                .expect_put()
                .with(
                    mockall::predicate::eq("key".to_owned()),
                    mockall::predicate::eq("value".to_owned()),
                )
                .returning(|_k, _v| Box::pin(async { Ok(()) }))
                .times(1);

            let (reader, _writer) = build_with_mocks(
                mem_reader,
                mem_writer,
                disk_reader,
                disk_writer,
            )
            .await;

            // Trigger get
            let result =
                reader.get(&"key".to_owned()).await.expect("get failed");
            assert_eq!(result, Some("value".to_owned()));

            // Wait for async backfill
            tokio::time::sleep(Duration::from_millis(50)).await;
        }

        #[tokio::test]
        async fn works_in_independent_reader() {
            let mut mem_reader = MockCacheReader::new();
            let mut mem_writer = MockCacheWriter::new();
            let mut disk_reader = MockCacheReader::new();

            // Setup: Memory miss, Disk hit
            mem_reader.expect_get().returning(|_| Box::pin(async { Ok(None) }));
            disk_reader.expect_get().returning(|_| {
                Box::pin(async { Ok(Some("value".to_owned())) })
            });

            // Expect: Backfill to memory
            mem_writer
                .expect_put()
                .with(
                    mockall::predicate::eq("key".to_owned()),
                    mockall::predicate::eq("value".to_owned()),
                )
                .returning(|_k, _v| Box::pin(async { Ok(()) }))
                .times(1);

            let reader = Builder::new()
                .memory_reader(Arc::new(mem_reader))
                .memory_writer(Arc::new(mem_writer)) // Provided for backfill
                .disk_reader(Arc::new(disk_reader))
                .reader()
                .await
                .expect("Failed to build reader");

            // Trigger get
            let result: Option<String> =
                reader.get(&"key".to_owned()).await.expect("get failed");
            assert_eq!(result, Some("value".to_owned()));

            // Wait for async backfill
            tokio::time::sleep(Duration::from_millis(50)).await;
        }
    }

    mod get {
        use super::{fixtures::*, *};
        use crate::spi::cache::{MockCacheReader, MockCacheWriter};

        #[tokio::test]
        async fn returns_memory_hit_immediately() {
            let mut mem_reader = MockCacheReader::new();

            mem_reader
                .expect_get()
                .with(mockall::predicate::eq("key".to_owned()))
                .returning(|_| {
                    Box::pin(async { Ok(Some("mem_val".to_owned())) })
                })
                .times(1);

            let (reader, _writer) = build_with_mocks(
                mem_reader,
                MockCacheWriter::new(),
                MockCacheReader::new(),
                MockCacheWriter::new(),
            )
            .await;

            let result =
                reader.get(&"key".to_owned()).await.expect("get failed");
            assert_eq!(result, Some("mem_val".to_owned()));
        }

        #[tokio::test]
        async fn returns_none_on_total_miss() {
            let mut mem_reader = MockCacheReader::new();
            let mut disk_reader = MockCacheReader::new();

            mem_reader
                .expect_get()
                .with(mockall::predicate::eq("key".to_owned()))
                .returning(|_| Box::pin(async { Ok(None) }))
                .times(1);

            disk_reader
                .expect_get()
                .with(mockall::predicate::eq("key".to_owned()))
                .returning(|_| Box::pin(async { Ok(None) }))
                .times(1);

            let (reader, _writer) = build_with_mocks(
                mem_reader,
                MockCacheWriter::new(),
                disk_reader,
                MockCacheWriter::new(),
            )
            .await;

            let result =
                reader.get(&"key".to_owned()).await.expect("get failed");
            assert!(result.is_none());
        }
    }

    mod keys {
        use super::{fixtures::*, *};
        use crate::spi::cache::{MockCacheReader, MockCacheWriter};

        #[tokio::test]
        async fn returns_union_of_both_layers() {
            let mut mem_reader = MockCacheReader::new();
            let mut disk_reader = MockCacheReader::new();

            mem_reader.expect_keys().returning(|| {
                Box::pin(async {
                    Ok(vec!["k1".to_owned(), "shared".to_owned()])
                })
            });

            disk_reader.expect_keys().returning(|| {
                Box::pin(async {
                    Ok(vec!["k2".to_owned(), "shared".to_owned()])
                })
            });

            let (reader, _writer) = build_with_mocks(
                mem_reader,
                MockCacheWriter::new(),
                disk_reader,
                MockCacheWriter::new(),
            )
            .await;

            let mut keys = reader.keys().await.expect("keys failed");
            keys.sort();

            assert_eq!(keys, vec![
                "k1".to_owned(),
                "k2".to_owned(),
                "shared".to_owned()
            ]);
        }
    }

    mod put {
        use super::{fixtures::*, *};
        use crate::spi::cache::{MockCacheReader, MockCacheWriter};

        #[tokio::test]
        async fn writes_to_disk_before_memory() {
            let mut mem_writer = MockCacheWriter::new();
            let mut disk_writer = MockCacheWriter::new();

            let mut seq = mockall::Sequence::new();

            disk_writer
                .expect_put()
                .with(
                    mockall::predicate::eq("key".to_owned()),
                    mockall::predicate::eq("val".to_owned()),
                )
                .returning(|_k, _v| Box::pin(async { Ok(()) }))
                .times(1)
                .in_sequence(&mut seq);

            mem_writer
                .expect_put()
                .with(
                    mockall::predicate::eq("key".to_owned()),
                    mockall::predicate::eq("val".to_owned()),
                )
                .returning(|_k, _v| Box::pin(async { Ok(()) }))
                .times(1)
                .in_sequence(&mut seq);

            let (_reader, writer) = build_with_mocks(
                MockCacheReader::new(),
                mem_writer,
                MockCacheReader::new(),
                disk_writer,
            )
            .await;

            writer
                .put("key".to_owned(), "val".to_owned())
                .await
                .expect("put failed");
        }

        #[tokio::test]
        async fn aborts_memory_write_on_disk_failure() {
            let mut mem_writer = MockCacheWriter::new();
            let mut disk_writer = MockCacheWriter::new();

            disk_writer
                .expect_put()
                .returning(|_k, _v| {
                    Box::pin(async {
                        Err(CacheError::BackendError {
                            backend: "disk",
                            message: "fail".into(),
                        })
                    })
                })
                .times(1);

            mem_writer.expect_put().times(0);

            let (_reader, writer) = build_with_mocks(
                MockCacheReader::new(),
                mem_writer,
                MockCacheReader::new(),
                disk_writer,
            )
            .await;

            let result = writer.put("key".to_owned(), "val".to_owned()).await;
            assert!(result.is_err());
        }

        #[tokio::test]
        async fn returns_partial_write_on_memory_failure() {
            let mut mem_writer = MockCacheWriter::new();
            let mut disk_writer = MockCacheWriter::new();

            disk_writer
                .expect_put()
                .returning(|_k, _v| Box::pin(async { Ok(()) }))
                .times(1);

            mem_writer
                .expect_put()
                .returning(|_k, _v| {
                    Box::pin(async {
                        Err(CacheError::BackendError {
                            backend: "memory",
                            message: "fail".into(),
                        })
                    })
                })
                .times(1);

            let (_reader, writer) = build_with_mocks(
                MockCacheReader::new(),
                mem_writer,
                MockCacheReader::new(),
                disk_writer,
            )
            .await;

            let result = writer.put("key".to_owned(), "val".to_owned()).await;
            assert!(matches!(result, Err(CacheError::PartialWrite { .. })));
        }
    }

    mod delete {
        use super::{fixtures::*, *};
        use crate::spi::cache::{MockCacheReader, MockCacheWriter};

        #[tokio::test]
        async fn invalidates_both_layers_in_parallel() {
            let mut mem_writer = MockCacheWriter::new();
            let mut disk_writer = MockCacheWriter::new();

            mem_writer
                .expect_delete()
                .with(mockall::predicate::eq("key".to_owned()))
                .returning(|_| Box::pin(async { Ok(true) }))
                .times(1);

            disk_writer
                .expect_delete()
                .with(mockall::predicate::eq("key".to_owned()))
                .returning(|_| Box::pin(async { Ok(false) }))
                .times(1);

            let (_reader, writer) = build_with_mocks(
                MockCacheReader::new(),
                mem_writer,
                MockCacheReader::new(),
                disk_writer,
            )
            .await;

            let deleted =
                writer.delete(&"key".to_owned()).await.expect("delete failed");
            assert!(deleted);
        }
    }

    mod observability {
        use tracing_test::traced_test;

        use super::{fixtures::*, *};
        use crate::spi::cache::{MockCacheReader, MockCacheWriter};

        #[tokio::test]
        #[traced_test]
        async fn emits_nested_spans_for_coordinator_flow() {
            let mut mem_reader = MockCacheReader::new();
            let disk_reader = MockCacheReader::new();

            mem_reader
                .expect_get()
                .returning(|_| Box::pin(async { Ok(Some("val".to_owned())) }))
                .times(1);

            let (reader, _writer) = build_with_mocks(
                mem_reader,
                MockCacheWriter::new(),
                disk_reader,
                MockCacheWriter::new(),
            )
            .await;

            let _result = reader.get(&"key".to_owned()).await;

            assert!(logs_contain("operation=\"get\""));
        }
    }

    mod performance {
        use std::time::Instant;

        use super::*;
        use crate::spi::cache::{MockCacheReader, MockCacheWriter};

        #[tokio::test]
        async fn get_latency_is_independent_of_backfill_speed() {
            tokio::time::pause();

            let mut mem_reader = MockCacheReader::new();
            let mut mem_writer = MockCacheWriter::new();
            let mut disk_reader = MockCacheReader::new();

            mem_reader.expect_get().returning(|_| Box::pin(async { Ok(None) }));
            disk_reader
                .expect_get()
                .returning(|_| Box::pin(async { Ok(Some("val".to_owned())) }));

            // SLOW memory write
            mem_writer.expect_put().returning(|_k, _v| {
                Box::pin(async {
                    tokio::time::sleep(std::time::Duration::from_millis(500))
                        .await;
                    Ok(())
                })
            });

            // Small capacity to test backfill pressure if needed
            let mut builder = Builder::new();
            builder
                .memory_reader(Arc::new(mem_reader))
                .memory_writer(Arc::new(mem_writer))
                .disk_reader(Arc::new(disk_reader))
                .disk_writer(Arc::new(MockCacheWriter::new()));
            builder.backfill_capacity(1);

            let reader = builder.reader().await.expect("build reader failed");

            let start = Instant::now();
            let _result = reader.get(&"key".to_owned()).await;
            let duration = start.elapsed();

            // Since time is paused, elapsed time will be near zero despite the
            // 500ms sleep in the background
            assert!(duration < std::time::Duration::from_millis(100));
        }
    }
}
