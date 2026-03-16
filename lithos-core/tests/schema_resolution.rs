//! Integration tests for schema resolution pipeline.
//!
//! Tests the Loader's ability to resolve schemas from files, including:
//! - Reference expansion (`property_bank` refs)
//! - Inheritance resolution (parent schemas)
//! - Property merging and validation

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
    schema::{loader::Loader, storage::Repository as _},
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

// ========================================================================
//                       Reference Resolution Tests
// ========================================================================

/// Test that schemas with property bank references are resolved correctly.
#[test]
fn resolves_property_bank_references() -> TestResult {
    let vault_dir = TempDir::new()?;
    let test_db = TestDb::new()?;

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
        r#"{"$version": "1.0", "properties": {
            "title": {"$ref": "property_bank#/title"},
            "status": {"$ref": "property_bank#/status"}
        }}"#,
    )?;

    let config = test_config(vault_dir.path())?;
    let repository = setup_repository(test_db.db());
    let source = FsReader::new(vault_dir.path());
    let loader = Loader::new(repository, source, &config);

    let resolved = loader.load()?;

    assert_eq!(resolved.len(), 1, "Should resolve 1 schema");
    let schema = resolved.first().expect("Should have schema");
    assert_eq!(schema.name().as_ref(), "task");
    assert_eq!(schema.properties().len(), 2, "Should have 2 properties");

    Ok(())
}

/// Test that schemas with inline properties are resolved correctly.
#[test]
fn resolves_inline_properties() -> TestResult {
    let vault_dir = TempDir::new()?;
    let test_db = TestDb::new()?;

    // Property bank is required (can be empty)
    write_file(
        vault_dir.path(),
        "schemas/property_bank.json",
        r#"{"$version": "1.0", "properties": {}}"#,
    )?;
    write_file(
        vault_dir.path(),
        "schemas/note.json",
        r#"{"$version": "1.0", "properties": {
            "title": {"type": "string"},
            "done": {"type": "bool"}
        }}"#,
    )?;

    let config = test_config(vault_dir.path())?;
    let repository = setup_repository(test_db.db());
    let source = FsReader::new(vault_dir.path());
    let loader = Loader::new(repository, source, &config);

    let resolved = loader.load()?;

    assert_eq!(resolved.len(), 1, "Should resolve 1 schema");
    let schema = resolved.first().expect("Should have schema");
    assert_eq!(schema.name().as_ref(), "note");
    assert_eq!(schema.properties().len(), 2, "Should have 2 properties");

    Ok(())
}

/// Test that multiple schemas can be resolved in a single load.
#[test]
fn resolves_multiple_schemas() -> TestResult {
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
    write_file(
        vault_dir.path(),
        "schemas/note.json",
        r#"{"$version": "1.0", "properties": {"title": {"$ref": "property_bank#/title"}}}"#,
    )?;
    write_file(
        vault_dir.path(),
        "schemas/project.json",
        r#"{"$version": "1.0", "properties": {"name": {"type": "string"}}}"#,
    )?;

    let config = test_config(vault_dir.path())?;
    let repository = setup_repository(test_db.db());
    let source = FsReader::new(vault_dir.path());
    let loader = Loader::new(repository, source, &config);

    let resolved = loader.load()?;

    assert_eq!(resolved.len(), 3, "Should resolve 3 schemas");
    let names: Vec<&str> = resolved.iter().map(|s| s.name().as_ref()).collect();
    assert!(names.contains(&"task"));
    assert!(names.contains(&"note"));
    assert!(names.contains(&"project"));

    Ok(())
}

// ========================================================================
//                       Inheritance Resolution Tests
// ========================================================================

