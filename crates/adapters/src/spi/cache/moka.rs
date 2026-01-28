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

use std::{
    marker::PhantomData,
    sync::{Arc, OnceLock},
    time::Duration,
};

use async_trait::async_trait;

use crate::spi::{
    cache::{CacheReader, CacheWriter},
    errors::CacheError,
};

/// Builder for Moka cache.
///
/// # Examples
///
/// ```rust
/// use std::time::Duration;
///
/// use lithos_adapters::spi::cache::MokaBuilder;
///
/// let mut builder = MokaBuilder::<String, String>::new();
/// builder.max_capacity(100).time_to_live(Duration::from_secs(60));
///
/// let reader = builder.reader().unwrap();
/// let writer = builder.writer().unwrap();
/// ```
///
/// For fail-fast validation, use [`Builder::try_max_capacity`].
#[derive(Debug, Clone)]
pub struct Builder<K, V>
where
    K: Clone + Eq + std::hash::Hash + Send + Sync + 'static,
    V: Clone + Send + Sync + 'static,
{
    max_capacity: usize,
    shared_inner: Arc<OnceLock<MokaInner<K, V>>>,
    time_to_idle: Option<Duration>,
    time_to_live: Option<Duration>,
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
            shared_inner: Arc::new(OnceLock::new()),
            time_to_idle: None,
            time_to_live: None,
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
    /// Set maximum capacity.
    #[inline]
    pub fn max_capacity(&mut self, capacity: usize) -> &mut Self {
        if let Err(e) = Self::validate_capacity(capacity) {
            tracing::warn!(?e, "Invalid capacity provided to max_capacity");
        }
        self.max_capacity = capacity;
        self.reset_state();
        self
    }

    /// Create a new builder with default settings.
    #[inline]
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Reset the internal state, forcing a fresh cache to be created on next
    /// access.
    #[inline]
    fn reset_state(&mut self) {
        self.shared_inner = Arc::new(OnceLock::new());
    }

    /// Set time to idle.
    #[inline]
    pub fn time_to_idle(&mut self, duration: Duration) -> &mut Self {
        self.time_to_idle = Some(duration);
        self.reset_state();
        self
    }

    /// Set time to live.
    #[inline]
    pub fn time_to_live(&mut self, duration: Duration) -> &mut Self {
        self.time_to_live = Some(duration);
        self.reset_state();
        self
    }

    /// Set maximum capacity with fail-fast validation.
    ///
    /// # Errors
    /// Returns `CacheError::BackendError` if the capacity is invalid.
    #[inline]
    pub fn try_max_capacity(
        &mut self,
        capacity: usize,
    ) -> Result<&mut Self, CacheError> {
        Self::validate_capacity(capacity)?;
        self.max_capacity = capacity;
        self.reset_state();
        Ok(self)
    }

    /// Validate the given capacity and return the converted value.
    #[inline]
    fn validate_capacity(capacity: usize) -> Result<u64, CacheError> {
        if capacity == 0 {
            return Err(CacheError::BackendError {
                backend: "moka",
                message: "max_capacity must be greater than 0".into(),
            });
        }
        capacity.try_into().map_err(|e| CacheError::BackendError {
            backend: "moka",
            message: format!("Invalid max_capacity: {e}").into(),
        })
    }
}

