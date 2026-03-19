//! Integration tests for schema loading pipeline.
//!
//! Tests the Loader's ability to load and resolve schemas from files,
//! including:
//! - Initial loading (file → ingest → resolve → persist)
//! - Reference expansion (`$ref` to `property_bank`)
//! - Inheritance resolution (extends/excludes)
//! - Incremental loading (staleness detection)
//! - Property bank updates (incremental re-resolution)
//! - Error handling (missing refs, circular inheritance)

#![expect(
    clippy::panic_in_result_fn,
    reason = "Integration tests use assertions which panic on failure."
)]
#![expect(
    clippy::tests_outside_test_module,
    reason = "Integration tests are top-level by default."
)]
#![expect(
    clippy::indexing_slicing,
    reason = "Integration tests assert on known-length test data."
)]
#![expect(
    clippy::arbitrary_source_item_ordering,
    reason = "Test file - organize by functionality, not declaration order."
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
//                       Initial Loading Tests
// ========================================================================

/// Tests for first-time schema loading (all files are NEW).
///
/// These tests verify the Loader's ability to process schemas from files
/// for the first time, including reference expansion and persistence.
mod initial_loading {
    use super::*;

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
        let names: Vec<&str> =
            resolved.iter().map(|s| s.name().as_ref()).collect();
        assert!(names.contains(&"task"));
        assert!(names.contains(&"note"));
        assert!(names.contains(&"project"));

        Ok(())
    }

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
}

// ========================================================================
//                       Inheritance Tests
// ========================================================================

/// Tests for schema inheritance (extends/excludes).
///
/// These tests verify the Loader's ability to resolve parent-child
/// relationships between schemas.
mod inheritance {
    use super::*;

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
}

// ========================================================================
//                       Incremental Loading Tests
// ========================================================================

/// Tests for staleness detection and incremental resolution.
///
/// These tests verify the Loader's ability to detect file changes and
/// perform incremental updates using real filesystem timing and persistence.
mod incremental_loading {
    use super::*;

    /// Test that file changes are detected via mtime/hash.
    #[test]
    fn detects_file_changes() -> TestResult {
        let vault_dir = TempDir::new()?;
        let test_db = TestDb::new()?;

        // SETUP: Write initial files
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
            r#"{"$version": "1.0", "properties": {"content": {"type": "string"}}}"#,
        )?;

        // FIRST LOAD: Both schemas should be NEW
        let config = test_config(vault_dir.path())?;
        let repository = setup_repository(test_db.db());
        let source = FsReader::new(vault_dir.path());
        let loader = Loader::new(repository, source, &config);
        let first = loader.load()?;
        assert_eq!(first.len(), 2, "First load: 2 schemas");