/// Test that schema inheritance is resolved correctly.
#[test]
fn resolves_schema_inheritance() -> TestResult {
    let vault_dir = TempDir::new()?;
    let test_db = TestDb::new()?;

    // Property bank is required (can be empty)
    write_file(
        vault_dir.path(),
        "schemas/property_bank.json",
        r#"{"$version": "1.0", "properties": {}}"#,
    )?;
    write_file(
        vault_dir.path(),
        "schemas/base.json",
        r#"{"$version": "1.0", "properties": {"id": {"type": "string"}}}"#,
    )?;
    write_file(
        vault_dir.path(),
        "schemas/task.json",
        r#"{"$version": "1.0", "extends": "base", "properties": {"title": {"type": "string"}}}"#,
    )?;

    let config = test_config(vault_dir.path())?;
    let repository = setup_repository(test_db.db());
    let source = FsReader::new(vault_dir.path());
    let loader = Loader::new(repository, source, &config);

    let resolved = loader.load()?;

    assert_eq!(resolved.len(), 2, "Should resolve 2 schemas");

    // Find the task schema
    let task = resolved
        .iter()
        .find(|s| s.name().as_ref() == "task")
        .expect("Should have task schema");

    // Task should have both inherited 'id' and its own 'title'
    assert_eq!(
        task.properties().len(),
        2,
        "Should have 2 properties (inherited + own)"
    );

    Ok(())
}

// ========================================================================
//                       Property Bank Tests
// ========================================================================

/// Test that property bank is loaded and persisted.
#[test]
fn loads_and_persists_property_bank() -> TestResult {
    let vault_dir = TempDir::new()?;
    let test_db = TestDb::new()?;

    write_file(
        vault_dir.path(),
        "schemas/property_bank.json",
        r#"{"$version": "1.0", "properties": {
            "title": {"type": "string"},
            "status": {"type": "string"},
            "priority": {"type": "number"}
        }}"#,
    )?;

    let config = test_config(vault_dir.path())?;
    let repository = setup_repository(test_db.db());
    let source = FsReader::new(vault_dir.path());
    let loader = Loader::new(repository, source, &config);

    // Load should process property bank
    let _resolved = loader.load()?;

    // Verify property bank was persisted (need new repository instance)
    let repository2 = setup_repository(test_db.db());
    let bank = repository2.get_property_bank()?;
    assert!(bank.is_some(), "Property bank should be persisted");
    let bank = bank.expect("Bank should exist");

    // Should have 3 properties
    assert!(bank.has(&"title".try_into()?));
    assert!(bank.has(&"status".try_into()?));
    assert!(bank.has(&"priority".try_into()?));

    Ok(())
}

// ========================================================================
//                       Error Handling Tests
// ========================================================================

/// Test that missing property bank reference is detected.
#[test]
fn detects_missing_property_bank_reference() -> TestResult {
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
        r#"{"$version": "1.0", "properties": {
            "missing": {"$ref": "property_bank#/nonexistent"}
        }}"#,
    )?;

    let config = test_config(vault_dir.path())?;
    let repository = setup_repository(test_db.db());
    let source = FsReader::new(vault_dir.path());
    let loader = Loader::new(repository, source, &config);

    // Should return error for missing reference
    let result = loader.load();
    assert!(result.is_err(), "Should fail with missing reference");

    Ok(())
}

/// Test that circular inheritance is detected.
#[test]
fn detects_circular_inheritance() -> TestResult {
    let vault_dir = TempDir::new()?;
    let test_db = TestDb::new()?;

    write_file(
        vault_dir.path(),
        "schemas/a.json",
        r#"{"$version": "1.0", "extends": "b", "properties": {}}"#,
    )?;
    write_file(
        vault_dir.path(),
        "schemas/b.json",
        r#"{"$version": "1.0", "extends": "a", "properties": {}}"#,
    )?;

    let config = test_config(vault_dir.path())?;
    let repository = setup_repository(test_db.db());
    let source = FsReader::new(vault_dir.path());
    let loader = Loader::new(repository, source, &config);

    // Should return error for circular inheritance
    let result = loader.load();
    assert!(result.is_err(), "Should fail with circular inheritance");

    Ok(())
}