impl<K, V> Builder<K, V>
where
    K: std::fmt::Debug + Clone + Eq + std::hash::Hash + Send + Sync + 'static,
    V: Clone + Send + Sync + 'static,
{
    /// Internal helper to obtain the shared inner state.
    fn get_or_init_inner(&self) -> Result<MokaInner<K, V>, CacheError> {
        if let Some(inner) = self.shared_inner.get() {
            return Ok(inner.clone());
        }

        let inner = self.inner_builder()?;
        // Try to set it. If someone else set it first, that's fine, we'll
        // return whatever is there.
        _ = self.shared_inner.set(inner.clone());
        Ok(inner)
    }

    /// Internal helper to build the Inner state.
    #[inline]
    fn inner_builder(&self) -> Result<MokaInner<K, V>, CacheError> {
        let capacity = Self::validate_capacity(self.max_capacity)?;

        let mut builder = moka::future::Cache::builder().max_capacity(capacity);

        if let Some(ttl) = self.time_to_live {
            builder = builder.time_to_live(ttl);
        }

        if let Some(tti) = self.time_to_idle {
            builder = builder.time_to_idle(tti);
        }

        Ok(builder.build())
    }

    /// Build a Reader handle.
    ///
    /// Creates a new cache (if not already initialized by this builder) and
    /// returns only a Reader handle.
    ///
    /// # Errors
    ///
    /// Returns `CacheError::BackendError` if the configuration is invalid.
    #[inline]
    pub fn reader(&self) -> Result<Reader<K, V>, CacheError> {
        let cache = self.get_or_init_inner()?;
        Ok(Reader {
            cache,
        })
    }

    /// Build a Writer handle.
    ///
    /// Creates a new cache (if not already initialized by this builder) and
    /// returns only a Writer handle.
    ///
    /// # Errors
    ///
    /// Returns `CacheError::BackendError` if the configuration is invalid.
    #[inline]
    pub fn writer(&self) -> Result<Writer<K, V>, CacheError> {
        let cache = self.get_or_init_inner()?;
        Ok(Writer {
            cache,
        })
    }
}

/// Read-only handle for Moka cache.
///
/// This handle provides read-only access to the cache following CQRS
/// principles.
///
/// # Examples
///
/// ```rust
/// # use lithos_adapters::spi::cache::{MokaBuilder, CacheReader};
/// # tokio::runtime::Runtime::new().unwrap().block_on(async {
/// let reader = MokaBuilder::<String, String>::new().reader().unwrap();
/// let value = reader.get(&"key".to_string()).await.unwrap();
/// assert!(value.is_none());
/// # });
/// ```
#[derive(Debug, Clone)]
pub struct Reader<K, V>
where
    K: Clone + Eq + std::hash::Hash + Send + Sync + 'static,
    V: Clone + Send + Sync + 'static,
{
    cache: moka::future::Cache<K, V>,
}

#[async_trait]
impl<K, V> CacheReader<K, V> for Reader<K, V>
where
    K: Clone + Eq + std::hash::Hash + Send + Sync + 'static,
    V: Clone + Send + Sync + 'static,
{
    #[tracing::instrument(skip(self, key), level = "debug")]
    #[inline]
    async fn get(&self, key: &K) -> Result<Option<V>, CacheError> {
        let hit = self.cache.get(key).await;
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
        // `contains_key` is synchronous and may be approximate under eviction
        // pressure; use `get` if you need a definitive value check.
        let exists = self.cache.contains_key(key);
        tracing::event!(
            tracing::Level::DEBUG,
            cache_layer = "memory",
            operation = "has",
            exists = exists
        );
        Ok(exists)
    }

    #[tracing::instrument(skip(self), level = "debug")]
    #[inline]
    async fn keys(&self) -> Result<Vec<K>, CacheError> {
        // This may hold internal locks and clone all keys; prefer targeted
        // lookups for large caches.
        let keys: Vec<K> =
            self.cache.iter().map(|(key, _)| (*key).clone()).collect();
        tracing::event!(
            tracing::Level::DEBUG,
            cache_layer = "memory",
            operation = "keys",
            count = keys.len()
        );
        Ok(keys)
    }
}

/// Write-only handle for Moka cache.
///
/// This handle provides write-only access to the cache following CQRS
/// principles.
///
/// # Examples
///
/// ```rust
/// # use lithos_adapters::spi::cache::{MokaBuilder, CacheWriter};
/// # tokio::runtime::Runtime::new().unwrap().block_on(async {
/// let writer = MokaBuilder::<String, String>::new().writer().unwrap();
/// writer.put("key".to_string(), "value".to_string()).await.unwrap();
/// # });
/// ```
#[derive(Debug, Clone)]
pub struct Writer<K, V>
where
    K: Clone + Eq + std::hash::Hash + Send + Sync + 'static,
    V: Clone + Send + Sync + 'static,
{
    cache: moka::future::Cache<K, V>,
}

