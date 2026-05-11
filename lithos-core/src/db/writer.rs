//! Database write operations.
//!
//! This module contains all write and batch-write operations, keeping
//! transactions properly scoped and centralized.

use redb::{MultimapTableDefinition, ReadableTable as _, TableDefinition};

use super::{Database, DbError};
use crate::utils::UuidV7;

#[inline]
fn with_uuid_v7_key<R>(id: UuidV7, f: impl FnOnce(&str) -> R) -> R {
    let mut buf = [0u8; 36];
    let key = id.as_uuid().hyphenated().encode_lower(&mut buf);
    f(key)
}

impl Database {
    /// Insert or update a value in a table definition.
    ///
    /// Serializes the value using rkyv and writes it to the database.
    /// This replaces any existing value for the given key.
    ///
    /// # Errors
    ///
    /// - `DbError::Serialization` - Serialization failed
    /// - `DbError::Transaction` - Transaction or table operation failed
    #[inline]
    pub fn put<V>(
        &self,
        table: TableDefinition<&str, &[u8]>,
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
        serialize_and_put::<V>(&self.inner, table, key, value)
    }

    /// Insert or update a value with UUID key in a table definition.
    ///
    /// Uses stack-allocated buffer to avoid heap allocation from UUID
    /// formatting.
    ///
    /// # Errors
    ///
    /// - `DbError::Serialization` - Serialization failed
    /// - `DbError::Transaction` - Transaction or table operation failed
    #[inline]
    pub fn put_by_uuid<V>(
        &self,
        table: TableDefinition<&str, &[u8]>,
        id: UuidV7,
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
        with_uuid_v7_key(id, |key| {
            serialize_and_put::<V>(&self.inner, table, key, value)
        })
    }

    /// Delete a value by key in a table definition.
    ///
    /// Returns `true` if a value was deleted, `false` if key didn't exist.
    ///
    /// # Errors
    ///
    /// - `DbError::Transaction` - Transaction or table operation failed
    #[inline]
    pub fn delete(
        &self,
        table: TableDefinition<&str, &[u8]>,
        key: &str,
    ) -> Result<bool, DbError> {
        delete_key(&self.inner, table, key)
    }

    /// Delete a value by UUID key in a table definition.
    ///
    /// Uses stack-allocated buffer to avoid heap allocation from UUID
    /// formatting.
    ///
    /// Returns `true` if a value was deleted, `false` if key didn't exist.
    ///
    /// # Errors
    ///
    /// - `DbError::Transaction` - Transaction or table operation failed
    #[inline]
    pub fn delete_by_uuid(
        &self,
        table: TableDefinition<&str, &[u8]>,
        id: UuidV7,
    ) -> Result<bool, DbError> {
        with_uuid_v7_key(id, |key| delete_key(&self.inner, table, key))
    }

    /// Execute multiple writes in a batch with a single commit.
    ///
    /// The closure receives a [`BatchWriter`] for performing operations.
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
    /// //     let table = redb::TableDefinition::new("notes");
    /// //     for i in 0..1000 {
    /// //         batch.put(table, &format!("note_{i}"), &note)?;
    /// //     }
    /// //     Ok(())
    /// // })?;
    /// # Ok(())
    /// # }
    /// ```
    #[inline]
    pub fn batch_write<F>(&self, f: F) -> Result<(), DbError>
    where
        F: FnOnce(&mut BatchWriter) -> Result<(), DbError>,
    {
        batch_write_impl(&self.inner, f)
    }

    /// Execute a read-write unit of work with both read and write operations.
    ///
    /// This method allows atomic read and write operations within a single
    /// transaction, ensuring consistency for operations like "read current
    /// value, compute next value, write new value".
    ///
    /// The closure receives a [`ReadWriteUnitOfWork`] that supports both read
    /// and write operations.
    ///
    /// # Errors
    ///
    /// - `DbError::Transaction` - Transaction operation failed
    /// - Propagates errors from read or write operations
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use lithos_core::db::Database;
    /// # use std::path::Path;
    /// # use redb::TableDefinition;
    /// # fn main() -> Result<(), lithos_core::db::DbError> {
    /// let db = Database::open(Path::new("/tmp/test.db"))?;
    /// let table = TableDefinition::new("counters");
    /// db.read_write_unit_of_work(|tx| {
    ///     // Read current value
    ///     let current: Option<u64> = tx.get_owned(table, "counter")?;
    ///     let next = current.unwrap_or(0) + 1;
    ///     // Write new value
    ///     tx.put(table, "counter", &next)?;
    ///     Ok(next)
    /// })?;
    /// # Ok(())
    /// # }
    /// ```
    #[inline]
    pub fn read_write_unit_of_work<R, F>(&self, f: F) -> Result<R, DbError>
    where
        F: FnOnce(&mut ReadWriteUnitOfWork) -> Result<R, DbError>,
    {
        read_write_uow_impl(&self.inner, f)
    }

