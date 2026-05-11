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
    clippy::impl_trait_in_params,
    reason = "This module intentionally re-exports a small public surface \
              (db::DbError, db::BatchWriter) for ergonomic crate consumers"
)]
#![allow(
    clippy::module_name_repetitions,
    reason = "DbError is intentionally explicit at the crate API boundary"
)]

mod error;
mod read;
mod reader;
pub mod retry;
mod rkyv;
mod table;
mod uuid;
mod write;
mod writer;

use std::path::Path;

pub use error::{DbError, DbErrorKind};
pub use read::ReadTx;
pub use reader::BatchReader;
use redb::ReadableDatabase as _;
pub use table::{PathTable, Table, UuidMultimap, UuidTable};
pub use uuid::UuidV7DbType;
pub use write::WriteTx;
pub use writer::BatchWriter;

/// Transaction-scoped database handle.
///
/// Provides closure-based transaction API with automatic commit/rollback.
/// New code should use `Store`; `Database` is kept for migration compatibility.
#[derive(Debug)]
pub struct Store {
    inner: redb::Database,
}

/// Concrete database type wrapping redb.
///
/// Provides zero-copy read/write primitives using rkyv serialization.
/// Follows the `std::fs::File` pattern with concrete methods instead of traits.
#[derive(Debug)]
#[non_exhaustive]
pub struct Database {
    inner: redb::Database,
}

impl Store {
    /// Open or create a database at the given path.
    ///
    /// # Errors
    ///
    /// Returns `DbError::Database` if the database cannot be opened or created.
    #[inline]
    pub fn open(path: &Path) -> Result<Self, DbError> {
        let inner = redb::Database::create(path)?;
        Ok(Self {
            inner,
        })
    }

    /// Execute read-only operations within a transaction.
    ///
    /// # Errors
    ///
    /// Returns `DbError` if the transaction fails or the closure returns an
    /// error.
    #[inline]
    pub fn read<R>(
        &self,
        f: impl FnOnce(&ReadTx) -> Result<R, DbError>,
    ) -> Result<R, DbError> {
        let tx = self.inner.begin_read()?;
        let read_tx = ReadTx {
            inner: tx,
        };
        f(&read_tx)
    }

    /// Execute read-write operations within a transaction.
    ///
    /// Automatically commits on `Ok`, rolls back on `Err`.
    ///
    /// # Errors
    ///
    /// Returns `DbError` if the transaction fails or the closure returns an
    /// error.
    #[inline]
    pub fn write<R>(
        &self,
        f: impl FnOnce(&mut WriteTx) -> Result<R, DbError>,
    ) -> Result<R, DbError> {
        let tx = self.inner.begin_write()?;
        let mut write_tx = WriteTx {
            inner: tx,
        };
        let result = f(&mut write_tx)?;
        write_tx.inner.commit()?;
        Ok(result)
    }
}

#[cfg(test)]
mod store_tests {
    use redb::TableDefinition;

    use super::*;

    const TEST_TABLE: TableDefinition<&str, &str> =
        TableDefinition::new("test");