#[async_trait]
impl<K, V> CacheWriter<K, V> for Writer<K, V>
where
    K: Clone + Eq + std::hash::Hash + Send + Sync + 'static,
    V: Clone + Send + Sync + 'static,
{
    #[tracing::instrument(skip(self), level = "debug")]
    #[inline]
    async fn clear(&self) -> Result<(), CacheError> {
        self.cache.invalidate_all();
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
        let existed = self.cache.remove(key).await.is_some();
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
        self.cache.insert(key, value).await;
        tracing::event!(
            tracing::Level::DEBUG,
            cache_layer = "memory",
            operation = "put"
        );
        Ok(())
    }
}

/// Type alias for the Inner state of Moka cache.
pub(crate) type MokaInner<K, V> = moka::future::Cache<K, V>;

#[cfg(test)]
mod tests {
    use super::*;

    mod fixtures {
        use super::*;

        pub fn builder() -> Builder<String, String> {
            Builder::default()
        }

        pub fn reader(capacity: usize) -> Reader<String, String> {
            Builder::default().max_capacity(capacity).reader().unwrap()
        }

        pub fn writer(capacity: usize) -> Writer<String, String> {
            Builder::default().max_capacity(capacity).writer().unwrap()
        }
    }

    mod builder {
        use super::{fixtures::*, *};

        /// [5.4-U-03] P0: Test independent handle creation.
        #[test]
        fn builds_reader_independently() {
            // GIVEN: a Moka builder
            let mut builder = builder();

            // WHEN: the reader is built independently
            let result = builder.max_capacity(50).reader();

            // THEN: the handle is correct and independent
            assert!(result.is_ok());
            let reader = result.unwrap();
            let _: Reader<String, String> = reader;
        }

        /// [5.4-U-03] P0: Test independent handle creation.
        #[test]
        fn builds_writer_independently() {
            // GIVEN: a Moka builder
            let mut builder = builder();

            // WHEN: the writer is built independently
            let result = builder.max_capacity(50).writer();

            // THEN: the handle is correct and independent
            assert!(result.is_ok());
            let writer = result.unwrap();
            let _: Writer<String, String> = writer;
        }
    }

    mod initialization {
        use super::{fixtures::*, *};

        /// [5.4-U-03] P2: Test builder defaults.
        #[test]
        fn should_return_builder_instance() {
            // GIVEN: nothing
            // WHEN: default is called
            let _builder = builder();
            // THEN: it succeeds
        }

        /// [5.4-U-03] P2: Test max capacity configuration.
        #[test]
        fn should_allow_configuring_max_capacity() {
            // GIVEN: a builder
            let mut builder = builder();
            // WHEN: capacity is set
            let _: &mut Builder<String, String> =
                builder.max_capacity(100usize);
            // THEN: it succeeds
        }

        /// [5.4-U-03] P2: Test TTL configuration.
        #[test]
        fn should_allow_configuring_ttl() {
            // GIVEN: a builder
            let mut builder = builder();
            // WHEN: ttl is set
            let _: &mut Builder<String, String> =
                builder.time_to_live(Duration::from_secs(10u64));
            // THEN: it succeeds
        }

        /// [5.4-U-03] P2: Test TTI configuration.
        #[test]
        fn should_allow_configuring_tti() {
            // GIVEN: a builder
            let mut builder = builder();
            // WHEN: tti is set
            let _: &mut Builder<String, String> =
                builder.time_to_idle(Duration::from_secs(10u64));
            // THEN: it succeeds
        }

        /// [5.4-U-03] P1: Test successful handle production.
        #[test]
        fn should_build_cache_instance() {
            // GIVEN: a builder
            let builder = builder();
            // WHEN: reader is called
            let result = builder.reader();
            // THEN: it produces handle
            let _reader = result.expect("Failed to build cache");
        }

