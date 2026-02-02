//! Zero-copy database layer using redb and rkyv.
//!
//! This module provides concrete types (not traits) for database operations,
//! following the `std::fs::File` pattern. Zero-copy reads are achieved through
//! closure-based APIs that keep transactions properly scoped.
//!
//! # Architecture
//!
//! - `Database` - Concrete type wrapping `redb::Database`
//! - `RkyvValue<T>` - Newtype wrapper for rkyv-serialized values (per ADR 0002)
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

use std::path::Path;

use redb::{ReadableDatabase as _, TableDefinition};

/// Concrete database type wrapping redb.
///
/// Provides zero-copy read/write primitives using rkyv serialization.
/// Follows the `std::fs::File` pattern with concrete methods instead of traits.
#[derive(Debug)]
#[non_exhaustive]
pub struct Database {
    inner: redb::Database,
}

/// Database error types.
#[expect(
    clippy::module_name_repetitions,
    reason = "Context-specific error name is intentional"
)]
#[non_exhaustive]
#[derive(Debug, thiserror::Error)]
pub enum DbError {
    /// Database operation failed.
    #[error("database error: {0}")]
    Database(String),

    /// Database file not found or cannot be opened.
    #[error("failed to open database: {0}")]
    Open(String),

    /// Key not found in database.
    #[error("key not found")]
    NotFound,

    /// Serialization failed.
    #[error("serialization error: {0}")]
    Serialization(String),

    /// Deserialization or validation failed.
    #[error("deserialization error: {0}")]
    Deserialization(String),

    /// Transaction failed.
    #[error("transaction error: {0}")]
    Transaction(String),

    /// Table operation failed.
    #[error("table error: {0}")]
    Table(String),
}

impl From<redb::DatabaseError> for DbError {
    #[inline]
    fn from(e: redb::DatabaseError) -> Self {
        Self::Database(e.to_string())
    }
}

impl From<redb::TransactionError> for DbError {
    #[inline]
    fn from(e: redb::TransactionError) -> Self {
        Self::Transaction(e.to_string())
    }
}

impl From<redb::TableError> for DbError {
    #[inline]
    fn from(e: redb::TableError) -> Self {
        Self::Table(e.to_string())
    }
}

impl From<redb::StorageError> for DbError {
    #[inline]
    fn from(e: redb::StorageError) -> Self {
        Self::Database(e.to_string())
    }
}

