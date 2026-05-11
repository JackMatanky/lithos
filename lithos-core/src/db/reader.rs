//! Internal read operations and performance-critical access paths.
//!
//! This module provides the implementation for zero-copy and owned reads.
//!
//! # Transaction Strategy
//!
//! To ensure safety, all read operations check that the underlying data
//! is properly aligned. Redb typically stores data page-aligned, allowing
//! for a fast, truly zero-copy path. If data is unaligned, a temporary
//! aligned copy is created to satisfy `rkyv`'s safety requirements.

use redb::{
    MultimapTableDefinition, ReadableDatabase as _, ReadableTable as _,
    TableDefinition, TableError,
};
use rkyv::util::AlignedVec;

use super::{Database, DbError};
use crate::utils::UuidV7;

#[inline]
fn with_uuid_v7_key<R>(id: UuidV7, f: impl FnOnce(&str) -> R) -> R {
    let mut buf = [0u8; 36];
    let key = id.as_uuid().hyphenated().encode_lower(&mut buf);
    f(key)
}

impl Database {
    /// Zero-copy read from a specific table definition (HOT PATH for LSP).
    ///
    /// The closure receives a reference to the archived data within the
    /// transaction scope, ensuring safety without unsafe code.
    ///
    /// # Errors
    ///
    /// - [`DbError::Deserialization`] - Data validation failed
    /// - [`DbError::Transaction`] - Transaction or table operation failed
    #[inline]
    pub fn get<V, F, R>(
        &self,
        table: TableDefinition<&str, &[u8]>,
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
        read_archived::<V, _, _>(self.inner(), table, key, f)
    }

    /// Full deserialization from a specific table definition (COLD PATH).
    ///
    /// # Errors
    ///
    /// - [`DbError::Deserialization`] - Data validation or deserialization
    ///   failed
    /// - [`DbError::Transaction`] - Transaction or table operation failed
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
        deserialize_owned::<V>(self.inner(), table, key)
    }

    /// Zero-copy read using UUID as key (HOT PATH - eliminates allocation).
    ///
    /// Same as [`get`](Self::get) but accepts a UUID directly, avoiding
    /// the 36-byte String allocation from `uuid.to_string()`.
    ///
    /// # Errors
    /// Same as [`get`](Self::get).
    #[inline]
    pub fn get_by_uuid<V, F, R>(
        &self,
        table: TableDefinition<&str, &[u8]>,
        id: UuidV7,
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
        with_uuid_v7_key(id, |key| {
            read_archived::<V, _, _>(self.inner(), table, key, f)
        })
    }

    /// Full deserialization using UUID as key (eliminates allocation).
    ///
    /// Same as [`get_owned`](Self::get_owned) but accepts a UUID directly.
    ///
    /// # Errors
    /// Same as [`get_owned`](Self::get_owned).
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
        with_uuid_v7_key(id, |key| {
            deserialize_owned::<V>(self.inner(), table, key)
        })
    }

    /// Get all values for a key from a multimap table definition.
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
        table: MultimapTableDefinition<&str, &str>,
        key: &str,
    ) -> Result<Vec<String>, DbError> {
        multimap_get_impl(self.inner(), table, key)
    }

    /// List all values in a table definition (owned).
    ///
    /// # Errors
    ///
    /// Returns [`DbError`] if transaction or deserialization fails.
    #[inline]
    pub fn list_owned<V>(
        &self,
        table: TableDefinition<&str, &[u8]>,
    ) -> Result<Vec<V>, DbError>
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
        scan_table::<V>(self.inner(), table)
    }

    /// Scan a table and return key-value pairs as owned types.
    ///
    /// Keys are returned as `String`, values are deserialized via rkyv.
    ///
    /// # Errors
    ///
    /// Returns [`DbError`] if transaction or deserialization fails.
    #[inline]
    pub fn list_key_value_pairs<V>(
        &self,
        table: TableDefinition<&str, &[u8]>,
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
        scan_table_key_value::<V>(self.inner(), table)
    }

    /// Scan a table range matching a key prefix and return key-value pairs.
    ///
    /// This is significantly more efficient than
    /// [`list_key_value_pairs`](Self::list_key_value_pairs) followed by
    /// filtering, as it uses redb's native range query support
    /// to only iterate over matching entries (O(K) where K is matches, not O(N)
    /// where N is total table size).
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// # use lithos_core::db::Database;
    /// # use redb::TableDefinition;
    /// # #[derive(rkyv::Archive, rkyv::Serialize, rkyv::Deserialize)]
    /// # #[rkyv(bytecheck(bounds()))]
    /// # struct Property { id: String }
    /// # fn example(db: &Database) -> Result<(), Box<dyn std::error::Error>> {
    /// # const TABLE: TableDefinition<&str, &[u8]> = TableDefinition::new("props");
    /// // Get all properties for schema version 2
    /// let properties: Vec<(String, Property)> =
    ///     db.scan_range(TABLE, "schema:v2:")?;
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// # Errors
    ///
    /// Returns [`DbError`] if transaction or deserialization fails.
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
        scan_range_impl::<V>(self.inner(), table, key_prefix)
    }

    /// Execute multiple read operations within a single transaction.
    ///
    /// This amortizes transaction creation cost across multiple reads,
    /// improving performance for batch operations by ~50-100ns per query.
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// # use lithos_core::db::Database;
    /// # fn example(db: &Database) -> Result<(), Box<dyn std::error::Error>> {
    /// // Execute multiple reads in a single transaction
    /// let count = db.batch_read(|reader| {
    ///     // Perform multiple table operations within one transaction
    ///     // This is more efficient than separate transactions
    ///     Ok(42usize)
    /// })?;
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// # Errors
    ///
    /// Returns `DbError::Transaction` if the transaction fails or the closure
    /// returns an error.
    #[inline]
    pub fn batch_read<R, F>(&self, f: F) -> Result<R, DbError>
    where
        F: FnOnce(&BatchReader) -> Result<R, DbError>,
    {
        let tx = self.inner().begin_read()?;
        let reader = BatchReader::new(tx);
        f(&reader)
    }
}