        // WAIT: Ensure mtime changes (filesystem granularity)
        #[expect(
            clippy::disallowed_methods,
            reason = "Integration test needs real filesystem timing"
        )]
        std::thread::sleep(std::time::Duration::from_millis(10));

        // MODIFY: Change only task.json
        write_file(
            vault_dir.path(),
            "schemas/task.json",
            r#"{"$version": "1.0", "properties": {
                "title": {"$ref": "property_bank#/title"},
                "done": {"type": "bool"}
            }}"#,
        )?;

        // SECOND LOAD: Only task.json should be re-resolved
        let repository2 = setup_repository(test_db.db());
        let source2 = FsReader::new(vault_dir.path());
        let loader2 = Loader::new(repository2, source2, &config);
        let second = loader2.load()?;

        assert_eq!(second.len(), 1, "Second load: only changed schema");
        assert_eq!(second[0].name().as_ref(), "task");
        assert_eq!(
            second[0].properties().len(),
            2,
            "Should have 2 properties now"
        );

        Ok(())
    }

    /// Test that staleness detection persists across database sessions.
    #[test]
    fn staleness_persists_across_reopens() -> TestResult {
        let vault_dir = TempDir::new()?;
        let mut test_db = TestDb::new()?;

        // SETUP: Write files
        write_file(
            vault_dir.path(),
            "schemas/property_bank.json",
            r#"{"$version": "1.0", "properties": {}}"#,
        )?;
        write_file(
            vault_dir.path(),
            "schemas/task.json",
            r#"{"$version": "1.0", "properties": {"title": {"type": "string"}}}"#,
        )?;

        let config = test_config(vault_dir.path())?;

        // FIRST SESSION: Load schemas
        {
            let repository = setup_repository(test_db.db());
            let source = FsReader::new(vault_dir.path());
            let loader = Loader::new(repository, source, &config);
            let first = loader.load()?;
            assert_eq!(first.len(), 1);
        }; // Repository dropped, but database Arc still held by test_db

        // IMPORTANT: Must drop test_db.db() Arc before reopen
        // This happens implicitly when reopen() is called (it replaces the Arc)

        // REOPEN DATABASE: Simulate fresh application start
        let fresh_db = test_db.reopen()?;

        // SECOND SESSION: Load again without file changes
        let second = {
            let repository2 = setup_repository(&fresh_db);
            let source2 = FsReader::new(vault_dir.path());
            let loader2 = Loader::new(repository2, source2, &config);
            loader2.load()?
        };

        // Schemas should be FRESH (views were persisted)
        assert_eq!(
            second.len(),
            0,
            "No schemas should be re-resolved (all fresh)"
        );

        // VERIFY: Check that RawSchemaView was persisted
        let repository3 = setup_repository(&fresh_db);
        let path = std::path::PathBuf::from("schemas/task.json");
        let view = repository3
            .find_raw_schema_view_by_path(&path.to_string_lossy())?;
        assert!(view.is_some(), "RawSchemaView should be persisted");

        Ok(())
    }

    /// Test that property bank changes trigger schema re-resolution.
    #[test]
    fn property_bank_update_triggers_re_resolution() -> TestResult {
        let vault_dir = TempDir::new()?;
        let test_db = TestDb::new()?;

        // SETUP: Initial files
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

        // FIRST LOAD
        let config = test_config(vault_dir.path())?;
        let repository = setup_repository(test_db.db());
        let source = FsReader::new(vault_dir.path());
        let loader = Loader::new(repository, source, &config);
        let first = loader.load()?;
        assert_eq!(first.len(), 1);
        assert_eq!(first[0].properties().len(), 1, "Should have 1 property");

        // WAIT for filesystem timing
        #[expect(
            clippy::disallowed_methods,
            reason = "Integration test needs real filesystem timing"
        )]
        std::thread::sleep(std::time::Duration::from_millis(10));

        // MODIFY: Add new property to property_bank
        write_file(
            vault_dir.path(),
            "schemas/property_bank.json",
            r#"{"$version": "1.0", "properties": {
                "title": {"type": "string"},
                "status": {"type": "string"}
            }}"#,
        )?;

        // UPDATE: task.json to use new property
        write_file(
            vault_dir.path(),
            "schemas/task.json",
            r#"{"$version": "1.0", "properties": {
                "title": {"$ref": "property_bank#/title"},
                "status": {"$ref": "property_bank#/status"}
            }}"#,
        )?;

        // SECOND LOAD: Schema should be re-resolved with new property
        let repository2 = setup_repository(test_db.db());
        let source2 = FsReader::new(vault_dir.path());
        let loader2 = Loader::new(repository2, source2, &config);
        let second = loader2.load()?;

        assert_eq!(second.len(), 1, "Should re-resolve schema");
        assert_eq!(
            second[0].properties().len(),
            2,
            "Should have 2 properties now"
        );

        // VERIFY: Check that both properties resolved correctly
        let task = &second[0];
        assert!(task.properties().contains_key("title"));
        assert!(task.properties().contains_key("status"));

        Ok(())
    }
}

// ========================================================================
//                       Error Handling Tests
// ========================================================================

/// Tests for error detection and propagation.
///
/// These tests verify the Loader's ability to detect and report errors
/// during the loading pipeline.
mod error_handling {
    use super::*;

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
}