    /// Insert a value into a multimap table definition (1:N relationship).
    ///
    /// # Errors
    ///
    /// - `DbError::Transaction` - Transaction or table operation failed
    #[inline]
    pub fn multimap_insert(
        &self,
        table: MultimapTableDefinition<&str, &str>,
        key: &str,
        value: &str,
    ) -> Result<(), DbError> {
        multimap_insert_impl(&self.inner, table, key, value)
    }

    /// Remove a value from a multimap table definition.
    ///
    /// Returns `true` if the value was removed, `false` if not found.
    ///
    /// # Errors
    ///
    /// - `DbError::Transaction` - Transaction or table operation failed
    #[inline]
    pub fn multimap_remove(
        &self,
        table: MultimapTableDefinition<&str, &str>,
        key: &str,
        value: &str,
    ) -> Result<bool, DbError> {
        multimap_remove_impl(&self.inner, table, key, value)
    }
}

/// A single write transaction for batching many operations.
///
/// This is intentionally scoped to a closure (see `Database::batch_write`) so
/// callers cannot accidentally hold a transaction across unrelated work.
pub struct BatchWriter {
    tx: redb::WriteTransaction,
}

impl BatchWriter {
    #[inline]
    pub(super) fn new(tx: redb::WriteTransaction) -> Self {
        Self {
            tx,
        }
    }

    #[inline]
    pub(super) fn commit(self) -> Result<(), DbError> {
        self.tx.commit()?;
        Ok(())
    }

    /// Insert or update a value within the batch transaction using a table
    /// definition.
    ///
    /// # Errors
    ///
    /// Returns `DbError` if serialization fails or if the underlying redb table
    /// operation fails.
    #[inline]
    pub fn put<V>(
        &mut self,
        table: TableDefinition<&str, &[u8]>,
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
        serialize_and_put_tx::<V>(&mut self.tx, table, key, value)
    }

    /// Delete a value by key within the batch transaction using a table
    /// definition.
    ///
    /// # Errors
    ///
    /// Returns `DbError` if the underlying redb table operation fails.
    #[inline]
    pub fn delete(
        &mut self,
        table: TableDefinition<&str, &[u8]>,
        key: &str,
    ) -> Result<bool, DbError> {
        delete_key_tx(&mut self.tx, table, key)
    }

    /// Insert or update a value with UUID key within the batch transaction.
    ///
    /// # Errors
    ///
    /// Returns `DbError` if serialization or transaction fails.
    #[inline]
    pub fn put_by_uuid<V>(
        &mut self,
        table: TableDefinition<&str, &[u8]>,
        id: UuidV7,
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
        with_uuid_v7_key(id, |key| {
            serialize_and_put_tx::<V>(&mut self.tx, table, key, value)
        })
    }

    /// Delete a value by UUID key within the batch transaction.
    ///
    /// # Errors
    ///
    /// Returns `DbError` if the underlying redb table operation fails.
    #[inline]
    pub fn delete_by_uuid(
        &mut self,
        table: TableDefinition<&str, &[u8]>,
        id: UuidV7,
    ) -> Result<bool, DbError> {
        with_uuid_v7_key(id, |key| delete_key_tx(&mut self.tx, table, key))
    }

    /// Insert a value into a multimap within the batch transaction using a
    /// table definition.
    ///
    /// # Errors
    ///
    /// Returns `DbError` if the underlying redb multimap table operation fails.
    #[inline]
    pub fn multimap_insert(
        &mut self,
        table: MultimapTableDefinition<&str, &str>,
        key: &str,
        value: &str,
    ) -> Result<(), DbError> {
        multimap_insert_tx(&mut self.tx, table, key, value)
    }

    /// Remove a value from a multimap within the batch transaction using a
    /// table definition.
    ///
    /// # Errors
    ///
    /// Returns `DbError` if the underlying redb multimap table operation fails.
    #[inline]
    pub fn multimap_remove(
        &mut self,
        table: MultimapTableDefinition<&str, &str>,
        key: &str,
        value: &str,
    ) -> Result<bool, DbError> {
        multimap_remove_tx(&mut self.tx, table, key, value)
    }

