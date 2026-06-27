//! In-memory repository implementation for testing.

use std::{
    collections::HashMap,
    sync::{Arc, RwLock},
};

use traces_fs::{FileFormat, path::PathKey};

use crate::{
    error::IndexerRepositoryError,
    model::{DirRecord, FileRecord, FsParentId, FsRecordId},
    repository::{ReadRepository, WriteRepository},
};

/// A test-only in-memory repository implementation.
#[derive(Clone)]
pub(crate) struct InMemoryRepository {
    state: Arc<RwLock<RepositoryState>>,
}

struct RepositoryState {
    files: HashMap<FsRecordId, FileRecord>,
    dirs: HashMap<FsRecordId, DirRecord>,
    file_path_to_id: HashMap<PathKey, FsRecordId>,
    dir_path_to_id: HashMap<PathKey, FsRecordId>,
    files_by_basename: HashMap<String, Vec<FsRecordId>>,
    files_by_parent: HashMap<FsRecordId, Vec<FsRecordId>>,
    files_by_format: HashMap<FileFormat, Vec<FsRecordId>>,
    dirs_by_parent: HashMap<FsRecordId, Vec<FsRecordId>>,
}

impl InMemoryRepository {
    /// Creates a new, empty in-memory repository.
    pub(crate) fn new() -> Self {
        Self {
            state: Arc::new(RwLock::new(RepositoryState {
                files: HashMap::new(),
                dirs: HashMap::new(),
                file_path_to_id: HashMap::new(),
                dir_path_to_id: HashMap::new(),
                files_by_basename: HashMap::new(),
                files_by_parent: HashMap::new(),
                files_by_format: HashMap::new(),
                dirs_by_parent: HashMap::new(),
            })),
        }
    }
}

impl ReadRepository for InMemoryRepository {
    fn find_file(
        &self,
        id: FsRecordId,
    ) -> Result<Option<FileRecord>, IndexerRepositoryError> {
        let state = self.state.read().unwrap();
        Ok(state.files.get(&id).cloned())
    }

    fn find_dir(
        &self,
        id: FsRecordId,
    ) -> Result<Option<DirRecord>, IndexerRepositoryError> {
        let state = self.state.read().unwrap();
        Ok(state.dirs.get(&id).cloned())
    }

    fn find_file_by_path(
        &self,
        path: &PathKey,
    ) -> Result<Option<FileRecord>, IndexerRepositoryError> {
        let state = self.state.read().unwrap();
        if let Some(id) = state.file_path_to_id.get(path) {
            Ok(state.files.get(id).cloned())
        } else {
            Ok(None)
        }
    }

    fn find_dir_by_path(
        &self,
        path: &PathKey,
    ) -> Result<Option<DirRecord>, IndexerRepositoryError> {
        let state = self.state.read().unwrap();
        if let Some(id) = state.dir_path_to_id.get(path) {
            Ok(state.dirs.get(id).cloned())
        } else {
            Ok(None)
        }
    }

    fn list_files_by_parent(
        &self,
        parent_id: FsParentId,
    ) -> Result<Box<[FileRecord]>, IndexerRepositoryError> {
        let state = self.state.read().unwrap();
        let key = parent_id.to_storage_key();
        if let Some(ids) = state.files_by_parent.get(&key) {
            Ok(ids
                .iter()
                .filter_map(|id| state.files.get(id))
                .cloned()
                .collect())
        } else {
            Ok(Box::new([]))
        }
    }

    fn list_dirs_by_parent(
        &self,
        parent_id: FsParentId,
    ) -> Result<Box<[DirRecord]>, IndexerRepositoryError> {
        let state = self.state.read().unwrap();
        let key = parent_id.to_storage_key();
        if let Some(ids) = state.dirs_by_parent.get(&key) {
            Ok(ids
                .iter()
                .filter_map(|id| state.dirs.get(id))
                .cloned()
                .collect())
        } else {
            Ok(Box::new([]))
        }
    }

    fn list_files_by_format(
        &self,
        format: FileFormat,
    ) -> Result<Box<[FileRecord]>, IndexerRepositoryError> {
        let state = self.state.read().unwrap();
        if let Some(ids) = state.files_by_format.get(&format) {
            Ok(ids
                .iter()
                .filter_map(|id| state.files.get(id))
                .cloned()
                .collect())
        } else {
            Ok(Box::new([]))
        }
    }

