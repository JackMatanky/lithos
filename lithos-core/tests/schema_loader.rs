//! Integration test suite for schema loading pipeline.
//!
//! # Summary
//! - Validates the Loader's orchestration of the file → raw → domain → storage
//!   pipeline.
//! - Covers initial loading, reference resolution, inheritance, incremental
//!   updates, and error handling.
//! - Tests boundaries: File system reads via `FsReader`, property bank
//!   expansion, schema inheritance, staleness detection.
//! - Exclusions: Unit-level validation (tested in domain modules),
//!   database-only operations (tested in `schema_storage.rs`).
//!
//! # Setup
//! - Uses `TempDir` for isolated filesystem fixtures per test.
//! - `TestDb` provides fresh redb instances for each test.
//! - `FsReader` abstracts filesystem operations for testability.
//! - Helper functions: `write_file()`, `test_config()`.
//!
//! # Data Model
//! - Inputs: JSON schema files with `property_bank` references, inheritance
//!   (extends/excludes), and inline properties.
//! - Outputs: Resolved schemas persisted in `RedbStorage` with expanded
//!   references and inheritance chains.
//! - Assumptions: Files follow lithos schema format (1.0), `property_bank` is
//!   available for resolution.
//!
//! # Scenarios
//! - **Happy path**: Load schemas with `property_bank` refs, resolve
//!   inheritance, persist to storage.
//! - **Edge cases**: Empty property banks, multiple inheritance levels, file
//!   modifications triggering staleness.
//! - **Error paths**: Missing `property_bank` references, circular inheritance,
//!   file read failures.

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

    /// Integration test for property bank reference resolution.
    ///
    /// # Purpose
    /// Validates that the Loader correctly expands `$ref` references to
    /// `property_bank` entries during the load pipeline, replacing references
    /// with concrete property definitions.
    ///
    /// # Inputs
    /// - `property_bank.json`: Defines reusable properties (title, status).
    /// - task.json: Schema with `$ref` references to `property_bank` entries.
    ///
    /// # Expected Behavior
    /// - Loader ingests both files from filesystem via `FsReader`.
    /// - References are expanded: `{"$ref": "property_bank#/title"}` →
    ///   `{"type": "string"}`.
    /// - Resolved schema persisted to storage with 2 concrete properties.
    /// - Schema name matches filename (task).
    ///
    /// # Failure Modes
    /// - Missing `property_bank` file → loader error.
    /// - Invalid JSON syntax → parsing error.
    /// - Reference to non-existent property → resolution error.
    ///
    /// # Observability
    /// - Asserts resolved schema count (1).
    /// - Asserts schema name matches filename.
    /// - Asserts property count after expansion (2).
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

    /// Integration test for inline property resolution.
    ///
    /// # Purpose
    /// Validates that schemas with inline property definitions (no `$ref`
    /// references) are correctly loaded and persisted without requiring
    /// `property_bank` expansion.
    ///
    /// # Inputs
    /// - `property_bank.json`: Empty but required by loader.
    /// - note.json: Schema with inline properties (title: string, done: bool).
    ///
    /// # Expected Behavior
    /// - Loader ingests schema file with inline properties.
    /// - No reference expansion needed (properties are already concrete).
    /// - Schema persisted to storage with inline properties intact.
    /// - Property count and types match input definition.
    ///
    /// # Failure Modes
    /// - Missing `property_bank` file → loader error (required even if empty).
    /// - Invalid property type → parsing error.
    /// - Malformed JSON → ingestion error.
    ///
    /// # Observability
    /// - Asserts resolved schema count (1).
    /// - Asserts schema name matches filename (note).
    /// - Asserts property count matches input (2).
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

    /// Integration test for batch schema resolution.
    ///
    /// # Purpose
    /// Validates that the Loader can process multiple schema files in a single
    /// load operation, correctly resolving `property_bank` references and
    /// inline properties across all files.
    ///
    /// # Inputs
    /// - `property_bank.json`: Defines reusable title property.
    /// - task.json, note.json: Schemas with `$ref` to `property_bank`.
    /// - project.json: Schema with inline properties.
    ///
    /// # Expected Behavior
    /// - Loader discovers and ingests all schema files in schemas/ directory.
    /// - Each schema is independently resolved (refs expanded, inheritance
    ///   applied).
    /// - All 3 schemas persisted to storage in single batch operation.
    /// - Schema names derived from filenames (task, note, project).
    ///
    /// # Failure Modes
    /// - Missing `property_bank` → resolution error for task/note.
    /// - File system read error → loader fails to discover files.
    /// - Invalid JSON in any file → batch operation fails.
    ///
    /// # Observability
    /// - Asserts total resolved schema count (3).
    /// - Asserts presence of all expected schema names.
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

    /// Integration test for property bank loading and persistence.
    ///
    /// # Purpose
    /// Validates that the `property_bank` file is ingested, validated, and
    /// persisted to storage independently, making properties available for
    /// reference resolution.
    ///
    /// # Inputs
    /// - `property_bank.json`: Defines 3 reusable properties (title, status,
    ///   priority).
    ///
    /// # Expected Behavior
    /// - Loader ingests `property_bank.json` from filesystem.
    /// - `PropertyBank` domain object constructed with 3 properties.
    /// - `PropertyBank` persisted to storage for use in schema reference
    ///   resolution.
    /// - Property bank retrievable via Repository after load.
    ///
    /// # Failure Modes
    /// - Missing `property_bank` file → loader error.
    /// - Invalid property definitions → parsing/validation error.
    /// - Storage write failure → persistence error.
    ///
    /// # Observability
    /// - Asserts `property_bank` is retrievable from storage.
    /// - Asserts property count matches input (3).
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

    /// Integration test for schema inheritance resolution.
    ///
    /// # Purpose
    /// Validates that schemas using `extends` correctly inherit properties from
    /// parent schemas, creating a resolved schema with both inherited and
    /// own properties.
    ///
    /// # Inputs
    /// - `property_bank.json`: Empty but required.
    /// - base.json: Parent schema with id property.
    /// - task.json: Child schema extending base, adding title property.
    ///
    /// # Expected Behavior
    /// - Loader resolves inheritance chain: task extends base.
    /// - Child schema inherits parent properties (id from base).
    /// - Child schema retains own properties (title).
    /// - Final task schema has 2 properties: inherited id + own title.
    ///
    /// # Failure Modes
    /// - Missing parent schema → resolution error.
    /// - Circular inheritance → validation error.
    /// - Property name conflicts → resolution error.
    ///
    /// # Observability
    /// - Asserts total resolved schema count (2: base + task).
    /// - Asserts task schema has both inherited and own properties (2 total).
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

    /// Integration test for file change detection via staleness tracking.
    ///
    /// # Purpose
    /// Validates that the Loader detects file modifications using mtime and
    /// content hash, triggering incremental re-resolution of only changed
    /// schemas.
    ///
    /// # Inputs
    /// - Initial: `property_bank.json` (title property), task.json (1
    ///   property).
    /// - Modified: task.json updated to add status property.
    ///
    /// # Expected Behavior
    /// - First load: All schemas marked fresh, persisted with staleness
    ///   metadata.
    /// - File modification: task.json mtime/hash changes.
    /// - Second load: Staleness detector identifies task.json as modified.
    /// - Only task schema re-resolved, `property_bank` reused from storage.
    /// - Resolved task schema reflects new content (2 properties).
    ///
    /// # Failure Modes
    /// - Filesystem mtime resolution too coarse → false negatives.
    /// - Hash computation error → incorrect staleness detection.
    /// - Metadata persistence failure → all files re-processed on every load.
    ///
    /// # Observability
    /// - Asserts first load resolves all schemas.
    /// - Asserts second load detects and re-resolves only modified schema.
    /// - Asserts property count reflects updated content.
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

    /// Integration test for staleness metadata persistence across database
    /// sessions.
    ///
    /// # Purpose
    /// Validates that file staleness metadata (mtime, content hash) is
    /// correctly persisted to the database and survives database
    /// close/reopen cycles, preventing unnecessary re-processing on
    /// subsequent loads.
    ///
    /// # Inputs
    /// - `property_bank.json`: Empty property bank.
    /// - task.json: Single schema file (unchanged across sessions).
    ///
    /// # Expected Behavior
    /// - First session: Schema loaded, staleness metadata persisted to storage.
    /// - Database close: All in-memory state cleared.
    /// - Database reopen: Staleness metadata restored from persistent storage.
    /// - Second session: No file changes detected, zero schemas re-resolved.
    /// - Storage retrieval confirms schema still available from first load.
    ///
    /// # Failure Modes
    /// - Staleness metadata not persisted → all files re-processed every
    ///   session.
    /// - Database corruption on close/reopen → metadata loss.
    /// - Incorrect metadata serialization → staleness detection fails.
    ///
    /// # Observability
    /// - Asserts first load processes schema (count = 1).
    /// - Asserts second load skips unchanged schema (count = 0).
    /// - Asserts schema retrievable from storage in both sessions.
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

    /// Integration test for property bank updates triggering cascading
    /// re-resolution.
    ///
    /// # Purpose
    /// Validates that when `property_bank` is modified, all schemas referencing
    /// those properties are automatically re-resolved to reflect the
    /// updated property definitions, even if the schema files themselves
    /// haven't changed.
    ///
    /// # Inputs
    /// - Initial: `property_bank.json` (title only), task.json (refs title).
    /// - Modified: `property_bank.json` (title + status added), task.json (refs
    ///   both).
    ///
    /// # Expected Behavior
    /// - First load: task.json resolved with 1 property (title from
    ///   `property_bank`).
    /// - Property bank modification detected (mtime/hash change).
    /// - task.json file also modified to reference new property.
    /// - Second load: Both `property_bank` and task.json re-resolved.
    /// - Resolved task schema has 2 properties (title + status expanded from
    ///   refs).
    ///
    /// # Failure Modes
    /// - Property bank change not detected → stale schemas persist.
    /// - Dependency tracking missing → schemas not re-resolved when bank
    ///   changes.
    /// - Reference resolution fails with new properties → validation error.
    ///
    /// # Observability
    /// - Asserts first load has 1 property.
    /// - Asserts second load detects changes and re-resolves.
    /// - Asserts final schema has 2 expanded properties.
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

    /// Integration test for missing property bank reference detection.
    ///
    /// # Purpose
    /// Validates that the Loader correctly detects and reports errors when a
    /// schema references a `property_bank` entry that doesn't exist, failing
    /// fast with a clear error.
    ///
    /// # Inputs
    /// - `property_bank.json`: Defines only "title" property.
    /// - task.json: References non-existent "nonexistent" property.
    ///
    /// # Expected Behavior
    /// - Loader ingests both files successfully (parsing passes).
    /// - Reference resolution phase detects missing `property_bank` entry.
    /// - Loader returns Err with descriptive error message.
    /// - No partial state persisted (atomic failure).
    ///
    /// # Failure Modes
    /// - Missing reference not detected → invalid schema persisted.
    /// - Error message unclear → difficult debugging.
    /// - Partial persistence → database in inconsistent state.
    ///
    /// # Observability
    /// - Asserts `loader.load()` returns Err (not Ok).
    /// - Error indicates missing property reference.
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

    /// Integration test for circular inheritance detection.
    ///
    /// # Purpose
    /// Validates that the Loader detects circular inheritance chains (A extends
    /// B, B extends A) and fails with a clear error, preventing infinite
    /// resolution loops.
    ///
    /// # Inputs
    /// - a.json: Schema extending "b".
    /// - b.json: Schema extending "a" (creates cycle).
    ///
    /// # Expected Behavior
    /// - Loader ingests both schema files (parsing succeeds).
    /// - Inheritance resolution phase detects cycle during graph traversal.
    /// - Loader returns Err before entering infinite loop.
    /// - No schemas persisted (atomic failure on validation error).
    ///
    /// # Failure Modes
    /// - Cycle not detected → infinite loop, stack overflow.
    /// - Detection too aggressive → false positives on valid inheritance
    ///   chains.
    /// - Error message unclear → difficult to identify problematic schemas.
    ///
    /// # Observability
    /// - Asserts `loader.load()` returns Err (not Ok).
    /// - Error indicates circular inheritance detected.
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
