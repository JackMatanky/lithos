//! Template repository persistence implementation using redb.
//!
//! This module provides the [`RedbRepository`] struct, which implements the
//! segregated repository traits ([`ReadRepository`], [`WriteRepository`], and
//! [`Repository`]) for template persistence using `redb` as the storage engine.
//!
//! # Architecture
//!
//! - **Transaction Boundaries**: Each repository method manages its own
//!   transaction via the provided [`Store`].
//! - **Segregated Traits**: Read and write operations are separated for
//!   capability-based access control. The unified [`Repository`] trait is
//!   automatically implemented via blanket impl for any type implementing both.
//!
//! # Example
//!
//! ```rust,ignore
//! use std::sync::Arc;
//! use traces_db::Store;
//! use traces_template::storage::RedbRepository;
//! use traces_template::storage::Repository;
//!
//! let store = Arc::new(Store::open("templates.db")?);
//! let repo = RedbRepository::new(store);
//! // Use repo for read/write operations
//! # Ok::<(), Box<dyn std::error::Error>>(())
//! ```
//!
//! [`ReadRepository`]: super::ReadRepository
//! [`WriteRepository`]: super::WriteRepository
//! [`Repository`]: super::Repository

use std::sync::Arc;

use traces_db::Store;

/// Repository implementation for `redb`-backed template storage.
///
/// This struct implements the segregated repository traits using `redb`
/// as the underlying storage engine. It wraps a [`Store`] instance and
/// manages its own transaction boundaries for all persistence operations.
///
/// # Transaction Management
///
/// Each repository method opens and commits its own transaction. For batch
/// operations (e.g., `save_many_raw_template_views`,
/// `find_raw_template_views_by_paths`), multiple operations are grouped
/// into a single transaction for atomicity and efficiency.
///
/// # Thread Safety
///
/// `RedbRepository` is `Send + Sync` when the wrapped `Store` is thread-safe
/// (requires `Arc<Store>`). Multiple repository instances can safely share
/// the same `Store`.
#[derive(Debug)]
#[allow(dead_code, reason = "redb adapter is wired by integration callers")]
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
    /// use traces_template::storage::RedbRepository;
    ///
    /// let store = Arc::new(Store::open("templates.db")?);
    /// let repo = RedbRepository::new(Arc::clone(&store));
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    #[inline]
    #[must_use]
    #[allow(dead_code, reason = "redb adapter is wired by integration callers")]
    pub fn new(store: Arc<Store>) -> Self {
        Self {
            store,
        }
    }
}
