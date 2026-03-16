//! Integration tests for schema incremental resolution.
//!
//! Validates that the Loader correctly uses incremental resolution for existing
//! schemas when only property bank changes, and full resolution for new schemas
//! or schemas with file changes.

#![expect(
    clippy::panic_in_result_fn,
    reason = "Integration tests use assertions which panic on failure."
)]
#![expect(
    clippy::tests_outside_test_module,
    reason = "Integration tests are top-level by default."
)]

mod common;

use std::path::Path;

use common::*;
use lithos_core::{
    config::{
        aggregate::Config,
        raw::RawConfig,
        vault::{VaultId, VaultRoot},
    },
    fs::FsReader,
    schema::loader::Loader,
};
use tempfile::TempDir;

/// Write a file to the test directory.
fn write_file(root: &Path, relative: &str, content: &str) -> TestResult {
    let path = root.join(relative);
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    std::fs::write(&path, content)?;
    Ok(())
}

/// Create a test config for a vault root.
fn test_config(root: &Path) -> TestResult<Config> {
    let raw = RawConfig::default();
    let root = VaultRoot::try_new(root.to_path_buf())?;
    let config = Config::build(
        &raw,
        VaultId::new(),
        root,
        lithos_core::config::aggregate::Version::initial(),
    )?;
    Ok(config)
}

/// **INT-001**: New schema uses full resolution pipeline.
#[test]
fn new_schema_uses_full_resolution() -> TestResult {
    // GIVEN: Empty DB + new schema file
    let vault_dir = TempDir::new()?;
    let test_db = TestDb::new()?;

    write_file(
        vault_dir.path(),
        "schemas/property_bank.json",
        r#"{"$version": "1.0", "properties": {"title": {"type": "string"}}}"#,
    )?;
    write_file(
        vault_dir.path(),
        "schemas/task.json",
        r#"{"$version": "1.0", "properties": {"title": {"$ref": "property_bank#/title"}}}"#,
    )?;

    let config = test_config(vault_dir.path())?;
    let repository = setup_repository(test_db.db());
    let source = FsReader::new(vault_dir.path());
    let loader = Loader::new(repository, source, &config);

    // WHEN: Loading schemas
    let resolved = loader.load()?;

    // THEN: Schema is resolved via full pipeline
    assert_eq!(resolved.len(), 1, "Should resolve 1 new schema");
    let schema = resolved.first().expect("Should have at least one schema");
    assert_eq!(schema.name().as_ref(), "task");
    assert_eq!(schema.properties().len(), 1, "Should have 1 property");

    Ok(())
}

/// **INT-002**: Existing schema with file change uses full resolution.
#[test]
#[ignore = "redb file locking prevents database reopening in same process"]
fn existing_schema_file_change_uses_full_resolution() -> TestResult {
    let vault_dir = TempDir::new()?;
    let mut test_db = TestDb::new()?;

    write_file(
        vault_dir.path(),
        "schemas/property_bank.json",
        r#"{"$version": "1.0", "properties": {"title": {"type": "string"}}}"#,
    )?;
    write_file(
        vault_dir.path(),
        "schemas/task.json",
        r#"{"$version": "1.0", "properties": {"title": {"$ref": "property_bank#/title"}}}"#,
    )?;

    let config = test_config(vault_dir.path())?;

    // First load - use a scope to ensure everything drops
    let initial = {
        let repository = setup_repository(test_db.db());
        let source = FsReader::new(vault_dir.path());
        let loader = Loader::new(repository, source, &config);
        loader.load()?
    };

    assert_eq!(initial.len(), 1);

    // WHEN: File changes (add property)
    #[expect(
        clippy::disallowed_methods,
        reason = "Test needs filesystem timing"
    )]
    std::thread::sleep(std::time::Duration::from_millis(10));

    write_file(
        vault_dir.path(),
        "schemas/task.json",
        r#"{"$version": "1.0", "properties": {
            "title": {"$ref": "property_bank#/title"},
            "status": {"type": "bool"}
        }}"#,
    )?;

    // THEN: Full resolution updates schema
    // Reopen database to work around rkyv deserialization limitations
    let db2 = test_db.reopen()?; // This replaces the internal Arc, closing the old DB
    let repository2 = setup_repository(&db2);
    let source2 = FsReader::new(vault_dir.path());
    let loader2 = Loader::new(repository2, source2, &config);

    let updated = loader2.load()?;
    assert_eq!(updated.len(), 1, "Should resolve 1 updated schema");
    let schema = updated.first().expect("Should have at least one schema");
    assert_eq!(schema.properties().len(), 2, "Should have 2 properties");

    Ok(())
}

/// **INT-003**: Existing schema with only bank change uses incremental
/// resolution.
#[test]
#[ignore = "redb file locking prevents database reopening in same process"]
fn existing_schema_bank_change_uses_incremental() -> TestResult {
    let vault_dir = TempDir::new()?;
    let mut test_db = TestDb::new()?;

    write_file(
        vault_dir.path(),
        "schemas/property_bank.json",
        r#"{"$version": "1.0", "properties": {"status": {"type": "string"}}}"#,
    )?;
    write_file(
        vault_dir.path(),
        "schemas/task.json",
        r#"{"$version": "1.0", "properties": {"status": {"$ref": "property_bank#/status"}}}"#,
    )?;

    let config = test_config(vault_dir.path())?;
    let repository = setup_repository(test_db.db());
    let source = FsReader::new(vault_dir.path());
    let loader = Loader::new(repository, source, &config);

    // First load
    let initial = loader.load()?;
    assert_eq!(initial.len(), 1);

    // WHEN: Property bank changes (modify status property)
    #[expect(
        clippy::disallowed_methods,
        reason = "Test needs filesystem timing"
    )]
    std::thread::sleep(std::time::Duration::from_millis(10));

    write_file(
        vault_dir.path(),
        "schemas/property_bank.json",
        r#"{"$version": "1.0", "properties": {"status": {"type": "bool"}}}"#,
    )?;

    // THEN: Incremental resolution updates schema
    // Reopen database to work around rkyv deserialization limitations
    let db2 = test_db.reopen()?;
    let repository2 = setup_repository(&db2);
    let source2 = FsReader::new(vault_dir.path());
    let loader2 = Loader::new(repository2, source2, &config);

    let updated = loader2.load()?;
    assert_eq!(updated.len(), 1, "Should update 1 schema incrementally");

    Ok(())
}