    /// Insert a serialized value into a multimap with byte-slice values.
    ///
    /// For multimaps storing complex types via rkyv serialization.
    ///
    /// # Errors
    ///
    /// Returns `DbError` if the underlying redb multimap table operation fails.
    #[inline]
    pub fn multimap_insert_bytes(
        &mut self,
        table: MultimapTableDefinition<&str, &[u8]>,
        key: &str,
        value: &[u8],
    ) -> Result<(), DbError> {
        let mut tbl = self.tx.open_multimap_table(table)?;
        tbl.insert(key, value)?;
        Ok(())
    }

    /// Remove a serialized value from a multimap with byte-slice values.
    ///
    /// For multimaps storing complex types via rkyv serialization.
    ///
    /// # Errors
    ///
    /// Returns `DbError` if the underlying redb multimap table operation fails.
    #[inline]
    pub fn multimap_remove_bytes(
        &mut self,
        table: MultimapTableDefinition<&str, &[u8]>,
        key: &str,
        value: &[u8],
    ) -> Result<bool, DbError> {
        let mut tbl = self.tx.open_multimap_table(table)?;
        Ok(tbl.remove(key, value)?)
    }
}

/// A single read-write unit of work supporting both read and write operations.
///
/// This is intentionally scoped to a closure (see
/// `Database::read_write_unit_of_work`) so callers cannot accidentally hold a
/// transaction across unrelated work.
///
/// Unlike [`BatchWriter`], this type supports read operations for atomic
/// read-compute-write patterns.
pub struct ReadWriteUnitOfWork {
    tx: redb::WriteTransaction,
}

impl ReadWriteUnitOfWork {
    #[inline]
    pub(super) fn new(tx: redb::WriteTransaction) -> Self {
        Self {
            tx,
        }
    }

    #[inline]
    pub(super) fn commit(self) -> Result<(), DbError> {
        self.tx.commit()?;
        Ok(())
    }

    /// Read a value from a table and deserialize to an owned value.
    ///
    /// # Errors
    ///
    /// Returns `DbError` if deserialization or transaction fails.
    #[inline]
    pub fn get_owned<V>(
        &self,
        table: TableDefinition<&str, &[u8]>,
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
        get_owned_tx(&self.tx, table, key)
    }

    /// Read a value with UUID key and deserialize to an owned value.
    ///
    /// # Errors
    ///
    /// Returns `DbError` if deserialization or transaction fails.
    #[inline]
    pub fn get_owned_by_uuid<V>(
        &self,
        table: TableDefinition<&str, &[u8]>,
        id: UuidV7,
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
        with_uuid_v7_key(id, |key| get_owned_tx(&self.tx, table, key))
    }

    /// Insert or update a value within the transaction using a table
    /// definition.
    ///
    /// # Errors
    ///
    /// Returns `DbError` if serialization fails or if the underlying redb table
    /// operation fails.
    #[inline]
    pub fn put<V>(
        &mut self,
        table: TableDefinition<&str, &[u8]>,
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
        serialize_and_put_tx::<V>(&mut self.tx, table, key, value)
    }

    /// Insert or update a value with UUID key within the transaction.
    ///
    /// # Errors
    ///
    /// Returns `DbError` if serialization or transaction fails.
    #[inline]
    pub fn put_by_uuid<V>(
        &mut self,
        table: TableDefinition<&str, &[u8]>,
        id: UuidV7,
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
        with_uuid_v7_key(id, |key| {
            serialize_and_put_tx::<V>(&mut self.tx, table, key, value)
        })
    }

    /// Delete a value by key within the transaction using a table definition.
    ///
    /// # Errors
    ///
    /// Returns `DbError` if the underlying redb table operation fails.
    #[inline]
    pub fn delete(
        &mut self,
        table: TableDefinition<&str, &[u8]>,
        key: &str,
    ) -> Result<bool, DbError> {
        delete_key_tx(&mut self.tx, table, key)
    }

    /// Delete a value by UUID key within the transaction.
    ///
    /// # Errors
    ///
    /// Returns `DbError` if the underlying redb table operation fails.
    #[inline]
    pub fn delete_by_uuid(
        &mut self,
        table: TableDefinition<&str, &[u8]>,
        id: UuidV7,
    ) -> Result<bool, DbError> {
        with_uuid_v7_key(id, |key| delete_key_tx(&mut self.tx, table, key))
    }

