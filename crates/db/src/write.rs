//! Write transaction wrapper.

#![allow(dead_code, reason = "dead code until issue-02")]

use redb::{
    Key, MultimapTable, MultimapTableDefinition, Table, TableDefinition, Value,
};

use crate::DbError;

/// Read-write transaction wrapper.
///
/// Provides scoped access to read-write database operations within a
/// [`Store`](crate::Store).
pub struct WriteTx {
    /// The underlying redb write transaction.
    pub inner: redb::WriteTransaction,
}

impl WriteTx {
    /// Attempt to open a table for writing.
    ///
    /// Automatically creates the table if it does not already exist.
    ///
    /// # Errors
    ///
    /// Returns [`DbError`] if the underlying database operation fails.
    #[inline]
    pub fn try_open_table<K, V>(
        &self,
        definition: TableDefinition<'static, K, V>,
    ) -> Result<Table<'_, K, V>, DbError>
    where
        K: Key + 'static,
        V: Value + 'static,
    {
        self.inner.open_table(definition).map_err(Into::into)
    }

    /// Attempt to open a multimap table for writing.
    ///
    /// Automatically creates the table if it does not already exist.
    ///
    /// # Errors
    ///
    /// Returns [`DbError`] if the underlying database operation fails.
    #[inline]
    pub fn try_open_multimap<K, V>(
        &self,
        definition: MultimapTableDefinition<'static, K, V>,
    ) -> Result<MultimapTable<'_, K, V>, DbError>
    where
        K: Key + 'static,
        V: Key + 'static,
    {
        self.inner.open_multimap_table(definition).map_err(Into::into)
    }
}

// ----------------------------------------------------------- //
//                            Tests                            //
// ----------------------------------------------------------- //

#[cfg(test)]
mod tests {
    use redb::{TableDefinition, TableHandle};
    use tempfile::tempdir;

    // use super::*;
    use crate::Store;

    const TEST_TABLE: TableDefinition<&str, &str> =
        TableDefinition::new("test");

    #[test]
    #[allow(
        clippy::panic_in_result_fn,
        reason = "Test assertions are expected to panic"
    )]
    fn try_open_table_creates_table() -> Result<(), Box<dyn std::error::Error>>
    {
        let temp = tempdir()?;
        let db_path = temp.path().join("test.db");
        let store = Store::open(&db_path)?;

        // Open table (should create it)
        store.write(|tx| {
            let _ = tx.try_open_table(TEST_TABLE)?;
            Ok(())
        })?;

        // Verify it exists
        store.read(|tx| {
            let table = tx.inner.open_table(TEST_TABLE)?;
            assert_eq!(table.name(), "test");
            Ok(())
        })?;

        Ok(())
    }
}
