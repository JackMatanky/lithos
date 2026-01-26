//! # Eviction Policy
//!
//! `MokaCache` uses the `TinyLFU` eviction policy, which is highly resistant to
//! scan pollution. This is particularly important for Obsidian vaults where
//! sequential indexing of many files could otherwise evict frequently accessed
//! "hot" metadata.
//!
//! # Async Safety
//!
//! This implementation uses `moka::future::Cache`, which is designed for
//! asynchronous runtimes like Tokio. It performs non-blocking operations and
//! handles internal coordination efficiently.

use std::{marker::PhantomData, sync::Arc, time::Duration};

use async_trait::async_trait;

use crate::spi::{
    cache::{CacheReader, CacheWriter, deserializer::IdentityCodec},
    errors::CacheError,
};

/// Inner state for Moka cache.
///
/// This struct holds the actual Moka cache and is wrapped by Reader/Writer
/// handles. It's not directly clonable to enforce the use of Arc for sharing.
#[derive(Debug)]
pub(crate) struct Inner<K, V>
where
    K: Clone + Eq + std::hash::Hash + Send + Sync + 'static,
    V: Clone + Send + Sync + 'static,
{
    cache: moka::future::Cache<K, V>,
    #[expect(
        dead_code,
        reason = "Codec field reserved for future use in generic Handle/Inner \
                  pattern"
    )]
    codec: IdentityCodec,
}

/// Read-only handle for Moka cache.
///
/// This handle provides read-only access to the cache following CQRS
/// principles.
#[derive(Debug, Clone)]
pub struct Reader<K, V, C = IdentityCodec>
where
    K: Clone + Eq + std::hash::Hash + Send + Sync + 'static,
    V: Clone + Send + Sync + 'static,
{
    inner: Arc<Inner<K, V>>,
    _codec: PhantomData<C>,
}

/// Write-only handle for Moka cache.
///
/// This handle provides write-only access to the cache following CQRS
/// principles.
#[derive(Debug, Clone)]
pub struct Writer<K, V, C = IdentityCodec>
where
    K: Clone + Eq + std::hash::Hash + Send + Sync + 'static,
    V: Clone + Send + Sync + 'static,
{
    inner: Arc<Inner<K, V>>,
    _codec: PhantomData<C>,
}

// Implement CacheReader for Reader
#[async_trait]
impl<K, V> CacheReader<K, V> for Reader<K, V, IdentityCodec>
where
    K: Clone + Eq + std::hash::Hash + Send + Sync + 'static,
    V: Clone + Send + Sync + 'static,
{
    #[tracing::instrument(skip(self, key), level = "debug")]
    #[inline]
    async fn get(&self, key: &K) -> Result<Option<V>, CacheError> {
        let hit = self.inner.cache.get(key).await;
        tracing::event!(
            tracing::Level::DEBUG,
            cache_layer = "memory",
            operation = "get",
            hit = hit.is_some()
        );
        Ok(hit)
    }

    #[tracing::instrument(skip(self, key), level = "debug")]
    #[inline]
    async fn has(&self, key: &K) -> Result<bool, CacheError> {
        let exists = self.inner.cache.contains_key(key);
        tracing::event!(
            tracing::Level::DEBUG,
            cache_layer = "memory",
            operation = "has",
            exists = exists
        );
        Ok(exists)
    }
}

// Implement CacheWriter for Writer
#[async_trait]
impl<K, V> CacheWriter<K, V> for Writer<K, V, IdentityCodec>
where
    K: Clone + Eq + std::hash::Hash + Send + Sync + 'static,
    V: Clone + Send + Sync + 'static,
{
    #[tracing::instrument(skip(self), level = "debug")]
    #[inline]
    async fn clear(&self) -> Result<(), CacheError> {
        self.inner.cache.invalidate_all();
        tracing::event!(
            tracing::Level::DEBUG,
            cache_layer = "memory",
            operation = "clear"
        );
        Ok(())
    }

    #[tracing::instrument(skip(self, key), level = "debug")]
    #[inline]
    async fn delete(&self, key: &K) -> Result<bool, CacheError> {
        let existed = self.inner.cache.remove(key).await.is_some();
        tracing::event!(
            tracing::Level::DEBUG,
            cache_layer = "memory",
            operation = "delete",
            existed = existed
        );
        Ok(existed)
    }

    #[tracing::instrument(skip(self, key), level = "debug")]
    #[inline]
    async fn invalidate(&self, key: &K) -> Result<bool, CacheError> {
        let existed = self.delete(key).await?;
        tracing::event!(
            tracing::Level::DEBUG,
            cache_layer = "memory",
            operation = "invalidate",
            existed = existed
        );
        Ok(existed)
    }

    #[tracing::instrument(skip(self, key, value), level = "debug")]
    #[inline]
    async fn put(&self, key: K, value: V) -> Result<(), CacheError> {
        self.inner.cache.insert(key, value).await;
        tracing::event!(
            tracing::Level::DEBUG,
            cache_layer = "memory",
            operation = "put"
        );
        Ok(())
    }
}

