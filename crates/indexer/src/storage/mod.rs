//! Storage layer for the indexer context.
//!
//! Provides a [`RedbRepository`] backed by redb tables that implements both
//! [`ReadRepository`] and [`WriteRepository`] traits. Records are stored as
//! rkyv-archived bytes (`&[u8]`) and deserialized on read, following the same
//! pattern used by all other contexts.
//!
//! ## Submodules
//!
//! | Module | Purpose |
//! |--------|---------|
//! | [`tables`] | Database table definitions (constants + type wrappers) |
//! | [`read`]   | Read-only query implementation |
//! | [`write`]  | Write (create/update/delete) implementation |
//! | [`testing`] | In-memory repository for tests (cfg test only) |
//!
//! [`ReadRepository`]: crate::repository::ReadRepository
//! [`WriteRepository`]: crate::repository::WriteRepository

use std::sync::Arc;

use crate::{
    IndexerRepositoryError,
    storage::tables::{
        DIR_ID_BY_PATH, DIR_IDS_BY_PARENT, DIRS, FILE_ID_BY_PATH,
        FILE_IDS_BY_BASENAME, FILE_IDS_BY_FORMAT, FILE_IDS_BY_PARENT, FILES,
    },
};

/// Redb-backed repository implementing [`ReadRepository`] and
/// [`WriteRepository`] for the indexer context.
///
/// Wraps a shared [`Store`] reference and opens all required tables
/// at construction.
///
/// [`ReadRepository`]: crate::repository::ReadRepository
/// [`WriteRepository`]: crate::repository::WriteRepository
/// [`Store`]: trace_db::Store
pub struct RedbRepository {
    pub(crate) store: Arc<trace_db::Store>,
}

impl RedbRepository {
    /// Opens all indexer tables and returns a ready-to-use repository.
    ///
    /// # Errors
    ///
    /// Returns [`IndexerRepositoryError`] if any table cannot be created
    /// or opened.
    #[inline]
    pub fn try_new(
        store: Arc<trace_db::Store>,
    ) -> Result<Self, IndexerRepositoryError> {
        // Ensure all tables are created
        store.write(|tx| {
            tx.inner.open_table(FILES.definition())?;
            tx.inner.open_table(DIRS.definition())?;
            tx.inner.open_table(FILE_ID_BY_PATH.definition())?;
            tx.inner.open_table(DIR_ID_BY_PATH.definition())?;
            tx.inner.open_multimap_table(FILE_IDS_BY_BASENAME)?;
            tx.inner.open_multimap_table(FILE_IDS_BY_PARENT.definition())?;
            tx.inner.open_multimap_table(DIR_IDS_BY_PARENT.definition())?;
            tx.inner.open_multimap_table(FILE_IDS_BY_FORMAT)?;
            Ok(())
        })?;

        Ok(Self {
            store,
        })
    }
}

pub(crate) mod read;
pub(crate) mod tables;
pub(crate) mod write;

#[cfg(test)]
pub(crate) mod testing;

#[cfg(test)]
pub(crate) use self::testing::InMemoryRepository;
/// The filename of the Redb database.
pub const INDEX_DB_FILENAME: &str = "index.redb";

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use tempfile::NamedTempFile;
    use trace_db::Store;

    use super::*;

    #[test]
    fn test_redb_repository_construction() {
        let tmp_file = NamedTempFile::new().unwrap();
        let store = Arc::new(
            Store::open(tmp_file.path()).expect("failed to open database"),
        );
        let _repo = RedbRepository::try_new(store).unwrap();
    }
}
