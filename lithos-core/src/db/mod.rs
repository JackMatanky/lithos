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

mod batch;
mod error;

use std::path::Path;

pub use batch::WriteBatch;
pub use error::DbError;
use redb::{ReadableDatabase as _, TableDefinition};
use rkyv::util::AlignedVec;

const DATA_TABLE: TableDefinition<&str, &[u8]> = TableDefinition::new("data");

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

    /// Zero-copy read (HOT PATH for LSP).
    ///
    /// The closure receives a reference to the archived data within the
    /// transaction scope, ensuring safety without unsafe code.
    ///
    /// Note: redb does not guarantee alignment of returned byte slices, so this
    /// method performs an alignment copy into an `AlignedVec` before validating
    /// and accessing archived bytes.
    ///
    /// # Type Parameters
    ///
    /// - `V`: Value type (must implement `rkyv::Archive`)
    /// - `F`: Closure type
    /// - `R`: Return type of closure
    ///
    /// # Errors
    ///
    /// - `DbError::Deserialization` - Data validation failed
    /// - `DbError::Transaction` - Transaction or table operation failed
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use lithos_core::db::Database;
    /// # use std::path::Path;
    /// # fn main() -> Result<(), lithos_core::db::DbError> {
    /// let db = Database::open(Path::new("/tmp/test.db"))?;
    /// // db.get::<MyType, _>("my_table", "my_key", |archived| {
    /// //     println!("ID: {:?}", archived.id);
    /// //     archived.id  // return value
    /// // })?;
    /// # Ok(())
    /// # }
    /// ```
    #[inline]
    pub fn get<V, F, R>(
        &self,
        table: &str,
        key: &str,
        f: F,
    ) -> Result<Option<R>, DbError>
    where
        V: rkyv::Archive,
        V::Archived: rkyv::Portable
            + for<'archived> rkyv::bytecheck::CheckBytes<
                rkyv::api::high::HighValidator<'archived, rkyv::rancor::Error>,
            >,
        F: FnOnce(&rkyv::Archived<V>) -> R,
    {
        let namespaced_key = format!("{table}:{key}");

        let tx = self.inner.begin_read()?;
        let table_ref = tx.open_table(DATA_TABLE)?;

        match table_ref.get(namespaced_key.as_str())? {
            Some(value) => {
                let bytes: &[u8] = value.value();

                let mut aligned = AlignedVec::<16>::new();
                aligned.extend_from_slice(bytes);

                let archived = rkyv::access::<
                    rkyv::Archived<V>,
                    rkyv::rancor::Error,
                >(&aligned)
                .map_err(|e| DbError::Deserialization(e.to_string()))?;
                let result = f(archived);
                Ok(Some(result))
            }
            None => Ok(None),
        }
    }

    /// Full deserialization (COLD PATH for mutations).
    ///
    /// Deserializes the value into an owned `V`. This is slower than `get()`
    /// because it allocates and copies data.
    ///
    /// # Type Parameters
    ///
    /// - `V`: Value type (must implement `rkyv::Archive` and
    ///   `rkyv::Deserialize`)
    ///
    /// # Errors
    ///
    /// - `DbError::Deserialization` - Data validation or deserialization failed
    /// - `DbError::Transaction` - Transaction or table operation failed
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use lithos_core::db::Database;
    /// # use std::path::Path;
    /// # fn main() -> Result<(), lithos_core::db::DbError> {
    /// let db = Database::open(Path::new("/tmp/test.db"))?;
    /// // let value: Option<MyType> = db.get_owned("my_table", "my_key")?;
    /// # Ok(())
    /// # }
    /// ```
    #[inline]
    pub fn get_owned<V>(
        &self,
        table: &str,
        key: &str,
    ) -> Result<Option<V>, DbError>
    where
        V: rkyv::Archive,
        V::Archived: rkyv::Portable
            + for<'archived> rkyv::bytecheck::CheckBytes<
                rkyv::api::high::HighValidator<'archived, rkyv::rancor::Error>,
            > + rkyv::Deserialize<
                V,
                rkyv::api::high::HighDeserializer<rkyv::rancor::Error>,
            >,
    {
        let namespaced_key = format!("{table}:{key}");

        let tx = self.inner.begin_read()?;
        let table_ref = tx.open_table(DATA_TABLE)?;

        match table_ref.get(namespaced_key.as_str())? {
            Some(value) => {
                let bytes: &[u8] = value.value();

                let mut aligned = AlignedVec::<16>::new();
                aligned.extend_from_slice(bytes);

                let archived = rkyv::access::<
                    rkyv::Archived<V>,
                    rkyv::rancor::Error,
                >(&aligned)
                .map_err(|e| DbError::Deserialization(e.to_string()))?;
                let deserialized =
                    rkyv::deserialize::<V, rkyv::rancor::Error>(archived)
                        .map_err(|e| DbError::Deserialization(e.to_string()))?;
                Ok(Some(deserialized))
            }
            None => Ok(None),
        }
    }

    /// Insert or update a value.
    ///
    /// Serializes the value using rkyv and writes it to the database.
    /// This replaces any existing value for the given key.
    ///
    /// # Type Parameters
    ///
    /// - `V`: Value type (must implement `rkyv::Serialize`)
    ///
    /// # Errors
    ///
    /// - `DbError::Serialization` - Serialization failed
    /// - `DbError::Transaction` - Transaction or table operation failed
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use lithos_core::db::Database;
    /// # use std::path::Path;
    /// # fn main() -> Result<(), lithos_core::db::DbError> {
    /// let db = Database::open(Path::new("/tmp/test.db"))?;
    /// // db.put("my_table", "my_key", &my_value)?;
    /// # Ok(())
    /// # }
    /// ```
    #[inline]
    pub fn put<V>(
        &self,
        table: &str,
        key: &str,
        value: &V,
    ) -> Result<(), DbError>
    where
        V: rkyv::Archive
            + for<'ser> rkyv::Serialize<
                rkyv::api::high::HighSerializer<
                    rkyv::util::AlignedVec,
                    rkyv::ser::allocator::ArenaHandle<'ser>,
                    rkyv::rancor::Error,
                >,
            >,
    {
        let namespaced_key = format!("{table}:{key}");

        let bytes = rkyv::to_bytes::<rkyv::rancor::Error>(value)
            .map_err(|e| DbError::Serialization(e.to_string()))?;

        let tx = self.inner.begin_write()?;
        {
            let mut table_ref = tx.open_table(DATA_TABLE)?;
            table_ref.insert(namespaced_key.as_str(), bytes.as_slice())?;
        };
        tx.commit()?;
        Ok(())
    }

    /// Delete a value by key.
    ///
    /// Returns `true` if a value was deleted, `false` if key didn't exist.
    ///
    /// # Errors
    ///
    /// - `DbError::Transaction` - Transaction or table operation failed
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use lithos_core::db::Database;
    /// # use std::path::Path;
    /// # fn main() -> Result<(), lithos_core::db::DbError> {
    /// let db = Database::open(Path::new("/tmp/test.db"))?;
    /// // let was_deleted = db.delete("my_table", "my_key")?;
    /// # Ok(())
    /// # }
    /// ```
    #[inline]
    pub fn delete(&self, table: &str, key: &str) -> Result<bool, DbError> {
        let namespaced_key = format!("{table}:{key}");

        let tx = self.inner.begin_write()?;
        let existed = {
            let mut table_ref = tx.open_table(DATA_TABLE)?;
            table_ref.remove(namespaced_key.as_str())?.is_some()
        };
        tx.commit()?;
        Ok(existed)
    }

    /// Execute multiple writes in a batch with a single commit.
    ///
    /// The closure receives a [`WriteBatch`] for performing operations.
    /// All operations in the batch share the same write transaction.
    ///
    /// # Errors
    ///
    /// - `DbError::Transaction` - Transaction operation failed
    /// - Propagates errors from batch operations
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use lithos_core::db::Database;
    /// # use std::path::Path;
    /// # fn main() -> Result<(), lithos_core::db::DbError> {
    /// let db = Database::open(Path::new("/tmp/test.db"))?;
    /// // db.batch_write(|batch| {
    /// //     for i in 0..1000 {
    /// //         batch.put("notes", &format!("note_{i}"), &note)?;
    /// //     }
    /// //     Ok(())
    /// // })?;
    /// # Ok(())
    /// # }
    /// ```
    #[inline]
    pub fn batch_write<F>(&self, f: F) -> Result<(), DbError>
    where
        F: FnOnce(&mut WriteBatch) -> Result<(), DbError>,
    {
        let tx = self.inner.begin_write()?;
        let mut batch = WriteBatch::new(tx);

        f(&mut batch)?;
        batch.commit()?;
        Ok(())
    }

    /// Insert a value into a multimap (1:N relationship).
    ///
    /// Multimap tables allow multiple values per key. Useful for indexes
    /// like tags → notes or backlinks.
    ///
    /// # Errors
    ///
    /// - `DbError::Transaction` - Transaction or table operation failed
    #[inline]
    pub fn multimap_insert(
        &self,
        table: &str,
        key: &str,
        value: &str,
    ) -> Result<(), DbError> {
        use redb::MultimapTableDefinition;

        let table_def: MultimapTableDefinition<&str, &str> =
            MultimapTableDefinition::new(table);
        let namespaced_key = format!("multimap:{key}");

        let tx = self.inner.begin_write()?;
        {
            let mut tbl = tx.open_multimap_table(table_def)?;
            tbl.insert(namespaced_key.as_str(), value)?;
        };
        tx.commit()?;
        Ok(())
    }

    /// Remove a value from a multimap.
    ///
    /// Returns `true` if the value was removed, `false` if not found.
    ///
    /// # Errors
    ///
    /// - `DbError::Transaction` - Transaction or table operation failed
    #[inline]
    pub fn multimap_remove(
        &self,
        table: &str,
        key: &str,
        value: &str,
    ) -> Result<bool, DbError> {
        use redb::MultimapTableDefinition;

        let table_def: MultimapTableDefinition<&str, &str> =
            MultimapTableDefinition::new(table);
        let namespaced_key = format!("multimap:{key}");

        let tx = self.inner.begin_write()?;
        let removed = {
            let mut tbl = tx.open_multimap_table(table_def)?;
            tbl.remove(namespaced_key.as_str(), value)?
        };
        tx.commit()?;
        Ok(removed)
    }

    /// Get all values for a key from a multimap.
    ///
    /// Returns a `Vec<String>` containing all values. For large result sets,
    /// consider adding a closure-based API to avoid allocations.
    ///
    /// # Errors
    ///
    /// - `DbError::Transaction` - Transaction or table operation failed
    #[inline]
    pub fn multimap_get(
        &self,
        table: &str,
        key: &str,
    ) -> Result<Vec<String>, DbError> {
        use redb::MultimapTableDefinition;

        let table_def: MultimapTableDefinition<&str, &str> =
            MultimapTableDefinition::new(table);
        let namespaced_key = format!("multimap:{key}");

        let tx = self.inner.begin_read()?;
        let tbl = tx.open_multimap_table(table_def)?;

        let mut values = Vec::new();
        let range = tbl.get(namespaced_key.as_str())?;
        for result in range {
            let guard = result?;
            let value: &str = guard.value();
            values.push(value.to_owned());
        }

        Ok(values)
    }

    /// List all values in a table (owned).
    ///
    /// # Errors
    ///
    /// Returns `DbError` if transaction or deserialization fails.
    #[inline]
    pub fn list_owned<V>(&self, table: &str) -> Result<Vec<V>, DbError>
    where
        V: rkyv::Archive,
        V::Archived: rkyv::Portable
            + for<'archived> rkyv::bytecheck::CheckBytes<
                rkyv::api::high::HighValidator<'archived, rkyv::rancor::Error>,
            > + rkyv::Deserialize<
                V,
                rkyv::api::high::HighDeserializer<rkyv::rancor::Error>,
            >,
    {
        let prefix = format!("{table}:");

        let tx = self.inner.begin_read()?;
        let table_ref = tx.open_table(DATA_TABLE)?;

        let mut results = Vec::new();
        for result in table_ref.range(prefix.as_str()..)? {
            let (key, value) = result?;
            if !key.value().starts_with(&prefix) {
                break;
            }

            let bytes: &[u8] = value.value();

            let mut aligned = AlignedVec::<16>::new();
            aligned.extend_from_slice(bytes);

            let archived =
                rkyv::access::<rkyv::Archived<V>, rkyv::rancor::Error>(
                    &aligned,
                )
                .map_err(|e| DbError::Deserialization(e.to_string()))?;
            let deserialized =
                rkyv::deserialize::<V, rkyv::rancor::Error>(archived)
                    .map_err(|e| DbError::Deserialization(e.to_string()))?;
            results.push(deserialized);
        }

        Ok(results)
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