    fn list_files_by_basename(
        &self,
        basename: &str,
    ) -> Result<Box<[FileRecord]>, IndexerRepositoryError> {
        let state = self.state.read().unwrap();
        if let Some(ids) = state.files_by_basename.get(basename) {
            Ok(ids
                .iter()
                .filter_map(|id| state.files.get(id))
                .cloned()
                .collect())
        } else {
            Ok(Box::new([]))
        }
    }

    fn all_paths(&self) -> Result<Box<[PathKey]>, IndexerRepositoryError> {
        let state = self.state.read().unwrap();
        let mut paths = Vec::new();
        let mut seen = std::collections::HashSet::new();
        for pk in
            state.file_path_to_id.keys().chain(state.dir_path_to_id.keys())
        {
            if seen.insert(pk.clone()) {
                paths.push(pk.clone());
            }
        }
        Ok(paths.into_boxed_slice())
    }
}

impl WriteRepository for InMemoryRepository {
    fn save_file(
        &self,
        record: &FileRecord,
    ) -> Result<(), IndexerRepositoryError> {
        let mut state = self.state.write().unwrap();
        Self::save_file_internal(&mut state, record);
        Ok(())
    }

    fn save_dir(
        &self,
        record: &DirRecord,
    ) -> Result<(), IndexerRepositoryError> {
        let mut state = self.state.write().unwrap();
        Self::save_dir_internal(&mut state, record);
        Ok(())
    }

    fn delete_file(
        &self,
        id: FsRecordId,
    ) -> Result<(), IndexerRepositoryError> {
        let mut state = self.state.write().unwrap();
        Self::delete_file_internal(&mut state, id);
        Ok(())
    }

    fn delete_dir(&self, id: FsRecordId) -> Result<(), IndexerRepositoryError> {
        let mut state = self.state.write().unwrap();
        Self::delete_dir_internal(&mut state, id);
        Ok(())
    }

    fn save_many_records(
        &self,
        files: &[FileRecord],
        dirs: &[DirRecord],
    ) -> Result<(), IndexerRepositoryError> {
        let mut state = self.state.write().unwrap();
        for file in files {
            Self::save_file_internal(&mut state, file);
        }
        for dir in dirs {
            Self::save_dir_internal(&mut state, dir);
        }
        Ok(())
    }

    fn delete_many_records(
        &self,
        file_ids: &[FsRecordId],
        dir_ids: &[FsRecordId],
    ) -> Result<(), IndexerRepositoryError> {
        let mut state = self.state.write().unwrap();
        for id in file_ids {
            Self::delete_file_internal(&mut state, *id);
        }
        for id in dir_ids {
            Self::delete_dir_internal(&mut state, *id);
        }
        Ok(())
    }

    fn clear(&self) -> Result<(), IndexerRepositoryError> {
        let mut state = self.state.write().unwrap();
        Self::clear_internal(&mut state);
        Ok(())
    }
}

impl InMemoryRepository {
    fn clear_internal(state: &mut RepositoryState) {
        state.files.clear();
        state.dirs.clear();
        state.file_path_to_id.clear();
        state.dir_path_to_id.clear();
        state.files_by_basename.clear();
        state.files_by_parent.clear();
        state.files_by_format.clear();
        state.dirs_by_parent.clear();
    }

    fn save_file_internal(state: &mut RepositoryState, record: &FileRecord) {
        let id = record.id();

        // Cleanup old indexes if updating
        if let Some(old) = state.files.get(&id) {
            state.file_path_to_id.remove(old.path());
            if let Some(ids) =
                state.files_by_basename.get_mut(old.name().as_str())
            {
                ids.retain(|x| x != &id);
            }
            if let Some(ids) =
                state.files_by_parent.get_mut(&old.parent_id().to_storage_key())
            {
                ids.retain(|x| x != &id);
            }
            if let Some(ids) = state.files_by_format.get_mut(&old.format()) {
                ids.retain(|x| x != &id);
            }
        }

        // Primary
        state.files.insert(id, record.clone());

        // Indexes
        state.file_path_to_id.insert(record.path().clone(), id);
        state
            .files_by_basename
            .entry(record.name().as_str().to_owned())
            .or_default()
            .push(id);
        state
            .files_by_parent
            .entry(record.parent_id().to_storage_key())
            .or_default()
            .push(id);
        state.files_by_format.entry(record.format()).or_default().push(id);
    }