    /// Tracer bullet: `Store::write()` auto-commits on Ok.
    ///
    /// Behavior: When a write closure returns Ok, changes are persisted to
    /// disk. Verification: Open a new Store instance and read back the
    /// data.
    #[test]
    #[expect(
        clippy::unwrap_in_result,
        clippy::panic_in_result_fn,
        reason = "Test code intentionally uses unwrap/assert for setup \
                  verification"
    )]
    fn write_commits_on_ok() -> Result<(), DbError> {
        // Setup: Create temporary database
        let temp_dir = tempfile::TempDir::new().unwrap();
        let db_path = temp_dir.path().join("test.db");

        // Act: Write data using Store::write()
        {
            let store = Store::open(&db_path)?;
            store.write(|tx| {
                let mut table = tx.inner.open_table(TEST_TABLE)?;
                table.insert("key", "value")?;
                Ok(())
            })?;
        };

        // Verify: Open fresh Store instance and confirm data persisted
        {
            let store = Store::open(&db_path)?;
            store.read(|tx| {
                let table = tx.inner.open_table(TEST_TABLE)?;
                let value = table.get("key")?.expect("value should exist");
                assert_eq!(value.value(), "value");
                Ok(())
            })?;
        };

        Ok(())
    }
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

    /// Begin a new read-only transaction.
    ///
    /// # Errors
    ///
    /// Returns `DbError::Transaction` if the transaction cannot be started.
    #[inline]
    pub fn begin_read(&self) -> Result<redb::ReadTransaction, DbError> {
        self.inner.begin_read().map_err(|e| DbError::Transaction(e.to_string()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    mod store {
        use redb::TableDefinition;

        use super::*;

        const TEST_TABLE: TableDefinition<&str, &str> =
            TableDefinition::new("test");

        /// Tracer bullet: `Store::write()` auto-commits on Ok.
        ///
        /// Behavior: When a write closure returns Ok, changes are persisted to
        /// disk. Verification: Open a new Store instance and read back
        /// the data.
        #[test]
        #[expect(
            clippy::unwrap_in_result,
            clippy::panic_in_result_fn,
            reason = "Test code intentionally uses unwrap/assert for setup \
                      verification"
        )]
        fn write_commits_on_ok() -> Result<(), DbError> {
            // Setup: Create temporary database
            let temp_dir = tempfile::TempDir::new().unwrap();
            let db_path = temp_dir.path().join("test.db");

            // Act: Write data using Store::write()
            {
                let store = Store::open(&db_path)?;
                store.write(|tx| {
                    let mut table = tx.inner.open_table(TEST_TABLE)?;
                    table.insert("key", "value")?;
                    Ok(())
                })?;
            };

            // Verify: Open fresh Store instance and confirm data persisted
            {
                let store = Store::open(&db_path)?;
                store.read(|tx| {
                    let table = tx.inner.open_table(TEST_TABLE)?;
                    let value = table.get("key")?.expect("value should exist");
                    assert_eq!(value.value(), "value");
                    Ok(())
                })?;
            };

            Ok(())
        }

        /// `Store::write()` rolls back on Err.
        ///
        /// Behavior: When a write closure returns Err, changes are NOT
        /// persisted. Verification: Create table successfully, then
        /// attempt failed write, verify data unchanged.
        #[test]
        #[expect(
            clippy::unwrap_in_result,
            clippy::panic_in_result_fn,
            reason = "Test code intentionally uses unwrap/assert for setup \
                      verification"
        )]
        fn write_rolls_back_on_err() -> Result<(), DbError> {
            // Setup: Create temporary database with initial data
            let temp_dir = tempfile::TempDir::new().unwrap();
            let db_path = temp_dir.path().join("test.db");

            {
                let store = Store::open(&db_path)?;
                store.write(|tx| {
                    let mut table = tx.inner.open_table(TEST_TABLE)?;
                    table.insert("initial", "data")?;
                    Ok(())
                })?;
            };

            // Act: Attempt to write more data but return error
            {
                let store = Store::open(&db_path)?;
                let result: Result<(), DbError> = store.write(|tx| {
                    let mut table = tx.inner.open_table(TEST_TABLE)?;
                    table.insert("new_key", "new_value")?;
                    // Return error to trigger rollback
                    Err(DbError::Serialization("intentional error".to_owned()))
                });

                // Write should fail with our error
                assert!(result.is_err());
            };

            // Verify: Open fresh Store instance and confirm only initial data
            // exists
            {
                let store = Store::open(&db_path)?;
                store.read(|tx| {
                    let table = tx.inner.open_table(TEST_TABLE)?;

                    // Initial data should still exist
                    let initial = table
                        .get("initial")?
                        .expect("initial data should exist");
                    assert_eq!(initial.value(), "data");

                    // New data should NOT exist (was rolled back)
                    let new_value = table.get("new_key")?;
                    assert!(
                        new_value.is_none(),
                        "new data should not exist after rollback"
                    );

                    Ok(())
                })?;
            };

            Ok(())
        }
    }

    mod database {
        use super::*;

        #[test]
        fn can_be_constructed() -> Result<(), Box<dyn std::error::Error>> {
            let temp = tempfile::tempdir()?;
            let db_path = temp.path().join("test.db");
            Database::open(&db_path).map_err(|err| {
                format!("Database should open successfully, got: {err:?}")
            })?;
            Ok(())
        }
    }
}
