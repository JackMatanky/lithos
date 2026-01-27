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
//! let (reader, writer) = CacheCoordinatorBuilder::<String, String>::new()
//!     .memory_reader(Arc::new(DummyReader))
//!     .memory_writer(Arc::new(DummyWriter))
//!     .disk_reader(Arc::new(DummyReader))
//!     .disk_writer(Arc::new(DummyWriter))
//!     .build()
//!     .unwrap();
//! # });
//! ```

use std::sync::Arc;

use async_trait::async_trait;
use tokio::sync::mpsc;
use tracing::{debug, info, instrument};

use crate::spi::{
    cache::{CacheReader, CacheWriter},
    errors::CacheError,
};

/// Type alias for a Reader/Writer pair returned by `Builder::build()`.
pub type ReaderWriterPair<K, V> = (Reader<K, V>, Writer<K, V>);

/// Builder for constructing a `CacheCoordinator` pair.
pub struct Builder<K, V>
where
    K: Clone + Eq + std::hash::Hash + Send + Sync + std::fmt::Debug + 'static,
    V: Clone + Send + Sync + std::fmt::Debug + 'static,
{
    backfill_capacity: usize,
    disk_reader: Option<Arc<dyn CacheReader<K, V>>>,
    disk_writer: Option<Arc<dyn CacheWriter<K, V>>>,
    memory_reader: Option<Arc<dyn CacheReader<K, V>>>,
    memory_writer: Option<Arc<dyn CacheWriter<K, V>>>,
}

