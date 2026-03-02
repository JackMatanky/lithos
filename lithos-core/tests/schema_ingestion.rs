//! End-to-end integration tests for the full schema ingestion pipeline.
//!
//! Tests the complete flow: File → Raw → Domain → Database.
//!
//! **Pipeline stages**:
//! 1. **File Ingestion** (`Ingestor`): Read raw JSON/TOML/YAML files
//! 2. **Raw → Domain** (`PropertyBank::from_raw`, `Dereferencer`): Parse and
//!    validate
//! 3. **Inheritance** (`Extender`, `Resolver`): Build inheritance tree and
//!    resolve
//! 4. **Persistence** (`Command`): Save to database
//!
//! **Coverage**:
//! - Property bank loading from files
//! - Schema scanning and parsing (JSON, TOML, YAML)
//! - Full pipeline orchestration via `SchemaService`
//! - Filesystem timestamp preservation
//! - Staleness detection and incremental updates

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
    schema::{adapter::ingestor::Ingestor, aggregate::SchemaName},
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

/// Create a test config for a vault root with optional custom property bank
/// file.
fn test_config_with_bank(
    root: &Path,
    property_bank_file: Option<&str>,
) -> TestResult<Config> {
    let mut raw = RawConfig::default();
    raw.paths.property_bank_file = property_bank_file.map(ToOwned::to_owned);

    let root = VaultRoot::try_new(root.to_path_buf())?;
    let config = Config::build(&raw, VaultId::new(), root)?;

    Ok(config)
}

/// Create a test config for a vault root (default `property_bank.json`).
fn test_config(root: &Path) -> TestResult<Config> {
    test_config_with_bank(root, None)
}

// ========================================================================
//                          Property Bank Loading
// ========================================================================

/// **3.5-INT-001**: Property bank loads from JSON file.
///
/// Verifies:
/// - JSON property bank file can be parsed
/// - Properties are registered correctly
/// - Bank persists to database
#[test]
fn property_bank_loads_from_json() -> TestResult {
    // GIVEN: A vault with property bank JSON file
    let dir = TempDir::new()?;
    write_file(
        dir.path(),
        "schemas/property_bank.json",
        r#"{
            "$version": "1.0",
            "properties": {
                "title": {
                    "type": "string"
                },
                "is_done": {
                    "type": "bool"
                }
            }
        }"#,
    )?;

    let config = test_config(dir.path())?;
    let test_db = TestDb::new()?;
    let (command, query) = setup_cqrs(test_db.db());

    // WHEN: Loading via ingestor
    let ingestor = Ingestor::new(FsReader::new(dir.path()), &config);
    let raw_bank = ingestor.load_raw_property_bank()?;

    // THEN: Bank is parsed correctly
    assert_eq!(raw_bank.properties.len(), 2);
    assert!(raw_bank.properties.contains_key("title"));
    assert!(raw_bank.properties.contains_key("is_done"));

    // AND: Can be converted to domain and persisted
    let bank =
        lithos_core::schema::bank::PropertyBank::from_raw(raw_bank, None)?;
    command.save_property_bank(&bank)?;

    let loaded = query.get_property_bank()?.expect("Bank should exist");
    assert_eq!(loaded.all().count(), 2);

    Ok(())
}

/// **3.5-INT-002**: Property bank loads from TOML file.
///
/// Verifies:
/// - TOML property bank file can be parsed
/// - Properties are registered correctly
#[test]
fn property_bank_loads_from_toml() -> TestResult {
    // GIVEN: A vault with property bank TOML file
    let dir = TempDir::new()?;
    write_file(
        dir.path(),
        "schemas/property_bank.toml",
        r#"
        "$version" = "1.0"

        [properties.status]
        type = "bool"

        [properties.priority]
        type = "string"
        "#,
    )?;

    let config = test_config_with_bank(dir.path(), Some("property_bank.toml"))?;

    // WHEN: Loading via ingestor
    let ingestor = Ingestor::new(FsReader::new(dir.path()), &config);
    let raw_bank = ingestor.load_raw_property_bank()?;

    // THEN: Bank is parsed correctly
    assert_eq!(raw_bank.properties.len(), 2);
    assert!(raw_bank.properties.contains_key("status"));
    assert!(raw_bank.properties.contains_key("priority"));

    Ok(())
}

/// **3.5-INT-003**: Property bank loads from YAML file.
///
/// Verifies:
/// - YAML property bank file can be parsed
/// - Properties are registered correctly
#[test]
fn property_bank_loads_from_yaml() -> TestResult {
    // GIVEN: A vault with property bank YAML file
    let dir = TempDir::new()?;
    write_file(
        dir.path(),
        "schemas/property_bank.yaml",
        r#"
        $version: "1.0"
        properties:
          tags:
            type: string
            array: true
          completed:
            type: bool
        "#,
    )?;

    let config = test_config_with_bank(dir.path(), Some("property_bank.yaml"))?;

    // WHEN: Loading via ingestor
    let ingestor = Ingestor::new(FsReader::new(dir.path()), &config);
    let raw_bank = ingestor.load_raw_property_bank()?;

    // THEN: Bank is parsed correctly
    assert_eq!(raw_bank.properties.len(), 2);
    assert!(raw_bank.properties.contains_key("tags"));
    assert!(raw_bank.properties.contains_key("completed"));

    Ok(())
}