/// Deprecated unified cache struct.
///
/// **DEPRECATED**: Use `Reader` and `Writer` handles instead.
/// This struct is kept temporarily for backwards compatibility.
///
/// # Migration Guide
///
/// Old code:
/// ```ignore
/// let cache = Cache::builder().build()?;
/// cache.put(key, value).await?;
/// let val = cache.get(&key).await?;
/// ```
///
/// New code:
/// ```rust
/// use lithos_adapters::spi::cache::{
///     CacheReader, CacheWriter,
///     moka::{Cache, Reader, Writer},
/// };
/// # tokio::runtime::Runtime::new().unwrap().block_on(async {
/// let (reader, writer) = Cache::<String, String>::builder().build().unwrap();
/// writer.put("key".to_string(), "value".to_string()).await.unwrap();
/// let val = reader.get(&"key".to_string()).await.unwrap();
/// assert_eq!(val, Some("value".to_string()));
/// # });
/// ```
///
/// # `TinyLFU` and Scan Pollution
///
/// `TinyLFU` is used to protect frequently accessed entries from being evicted
/// by sequential scans (e.g., during vault indexing).
///
/// ```rust
/// use lithos_adapters::spi::cache::{CacheReader, CacheWriter, moka::Cache};
/// # tokio::runtime::Runtime::new().unwrap().block_on(async {
/// let (reader, writer) =
///     Cache::<String, String>::builder().max_capacity(10).build().unwrap();
///
/// // Access a "hot" key many times
/// for _ in 0..20 {
///     writer.put("hot".to_string(), "value".to_string()).await.unwrap();
///     let _ = reader.get(&"hot".to_string()).await.unwrap();
/// }
///
/// // Perform a "scan" that exceeds capacity
/// for i in 0..100 {
///     writer.put(format!("scan-{}", i), "val".to_string()).await.unwrap();
/// }
///
/// // The "hot" key should still be present because of TinyLFU
/// tokio::time::sleep(std::time::Duration::from_millis(100)).await;
/// assert!(reader.get(&"hot".to_string()).await.unwrap().is_some());
/// # });
/// ```
#[derive(Debug, Clone)]
#[deprecated(
    since = "0.1.0",
    note = "Use Reader and Writer handles instead for CQRS compliance"
)]
pub struct Cache<K, V>
where
    K: Clone + Eq + std::hash::Hash + Send + Sync + 'static,
    V: Clone + Send + Sync + 'static,
{
    _marker: PhantomData<(K, V)>,
}

#[expect(
    deprecated,
    reason = "Cache struct is deprecated but must remain for backwards \
              compatibility during migration. It only provides a builder() \
              method that returns the new Builder API."
)]
impl<K, V> Cache<K, V>
where
    K: Clone + Eq + std::hash::Hash + Send + Sync + 'static,
    V: Clone + Send + Sync + 'static,
{
    /// Create a new builder for Moka cache.
    ///
    /// Returns a builder that constructs split Reader/Writer handles.
    #[inline]
    #[must_use]
    pub fn builder() -> Builder<K, V> {
        Builder::default()
    }
}

/// Type alias for the tuple returned by Moka builder.
pub type BuildResult<K, V> = Result<(Reader<K, V>, Writer<K, V>), CacheError>;

/// Builder for `MokaCache`.
#[derive(Debug, Clone)]
pub struct Builder<K, V>
where
    K: Clone + Eq + std::hash::Hash + Send + Sync + 'static,
    V: Clone + Send + Sync + 'static,
{
    max_capacity: usize,
    time_to_live: Option<Duration>,
    time_to_idle: Option<Duration>,
    _k: PhantomData<K>,
    _v: PhantomData<V>,
}

impl<K, V> Default for Builder<K, V>
where
    K: Clone + Eq + std::hash::Hash + Send + Sync + 'static,
    V: Clone + Send + Sync + 'static,
{
    #[inline]
    fn default() -> Self {
        Self {
            max_capacity: 10_000,
            time_to_live: None,
            time_to_idle: None,
            _k: PhantomData,
            _v: PhantomData,
        }
    }
}

