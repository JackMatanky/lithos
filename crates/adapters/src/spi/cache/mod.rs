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

pub mod moka;
pub mod redb;

use async_trait::async_trait;

#[expect(
    clippy::module_name_repetitions,
    reason = "Re-exporting with prefixed names is intentional for clarity at \
              the crate level while keeping internal implementation names \
              concise."
)]
pub use self::moka::{Builder as MokaCacheBuilder, Cache as MokaCache};
#[expect(
    clippy::module_name_repetitions,
    reason = "Re-exporting with prefixed names is intentional for clarity at \
              the crate level while keeping internal implementation names \
              concise."
)]
pub use self::redb::{
    Cache as RedbCache, Entry as CacheEntry, Outcome as CacheResult,
};
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
/// #     async fn clear(&self) -> Result<(), CacheError> { Ok(()) }
/// #     async fn delete(&self, _k: &String) -> Result<bool, CacheError> { Ok(false) }
/// #     async fn get(&self, _k: &String) -> Result<Option<String>, CacheError> { Ok(None) }
/// #     async fn put(&self, _k: String, _v: String) -> Result<(), CacheError> { Ok(()) }
/// # }
/// # tokio::runtime::Runtime::new().unwrap().block_on(async {
/// let cache = MemoryCache;
/// let result: Option<String> = cache.get(&"key".to_string()).await?;
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
    /// Clear all entries from the cache.
    ///
    /// # Errors
    /// Returns `CacheError` if the underlying storage fails.
    async fn clear(&self) -> Result<(), CacheError>;

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

    /// Check if key exists in cache.
    ///
    /// This is a performance optimization to avoid cloning the value when only
    /// existence needs to be verified.
    ///
    /// # Errors
    /// Returns `CacheError` if the underlying storage fails.
    #[inline]
    async fn has(&self, key: &K) -> Result<bool, CacheError> {
        Ok(self.get(key).await?.is_some())
    }

    /// Alias for `delete` (cache-specific terminology).
    ///
    /// Returns `true` if the entry existed and was removed.
    ///
    /// # Errors
    /// Returns `CacheError` if the underlying storage fails.
    #[inline]
    async fn invalidate(&self, key: &K) -> Result<bool, CacheError> {
        self.delete(key).await
    }

    /// Store key-value pair.
    ///
    /// # Errors
    /// Returns `CacheError` if the underlying storage fails.
    async fn put(&self, key: K, value: V) -> Result<(), CacheError>;
}

#[cfg(test)]
#[expect(
    clippy::excessive_nesting,
    reason = "Mockall async trait expectations require nested Box::pin(async \
              { ... }) blocks which trigger this lint. Nesting is unavoidable \
              for trait mocking."
)]
mod tests {
    use super::*;

    mod trait_behavior {
        use super::*;

        struct Dummy;

        #[async_trait]
        #[expect(
            clippy::missing_trait_methods,
            reason = "Dummy intentionally uses default implementation for \
                      invalidate/has to test trait-level behavior."
        )]
        impl Cache<String, String> for Dummy {
            async fn clear(&self) -> Result<(), CacheError> {
                Ok(())
            }

            async fn delete(&self, _key: &String) -> Result<bool, CacheError> {
                Ok(false)
            }

            async fn get(
                &self,
                _key: &String,
            ) -> Result<Option<String>, CacheError> {
                Ok(None)
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

        #[test]
        fn should_have_has_method() {
            // Verified by Dummy implementation
        }

        #[test]
        fn should_have_clear_method() {
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
    }

    mod mock_expectations {
        use super::*;

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
        async fn mock_should_allow_has_expectation() {
            let mut mock = MockCache::<String, String>::new();
            mock.expect_has().returning(|_| Box::pin(async { Ok(true) }));
            let result = mock.has(&"key".to_owned()).await;
            assert!(result.is_ok(), "Mock has should return Ok");
        }

        #[tokio::test]
        async fn mock_should_allow_clear_expectation() {
            let mut mock = MockCache::<String, String>::new();
            mock.expect_clear().returning(|| Box::pin(async { Ok(()) }));
            let result = mock.clear().await;
            assert!(result.is_ok(), "Mock clear should return Ok");
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
            mock.expect_invalidate()
                .returning(|_| Box::pin(async { Ok(true) }));
            let result = mock.invalidate(&"key".to_owned()).await;
            assert!(result.is_ok(), "Mock invalidate should return Ok");
        }
    }

    mod contract_behavior {
        use super::*;

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
        async fn get_should_return_value_for_existing_key() {
            let mut mock = MockCache::<String, String>::new();
            mock.expect_get()
                .with(mockall::predicate::eq("exists".to_owned()))
                .returning(|_| {
                    Box::pin(async { Ok(Some("value".to_owned())) })
                });

            let result = mock.get(&"exists".to_owned()).await;
            assert!(
                matches!(&result, Ok(Some(v)) if v == "value"),
                "Expected Ok(Some('value')), got {result:?}"
            );
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
    }

    mod error_handling {
        use super::*;

        // [5.1-U-14] Cache Error Handling
        #[tokio::test]
        async fn get_should_propagate_io_error() {
            let mut mock = MockCache::<String, String>::new();
            mock.expect_get().returning(|_| {
                Box::pin(async {
                    Err(CacheError::IoError(std::io::Error::other("io error")))
                })
            });

            let result = mock.get(&"key".to_owned()).await;
            assert!(
                matches!(
                    &result,
                    Err(CacheError::IoError(e)) if e.to_string() == "io error"
                ),
                "Expected IoError with message 'io error', got {result:?}"
            );
        }

        #[tokio::test]
        async fn put_should_propagate_serialization_error() {
            let mut mock = MockCache::<String, String>::new();
            mock.expect_put().returning(|_, _| {
                Box::pin(async {
                    Err(CacheError::SerializationError {
                        type_name: "String",
                        message: "ser error".into(),
                    })
                })
            });

            let result = mock.put("key".to_owned(), "value".to_owned()).await;
            assert!(
                matches!(
                    &result,
                    Err(CacheError::SerializationError {
                        type_name: "String",
                        message
                    })
                    if message.as_ref() == "ser error"
                ),
                "Expected SerializationError for String, got {result:?}"
            );
        }

        #[tokio::test]
        async fn delete_should_propagate_backend_error() {
            let mut mock = MockCache::<String, String>::new();
            mock.expect_delete().returning(|_| {
                Box::pin(async {
                    Err(CacheError::BackendError {
                        backend: "moka",
                        message: "backend error".into(),
                    })
                })
            });

            let result = mock.delete(&"key".to_owned()).await;
            assert!(
                matches!(
                    &result,
                    Err(CacheError::BackendError {
                        backend: "moka",
                        message
                    })
                    if message.as_ref() == "backend error"
                ),
                "Expected BackendError for moka, got {result:?}"
            );
        }
    }
}