/// A single read transaction for batching many read operations.
///
/// This is intentionally scoped to a closure (see `Database::batch_read`) so
/// callers cannot accidentally hold a transaction across unrelated work.
pub struct BatchReader {
    tx: redb::ReadTransaction,
}

impl BatchReader {
    #[inline]
    pub(super) fn new(tx: redb::ReadTransaction) -> Self {
        Self {
            tx,
        }
    }

    /// Zero-copy read from a specific table definition.
    ///
    /// # Errors
    ///
    /// Returns `DbError` if deserialization or transaction fails.
    #[inline]
    pub fn get<V, F, R>(
        &self,
        table: TableDefinition<&str, &[u8]>,
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
        read_archived_tx::<V, _, _>(&self.tx, table, key, f)
    }

    /// Full deserialization from a specific table definition.
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

    /// Zero-copy read using UUID as key within a transaction.
    ///
    /// # Errors
    ///
    /// Returns `DbError` if deserialization or transaction fails.
    #[inline]
    pub fn get_by_uuid<V, F, R>(
        &self,
        table: TableDefinition<&str, &[u8]>,
        id: UuidV7,
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
        with_uuid_v7_key(id, |key| {
            read_archived_tx::<V, _, _>(&self.tx, table, key, f)
        })
    }

    /// Full deserialization using UUID as key within a transaction.
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

    /// Scan a table and return all owned values.
    ///
    /// # Errors
    ///
    /// Returns [`DbError`] if transaction or deserialization fails.
    #[inline]
    pub fn list_owned<V>(
        &self,
        table: TableDefinition<&str, &[u8]>,
    ) -> Result<Vec<V>, DbError>
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
        scan_table_tx::<V>(&self.tx, table)
    }

    /// Scan a table and return key-value pairs as owned types.
    ///
    /// Keys are returned as `String`, values are deserialized via rkyv.
    ///
    /// # Errors
    ///
    /// Returns [`DbError`] if transaction or deserialization fails.
    #[inline]
    pub fn list_key_value_pairs<V>(
        &self,
        table: TableDefinition<&str, &[u8]>,
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
        scan_table_key_value_tx::<V>(&self.tx, table)
    }

    /// Scan a table range matching a key prefix and return key-value pairs.
    ///
    /// This is significantly more efficient than
    /// [`list_key_value_pairs`](Self::list_key_value_pairs) followed by
    /// filtering, as it uses redb's native range query support.
    ///
    /// # Errors
    ///
    /// Returns [`DbError`] if transaction or deserialization fails.
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
        scan_range_tx::<V>(&self.tx, table, key_prefix)
    }
}

// Private helper functions for internal use

