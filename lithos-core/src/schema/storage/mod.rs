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
//! use lithos_core::db::Store;
//! use lithos_core::schema::storage::RedbRepository;
//! use lithos_core::schema::repository::Repository;
//!
//! let store = Arc::new(Store::open("schemas.db")?);
//! let repo = RedbRepository::new(store);
//! // Use repo for read/write operations
//! # Ok::<(), Box<dyn std::error::Error>>(())
//! ```
//!
//! [`ReadRepository`]: crate::schema::repository::ReadRepository
//! [`WriteRepository`]: crate::schema::repository::WriteRepository
//! [`Repository`]: crate::schema::repository::Repository

mod read;
mod write;

pub mod tables;

#[cfg(test)]
pub mod testing;

use std::sync::Arc;

use crate::{db::Store, fs::RelativePath};

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
    /// use lithos_core::db::Store;
    /// use lithos_core::schema::storage::RedbRepository;
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

/// Converts a [`RelativePath`] to an owned `String` for use as a `PathTable`
/// key.
///
/// This helper centralizes the path-to-key conversion logic used across read
/// and write implementations. It uses `to_string_lossy()` to handle non-UTF8
/// paths gracefully (rare in practice for schema files).
///
/// # Performance Note
///
/// Allocates a new `String` on each call. Callers should avoid repeated
/// conversions of the same path in hot loops. For read-heavy operations,
/// consider caching the key.
///
/// # Example
///
/// ```rust,ignore
/// use lithos_core::fs::RelativePath;
/// use lithos_core::schema::storage::path_key;
///
/// let path = RelativePath::try_from("schemas/note.json")?;
/// assert_eq!(path_key(&path), "schemas/note.json");
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
#[inline]
pub(super) fn path_key(path: &RelativePath) -> String {
    path.as_path().to_string_lossy().into_owned()
}

// Blanket implementation: any type that implements both Read and Write
// automatically implements the unified trait.
impl<T> crate::schema::repository::Repository for T where
    T: crate::schema::repository::ReadRepository
        + crate::schema::repository::WriteRepository
{
}

#[cfg(test)]
mod tests {
    use super::path_key;
    use crate::fs::RelativePath;

    #[test]
    fn path_key_matches_relative_path_display() {
        let path = RelativePath::try_from("schemas/note.json").unwrap();
        assert_eq!(path_key(&path), "schemas/note.json");
    }
}
