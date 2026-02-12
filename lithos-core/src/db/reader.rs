//! Database read operations.
//!
//! This module contains all zero-copy read operations using closure-based APIs
//! to keep transactions properly scoped.

use redb::{
    MultimapTableDefinition, ReadableDatabase as _, ReadableTable as _,
    TableDefinition, TableError,
};
use rkyv::util::AlignedVec;

use super::{
    DATA_TABLE, Database, DbError,
    keys::{MultimapKey, NamespacedKey, TablePrefix},
};

impl Database {
    /// Zero-copy read from a specific table definition (HOT PATH for LSP).
    ///
    /// The closure receives a reference to the archived data within the
    /// transaction scope, ensuring safety without unsafe code.
    ///
    /// # Errors
    ///
    /// - `DbError::Deserialization` - Data validation failed
    /// - `DbError::Transaction` - Transaction or table operation failed
    #[inline]
    pub fn get_in_table<V, F, R>(
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
        read_archived_in_table::<V, _, _>(&self.inner, table, key, f)
    }

    /// Full deserialization from a specific table definition (COLD PATH).
    ///
    /// # Errors
    ///
    /// - `DbError::Deserialization` - Data validation or deserialization failed
    /// - `DbError::Transaction` - Transaction or table operation failed
    #[inline]
    pub fn get_owned_in_table<V>(
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
        deserialize_owned_in_table::<V>(&self.inner, table, key)
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
    pub fn multimap_get_in_table(
        &self,
        table: MultimapTableDefinition<&str, &str>,
        key: &str,
    ) -> Result<Vec<String>, DbError> {
        multimap_get_in_table_impl(&self.inner, table, key)
    }

    /// List all values in a table definition (owned).
    ///
    /// # Errors
    ///
    /// Returns `DbError` if transaction or deserialization fails.
    #[inline]
    pub fn list_owned_in_table<V>(
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
        scan_table::<V>(&self.inner, table)
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
    /// // db.get::<MyType, _, _>("my_table", "my_key", |archived| {
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
        let namespaced_key = NamespacedKey::new(table, key);
        read_archived::<V, _, _>(&self.inner, &namespaced_key, f)
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
        let namespaced_key = NamespacedKey::new(table, key);
        deserialize_owned::<V>(&self.inner, &namespaced_key)
    }

    /// Zero-copy read with UUID key (HOT PATH for LSP).
    ///
    /// Optimized version of [`get`](Self::get) that formats UUID inline
    /// without allocating a separate UUID string.
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
    /// # use uuid::Uuid;
    /// # fn main() -> Result<(), lithos_core::db::DbError> {
    /// let db = Database::open(Path::new("/tmp/test.db"))?;
    /// let id = Uuid::new_v7(uuid::Timestamp::now(uuid::NoContext));
    /// // db.get_by_uuid::<MyType, _, _>("my_table", id, |archived| {
    /// //     archived.field.clone()
    /// // })?;
    /// # Ok(())
    /// # }
    /// ```
    #[inline]
    pub fn get_by_uuid<V, F, R>(
        &self,
        table: &str,
        id: uuid::Uuid,
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
        let namespaced_key = NamespacedKey::from_uuid(table, id);
        read_archived::<V, _, _>(&self.inner, &namespaced_key, f)
    }

    /// Full deserialization with UUID key (COLD PATH).
    ///
    /// Optimized version of [`get_owned`](Self::get_owned) that formats UUID
    /// inline without allocating a separate UUID string.
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
    /// # use uuid::Uuid;
    /// # fn main() -> Result<(), lithos_core::db::DbError> {
    /// let db = Database::open(Path::new("/tmp/test.db"))?;
    /// let id = Uuid::new_v7(uuid::Timestamp::now(uuid::NoContext));
    /// // let value: Option<MyType> = db.get_owned_by_uuid("my_table", id)?;
    /// # Ok(())
    /// # }
    /// ```
    #[inline]
    pub fn get_owned_by_uuid<V>(
        &self,
        table: &str,
        id: uuid::Uuid,
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
        let namespaced_key = NamespacedKey::from_uuid(table, id);
        deserialize_owned::<V>(&self.inner, &namespaced_key)
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
        let multimap_key = MultimapKey::new(key);
        multimap_get_impl(&self.inner, table, &multimap_key)
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
        let prefix = TablePrefix::new(table);
        scan_prefix::<V>(&self.inner, &prefix)
    }
}

// Private helper functions for internal use

/// Zero-copy read of archived data with closure.
fn read_archived<V, F, R>(
    db: &redb::Database,
    key: &NamespacedKey,
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
    let table_ref = tx.open_table(DATA_TABLE)?;

    match table_ref.get(key.as_str())? {
        Some(value) => {
            let bytes: &[u8] = value.value();

            let mut aligned = AlignedVec::<16>::new();
            aligned.extend_from_slice(bytes);

            let archived =
                rkyv::access::<rkyv::Archived<V>, rkyv::rancor::Error>(
                    &aligned,
                )
                .map_err(|e| DbError::Deserialization(e.to_string()))?;
            let result = f(archived);
            Ok(Some(result))
        }
        None => Ok(None),
    }
}

/// Zero-copy read of archived data with closure from a table definition.
fn read_archived_in_table<V, F, R>(
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
        Err(err) => return Err(DbError::Transaction(err.to_string())),
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
            let result = f(archived);
            Ok(Some(result))
        }
        None => Ok(None),
    }
}

