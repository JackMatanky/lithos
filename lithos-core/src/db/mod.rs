//! Zero-copy database layer using redb and rkyv.
//!
//! This module provides concrete types (not traits) for database operations,
//! following the `std::fs::File` pattern. “Zero-copy” reads are achieved via
//! closure-based APIs that keep transactions properly scoped.
//!
//! # Architecture
//!
//! - [`Database`] - Concrete type wrapping `redb::Database`
//! - Closure-based API - Transactions scoped within closures (safe, no unsafe)
//! - Sync-first design - No async overhead
//!
//! # Examples
//!
//! ```no_run
//! use std::path::Path;
//!
//! use lithos_core::db::Database;
//!
//! # fn main() -> Result<(), Box<dyn std::error::Error>> {
//! let db = Database::open(Path::new("/tmp/test.db"))?;
//! # Ok(())
//! # }
//! ```

#![allow(
    clippy::pub_use,
    reason = "This module intentionally re-exports a small public surface \
              (db::DbError, db::WriteBatch) for ergonomic crate consumers"
)]
#![allow(
    clippy::module_name_repetitions,
    reason = "DbError is intentionally explicit at the crate API boundary"
)]

pub mod config_adapter;
mod error;
mod reader;
pub mod schema_adapter;
mod writer;

use std::path::Path;

pub use error::DbError;
pub use writer::WriteBatch;

/// Concrete database type wrapping redb.
///
/// Provides zero-copy read/write primitives using rkyv serialization.
/// Follows the `std::fs::File` pattern with concrete methods instead of traits.
#[derive(Debug)]
#[non_exhaustive]
pub struct Database {
    inner: redb::Database,
}

impl Database {
    /// Open or create a database at the given path.
    ///
    /// # Errors
    ///
    /// Returns `DbError::Open` if the database cannot be opened or created.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use std::path::Path;
    ///
    /// use lithos_core::db::Database;
    ///
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// let db = Database::open(Path::new("/tmp/lithos.db"))?;
    /// # Ok(())
    /// # }
    /// ```
    #[inline]
    pub fn open(path: &Path) -> Result<Self, DbError> {
        let inner = redb::Database::create(path)
            .map_err(|e| DbError::Open(e.to_string()))?;
        Ok(Self {
            inner,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn database_can_be_constructed() -> Result<(), Box<dyn std::error::Error>> {
        let temp = tempfile::tempdir()?;
        let db_path = temp.path().join("test.db");
        Database::open(&db_path).map_err(|err| {
            format!("Database should open successfully, got: {err:?}")
        })?;
        Ok(())
    }
}