    fn save_dir_internal(state: &mut RepositoryState, record: &DirRecord) {
        let id = record.id();

        // Cleanup old indexes
        if let Some(old) = state.dirs.get(&id) {
            state.dir_path_to_id.remove(old.path());
            let pkey = old.parent_id().to_storage_key();
            if let Some(ids) = state.dirs_by_parent.get_mut(&pkey) {
                ids.retain(|x| x != &id);
            }
        }

        // Primary
        state.dirs.insert(id, record.clone());

        // Indexes
        state.dir_path_to_id.insert(record.path().clone(), id);
        state
            .dirs_by_parent
            .entry(record.parent_id().to_storage_key())
            .or_default()
            .push(id);
    }

    fn delete_file_internal(state: &mut RepositoryState, id: FsRecordId) {
        if let Some(old) = state.files.remove(&id) {
            state.file_path_to_id.remove(old.path());
            if let Some(ids) =
                state.files_by_basename.get_mut(old.name().as_str())
            {
                ids.retain(|x| x != &id);
            }
            if let Some(ids) =
                state.files_by_parent.get_mut(&old.parent_id().to_storage_key())
            {
                ids.retain(|x| x != &id);
            }
            if let Some(ids) = state.files_by_format.get_mut(&old.format()) {
                ids.retain(|x| x != &id);
            }
        }
    }

    fn delete_dir_internal(state: &mut RepositoryState, id: FsRecordId) {
        if let Some(old) = state.dirs.remove(&id) {
            state.dir_path_to_id.remove(old.path());
            let pkey = old.parent_id().to_storage_key();
            if let Some(ids) = state.dirs_by_parent.get_mut(&pkey) {
                ids.retain(|x| x != &id);
            }
        }
    }
}

// ----------------------------------------------------------- //
//                            Tests                            //
// ----------------------------------------------------------- //

#[cfg(test)]
mod tests {
    use std::time::SystemTime;

    use traces_fs::{
        DirMetadata, FileMetadata,
        metadata::FsTimes,
        name::{DirName, FileName},
    };

    use super::*;

    #[test]
    fn all_paths_deduplicates_across_file_and_dir_tables() {
        let repo = InMemoryRepository::new();
        let shared = PathKey::try_new("shared").unwrap();

        repo.save_file(&FileRecord::new(
            FsRecordId::new(),
            FsParentId::Root,
            shared.clone(),
            FileName::new("shared".into()),
            FileFormat::Unknown,
            FileMetadata::new(FsTimes::new(None, None), 0, false),
            SystemTime::now(),
        ))
        .unwrap();
        repo.save_dir(&DirRecord::new(
            FsRecordId::new(),
            FsParentId::Root,
            shared.clone(),
            DirName::new("shared".into()),
            DirMetadata::new(FsTimes::new(None, None), false),
            SystemTime::now(),
        ))
        .unwrap();

        let results = repo.all_paths().unwrap();
        assert_eq!(results.len(), 1);
        assert_eq!(results.first(), Some(&shared));
    }

    #[test]
    fn repository_double_supports_save_and_find() {
        let repo = InMemoryRepository::new();
        let id = FsRecordId::new();
        let path = PathKey::try_new("test.txt").unwrap();
        let record = FileRecord::new(
            id,
            FsParentId::Root,
            path.clone(),
            FileName::new("test.txt".into()),
            FileFormat::Markdown,
            FileMetadata::new(FsTimes::new(None, None), 123, false),
            SystemTime::now(),
        );

        repo.save_file(&record).unwrap();
        let found = repo.find_file(id).unwrap().unwrap();
        assert_eq!(found, record);

        let found_by_path = repo.find_file_by_path(&path).unwrap().unwrap();
        assert_eq!(found_by_path, record);

        repo.delete_file(id).unwrap();
        assert!(repo.find_file(id).unwrap().is_none());
        assert!(repo.find_file_by_path(&path).unwrap().is_none());
    }
}
