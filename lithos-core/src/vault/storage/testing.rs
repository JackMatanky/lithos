//! In-memory repository test double for vault storage.
//!
//! Provides [`InMemoryRepository`], which implements the segregated repository
//! traits ([`ReadRepository`] and [`WriteRepository`]) using `HashMap`-backed
//! storage. This enables deterministic, fast unit tests without a real
//! database.
//!
//! # Architecture
//!
//! - All state lives behind `Arc<RwLock<...>>` for `Clone`-ability and
//!   thread-safe concurrent access (within a single test).
//! - Primary tables and index maps are kept in lockstep — every write mutates
//!   both the view table and its corresponding path/basename/parent/format
//!   indexes.
//! - An [`InMemoryHarness`] provides failure injection and operation counting
//!   for verifying error handling and measuring read/write/batch call volumes.
//!
//! # Test Organisation
//!
//! Tests in `mod tests` are grouped by capability:
//! - `defaults` — empty-repository invariants
//! - `lookup` — direct lookups and path-based lookups
//! - `list` — table and index scans
//! - `indexes` — multimap index queries
//! - `update` — save / overwrite semantics
//! - `delete` — remove and idempotency
//! - `counters` — operation counting
//! - `injection` — failure injection at `BeforeRead` / `BeforeWrite`

use std::{
    collections::HashMap,
    sync::{Arc, RwLock},
};

use crate::{
    db::testing::{
        FailureInjector, FailurePoint, InMemoryDbError, InMemoryHarness,
        read_lock, write_lock,
    },
    fs::{BaseName, FileFormat, NormalizedPath},
    vault::{
        error::VaultRepositoryError,
        model::{DirId, DirView, FileId, FileView, FsEntryView},
        repository::{ReadRepository, WriteRepository},
    },
};

type FileIdList = Vec<FileId>;
type ByBasename = Arc<RwLock<HashMap<String, FileIdList>>>;
type ByParent = Arc<RwLock<HashMap<DirId, FileIdList>>>;
type ByFormat = Arc<RwLock<HashMap<FileFormat, FileIdList>>>;

/// In-memory repository that implements both [`ReadRepository`] and
/// [`WriteRepository`] using `HashMap`-backed storage.
///
/// Designed as a test double for vault persistence — fast, deterministic,
/// and instrumented with failure injection and operation counting via its
/// [`InMemoryHarness`].
///
/// # Index Maintenance
///
/// File views maintain five storage locations in lockstep:
/// - Primary file table (by [`FileId`])
/// - Path-to-ID index (by [`NormalizedPath`])
/// - Basename multimap (by basename)
/// - Parent multimap (by [`DirId`])
/// - Format multimap (by [`FileFormat`])
///
/// Directory views maintain two:
/// - Primary directory table (by [`DirId`])
/// - Path-to-ID index (by [`NormalizedPath`])
///
/// # Threading
///
/// `Clone` is cheap — all state is `Arc`-wrapped. Multiple references to the
/// same repository share the same underlying maps and harness, enabling shared
/// state verification patterns in tests.
///
/// [`ReadRepository`]: crate::vault::repository::ReadRepository
/// [`WriteRepository`]: crate::vault::repository::WriteRepository
#[derive(Debug, Clone)]
pub(crate) struct InMemoryRepository {
    harness: Arc<InMemoryHarness>,
    file_views: Arc<RwLock<HashMap<FileId, FileView>>>,
    dir_views: Arc<RwLock<HashMap<DirId, DirView>>>,
    file_path_to_id: Arc<RwLock<HashMap<NormalizedPath, FileId>>>,
    dir_path_to_id: Arc<RwLock<HashMap<NormalizedPath, DirId>>>,
    files_by_basename: ByBasename,
    files_by_parent: ByParent,
    files_by_format: ByFormat,
}

impl InMemoryRepository {
    /// Creates a new empty repository with a default [`InMemoryHarness`].
    ///
    /// The harness starts with no failure injector and zeroed counters.
    #[must_use]
    pub(crate) fn new() -> Self {
        Self::with_harness(InMemoryHarness::new())
    }

