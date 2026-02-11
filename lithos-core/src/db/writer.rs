//! Database write operations.
//!
//! This module contains all write and batch-write operations, keeping
//! transactions properly scoped and centralized.

use redb::MultimapTableDefinition;

use super::{
    DATA_TABLE, Database, DbError,
    keys::{MultimapKey, NamespacedKey},
};

impl Database {
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
        let namespaced_key = NamespacedKey::new(table, key);
        serialize_and_put::<V>(&self.inner, &namespaced_key, value)
    }

    /// Insert or update a value with UUID key.
    ///
    /// Optimized version of [`put`](Self::put) that formats UUID inline
    /// without allocating a separate UUID string.
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
    /// # use uuid::Uuid;
    /// # fn main() -> Result<(), lithos_core::db::DbError> {
    /// let db = Database::open(Path::new("/tmp/test.db"))?;
    /// let id = Uuid::new_v7(uuid::Timestamp::now(uuid::NoContext));
    /// // db.put_by_uuid("my_table", id, &my_value)?;
    /// # Ok(())
    /// # }
    /// ```
    #[inline]
    pub fn put_by_uuid<V>(
        &self,
        table: &str,
        id: uuid::Uuid,
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
        let namespaced_key = NamespacedKey::from_uuid(table, id);
        serialize_and_put::<V>(&self.inner, &namespaced_key, value)
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
        let namespaced_key = NamespacedKey::new(table, key);
        delete_key(&self.inner, &namespaced_key)
    }

    /// Delete a value by UUID key.
    ///
    /// Optimized version of [`delete`](Self::delete) that formats UUID inline
    /// without allocating a separate UUID string.
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
    /// # use uuid::Uuid;
    /// # fn main() -> Result<(), lithos_core::db::DbError> {
    /// let db = Database::open(Path::new("/tmp/test.db"))?;
    /// let id = Uuid::new_v7(uuid::Timestamp::now(uuid::NoContext));
    /// // let was_deleted = db.delete_by_uuid("my_table", id)?;
    /// # Ok(())
    /// # }
    /// ```
    #[inline]
    pub fn delete_by_uuid(
        &self,
        table: &str,
        id: uuid::Uuid,
    ) -> Result<bool, DbError> {
        let namespaced_key = NamespacedKey::from_uuid(table, id);
        delete_key(&self.inner, &namespaced_key)
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
        batch_write_impl(&self.inner, f)
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
        let multimap_key = MultimapKey::new(key);
        multimap_insert_impl(&self.inner, table, &multimap_key, value)
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
        let multimap_key = MultimapKey::new(key);
        multimap_remove_impl(&self.inner, table, &multimap_key, value)
    }
}

/// A single write transaction for batching many operations.
///
/// This is intentionally scoped to a closure (see `Database::batch_write`) so
/// callers cannot accidentally hold a transaction across unrelated work.
pub struct WriteBatch {
    tx: redb::WriteTransaction,
}

impl WriteBatch {
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

    /// Insert or update a value within the batch transaction.
    ///
    /// # Errors
    ///
    /// Returns `DbError` if serialization fails or if the underlying redb table
    /// operation fails.
    #[inline]
    pub fn put<V>(
        &mut self,
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
        let namespaced_key = NamespacedKey::new(table, key);
        serialize_and_put_tx::<V>(&mut self.tx, &namespaced_key, value)
    }

    /// Delete a value by key within the batch transaction.
    ///
    /// # Errors
    ///
    /// Returns `DbError` if the underlying redb table operation fails.
    #[inline]
    pub fn delete(&mut self, table: &str, key: &str) -> Result<bool, DbError> {
        let namespaced_key = NamespacedKey::new(table, key);
        delete_key_tx(&mut self.tx, &namespaced_key)
    }

    /// Insert a value into a multimap within the batch transaction.
    ///
    /// # Errors
    ///
    /// Returns `DbError` if the underlying redb multimap table operation fails.
    #[inline]
    pub fn multimap_insert(
        &mut self,
        table: &str,
        key: &str,
        value: &str,
    ) -> Result<(), DbError> {
        let multimap_key = MultimapKey::new(key);
        multimap_insert_tx(&mut self.tx, table, &multimap_key, value)
    }

    /// Remove a value from a multimap within the batch transaction.
    ///
    /// # Errors
    ///
    /// Returns `DbError` if the underlying redb multimap table operation fails.
    #[inline]
    pub fn multimap_remove(
        &mut self,
        table: &str,
        key: &str,
        value: &str,
    ) -> Result<bool, DbError> {
        let multimap_key = MultimapKey::new(key);
        multimap_remove_tx(&mut self.tx, table, &multimap_key, value)
    }
}