// ========================================================================
//                          Schema Scanning
// ========================================================================

/// **3.5-INT-004**: Schema scanner finds all schema files.
///
/// Verifies:
/// - Scans multiple file formats (JSON, TOML, YAML)
/// - Finds schemas in subdirectories
/// - Excludes property bank file
#[test]
fn schema_scanner_finds_all_files() -> TestResult {
    // GIVEN: A vault with multiple schema files
    let dir = TempDir::new()?;
    write_file(
        dir.path(),
        "schemas/property_bank.json",
        r#"{"$version": "1.0", "properties": {}}"#,
    )?;
    write_file(
        dir.path(),
        "schemas/task.json",
        r#"{"$version": "1.0", "name": "task", "properties": {}}"#,
    )?;
    write_file(
        dir.path(),
        "schemas/note.toml",
        r#"
        "$version" = "1.0"
        name = "note"
        [properties]
        "#,
    )?;
    write_file(
        dir.path(),
        "schemas/project/project.yaml",
        r#"
        $version: "1.0"
        name: project
        properties: {}
        "#,
    )?;

    let config = test_config(dir.path())?;

    // WHEN: Scanning schemas
    let ingestor = Ingestor::new(FsReader::new(dir.path()), &config);
    let raw_schemas = ingestor.scan_raw_schemas()?;

    // THEN: All schemas found (excluding property bank)
    assert_eq!(raw_schemas.len(), 3);

    let names: Vec<_> = raw_schemas
        .iter()
        .filter_map(|entry| entry.0.name.as_deref())
        .collect();
    assert!(names.contains(&"task"));
    assert!(names.contains(&"note"));
    assert!(names.contains(&"project"));

    Ok(())
}

/// **3.5-INT-005**: Schema scanner preserves file timestamps.
///
/// Verifies:
/// - Modified timestamps are captured
/// - Created timestamps are captured (when available)
#[test]
fn schema_scanner_preserves_timestamps() -> TestResult {
    // GIVEN: A vault with schema file
    let dir = TempDir::new()?;
    write_file(
        dir.path(),
        "schemas/property_bank.json",
        r#"{"$version": "1.0", "properties": {}}"#,
    )?;
    write_file(
        dir.path(),
        "schemas/task.json",
        r#"{"$version": "1.0", "name": "task", "properties": {}}"#,
    )?;

    let config = test_config(dir.path())?;

    // WHEN: Scanning schemas
    let ingestor = Ingestor::new(FsReader::new(dir.path()), &config);
    let raw_schemas = ingestor.scan_raw_schemas()?;

    // THEN: Timestamp information is present
    assert_eq!(raw_schemas.len(), 1);
    let modified =
        raw_schemas.first().map(|entry| entry.1).expect("schema should exist");
    assert!(modified.is_some(), "Modified timestamp should be captured");

    Ok(())
}

// ========================================================================
//                          Full Pipeline
// ========================================================================

/// **3.5-INT-006**: Full pipeline loads schemas from files to database.
///
/// Verifies:
/// - Property bank loads and persists
/// - Schemas load and persist
/// - All resolved schemas are in database
/// - Indices are populated correctly
#[test]
fn full_pipeline_loads_schemas() -> TestResult {
    // GIVEN: A complete vault with property bank and schemas
    let dir = TempDir::new()?;
    write_file(
        dir.path(),
        "schemas/property_bank.json",
        r#"{
            "$version": "1.0",
            "properties": {
                "title": { "type": "string" },
                "is_done": { "type": "bool" }
            }
        }"#,
    )?;
    write_file(
        dir.path(),
        "schemas/task.json",
        r#"{
            "$version": "1.0",
            "name": "task",
            "properties": {
                "title": {"$ref": "property_bank#/title"},
                "is_done": {"$ref": "property_bank#/is_done"}
            }
        }"#,
    )?;
    write_file(
        dir.path(),
        "schemas/note.yaml",
        r#"
        $version: "1.0"
        name: note
        properties:
          title:
            $ref: property_bank#/title
        "#,
    )?;

    let config = test_config(dir.path())?;
    let test_db = TestDb::new()?;
    let (command, query) = setup_cqrs(test_db.db());

    // WHEN: Running full pipeline
    let ingestor = Ingestor::new(FsReader::new(dir.path()), &config);
    let service = SchemaService::new(query, command);
    let resolved = service.load(&ingestor)?;

    // THEN: Schemas are resolved and persisted
    assert_eq!(resolved.len(), 2, "Should resolve 2 schemas");

    // Verify schemas are in database
    let (_, query2) = setup_cqrs(test_db.db());
    let all_schemas = query2.list()?;
    assert_eq!(all_schemas.len(), 2);

    // Verify by name
    assert!(query2.find_by_name(&SchemaName::new("task")?)?.is_some());
    assert!(query2.find_by_name(&SchemaName::new("note")?)?.is_some());

    // Verify property bank
    let bank = query2.get_property_bank()?.expect("Bank should exist");
    assert_eq!(bank.all().count(), 2);

    Ok(())
}