    /// Creates a new repository with a pre-configured harness.
    ///
    /// Use this when you need custom failure injection, e.g.:
    /// ```rust,ignore
    /// let harness = InMemoryHarness::with_injector(Box::new(WriteFailInjector));
    /// let repo = InMemoryRepository::with_harness(harness);
    /// ```
    #[must_use]
    pub(crate) fn with_harness(harness: InMemoryHarness) -> Self {
        Self {
            harness: Arc::new(harness),
            file_views: Arc::new(RwLock::new(HashMap::new())),
            dir_views: Arc::new(RwLock::new(HashMap::new())),
            file_path_to_id: Arc::new(RwLock::new(HashMap::new())),
            dir_path_to_id: Arc::new(RwLock::new(HashMap::new())),
            files_by_basename: Arc::new(RwLock::new(HashMap::new())),
            files_by_parent: Arc::new(RwLock::new(HashMap::new())),
            files_by_format: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Provides access to the underlying harness for test assertions.
    ///
    /// Use this to inspect operation counters or configure failure injection
    /// between calls:
    /// ```rust,ignore
    /// let snapshot = repo.harness().counters().snapshot();
    /// assert_eq!(snapshot.writes, 1);
    /// ```
    #[must_use]
    pub(crate) fn harness(&self) -> &InMemoryHarness {
        &self.harness
    }

    // Removes a file from all multimap indexes (basename, parent, format).
    //
    // Called before inserting an updated view or after deleting a view so that
    // stale index entries don't persist across overwrites. Each index is
    // checked independently — if a file wasn't in an index (e.g. no parent),
    // that lock is simply skipped.
    fn remove_file_from_indexes(
        &self,
        file_id: FileId,
        prior: &FileView,
    ) -> Result<(), VaultRepositoryError> {
        if let Ok(base) = BaseName::try_from(prior.name().clone()) {
            let mut by_basename =
                write_lock(&self.files_by_basename, "remove basename")?;
            if let Some(ids) = by_basename.get_mut(base.as_str()) {
                ids.retain(|id| *id != file_id);
            }
        }

        if let Some(parent) = prior.parent_id() {
            let mut by_parent =
                write_lock(&self.files_by_parent, "remove parent")?;
            if let Some(ids) = by_parent.get_mut(&parent) {
                ids.retain(|id| *id != file_id);
            }
        }

        let mut by_format = write_lock(&self.files_by_format, "remove format")?;
        if let Some(ids) = by_format.get_mut(&prior.format()) {
            ids.retain(|id| *id != file_id);
        }
        Ok(())
    }
}

/// Defaults to an empty repository with a clean [`InMemoryHarness`].
impl Default for InMemoryRepository {
    #[inline]
    fn default() -> Self {
        Self::new()
    }
}

impl ReadRepository for InMemoryRepository {
    #[inline]
    fn get_file_view(
        &self,
        id: FileId,
    ) -> Result<Option<FileView>, VaultRepositoryError> {
        self.harness.fail_at(FailurePoint::BeforeRead)?;
        self.harness.counters().inc_read();
        let map = read_lock(&self.file_views, "get_file_view")?;
        Ok(map.get(&id).cloned())
    }

    #[inline]
    fn get_dir_view(
        &self,
        id: DirId,
    ) -> Result<Option<DirView>, VaultRepositoryError> {
        self.harness.fail_at(FailurePoint::BeforeRead)?;
        self.harness.counters().inc_read();
        let map = read_lock(&self.dir_views, "get_dir_view")?;
        Ok(map.get(&id).cloned())
    }

    #[inline]
    fn find_file_view_by_path(
        &self,
        path: &NormalizedPath,
    ) -> Result<Option<FileView>, VaultRepositoryError> {
        self.harness.fail_at(FailurePoint::BeforeRead)?;
        self.harness.counters().inc_read();
        let path_map = read_lock(&self.file_path_to_id, "find_file_by_path")?;
        let Some(id) = path_map.get(path).copied() else {
            return Ok(None);
        };
        let files = read_lock(&self.file_views, "find_file_by_path views")?;
        Ok(files.get(&id).cloned())
    }

    #[inline]
    fn find_dir_view_by_path(
        &self,
        path: &NormalizedPath,
    ) -> Result<Option<DirView>, VaultRepositoryError> {
        self.harness.fail_at(FailurePoint::BeforeRead)?;
        self.harness.counters().inc_read();
        let path_map = read_lock(&self.dir_path_to_id, "find_dir_by_path")?;
        let Some(id) = path_map.get(path).copied() else {
            return Ok(None);
        };
        let dirs = read_lock(&self.dir_views, "find_dir_by_path views")?;
        Ok(dirs.get(&id).cloned())
    }

    #[inline]
    fn get_entry(
        &self,
        path: &NormalizedPath,
    ) -> Result<Option<FsEntryView>, VaultRepositoryError> {
        if let Some(file) = self.find_file_view_by_path(path)? {
            return Ok(Some(FsEntryView::File(file)));
        }
        if let Some(dir) = self.find_dir_view_by_path(path)? {
            return Ok(Some(FsEntryView::Dir(dir)));
        }
        Ok(None)
    }

    #[inline]
    fn find_file_views_by_basename(
        &self,
        basename: &str,
    ) -> Result<Vec<FileView>, VaultRepositoryError> {
        self.harness.fail_at(FailurePoint::BeforeRead)?;
        self.harness.counters().inc_read();
        let by_basename = read_lock(&self.files_by_basename, "by_basename")?;
        let ids = by_basename.get(basename).cloned().unwrap_or_default();
        let files = read_lock(&self.file_views, "by_basename views")?;
        Ok(ids.into_iter().filter_map(|id| files.get(&id).cloned()).collect())
    }

    #[inline]
    fn find_file_views_by_parent(
        &self,
        parent_id: DirId,
    ) -> Result<Vec<FileView>, VaultRepositoryError> {
        self.harness.fail_at(FailurePoint::BeforeRead)?;
        self.harness.counters().inc_read();
        let by_parent = read_lock(&self.files_by_parent, "by_parent")?;
        let ids = by_parent.get(&parent_id).cloned().unwrap_or_default();
        let files = read_lock(&self.file_views, "by_parent views")?;
        Ok(ids.into_iter().filter_map(|id| files.get(&id).cloned()).collect())
    }

    #[inline]
    fn list_file_views_by_format(
        &self,
        format: FileFormat,
    ) -> Result<Vec<FileView>, VaultRepositoryError> {
        self.harness.fail_at(FailurePoint::BeforeRead)?;
        self.harness.counters().inc_read();
        let by_format = read_lock(&self.files_by_format, "by_format")?;
        let ids = by_format.get(&format).cloned().unwrap_or_default();
        let files = read_lock(&self.file_views, "by_format views")?;
        Ok(ids.into_iter().filter_map(|id| files.get(&id).cloned()).collect())
    }

    #[inline]
    fn list_markdown_file_views(
        &self,
    ) -> Result<Vec<FileView>, VaultRepositoryError> {
        self.list_file_views_by_format(FileFormat::Markdown)
    }

    #[inline]
    fn list_file_views(&self) -> Result<Vec<FileView>, VaultRepositoryError> {
        self.harness.fail_at(FailurePoint::BeforeRead)?;
        self.harness.counters().inc_read();
        let files = read_lock(&self.file_views, "list_file_views")?;
        Ok(files.values().cloned().collect())
    }

    #[inline]
    fn list_file_paths(
        &self,
    ) -> Result<Vec<NormalizedPath>, VaultRepositoryError> {
        self.harness.fail_at(FailurePoint::BeforeRead)?;
        self.harness.counters().inc_read();
        let map = read_lock(&self.file_path_to_id, "list_file_paths")?;
        Ok(map.keys().cloned().collect())
    }

    #[inline]
    fn list_dir_views(&self) -> Result<Vec<DirView>, VaultRepositoryError> {
        self.harness.fail_at(FailurePoint::BeforeRead)?;
        self.harness.counters().inc_read();
        let dirs = read_lock(&self.dir_views, "list_dir_views")?;
        Ok(dirs.values().cloned().collect())
    }

    #[inline]
    fn list_dir_paths(
        &self,
    ) -> Result<Vec<NormalizedPath>, VaultRepositoryError> {
        self.harness.fail_at(FailurePoint::BeforeRead)?;
        self.harness.counters().inc_read();
        let map = read_lock(&self.dir_path_to_id, "list_dir_paths")?;
        Ok(map.keys().cloned().collect())
    }
}

impl WriteRepository for InMemoryRepository {
    #[inline]
    fn save_file_view(
        &self,
        path: &NormalizedPath,
        file: &FileView,
    ) -> Result<(), VaultRepositoryError> {
        self.harness.fail_at(FailurePoint::BeforeWrite)?;
        self.harness.counters().inc_write();

        let mut files = write_lock(&self.file_views, "save file views")?;
        if let Some(prior) = files.insert(file.id(), file.clone()) {
            self.remove_file_from_indexes(file.id(), &prior)?;
        }

        let mut path_map = write_lock(&self.file_path_to_id, "save file path")?;
        path_map.retain(|_, id| *id != file.id());
        path_map.insert(path.clone(), file.id());

        if let Ok(base) = BaseName::try_from(file.name().clone()) {
            let mut by_basename =
                write_lock(&self.files_by_basename, "save basename")?;
            let ids = by_basename.entry(base.as_str().to_owned()).or_default();
            if !ids.contains(&file.id()) {
                ids.push(file.id());
            }
        }

        if let Some(parent) = file.parent_id() {
            let mut by_parent =
                write_lock(&self.files_by_parent, "save parent")?;
            let ids = by_parent.entry(parent).or_default();
            if !ids.contains(&file.id()) {
                ids.push(file.id());
            }
        }

        let mut by_format = write_lock(&self.files_by_format, "save format")?;
        let ids = by_format.entry(file.format()).or_default();
        if !ids.contains(&file.id()) {
            ids.push(file.id());
        }
        Ok(())
    }

    #[inline]
    fn save_dir_view(
        &self,
        path: &NormalizedPath,
        dir: &DirView,
    ) -> Result<(), VaultRepositoryError> {
        self.harness.fail_at(FailurePoint::BeforeWrite)?;
        self.harness.counters().inc_write();
        let mut dirs = write_lock(&self.dir_views, "save dir views")?;
        dirs.insert(dir.id(), dir.clone());
        let mut path_map = write_lock(&self.dir_path_to_id, "save dir path")?;
        path_map.retain(|_, id| *id != dir.id());
        path_map.insert(path.clone(), dir.id());
        Ok(())
    }

    #[inline]
    fn delete_file_view(&self, id: FileId) -> Result<(), VaultRepositoryError> {
        self.harness.fail_at(FailurePoint::BeforeWrite)?;
        self.harness.counters().inc_write();
        let mut files = write_lock(&self.file_views, "delete file views")?;
        if let Some(prior) = files.remove(&id) {
            self.remove_file_from_indexes(id, &prior)?;
        }
        let mut paths = write_lock(&self.file_path_to_id, "delete file path")?;
        paths.retain(|_, fid| *fid != id);
        Ok(())
    }

    #[inline]
    fn delete_dir_view(&self, id: DirId) -> Result<(), VaultRepositoryError> {
        self.harness.fail_at(FailurePoint::BeforeWrite)?;
        self.harness.counters().inc_write();
        let mut dirs = write_lock(&self.dir_views, "delete dir views")?;
        dirs.remove(&id);
        let mut paths = write_lock(&self.dir_path_to_id, "delete dir path")?;
        paths.retain(|_, did| *did != id);
        Ok(())
    }

    #[inline]
    fn save_many_file_views(
        &self,
        entries: &[(NormalizedPath, FileView)],
    ) -> Result<(), VaultRepositoryError> {
        self.harness.fail_at(FailurePoint::BeforeWrite)?;
        self.harness.counters().inc_batch();
        for (path, file) in entries {
            self.save_file_view(path, file)?;
        }
        Ok(())
    }

    #[inline]
    fn save_many_dir_views(
        &self,
        entries: &[(NormalizedPath, DirView)],
    ) -> Result<(), VaultRepositoryError> {
        self.harness.fail_at(FailurePoint::BeforeWrite)?;
        self.harness.counters().inc_batch();
        for (path, dir) in entries {
            self.save_dir_view(path, dir)?;
        }
        Ok(())
    }

    #[inline]
    fn delete_many_file_views(
        &self,
        ids: &[FileId],
    ) -> Result<(), VaultRepositoryError> {
        self.harness.fail_at(FailurePoint::BeforeWrite)?;
        self.harness.counters().inc_batch();
        for id in ids {
            self.delete_file_view(*id)?;
        }
        Ok(())
    }

    #[inline]
    fn delete_many_dir_views(
        &self,
        ids: &[DirId],
    ) -> Result<(), VaultRepositoryError> {
        self.harness.fail_at(FailurePoint::BeforeWrite)?;
        self.harness.counters().inc_batch();
        for id in ids {
            self.delete_dir_view(*id)?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{db::testing::InMemoryHarness, fs::FsTimes};

    /// Shared fixtures for vault storage tests.
    mod fixtures {
        use super::*;

        /// Injects failures at `BeforeWrite` to test write-error pathways.
        pub(crate) struct WriteFailInjector;

        impl FailureInjector for WriteFailInjector {
            fn fail_at(
                &self,
                point: FailurePoint,
            ) -> Result<(), InMemoryDbError> {
                match point {
                    FailurePoint::BeforeWrite => {
                        Err(InMemoryDbError::InjectedFailure {
                            point,
                            reason: "write fail".into(),
                        })
                    }
                    _ => Ok(()),
                }
            }
        }

        /// Injects failures at `BeforeRead` to test read-error pathways.
        pub(crate) struct ReadFailInjector;

        impl FailureInjector for ReadFailInjector {
            fn fail_at(
                &self,
                point: FailurePoint,
            ) -> Result<(), InMemoryDbError> {
                match point {
                    FailurePoint::BeforeRead => {
                        Err(InMemoryDbError::InjectedFailure {
                            point,
                            reason: "read fail".into(),
                        })
                    }
                    _ => Ok(()),
                }
            }
        }

        /// Creates a minimal markdown file view at the given path.
        ///
        /// The returned tuple is suitable for passing to `save_file_view`.
        /// The file has no parent, a fixed content hash of `[1u8; 32]`, and
        /// a metadata size of 64 bytes.
        pub(crate) fn sample_file(name: &str) -> (NormalizedPath, FileView) {
            let id = FileId::new();
            (
                NormalizedPath::try_new(name).unwrap(),
                FileView::new(
                    id,
                    None,
                    crate::fs::FileName::new(name.into()),
                    FileFormat::Markdown,
                    crate::fs::FileMetadata::new(
                        crate::fs::FsTimes::new(None, None),
                        64,
                        false,
                    ),
                    [1u8; 32],
                ),
            )
        }

        /// Creates a minimal directory view with the given name.
        ///
        /// The returned tuple is suitable for passing to `save_dir_view`.
        /// The directory has no parent and default metadata.
        pub(crate) fn sample_dir(name: &str) -> (NormalizedPath, DirView) {
            let id = DirId::new();
            (
                NormalizedPath::try_new(name).unwrap(),
                DirView::new(
                    id,
                    None,
                    crate::fs::DirName::new(name.into()),
                    crate::fs::DirMetadata::new(
                        crate::fs::FsTimes::new(None, None),
                        false,
                    ),
                ),
            )
        }
    }

    /// Tests that a freshly created repository is empty.
    mod defaults {
        use super::*;

        #[test]
        fn returns_empty_collections_by_default() {
            let repo = InMemoryRepository::new();
            assert!(repo.list_file_views().unwrap().is_empty());
            assert!(repo.list_dir_views().unwrap().is_empty());
            assert!(repo.list_file_paths().unwrap().is_empty());
            assert!(repo.list_dir_paths().unwrap().is_empty());
        }
    }

    /// Tests for direct and path-based lookups.
    mod lookup {
        use super::*;

        #[test]
        fn get_file_view_returns_stored_file() {
            let repo = InMemoryRepository::new();
            let (path, file) = fixtures::sample_file("test.md");
            repo.save_file_view(&path, &file).unwrap();

            let found = repo.get_file_view(file.id()).unwrap().unwrap();
            assert_eq!(found.id(), file.id());
        }

        #[test]
        fn get_dir_view_returns_stored_dir() {
            let repo = InMemoryRepository::new();
            let (path, dir) = fixtures::sample_dir("notes");
            repo.save_dir_view(&path, &dir).unwrap();

            let found = repo.get_dir_view(dir.id()).unwrap().unwrap();
            assert_eq!(found.id(), dir.id());
        }

        #[test]
        fn find_file_view_by_path_returns_stored_file() {
            let repo = InMemoryRepository::new();
            let (path, file) = fixtures::sample_file("test.md");
            repo.save_file_view(&path, &file).unwrap();

            let found = repo.find_file_view_by_path(&path).unwrap().unwrap();
            assert_eq!(found.id(), file.id());
        }

        #[test]
        fn find_dir_view_by_path_returns_stored_dir() {
            let repo = InMemoryRepository::new();
            let (path, dir) = fixtures::sample_dir("notes");
            repo.save_dir_view(&path, &dir).unwrap();

            let found = repo.find_dir_view_by_path(&path).unwrap().unwrap();
            assert_eq!(found.id(), dir.id());
        }

        #[test]
        fn get_entry_returns_file_or_dir() {
            let repo = InMemoryRepository::new();
            let (f_path, file) = fixtures::sample_file("file.md");
            let (d_path, dir) = fixtures::sample_dir("dir");

            repo.save_file_view(&f_path, &file).unwrap();
            repo.save_dir_view(&d_path, &dir).unwrap();

            let f_entry = repo.get_entry(&f_path).unwrap().unwrap();
            assert!(f_entry.is_file());

            let d_entry = repo.get_entry(&d_path).unwrap().unwrap();
            assert!(d_entry.is_dir());
        }

        #[test]
        fn get_entry_returns_none_for_missing_path() {
            let repo = InMemoryRepository::new();
            let path = NormalizedPath::try_new("missing").unwrap();
            let entry = repo.get_entry(&path).unwrap();
            assert!(entry.is_none());
        }
    }

    /// Tests for table and index scan operations.
    mod list {
        use super::*;

        #[test]
        fn list_methods_return_all_stored_entries() {
            let repo = InMemoryRepository::new();
            let (p1, f1) = fixtures::sample_file("a.md");
            let (p2, f2) = fixtures::sample_file("b.md");
            let (p3, d1) = fixtures::sample_dir("dir1");

            repo.save_file_view(&p1, &f1).unwrap();
            repo.save_file_view(&p2, &f2).unwrap();
            repo.save_dir_view(&p3, &d1).unwrap();

            assert_eq!(repo.list_file_views().unwrap().len(), 2);
            assert_eq!(repo.list_file_paths().unwrap().len(), 2);
            assert_eq!(repo.list_dir_views().unwrap().len(), 1);
            assert_eq!(repo.list_dir_paths().unwrap().len(), 1);
        }
    }

    /// Tests for multimap index queries (basename, parent, format).
    mod indexes {
        use super::*;

        #[test]
        fn find_file_views_by_basename_returns_matches() {
            let repo = InMemoryRepository::new();
            let (p1, f1) = fixtures::sample_file("shared.md");
            let (p2, f2) = fixtures::sample_file("other/shared.md");

            repo.save_file_view(&p1, &f1).unwrap();
            repo.save_file_view(&p2, &f2).unwrap();

            let matches = repo.find_file_views_by_basename("shared").unwrap();
            assert_eq!(matches.len(), 2);
        }

        #[test]
        fn find_file_views_by_basename_returns_empty_when_no_match() {
            let repo = InMemoryRepository::new();
            let matches = repo.find_file_views_by_basename("missing").unwrap();
            assert!(matches.is_empty());
        }

        #[test]
        fn find_file_views_by_parent_returns_children() {
            let repo = InMemoryRepository::new();
            let parent_id = DirId::new();
            let id = FileId::new();
            let path = NormalizedPath::try_new("child.md").unwrap();
            let file = FileView::new(
                id,
                Some(parent_id),
                crate::fs::FileName::new("child.md".into()),
                FileFormat::Markdown,
                crate::fs::FileMetadata::new(
                    FsTimes::new(None, None),
                    0,
                    false,
                ),
                [0u8; 32],
            );

            repo.save_file_view(&path, &file).unwrap();

            let children = repo.find_file_views_by_parent(parent_id).unwrap();
            assert_eq!(children.len(), 1);
            let first = children.first().unwrap();
            assert_eq!(first.id(), id);
        }

        #[test]
        fn find_file_views_by_parent_returns_empty_when_no_children() {
            let repo = InMemoryRepository::new();
            let children =
                repo.find_file_views_by_parent(DirId::new()).unwrap();
            assert!(children.is_empty());
        }

        #[test]
        fn list_file_views_by_format_filters_correctly() {
            let repo = InMemoryRepository::new();
            let (p1, f1) = fixtures::sample_file("a.md");
            let id2 = FileId::new();
            let p2 = NormalizedPath::try_new("b.json").unwrap();
            let f2 = FileView::new(
                id2,
                None,
                crate::fs::FileName::new("b.json".into()),
                FileFormat::Json,
                crate::fs::FileMetadata::new(
                    FsTimes::new(None, None),
                    0,
                    false,
                ),
                [0u8; 32],
            );

            repo.save_file_view(&p1, &f1).unwrap();
            repo.save_file_view(&p2, &f2).unwrap();

            assert_eq!(
                repo.list_file_views_by_format(FileFormat::Markdown)
                    .unwrap()
                    .len(),
                1
            );
            assert_eq!(repo.list_markdown_file_views().unwrap().len(), 1);
            assert_eq!(
                repo.list_file_views_by_format(FileFormat::Json).unwrap().len(),
                1
            );
        }

        #[test]
        fn list_file_views_by_format_returns_empty_when_no_match() {
            let repo = InMemoryRepository::new();
            let matches =
                repo.list_file_views_by_format(FileFormat::Image).unwrap();
            assert!(matches.is_empty());
        }
    }

    /// Tests for save and overwrite semantics.
    mod update {
        use super::*;

        #[test]
        fn save_file_view_overwrites_existing_and_cleans_indexes() {
            let repo = InMemoryRepository::new();
            let (path1, file1) = fixtures::sample_file("old.md");
            let id = file1.id();
            let path2 = NormalizedPath::try_new("new.md").unwrap();
            let file2 = FileView::new(
                id,
                None,
                crate::fs::FileName::new("new.md".into()),
                FileFormat::Markdown,
                crate::fs::FileMetadata::new(
                    FsTimes::new(None, None),
                    0,
                    false,
                ),
                [0u8; 32],
            );

            repo.save_file_view(&path1, &file1).unwrap();
            repo.save_file_view(&path2, &file2).unwrap();

            assert!(repo.find_file_view_by_path(&path1).unwrap().is_none());
            assert!(repo.find_file_view_by_path(&path2).unwrap().is_some());
            assert!(
                repo.find_file_views_by_basename("old").unwrap().is_empty()
            );
            assert_eq!(
                repo.find_file_views_by_basename("new").unwrap().len(),
                1
            );
        }

        #[test]
        fn save_dir_view_overwrites_existing_and_cleans_indexes() {
            let repo = InMemoryRepository::new();
            let (path1, dir1) = fixtures::sample_dir("old_dir");
            let id = dir1.id();
            let (path2, dir2) = (
                NormalizedPath::try_new("new_dir").unwrap(),
                DirView::new(
                    id,
                    None,
                    crate::fs::DirName::new("new_dir".into()),
                    crate::fs::DirMetadata::new(
                        FsTimes::new(None, None),
                        false,
                    ),
                ),
            );

            repo.save_dir_view(&path1, &dir1).unwrap();
            repo.save_dir_view(&path2, &dir2).unwrap();

            assert!(repo.find_dir_view_by_path(&path1).unwrap().is_none());
            assert!(repo.find_dir_view_by_path(&path2).unwrap().is_some());
        }

        #[test]
        fn save_many_methods_persist_multiple_entries() {
            let repo = InMemoryRepository::new();
            let (p1, f1) = fixtures::sample_file("f1.md");
            let (p2, f2) = fixtures::sample_file("f2.md");
            let (p3, d1) = fixtures::sample_dir("d1");
            let (p4, d2) = fixtures::sample_dir("d2");

            repo.save_many_file_views(&[(p1, f1), (p2, f2)]).unwrap();
            repo.save_many_dir_views(&[(p3, d1), (p4, d2)]).unwrap();

            assert_eq!(repo.list_file_views().unwrap().len(), 2);
            assert_eq!(repo.list_dir_views().unwrap().len(), 2);
        }
    }

    /// Tests for delete operations and idempotency.
    mod delete {
        use super::*;

        #[test]
        fn delete_methods_remove_entries_and_indexes() {
            let repo = InMemoryRepository::new();
            let (p1, f1) = fixtures::sample_file("test.md");
            let (p2, d1) = fixtures::sample_dir("notes");

            repo.save_file_view(&p1, &f1).unwrap();
            repo.save_dir_view(&p2, &d1).unwrap();

            repo.delete_file_view(f1.id()).unwrap();
            repo.delete_dir_view(d1.id()).unwrap();

            assert!(repo.get_file_view(f1.id()).unwrap().is_none());
            assert!(repo.get_dir_view(d1.id()).unwrap().is_none());
            assert!(repo.find_file_view_by_path(&p1).unwrap().is_none());
            assert!(repo.find_dir_view_by_path(&p2).unwrap().is_none());
        }

        #[test]
        fn delete_methods_are_idempotent_for_missing_ids() {
            let repo = InMemoryRepository::new();
            assert!(repo.delete_file_view(FileId::new()).is_ok());
            assert!(repo.delete_dir_view(DirId::new()).is_ok());
        }

        #[test]
        fn delete_many_methods_remove_multiple_entries() {
            let repo = InMemoryRepository::new();
            let (p1, f1) = fixtures::sample_file("f1.md");
            let (p2, f2) = fixtures::sample_file("f2.md");

            repo.save_many_file_views(&[(p1, f1.clone()), (p2, f2.clone())])
                .unwrap();
            repo.delete_many_file_views(&[f1.id(), f2.id()]).unwrap();

            assert!(repo.list_file_views().unwrap().is_empty());
        }
    }

    /// Tests for operation counter accuracy.
    mod counters {
        use super::*;

        #[test]
        fn operations_increment_counters() {
            let repo = InMemoryRepository::new();
            let (path, file) = fixtures::sample_file("test.md");

            repo.save_file_view(&path, &file).unwrap();
            let _ = repo.get_file_view(file.id()).unwrap();
            repo.save_many_file_views(&[]).unwrap();

            let snapshot = repo.harness().counters().snapshot();
            assert_eq!(snapshot.writes, 1);
            assert_eq!(snapshot.reads, 1);
            assert_eq!(snapshot.batches, 1);
        }
    }

    /// Tests for failure injection at read and write points.
    mod injection {
        use super::*;

        #[test]
        fn read_failure_injection_returns_error() {
            let harness = InMemoryHarness::with_injector(Box::new(
                fixtures::ReadFailInjector,
            ));
            let repo = InMemoryRepository::with_harness(harness);

            let result = repo.get_file_view(FileId::new());
            assert!(matches!(result, Err(VaultRepositoryError::Storage(_))));
        }

        #[test]
        fn write_failure_injection_returns_error() {
            let harness = InMemoryHarness::with_injector(Box::new(
                fixtures::WriteFailInjector,
            ));
            let repo = InMemoryRepository::with_harness(harness);
            let (path, file) = fixtures::sample_file("test.md");

            let result = repo.save_file_view(&path, &file);
            assert!(matches!(result, Err(VaultRepositoryError::Storage(_))));
            let _ = repo.harness().counters().snapshot();
        }
    }
}