// Private helper functions for internal use

/// Serialize a value and write it under the provided key.
fn serialize_and_put<V>(
    db: &redb::Database,
    key: &NamespacedKey,
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
    serialize_and_put_tx::<V>(&mut tx, key, value)?;
    tx.commit()?;
    Ok(())
}

/// Serialize a value and write it under the provided key in a transaction.
fn serialize_and_put_tx<V>(
    tx: &mut redb::WriteTransaction,
    key: &NamespacedKey,
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
        let mut table_ref = tx.open_table(DATA_TABLE)?;
        table_ref.insert(key.as_str(), bytes.as_slice())?;
    };
    Ok(())
}

/// Delete a value by key.
fn delete_key(
    db: &redb::Database,
    key: &NamespacedKey,
) -> Result<bool, DbError> {
    let mut tx = db.begin_write()?;
    let existed = delete_key_tx(&mut tx, key)?;
    tx.commit()?;
    Ok(existed)
}

/// Delete a value by key in a transaction.
fn delete_key_tx(
    tx: &mut redb::WriteTransaction,
    key: &NamespacedKey,
) -> Result<bool, DbError> {
    let existed = {
        let mut table_ref = tx.open_table(DATA_TABLE)?;
        table_ref.remove(key.as_str())?.is_some()
    };
    Ok(existed)
}

/// Insert a value into a multimap.
fn multimap_insert_impl(
    db: &redb::Database,
    table: &str,
    key: &MultimapKey,
    value: &str,
) -> Result<(), DbError> {
    let mut tx = db.begin_write()?;
    multimap_insert_tx(&mut tx, table, key, value)?;
    tx.commit()?;
    Ok(())
}

/// Insert a value into a multimap in a transaction.
fn multimap_insert_tx(
    tx: &mut redb::WriteTransaction,
    table: &str,
    key: &MultimapKey,
    value: &str,
) -> Result<(), DbError> {
    let table_def: MultimapTableDefinition<&str, &str> =
        MultimapTableDefinition::new(table);
    {
        let mut tbl = tx.open_multimap_table(table_def)?;
        tbl.insert(key.as_str(), value)?;
    };
    Ok(())
}

/// Remove a value from a multimap.
fn multimap_remove_impl(
    db: &redb::Database,
    table: &str,
    key: &MultimapKey,
    value: &str,
) -> Result<bool, DbError> {
    let mut tx = db.begin_write()?;
    let removed = multimap_remove_tx(&mut tx, table, key, value)?;
    tx.commit()?;
    Ok(removed)
}

/// Remove a value from a multimap in a transaction.
fn multimap_remove_tx(
    tx: &mut redb::WriteTransaction,
    table: &str,
    key: &MultimapKey,
    value: &str,
) -> Result<bool, DbError> {
    let table_def: MultimapTableDefinition<&str, &str> =
        MultimapTableDefinition::new(table);
    let removed = {
        let mut tbl = tx.open_multimap_table(table_def)?;
        tbl.remove(key.as_str(), value)?
    };
    Ok(removed)
}

/// Execute a batch of write operations with a single commit.
fn batch_write_impl<F>(db: &redb::Database, f: F) -> Result<(), DbError>
where
    F: FnOnce(&mut WriteBatch) -> Result<(), DbError>,
{
    let tx = db.begin_write()?;
    let mut batch = WriteBatch::new(tx);

    f(&mut batch)?;
    batch.commit()?;
    Ok(())
}

#[cfg(test)]
#[expect(
    clippy::disallowed_methods,
    reason = "Test setup uses expect for concise failure messages"
)]
mod tests {
    use super::*;

    mod write_operations {
        use super::*;

        mod put {
            use super::*;

            #[test]
            fn inserts_value() {
                let (_temp, db) = temp_db().expect("temp db");
                let value = TestValue {
                    id: 1,
                    name: "Alice".to_owned(),
                };

                db.put("users", "alice", &value).expect("put");

                let fetched: Option<TestValue> =
                    db.get_owned("users", "alice").expect("get_owned");
                assert_eq!(fetched, Some(value));
            }
        }

        mod delete {
            use super::*;

