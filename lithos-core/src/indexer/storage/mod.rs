use std::sync::Arc;

use crate::{
    db::DbError,
    indexer::storage::tables::{
        DIR_ID_BY_PATH, DIRS, FILE_ID_BY_PATH, FILE_IDS_BY_BASENAME,
        FILE_IDS_BY_FORMAT, FILE_IDS_BY_PARENT, FILES,
    },
};

pub(crate) struct RedbRepository {
    pub(crate) store: Arc<crate::db::Store>,
}

impl RedbRepository {
    pub(crate) fn new(store: Arc<crate::db::Store>) -> Self {
        // Ensure all tables are created
        if let Err(e) = store.write(|tx| {
            tx.inner.open_table(FILES)?;
            tx.inner.open_table(DIRS)?;
            tx.inner.open_table(FILE_ID_BY_PATH)?;
            tx.inner.open_table(DIR_ID_BY_PATH)?;
            tx.inner.open_multimap_table(FILE_IDS_BY_BASENAME)?;
            tx.inner.open_multimap_table(FILE_IDS_BY_PARENT)?;
            tx.inner.open_multimap_table(FILE_IDS_BY_FORMAT)?;
            Ok(())
        }) {
            eprintln!("failed to initialize indexer tables: {e}");
        }

        Self {
            store,
        }
    }
}

pub(crate) mod read;
pub(crate) mod tables;

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
        let _repo = RedbRepository::new(store);
    }
}