    /// Scan entries matching a key prefix within the transaction.
    ///
    /// This enables atomic read-scan-compute-write patterns, such as finding
    /// the maximum version and incrementing it atomically.
    ///
    /// # Errors
    ///
    /// Returns `DbError` if the scan or deserialization fails.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use lithos_core::db::Database;
    /// # use std::path::Path;
    /// # use redb::TableDefinition;
    /// # fn main() -> Result<(), lithos_core::db::DbError> {
    /// # let db = Database::open(Path::new("/tmp/test.db"))?;
    /// # let table = TableDefinition::new("configs");
    /// # let vault_id = "vault-123";
    /// // Atomically find max version and write next version
    /// db.read_write_unit_of_work(|tx| {
    ///     let prefix = format!("{}:", vault_id);
    ///     let versions: Vec<(String, u64)> = tx.scan_range(table, &prefix)?;
    ///     let max = versions.iter().map(|(_, v)| v).max();
    ///     let next = max.map_or(1, |v| v + 1);
    ///     tx.put(table, &format!("{}:{}", vault_id, next), &next)?;
    ///     Ok(next)
    /// })?;
    /// # Ok(())
    /// # }
    /// ```
    #[inline]
    pub fn scan_range<V>(
        &self,
        table: TableDefinition<&str, &[u8]>,
        key_prefix: &str,
    ) -> Result<Vec<(String, V)>, DbError>
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
        scan_range_write_tx::<V>(&self.tx, table, key_prefix)
    }
}

// Private helper functions for internal use

/// Serialize a value and write it under the provided key in a table definition.
fn serialize_and_put<V>(
    db: &redb::Database,
    table: TableDefinition<&str, &[u8]>,
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
    let mut tx = db.begin_write()?;
    serialize_and_put_tx::<V>(&mut tx, table, key, value)?;
    tx.commit()?;
    Ok(())
}

/// Serialize a value and write it under the provided key in a table definition
/// within a transaction.
fn serialize_and_put_tx<V>(
    tx: &mut redb::WriteTransaction,
    table: TableDefinition<&str, &[u8]>,
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
    let bytes = rkyv::to_bytes::<rkyv::rancor::Error>(value)
        .map_err(|e| DbError::Serialization(e.to_string()))?;
    {
        let mut table_ref = tx.open_table(table)?;
        table_ref.insert(key, bytes.as_slice())?;
    };
    Ok(())
}

/// Delete a value by key in a table definition.
fn delete_key(
    db: &redb::Database,
    table: TableDefinition<&str, &[u8]>,
    key: &str,
) -> Result<bool, DbError> {
    let mut tx = db.begin_write()?;
    let existed = delete_key_tx(&mut tx, table, key)?;
    tx.commit()?;
    Ok(existed)
}

/// Delete a value by key in a table definition within a transaction.
fn delete_key_tx(
    tx: &mut redb::WriteTransaction,
    table: TableDefinition<&str, &[u8]>,
    key: &str,
) -> Result<bool, DbError> {
    let existed = {
        let mut table_ref = tx.open_table(table)?;
        table_ref.remove(key)?.is_some()
    };
    Ok(existed)
}

/// Insert a value into a multimap table definition.
fn multimap_insert_impl(
    db: &redb::Database,
    table: MultimapTableDefinition<&str, &str>,
    key: &str,
    value: &str,
) -> Result<(), DbError> {
    let mut tx = db.begin_write()?;
    multimap_insert_tx(&mut tx, table, key, value)?;
    tx.commit()?;
    Ok(())
}

/// Insert a value into a multimap table definition in a transaction.
fn multimap_insert_tx(
    tx: &mut redb::WriteTransaction,
    table: MultimapTableDefinition<&str, &str>,
    key: &str,
    value: &str,
) -> Result<(), DbError> {
    {
        let mut tbl = tx.open_multimap_table(table)?;
        tbl.insert(key, value)?;
    };
    Ok(())
}

/// Remove a value from a multimap table definition.
fn multimap_remove_impl(
    db: &redb::Database,
    table: MultimapTableDefinition<&str, &str>,
    key: &str,
    value: &str,
) -> Result<bool, DbError> {
    let mut tx = db.begin_write()?;
    let removed = multimap_remove_tx(&mut tx, table, key, value)?;
    tx.commit()?;
    Ok(removed)
}

