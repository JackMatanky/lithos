//! Generic caching SPI for adapter-layer use.
//!
//! This module defines the `Cache` trait and its associated error types,
//! following hexagonal architecture principles where the cache serves as a
//! Service Provider Interface (SPI) for other adapters.

#![cfg_attr(
    test,
    expect(
        clippy::disallowed_methods,
        reason = "Mockall macro expansion and test assertions require \
                  internal unwrap/expect calls. Unavoidable due to library \
                  design and test brevity. Inner attribute scoped to test \
                  ensures expectation is not unfulfilled in standard builds."
    )
)]

use async_trait::async_trait;

use crate::spi::errors::CacheError;

/// Generic caching SPI for adapter-layer use.
///
/// # Implementations
/// - `MokaCache`: In-memory cache using the `moka` library.
/// - `RedbCache`: Persistent cache using the `redb` KV store. Note: For
///   persistent caches, values MUST also implement `rkyv` traits:
///   `rkyv::Archive + rkyv::Serialize + rkyv::Deserialize`.
/// - `Coordinator`: A multi-tier cache combining memory and disk storage.
///
/// # Example
///
/// ```rust
/// # use async_trait::async_trait;
/// # use lithos_adapters::spi::cache::Cache;
/// # use lithos_adapters::spi::errors::CacheError;
/// # struct MemoryCache;
/// # #[async_trait]
/// # impl Cache<String, String> for MemoryCache {
/// #     async fn delete(&self, _k: &String) -> Result<bool, CacheError> { Ok(false) }
/// #     async fn get(&self, _k: &String) -> Result<Option<String>, CacheError> { Ok(None) }
/// #     async fn invalidate(&self, _k: &String) -> Result<bool, CacheError> { Ok(false) }
/// #     async fn put(&self, _k: String, _v: String) -> Result<(), CacheError> { Ok(()) }
/// # }
/// # tokio::runtime::Runtime::new().unwrap().block_on(async {
/// let cache = MemoryCache;
/// let result = cache.get(&"key".to_string()).await?;
/// assert!(result.is_none());
/// # Ok::<(), CacheError>(())
/// # }).unwrap();
/// ```
#[async_trait]
#[cfg_attr(test, mockall::automock)]
pub trait Cache<K, V>
where
    K: Clone + Eq + std::hash::Hash + Send + Sync + 'static,
    V: Clone + Send + Sync + 'static,
{
    /// Remove entry from cache.
    ///
    /// Returns `true` if the entry existed and was removed.
    ///
    /// # Errors
    /// Returns `CacheError` if the underlying storage fails.
    async fn delete(&self, key: &K) -> Result<bool, CacheError>;

    /// Retrieve value by key.
    ///
    /// # Errors
    /// Returns `CacheError` if the underlying storage fails.
    async fn get(&self, key: &K) -> Result<Option<V>, CacheError>;

    /// Alias for `delete` (cache-specific terminology).
    ///
    /// Returns `true` if the entry existed and was removed.
    ///
    /// # Errors
    /// Returns `CacheError` if the underlying storage fails.
    async fn invalidate(&self, key: &K) -> Result<bool, CacheError>;

    /// Store key-value pair.
    ///
    /// # Errors
    /// Returns `CacheError` if the underlying storage fails.
    async fn put(&self, key: K, value: V) -> Result<(), CacheError>;
}

#[cfg(test)]
mod tests {
    use super::*;

    struct Dummy;

    #[async_trait]
    impl Cache<String, String> for Dummy {
        async fn delete(&self, _key: &String) -> Result<bool, CacheError> {
            Ok(false)
        }

        async fn get(
            &self,
            _key: &String,
        ) -> Result<Option<String>, CacheError> {
            Ok(None)
        }

        async fn invalidate(&self, _key: &String) -> Result<bool, CacheError> {
            Ok(false)
        }

        async fn put(
            &self,
            _key: String,
            _value: String,
        ) -> Result<(), CacheError> {
            Ok(())
        }
    }

    // [5.1-U-02] Cache Trait Existence
    #[test]
    fn should_find_cache_trait() {
        fn assert_is_cache<T: Cache<String, String>>() {}
        assert_is_cache::<Dummy>();
    }

