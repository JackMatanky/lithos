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

use std::{marker::PhantomData, time::Duration};

use async_trait::async_trait;

use crate::spi::{cache::Cache as CachePort, errors::CacheError};

/// In-memory cache using the `moka` library.
///
/// # Example
///
/// ```rust
/// use std::time::Duration;
///
/// use lithos_adapters::spi::cache::{Cache, MokaCache};
///
/// # tokio::runtime::Runtime::new().unwrap().block_on(async {
/// let cache = MokaCache::build()
///     .max_capacity(100)
///     .time_to_live(Duration::from_secs(60))
///     .new()
///     .unwrap();
///
/// cache.put("key".to_string(), "value".to_string()).await.unwrap();
/// let val = cache.get(&"key".to_string()).await.unwrap();
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
/// # use lithos_adapters::spi::cache::MokaCache;
/// # use lithos_adapters::spi::cache::Cache;
/// # tokio::runtime::Runtime::new().unwrap().block_on(async {
/// let cache = MokaCache::build().max_capacity(10).new().unwrap();
///
/// // Access a "hot" key many times
/// for _ in 0..20 {
///     cache.put("hot".to_string(), "value".to_string()).await.unwrap();
///     let _ = cache.get(&"hot".to_string()).await.unwrap();
/// }
///
/// // Perform a "scan" that exceeds capacity
/// for i in 0..100 {
///     cache.put(format!("scan-{}", i), "val".to_string()).await.unwrap();
/// }
///
/// // The "hot" key should still be present because of TinyLFU
/// // (Moka's eviction is eventual, so we wait a bit)
/// tokio::time::sleep(std::time::Duration::from_millis(100)).await;
/// assert!(cache.get(&"hot".to_string()).await.unwrap().is_some());
/// # });
/// ```
#[derive(Debug)]
pub struct Cache<K, V>
where
    K: Clone + Eq + std::hash::Hash + Send + Sync + std::fmt::Debug + 'static,
    V: Clone + Send + Sync + std::fmt::Debug + 'static,
{
    inner: moka::future::Cache<K, V>,
}

#[async_trait]
impl<K, V> CachePort<K, V> for Cache<K, V>
where
    K: Clone + Eq + std::hash::Hash + Send + Sync + std::fmt::Debug + 'static,
    V: Clone + Send + Sync + std::fmt::Debug + 'static,
{
    #[tracing::instrument(skip(self), level = "debug")]
    #[inline]
    async fn get(&self, key: &K) -> Result<Option<V>, CacheError> {
        let hit = self.inner.get(key).await;
        tracing::event!(
            tracing::Level::DEBUG,
            cache_layer = "memory",
            operation = "get",
            hit = hit.is_some()
        );
        Ok(hit)
    }

    #[tracing::instrument(skip(self, value), level = "debug")]
    #[inline]
    async fn put(&self, key: K, value: V) -> Result<(), CacheError> {
        self.inner.insert(key, value).await;
        tracing::event!(
            tracing::Level::DEBUG,
            cache_layer = "memory",
            operation = "put"
        );
        Ok(())
    }

    #[tracing::instrument(skip(self), level = "debug")]
    #[inline]
    async fn delete(&self, key: &K) -> Result<bool, CacheError> {
        let existed = self.inner.remove(key).await.is_some();
        tracing::event!(
            tracing::Level::DEBUG,
            cache_layer = "memory",
            operation = "delete",
            existed = existed
        );
        Ok(existed)
    }

    #[inline]
    async fn invalidate(&self, key: &K) -> Result<bool, CacheError> {
        self.delete(key).await
    }
}

impl<K, V> Cache<K, V>
where
    K: Clone + Eq + std::hash::Hash + Send + Sync + std::fmt::Debug + 'static,
    V: Clone + Send + Sync + std::fmt::Debug + 'static,
{
    /// Create a new builder for `MokaCache`.
    #[inline]
    #[must_use]
    pub fn build() -> Builder<K, V> {
        Builder::default()
    }
}