/// Zero-copy read of archived data with closure from a table definition.
fn read_archived<V, F, R>(
    db: &redb::Database,
    table: TableDefinition<&str, &[u8]>,
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
    let tx = db.begin_read()?;
    let table_ref = match tx.open_table(table) {
        Ok(table_ref) => table_ref,
        Err(TableError::TableDoesNotExist(_)) => return Ok(None),
        Err(err) => return Err(err.into()),
    };

    match table_ref.get(key)? {
        Some(value) => {
            let bytes: &[u8] = value.value();

            // Fast path: Check if data is already 16-byte aligned.
            // Redb often stores data page-aligned, making this common case
            // zero-copy.
            #[expect(
                clippy::as_conversions,
                reason = "Pointer to usize conversion required for alignment \
                          check"
            )]
            let ptr_usize: usize = bytes.as_ptr() as usize;
            if ptr_usize.is_multiple_of(16) {
                // Zero-copy fast path: Direct access, no allocation
                let archived = rkyv::access::<
                    rkyv::Archived<V>,
                    rkyv::rancor::Error,
                >(bytes)
                .map_err(|e| DbError::Deserialization(e.to_string()))?;
                let result = f(archived);
                Ok(Some(result))
            } else {
                // Slow path: Copy to aligned buffer (rare for redb)
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
        }
        None => Ok(None),
    }
}

/// Full deserialization of archived data into owned value from a table.
fn deserialize_owned<V>(
    db: &redb::Database,
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
    let tx = db.begin_read()?;
    let table_ref = match tx.open_table(table) {
        Ok(table_ref) => table_ref,
        Err(TableError::TableDoesNotExist(_)) => return Ok(None),
        Err(err) => return Err(err.into()),
    };

    match table_ref.get(key)? {
        Some(value) => {
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
            Ok(Some(deserialized))
        }
        None => Ok(None),
    }
}

/// Scan all entries in a table definition and deserialize to owned values.
fn scan_table<V>(
    db: &redb::Database,
    table: TableDefinition<&str, &[u8]>,
) -> Result<Vec<V>, DbError>
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
    let tx = db.begin_read()?;
    let table_ref = match tx.open_table(table) {
        Ok(table_ref) => table_ref,
        Err(TableError::TableDoesNotExist(_)) => return Ok(Vec::new()),
        Err(err) => return Err(err.into()),
    };

    // BUGFIX: Separate iteration from deserialization to avoid redb
    // AccessGuard/rkyv conflict See: RKYV_BUG_ROOT_CAUSE_AND_FIX.md

    // Step 1: Collect all byte buffers while AccessGuards are alive
    let mut all_bytes = Vec::new();
    for result in table_ref.iter()? {
        let (_key, value): (_, redb::AccessGuard<&[u8]>) = result?;
        let bytes: &[u8] = value.value();
        let mut aligned = AlignedVec::<16>::new();
        aligned.extend_from_slice(bytes);
        all_bytes.push(aligned);
    }
    // AccessGuards are now dropped

    // Step 2: Deserialize after iteration is complete
    let mut results = Vec::new();
    for aligned in all_bytes {
        let archived =
            rkyv::access::<rkyv::Archived<V>, rkyv::rancor::Error>(&aligned)
                .map_err(|e| DbError::Deserialization(e.to_string()))?;
        let deserialized =
            rkyv::deserialize::<V, rkyv::rancor::Error>(archived)
                .map_err(|e| DbError::Deserialization(e.to_string()))?;
        results.push(deserialized);
    }

    Ok(results)
}