impl<K, V> Builder<K, V>
where
    K: Clone + Eq + std::hash::Hash + Send + Sync + 'static,
    V: Clone + Send + Sync + 'static,
{
    /// Build split Reader/Writer handles for the Moka cache.
    ///
    /// Returns a tuple of (Reader, Writer) handles that share the same
    /// underlying cache. Use this when both read and write access is needed.
    ///
    /// # Errors
    ///
    /// Returns `CacheError::BackendError` if the configuration is invalid
    /// (e.g., `max_capacity` is 0).
    #[inline]
    pub fn build(&self) -> BuildResult<K, V> {
        let inner = self.build_inner()?;

        let reader = Reader {
            inner: Arc::clone(&inner),
            _codec: PhantomData,
        };

        let writer = Writer {
            inner,
            _codec: PhantomData,
        };

        Ok((reader, writer))
    }

    /// Internal helper to build the Inner state.
    #[inline]
    fn build_inner(&self) -> Result<Arc<Inner<K, V>>, CacheError> {
        if self.max_capacity == 0 {
            return Err(CacheError::BackendError {
                backend: "moka",
                message: "max_capacity must be greater than 0".into(),
            });
        }

        let mut builder = moka::future::Cache::builder().max_capacity(
            self.max_capacity.try_into().map_err(|e| {
                CacheError::BackendError {
                    backend: "moka",
                    message: format!("Invalid max_capacity: {e}").into(),
                }
            })?,
        );

        if let Some(ttl) = self.time_to_live {
            builder = builder.time_to_live(ttl);
        }

        if let Some(tti) = self.time_to_idle {
            builder = builder.time_to_idle(tti);
        }

        let cache = builder.build();
        Ok(Arc::new(Inner {
            cache,
            codec: IdentityCodec,
        }))
    }

    /// Build a Reader handle independently.
    ///
    /// Creates a new cache and returns only a Reader handle. This is
    /// efficient when only read access is needed.
    ///
    /// # Errors
    ///
    /// Returns `CacheError::BackendError` if the configuration is invalid
    /// (e.g., `max_capacity` is 0).
    #[inline]
    pub fn build_reader(&self) -> Result<Reader<K, V>, CacheError> {
        let inner = self.build_inner()?;
        Ok(Reader {
            inner,
            _codec: PhantomData,
        })
    }

    /// Build a Writer handle independently.
    ///
    /// Creates a new cache and returns only a Writer handle. This is
    /// efficient when only write access is needed.
    ///
    /// # Errors
    ///
    /// Returns `CacheError::BackendError` if the configuration is invalid
    /// (e.g., `max_capacity` is 0).
    #[inline]
    pub fn build_writer(&self) -> Result<Writer<K, V>, CacheError> {
        let inner = self.build_inner()?;
        Ok(Writer {
            inner,
            _codec: PhantomData,
        })
    }

    /// Set maximum capacity.
    #[inline]
    pub fn max_capacity(&mut self, capacity: usize) -> &mut Self {
        self.max_capacity = capacity;
        self
    }

    /// Set time to idle.
    #[inline]
    pub fn time_to_idle(&mut self, duration: Duration) -> &mut Self {
        self.time_to_idle = Some(duration);
        self
    }

    /// Set time to live.
    #[inline]
    pub fn time_to_live(&mut self, duration: Duration) -> &mut Self {
        self.time_to_live = Some(duration);
        self
    }
}

#[cfg(test)]
#[expect(
    deprecated,
    reason = "Tests use deprecated Cache struct for backwards compatibility \
              verification"
)]
mod tests {
    use super::*;

    mod moka_builder {
        use super::*;

        #[test]
        fn builds_reader_independently() {
            let result = Builder::<String, String>::default()
                .max_capacity(50)
                .build_reader();

            assert!(result.is_ok());
            let reader = result.unwrap();

            // Verify handle is correct type
            let _: Reader<String, String> = reader;
        }

        #[test]
        fn builds_writer_independently() {
            let result = Builder::<String, String>::default()
                .max_capacity(50)
                .build_writer();

            assert!(result.is_ok());
            let writer = result.unwrap();

            // Verify handle is correct type
            let _: Writer<String, String> = writer;
        }

        #[test]
        fn builds_split_handles_with_custom_capacity() {
            let result =
                Builder::<String, String>::default().max_capacity(50).build();

            assert!(result.is_ok());
            let (reader, writer) = result.unwrap();

            // Verify handles are distinct types
            let _: Reader<String, String> = reader;
            let _: Writer<String, String> = writer;
        }
    }

    mod initialization {
        use super::*;

