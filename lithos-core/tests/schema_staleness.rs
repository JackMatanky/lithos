//! Integration tests for schema staleness detection.
//!
//! Validates `is_schema_stale` and `is_bank_stale` behavior using
//! persisted metadata and bank versions.

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
    application::schema::SchemaService,
    config::{
        aggregate::Config,
        raw::RawConfig,
        vault::{VaultId, VaultRoot},
    },
    fs::FsReader,
    schema::{
        adapter::ingestor::Ingestor,
        aggregate::{SchemaId, SchemaName, Timestamp},
        bank::{BankVersion, PropertyBank},
    },
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
    let config = Config::build(&raw, VaultId::new(), root)?;
    Ok(config)
}

/// Read file timestamps into `Timestamp` values.
fn file_times(path: &Path) -> (Option<Timestamp>, Option<Timestamp>) {
    let metadata = std::fs::metadata(path).ok();
    let modified = metadata
        .as_ref()
        .and_then(|meta| meta.modified().ok())
        .and_then(|time| {
            time.duration_since(std::time::SystemTime::UNIX_EPOCH).ok()
        })
        .map(|duration| Timestamp::from_secs(duration.as_secs()));
    let created = metadata
        .as_ref()
        .and_then(|meta| meta.created().ok())
        .and_then(|time| {
            time.duration_since(std::time::SystemTime::UNIX_EPOCH).ok()
        })
        .map(|duration| Timestamp::from_secs(duration.as_secs()));
    (modified, created)
}

// ========================================================================
//                          Bank Staleness
// ========================================================================

/// **3.7-INT-001**: Missing property bank is stale.
#[test]
fn bank_stale_when_missing() -> TestResult {
    let test_db = TestDb::new()?;
    let (_command, query) = setup_cqrs(test_db.db());

    let stale = query.is_bank_stale(BankVersion::initial())?;
    assert!(stale, "Missing bank should be stale");

    Ok(())
}

/// **3.7-INT-002**: Bank is fresh when versions match.
#[test]
fn bank_fresh_when_versions_match() -> TestResult {
    let test_db = TestDb::new()?;
    let (command, query) = setup_cqrs(test_db.db());

    let bank = PropertyBank::new();
    command.save_property_bank(&bank)?;

    let stale = query.is_bank_stale(bank.version())?;
    assert!(!stale, "Matching bank version should be fresh");

    Ok(())
}

/// **3.7-INT-003**: Bank is stale when version differs.
#[test]
fn bank_stale_when_version_differs() -> TestResult {
    let test_db = TestDb::new()?;
    let (command, query) = setup_cqrs(test_db.db());

    let mut bank = PropertyBank::new();
    let prop = bool_property("flag")?;
    bank.register(prop)?;
    command.save_property_bank(&bank)?;

    let stale = query.is_bank_stale(BankVersion::initial())?;
    assert!(stale, "Different bank version should be stale");

    Ok(())
}

// ========================================================================
//                          Schema Staleness
// ========================================================================

/// **3.7-INT-004**: Missing schema is stale.
#[test]
fn schema_stale_when_missing() -> TestResult {
    let test_db = TestDb::new()?;
    let (_command, query) = setup_cqrs(test_db.db());

    let stale = query.is_schema_stale(
        SchemaId::new(),
        None,
        None,
        BankVersion::initial(),
    )?;
    assert!(stale, "Missing schema should be stale");

    Ok(())
}