/// Remove a value from a multimap table definition in a transaction.
fn multimap_remove_tx(
    tx: &mut redb::WriteTransaction,
    table: MultimapTableDefinition<&str, &str>,
    key: &str,
    value: &str,
) -> Result<bool, DbError> {
    let removed = {
        let mut tbl = tx.open_multimap_table(table)?;
        tbl.remove(key, value)?
    };
    Ok(removed)
}

/// Execute a batch of write operations with a single commit.
fn batch_write_impl<F>(db: &redb::Database, f: F) -> Result<(), DbError>
where
    F: FnOnce(&mut BatchWriter) -> Result<(), DbError>,
{
    let tx = db.begin_write()?;
    let mut batch = BatchWriter::new(tx);

    f(&mut batch)?;
    batch.commit()?;
    Ok(())
}

/// Execute a read-write unit of work with both read and write operations.
fn read_write_uow_impl<R, F>(db: &redb::Database, f: F) -> Result<R, DbError>
where
    F: FnOnce(&mut ReadWriteUnitOfWork) -> Result<R, DbError>,
{
    let tx = db.begin_write()?;
    let mut tx = ReadWriteUnitOfWork::new(tx);

    let result = f(&mut tx)?;
    tx.commit()?;
    Ok(result)
}

/// Read and deserialize a value from a table within a transaction.
fn get_owned_tx<V>(
    tx: &redb::WriteTransaction,
    table: TableDefinition<&str, &[u8]>,
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
    let table_ref = match tx.open_table(table) {
        Ok(table_ref) => table_ref,
        Err(redb::TableError::TableDoesNotExist(_)) => return Ok(None),
        Err(err) => return Err(err.into()),
    };

    match table_ref.get(key)? {
        Some(value) => {
            let bytes: &[u8] = value.value();

            let mut aligned: rkyv::util::AlignedVec<16> =
                rkyv::util::AlignedVec::new();
            aligned.extend_from_slice(bytes);

            let archived =
                rkyv::access::<rkyv::Archived<V>, rkyv::rancor::Error>(
                    &aligned,
                )
                .map_err(|e| DbError::Deserialization(e.to_string()))?;
            let deserialized =
                rkyv::deserialize::<V, rkyv::rancor::Error>(archived)
                    .map_err(|e| DbError::Deserialization(e.to_string()))?;
            Ok(Some(deserialized))
        }
        None => Ok(None),
    }
}

/// Scan entries in a table matching a key prefix within a write transaction.
///
/// This is similar to `scan_range_tx` in reader.rs but works with
/// `WriteTransaction` instead of `ReadTransaction`. Both transaction types
/// support the same table read operations.
fn scan_range_write_tx<V>(
    tx: &redb::WriteTransaction,
    table: TableDefinition<&str, &[u8]>,
    key_prefix: &str,
) -> Result<Vec<(String, V)>, DbError>
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
    use redb::TableError;

    let table_ref = match tx.open_table(table) {
        Ok(table_ref) => table_ref,
        Err(TableError::TableDoesNotExist(_)) => return Ok(Vec::new()),
        Err(err) => return Err(err.into()),
    };

    // Compute the exclusive end bound for the range scan.
    // For prefix "abc", we want to scan ["abc", "abd"), which captures all keys
    // starting with "abc".
    let end_bound = next_prefix(key_prefix);

    let mut results = Vec::new();
    let range = table_ref.range(key_prefix..end_bound.as_str())?;
    for result in range {
        let (key, value): (redb::AccessGuard<&str>, redb::AccessGuard<&[u8]>) =
            result?;
        let key_str = key.value().to_owned();
        let bytes: &[u8] = value.value();

        let mut aligned = rkyv::util::AlignedVec::<16>::new();
        aligned.extend_from_slice(bytes);

        let archived =
            rkyv::access::<rkyv::Archived<V>, rkyv::rancor::Error>(&aligned)
                .map_err(|e| DbError::Deserialization(e.to_string()))?;
        let deserialized =
            rkyv::deserialize::<V, rkyv::rancor::Error>(archived)
                .map_err(|e| DbError::Deserialization(e.to_string()))?;
        results.push((key_str, deserialized));
    }

    Ok(results)
}