impl<K, V> Default for Builder<K, V>
where
    K: Clone + Eq + std::hash::Hash + Send + Sync + std::fmt::Debug + 'static,
    V: Clone + Send + Sync + std::fmt::Debug + 'static,
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
    K: Clone + Eq + std::hash::Hash + Send + Sync + std::fmt::Debug + 'static,
    V: Clone + Send + Sync + std::fmt::Debug + 'static,
{
    /// Set the backfill channel capacity.
    #[inline]
    pub fn backfill_capacity(&mut self, capacity: usize) -> &mut Self {
        self.backfill_capacity = capacity;
        self
    }

    /// Build the coordinator handles.
    ///
    /// # Errors
    /// Returns `CacheError::BackendError` if any of the required cache ports
    /// are not set.
    #[inline]
    pub fn build(&self) -> Result<ReaderWriterPair<K, V>, CacheError> {
        let inner = self.inner_builder()?;

        Ok((
            Reader {
                inner: Arc::clone(&inner),
            },
            Writer {
                inner,
            },
        ))
    }

    /// Build a Reader handle independently.
    ///
    /// # Errors
    /// Returns `CacheError::BackendError` if any of the required cache ports
    /// are not set.
    #[inline]
    pub fn build_reader(&self) -> Result<Reader<K, V>, CacheError> {
        let inner = self.inner_builder()?;
        Ok(Reader {
            inner,
        })
    }

    /// Build a Writer handle independently.
    ///
    /// # Errors
    /// Returns `CacheError::BackendError` if any of the required cache ports
    /// are not set.
    #[inline]
    pub fn build_writer(&self) -> Result<Writer<K, V>, CacheError> {
        let inner = self.inner_builder()?;
        Ok(Writer {
            inner,
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

    /// Internal helper to build the shared state and spawn the backfill task.
    #[inline]
    fn inner_builder(&self) -> Result<Arc<Inner<K, V>>, CacheError> {
        let memory_writer = self.memory_writer.clone().ok_or_else(|| {
            CacheError::BackendError {
                backend: "coordinator",
                message: "memory_writer is required".into(),
            }
        })?;

        let (backfill_tx, backfill_rx) = mpsc::channel(self.backfill_capacity);

        spawn_backfill_task(Arc::clone(&memory_writer), backfill_rx);

        Ok(Arc::new(Inner {
            backfill_tx,
            memory_reader: self.memory_reader.clone().ok_or_else(|| {
                CacheError::BackendError {
                    backend: "coordinator",
                    message: "memory_reader is required".into(),
                }
            })?,
            memory_writer,
            disk_reader: self.disk_reader.clone().ok_or_else(|| {
                CacheError::BackendError {
                    backend: "coordinator",
                    message: "disk_reader is required".into(),
                }
            })?,
            disk_writer: self.disk_writer.clone().ok_or_else(|| {
                CacheError::BackendError {
                    backend: "coordinator",
                    message: "disk_writer is required".into(),
                }
            })?,
        }))
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

    /// Create a new builder.
    #[inline]
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }
}

/// Cache reader coordinator handle.
#[derive(Clone)]
pub struct Reader<K, V>
where
    K: Clone + Eq + std::hash::Hash + Send + Sync + std::fmt::Debug + 'static,
    V: Clone + Send + Sync + std::fmt::Debug + 'static,
{
    inner: Arc<Inner<K, V>>,
}

#[async_trait]
impl<K, V> CacheReader<K, V> for Reader<K, V>
where
    K: Clone + Eq + std::hash::Hash + Send + Sync + std::fmt::Debug + 'static,
    V: Clone + Send + Sync + std::fmt::Debug + 'static,
{
    #[instrument(skip(self), fields(operation = "get"))]
    async fn get(&self, key: &K) -> Result<Option<V>, CacheError> {
        // 1. Check memory cache
        if let Some(value) = self.inner.memory_reader.get(key).await? {
            debug!(?key, "Memory Hit");
            return Ok(Some(value));
        }

        // 2. Memory Miss -> Check disk cache
        if let Some(value) = self.inner.disk_reader.get(key).await? {
            info!(?key, "Memory Miss / Disk Hit");

            // Trigger Asynchronous Backfill
            let request = BackfillRequest {
                key: key.clone(),
                value: value.clone(),
            };

            // Non-blocking send: drop if channel is full to ensure latency is
            // never affected
            match self.inner.backfill_tx.try_send(request) {
                Ok(()) => {
                    debug!(?key, operation = "backfill", status = "triggered");
                }
                Err(mpsc::error::TrySendError::Full(_)) => {
                    info!(
                        ?key,
                        operation = "backfill",
                        status = "dropped",
                        reason = "channel full"
                    );
                }
                Err(mpsc::error::TrySendError::Closed(_)) => {
                    info!(
                        ?key,
                        operation = "backfill",
                        status = "dropped",
                        reason = "channel closed"
                    );
                }
            }

            return Ok(Some(value));
        }

        info!(?key, "Disk Miss");
        Ok(None)
    }

    #[instrument(skip(self), fields(operation = "has"))]
    async fn has(&self, key: &K) -> Result<bool, CacheError> {
        // Check memory then disk
        if self.inner.memory_reader.has(key).await? {
            return Ok(true);
        }
        self.inner.disk_reader.has(key).await
    }

    #[instrument(skip(self), fields(operation = "keys"))]
    async fn keys(&self) -> Result<Vec<K>, CacheError> {
        use std::collections::HashSet;

        let mem_keys = self.inner.memory_reader.keys().await?;
        let disk_keys = self.inner.disk_reader.keys().await?;

        let mut unique_keys: HashSet<K> = mem_keys.into_iter().collect();
        unique_keys.extend(disk_keys);

        Ok(unique_keys.into_iter().collect())
    }
}

/// Cache writer coordinator handle.
#[derive(Clone)]
pub struct Writer<K, V>
where
    K: Clone + Eq + std::hash::Hash + Send + Sync + std::fmt::Debug + 'static,
    V: Clone + Send + Sync + std::fmt::Debug + 'static,
{
    inner: Arc<Inner<K, V>>,
}

#[async_trait]
impl<K, V> CacheWriter<K, V> for Writer<K, V>
where
    K: Clone + Eq + std::hash::Hash + Send + Sync + std::fmt::Debug + 'static,
    V: Clone + Send + Sync + std::fmt::Debug + 'static,
{
    #[instrument(skip(self), fields(operation = "clear"))]
    async fn clear(&self) -> Result<(), CacheError> {
        // Parallel invalidation
        let (mem_res, disk_res) = tokio::join!(
            self.inner.memory_writer.clear(),
            self.inner.disk_writer.clear()
        );

        mem_res?;
        disk_res?;
        Ok(())
    }

    #[instrument(skip(self), fields(operation = "delete"))]
    async fn delete(&self, key: &K) -> Result<bool, CacheError> {
        // Parallel invalidation
        let (mem_res, disk_res) = tokio::join!(
            self.inner.memory_writer.delete(key),
            self.inner.disk_writer.delete(key)
        );

        let mem_deleted = mem_res?;
        let disk_deleted = disk_res?;

        Ok(mem_deleted || disk_deleted)
    }

    #[instrument(skip(self), fields(operation = "invalidate"))]
    async fn invalidate(&self, key: &K) -> Result<bool, CacheError> {
        self.delete(key).await
    }

    #[instrument(skip(self, value), fields(operation = "put"))]
    async fn put(&self, key: K, value: V) -> Result<(), CacheError> {
        // 1. Write to disk first to ensure persistence
        self.inner.disk_writer.put(key.clone(), value.clone()).await?;

        // 2. Only write to memory if disk write succeeds
        self.inner.memory_writer.put(key, value).await?;

        debug!("Cache Write success (Disk then Memory)");
        Ok(())
    }
}

/// Request to backfill a value from disk to memory.
struct BackfillRequest<K, V> {
    key: K,
    value: V,
}

/// Internal state shared between Reader and Writer handles.
pub(crate) struct Inner<K, V>
where
    K: Clone + Eq + std::hash::Hash + Send + Sync + std::fmt::Debug + 'static,
    V: Clone + Send + Sync + std::fmt::Debug + 'static,
{
    backfill_tx: mpsc::Sender<BackfillRequest<K, V>>,
    disk_reader: Arc<dyn CacheReader<K, V>>,
    disk_writer: Arc<dyn CacheWriter<K, V>>,
    memory_reader: Arc<dyn CacheReader<K, V>>,
    memory_writer: Arc<dyn CacheWriter<K, V>>,
}

/// Spawn a background task to process backfill requests.
fn spawn_backfill_task<K, V>(
    memory_writer: Arc<dyn CacheWriter<K, V>>,
    mut backfill_rx: mpsc::Receiver<BackfillRequest<K, V>>,
) where
    K: Clone + Eq + std::hash::Hash + Send + Sync + std::fmt::Debug + 'static,
    V: Clone + Send + Sync + std::fmt::Debug + 'static,
{
    tokio::spawn(async move {
        while let Some(request) = backfill_rx.recv().await {
            let result = memory_writer.put(request.key, request.value).await;
            if let Err(e) = result {
                info!(error = ?e, "Async backfill failed");
            }
        }
    });
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

        /// Helper to build handles with provided mocks.
        pub fn build_with_mocks(
            mem_reader: MockCacheReader<String, String>,
            mem_writer: MockCacheWriter<String, String>,
            disk_reader: MockCacheReader<String, String>,
            disk_writer: MockCacheWriter<String, String>,
        ) -> ReaderWriterPair<String, String> {
            let mut builder = Builder::new();
            builder
                .memory_reader(Arc::new(mem_reader))
                .memory_writer(Arc::new(mem_writer))
                .disk_reader(Arc::new(disk_reader))
                .disk_writer(Arc::new(disk_writer));
            builder.build().expect("Failed to build coordinator")
        }
    }

    mod coordinator_init {
        use super::{fixtures::*, *};
        use crate::spi::cache::{MockCacheReader, MockCacheWriter};

        #[test]
        fn verify_linkage() {
            let _reader: crate::spi::cache::ReaderCoordinator<String, String>;
            let _writer: crate::spi::cache::WriterCoordinator<String, String>;
        }

        #[tokio::test]
        async fn shares_inner_state_between_handles() {
            let (reader, writer) = build_with_mocks(
                MockCacheReader::new(),
                MockCacheWriter::new(),
                MockCacheReader::new(),
                MockCacheWriter::new(),
            );

            assert!(Arc::ptr_eq(&reader.inner, &writer.inner));
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
                builder.build_reader().expect("Failed to build reader");
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

            let writer =
                builder.build_writer().expect("Failed to build writer");
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
                    #[expect(
                        clippy::excessive_nesting,
                        reason = "Mockall async trait expectations require \
                                  nested Box::pin(async { ... }) blocks."
                    )]
                    Box::pin(async { Ok(Some("value".to_owned())) })
                });

            // Expect: Backfill to memory
            mem_writer
                .expect_put()
                .with(
                    mockall::predicate::eq("key".to_owned()),
                    mockall::predicate::eq("value".to_owned()),
                )
                .returning(|_, _| Box::pin(async { Ok(()) }))
                .times(1);

            let (reader, _writer) = build_with_mocks(
                mem_reader,
                mem_writer,
                disk_reader,
                disk_writer,
            );

            // Trigger get
            let result =
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
                    #[expect(
                        clippy::excessive_nesting,
                        reason = "Mockall async trait expectations require \
                                  nested Box::pin(async { ... }) blocks."
                    )]
                    Box::pin(async { Ok(Some("mem_val".to_owned())) })
                })
                .times(1);

            let (reader, _writer) = build_with_mocks(
                mem_reader,
                MockCacheWriter::new(),
                MockCacheReader::new(),
                MockCacheWriter::new(),
            );

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
            );

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
                #[expect(
                    clippy::excessive_nesting,
                    reason = "Mockall async trait expectations require nested \
                              Box::pin(async { ... }) blocks."
                )]
                Box::pin(async {
                    Ok(vec!["k1".to_owned(), "shared".to_owned()])
                })
            });

            disk_reader.expect_keys().returning(|| {
                #[expect(
                    clippy::excessive_nesting,
                    reason = "Mockall async trait expectations require nested \
                              Box::pin(async { ... }) blocks."
                )]
                Box::pin(async {
                    Ok(vec!["k2".to_owned(), "shared".to_owned()])
                })
            });

            let (reader, _writer) = build_with_mocks(
                mem_reader,
                MockCacheWriter::new(),
                disk_reader,
                MockCacheWriter::new(),
            );

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
                .returning(|_, _| Box::pin(async { Ok(()) }))
                .times(1)
                .in_sequence(&mut seq);

            mem_writer
                .expect_put()
                .with(
                    mockall::predicate::eq("key".to_owned()),
                    mockall::predicate::eq("val".to_owned()),
                )
                .returning(|_, _| Box::pin(async { Ok(()) }))
                .times(1)
                .in_sequence(&mut seq);

            let (_reader, writer) = build_with_mocks(
                MockCacheReader::new(),
                mem_writer,
                MockCacheReader::new(),
                disk_writer,
            );

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
                .returning(|_, _| {
                    #[expect(
                        clippy::excessive_nesting,
                        reason = "Mockall async trait expectations require \
                                  nested Box::pin(async { ... }) blocks."
                    )]
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
            );

            let result = writer.put("key".to_owned(), "val".to_owned()).await;
            assert!(result.is_err());
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
            );

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
                .returning(|_| {
                    #[expect(
                        clippy::excessive_nesting,
                        reason = "Mockall async trait expectations require \
                                  nested Box::pin(async { ... }) blocks."
                    )]
                    Box::pin(async { Ok(Some("val".to_owned())) })
                })
                .times(1);

            let (reader, _writer) = build_with_mocks(
                mem_reader,
                MockCacheWriter::new(),
                disk_reader,
                MockCacheWriter::new(),
            );

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
            let mut mem_reader = MockCacheReader::new();
            let mut mem_writer = MockCacheWriter::new();
            let mut dr = MockCacheReader::new();

            mem_reader.expect_get().returning(|_| Box::pin(async { Ok(None) }));
            dr.expect_get().returning(|_| {
                #[expect(
                    clippy::excessive_nesting,
                    reason = "Mockall async trait expectations require nested \
                              Box::pin(async { ... }) blocks."
                )]
                Box::pin(async { Ok(Some("val".to_owned())) })
            });

            // SLOW memory write
            mem_writer.expect_put().returning(|_, _| {
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
                .disk_reader(Arc::new(dr))
                .disk_writer(Arc::new(MockCacheWriter::new()));
            builder.backfill_capacity(1);

            let (reader, _writer) = builder.build().expect("build failed");

            let start = Instant::now();
            let _result = reader.get(&"key".to_owned()).await;
            let duration = start.elapsed();

            // Should be much faster than the 500ms sleep
            assert!(duration < std::time::Duration::from_millis(100));
        }
    }
}
