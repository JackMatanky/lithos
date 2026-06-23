//! Integration tests for the indexing pipeline wiring.
//!
//! This module verifies that the `trace_app` composition root successfully
//! orchestrates the concrete `WalkdirAdapter`, `RedbRepository`, and
//! `IndexerService` by running actual filesystem traversals against
//! temporary test vaults.

use std::fs;

use pretty_assertions::assert_eq;
use tempfile::tempdir;
use trace_app::index::{IndexCommand, run_index};
use trace_fs::DirPath;
use trace_indexer::{IndexOptions, IndexScope, ScanFilters};

#[test]
fn run_index_with_temp_vault_returns_correct_counts() {
    let test_db = trace_db::testing::TestDb::new().unwrap();
    let cache_tmp = test_db.dir_path();
    let vault_tmp = tempdir().unwrap();

    // Create some files
    let file1 = vault_tmp.path().join("file1.md");
    let dir1 = vault_tmp.path().join("dir1");
    let file2 = dir1.join("file2.md");

    fs::write(&file1, "content1").unwrap();
    fs::create_dir(&dir1).unwrap();
    fs::write(&file2, "content2").unwrap();

    let root = DirPath::try_from(vault_tmp.path().to_path_buf()).unwrap();
    let cache_dir = DirPath::try_from(cache_tmp.to_path_buf()).unwrap();

    let scope = IndexScope::Full {
        root: root.clone(),
        filters: ScanFilters::default(),
    };
    let cmd = IndexCommand::new(scope, IndexOptions::default());

    let result =
        run_index(&root, &cache_dir, &cmd).expect("run_index should succeed");

    let indexed = result.indexed();
    // vault root is not indexed, so only dir1 is indexed = 1 dir
    assert_eq!(indexed.dirs().len(), 1);
    // file1 + file2 = 2 files
    assert_eq!(indexed.files().len(), 2);

    assert_eq!(result.report().new_count(), 3);
    assert_eq!(result.report().scanned(), 3);
}

#[test]
fn run_index_handles_empty_vault() {
    let test_db = trace_db::testing::TestDb::new().unwrap();
    let cache_tmp = test_db.dir_path();
    let vault_tmp = tempdir().unwrap();

    let root = DirPath::try_from(vault_tmp.path().to_path_buf()).unwrap();
    let cache_dir = DirPath::try_from(cache_tmp.to_path_buf()).unwrap();

    let scope = IndexScope::Full {
        root: root.clone(),
        filters: ScanFilters::default(),
    };
    let cmd = IndexCommand::new(scope, IndexOptions::default());

    let result =
        run_index(&root, &cache_dir, &cmd).expect("run_index should succeed");

    let indexed = result.indexed();
    // vault root is not indexed
    assert_eq!(indexed.dirs().len(), 0);
    assert_eq!(indexed.files().len(), 0);
}