/// Compute the next string in lexicographic order for range queries.
///
/// This is used to create exclusive upper bounds for prefix scans.
/// For example, `next_prefix("user:")` returns `"user;"` which allows
/// scanning all keys starting with `"user:"`.
fn next_prefix(prefix: &str) -> String {
    let bytes = prefix.as_bytes();

    // Find the last byte that can be incremented (not 0xFF)
    for i in (0..bytes.len()).rev() {
        if let Some(&byte) = bytes.get(i)
            && byte < 255
        {
            let mut next =
                bytes.get(..=i).map_or_else(|| bytes.to_vec(), <[u8]>::to_vec);
            if let Some(last) = next.get_mut(i) {
                *last = last.saturating_add(1);
            }
            // Safe because we're only incrementing valid UTF-8 bytes
            return String::from_utf8(next)
                .unwrap_or_else(|_| format!("{prefix}\u{FFFF}"));
        }
    }

    // All bytes are 0xFF or empty string, append a high Unicode character
    format!("{prefix}\u{FFFF}")
}

#[cfg(test)]
mod tests {
    use super::*;

    mod write_operations {
        use super::*;

        mod tables {
            use super::*;

            pub(super) const USERS_TABLE: TableDefinition<&str, &[u8]> =
                TableDefinition::new("users");
            pub(super) const TAGS_TABLE: MultimapTableDefinition<&str, &str> =
                MultimapTableDefinition::new("tags");
        }

        mod put {
            use super::{tables::USERS_TABLE, *};

            #[test]
            fn inserts_value() {
                let (_temp, db) = temp_db().expect("temp db");
                let value = TestValue {
                    id: 1,
                    name: "Alice".to_owned(),
                };

                db.put(USERS_TABLE, "alice", &value).expect("put");

                let fetched: Option<TestValue> =
                    db.get_owned(USERS_TABLE, "alice").expect("get_owned");
                assert_eq!(fetched, Some(value));
            }

            #[test]
            fn inserts_value_in_table() {
                let (_temp, db) = temp_db().expect("temp db");
                let value = TestValue {
                    id: 4,
                    name: "Daria".to_owned(),
                };

                db.put(USERS_TABLE, "daria", &value).expect("put");

                let fetched: Option<TestValue> =
                    db.get_owned(USERS_TABLE, "daria").expect("get_owned");
                assert_eq!(fetched, Some(value));
            }
        }

        mod delete {
            use super::{tables::USERS_TABLE, *};

            #[test]
            fn removes_value() {
                let (_temp, db) = temp_db().expect("temp db");
                let value = TestValue {
                    id: 2,
                    name: "Bob".to_owned(),
                };

                db.put(USERS_TABLE, "bob", &value).expect("put");

                let removed = db.delete(USERS_TABLE, "bob").expect("delete");
                assert!(removed);

                let fetched: Option<TestValue> =
                    db.get_owned(USERS_TABLE, "bob").expect("get_owned");
                assert_eq!(fetched, None);
            }

            #[test]
            fn returns_false_for_missing_key() {
                let (_temp, db) = temp_db().expect("temp db");
                let value = TestValue {
                    id: 3,
                    name: "Existing".to_owned(),
                };

                db.put(USERS_TABLE, "existing", &value).expect("put");

                let removed =
                    db.delete(USERS_TABLE, "missing").expect("delete");
                assert!(!removed);
            }

            #[test]
            fn removes_value_in_table() {
                let (_temp, db) = temp_db().expect("temp db");
                let value = TestValue {
                    id: 5,
                    name: "Remy".to_owned(),
                };

                db.put(USERS_TABLE, "remy", &value).expect("put");

                let removed = db.delete(USERS_TABLE, "remy").expect("delete");
                assert!(removed);

                let fetched: Option<TestValue> =
                    db.get_owned(USERS_TABLE, "remy").expect("get_owned");
                assert_eq!(fetched, None);
            }
        }

        mod uuid_operations {
            use super::{tables::USERS_TABLE, *};

            #[test]
            fn writes_and_reads_by_uuid() {
                let (_temp, db) = temp_db().expect("temp db");
                let id =
                    uuid::Uuid::new_v7(uuid::Timestamp::now(uuid::NoContext));
                let value = TestValue {
                    id: 10,
                    name: "Note".to_owned(),
                };
                let id_v7 = crate::utils::UuidV7::try_from(id)
                    .expect("generated uuid should be v7");

                db.put_by_uuid(USERS_TABLE, id_v7, &value)
                    .expect("put_by_uuid");

                let fetched: Option<TestValue> = db
                    .get_owned(USERS_TABLE, &id.to_string())
                    .expect("get_owned");
                assert_eq!(fetched, Some(value));
            }

