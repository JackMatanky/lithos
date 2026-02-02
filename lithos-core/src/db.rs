//! Zero-copy database layer using redb and rkyv.
//!
//! This module provides concrete types (not traits) for database operations,
//! following the `std::fs::File` pattern. Zero-copy reads are achieved through
//! `ArchivedGuard` which wraps redb's `AccessGuard`.
//!
//! # Architecture
//!
//! - `Database` - Concrete type wrapping `redb::Database`
//! - `ArchivedGuard<'txn, V>` - Zero-copy Deref wrapper for archived data
//! - Generic methods (not macros) for type-safe operations
//! - Sync-first design (no async overhead)
//!
//! # Phase 4 Status
//!
//! This is a **stub implementation** with proper type signatures and
//! documentation. The actual redb/rkyv integration will be implemented in Phase
//! 6.
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

use std::{marker::PhantomData, ops::Deref, path::Path};

/// Concrete database type wrapping redb.
///
/// Provides zero-copy read/write primitives using rkyv serialization.
/// Follows the `std::fs::File` pattern with concrete methods instead of traits.
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
#[non_exhaustive]
pub struct Database {
    #[expect(dead_code, reason = "Will be used in Phase 6 implementation")]
    inner: redb::Database,
}

/// Zero-copy guard that provides Deref access to archived data.
///
/// The guard borrows the transaction and provides safe access to rkyv-archived
/// data directly from the database page (zero-copy). The lifetime `'txn` is
/// tied to the underlying redb transaction.
///
/// # Type Parameters
///
/// - `V`: The original type that was archived (must implement `rkyv::Archive`)
///
/// # Phase 4 Note
///
/// This is a stub type. The actual implementation will be added in Phase 6.
pub struct ArchivedGuard<'txn, V> {
    _phantom: PhantomData<(&'txn (), V)>,
}