impl From<redb::CommitError> for DbError {
    #[inline]
    fn from(e: redb::CommitError) -> Self {
        Self::Transaction(e.to_string())
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

    /// Zero-copy read (HOT PATH for LSP).
    ///
    /// The closure receives a reference to the archived data directly from
    /// the database page. The transaction remains alive for the duration
    /// of the closure, ensuring memory safety without unsafe code.
    ///
    /// This is the default read method - it's fast and requires no allocations.
    /// For full deserialization (e.g., for mutations), use `get_owned()`.
    ///
    /// # Type Parameters
    ///
    /// - `V`: Value type (must implement `rkyv::Archive`)
    /// - `F`: Closure type
    /// - `R`: Return type of closure
    ///
    /// # Errors
    ///
    /// - `DbError::NotFound` - Key does not exist (returns `None`)
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
        const TABLE: TableDefinition<&str, &[u8]> =
            TableDefinition::new("data");
        let namespaced_key = format!("{table}:{key}");

        let tx = self.inner.begin_read()?;
        let table_ref = tx.open_table(TABLE)?;

        match table_ref.get(namespaced_key.as_str())? {
            Some(value) => {
                let bytes: &[u8] = value.value();
                let archived = rkyv::access::<
                    rkyv::Archived<V>,
                    rkyv::rancor::Error,
                >(bytes)
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
    /// because it allocates and copies data. Use this only when you need to
    /// mutate the value.
    ///
    /// # Type Parameters
    ///
    /// - `V`: Value type (must implement `rkyv::Archive` and
    ///   `rkyv::Deserialize`)
    ///
    /// # Errors
    ///
    /// - `DbError::NotFound` - Key does not exist (returns `None`, not error)
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
        const TABLE: TableDefinition<&str, &[u8]> =
            TableDefinition::new("data");
        let namespaced_key = format!("{table}:{key}");

        let tx = self.inner.begin_read()?;
        let table_ref = tx.open_table(TABLE)?;

        match table_ref.get(namespaced_key.as_str())? {
            Some(value) => {
                let bytes: &[u8] = value.value();
                let archived = rkyv::access::<
                    rkyv::Archived<V>,
                    rkyv::rancor::Error,
                >(bytes)
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
        const TABLE: TableDefinition<&str, &[u8]> =
            TableDefinition::new("data");
        let namespaced_key = format!("{table}:{key}");

        let bytes = rkyv::to_bytes::<rkyv::rancor::Error>(value)
            .map_err(|e| DbError::Serialization(e.to_string()))?;

        let tx = self.inner.begin_write()?;
        {
            let mut table_ref = tx.open_table(TABLE)?;
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
    #[expect(clippy::todo, reason = "Phase 6 - requires implementation")]
    pub fn delete(&self, _table: &str, _key: &str) -> Result<bool, DbError> {
        // Phase 6 implementation:
        // 1. Begin write transaction
        // 2. Open table
        // 3. Remove key
        // 4. Commit transaction
        // 5. Return whether value existed
        todo!("Implement delete")
    }

    /// Execute multiple writes in a batch with single fsync.
    ///
    /// Per ADR 0002: Uses `Durability::None` for batch operations,
    /// then a final commit with `Durability::Immediate` for durability.
    ///
    /// The closure receives a `&Database` for performing operations.
    /// All operations in the batch share the same transaction.
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
    /// // db.batch_write(|db| {
    /// //     for i in 0..1000 {
    /// //         db.put("notes", &format!("note_{}", i), &note)?;
    /// //     }
    /// //     Ok(())
    /// // })?;
    /// # Ok(())
    /// # }
    /// ```
    #[inline]
    #[expect(clippy::todo, reason = "Phase 6 - requires implementation")]
    pub fn batch_write<F>(&self, _f: F) -> Result<(), DbError>
    where
        F: FnOnce(&Self) -> Result<(), DbError>,
    {
        // Phase 6 implementation:
        // Note: redb 3.1 API doesn't expose set_durability on WriteTransaction
        // We'll need to implement this differently or use a different approach
        // For now, this is equivalent to individual writes
        todo!("Implement batch write with deferred fsync")
    }

    /// Insert a value into a multimap (1:N relationship).
    ///
    /// Multimap tables allow multiple values per key. Useful for indexes
    /// like tags → notes or backlinks.
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
    /// // db.multimap_insert("tag_index", "rust", "note-1")?;
    /// // db.multimap_insert("tag_index", "rust", "note-2")?;
    /// # Ok(())
    /// # }
    /// ```
    #[inline]
    #[expect(clippy::todo, reason = "Phase 6 - requires implementation")]
    pub fn multimap_insert(
        &self,
        _table: &str,
        _key: &str,
        _value: &str,
    ) -> Result<(), DbError> {
        // Phase 6 implementation:
        // 1. Begin write transaction
        // 2. Open multimap table
        // 3. Insert key-value pair
        // 4. Commit transaction
        todo!("Implement multimap_insert")
    }

    /// Remove a value from a multimap.
    ///
    /// Returns `true` if the value was removed, `false` if not found.
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
    /// // let removed = db.multimap_remove("tag_index", "rust", "note-1")?;
    /// # Ok(())
    /// # }
    /// ```
    #[inline]
    #[expect(clippy::todo, reason = "Phase 6 - requires implementation")]
    pub fn multimap_remove(
        &self,
        _table: &str,
        _key: &str,
        _value: &str,
    ) -> Result<bool, DbError> {
        // Phase 6 implementation:
        // 1. Begin write transaction
        // 2. Open multimap table
        // 3. Remove key-value pair
        // 4. Commit transaction
        // 5. Return whether value existed
        todo!("Implement multimap_remove")
    }

    /// Get all values for a key from a multimap.
    ///
    /// Returns a `Vec<String>` containing all values. For large result sets,
    /// consider using a closure-based API in the future.
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
    /// // let note_ids = db.multimap_get("tag_index", "rust")?;
    /// # Ok(())
    /// # }
    /// ```
    #[inline]
    #[expect(clippy::todo, reason = "Phase 6 - requires implementation")]
    pub fn multimap_get(
        &self,
        _table: &str,
        _key: &str,
    ) -> Result<Vec<String>, DbError> {
        // Phase 6 implementation:
        // 1. Begin read transaction
        // 2. Open multimap table
        // 3. Get all values for key
        // 4. Collect into Vec<String>
        // 5. Return values
        todo!("Implement multimap_get")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn database_can_be_constructed() {
        // This test verifies the API compiles and basic types are correct
        let _result: Result<Database, DbError> =
            Database::open(Path::new("/tmp/test.db"));
    }

    #[test]
    fn db_error_converts_from_redb_errors() {
        // Verify From impls compile
        let db_err = redb::DatabaseError::from(std::io::Error::new(
            std::io::ErrorKind::NotFound,
            "test",
        ));
        let result: DbError = db_err.into();
        assert!(result.to_string().contains("database error"));
    }
}