            #[test]
            fn delete_by_uuid_removes_value() {
                let (_temp, db) = temp_db().expect("temp db");
                let id =
                    uuid::Uuid::new_v7(uuid::Timestamp::now(uuid::NoContext));
                let value = TestValue {
                    id: 11,
                    name: "Delete".to_owned(),
                };
                let id_v7 = crate::utils::UuidV7::try_from(id)
                    .expect("generated uuid should be v7");

                db.put_by_uuid(USERS_TABLE, id_v7, &value)
                    .expect("put_by_uuid");

                let removed = db
                    .delete_by_uuid(USERS_TABLE, id_v7)
                    .expect("delete_by_uuid");
                assert!(removed);

                let fetched: Option<TestValue> = db
                    .get_owned(USERS_TABLE, &id.to_string())
                    .expect("get_owned");
                assert_eq!(fetched, None);
            }

            #[test]
            fn writes_and_reads_by_uuid_in_table() {
                let (_temp, db) = temp_db().expect("temp db");
                let id =
                    uuid::Uuid::new_v7(uuid::Timestamp::now(uuid::NoContext));
                let value = TestValue {
                    id: 12,
                    name: "Nova".to_owned(),
                };
                let id_v7 = crate::utils::UuidV7::try_from(id)
                    .expect("generated uuid should be v7");

                db.put_by_uuid(USERS_TABLE, id_v7, &value)
                    .expect("put_by_uuid");

                let fetched: Option<TestValue> = db
                    .get_owned(USERS_TABLE, &id.to_string())
                    .expect("get_owned");
                assert_eq!(fetched, Some(value));
            }
        }

        mod multimap {
            use super::{tables::TAGS_TABLE, *};

            #[test]
            fn insert_and_remove_values() {
                let (_temp, db) = temp_db().expect("temp db");

                db.multimap_insert(TAGS_TABLE, "work", "note1")
                    .expect("multimap_insert");
                db.multimap_insert(TAGS_TABLE, "work", "note2")
                    .expect("multimap_insert");

                let values_before =
                    db.multimap_get(TAGS_TABLE, "work").expect("multimap_get");
                assert_eq!(values_before.len(), 2);
                assert!(values_before.iter().any(|value| value == "note1"));
                assert!(values_before.iter().any(|value| value == "note2"));

                let removed = db
                    .multimap_remove(TAGS_TABLE, "work", "note1")
                    .expect("multimap_remove");
                assert!(removed);

                let values_after =
                    db.multimap_get(TAGS_TABLE, "work").expect("multimap_get");
                assert_eq!(values_after.len(), 1);
                let value = values_after.first().expect("multimap value");
                assert_eq!(value, "note2");
            }

            #[test]
            fn remove_returns_false_for_missing_value() {
                let (_temp, db) = temp_db().expect("temp db");

                db.multimap_insert(TAGS_TABLE, "other", "note1")
                    .expect("multimap_insert");

                let removed = db
                    .multimap_remove(TAGS_TABLE, "missing", "note1")
                    .expect("multimap_remove");
                assert!(!removed);
            }
        }
    }

    mod batch_operations {
        use super::*;

        mod tables {
            use super::*;

            pub(super) const USERS_TABLE: TableDefinition<&str, &[u8]> =
                TableDefinition::new("users");
            pub(super) const TAGS_TABLE: MultimapTableDefinition<&str, &str> =
                MultimapTableDefinition::new("tags");
        }

        #[test]
        fn batch_write_inserts_values() {
            use tables::{TAGS_TABLE, USERS_TABLE};

            let (_temp, db) = temp_db().expect("temp db");
            let value1 = TestValue {
                id: 20,
                name: "One".to_owned(),
            };
            let value2 = TestValue {
                id: 21,
                name: "Two".to_owned(),
            };

            db.batch_write(|batch| {
                batch.put(USERS_TABLE, "one", &value1)?;
                batch.put(USERS_TABLE, "two", &value2)?;
                batch.multimap_insert(TAGS_TABLE, "batch", "one")?;
                Ok(())
            })
            .expect("batch_write");

            let fetched_one: Option<TestValue> =
                db.get_owned(USERS_TABLE, "one").expect("get_owned");
            assert_eq!(fetched_one, Some(value1));

            let fetched_two: Option<TestValue> =
                db.get_owned(USERS_TABLE, "two").expect("get_owned");
            assert_eq!(fetched_two, Some(value2));

            let tags =
                db.multimap_get(TAGS_TABLE, "batch").expect("multimap_get");
            assert_eq!(tags.len(), 1);
            let tag = tags.first().unwrap();
            assert_eq!(tag, "one");
        }