/// Scan all entries in a table and return key-value pairs.
fn scan_table_key_value<V>(
    db: &redb::Database,
    table: TableDefinition<&str, &[u8]>,
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
    let tx = db.begin_read()?;
    let table_ref = match tx.open_table(table) {
        Ok(table_ref) => table_ref,
        Err(TableError::TableDoesNotExist(_)) => return Ok(Vec::new()),
        Err(err) => return Err(err.into()),
    };

    // BUGFIX: Separate iteration from deserialization to avoid redb
    // AccessGuard/rkyv conflict See: RKYV_BUG_ROOT_CAUSE_AND_FIX.md

    // Step 1: Collect all key-value byte buffers while AccessGuards are alive
    let mut all_data = Vec::new();
    for result in table_ref.iter()? {
        let (key, value): (redb::AccessGuard<&str>, redb::AccessGuard<&[u8]>) =
            result?;
        let key_str = key.value().to_owned();
        let mut aligned = AlignedVec::<16>::new();
        aligned.extend_from_slice(value.value());
        all_data.push((key_str, aligned));
    }
    // AccessGuards are now dropped

    // Step 2: Deserialize after iteration is complete
    let mut results = Vec::new();
    for (key_str, aligned) in all_data {
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

/// Get all values for a multimap table definition.
fn multimap_get_impl(
    db: &redb::Database,
    table: MultimapTableDefinition<&str, &str>,
    key: &str,
) -> Result<Vec<String>, DbError> {
    let tx = db.begin_read()?;
    let tbl = match tx.open_multimap_table(table) {
        Ok(tbl) => tbl,
        Err(TableError::TableDoesNotExist(_)) => return Ok(Vec::new()),
        Err(err) => return Err(err.into()),
    };

    let mut values = Vec::new();
    let range = tbl.get(key)?;
    for result in range {
        let guard = result?;
        let value: &str = guard.value();
        values.push(value.to_owned());
    }

    Ok(values)
}

// Transaction-based helper functions for BatchReader

/// Zero-copy read of archived data with closure from a table within a
/// transaction.
fn read_archived_tx<V, F, R>(
    tx: &redb::ReadTransaction,
    table: TableDefinition<&str, &[u8]>,
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
    let table_ref = match tx.open_table(table) {
        Ok(table_ref) => table_ref,
        Err(TableError::TableDoesNotExist(_)) => return Ok(None),
        Err(err) => return Err(err.into()),
    };

    match table_ref.get(key)? {
        Some(value) => {
            let bytes: &[u8] = value.value();

            #[expect(
                clippy::as_conversions,
                reason = "Pointer to usize conversion required for alignment \
                          check"
            )]
            let ptr_usize: usize = bytes.as_ptr() as usize;
            if ptr_usize.is_multiple_of(16) {
                let archived = rkyv::access::<
                    rkyv::Archived<V>,
                    rkyv::rancor::Error,
                >(bytes)
                .map_err(|e| DbError::Deserialization(e.to_string()))?;
                let result = f(archived);
                Ok(Some(result))
            } else {
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
        }
        None => Ok(None),
    }
}

/// Full deserialization of archived data into owned value from a table within a
/// transaction.
fn get_owned_tx<V>(
    tx: &redb::ReadTransaction,
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
        Err(TableError::TableDoesNotExist(_)) => return Ok(None),
        Err(err) => return Err(err.into()),
    };

    match table_ref.get(key)? {
        Some(value) => {
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
            Ok(Some(deserialized))
        }
        None => Ok(None),
    }
}

/// Scan all entries in a table definition and deserialize to owned values
/// within a transaction.
fn scan_table_tx<V>(
    tx: &redb::ReadTransaction,
    table: TableDefinition<&str, &[u8]>,
) -> Result<Vec<V>, DbError>
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
        Err(TableError::TableDoesNotExist(_)) => return Ok(Vec::new()),
        Err(err) => return Err(err.into()),
    };

    // BUGFIX: Separate redb iteration from rkyv deserialization to avoid
    // AccessGuard lifetime issues that cause "subtree pointer overran range"
    // errors. Collect all bytes first, then deserialize.
    //
    // Root cause: Mixing redb::AccessGuard iteration with rkyv deserialization
    // in the same loop causes memory corruption on the 2nd iteration.
    // See: RKYV_BUG_ROOT_CAUSE_AND_FIX.md

    // Step 1: Collect all byte buffers while AccessGuards are alive
    let mut all_bytes = Vec::new();
    for result in table_ref.iter()? {
        let (_key, value): (_, redb::AccessGuard<&[u8]>) = result?;
        let mut aligned = AlignedVec::<16>::new();
        aligned.extend_from_slice(value.value());
        all_bytes.push(aligned);
    }
    // AccessGuards are now dropped, safe to deserialize

    // Step 2: Deserialize after iteration is complete
    let mut results = Vec::new();
    for aligned in &all_bytes {
        let archived =
            rkyv::access::<rkyv::Archived<V>, rkyv::rancor::Error>(aligned)
                .map_err(|e| DbError::Deserialization(e.to_string()))?;
        let deserialized =
            rkyv::deserialize::<V, rkyv::rancor::Error>(archived)
                .map_err(|e| DbError::Deserialization(e.to_string()))?;
        results.push(deserialized);
    }

    Ok(results)
}

