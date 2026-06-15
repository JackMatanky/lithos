use std::sync::Arc;

use crate::{
    db::DbError,
    indexer::{
        IndexerRepositoryError,
        storage::tables::{
            DIR_ID_BY_PATH, DIR_IDS_BY_PARENT, DIRS, FILE_ID_BY_PATH,
            FILE_IDS_BY_BASENAME, FILE_IDS_BY_FORMAT, FILE_IDS_BY_PARENT,
            FILES,
        },
    },
};

pub(crate) struct RedbRepository {
    pub(crate) store: Arc<crate::db::Store>,
}

impl RedbRepository {
    pub(crate) fn try_new(
        store: Arc<crate::db::Store>,
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

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use tempfile::NamedTempFile;

    use super::*;
    use crate::db::Store;

    #[test]
    fn test_redb_repository_construction() {
        let tmp_file = NamedTempFile::new().unwrap();
        let store = Arc::new(
            Store::open(tmp_file.path()).expect("failed to open database"),
        );
        let _repo = RedbRepository::try_new(store).unwrap();
    }
}
