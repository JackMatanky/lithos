//! Vault repository persistence implementation using redb.
//!
//! This module provides the [`RedbRepository`] struct, which implements the
//! segregated repository traits ([`ReadRepository`], [`WriteRepository`], and
//! [`Repository`]) for vault file and directory view persistence using `redb`
//! as the storage engine.
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
//! [`ReadRepository`]: crate::vault::repository::ReadRepository
//! [`WriteRepository`]: crate::vault::repository::WriteRepository
//! [`Repository`]: crate::vault::repository::Repository

mod read;
pub(crate) mod tables;
mod write;

use std::sync::Arc;

// Re-export for vault/mod.rs consumption by legacy storage
#[expect(
    unused_imports,
    reason = "Used by legacy storage, will be removed after migration"
)]
pub(crate) use tables::{
    DIR_ID_BY_PATH, DIR_VIEWS, FILE_ID_BY_PATH, FILE_IDS_BY_BASENAME,
    FILE_IDS_BY_FORMAT, FILE_IDS_BY_PARENT, FILE_VIEWS,
};

use crate::db::Store;

/// Repository implementation for `redb`-backed vault storage.
///
/// This struct implements the segregated repository traits using `redb`
/// as the underlying storage engine. It wraps a [`Store`] instance and
/// manages its own transaction boundaries for all persistence operations.
///
/// # Transaction Management
///
/// Each repository method opens and commits its own transaction. For batch
/// operations (e.g., `save_many_file_views`, `delete_many_file_views`),
/// multiple operations are grouped into a single transaction for atomicity
/// and efficiency.
///
/// # Index Maintenance
///
/// File views maintain five locations atomically:
/// - Primary table (`FILE_VIEWS`)
/// - Path index (`FILE_ID_BY_PATH`)
/// - Basename multimap (`FILE_IDS_BY_BASENAME`)
/// - Parent multimap (`FILE_IDS_BY_PARENT`)
/// - Format multimap (`FILE_IDS_BY_FORMAT`)
///
/// Directory views maintain two locations:
/// - Primary table (`DIR_VIEWS`)
/// - Path index (`DIR_ID_BY_PATH`)
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
    /// use lithos_core::vault::storage::RedbRepository;
    ///
    /// let store = Arc::new(Store::open("vault.db")?);
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