/// Builder for `MokaCache`.
#[derive(Debug)]
pub struct Builder<K, V>
where
    K: Clone + Eq + std::hash::Hash + Send + Sync + std::fmt::Debug + 'static,
    V: Clone + Send + Sync + std::fmt::Debug + 'static,
{
    max_capacity: usize,
    time_to_live: Option<Duration>,
    time_to_idle: Option<Duration>,
    _k: PhantomData<K>,
    _v: PhantomData<V>,
}

impl<K, V> Default for Builder<K, V>
where
    K: Clone + Eq + std::hash::Hash + Send + Sync + std::fmt::Debug + 'static,
    V: Clone + Send + Sync + std::fmt::Debug + 'static,
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
    K: Clone + Eq + std::hash::Hash + Send + Sync + std::fmt::Debug + 'static,
    V: Clone + Send + Sync + std::fmt::Debug + 'static,
{
    /// Set maximum capacity.
    #[inline]
    pub fn max_capacity(&mut self, capacity: usize) -> &mut Self {
        self.max_capacity = capacity;
        self
    }

    /// Build the `MokaCache`.
    ///
    /// # Errors
    ///
    /// Returns `CacheError::BackendError` if the configuration is invalid
    /// (e.g., `max_capacity` is 0).
    #[inline]
    #[expect(
        clippy::new_ret_no_self,
        reason = "The name 'new' is used as the builder terminal method here \
                  to indicate the final construction of the actual Cache \
                  instance, as per project preference."
    )]
    pub fn new(&self) -> Result<Cache<K, V>, CacheError> {
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

        let inner = builder.build();

        Ok(Cache {
            inner,
        })
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
mod tests {
    use super::*;

    #[test]
    fn moka_config_should_return_builder_instance() {
        let _builder: Builder<String, String> =
            Cache::<String, String>::build();
    }

    #[test]
    fn moka_config_should_allow_configuring_max_capacity() {
        let mut builder = Cache::<String, String>::build();
        let _: &mut Builder<String, String> = builder.max_capacity(100usize);
    }

    #[test]
    fn moka_config_should_allow_configuring_ttl() {
        let mut builder = Cache::<String, String>::build();
        let _: &mut Builder<String, String> =
            builder.time_to_live(Duration::from_secs(10u64));
    }

    #[test]
    fn moka_config_should_allow_configuring_tti() {
        let mut builder = Cache::<String, String>::build();
        let _: &mut Builder<String, String> =
            builder.time_to_idle(Duration::from_secs(10u64));
    }

    #[test]
    fn moka_config_should_build_cache_instance() {
        let cache = Cache::<String, String>::build().new();
        let _: Cache<String, String> = cache.expect("Failed to build cache");
    }

    #[tokio::test]
    async fn moka_trait_should_get_none_from_empty_cache() {
        let cache = Cache::<String, String>::build().new().unwrap();
        let result = cache.get(&"key".to_owned()).await.unwrap();
        assert!(result.is_none());
    }

    #[tokio::test]
    async fn moka_trait_should_put_and_get_value() {
        let cache = Cache::<String, String>::build().new().unwrap();
        cache.put("key".to_owned(), "value".to_owned()).await.unwrap();
        let result = cache.get(&"key".to_owned()).await.unwrap();
        assert_eq!(result, Some("value".to_owned()));
    }

    #[tokio::test]
    async fn moka_trait_should_delete_value() {
        let cache = Cache::<String, String>::build().new().unwrap();
        cache.put("key".to_owned(), "value".to_owned()).await.unwrap();
        let existed = cache.delete(&"key".to_owned()).await.unwrap();
        assert!(existed);
        let result = cache.get(&"key".to_owned()).await.unwrap();
        assert!(result.is_none());

        let existed_again = cache.delete(&"key".to_owned()).await.unwrap();
        assert!(!existed_again);
    }

    #[tokio::test]
    #[tracing_test::traced_test]
    async fn moka_tracing_should_emit_events_on_get() {
        let cache = Cache::<String, String>::build().new().unwrap();
        let _: Option<String> = cache.get(&"key".to_owned()).await.unwrap();

        assert!(logs_contain("operation=\"get\""));
        assert!(logs_contain("hit=false"));
    }

    #[tokio::test]
    #[tracing_test::traced_test]
    async fn moka_tracing_should_emit_events_on_put() {
        let cache = Cache::<String, String>::build().new().unwrap();
        cache.put("key".to_owned(), "value".to_owned()).await.unwrap();

        assert!(logs_contain("operation=\"put\""));
    }

    #[tokio::test]
    #[tracing_test::traced_test]
    async fn moka_tracing_should_emit_events_on_delete() {
        let cache = Cache::<String, String>::build().new().unwrap();
        cache.put("key".to_owned(), "value".to_owned()).await.unwrap();
        let _: bool = cache.delete(&"key".to_owned()).await.unwrap();

        assert!(logs_contain("operation=\"delete\""));
        assert!(logs_contain("existed=true"));
    }

    #[tokio::test]
    async fn moka_eviction_should_respect_ttl() {
        let cache = Cache::<String, String>::build()
            .time_to_live(Duration::from_millis(50u64))
            .new()
            .unwrap();

        cache.put("key".to_owned(), "value".to_owned()).await.unwrap();
        assert_eq!(
            cache.get(&"key".to_owned()).await.unwrap(),
            Some("value".to_owned())
        );

        tokio::time::sleep(Duration::from_millis(100u64)).await;
        assert_eq!(cache.get(&"key".to_owned()).await.unwrap(), None);
    }

    #[tokio::test]
    async fn moka_eviction_should_respect_tti() {
        let cache = Cache::<String, String>::build()
            .time_to_idle(Duration::from_millis(100u64))
            .new()
            .unwrap();

        cache.put("key".to_owned(), "value".to_owned()).await.unwrap();

        tokio::time::sleep(Duration::from_millis(60u64)).await;
        // Access it to reset TTI
        assert_eq!(
            cache.get(&"key".to_owned()).await.unwrap(),
            Some("value".to_owned())
        );

        tokio::time::sleep(Duration::from_millis(60u64)).await;
        // Should still be there because TTI was reset at 60ms
        assert_eq!(
            cache.get(&"key".to_owned()).await.unwrap(),
            Some("value".to_owned())
        );

        tokio::time::sleep(Duration::from_millis(150u64)).await;
        // Now it should be gone
        assert_eq!(cache.get(&"key".to_owned()).await.unwrap(), None);
    }

    #[tokio::test]
    async fn moka_eviction_should_respect_max_capacity() {
        let cache = Cache::<String, String>::build()
            .max_capacity(10usize)
            .new()
            .unwrap();

        for i in 0i32..100i32 {
            cache.put(format!("key{i}"), format!("value{i}")).await.unwrap();
        }

        // Moka's eviction is eventual, but after some time/ops it should stay
        // within limits We can't strictly check exact size without an
        // entry count method which isn't in our trait but we can check
        // that NOT all 100 are there.
        tokio::time::sleep(Duration::from_millis(100u64)).await;

        let mut found = 0i32;
        for i in 0i32..100i32 {
            if cache.get(&format!("key{i}")).await.unwrap().is_some() {
                found += 1i32;
            }
        }

        // Allow some slack for eventual eviction
        assert!(found <= 10i32 + 5i32, "Found too many items: {found}");
    }

    #[test]
    #[expect(
        clippy::panic,
        reason = "Panic is used in tests to fail fast when expectations are \
                  not met."
    )]
    fn moka_config_should_return_error_for_zero_capacity() {
        let result =
            Cache::<String, String>::build().max_capacity(0usize).new();

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