        /// [5.4-U-03] P1: Edge Case - zero capacity.
        #[test]
        fn should_return_error_for_zero_capacity() {
            // GIVEN: a builder with zero capacity
            let mut builder = builder();
            let result = builder.max_capacity(0usize).reader();

            // WHEN: building is attempted
            // THEN: a BackendError is returned
            assert!(result.is_err());
            let err = result.unwrap_err();
            assert!(matches!(err, CacheError::BackendError {
                backend: "moka",
                ..
            }));
            if let CacheError::BackendError {
                message,
                ..
            } = err
            {
                assert!(message.contains("capacity"));
            }
        }
    }

    mod core_ops {
        use super::{fixtures::*, *};

        /// [5.4-U-08] P0: Test basic read operations.
        #[tokio::test]
        async fn should_get_none_from_empty_cache() {
            // GIVEN: an empty cache
            let reader = reader(50);
            // WHEN: retrieving a missing key
            let result = reader.get(&"key".to_owned()).await.unwrap();
            // THEN: None is returned
            assert!(result.is_none());
        }

        /// [5.4-U-08] P0: Test basic write/read operations.
        #[tokio::test]
        async fn should_put_and_get_value() {
            // GIVEN: a cache and a value
            let mut builder = builder();
            builder.max_capacity(50);
            let reader = builder.reader().unwrap();
            let writer = builder.writer().unwrap();

            // WHEN: putting and then getting the value
            writer.put("key".to_owned(), "value".to_owned()).await.unwrap();
            let result = reader.get(&"key".to_owned()).await.unwrap();
            // THEN: the retrieved value matches
            assert_eq!(result, Some("value".to_owned()));
        }

        /// [5.4-U-08] P0: Test delete operation.
        #[tokio::test]
        async fn should_delete_value() {
            // GIVEN: a cache with a value
            let mut builder = builder();
            builder.max_capacity(50);
            let reader = builder.reader().unwrap();
            let writer = builder.writer().unwrap();

            writer.put("key".to_owned(), "value".to_owned()).await.unwrap();

            // WHEN: deleting the value
            let existed = writer.delete(&"key".to_owned()).await.unwrap();
            // THEN: delete reports success and the value is gone
            assert!(existed);
            let result = reader.get(&"key".to_owned()).await.unwrap();
            assert!(result.is_none());

            let existed_again = writer.delete(&"key".to_owned()).await.unwrap();
            assert!(!existed_again);
        }

        /// [5.4-U-08] P1: Test existence check.
        #[tokio::test]
        async fn should_check_has() {
            // GIVEN: a cache
            let mut builder = builder();
            builder.max_capacity(50);
            let reader = builder.reader().unwrap();
            let writer = builder.writer().unwrap();

            assert!(!reader.has(&"key".to_owned()).await.unwrap());

            // WHEN: putting a value
            writer.put("key".to_owned(), "value".to_owned()).await.unwrap();
            // THEN: has reports existence
            assert!(reader.has(&"key".to_owned()).await.unwrap());

            writer.delete(&"key".to_owned()).await.unwrap();
            assert!(!reader.has(&"key".to_owned()).await.unwrap());
        }

        /// [5.4-U-08] P1: Test clear operation.
        #[tokio::test]
        async fn should_clear_all_entries() {
            // GIVEN: a cache with multiple values
            let mut builder = builder();
            builder.max_capacity(50);
            let reader = builder.reader().unwrap();
            let writer = builder.writer().unwrap();

            writer.put("k1".to_owned(), "v1".to_owned()).await.unwrap();
            writer.put("k2".to_owned(), "v2".to_owned()).await.unwrap();

            // WHEN: clear is called
            writer.clear().await.unwrap();

            // THEN: all entries are gone
            assert!(!reader.has(&"k1".to_owned()).await.unwrap());
            assert!(!reader.has(&"k2".to_owned()).await.unwrap());
        }