    // [5.1-U-08] Cache::put Method
    #[test]
    fn should_have_put_method() {
        // Verified by Dummy implementation
    }

    // [5.1-U-09] Cache::delete Method
    #[test]
    fn should_have_delete_method() {
        // Verified by Dummy implementation
    }

    // [5.1-U-10] Cache::invalidate Method
    #[test]
    fn should_have_invalidate_method() {
        // Verified by Dummy implementation
    }

    // [5.1-U-11] Cache Trait Bounds
    #[test]
    fn should_require_proper_trait_bounds() {
        fn assert_bounds<K, V, C>()
        where
            K: Clone + Eq + std::hash::Hash + Send + Sync + 'static,
            V: Clone + Send + Sync + 'static,
            C: Cache<K, V>,
        {
        }
        assert_bounds::<String, String, Dummy>();
    }

    // [5.1-U-03] MockCache Existence
    #[test]
    fn should_find_mock_cache() {
        let _mock = MockCache::<String, String>::new();
    }

    // [5.1-U-12] MockCache Expectations
    #[tokio::test]
    async fn mock_should_allow_get_expectation() {
        let mut mock = MockCache::<String, String>::new();
        mock.expect_get().returning(|_| Box::pin(async { Ok(None) }));
        let result = mock.get(&"key".to_owned()).await;
        assert!(result.is_ok(), "Mock get should return Ok");
    }

    #[tokio::test]
    async fn mock_should_allow_put_expectation() {
        let mut mock = MockCache::<String, String>::new();
        mock.expect_put().returning(|_, _| Box::pin(async { Ok(()) }));
        let result = mock.put("key".to_owned(), "value".to_owned()).await;
        assert!(result.is_ok(), "Mock put should return Ok");
    }

    #[tokio::test]
    async fn mock_should_allow_delete_expectation() {
        let mut mock = MockCache::<String, String>::new();
        mock.expect_delete().returning(|_| Box::pin(async { Ok(true) }));
        let result = mock.delete(&"key".to_owned()).await;
        assert!(result.is_ok(), "Mock delete should return Ok");
    }

    #[tokio::test]
    async fn mock_should_allow_invalidate_expectation() {
        let mut mock = MockCache::<String, String>::new();
        mock.expect_invalidate().returning(|_| Box::pin(async { Ok(true) }));
        let result = mock.invalidate(&"key".to_owned()).await;
        assert!(result.is_ok(), "Mock invalidate should return Ok");
    }

    // [5.1-U-13] Cache Contract Behavior
    #[tokio::test]
    async fn get_should_return_none_for_missing_key() {
        let mut mock = MockCache::<String, String>::new();
        mock.expect_get()
            .with(mockall::predicate::eq("missing".to_owned()))
            .returning(|_| Box::pin(async { Ok(None) }));

        let result = mock.get(&"missing".to_owned()).await;
        assert!(
            matches!(result, Ok(None)),
            "Expected Ok(None), got {result:?}"
        );
    }