/// **INT-004**: No resolution when property hash unchanged.
#[test]
#[ignore = "redb file locking prevents database reopening in same process"]
fn no_resolution_when_property_unchanged() -> TestResult {
    let vault_dir = TempDir::new()?;
    let mut test_db = TestDb::new()?;

    write_file(
        vault_dir.path(),
        "schemas/property_bank.json",
        r#"{"$version": "1.0", "properties": {"title": {"type": "string"}}}"#,
    )?;
    write_file(
        vault_dir.path(),
        "schemas/task.json",
        r#"{"$version": "1.0", "properties": {"title": {"$ref": "property_bank#/title"}}}"#,
    )?;

    let config = test_config(vault_dir.path())?;
    let repository = setup_repository(test_db.db());
    let source = FsReader::new(vault_dir.path());
    let loader = Loader::new(repository, source, &config);

    let initial = loader.load()?;
    assert_eq!(initial.len(), 1);

    // WHEN: Touch file without changing content hash
    #[expect(
        clippy::disallowed_methods,
        reason = "Test needs filesystem timing"
    )]
    std::thread::sleep(std::time::Duration::from_millis(10));

    // Rewrite same content
    write_file(
        vault_dir.path(),
        "schemas/property_bank.json",
        r#"{"$version": "1.0", "properties": {"title": {"type": "string"}}}"#,
    )?;

    // THEN: No schemas re-resolved (hash unchanged)
    // Reopen database to work around rkyv deserialization limitations
    let db2 = test_db.reopen()?;
    let repository2 = setup_repository(&db2);
    let source2 = FsReader::new(vault_dir.path());
    let loader2 = Loader::new(repository2, source2, &config);

    let updated = loader2.load()?;
    assert_eq!(updated.len(), 0, "Should not resolve when hash unchanged");

    Ok(())
}

/// **INT-005**: Mixed scenario - new, file-changed, and incremental.
#[test]
#[ignore = "redb file locking prevents database reopening in same process"]
fn mixed_scenario_handles_all_three_paths() -> TestResult {
    let vault_dir = TempDir::new()?;
    let mut test_db = TestDb::new()?;

    write_file(
        vault_dir.path(),
        "schemas/property_bank.json",
        r#"{"$version": "1.0", "properties": {
            "title": {"type": "string"},
            "status": {"type": "string"}
        }}"#,
    )?;
    write_file(
        vault_dir.path(),
        "schemas/task.json",
        r#"{"$version": "1.0", "properties": {"title": {"$ref": "property_bank#/title"}}}"#,
    )?;
    write_file(
        vault_dir.path(),
        "schemas/note.json",
        r#"{"$version": "1.0", "properties": {"title": {"$ref": "property_bank#/title"}}}"#,
    )?;

    let config = test_config(vault_dir.path())?;
    let repository = setup_repository(test_db.db());
    let source = FsReader::new(vault_dir.path());
    let loader = Loader::new(repository, source, &config);

    // First load: 2 new schemas
    let initial = loader.load()?;
    assert_eq!(initial.len(), 2);

    #[expect(
        clippy::disallowed_methods,
        reason = "Test needs filesystem timing"
    )]
    std::thread::sleep(std::time::Duration::from_millis(10));

    // WHEN: Mixed changes:
    // 1. Add new schema (project.json) - NEW path
    // 2. Modify task.json - FILE-CHANGED path
    // 3. Modify property bank title - affects note.json via INCREMENTAL path

    write_file(
        vault_dir.path(),
        "schemas/project.json",
        r#"{"$version": "1.0", "properties": {"title": {"$ref": "property_bank#/title"}}}"#,
    )?;

    write_file(
        vault_dir.path(),
        "schemas/task.json",
        r#"{"$version": "1.0", "properties": {
            "title": {"$ref": "property_bank#/title"},
            "done": {"type": "bool"}
        }}"#,
    )?;

    write_file(
        vault_dir.path(),
        "schemas/property_bank.json",
        r#"{"$version": "1.0", "properties": {
            "title": {"type": "string", "max_length": 100},
            "status": {"type": "string"}
        }}"#,
    )?;

    // THEN: All three paths exercised
    // Reopen database to work around rkyv deserialization limitations
    let db2 = test_db.reopen()?;
    let repository2 = setup_repository(&db2);
    let source2 = FsReader::new(vault_dir.path());
    let loader2 = Loader::new(repository2, source2, &config);

    let updated = loader2.load()?;
    assert!(
        updated.len() >= 2,
        "Should process at least project (new) and task (file-changed)"
    );

    // Verify we got the expected schemas
    let names: Vec<&str> = updated.iter().map(|s| s.name().as_ref()).collect();
    assert!(names.contains(&"project"), "Should include new project");
    assert!(names.contains(&"task"), "Should include file-changed task");

    Ok(())
}