        #[test]
        fn should_return_builder_instance() {
            let _builder: Builder<String, String> =
                Cache::<String, String>::builder();
        }

        #[test]
        fn should_allow_configuring_max_capacity() {
            let mut builder = Cache::<String, String>::builder();
            let _: &mut Builder<String, String> =
                builder.max_capacity(100usize);
        }

        #[test]
        fn should_allow_configuring_ttl() {
            let mut builder = Cache::<String, String>::builder();
            let _: &mut Builder<String, String> =
                builder.time_to_live(Duration::from_secs(10u64));
        }

        #[test]
        fn should_allow_configuring_tti() {
            let mut builder = Cache::<String, String>::builder();
            let _: &mut Builder<String, String> =
                builder.time_to_idle(Duration::from_secs(10u64));
        }

        #[test]
        fn should_build_cache_instance() {
            let result = Cache::<String, String>::builder().build();
            let (_reader, _writer) = result.expect("Failed to build cache");
        }

        #[test]
        #[expect(
            clippy::panic,
            reason = "Panic is used in tests to fail fast when expectations \
                      are not met."
        )]
        fn should_return_error_for_zero_capacity() {
            let result =
                Cache::<String, String>::builder().max_capacity(0usize).build();

            assert!(result.is_err());
            match result.unwrap_err() {
                CacheError::BackendError {
                    backend,
                    message,
                } => {
                    assert_eq!(backend, "moka");
                    assert!(message.contains("capacity"));
                }
                CacheError::IoError(_)
                | CacheError::SerializationError {
                    ..
                } => {
                    panic!("Expected BackendError")
                }
            }
        }
    }

    mod core_ops {
        use super::*;

        #[tokio::test]
        async fn should_get_none_from_empty_cache() {
            let (reader, _writer) =
                Cache::<String, String>::builder().build().unwrap();
            let result = reader.get(&"key".to_owned()).await.unwrap();
            assert!(result.is_none());
        }

        #[tokio::test]
        async fn should_put_and_get_value() {
            let (reader, writer) =
                Cache::<String, String>::builder().build().unwrap();
            writer.put("key".to_owned(), "value".to_owned()).await.unwrap();
            let result = reader.get(&"key".to_owned()).await.unwrap();
            assert_eq!(result, Some("value".to_owned()));
        }

        #[tokio::test]
        async fn should_delete_value() {
            let (reader, writer) =
                Cache::<String, String>::builder().build().unwrap();
            writer.put("key".to_owned(), "value".to_owned()).await.unwrap();
            let existed = writer.delete(&"key".to_owned()).await.unwrap();
            assert!(existed);
            let result = reader.get(&"key".to_owned()).await.unwrap();
            assert!(result.is_none());

            let existed_again = writer.delete(&"key".to_owned()).await.unwrap();
            assert!(!existed_again);
        }

        #[tokio::test]
        async fn should_check_has() {
            let (reader, writer) =
                Cache::<String, String>::builder().build().unwrap();
            assert!(!reader.has(&"key".to_owned()).await.unwrap());

            writer.put("key".to_owned(), "value".to_owned()).await.unwrap();
            assert!(reader.has(&"key".to_owned()).await.unwrap());

            writer.delete(&"key".to_owned()).await.unwrap();
            assert!(!reader.has(&"key".to_owned()).await.unwrap());
        }

        #[tokio::test]
        async fn should_clear_all_entries() {
            let (reader, writer) =
                Cache::<String, String>::builder().build().unwrap();
            writer.put("k1".to_owned(), "v1".to_owned()).await.unwrap();
            writer.put("k2".to_owned(), "v2".to_owned()).await.unwrap();

            writer.clear().await.unwrap();

            assert!(!reader.has(&"k1".to_owned()).await.unwrap());
            assert!(!reader.has(&"k2".to_owned()).await.unwrap());
        }
    }

    mod observability {
        use tracing_test::traced_test;

        use super::*;

        #[tokio::test]
        #[traced_test]
        async fn should_emit_events_on_get() {
            let (reader, _writer) =
                Cache::<String, String>::builder().build().unwrap();
            let _: Option<String> =
                reader.get(&"key".to_owned()).await.unwrap();

            assert!(logs_contain("operation=\"get\""));
            assert!(logs_contain("hit=false"));
        }

        #[tokio::test]
        #[traced_test]
        async fn should_emit_events_on_put() {
            let (_reader, writer) =
                Cache::<String, String>::builder().build().unwrap();
            writer.put("key".to_owned(), "value".to_owned()).await.unwrap();

            assert!(logs_contain("operation=\"put\""));
        }

        #[tokio::test]
        #[traced_test]
        async fn should_emit_events_on_delete() {
            let (_reader, writer) =
                Cache::<String, String>::builder().build().unwrap();
            writer.put("key".to_owned(), "value".to_owned()).await.unwrap();
            let _: bool = writer.delete(&"key".to_owned()).await.unwrap();

            assert!(logs_contain("operation=\"delete\""));
            assert!(logs_contain("existed=true"));
        }

        #[tokio::test]
        #[traced_test]
        async fn should_emit_events_on_invalidate() {
            let (_reader, writer) =
                Cache::<String, String>::builder().build().unwrap();
            writer.put("key".to_owned(), "value".to_owned()).await.unwrap();
            let _: bool = writer.invalidate(&"key".to_owned()).await.unwrap();

            assert!(logs_contain("operation=\"invalidate\""));
            assert!(logs_contain("existed=true"));
        }
    }

    mod eviction {
        use super::*;

        #[tokio::test]
        async fn should_respect_ttl() {
            let (reader, writer) = Cache::<String, String>::builder()
                .time_to_live(Duration::from_millis(50u64))
                .build()
                .unwrap();

            writer.put("key".to_owned(), "value".to_owned()).await.unwrap();
            assert_eq!(
                reader.get(&"key".to_owned()).await.unwrap(),
                Some("value".to_owned())
            );

            tokio::time::sleep(Duration::from_millis(100u64)).await;
            assert_eq!(reader.get(&"key".to_owned()).await.unwrap(), None);
        }

        #[tokio::test]
        async fn should_respect_tti() {
            let (reader, writer) = Cache::<String, String>::builder()
                .time_to_idle(Duration::from_millis(100u64))
                .build()
                .unwrap();

            writer.put("key".to_owned(), "value".to_owned()).await.unwrap();

            tokio::time::sleep(Duration::from_millis(60u64)).await;
            // Access it to reset TTI
            assert_eq!(
                reader.get(&"key".to_owned()).await.unwrap(),
                Some("value".to_owned())
            );

            tokio::time::sleep(Duration::from_millis(60u64)).await;
            // Should still be there because TTI was reset at 60ms
            assert_eq!(
                reader.get(&"key".to_owned()).await.unwrap(),
                Some("value".to_owned())
            );

            tokio::time::sleep(Duration::from_millis(150u64)).await;
            // Now it should be gone
            assert_eq!(reader.get(&"key".to_owned()).await.unwrap(), None);
        }

        #[tokio::test]
        #[expect(
            clippy::excessive_nesting,
            reason = "Test logic requires multiple loops and async blocks \
                      which trigger this lint in the test suite."
        )]
        async fn should_respect_max_capacity() {
            let (reader, writer) = Cache::<String, String>::builder()
                .max_capacity(10usize)
                .build()
                .unwrap();

            for i in 0i32..100i32 {
                writer
                    .put(format!("key{i}"), format!("value{i}"))
                    .await
                    .unwrap();
            }

            // Moka's eviction is eventual, but after some time/ops it should
            // stay within limits We can't strictly check exact size
            // without an entry count method which isn't in our trait but
            // we can check that NOT all 100 are there.
            tokio::time::sleep(Duration::from_millis(100u64)).await;

            let mut found = 0i32;
            for i in 0i32..100i32 {
                if reader.get(&format!("key{i}")).await.unwrap().is_some() {
                    found += 1i32;
                }
            }

            // Allow some slack for eventual eviction
            assert!(found <= 10i32 + 5i32, "Found too many items: {found}");
        }

        #[tokio::test]
        async fn tinylfu_should_protect_hot_key() {
            let (reader, writer) = Cache::<String, String>::builder()
                .max_capacity(10usize)
                .build()
                .unwrap();

            // Access a "hot" key many times
            for _ in 0i32..20i32 {
                writer.put("hot".to_owned(), "value".to_owned()).await.unwrap();
                let _: Option<String> =
                    reader.get(&"hot".to_owned()).await.unwrap();
            }

            // Perform a "scan" that exceeds capacity
            for i in 0i32..100i32 {
                writer
                    .put(format!("scan-{i}"), "val".to_owned())
                    .await
                    .unwrap();
            }

            // The "hot" key should still be present because of TinyLFU
            // (Moka's eviction is eventual, so we wait a bit)
            tokio::time::sleep(Duration::from_millis(100u64)).await;
            assert!(
                reader.get(&"hot".to_owned()).await.unwrap().is_some(),
                "Hot key was evicted by scan pollution!"
            );
        }
    }
}