/// **3.7-INT-005**: Schema is fresh when metadata matches.
#[test]
fn schema_fresh_when_metadata_matches() -> TestResult {
    let dir = TempDir::new()?;
    write_file(
        dir.path(),
        "schemas/property_bank.json",
        r#"{"$version": "1.0", "properties": {"flag": {"type": "bool"}}}"#,
    )?;
    write_file(
        dir.path(),
        "schemas/task.json",
        r#"{
            "$version": "1.0",
            "name": "task",
            "properties": {
                "flag": {"$ref": "property_bank#/flag"}
            }
        }"#,
    )?;

    let config = test_config(dir.path())?;
    let test_db = TestDb::new()?;
    let (command, query) = setup_cqrs(test_db.db());

    let ingestor = Ingestor::new(FsReader::new(dir.path()), &config);
    let service = SchemaService::new(query, command);
    service.load(&ingestor)?;

    let (_cmd, query2) = setup_cqrs(test_db.db());
    let schema = query2
        .find_by_name(&SchemaName::new("task")?)?
        .expect("schema should exist");
    let schema_id = schema.id();

    let bank = query2.get_property_bank()?.expect("bank should exist");
    let bank_version = bank.version();

    let schema_path = dir.path().join("schemas/task.json");
    let (modified_at, created_at) = file_times(&schema_path);

    let stale = query2.is_schema_stale(
        schema_id,
        created_at,
        modified_at,
        bank_version,
    )?;
    assert!(!stale, "Matching metadata should be fresh");

    Ok(())
}

/// **3.7-INT-006**: Schema is stale when modified time differs.
#[test]
fn schema_stale_when_modified_differs() -> TestResult {
    let dir = TempDir::new()?;
    write_file(
        dir.path(),
        "schemas/property_bank.json",
        r#"{"$version": "1.0", "properties": {"flag": {"type": "bool"}}}"#,
    )?;
    write_file(
        dir.path(),
        "schemas/task.json",
        r#"{
            "$version": "1.0",
            "name": "task",
            "properties": {
                "flag": {"$ref": "property_bank#/flag"}
            }
        }"#,
    )?;

    let config = test_config(dir.path())?;
    let test_db = TestDb::new()?;
    let (command, query) = setup_cqrs(test_db.db());

    let ingestor = Ingestor::new(FsReader::new(dir.path()), &config);
    let service = SchemaService::new(query, command);
    service.load(&ingestor)?;

    let (_cmd, query2) = setup_cqrs(test_db.db());
    let schema = query2
        .find_by_name(&SchemaName::new("task")?)?
        .expect("schema should exist");
    let schema_id = schema.id();

    let bank = query2.get_property_bank()?.expect("bank should exist");
    let bank_version = bank.version();

    let schema_path = dir.path().join("schemas/task.json");
    let (modified_at, created_at) = file_times(&schema_path);
    let Some(modified_at) = modified_at else {
        return Err("modified timestamp unavailable".into());
    };

    let stale = query2.is_schema_stale(
        schema_id,
        created_at,
        Some(Timestamp::from_secs(modified_at.as_secs() + 1)),
        bank_version,
    )?;
    assert!(stale, "Modified time mismatch should be stale");

    Ok(())
}

/// **3.7-INT-007**: Schema is stale when bank version differs.
#[test]
fn schema_stale_when_bank_version_differs() -> TestResult {
    let dir = TempDir::new()?;
    write_file(
        dir.path(),
        "schemas/property_bank.json",
        r#"{"$version": "1.0", "properties": {"flag": {"type": "bool"}}}"#,
    )?;
    write_file(
        dir.path(),
        "schemas/task.json",
        r#"{
            "$version": "1.0",
            "name": "task",
            "properties": {
                "flag": {"$ref": "property_bank#/flag"}
            }
        }"#,
    )?;

    let config = test_config(dir.path())?;
    let test_db = TestDb::new()?;
    let (command, query) = setup_cqrs(test_db.db());

    let ingestor = Ingestor::new(FsReader::new(dir.path()), &config);
    let service = SchemaService::new(query, command);
    service.load(&ingestor)?;

    let (_cmd, query2) = setup_cqrs(test_db.db());
    let schema = query2
        .find_by_name(&SchemaName::new("task")?)?
        .expect("schema should exist");
    let schema_id = schema.id();

    let bank = query2.get_property_bank()?.expect("bank should exist");
    let bank_version = bank.version();

    let schema_path = dir.path().join("schemas/task.json");
    let (modified_at, created_at) = file_times(&schema_path);

    let stale = query2.is_schema_stale(
        schema_id,
        created_at,
        modified_at,
        bank_version.increment(),
    )?;
    assert!(stale, "Bank version mismatch should be stale");

    Ok(())
}
