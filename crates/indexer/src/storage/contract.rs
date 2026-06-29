//! Shared `Repository` contract test (test-only).
//!
//! A single set of behavioural assertions exercised against **every**
//! `Repository` adapter (`RedbRepository`, `InMemoryRepository`). Parity used
//! to be asserted by reading the two implementations side by side; this module
//! turns "the adapters diverged" from a review finding into a test failure.

#![cfg(test)]

use std::time::SystemTime;

use pretty_assertions::assert_eq;
use traces_fs::{
    FileFormat,
    metadata::{DirMetadata, FileMetadata, FsTimes},
    name::{DirName, FileName},
    path::PathKey,
};

use crate::{
    error::IndexerRepositoryError,
    model::{DirRecord, FileRecord, FsParentId, FsRecordId},
    repository::Repository,
};

fn file_at(id: FsRecordId, path: &PathKey) -> FileRecord {
    FileRecord::new(
        id,
        FsParentId::Root,
        path.clone(),
        FileName::new("file.txt".into()),
        FileFormat::Markdown,
        FileMetadata::new(FsTimes::new(None, None), 0, false),
        SystemTime::now(),
    )
}

fn dir_at(id: FsRecordId, path: &PathKey) -> DirRecord {
    DirRecord::new(
        id,
        FsParentId::Root,
        path.clone(),
        DirName::new("dir".into()),
        DirMetadata::new(FsTimes::new(None, None), false),
        SystemTime::now(),
    )
}

/// Asserts the behavioural contract every `Repository` adapter must satisfy.
///
/// Each adapter under test starts empty. The checks run in sequence on a
/// single repository instance:
///
/// 1. `all_paths` deduplicates a path stored in both the file and dir tables.
/// 2. `save_file`/`save_dir` with a *different* id at an existing path is
///    rejected with [`IndexerRepositoryError::DuplicatePath`].
/// 3. Re-saving the *same* id at the same path is a legitimate update.
/// 4. `save_many_records` rejects the whole batch on a single conflict, with no
///    partial writes.
pub(crate) fn assert_repository_contract(repo: &impl Repository) {
    assert_all_paths_deduplicates(repo);
    assert_duplicate_path_rejected(repo);
    assert_same_id_is_update(repo);
    assert_batch_rejects_whole_on_conflict(repo);
}

fn assert_all_paths_deduplicates(repo: &impl Repository) {
    let shared = PathKey::try_new("contract/shared").unwrap();
    repo.save_file(&file_at(FsRecordId::new(), &shared)).unwrap();
    repo.save_dir(&dir_at(FsRecordId::new(), &shared)).unwrap();

    let paths = repo.all_paths().unwrap();
    let occurrences = paths.iter().filter(|p| **p == shared).count();
    assert_eq!(occurrences, 1, "all_paths must not return the same path twice");
}

fn assert_duplicate_path_rejected(repo: &impl Repository) {
    let path = PathKey::try_new("contract/dup-file").unwrap();
    repo.save_file(&file_at(FsRecordId::new(), &path)).unwrap();
    let err = repo.save_file(&file_at(FsRecordId::new(), &path)).unwrap_err();
    assert!(
        matches!(err, IndexerRepositoryError::DuplicatePath(p) if p == path),
        "a different id at an existing file path must be rejected"
    );
    let dir_path = PathKey::try_new("contract/dup-dir").unwrap();
    repo.save_dir(&dir_at(FsRecordId::new(), &dir_path)).unwrap();
    let dir_err =
        repo.save_dir(&dir_at(FsRecordId::new(), &dir_path)).unwrap_err();
    assert!(
        matches!(dir_err, IndexerRepositoryError::DuplicatePath(p) if p == dir_path),
        "a different id at an existing dir path must be rejected"
    );
}

fn assert_same_id_is_update(repo: &impl Repository) {
    let id = FsRecordId::new();
    let path = PathKey::try_new("contract/same-id").unwrap();
    repo.save_file(&file_at(id, &path)).unwrap();
    repo.save_file(&file_at(id, &path))
        .expect("re-saving the same id at the same path is an update");
    assert!(repo.find_file_by_path(&path).unwrap().is_some());
}

fn assert_batch_rejects_whole_on_conflict(repo: &impl Repository) {
    let taken = PathKey::try_new("contract/batch-taken").unwrap();
    repo.save_file(&file_at(FsRecordId::new(), &taken)).unwrap();

    // One fresh path plus a conflict on `taken` (different id).
    let fresh = PathKey::try_new("contract/batch-fresh").unwrap();
    let files = [
        file_at(FsRecordId::new(), &fresh),
        file_at(FsRecordId::new(), &taken),
    ];
    let err = repo.save_many_records(&files, &[]).unwrap_err();
    assert!(
        matches!(err, IndexerRepositoryError::DuplicatePath(p) if p == taken),
        "batch must be rejected on a single conflict"
    );
    assert!(
        repo.find_file_by_path(&fresh).unwrap().is_none(),
        "a rejected batch must not write the non-conflicting record"
    );
}