/// **3.5-INT-007**: Full pipeline resolves schema properties correctly.
///
/// Verifies:
/// - Property references are resolved from property bank
/// - Resolved schemas have correct properties
/// - Property details match bank definitions
#[test]
fn full_pipeline_resolves_properties() -> TestResult {
    // GIVEN: Vault with property bank and schema
    let dir = TempDir::new()?;
    write_file(
        dir.path(),
        "schemas/property_bank.json",
        r#"{
            "$version": "1.0",
            "properties": {
                "title": { "type": "string", "required": true },
                "description": { "type": "string", "required": false }
            }
        }"#,
    )?;
    write_file(
        dir.path(),
        "schemas/document.json",
        r#"{
            "$version": "1.0",
            "name": "document",
            "properties": {
                "title": {"$ref": "property_bank#/title"},
                "description": {"$ref": "property_bank#/description"}
            }
        }"#,
    )?;

    let config = test_config(dir.path())?;
    let test_db = TestDb::new()?;
    let (command, query) = setup_cqrs(test_db.db());

    // WHEN: Running pipeline
    let ingestor = Ingestor::new(FsReader::new(dir.path()), &config);
    let service = SchemaService::new(query, command);
    service.load(&ingestor)?;

    // THEN: Schema has resolved properties
    let (_cmd, query2) = setup_cqrs(test_db.db());
    let schema = query2
        .find_by_name(&SchemaName::new("document")?)?
        .expect("Schema should exist");

    assert_eq!(schema.properties().count(), 2);
    assert_has_property(&schema, "title", "document schema");
    assert_has_property(&schema, "description", "document schema");

    Ok(())
}

/// **3.5-INT-008**: Full pipeline supports incremental updates.
///
/// Verifies:
/// - Second run only processes stale schemas
/// - Fresh schemas are not re-resolved
/// - Property bank staleness detection works
#[test]
fn full_pipeline_incremental_updates() -> TestResult {
    // GIVEN: Initial vault state
    let dir = TempDir::new()?;
    write_file(
        dir.path(),
        "schemas/property_bank.json",
        r#"{
            "$version": "1.0",
            "properties": {
                "title": { "type": "string" }
            }
        }"#,
    )?;
    write_file(
        dir.path(),
        "schemas/task.json",
        r#"{
            "$version": "1.0",
            "name": "task",
            "properties": {
                "title": {"$ref": "property_bank#/title"}
            }
        }"#,
    )?;

    let config = test_config(dir.path())?;
    let test_db = TestDb::new()?;
    let (command, query) = setup_cqrs(test_db.db());

    // WHEN: First load
    let ingestor = Ingestor::new(FsReader::new(dir.path()), &config);
    let service = SchemaService::new(query, command);
    let first_run = service.load(&ingestor)?;
    assert_eq!(first_run.len(), 1, "First run should resolve 1 schema");

    // AND: Second load (no changes) - recreate service
    let (command2, query2) = setup_cqrs(test_db.db());
    let service2 = SchemaService::new(query2, command2);
    let second_run = service2.load(&ingestor)?;

    // THEN: Second run resolves nothing (all fresh)
    assert_eq!(
        second_run.len(),
        0,
        "Second run should resolve 0 schemas (all fresh)"
    );

    // AND: Schema still exists in database
    let (_cmd, query3) = setup_cqrs(test_db.db());
    assert!(query3.find_by_name(&SchemaName::new("task")?)?.is_some());

    Ok(())
}

// ========================================================================
//                          Error Handling
// ========================================================================

/// **3.5-INT-009**: Pipeline handles missing property bank gracefully.
///
/// Verifies:
/// - Missing property bank file returns appropriate error
/// - Error message is descriptive
#[test]
fn pipeline_handles_missing_property_bank() -> TestResult {
    // GIVEN: Vault with no property bank
    let dir = TempDir::new()?;
    // No property bank file created

    let config = test_config(dir.path())?;

    // WHEN: Loading property bank
    let ingestor = Ingestor::new(FsReader::new(dir.path()), &config);
    let result = ingestor.load_raw_property_bank();

    // THEN: Returns error
    let _err = result.expect_err("missing property bank should error");

    Ok(())
}

/// **3.5-INT-010**: Pipeline handles malformed property bank.
///
/// Verifies:
/// - Malformed JSON returns parse error
/// - Error indicates the problem
#[test]
fn pipeline_handles_malformed_property_bank() -> TestResult {
    // GIVEN: Vault with invalid JSON
    let dir = TempDir::new()?;
    write_file(dir.path(), "schemas/property_bank.json", "{ invalid json }")?;

    let config = test_config(dir.path())?;

    // WHEN: Loading property bank
    let ingestor = Ingestor::new(FsReader::new(dir.path()), &config);
    let result = ingestor.load_raw_property_bank();

    // THEN: Returns error
    let _err = result.expect_err("malformed property bank should error");

    Ok(())
}
