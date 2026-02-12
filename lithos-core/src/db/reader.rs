//! Database read operations.
//!
//! This module contains all zero-copy read operations using closure-based APIs
//! to keep transactions properly scoped.

use redb::{
    MultimapTableDefinition, ReadableDatabase as _, ReadableTable as _,
    TableDefinition, TableError,
};
use rkyv::util::AlignedVec;

use super::{Database, DbError};

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
}

// Private helper functions for internal use

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
                db.put_in_table(USERS_TABLE, "alice", &value)
                    .expect("put_in_table");

                let result = db
                    .get_in_table::<TestValue, _, _>(
                        USERS_TABLE,
                        "alice",
                        |archived| archived.id.to_native(),
                    )
                    .expect("get_in_table");

                assert_eq!(result, Some(42));
            }

            #[test]
            fn returns_none_for_missing_key() {
                let (_temp, db) = temp_db().expect("temp db");

                let value = TestValue {
                    id: 7,
                    name: "Existing".to_owned(),
                };
                db.put_in_table(USERS_TABLE, "existing", &value)
                    .expect("put_in_table");

                let result = db
                    .get_in_table::<TestValue, _, _>(
                        USERS_TABLE,
                        "missing",
                        |archived| archived.id.to_native(),
                    )
                    .expect("get_in_table");

                assert_eq!(result, None);
            }

            #[test]
            fn closure_receives_archived_data() {
                let (_temp, db) = temp_db().expect("temp db");

                let value = TestValue {
                    id: 99,
                    name: "Test".to_owned(),
                };
                db.put_in_table(USERS_TABLE, "test", &value)
                    .expect("put_in_table");

                db.get_in_table::<TestValue, _, _>(
                    USERS_TABLE,
                    "test",
                    |archived| {
                        assert_eq!(archived.id.to_native(), 99);
                        assert_eq!(archived.name.as_str(), "Test");
                    },
                )
                .expect("get_in_table");
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
                db.put_in_table(USERS_TABLE, "casey", &original)
                    .expect("put_in_table");

                let result: Option<TestValue> = db
                    .get_owned_in_table(USERS_TABLE, "casey")
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
                db.put_in_table(USERS_TABLE, "bob", &original)
                    .expect("put_in_table");

                let result: Option<TestValue> = db
                    .get_owned_in_table(USERS_TABLE, "bob")
                    .expect("get_owned_in_table");

                assert_eq!(result, Some(original));
            }

            #[test]
            fn returns_none_for_missing_key() {
                let (_temp, db) = temp_db().expect("temp db");

                let value = TestValue {
                    id: 8,
                    name: "Existing".to_owned(),
                };
                db.put_in_table(USERS_TABLE, "existing", &value)
                    .expect("put_in_table");

                let result: Option<TestValue> = db
                    .get_owned_in_table(USERS_TABLE, "missing")
                    .expect("get_owned_in_table");

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

                db.put_by_uuid_in_table(NOTES_TABLE, id, &value)
                    .expect("put_by_uuid_in_table");

                let result = db
                    .get_in_table::<TestValue, _, _>(
                        NOTES_TABLE,
                        &id.to_string(),
                        |archived| archived.id.to_native(),
                    )
                    .expect("get_in_table");

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

                db.put_by_uuid_in_table(ITEMS_TABLE, id, &original)
                    .expect("put_by_uuid_in_table");

                let result: Option<TestValue> = db
                    .get_owned_in_table(ITEMS_TABLE, &id.to_string())
                    .expect("get_owned_in_table");

                assert_eq!(result, Some(original));
            }
        }

        mod multimap {
            use super::{tables::TAGS_TABLE, *};

            #[test]
            fn multimap_get_in_table_returns_all_values() {
                let (_temp, db) = temp_db().expect("temp db");

                db.multimap_insert_in_table(TAGS_TABLE, "work", "note1")
                    .expect("multimap_insert_in_table");
                db.multimap_insert_in_table(TAGS_TABLE, "work", "note2")
                    .expect("multimap_insert_in_table");
                db.multimap_insert_in_table(TAGS_TABLE, "work", "note3")
                    .expect("multimap_insert_in_table");

                let values = db
                    .multimap_get_in_table(TAGS_TABLE, "work")
                    .expect("multimap_get_in_table");

                assert_eq!(values.len(), 3);
                assert!(values.iter().any(|value| value == "note1"));
                assert!(values.iter().any(|value| value == "note2"));
                assert!(values.iter().any(|value| value == "note3"));
            }

            #[test]
            fn multimap_get_returns_all_values() {
                let (_temp, db) = temp_db().expect("temp db");

                db.multimap_insert_in_table(TAGS_TABLE, "work", "note1")
                    .expect("multimap_insert_in_table");
                db.multimap_insert_in_table(TAGS_TABLE, "work", "note2")
                    .expect("multimap_insert_in_table");
                db.multimap_insert_in_table(TAGS_TABLE, "work", "note3")
                    .expect("multimap_insert_in_table");

                let values = db
                    .multimap_get_in_table(TAGS_TABLE, "work")
                    .expect("multimap_get_in_table");

                assert_eq!(values.len(), 3);
                assert!(values.iter().any(|value| value == "note1"));
                assert!(values.iter().any(|value| value == "note2"));
                assert!(values.iter().any(|value| value == "note3"));
            }

            #[test]
            fn empty_multimap_returns_empty_vec() {
                let (_temp, db) = temp_db().expect("temp db");

                db.multimap_insert_in_table(TAGS_TABLE, "other", "note1")
                    .expect("multimap_insert_in_table");

                let values = db
                    .multimap_get_in_table(TAGS_TABLE, "nonexistent")
                    .expect("multimap_get_in_table");

                assert_eq!(values.len(), 0);
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

                db.put_in_table(USERS_TABLE, "alice", &TestValue {
                    id: 10,
                    name: "Alice".to_owned(),
                })
                .expect("put_in_table");
                db.put_in_table(USERS_TABLE, "bob", &TestValue {
                    id: 20,
                    name: "Bob".to_owned(),
                })
                .expect("put_in_table");

                let results: Vec<TestValue> = db
                    .list_owned_in_table(USERS_TABLE)
                    .expect("list_owned_in_table");

                assert_eq!(results.len(), 2);
                assert!(results.iter().any(|v| v.id == 10));
                assert!(results.iter().any(|v| v.id == 20));
            }

            #[test]
            fn lists_all_entries_in_table() {
                let (_temp, db) = temp_db().expect("temp db");

                db.put_in_table(USERS_TABLE, "alice", &TestValue {
                    id: 1,
                    name: "Alice".to_owned(),
                })
                .expect("put_in_table");
                db.put_in_table(USERS_TABLE, "bob", &TestValue {
                    id: 2,
                    name: "Bob".to_owned(),
                })
                .expect("put_in_table");
                db.put_in_table(USERS_TABLE, "charlie", &TestValue {
                    id: 3,
                    name: "Charlie".to_owned(),
                })
                .expect("put_in_table");

                let results: Vec<TestValue> = db
                    .list_owned_in_table(USERS_TABLE)
                    .expect("list_owned_in_table");

                assert_eq!(results.len(), 3);
                assert!(results.iter().any(|v| v.id == 1));
                assert!(results.iter().any(|v| v.id == 2));
                assert!(results.iter().any(|v| v.id == 3));
            }

            #[test]
            fn respects_table_prefix_boundaries() {
                let (_temp, db) = temp_db().expect("temp db");

                db.put_in_table(USERS_TABLE, "alice", &TestValue {
                    id: 1,
                    name: "Alice".to_owned(),
                })
                .expect("put_in_table");
                db.put_in_table(NOTES_TABLE, "note1", &TestValue {
                    id: 100,
                    name: "Note".to_owned(),
                })
                .expect("put_in_table");

                let users: Vec<TestValue> = db
                    .list_owned_in_table(USERS_TABLE)
                    .expect("list_owned_in_table");
                let notes: Vec<TestValue> = db
                    .list_owned_in_table(NOTES_TABLE)
                    .expect("list_owned_in_table");

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
