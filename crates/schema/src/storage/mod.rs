//! Schema repository persistence implementation using redb.
//!
//! This module provides the [`RedbRepository`] struct, which implements the
//! segregated repository traits ([`ReadRepository`], [`WriteRepository`], and
//! [`Repository`]) for schema persistence using `redb` as the storage engine.
//!
//! # Architecture
//!
//! - **Transaction Boundaries**: Each repository method manages its own
//!   transaction via the provided [`Store`].
//! - **Segregated Traits**: Read and write operations are separated for
//!   capability-based access control. The unified [`Repository`] trait is
//!   automatically implemented via blanket impl for any type implementing both.
//!
//! # Modules
//!
//! - [`tables`]: Public table definitions and constants
//! - `read`: Internal [`ReadRepository`] implementation (private)
//! - `write`: Internal [`WriteRepository`] implementation (private)
//! - [`testing`]: Test utilities (available in `#[cfg(test)]`)
//!
//! # Example
//!
//! ```rust,ignore
//! use std::sync::Arc;
//! use traces_db::Store;
//! use traces_schema::storage::RedbRepository;
//! use traces_schema::repository::Repository;
//!
//! let store = Arc::new(Store::open("schemas.db")?);
//! let repo = RedbRepository::new(store);
//! // Use repo for read/write operations
//! # Ok::<(), Box<dyn std::error::Error>>(())
//! ```
//!
//! [`ReadRepository`]: crate::repository::ReadRepository
//! [`WriteRepository`]: crate::repository::WriteRepository
//! [`Repository`]: crate::repository::Repository

mod read;
mod write;

pub mod tables;

#[cfg(test)]
pub(crate) mod testing;

use std::sync::Arc;

use traces_db::Store;

/// Repository implementation for `redb`-backed schema storage.
///
/// This struct implements the segregated repository traits using `redb`
/// as the underlying storage engine. It wraps a [`Store`] instance and
/// manages its own transaction boundaries for all persistence operations.
///
/// # Transaction Management
///
/// Each repository method opens and commits its own transaction. For batch
/// operations (e.g., `save_many_schemas`, `find_raw_schema_views_by_paths`),
/// multiple operations are grouped into a single transaction for atomicity
/// and efficiency.
///
/// # Thread Safety
///
/// `RedbRepository` is `Send + Sync` when the wrapped `Store` is thread-safe
/// (requires `Arc<Store>`). Multiple repository instances can safely share
/// the same `Store`.
#[derive(Debug)]
pub struct RedbRepository {
    pub(crate) store: Arc<Store>,
}

impl RedbRepository {
    /// Creates a new repository adapter from a database store.
    ///
    /// The provided [`Store`] instance must be shared across all repository
    /// instances to ensure transaction isolation and consistency. Multiple
    /// `RedbRepository` instances wrapping the same `Store` will share the
    /// same underlying database connection.
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// use std::sync::Arc;
    /// use traces_db::Store;
    /// use traces_schema::storage::RedbRepository;
    ///
    /// let store = Arc::new(Store::open("schemas.db")?);
    /// let repo = RedbRepository::new(Arc::clone(&store));
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    #[inline]
    #[must_use]
    pub fn new(store: Arc<Store>) -> Self {
        Self {
            store,
        }
    }
}

// Blanket implementation: any type that implements both Read and Write
// automatically implements the unified trait.
impl<T> crate::repository::Repository for T where
    T: crate::repository::ReadRepository + crate::repository::WriteRepository
{
}
