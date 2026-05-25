//! Note repository persistence implementation using redb.
//!
//! This module provides the [`RedbRepository`] struct, which implements the
//! segregated repository traits ([`ReadRepository`], [`WriteRepository`], and
//! [`Repository`]) for note persistence using `redb` as the storage engine.
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
//! - `read`: Internal [`ReadRepository`] implementation
//! - `write`: Internal [`WriteRepository`] implementation
//! - `testing`: In-memory [`Repository`] test double (test-only)
//!
//! [`ReadRepository`]: crate::note::repository::ReadRepository
//! [`WriteRepository`]: crate::note::repository::WriteRepository
//! [`Repository`]: crate::note::repository::Repository

mod read;
#[cfg(test)]
pub(crate) mod testing;
mod write;

pub(crate) mod tables;

use std::sync::Arc;

pub use tables::{LIST_VIEWS, NOTE_ID_BY_PATH, NOTES};

use crate::db::Store;

/// Repository implementation for `redb`-backed note storage.
///
/// This struct implements the segregated repository traits using `redb`
/// as the underlying storage engine. It wraps a [`Store`] instance and
/// manages its own transaction boundaries for all persistence operations.
///
/// # Transaction Management
///
/// Each repository method opens and commits its own transaction. For batch
/// operations (e.g., `save_many`, `find_many_by_id`), multiple operations
/// are grouped into a single transaction for atomicity and efficiency.
///
/// # Thread Safety
///
/// `RedbRepository` is `Send + Sync` when the wrapped `Store` is thread-safe
/// (requires `Arc<Store>`). Multiple repository instances can safely share
/// the same `Store`.
#[derive(Debug)]
pub struct RedbRepository {
    /// Shared database store handle.
    ///
    /// This field is `pub(crate)` to allow child modules (`read`, `write`)
    /// to access the store when implementing trait methods.
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
    /// use lithos_core::db::Store;
    /// use lithos_core::note::storage::RedbRepository;
    ///
    /// let store = Arc::new(Store::open("notes.db")?);
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