    #[tokio::test]
    #[expect(
        clippy::panic,
        reason = "Test assertion requires explicit failure for unrecoverable \
                  contract violation. Unavoidable in complex match patterns. \
                  Panic with descriptive message is idiomatic in tests."
    )]
    async fn get_should_return_value_for_existing_key() {
        let mut mock = MockCache::<String, String>::new();
        mock.expect_get()
            .with(mockall::predicate::eq("exists".to_owned()))
            .returning(|_| Box::pin(async { Ok(Some("value".to_owned())) }));

        let result = mock.get(&"exists".to_owned()).await;
        match result {
            Ok(Some(value)) => {
                assert_eq!(value, "value", "Should return expected value");
            }
            Ok(None) => panic!("Expected Some(value), got None"),
            Err(e) => panic!("Expected success, got error: {e:?}"),
        }
    }

    #[tokio::test]
    async fn put_should_succeed() {
        let mut mock = MockCache::<String, String>::new();
        mock.expect_put()
            .with(
                mockall::predicate::eq("key".to_owned()),
                mockall::predicate::eq("value".to_owned()),
            )
            .returning(|_, _| Box::pin(async { Ok(()) }));

        let result = mock.put("key".to_owned(), "value".to_owned()).await;
        assert!(
            result.is_ok(),
            "Put should succeed, but failed with: {:?}",
            result.err()
        );
    }

    #[tokio::test]
    async fn delete_should_return_false_for_missing_key() {
        let mut mock = MockCache::<String, String>::new();
        mock.expect_delete()
            .with(mockall::predicate::eq("missing".to_owned()))
            .returning(|_| Box::pin(async { Ok(false) }));

        let result = mock.delete(&"missing".to_owned()).await;
        assert!(
            matches!(result, Ok(false)),
            "Expected Ok(false), got {result:?}"
        );
    }

    #[tokio::test]
    async fn delete_should_return_true_for_existing_key() {
        let mut mock = MockCache::<String, String>::new();
        mock.expect_delete()
            .with(mockall::predicate::eq("exists".to_owned()))
            .returning(|_| Box::pin(async { Ok(true) }));

        let result = mock.delete(&"exists".to_owned()).await;
        assert!(
            matches!(result, Ok(true)),
            "Expected Ok(true), got {result:?}"
        );
    }

    #[tokio::test]
    async fn invalidate_should_behave_like_delete() {
        let mut mock = MockCache::<String, String>::new();
        mock.expect_invalidate()
            .with(mockall::predicate::eq("key".to_owned()))
            .returning(|_| Box::pin(async { Ok(true) }));

        let result = mock.invalidate(&"key".to_owned()).await;
        assert!(
            matches!(result, Ok(true)),
            "Expected Ok(true), got {result:?}"
        );
    }

    // [5.1-U-14] Cache Error Handling
    #[tokio::test]
    #[expect(
        clippy::panic,
        reason = "Test assertion requires explicit failure for unrecoverable \
                  contract violation. Unavoidable in complex match patterns. \
                  Panic with descriptive message is idiomatic in tests."
    )]
    async fn get_should_propagate_io_error() {
        let mut mock = MockCache::<String, String>::new();
        mock.expect_get().returning(|_| {
            Box::pin(async {
                Err(CacheError::IoError(std::io::Error::other("io error")))
            })
        });

        let result = mock.get(&"key".to_owned()).await;
        match result {
            Err(CacheError::IoError(e)) => {
                assert_eq!(
                    e.to_string(),
                    "io error",
                    "Error message should match"
                );
            }
            _ => panic!("Expected IoError, got {result:?}"),
        }
    }

    #[tokio::test]
    #[expect(
        clippy::panic,
        reason = "Test assertion requires explicit failure for unrecoverable \
                  contract violation. Unavoidable in complex match patterns. \
                  Panic with descriptive message is idiomatic in tests."
    )]
    async fn put_should_propagate_serialization_error() {
        let mut mock = MockCache::<String, String>::new();
        mock.expect_put().returning(|_, _| {
            Box::pin(async {
                Err(CacheError::SerializationError("ser error".to_owned()))
            })
        });

        let result = mock.put("key".to_owned(), "value".to_owned()).await;
        match result {
            Err(CacheError::SerializationError(e)) => {
                assert_eq!(e, "ser error", "Error message should match");
            }
            _ => panic!("Expected SerializationError, got {result:?}"),
        }
    }

    #[tokio::test]
    #[expect(
        clippy::panic,
        reason = "Test assertion requires explicit failure for unrecoverable \
                  contract violation. Unavoidable in complex match patterns. \
                  Panic with descriptive message is idiomatic in tests."
    )]
    async fn delete_should_propagate_backend_error() {
        let mut mock = MockCache::<String, String>::new();
        mock.expect_delete().returning(|_| {
            Box::pin(async {
                Err(CacheError::BackendError("backend error".to_owned()))
            })
        });

        let result = mock.delete(&"key".to_owned()).await;
        match result {
            Err(CacheError::BackendError(e)) => {
                assert_eq!(e, "backend error", "Error message should match");
            }
            _ => panic!("Expected BackendError, got {result:?}"),
        }
    }
}