        /// [5.4-U-08] P1: Test key retrieval.
        #[tokio::test]
        async fn should_return_all_keys() {
            // GIVEN: a cache with values
            let mut builder = builder();
            builder.max_capacity(50);
            let reader = builder.reader().unwrap();
            let writer = builder.writer().unwrap();

            writer.put("k1".to_owned(), "v1".to_owned()).await.unwrap();
            writer.put("k2".to_owned(), "v2".to_owned()).await.unwrap();

            // WHEN: keys are retrieved
            let mut keys = reader.keys().await.unwrap();
            keys.sort();

            // THEN: all expected keys are present
            assert_eq!(keys.len(), 2);
            assert!(keys.contains(&"k1".to_owned()));
            assert!(keys.contains(&"k2".to_owned()));
        }
    }

    mod observability {
        use tracing_test::traced_test;

        use super::{fixtures::*, *};

        /// [5.4-U-11] P1: Test tracing events for GET.
        #[tokio::test]
        #[traced_test]
        async fn should_emit_events_on_get() {
            // GIVEN: a cache
            let reader = reader(50);
            // WHEN: get is called
            let _: Option<String> =
                reader.get(&"key".to_owned()).await.unwrap();

            // THEN: tracing events are emitted
            assert!(logs_contain("operation=\"get\""));
            assert!(logs_contain("hit=false"));
        }

        /// [5.4-U-11] P1: Test tracing events for PUT.
        #[tokio::test]
        #[traced_test]
        async fn should_emit_events_on_put() {
            // GIVEN: a cache
            let writer = writer(50);
            // WHEN: put is called
            writer.put("key".to_owned(), "value".to_owned()).await.unwrap();

            // THEN: tracing event is emitted
            assert!(logs_contain("operation=\"put\""));
        }

        /// [5.4-U-11] P1: Test tracing events for DELETE.
        #[tokio::test]
        #[traced_test]
        async fn should_emit_events_on_delete() {
            // GIVEN: a cache with a value
            let mut builder = builder();
            builder.max_capacity(50);
            let _reader = builder.reader().unwrap();
            let writer = builder.writer().unwrap();

            writer.put("key".to_owned(), "value".to_owned()).await.unwrap();
            // WHEN: delete is called
            let _: bool = writer.delete(&"key".to_owned()).await.unwrap();

            // THEN: tracing event is emitted
            assert!(logs_contain("operation=\"delete\""));
            assert!(logs_contain("existed=true"));
        }

        /// [5.4-U-11] P1: Test tracing events for INVALIDATE.
        #[tokio::test]
        #[traced_test]
        async fn should_emit_events_on_invalidate() {
            // GIVEN: a cache with a value
            let mut builder = builder();
            builder.max_capacity(50);
            let reader = builder.reader().unwrap();
            let writer = builder.writer().unwrap();

            writer.put("key".to_owned(), "value".to_owned()).await.unwrap();

            // WHEN: invalidate is called
            let existed = writer.invalidate(&"key".to_owned()).await.unwrap();

            // THEN: tracing event is emitted and value is gone
            assert!(existed);
            assert!(!reader.has(&"key".to_owned()).await.unwrap());
            assert!(logs_contain("operation=\"invalidate\""));
            assert!(logs_contain("existed=true"));
        }
    }

    mod eviction {
        use super::{fixtures::*, *};

        /// [5.4-U-08] P0: Test TTL-based eviction.
        #[tokio::test]
        async fn should_respect_ttl() {
            // GIVEN: a cache with a short TTL
            let mut builder = Builder::<String, String>::default();
            builder.time_to_live(Duration::from_millis(50));
            let reader = builder.reader().unwrap();
            let writer = builder.writer().unwrap();

            // WHEN: a value is put
            writer.put("key".to_owned(), "value".to_owned()).await.unwrap();
            assert_eq!(
                reader.get(&"key".to_owned()).await.unwrap(),
                Some("value".to_owned())
            );

            // AND: time passes past TTL
            tokio::time::sleep(Duration::from_millis(200)).await;
            // Trigger maintenance
            let _: bool = reader.has(&"nonexistent".to_owned()).await.unwrap();

            // THEN: the value is gone
            assert_eq!(reader.get(&"key".to_owned()).await.unwrap(), None);
        }

