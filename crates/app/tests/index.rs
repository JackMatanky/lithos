//! Integration tests for the indexing pipeline wiring.
//!
//! This module verifies that the `traces_app` composition root successfully
//! orchestrates the concrete `WalkdirAdapter`, `RedbRepository`, and
//! `IndexerService` by running actual filesystem traversals against
//! temporary test vaults.

use std::fs;

use pretty_assertions::assert_eq;
use tempfile::tempdir;
use traces_app::index::{IndexCommand, run_index};
use traces_fs::DirPath;
use traces_indexer::{IndexOptions, IndexScope, ScanFilters};

fn setup_test_vault(vault_tmp: &tempfile::TempDir) {
    let file1 = vault_tmp.path().join("file1.md");
    let dir1 = vault_tmp.path().join("dir1");
    let file2 = dir1.join("file2.md");

    assert!(fs::write(&file1, "content1").is_ok());
    assert!(fs::create_dir(&dir1).is_ok());
    assert!(fs::write(&file2, "content2").is_ok());
}

#[test]
fn run_index_with_temp_vault_returns_correct_counts() {
    let test_db = traces_db::testing::TestStore::new().unwrap();
    let cache_tmp = test_db.dir_path();
    let vault_tmp = tempdir().unwrap();
    setup_test_vault(&vault_tmp);

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
    let test_db = traces_db::testing::TestStore::new().unwrap();
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

#[test]
fn rebuild_option_creates_all_new_nodes() {
    let test_db = traces_db::testing::TestStore::new().unwrap();
    let cache_tmp = test_db.dir_path();
    let vault_tmp = tempdir().unwrap();
    setup_test_vault(&vault_tmp);

    let root = DirPath::try_from(vault_tmp.path().to_path_buf()).unwrap();
    let cache_dir = DirPath::try_from(cache_tmp.to_path_buf()).unwrap();

    // First run
    let scope = IndexScope::Full {
        root: root.clone(),
        filters: ScanFilters::default(),
    };
    let cmd1 = IndexCommand::new(scope.clone(), IndexOptions::default());
    let _ = run_index(&root, &cache_dir, &cmd1).unwrap();

    // Second run with rebuild = true
    let opts = IndexOptions::new(true, false);
    let cmd2 = IndexCommand::new(scope, opts);
    let result = run_index(&root, &cache_dir, &cmd2).unwrap();

    // All should be "new" because of rebuild
    assert_eq!(result.report().new_count(), 3);
    assert_eq!(result.report().fresh_count(), 0);
    assert_eq!(result.report().scanned(), 3);
}

#[test]
fn partial_scope_restricts_scan() {
    let test_db = traces_db::testing::TestStore::new().unwrap();
    let cache_tmp = test_db.dir_path();
    let vault_tmp = tempdir().unwrap();
    setup_test_vault(&vault_tmp);

    let root = DirPath::try_from(vault_tmp.path().to_path_buf()).unwrap();
    let cache_dir = DirPath::try_from(cache_tmp.to_path_buf()).unwrap();

    let target_dir = DirPath::try_from(vault_tmp.path().join("dir1")).unwrap();
    let scope = IndexScope::Partial {
        root: target_dir,
        filters: ScanFilters::default(),
    };
    let cmd = IndexCommand::new(scope, IndexOptions::default());

    let result = run_index(&root, &cache_dir, &cmd).unwrap();

    // Only dir1 and file2.md should be indexed
    assert_eq!(result.report().scanned(), 1);
    assert_eq!(result.indexed().files().len(), 1);
}

#[test]
fn dry_run_performs_no_writes() {
    let test_db = traces_db::testing::TestStore::new().unwrap();
    let cache_tmp = test_db.dir_path();
    let vault_tmp = tempdir().unwrap();
    setup_test_vault(&vault_tmp);

    let root = DirPath::try_from(vault_tmp.path().to_path_buf()).unwrap();
    let cache_dir = DirPath::try_from(cache_tmp.to_path_buf()).unwrap();

    let scope = IndexScope::Full {
        root: root.clone(),
        filters: ScanFilters::default(),
    };
    let opts = IndexOptions::new(false, true);
    let cmd1 = IndexCommand::new(scope.clone(), opts);

    // First run with dry_run
    let result = run_index(&root, &cache_dir, &cmd1).unwrap();
    assert_eq!(result.report().new_count(), 3);

    // Second run without dry_run - since first was dry_run, all should STILL be
    // new
    let cmd2 = IndexCommand::new(scope, IndexOptions::default());
    let result2 = run_index(&root, &cache_dir, &cmd2).unwrap();
    assert_eq!(result2.report().new_count(), 3);
}