            #[test]
            fn removes_value() {
                let (_temp, db) = temp_db().expect("temp db");
                let value = TestValue {
                    id: 2,
                    name: "Bob".to_owned(),
                };

                db.put("users", "bob", &value).expect("put");

                let removed = db.delete("users", "bob").expect("delete");
                assert!(removed);

                let fetched: Option<TestValue> =
                    db.get_owned("users", "bob").expect("get_owned");
                assert_eq!(fetched, None);
            }

            #[test]
            fn returns_false_for_missing_key() {
                let (_temp, db) = temp_db().expect("temp db");
                let value = TestValue {
                    id: 3,
                    name: "Existing".to_owned(),
                };

                db.put("users", "existing", &value).expect("put");

                let removed = db.delete("users", "missing").expect("delete");
                assert!(!removed);
            }
        }

        mod uuid_operations {
            use super::*;

            #[test]
            fn writes_and_reads_by_uuid() {
                let (_temp, db) = temp_db().expect("temp db");
                let id =
                    uuid::Uuid::new_v7(uuid::Timestamp::now(uuid::NoContext));
                let value = TestValue {
                    id: 10,
                    name: "Note".to_owned(),
                };

                db.put_by_uuid("notes", id, &value).expect("put_by_uuid");

                let fetched: Option<TestValue> = db
                    .get_owned_by_uuid("notes", id)
                    .expect("get_owned_by_uuid");
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

                db.put_by_uuid("notes", id, &value).expect("put_by_uuid");

                let removed =
                    db.delete_by_uuid("notes", id).expect("delete_by_uuid");
                assert!(removed);

                let fetched: Option<TestValue> = db
                    .get_owned_by_uuid("notes", id)
                    .expect("get_owned_by_uuid");
                assert_eq!(fetched, None);
            }
        }

        mod multimap {
            use super::*;

            #[test]
            fn insert_and_remove_values() {
                let (_temp, db) = temp_db().expect("temp db");

                db.multimap_insert("tags", "work", "note1")
                    .expect("multimap_insert");
                db.multimap_insert("tags", "work", "note2")
                    .expect("multimap_insert");

                let values_before =
                    db.multimap_get("tags", "work").expect("multimap_get");
                assert_eq!(values_before.len(), 2);
                assert!(values_before.contains(&"note1".to_owned()));
                assert!(values_before.contains(&"note2".to_owned()));

                let removed = db
                    .multimap_remove("tags", "work", "note1")
                    .expect("multimap_remove");
                assert!(removed);

                let values_after =
                    db.multimap_get("tags", "work").expect("multimap_get");
                assert_eq!(values_after.len(), 1);
                let value = values_after.first().expect("multimap value");
                assert_eq!(value, "note2");
            }

            #[test]
            fn remove_returns_false_for_missing_value() {
                let (_temp, db) = temp_db().expect("temp db");

                db.multimap_insert("tags", "other", "note1")
                    .expect("multimap_insert");

                let removed = db
                    .multimap_remove("tags", "missing", "note1")
                    .expect("multimap_remove");
                assert!(!removed);
            }
        }
    }

    mod batch_operations {
        use super::*;

        #[test]
        fn batch_write_inserts_values() {
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
                batch.put("users", "one", &value1)?;
                batch.put("users", "two", &value2)?;
                batch.multimap_insert("tags", "batch", "one")?;
                Ok(())
            })
            .expect("batch_write");

            let fetched_one: Option<TestValue> =
                db.get_owned("users", "one").expect("get_owned");
            assert_eq!(fetched_one, Some(value1));

            let fetched_two: Option<TestValue> =
                db.get_owned("users", "two").expect("get_owned");
            assert_eq!(fetched_two, Some(value2));

            let tags = db.multimap_get("tags", "batch").expect("multimap_get");
            assert_eq!(tags, vec!["one".to_owned()]);
        }

        #[test]
        fn batch_write_rolls_back_on_error() {
            let (_temp, db) = temp_db().expect("temp db");
            let existing = TestValue {
                id: 30,
                name: "Existing".to_owned(),
            };
            let temp_value = TestValue {
                id: 31,
                name: "Temp".to_owned(),
            };

            db.put("users", "existing", &existing).expect("put");

            let result = db.batch_write(|batch| {
                batch.put("users", "temp", &temp_value)?;
                Err(DbError::Transaction("intentional".to_owned()))
            });

            assert!(result.is_err());

            let fetched_temp: Option<TestValue> =
                db.get_owned("users", "temp").expect("get_owned");
            assert_eq!(fetched_temp, None);

            let fetched_existing: Option<TestValue> =
                db.get_owned("users", "existing").expect("get_owned");
            assert_eq!(fetched_existing, Some(existing));
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