        /// [5.4-U-08] P0: Test TTI-based eviction.
        #[tokio::test]
        async fn should_respect_tti() {
            // GIVEN: a cache with short TTI
            let mut builder = Builder::<String, String>::default();
            builder.time_to_idle(Duration::from_millis(100));
            let reader = builder.reader().unwrap();
            let writer = builder.writer().unwrap();

            writer.put("key".to_owned(), "value".to_owned()).await.unwrap();

            // WHEN: time passes but we access before TTI expires
            tokio::time::sleep(Duration::from_millis(60)).await;
            // Access it to reset TTI
            assert_eq!(
                reader.get(&"key".to_owned()).await.unwrap(),
                Some("value".to_owned())
            );

            tokio::time::sleep(Duration::from_millis(60)).await;
            // THEN: it should still be there because access reset TTI
            assert_eq!(
                reader.get(&"key".to_owned()).await.unwrap(),
                Some("value".to_owned())
            );

            // WHEN: time passes past TTI without access
            tokio::time::sleep(Duration::from_millis(300)).await;
            // Trigger maintenance
            let _: bool = reader.has(&"nonexistent".to_owned()).await.unwrap();

            // THEN: it is evicted
            assert_eq!(reader.get(&"key".to_owned()).await.unwrap(), None);
        }

        /// [5.4-U-08] P1: Test capacity-based eviction.
        #[tokio::test]
        async fn should_respect_max_capacity() {
            // GIVEN: a cache with small capacity
            let mut builder = builder();
            builder.max_capacity(10);
            let reader = builder.reader().unwrap();
            let writer = builder.writer().unwrap();

            // WHEN: filling past capacity
            for i in 0i32..100i32 {
                writer
                    .put(format!("key{i}"), format!("value{i}"))
                    .await
                    .unwrap();
            }

            // AND: time passes to allow eventual eviction
            tokio::time::sleep(Duration::from_millis(200)).await;
            // Trigger maintenance
            let _: bool = reader.has(&"nonexistent".to_owned()).await.unwrap();

            // THEN: found count is near capacity
            let found = count_entries(&reader, 0i32..100i32).await;
            assert!(found <= 10i32 + 5i32, "Found too many items: {found}");
        }

        async fn count_entries(
            reader: &Reader<String, String>,
            range: std::ops::Range<i32>,
        ) -> i32 {
            let mut count: i32 = 0;
            for i in range {
                if reader.get(&format!("key{i}")).await.unwrap().is_some() {
                    count = count.saturating_add(1);
                }
            }
            count
        }

        /// [5.4-U-08] P1: Test `TinyLFU` protection.
        #[tokio::test]
        async fn tinylfu_should_protect_hot_key() {
            // GIVEN: a cache with small capacity
            let mut builder = builder();
            builder.max_capacity(10);
            let reader = builder.reader().unwrap();
            let writer = builder.writer().unwrap();

            // WHEN: a key is accessed frequently
            for _ in 0i32..20i32 {
                writer.put("hot".to_owned(), "value".to_owned()).await.unwrap();
                let _: Option<String> =
                    reader.get(&"hot".to_owned()).await.unwrap();
            }

            // AND: a large scan exceeds capacity
            for i in 0i32..100i32 {
                writer
                    .put(format!("scan-{i}"), "val".to_owned())
                    .await
                    .unwrap();
            }

            tokio::time::sleep(Duration::from_millis(100)).await;

            // THEN: the "hot" key survives eviction due to TinyLFU
            assert!(
                reader.get(&"hot".to_owned()).await.unwrap().is_some(),
                "Hot key was evicted by scan pollution!"
            );
        }
    }
}