/// Full deserialization of archived data into owned value.
fn deserialize_owned<V>(
    db: &redb::Database,
    key: &NamespacedKey,
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
    let table_ref = tx.open_table(DATA_TABLE)?;

    match table_ref.get(key.as_str())? {
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

/// Full deserialization of archived data into owned value from a table.
fn deserialize_owned_in_table<V>(
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
        Err(err) => return Err(DbError::Transaction(err.to_string())),
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

/// Scan all entries with a table prefix and deserialize to owned values.
fn scan_prefix<V>(
    db: &redb::Database,
    prefix: &TablePrefix,
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
    let table_ref = tx.open_table(DATA_TABLE)?;

    let mut results = Vec::new();
    for result in table_ref.range(prefix.as_str()..)? {
        let (key, value) = result?;
        if !key.value().starts_with(prefix.as_str()) {
            break;
        }

        let bytes: &[u8] = value.value();

        let mut aligned = AlignedVec::<16>::new();
        aligned.extend_from_slice(bytes);

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
        Err(err) => return Err(DbError::Transaction(err.to_string())),
    };

    let mut results = Vec::new();
    for result in table_ref.iter()? {
        let (_key, value): (_, redb::AccessGuard<&[u8]>) = result?;
        let bytes: &[u8] = value.value();

        let mut aligned = AlignedVec::<16>::new();
        aligned.extend_from_slice(bytes);

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

/// Get all values for a multimap key.
fn multimap_get_impl(
    db: &redb::Database,
    table: &str,
    key: &MultimapKey,
) -> Result<Vec<String>, DbError> {
    let table_def: MultimapTableDefinition<&str, &str> =
        MultimapTableDefinition::new(table);

    let tx = db.begin_read()?;
    let tbl = tx.open_multimap_table(table_def)?;

    let mut values = Vec::new();
    let range = tbl.get(key.as_str())?;
    for result in range {
        let guard = result?;
        let value: &str = guard.value();
        values.push(value.to_owned());
    }

    Ok(values)
}

/// Get all values for a multimap table definition.
fn multimap_get_in_table_impl(
    db: &redb::Database,
    table: MultimapTableDefinition<&str, &str>,
    key: &str,
) -> Result<Vec<String>, DbError> {
    let tx = db.begin_read()?;
    let tbl = match tx.open_multimap_table(table) {
        Ok(tbl) => tbl,
        Err(TableError::TableDoesNotExist(_)) => return Ok(Vec::new()),
        Err(err) => return Err(DbError::Transaction(err.to_string())),
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

#[cfg(test)]
#[expect(
    clippy::disallowed_methods,
    reason = "Test setup uses expect for concise failure messages"
)]
mod tests {
    use super::*;

    mod read_operations {
        use super::*;

        mod tables {
            use super::*;

            pub(super) const DATA_TABLE_DEF: TableDefinition<&str, &[u8]> =
                TableDefinition::new("data");
            pub(super) const TAGS_TABLE: MultimapTableDefinition<&str, &str> =
                MultimapTableDefinition::new("tags");
        }

        mod get {
            use super::*;

            #[test]
            fn zero_copy_read_works() {
                let (_temp, db) = temp_db().expect("temp db");

                let value = TestValue {
                    id: 42,
                    name: "Alice".to_owned(),
                };
                db.put("users", "alice", &value).expect("put");

                let result = db
                    .get::<TestValue, _, _>("users", "alice", |archived| {
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
                db.put("users", "existing", &value).expect("put");

                let result = db
                    .get::<TestValue, _, _>("users", "missing", |archived| {
                        archived.id.to_native()
                    })
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
                db.put("items", "test", &value).expect("put");

                db.get::<TestValue, _, _>("items", "test", |archived| {
                    assert_eq!(archived.id.to_native(), 99);
                    assert_eq!(archived.name.as_str(), "Test");
                })
                .expect("get");
            }
        }

        mod get_owned {
            use super::{tables::DATA_TABLE_DEF, *};

            #[test]
            fn deserialization_works_in_table() {
                let (_temp, db) = temp_db().expect("temp db");

                let original = TestValue {
                    id: 234,
                    name: "Casey".to_owned(),
                };
                db.put("users", "casey", &original).expect("put");

                let result: Option<TestValue> = db
                    .get_owned_in_table(DATA_TABLE_DEF, "users:casey")
                    .expect("get_owned_in_table");

                assert_eq!(result, Some(original));
            }

            #[test]
            fn deserialization_works() {
                let (_temp, db) = temp_db().expect("temp db");

                let original = TestValue {
                    id: 123,
                    name: "Bob".to_owned(),
                };
                db.put("users", "bob", &original).expect("put");

                let result: Option<TestValue> =
                    db.get_owned("users", "bob").expect("get_owned");

                assert_eq!(result, Some(original));
            }

            #[test]
            fn returns_none_for_missing_key() {
                let (_temp, db) = temp_db().expect("temp db");

                let value = TestValue {
                    id: 8,
                    name: "Existing".to_owned(),
                };
                db.put("users", "existing", &value).expect("put");

                let result: Option<TestValue> =
                    db.get_owned("users", "missing").expect("get_owned");

                assert_eq!(result, None);
            }
        }

        mod uuid_operations {
            use super::*;

            #[test]
            fn uuid_key_formatting() {
                let (_temp, db) = temp_db().expect("temp db");

                let id =
                    uuid::Uuid::new_v7(uuid::Timestamp::now(uuid::NoContext));
                let value = TestValue {
                    id: 456,
                    name: "Charlie".to_owned(),
                };

                db.put_by_uuid("notes", id, &value).expect("put_by_uuid");

                let result = db
                    .get_by_uuid::<TestValue, _, _>("notes", id, |archived| {
                        archived.id.to_native()
                    })
                    .expect("get_by_uuid");

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

                db.put_by_uuid("items", id, &original).expect("put_by_uuid");

                let result: Option<TestValue> = db
                    .get_owned_by_uuid("items", id)
                    .expect("get_owned_by_uuid");

                assert_eq!(result, Some(original));
            }
        }

        mod multimap {
            use super::{tables::TAGS_TABLE, *};

            #[test]
            fn multimap_get_in_table_returns_all_values() {
                let (_temp, db) = temp_db().expect("temp db");

                db.multimap_insert("tags", "work", "note1")
                    .expect("multimap_insert");
                db.multimap_insert("tags", "work", "note2")
                    .expect("multimap_insert");
                db.multimap_insert("tags", "work", "note3")
                    .expect("multimap_insert");

                let values = db
                    .multimap_get_in_table(TAGS_TABLE, "multimap:work")
                    .expect("multimap_get_in_table");

                assert_eq!(values.len(), 3);
                assert!(values.iter().any(|value| value == "note1"));
                assert!(values.iter().any(|value| value == "note2"));
                assert!(values.iter().any(|value| value == "note3"));
            }

            #[test]
            fn multimap_get_returns_all_values() {
                let (_temp, db) = temp_db().expect("temp db");

                db.multimap_insert("tags", "work", "note1")
                    .expect("multimap_insert");
                db.multimap_insert("tags", "work", "note2")
                    .expect("multimap_insert");
                db.multimap_insert("tags", "work", "note3")
                    .expect("multimap_insert");

                let values =
                    db.multimap_get("tags", "work").expect("multimap_get");

                assert_eq!(values.len(), 3);
                assert!(values.iter().any(|value| value == "note1"));
                assert!(values.iter().any(|value| value == "note2"));
                assert!(values.iter().any(|value| value == "note3"));
            }

            #[test]
            fn empty_multimap_returns_empty_vec() {
                let (_temp, db) = temp_db().expect("temp db");

                db.multimap_insert("tags", "other", "note1")
                    .expect("multimap_insert");

                let values = db
                    .multimap_get("tags", "nonexistent")
                    .expect("multimap_get");

                assert_eq!(values.len(), 0);
            }
        }

        mod list_owned {
            use super::{tables::DATA_TABLE_DEF, *};

            #[test]
            fn lists_all_entries_in_table_def() {
                let (_temp, db) = temp_db().expect("temp db");

                db.put("users", "alice", &TestValue {
                    id: 10,
                    name: "Alice".to_owned(),
                })
                .expect("put");
                db.put("users", "bob", &TestValue {
                    id: 20,
                    name: "Bob".to_owned(),
                })
                .expect("put");

                let results: Vec<TestValue> = db
                    .list_owned_in_table(DATA_TABLE_DEF)
                    .expect("list_owned_in_table");

                assert_eq!(results.len(), 2);
                assert!(results.iter().any(|v| v.id == 10));
                assert!(results.iter().any(|v| v.id == 20));
            }

            #[test]
            fn lists_all_entries_in_table() {
                let (_temp, db) = temp_db().expect("temp db");

                db.put("users", "alice", &TestValue {
                    id: 1,
                    name: "Alice".to_owned(),
                })
                .expect("put");
                db.put("users", "bob", &TestValue {
                    id: 2,
                    name: "Bob".to_owned(),
                })
                .expect("put");
                db.put("users", "charlie", &TestValue {
                    id: 3,
                    name: "Charlie".to_owned(),
                })
                .expect("put");

                let results: Vec<TestValue> =
                    db.list_owned("users").expect("list_owned");

                assert_eq!(results.len(), 3);
                assert!(results.iter().any(|v| v.id == 1));
                assert!(results.iter().any(|v| v.id == 2));
                assert!(results.iter().any(|v| v.id == 3));
            }

            #[test]
            fn respects_table_prefix_boundaries() {
                let (_temp, db) = temp_db().expect("temp db");

                db.put("users", "alice", &TestValue {
                    id: 1,
                    name: "Alice".to_owned(),
                })
                .expect("put");
                db.put("notes", "note1", &TestValue {
                    id: 100,
                    name: "Note".to_owned(),
                })
                .expect("put");

                let users: Vec<TestValue> =
                    db.list_owned("users").expect("list_owned");
                let notes: Vec<TestValue> =
                    db.list_owned("notes").expect("list_owned");

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
