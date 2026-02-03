use super::{DATA_TABLE, DbError};

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
        let namespaced_key = format!("{table}:{key}");

        let bytes = rkyv::to_bytes::<rkyv::rancor::Error>(value)
            .map_err(|e| DbError::Serialization(e.to_string()))?;
        {
            let mut table_ref = self.tx.open_table(DATA_TABLE)?;
            table_ref.insert(namespaced_key.as_str(), bytes.as_slice())?;
        };
        Ok(())
    }

    /// Delete a value by key within the batch transaction.
    ///
    /// # Errors
    ///
    /// Returns `DbError` if the underlying redb table operation fails.
    #[inline]
    pub fn delete(&mut self, table: &str, key: &str) -> Result<bool, DbError> {
        let namespaced_key = format!("{table}:{key}");

        let existed = {
            let mut table_ref = self.tx.open_table(DATA_TABLE)?;
            table_ref.remove(namespaced_key.as_str())?.is_some()
        };
        Ok(existed)
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
        use redb::MultimapTableDefinition;

        let table_def: MultimapTableDefinition<&str, &str> =
            MultimapTableDefinition::new(table);
        let namespaced_key = format!("multimap:{key}");

        {
            let mut tbl = self.tx.open_multimap_table(table_def)?;
            tbl.insert(namespaced_key.as_str(), value)?;
        };
        Ok(())
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
        use redb::MultimapTableDefinition;

        let table_def: MultimapTableDefinition<&str, &str> =
            MultimapTableDefinition::new(table);
        let namespaced_key = format!("multimap:{key}");

        let removed = {
            let mut tbl = self.tx.open_multimap_table(table_def)?;
            tbl.remove(namespaced_key.as_str(), value)?
        };
        Ok(removed)
    }
}
