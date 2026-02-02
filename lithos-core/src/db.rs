//! Zero-copy database layer using redb and rkyv.
//!
//! This module provides concrete types (not traits) for database operations,
//! following the `std::fs::File` pattern. Zero-copy reads are achieved through
//! `ArchivedGuard` which wraps redb's `AccessGuard`.

use std::path::Path;

/// Concrete database type wrapping redb.
///
/// Provides zero-copy read/write primitives using rkyv serialization.
/// This is a stub implementation for Phase 2.
#[non_exhaustive]
pub struct Database;

/// Database error types.
#[expect(
    clippy::module_name_repetitions,
    reason = "Context-specific error name is intentional"
)]
#[non_exhaustive]
#[derive(Debug, thiserror::Error)]
pub enum DbError {
    /// Database not found or cannot be opened.
    #[error("database error: {0}")]
    Database(String),
}

impl Database {
    /// Open or create a database at the given path.
    ///
    /// # Errors
    ///
    /// Returns `DbError::Database` if the database cannot be opened.
    #[inline]
    pub fn open(_path: &Path) -> Result<Self, DbError> {
        // Stub implementation - Phase 6 will implement real redb integration
        Ok(Self)
    }
}
