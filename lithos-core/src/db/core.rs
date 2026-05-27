//! Core database handles and transaction scaffolding.
//!
//! This module provides the primary [`Store`] handle for managing database
//! connections and scoped transactions.

use std::path::Path;

use redb::ReadableDatabase as _;

use crate::db::{error::DbError, read::ReadTx, write::WriteTx};

/// Thread-safe database handle for scoped transactions.
///
/// The `Store` is the primary entry point for the DB module. It manages the
/// physical database file and provides high-level, closure-based APIs for read
/// and write transactions.
///
/// # Concurrency
///
/// This type is `Send + Sync` and can be shared freely between threads (e.g.,
/// in an `Arc`). It uses `redb` internally, which supports multi-reader,
/// single-writer concurrency.
///
/// # Transaction Scoping
///
/// Transactions are strictly scoped to the provided closure. This prevents
/// accidental deadlocks and ensures that zero-copy references (which point to
/// memory-mapped database pages) cannot escape the transaction lifetime.
#[derive(Debug)]
pub struct Store {
    inner: redb::Database,
}

impl Store {
    /// Open or create a database at the given path.
    ///
    /// # Errors
    ///
    /// Returns [`DbError::Database`] if the database cannot be opened or
    /// created.
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
    /// Returns [`DbError`] if the transaction fails or the closure returns an
    /// error.
    #[inline]
    pub fn read<R, F>(&self, f: F) -> Result<R, DbError>
    where
        F: FnOnce(&ReadTx) -> Result<R, DbError>,
    {
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
    /// Returns [`DbError`] if the transaction fails or the closure returns an
    /// error.
    #[inline]
    pub fn write<R, F>(&self, f: F) -> Result<R, DbError>
    where
        F: FnOnce(&mut WriteTx) -> Result<R, DbError>,
    {
        let tx = self.inner.begin_write()?;
        let mut write_tx = WriteTx {
            inner: tx,
        };
        let result = f(&mut write_tx)?;
        write_tx.inner.commit()?;
        Ok(result)
    }

    /// Creates a temporary store that will be cleaned up when the `TempDir`
    /// is dropped.
    ///
    /// # Errors
    ///
    /// Returns `DbError` if the temporary directory or store cannot be created.
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// use lithos_core::db::Store;
    ///
    /// let (_tempdir, store) = Store::open_temp().unwrap();
    /// store.read(|tx| Ok(())).unwrap();
    /// ```
    #[cfg(test)]
    pub(crate) fn open_temp() -> Result<(tempfile::TempDir, Self), DbError> {
        let tempdir = tempfile::tempdir().map_err(|e| {
            DbError::Open(format!("Failed to create tempdir: {e}"))
        })?;
        let store = Self::open(&tempdir.path().join("test.redb"))?;
        Ok((tempdir, store))
    }

    /// Creates a temporary store wrapped in `Arc` for shared access.
    ///
    /// # Errors
    ///
    /// Returns `DbError` if the temporary directory or store cannot be created.
    #[cfg(test)]
    pub(crate) fn open_temp_arc()
    -> Result<(tempfile::TempDir, std::sync::Arc<Self>), DbError> {
        let (tempdir, store) = Self::open_temp()?;
        Ok((tempdir, std::sync::Arc::new(store)))
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
        #[test]
        #[expect(
            clippy::unwrap_in_result,
            clippy::panic_in_result_fn,
            reason = "Test code intentionally uses unwrap/assert for setup \
                      verification"
        )]
        fn write_commits_on_ok() -> Result<(), DbError> {
            let temp_dir = tempfile::TempDir::new().unwrap();
            let db_path = temp_dir.path().join("test.db");

            let mut store = Store::open(&db_path)?;
            store.write(|tx| {
                let mut table = tx.inner.open_table(TEST_TABLE)?;
                table.insert("key", "value")?;
                Ok(())
            })?;
            drop(store);

            store = Store::open(&db_path)?;
            store.read(|tx| {
                let table = tx.inner.open_table(TEST_TABLE)?;
                let value = table.get("key")?.expect("value should exist");
                assert_eq!(value.value(), "value");
                Ok(())
            })?;

            Ok(())
        }

        /// `Store::write()` rolls back on Err.
        #[test]
        #[expect(
            clippy::unwrap_in_result,
            clippy::panic_in_result_fn,
            reason = "Test code intentionally uses unwrap/assert for setup \
                      verification"
        )]
        fn write_rolls_back_on_err() -> Result<(), DbError> {
            let temp_dir = tempfile::TempDir::new().unwrap();
            let db_path = temp_dir.path().join("test.db");

            let mut store = Store::open(&db_path)?;
            store.write(|tx| {
                let mut table = tx.inner.open_table(TEST_TABLE)?;
                table.insert("initial", "data")?;
                Ok(())
            })?;
            drop(store);

            store = Store::open(&db_path)?;
            let result: Result<(), DbError> = store.write(|tx| {
                let mut table = tx.inner.open_table(TEST_TABLE)?;
                table.insert("new_key", "new_value")?;
                Err(DbError::Serialization("intentional error".to_owned()))
            });
            assert!(result.is_err());
            drop(store);

            store = Store::open(&db_path)?;
            store.read(|tx| {
                let table = tx.inner.open_table(TEST_TABLE)?;
                let initial =
                    table.get("initial")?.expect("initial data should exist");
                assert_eq!(initial.value(), "data");
                let new_value = table.get("new_key")?;
                assert!(new_value.is_none());
                Ok(())
            })?;

            Ok(())
        }

        #[test]
        fn creates_temp_store() {
            // Act
            let (_tempdir, store) = Store::open_temp().unwrap();

            // Assert: store is usable
            store.read(|_tx| Ok(())).unwrap();
        }

        #[test]
        fn creates_stores_with_unique_paths() {
            // Arrange & Act
            let (dir1, _store1) = Store::open_temp().unwrap();
            let (dir2, _store2) = Store::open_temp().unwrap();

            // Assert
            assert_ne!(dir1.path(), dir2.path());
            assert!(dir1.path().exists());
            assert!(dir2.path().exists());
        }

        #[test]
        fn can_be_constructed() -> Result<(), Box<dyn std::error::Error>> {
            let temp = tempfile::tempdir()?;
            let db_path = temp.path().join("test.db");
            Store::open(&db_path).map_err(|err| {
                format!("Store should open successfully, got: {err:?}")
            })?;
            Ok(())
        }
    }
}