/// Scan all entries in a table and return key-value pairs within a transaction.
fn scan_table_key_value_tx<V>(
    tx: &redb::ReadTransaction,
    table: TableDefinition<&str, &[u8]>,
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
    let table_ref = match tx.open_table(table) {
        Ok(table_ref) => table_ref,
        Err(TableError::TableDoesNotExist(_)) => return Ok(Vec::new()),
        Err(err) => return Err(err.into()),
    };

    // BUGFIX: Same fix as scan_table_tx - separate iteration from
    // deserialization See: RKYV_BUG_ROOT_CAUSE_AND_FIX.md

    // Step 1: Collect all key-value byte buffers
    let mut all_data = Vec::new();
    for result in table_ref.iter()? {
        let (key, value): (redb::AccessGuard<&str>, redb::AccessGuard<&[u8]>) =
            result?;
        let key_str = key.value().to_owned();
        let mut aligned = AlignedVec::<16>::new();
        aligned.extend_from_slice(value.value());
        all_data.push((key_str, aligned));
    }
    // AccessGuards are now dropped

    // Step 2: Deserialize after iteration is complete
    let mut results = Vec::new();
    for (key_str, aligned) in all_data {
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

/// Scan entries in a table matching a key prefix and return key-value pairs.
///
/// Uses redb's range query to only iterate over keys starting with the given
/// prefix. This is O(K) where K is the number of matching entries, versus O(N)
/// for a full table scan with filtering.
fn scan_range_impl<V>(
    db: &redb::Database,
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
    let tx = db.begin_read()?;
    scan_range_tx::<V>(&tx, table, key_prefix)
}

/// Scan entries in a table matching a key prefix within a transaction.
fn scan_range_tx<V>(
    tx: &redb::ReadTransaction,
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
    let table_ref = match tx.open_table(table) {
        Ok(table_ref) => table_ref,
        Err(TableError::TableDoesNotExist(_)) => return Ok(Vec::new()),
        Err(err) => return Err(err.into()),
    };

    // Compute the exclusive end bound for the range scan.
    // For prefix "abc", we want to scan ["abc", "abd"), which captures all keys
    // starting with "abc".
    let end_bound = next_prefix(key_prefix);

    // BUGFIX: Same fix as scan_table_tx - separate iteration from
    // deserialization See: RKYV_BUG_ROOT_CAUSE_AND_FIX.md

    // Step 1: Collect all key-value byte buffers
    let mut all_data = Vec::new();
    let range = table_ref.range(key_prefix..end_bound.as_str())?;
    for result in range {
        let (key, value): (redb::AccessGuard<&str>, redb::AccessGuard<&[u8]>) =
            result?;
        let key_str = key.value().to_owned();
        let mut aligned = AlignedVec::<16>::new();
        aligned.extend_from_slice(value.value());
        all_data.push((key_str, aligned));
    }
    // AccessGuards are now dropped

    // Step 2: Deserialize after iteration is complete
    let mut results = Vec::new();
    for (key_str, aligned) in all_data {
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
/// For a prefix "abc", this returns "abd", which is the smallest string that is
/// strictly greater than all strings starting with "abc". This is used as the
/// exclusive upper bound for range scans.
///
/// If the prefix ends in 0xFF bytes or is empty, this returns a string with an
/// additional 0xFF byte appended, ensuring the range query still works
/// correctly.
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
#[expect(
    clippy::pattern_type_mismatch,
    reason = "Iterator over Vec<(String, T)> yields &(String, T), requiring \
              |(k, _)| pattern instead of |&(k, _)| for ergonomic testing"
)]
mod tests {
    use super::*;

    mod read_operations {
        use super::*;

        mod tables {
            use super::*;

            pub(super) const USERS_TABLE: TableDefinition<&str, &[u8]> =
                TableDefinition::new("users");
            pub(super) const TAGS_TABLE: MultimapTableDefinition<&str, &str> =
                MultimapTableDefinition::new("tags");
            pub(super) const NOTES_TABLE: TableDefinition<&str, &[u8]> =
                TableDefinition::new("notes");
        }

        mod get {
            use super::{tables::USERS_TABLE, *};

            #[test]
            fn zero_copy_read_works() {
                let (_temp, db) = temp_db().expect("temp db");

                let value = TestValue {
                    id: 42,
                    name: "Alice".to_owned(),
                };
                db.put(USERS_TABLE, "alice", &value).expect("put");

                let result = db
                    .get::<TestValue, _, _>(USERS_TABLE, "alice", |archived| {
                        archived.id.to_native()
                    })
                    .expect("get");

                assert_eq!(result, Some(42));
            }

            #[test]
            fn returns_none_for_missing_key() {
                let (_temp, db) = temp_db().expect("temp db");

                let value = TestValue {
                    id: 7,
                    name: "Existing".to_owned(),
                };
                db.put(USERS_TABLE, "existing", &value).expect("put");

                let result = db
                    .get::<TestValue, _, _>(
                        USERS_TABLE,
                        "missing",
                        |archived| archived.id.to_native(),
                    )
                    .expect("get");

                assert_eq!(result, None);
            }

            #[test]
            fn closure_receives_archived_data() {
                let (_temp, db) = temp_db().expect("temp db");

                let value = TestValue {
                    id: 99,
                    name: "Test".to_owned(),
                };
                db.put(USERS_TABLE, "test", &value).expect("put");

                db.get::<TestValue, _, _>(USERS_TABLE, "test", |archived| {
                    assert_eq!(archived.id.to_native(), 99);
                    assert_eq!(archived.name.as_str(), "Test");
                })
                .expect("get");
            }
        }

        mod get_owned {
            use super::{tables::USERS_TABLE, *};

            #[test]
            fn deserialization_works_in_table() {
                let (_temp, db) = temp_db().expect("temp db");

                let original = TestValue {
                    id: 234,
                    name: "Casey".to_owned(),
                };
                db.put(USERS_TABLE, "casey", &original).expect("put");

                let result: Option<TestValue> =
                    db.get_owned(USERS_TABLE, "casey").expect("get_owned");

                assert_eq!(result, Some(original));
            }

            #[test]
            fn deserialization_works() {
                let (_temp, db) = temp_db().expect("temp db");

                let original = TestValue {
                    id: 123,
                    name: "Bob".to_owned(),
                };
                db.put(USERS_TABLE, "bob", &original).expect("put");

                let result: Option<TestValue> =
                    db.get_owned(USERS_TABLE, "bob").expect("get_owned");

                assert_eq!(result, Some(original));
            }

            #[test]
            fn returns_none_for_missing_key() {
                let (_temp, db) = temp_db().expect("temp db");

                let value = TestValue {
                    id: 8,
                    name: "Existing".to_owned(),
                };
                db.put(USERS_TABLE, "existing", &value).expect("put");

                let result: Option<TestValue> =
                    db.get_owned(USERS_TABLE, "missing").expect("get_owned");

                assert_eq!(result, None);
            }
        }

        mod uuid_operations {
            use super::*;

            const NOTES_TABLE: TableDefinition<&str, &[u8]> =
                TableDefinition::new("notes");
            const ITEMS_TABLE: TableDefinition<&str, &[u8]> =
                TableDefinition::new("items");

            #[test]
            fn uuid_key_formatting() {
                let (_temp, db) = temp_db().expect("temp db");

                let id =
                    uuid::Uuid::new_v7(uuid::Timestamp::now(uuid::NoContext));
                let value = TestValue {
                    id: 456,
                    name: "Charlie".to_owned(),
                };
                let id_v7 = crate::utils::UuidV7::try_from(id)
                    .expect("generated uuid should be v7");

                db.put_by_uuid(NOTES_TABLE, id_v7, &value)
                    .expect("put_by_uuid");

                let result = db
                    .get::<TestValue, _, _>(
                        NOTES_TABLE,
                        &id.to_string(),
                        |archived| archived.id.to_native(),
                    )
                    .expect("get");

                assert_eq!(result, Some(456));
            }

            #[test]
            fn reads_with_uuid_keys() {
                let (_temp, db) = temp_db().expect("temp db");

                let id =
                    uuid::Uuid::new_v7(uuid::Timestamp::now(uuid::NoContext));
                let original = TestValue {
                    id: 789,
                    name: "David".to_owned(),
                };
                let id_v7 = crate::utils::UuidV7::try_from(id)
                    .expect("generated uuid should be v7");

                db.put_by_uuid(ITEMS_TABLE, id_v7, &original)
                    .expect("put_by_uuid");

                let result: Option<TestValue> = db
                    .get_owned(ITEMS_TABLE, &id.to_string())
                    .expect("get_owned");

                assert_eq!(result, Some(original));
            }
        }

        mod multimap {
            use super::{tables::TAGS_TABLE, *};

            #[test]
            fn multimap_get_returns_all_values() {
                let (_temp, db) = temp_db().expect("temp db");

                db.multimap_insert(TAGS_TABLE, "work", "note1")
                    .expect("multimap_insert");
                db.multimap_insert(TAGS_TABLE, "work", "note2")
                    .expect("multimap_insert");
                db.multimap_insert(TAGS_TABLE, "work", "note3")
                    .expect("multimap_insert");

                let values =
                    db.multimap_get(TAGS_TABLE, "work").expect("multimap_get");

                assert_eq!(values.len(), 3);
                assert!(values.iter().any(|value| value == "note1"));
                assert!(values.iter().any(|value| value == "note2"));
                assert!(values.iter().any(|value| value == "note3"));
            }

            #[test]
            fn empty_multimap_returns_empty_vec() {
                let (_temp, db) = temp_db().expect("temp db");

                db.multimap_insert(TAGS_TABLE, "other", "note1")
                    .expect("multimap_insert");

                let values = db
                    .multimap_get(TAGS_TABLE, "nonexistent")
                    .expect("multimap_get");

                assert_eq!(values.len(), 0);
            }
        }

        mod scan_range {
            use super::{tables::USERS_TABLE, *};

            #[test]
            fn scan_range_filters_by_prefix() {
                let (_temp, db) = temp_db().expect("temp db");

                db.put(USERS_TABLE, "user:1:name", &TestValue {
                    id: 1,
                    name: "Alice".to_owned(),
                })
                .expect("put");
                db.put(USERS_TABLE, "user:1:email", &TestValue {
                    id: 2,
                    name: "alice@example.com".to_owned(),
                })
                .expect("put");
                db.put(USERS_TABLE, "user:2:name", &TestValue {
                    id: 3,
                    name: "Bob".to_owned(),
                })
                .expect("put");
                db.put(USERS_TABLE, "post:1:title", &TestValue {
                    id: 4,
                    name: "Post".to_owned(),
                })
                .expect("put");

                let results: Vec<(String, TestValue)> =
                    db.scan_range(USERS_TABLE, "user:1:").expect("scan_range");

                assert_eq!(results.len(), 2);
                assert!(
                    results.iter().any(|(k, _)| k.as_str() == "user:1:name")
                );
                assert!(
                    results.iter().any(|(k, _)| k.as_str() == "user:1:email")
                );
                assert!(!results.iter().any(|(k, _)| k.starts_with("user:2:")));
                assert!(!results.iter().any(|(k, _)| k.starts_with("post:")));
            }

            #[test]
            fn scan_range_returns_empty_for_no_matches() {
                let (_temp, db) = temp_db().expect("temp db");

                db.put(USERS_TABLE, "user:1:name", &TestValue {
                    id: 1,
                    name: "Alice".to_owned(),
                })
                .expect("put");

                let results: Vec<(String, TestValue)> = db
                    .scan_range(USERS_TABLE, "nonexistent:")
                    .expect("scan_range");

                assert_eq!(results.len(), 0);
            }

            #[test]
            fn scan_range_handles_nested_prefixes() {
                let (_temp, db) = temp_db().expect("temp db");

                db.put(USERS_TABLE, "a", &TestValue {
                    id: 1,
                    name: "A".to_owned(),
                })
                .expect("put");
                db.put(USERS_TABLE, "ab", &TestValue {
                    id: 2,
                    name: "AB".to_owned(),
                })
                .expect("put");
                db.put(USERS_TABLE, "abc", &TestValue {
                    id: 3,
                    name: "ABC".to_owned(),
                })
                .expect("put");
                db.put(USERS_TABLE, "abd", &TestValue {
                    id: 4,
                    name: "ABD".to_owned(),
                })
                .expect("put");
                db.put(USERS_TABLE, "b", &TestValue {
                    id: 5,
                    name: "B".to_owned(),
                })
                .expect("put");

                let results: Vec<(String, TestValue)> =
                    db.scan_range(USERS_TABLE, "ab").expect("scan_range");

                assert_eq!(results.len(), 3);
                assert!(
                    results
                        .iter()
                        .any(|(k, v)| k.as_str() == "ab" && v.id == 2)
                );
                assert!(
                    results
                        .iter()
                        .any(|(k, v)| k.as_str() == "abc" && v.id == 3)
                );
                assert!(
                    results
                        .iter()
                        .any(|(k, v)| k.as_str() == "abd" && v.id == 4)
                );
                assert!(!results.iter().any(|(k, _)| k.as_str() == "a"));
                assert!(!results.iter().any(|(k, _)| k.as_str() == "b"));
            }

            #[test]
            fn batch_reader_scan_range_works() {
                let (_temp, db) = temp_db().expect("temp db");

                db.put(USERS_TABLE, "schema:v1:prop1", &TestValue {
                    id: 1,
                    name: "Prop1".to_owned(),
                })
                .expect("put");
                db.put(USERS_TABLE, "schema:v1:prop2", &TestValue {
                    id: 2,
                    name: "Prop2".to_owned(),
                })
                .expect("put");
                db.put(USERS_TABLE, "schema:v2:prop1", &TestValue {
                    id: 3,
                    name: "V2Prop1".to_owned(),
                })
                .expect("put");

                let results = db
                    .batch_read(|reader| {
                        reader
                            .scan_range::<TestValue>(USERS_TABLE, "schema:v1:")
                    })
                    .expect("batch_read");

                assert_eq!(results.len(), 2);
                assert!(results.iter().any(|(_, v)| v.id == 1));
                assert!(results.iter().any(|(_, v)| v.id == 2));
                assert!(!results.iter().any(|(_, v)| v.id == 3));
            }
        }

        mod list_owned {
            use super::{
                tables::{NOTES_TABLE, USERS_TABLE},
                *,
            };

            #[test]
            fn lists_all_entries_in_table_def() {
                let (_temp, db) = temp_db().expect("temp db");

                db.put(USERS_TABLE, "alice", &TestValue {
                    id: 10,
                    name: "Alice".to_owned(),
                })
                .expect("put");
                db.put(USERS_TABLE, "bob", &TestValue {
                    id: 20,
                    name: "Bob".to_owned(),
                })
                .expect("put");

                let results: Vec<TestValue> =
                    db.list_owned(USERS_TABLE).expect("list_owned");

                assert_eq!(results.len(), 2);
                assert!(results.iter().any(|v| v.id == 10));
                assert!(results.iter().any(|v| v.id == 20));
            }

            #[test]
            fn lists_all_entries_in_table() {
                let (_temp, db) = temp_db().expect("temp db");

                db.put(USERS_TABLE, "alice", &TestValue {
                    id: 1,
                    name: "Alice".to_owned(),
                })
                .expect("put");
                db.put(USERS_TABLE, "bob", &TestValue {
                    id: 2,
                    name: "Bob".to_owned(),
                })
                .expect("put");
                db.put(USERS_TABLE, "charlie", &TestValue {
                    id: 3,
                    name: "Charlie".to_owned(),
                })
                .expect("put");

                let results: Vec<TestValue> =
                    db.list_owned(USERS_TABLE).expect("list_owned");

                assert_eq!(results.len(), 3);
                assert!(results.iter().any(|v| v.id == 1));
                assert!(results.iter().any(|v| v.id == 2));
                assert!(results.iter().any(|v| v.id == 3));
            }

            #[test]
            fn respects_table_prefix_boundaries() {
                let (_temp, db) = temp_db().expect("temp db");

                db.put(USERS_TABLE, "alice", &TestValue {
                    id: 1,
                    name: "Alice".to_owned(),
                })
                .expect("put");
                db.put(NOTES_TABLE, "note1", &TestValue {
                    id: 100,
                    name: "Note".to_owned(),
                })
                .expect("put");

                let users: Vec<TestValue> =
                    db.list_owned(USERS_TABLE).expect("list_owned");
                let notes: Vec<TestValue> =
                    db.list_owned(NOTES_TABLE).expect("list_owned");

                assert_eq!(users.len(), 1);
                assert_eq!(notes.len(), 1);
                let user = users.first().expect("user entry");
                let note = notes.first().expect("note entry");
                assert_eq!(user.id, 1);
                assert_eq!(note.id, 100);
            }
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

    // Test helper to create a temp database
    fn temp_db() -> TestResult<TempDb> {
        let temp = tempfile::tempdir()?;
        let db_path = temp.path().join("test.db");
        let db = Database::open(&db_path)?;
        Ok((temp, db))
    }
}
