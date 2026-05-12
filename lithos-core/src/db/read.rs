//! Read transaction wrapper.

#![expect(
    clippy::field_scoped_visibility_modifiers,
    reason = "Exposing inner field for storage adapter access"
)]
#![allow(dead_code, reason = "dead code until issue-02")]

use redb::{
    Key, MultimapTableDefinition, ReadOnlyMultimapTable, ReadOnlyTable,
    TableDefinition, Value,
};

use crate::db::DbError;

/// Read-only transaction wrapper.
///
/// Provides scoped access to read-only database operations within a
/// [`Store`](crate::db::Store).
pub struct ReadTx {
    pub(crate) inner: redb::ReadTransaction,
}

impl ReadTx {
    /// Attempt to open a table for reading.
    ///
    /// Returns `Ok(Some(table))` if the table exists, or `Ok(None)` if it does
    /// not.
    ///
    /// # Errors
    ///
    /// Returns [`DbError`] if the underlying database operation fails.
    #[inline]
    pub fn try_open_table<K, V>(
        &self,
        definition: TableDefinition<'static, K, V>,
    ) -> Result<Option<ReadOnlyTable<K, V>>, DbError>
    where
        K: Key + 'static,
        V: Value + 'static,
    {
        match self.inner.open_table(definition) {
            Ok(table) => Ok(Some(table)),
            Err(redb::TableError::TableDoesNotExist(_)) => Ok(None),
            Err(e) => Err(e.into()),
        }
    }

    /// Attempt to open a multimap table for reading.
    ///
    /// Returns `Ok(Some(table))` if the table exists, or `Ok(None)` if it does
    /// not.
    ///
    /// # Errors
    ///
    /// Returns [`DbError`] if the underlying database operation fails.
    #[inline]
    pub fn try_open_multimap<K, V>(
        &self,
        definition: MultimapTableDefinition<'static, K, V>,
    ) -> Result<Option<ReadOnlyMultimapTable<K, V>>, DbError>
    where
        K: Key + 'static,
        V: Key + 'static,
    {
        match self.inner.open_multimap_table(definition) {
            Ok(table) => Ok(Some(table)),
            Err(redb::TableError::TableDoesNotExist(_)) => Ok(None),
            Err(e) => Err(e.into()),
        }
    }
}

#[cfg(test)]
mod tests {
    use redb::TableDefinition;
    use tempfile::tempdir;

    // use super::*;
    use crate::db::Store;

    const TEST_TABLE: TableDefinition<&str, &str> =
        TableDefinition::new("test");

    #[test]
    #[allow(
        clippy::panic_in_result_fn,
        reason = "Test assertions are expected to panic"
    )]
    fn try_open_table_returns_some_when_exists()
    -> Result<(), Box<dyn std::error::Error>> {
        let temp = tempdir()?;
        let db_path = temp.path().join("test.db");
        let store = Store::open(&db_path)?;

        // Create table
        store.write(|tx| {
            let _ = tx.inner.open_table(TEST_TABLE)?;
            Ok(())
        })?;

        // Read table
        store.read(|tx| {
            let table = tx.try_open_table(TEST_TABLE)?;
            assert!(table.is_some());
            Ok(())
        })?;

        Ok(())
    }

    #[test]
    #[allow(
        clippy::panic_in_result_fn,
        reason = "Test assertions are expected to panic"
    )]
    fn try_open_table_returns_none_when_missing()
    -> Result<(), Box<dyn std::error::Error>> {
        let temp = tempdir()?;
        let db_path = temp.path().join("test.db");
        let store = Store::open(&db_path)?;

        // Read table without creating it
        store.read(|tx| {
            let table = tx.try_open_table(TEST_TABLE)?;
            assert!(table.is_none());
            Ok(())
        })?;

        Ok(())
    }
}