        #[test]
        fn batch_write_rolls_back_on_error() {
            use tables::USERS_TABLE;
            let (_temp, db) = temp_db().expect("temp db");
            let existing = TestValue {
                id: 30,
                name: "Existing".to_owned(),
            };
            let temp_value = TestValue {
                id: 31,
                name: "Temp".to_owned(),
            };

            db.put(USERS_TABLE, "existing", &existing).expect("put");

            let result = db.batch_write(|batch| {
                batch.put(USERS_TABLE, "temp", &temp_value)?;
                let io_err = std::io::Error::other("intentional");
                let storage_err = redb::StorageError::from(io_err);
                Err(DbError::Transaction(redb::TransactionError::from(
                    storage_err,
                )))
            });

            assert!(result.is_err());

            let fetched_temp: Option<TestValue> =
                db.get_owned(USERS_TABLE, "temp").expect("get_owned");
            assert_eq!(fetched_temp, None);

            let fetched_existing: Option<TestValue> =
                db.get_owned(USERS_TABLE, "existing").expect("get_owned");
            assert_eq!(fetched_existing, Some(existing));
        }
    }

    mod read_write_unit_of_work {
        use super::*;

        mod tables {
            use super::*;

            pub(super) const COUNTER_TABLE: TableDefinition<&str, &[u8]> =
                TableDefinition::new("counters");
        }

        #[test]
        fn read_write_unit_of_work_performs_atomic_read_modify_write() {
            use tables::COUNTER_TABLE;
            let (_temp, db) = temp_db().expect("temp db");

            let result = db.read_write_unit_of_work(|tx| {
                let current: Option<u64> =
                    tx.get_owned(COUNTER_TABLE, "counter")?;
                let next = current.unwrap_or(0) + 1;
                tx.put(COUNTER_TABLE, "counter", &next)?;
                Ok(next)
            });

            assert_eq!(result.unwrap(), 1);

            let result2 = db.read_write_unit_of_work(|tx| {
                let current: Option<u64> =
                    tx.get_owned(COUNTER_TABLE, "counter")?;
                let next = current.unwrap_or(0) + 1;
                tx.put(COUNTER_TABLE, "counter", &next)?;
                Ok(next)
            });

            assert_eq!(result2.unwrap(), 2);
        }

        #[test]
        fn read_write_unit_of_work_returns_none_for_missing_key() {
            use tables::COUNTER_TABLE;
            let (_temp, db) = temp_db().expect("temp db");

            let result = db.read_write_unit_of_work(|tx| {
                let current: Option<u64> =
                    tx.get_owned(COUNTER_TABLE, "missing")?;
                Ok(current.is_none())
            });

            assert!(result.unwrap());
        }

        #[test]
        fn read_write_unit_of_work_rolls_back_on_error() {
            use tables::COUNTER_TABLE;
            let (_temp, db) = temp_db().expect("temp db");

            let result: Result<(), DbError> =
                db.read_write_unit_of_work(|tx| {
                    tx.put(COUNTER_TABLE, "counter", &42u64)?;
                    let io_err = std::io::Error::other("intentional");
                    let storage_err = redb::StorageError::from(io_err);
                    Err(DbError::Transaction(redb::TransactionError::from(
                        storage_err,
                    )))
                });

            assert!(result.is_err());

            let fetched: Option<u64> =
                db.get_owned(COUNTER_TABLE, "counter").unwrap();
            assert_eq!(fetched, None);
        }
    }

    #[derive(
        Debug,
        Clone,
        PartialEq,
        Eq,
        rkyv::Archive,
        rkyv::Serialize,
        rkyv::Deserialize,
    )]
    #[rkyv(bytecheck(bounds()))]
    struct TestValue {
        id: u32,
        name: String,
    }

    type TestResult<T> = Result<T, Box<dyn std::error::Error>>;
    type TempDb = (tempfile::TempDir, Database);

    fn temp_db() -> TestResult<TempDb> {
        let temp = tempfile::tempdir()?;
        let db_path = temp.path().join("test.db");
        let db = Database::open(&db_path)?;
        Ok((temp, db))
    }
}