#[expect(
    clippy::elidable_lifetime_names,
    reason = "Lifetime is semantically important: tied to transaction"
)]
impl<'txn, V> Deref for ArchivedGuard<'txn, V>
where
    V: rkyv::Archive,
{
    type Target = rkyv::Archived<V>;

    #[inline]
    #[expect(
        clippy::todo,
        reason = "Phase 6 stub - will implement zero-copy deref"
    )]
    fn deref(&self) -> &Self::Target {
        // Phase 6: Will use unsafe { rkyv::access_unchecked(self.guard.value())
        // } after validation in get_archived()
        todo!("Implement in Phase 6: Zero-copy Deref to archived data")
    }
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

    /// Data alignment error (rkyv requires proper alignment).
    #[error("data misaligned for zero-copy access")]
    Misaligned,

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
    /// Returns a guard with lifetime tied to the transaction. The guard
    /// provides `Deref` access to the archived data directly from the
    /// database page.
    ///
    /// # Type Parameters
    ///
    /// - `K`: Key type
    /// - `V`: Value type (must implement `rkyv::Archive`)
    ///
    /// # Errors
    ///
    /// - `DbError::NotFound` - Key does not exist
    /// - `DbError::Misaligned` - Data is not properly aligned for zero-copy
    ///   access
    /// - `DbError::Deserialization` - Data validation failed
    /// - `DbError::Transaction` - Transaction or table operation failed
    ///
    /// # Phase 4 Note
    ///
    /// This is a stub. Phase 6 will implement:
    /// 1. Begin read transaction
    /// 2. Open table
    /// 3. Serialize key
    /// 4. Get `AccessGuard` from redb
    /// 5. Validate alignment
    /// 6. Validate bytes with `rkyv::check_archived_root`
    /// 7. Return `ArchivedGuard`
    #[inline]
    #[expect(
        clippy::todo,
        reason = "Phase 6 stub - will implement zero-copy read"
    )]
    pub fn get_archived<K, V>(
        &self,
        _table: &str,
        _key: &K,
    ) -> Result<ArchivedGuard<'_, V>, DbError>
    where
        V: rkyv::Archive,
    {
        todo!("Implement in Phase 6: Zero-copy read with rkyv validation")
    }

    /// Full deserialization (COLD PATH for mutations).
    ///
    /// Delegates to `get_archived()` and deserializes the result. Use this when
    /// you need an owned value for mutation.
    ///
    /// # Type Parameters
    ///
    /// - `K`: Key type
    /// - `V`: Value type (must implement `rkyv::Archive` + deserialization)
    ///
    /// # Errors
    ///
    /// Same as `get_archived()`, plus deserialization errors.
    ///
    /// # Phase 4 Note
    ///
    /// This is a stub. Phase 6 will implement:
    /// 1. Call `get_archived()`
    /// 2. Deserialize if found
    /// 3. Return `None` if `NotFound`
    #[inline]
    #[expect(
        clippy::todo,
        reason = "Phase 6 stub - will implement deserialization"
    )]
    pub fn get<K, V>(
        &self,
        _table: &str,
        _key: &K,
    ) -> Result<Option<V>, DbError>
    where
        V: rkyv::Archive,
    {
        todo!("Implement in Phase 6: Full deserialization via get_archived")
    }

    /// Zero-copy write using `insert_reserve` (per ADR 0002).
    ///
    /// Provides a mutable slice directly to the database page for writing.
    /// The closure should serialize data directly into the provided buffer.
    ///
    /// # Type Parameters
    ///
    /// - `K`: Key type
    /// - `F`: Closure that writes serialized data to the provided buffer
    ///
    /// # Errors
    ///
    /// - `DbError::Serialization` - Key serialization failed
    /// - `DbError::Transaction` - Transaction or table operation failed
    /// - Propagates errors from the write closure
    ///
    /// # Phase 4 Note
    ///
    /// This is a stub. Phase 6 will implement:
    /// 1. Begin write transaction
    /// 2. Open table
    /// 3. Serialize key
    /// 4. Call `table.insert_reserve()`
    /// 5. Pass mutable slice to `write_fn`
    /// 6. Commit transaction
    #[inline]
    #[expect(
        clippy::todo,
        reason = "Phase 6 stub - will implement zero-copy write"
    )]
    pub fn put_reserve<K, F>(
        &self,
        _table: &str,
        _key: &K,
        _value_size: usize,
        _write_fn: F,
    ) -> Result<(), DbError>
    where
        F: FnOnce(&mut [u8]) -> Result<(), DbError>,
    {
        todo!("Implement in Phase 6: Zero-copy write using insert_reserve")
    }

    /// Convenience wrapper for `put_reserve` (allocates temp buffer).
    ///
    /// Serializes the value and writes it to the database. This is less
    /// efficient than `put_reserve()` but more convenient.
    ///
    /// # Type Parameters
    ///
    /// - `K`: Key type
    /// - `V`: Value type
    ///
    /// # Errors
    ///
    /// Same as `put_reserve()`.
    ///
    /// # Phase 4 Note
    ///
    /// This is a stub. Phase 6 will implement:
    /// 1. Serialize value to calculate size
    /// 2. Call `put_reserve` with serialized data
    #[inline]
    #[expect(
        clippy::todo,
        reason = "Phase 6 stub - will implement put wrapper"
    )]
    pub fn put<K, V>(
        &self,
        _table: &str,
        _key: &K,
        _value: &V,
    ) -> Result<(), DbError> {
        todo!("Implement in Phase 6: Convenience wrapper for put_reserve")
    }

    /// `MultimapTable` for 1:N relations (per ADR 0002).
    ///
    /// Inserts a key-value pair into a multimap table, allowing multiple values
    /// per key. Useful for indexes like tags→notes or backlinks.
    ///
    /// # Type Parameters
    ///
    /// - `K`: Key type
    /// - `V`: Value type
    ///
    /// # Errors
    ///
    /// - `DbError::Serialization` - Serialization failed
    /// - `DbError::Transaction` - Transaction or table operation failed
    ///
    /// # Phase 4 Note
    ///
    /// This is a stub. Phase 6 will implement:
    /// 1. Begin write transaction
    /// 2. Open multimap table
    /// 3. Serialize key and value
    /// 4. Insert into multimap
    /// 5. Commit transaction
    #[inline]
    #[expect(
        clippy::todo,
        reason = "Phase 6 stub - will implement multimap insert"
    )]
    pub fn multimap_insert<K, V>(
        &self,
        _table: &str,
        _key: &K,
        _value: &V,
    ) -> Result<(), DbError> {
        todo!("Implement in Phase 6: Multimap insert for 1:N relations")
    }

    /// Retrieve all values for a key from a multimap table.
    ///
    /// Returns an iterator of `ArchivedGuard` for zero-copy access.
    ///
    /// # Type Parameters
    ///
    /// - `K`: Key type
    /// - `V`: Value type (must implement `rkyv::Archive`)
    ///
    /// # Errors
    ///
    /// - `DbError::Transaction` - Transaction or table operation failed
    /// - `DbError::Deserialization` - Data validation failed
    ///
    /// # Phase 4 Note
    ///
    /// This is a stub. Phase 6 will implement:
    /// 1. Begin read transaction
    /// 2. Open multimap table
    /// 3. Serialize key
    /// 4. Get range iterator
    /// 5. Map to `ArchivedGuard` with validation
    #[inline]
    pub fn multimap_get<K, V>(
        &self,
        _table: &str,
        _key: &K,
    ) -> Result<impl Iterator<Item = ArchivedGuard<'_, V>>, DbError>
    where
        V: rkyv::Archive,
    {
        // Stub: return empty iterator
        // Phase 6 will return actual iterator from redb multimap range
        #[expect(clippy::iter_skip_zero, reason = "Stub implementation")]
        Ok(std::iter::empty::<ArchivedGuard<'_, V>>().skip(0))
    }

    /// Bulk write with `Durability::None` (per ADR 0002).
    ///
    /// Executes a batch of writes with deferred fsync for better performance.
    /// A final fsync is performed after the batch completes.
    ///
    /// # Type Parameters
    ///
    /// - `F`: Closure that performs batch writes
    ///
    /// # Errors
    ///
    /// - `DbError::Transaction` - Transaction operation failed
    /// - Propagates errors from the batch closure
    ///
    /// # Phase 4 Note
    ///
    /// This is a stub. Phase 6 will implement:
    /// 1. Begin write transaction with `Durability::None`
    /// 2. Execute `batch_fn`
    /// 3. Commit with deferred fsync
    /// 4. Final transaction with `Durability::Immediate`
    #[inline]
    #[expect(
        clippy::todo,
        reason = "Phase 6 stub - will implement batch write"
    )]
    pub fn batch_write<F>(&self, _batch_fn: F) -> Result<(), DbError>
    where
        F: FnOnce(&Self) -> Result<(), DbError>,
    {
        todo!("Implement in Phase 6: Batch write with deferred fsync")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn database_can_be_constructed() {
        // This test verifies the stub compiles and basic types are correct
        // Real tests will be added in Phase 6 implementation
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
