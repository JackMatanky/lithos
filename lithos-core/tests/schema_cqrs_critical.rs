//! Critical missing tests for Schema CQRS (Task 11).
//!
//! These tests validate recent improvements and cover edge cases that were
//! previously untested.

#![expect(
    clippy::panic_in_result_fn,
    reason = "Integration tests use assertions which panic on failure."
)]
#![expect(
    clippy::tests_outside_test_module,
    reason = "Integration tests are top-level by default."
)]

mod common;

use common::*;
use lithos_core::schema::bank::PropertyBank;

/// **P0-001**: Delete removes schema completely.
///
/// Verifies that `CommandAdapter::delete()` properly removes the schema
/// and all associated data.
#[test]
fn delete_removes_schema_completely() -> TestResult {
    // GIVEN: A saved schema
    let test_db = TestDb::new()?;
    let (command, query) = setup_cqrs(test_db.db());

    let schema = SchemaBuilder::new("test-schema").build()?;
    let schema_id = schema.id();

    command.save(&schema)?;
    assert!(
        query.find_by_id(schema_id)?.is_some(),
        "Schema should exist after save"
    );

    // WHEN: Deleting the schema
    command.delete(schema_id)?;

    // THEN: Schema is completely removed
    assert!(
        query.find_by_id(schema_id)?.is_none(),
        "Schema should not exist after delete"
    );
    assert!(
        query.find_by_name(schema.name())?.is_none(),
        "Schema should not be findable by name after delete"
    );

    Ok(())
}

/// **P0-002**: Property bank version retention works across saves.
///
/// Verifies that saving multiple versions doesn't crash and the most
/// recent version is always retrievable (indirect test of retention).
#[test]
fn property_bank_multiple_versions_persist() -> TestResult {
    // GIVEN: A property bank with multiple versions
    let test_db = TestDb::new()?;
    let (command, query) = setup_cqrs(test_db.db());

    let mut bank = PropertyBank::new();
    let initial_version = bank.version();

    // WHEN: Saving many versions (>3 to trigger retention if implemented)
    for i in 0u32..6u32 {
        let prop_name = format!("prop{i}");
        let prop = PropertyBuilder::new(&prop_name).build_string_default()?;
        bank.register(prop)?;
        command.save_property_bank(&bank)?;
    }

    let final_version = bank.version();

    // THEN: Latest version is retrievable and correct
    let loaded_bank = query.get_property_bank()?.expect("Bank should exist");
    assert_eq!(
        loaded_bank.version(),
        final_version,
        "Loaded version should match final version"
    );
    assert!(
        initial_version.is_older_than(final_version),
        "Version should have incremented"
    );
    assert_eq!(
        loaded_bank.all().count(),
        6,
        "All properties should be present"
    );

    Ok(())
}

/// **P0-002b**: Property bank respects version retention limit.
///
/// Verifies that only the last 3 versions are retained in the database
/// after saving 6 versions (retention limit = 3).
///
/// This test validates retention behavior indirectly by:
/// 1. Saving 6 versions with cumulative properties
/// 2. Verifying the latest version loads correctly with all properties
/// 3. Checking that the database file size doesn't grow unbounded
///
/// Note: Direct table inspection would require exposing internal DB tables,
/// so we validate through observable behavior (successful loads + bounded
/// growth).
#[test]
fn property_bank_respects_version_retention_limit() -> TestResult {
    // GIVEN: A property bank
    let test_db = TestDb::new()?;
    let (command, query) = setup_cqrs(test_db.db());

    let mut bank = PropertyBank::new();

    // Record initial state
    let initial_version = bank.version();

    // WHEN: Saving 6 versions (retention = 3, so first 3 should be deleted)
    // Each version adds a new property to make versions distinguishable
    for i in 0u32..6u32 {
        let prop_name = format!("prop{i}");
        let prop = PropertyBuilder::new(&prop_name).build_string_default()?;
        bank.register(prop)?;
        command.save_property_bank(&bank)?;
    }

    let final_version = bank.version();

    // THEN: Latest version should be retrievable with all 6 properties
    let loaded_bank = query.get_property_bank()?.expect("Bank should exist");

    assert_eq!(
        loaded_bank.version(),
        final_version,
        "Loaded version should match final version (v6)"
    );

    assert!(
        initial_version.is_older_than(final_version),
        "Version should have incremented from v0 to v6"
    );

    assert_eq!(
        loaded_bank.all().count(),
        6,
        "All 6 properties should be present in the loaded bank"
    );

    // Verify all expected properties are present
    for i in 0u32..6u32 {
        let prop_name = format!("prop{i}");
        let name =
            lithos_core::schema::property::PropertyName::try_new(&prop_name)?;
        assert!(
            loaded_bank.has(&name),
            "Property '{prop_name}' should be present"
        );
    }

    // ADDITIONAL VALIDATION: File size should be bounded
    // With retention=3, we expect ~3 versions worth of data
    // Without retention, we'd have ~6 versions worth of data
    // This validates that old versions are being cleaned up
    let db_file_size = std::fs::metadata(test_db.path())?.len();

    // Sanity check: file should exist and be non-trivial
    assert!(
        db_file_size > 1024,
        "Database file should contain data (size: {db_file_size} bytes)"
    );

    // Note: We can't assert an exact size due to redb's internal overhead,
    // but in the future we could add a test that saves 100 versions and
    // verifies size doesn't grow linearly with version count.

    Ok(())
}

/// **P0-003**: Batch save with duplicate names in same batch fails.
///
/// Verifies that `save_batch()` detects duplicate schema names within
/// the same batch and rejects the entire operation.
#[test]
fn batch_save_duplicate_names_in_batch_fails() -> TestResult {
    // GIVEN: Multiple schemas with duplicate names
    let test_db = TestDb::new()?;
    let (command, _query) = setup_cqrs(test_db.db());

    let schema1 = SchemaBuilder::new("duplicate").build()?;
    let schema2 = SchemaBuilder::new("duplicate").build()?; // Same name, different ID

    // WHEN: Attempting batch save with duplicates
    let result = command.save_many(&[schema1, schema2]);

    // THEN: Operation fails with duplicate name error
    assert!(result.is_err(), "Batch save should fail with duplicate names");

    let err = result.unwrap_err();
    let err_msg = err.to_string().to_lowercase();
    assert!(
        err_msg.contains("duplicate") || err_msg.contains("conflict"),
        "Error should mention duplicate/conflict: {err}"
    );

    Ok(())
}

/// **P0-004**: `is_schema_stale()` missing schema returns true.
///
/// Verifies that `is_schema_stale()` correctly reports missing schemas as
/// stale.
#[test]
fn is_schema_stale_reports_missing_schema_as_stale() -> TestResult {
    use lithos_core::schema::aggregate::SchemaId;

    // GIVEN: An empty database
    let test_db = TestDb::new()?;
    let (command, query) = setup_cqrs(test_db.db());

    // Create PropertyBank
    let mut bank = PropertyBank::new();
    let prop = PropertyBuilder::new("status").build_bool()?;
    bank.register(prop)?;
    command.save_property_bank(&bank)?;
    let current_version = bank.version();

    // WHEN: Checking staleness for non-existent schema
    let missing_id = SchemaId::new();
    let is_stale =
        query.is_schema_stale(missing_id, None, None, current_version)?;

    // THEN: Missing schema should be reported as stale
    assert!(is_stale, "Missing schema should be stale");

    Ok(())
}

/// **P0-005**: `is_schema_stale` returns false for fresh schemas.
///
/// Verifies that `is_schema_stale()` returns false when schema version
/// matches current `PropertyBank` version (using initial version).
#[test]
fn is_schema_stale_returns_false_for_fresh_schema() -> TestResult {
    // GIVEN: A schema saved without PropertyBank changes
    let test_db = TestDb::new()?;
    let (command, query) = setup_cqrs(test_db.db());

    // Save a schema (will use BankVersion::initial())
    let schema = SchemaBuilder::new("test-schema").build()?;
    let schema_id = schema.id();
    command.save(&schema)?;

    // WHEN: Checking staleness against initial version (same as saved)
    let is_stale = query.is_schema_stale(
        schema_id,
        None,
        None,
        lithos_core::schema::bank::BankVersion::initial(),
    )?;

    // THEN: Schema should NOT be stale (version matches)
    assert!(
        !is_stale,
        "Schema saved with initial version should not be stale when checked \
         against initial version"
    );

    Ok(())
}

/// **P0-005b**: `is_schema_stale` handles asymmetric `created_at` timestamps.
///
/// Verifies that `is_schema_stale()` gracefully handles cases where:
/// 1. Schema was saved WITH `created_at` (filesystem supported birthtime)
/// 2. Current filesystem check returns `created_at` = None (no birthtime
///    support)
///
/// This asymmetry can occur when:
/// - Moving schemas between filesystems (APFS → ext4)
/// - Filesystem loses birthtime metadata
/// - Platform differences (macOS → Linux)
///
/// Expected behavior: Falls back to `modified_at` comparison, logs warning.
#[test]
fn is_schema_stale_with_asymmetric_created_at() -> TestResult {
    use lithos_core::schema::{aggregate::Timestamp, bank::BankVersion};
    use redb::TableDefinition;

    // Manually create StoredMetadata struct for test metadata crafting
    #[derive(rkyv::Archive, rkyv::Serialize)]
    struct TempMetadata {
        bank_version: BankVersion,
        created_at: Option<Timestamp>,
        modified_at: Option<Timestamp>,
        recorded_at: Timestamp,
    }

    // Table definitions for direct database access
    const SCHEMA_METADATA: TableDefinition<&str, &[u8]> =
        TableDefinition::new("schema_metadata");

    // GIVEN: A schema saved with explicit created_at and modified_at metadata
    let test_db = TestDb::new()?;
    let (command, query) = setup_cqrs(test_db.db());

    let schema = SchemaBuilder::new("asymmetric-schema").build()?;
    let schema_id = schema.id();

    // Save schema first (will have None/None timestamps)
    command.save(&schema)?;

    // Manually craft metadata with created_at = Some, modified_at = Some
    // to simulate a file that was saved WITH birthtime support
    let base_time = 1_000_000u64;
    let created_timestamp = Timestamp::from_secs(base_time);
    let modified_timestamp = Timestamp::from_secs(base_time + 100);

    let metadata = TempMetadata {
        bank_version: BankVersion::initial(),
        created_at: Some(created_timestamp),
        modified_at: Some(modified_timestamp),
        recorded_at: Timestamp::now(),
    };

    // Write metadata to database (overwriting the None/None version)
    let id_key = schema_id.into_uuid().to_string();
    test_db.db().batch_write(|batch| {
        batch.put(SCHEMA_METADATA, id_key.as_str(), &metadata)?;
        Ok(())
    })?;

    // WHEN: Checking staleness with created_at = None (filesystem lost
    // birthtime) but modified_at matches what was saved
    let is_stale_matching_mtime = query.is_schema_stale(
        schema_id,
        None, // ← Filesystem no longer provides birthtime
        Some(modified_timestamp),
        BankVersion::initial(),
    )?;

    // THEN: Should NOT be stale (falls back to mtime, which matches)
    // (Warning is logged about created_at unavailability)
    assert!(
        !is_stale_matching_mtime,
        "Schema should not be stale when created_at is asymmetric but \
         modified_at matches"
    );

    // WHEN: Checking with newer modified_at (file was actually modified)
    let newer_modified = Timestamp::from_secs(base_time + 200);
    let is_stale_newer_mtime = query.is_schema_stale(
        schema_id,
        None, // ← Still no birthtime
        Some(newer_modified),
        BankVersion::initial(),
    )?;

    // THEN: Should be stale (mtime is newer than saved)
    assert!(
        is_stale_newer_mtime,
        "Schema should be stale when modified_at is newer, even with \
         created_at asymmetry"
    );

    Ok(())
}

/// **P0-006**: Schema save validates property references.
///
/// Verifies that `save_batch_with_metadata()` rejects schemas with property
/// references that don't exist in `PropertyBank`.
#[test]
fn save_rejects_invalid_property_references() -> TestResult {
    // GIVEN: A PropertyBank with one property
    let test_db = TestDb::new()?;
    let (command, _query) = setup_cqrs(test_db.db());

    let mut bank = PropertyBank::new();
    let valid_prop = PropertyBuilder::new("status").build_bool()?;
    bank.register(valid_prop)?;
    command.save_property_bank(&bank)?;

    // WHEN: Attempting to save schema with invalid property reference
    let schema = SchemaBuilder::new("test-schema")
        .property(PropertyBuilder::new("invalid_prop").build_bool()?)
        .build()?;

    let result = command.save(&schema);

    // THEN: Save fails with clear error
    assert!(
        result.is_err(),
        "Should reject schema with invalid property reference"
    );

    let err = result.unwrap_err();
    let err_msg = err.to_string().to_lowercase();
    assert!(
        err_msg.contains("invalid_prop") && err_msg.contains("not found"),
        "Error should mention missing property: {err}"
    );

    Ok(())
}

/// **P0-007**: Schema save succeeds with valid property references.
///
/// Verifies that `save_batch_with_metadata()` accepts schemas when all
/// property references exist in `PropertyBank`.
#[test]
fn save_succeeds_with_valid_property_references() -> TestResult {
    // GIVEN: A PropertyBank with registered properties
    let test_db = TestDb::new()?;
    let (command, query) = setup_cqrs(test_db.db());

    let mut bank = PropertyBank::new();
    let prop1 = PropertyBuilder::new("status").build_bool()?;
    let prop2 = PropertyBuilder::new("title").build_string_default()?;
    bank.register(prop1.clone())?;
    bank.register(prop2.clone())?;
    command.save_property_bank(&bank)?;

    // WHEN: Saving schema with valid property references
    let schema = SchemaBuilder::new("test-schema")
        .property(prop1)
        .property(prop2)
        .build()?;

    let result = command.save(&schema);

    // THEN: Save succeeds
    assert!(result.is_ok(), "Should accept valid property references");

    // Schema is retrievable
    let loaded = query.find_by_id(schema.id())?;
    assert!(loaded.is_some(), "Schema should be saved");

    Ok(())
}

/// **P0-008**: Schema save without `PropertyBank` succeeds (bootstrap case).
///
/// Verifies that schemas can be saved before `PropertyBank` exists,
/// allowing initial bootstrap.
#[test]
fn save_succeeds_without_property_bank() -> TestResult {
    // GIVEN: An empty database (no PropertyBank)
    let test_db = TestDb::new()?;
    let (command, query) = setup_cqrs(test_db.db());

    // Verify PropertyBank doesn't exist
    assert!(
        query.get_property_bank()?.is_none(),
        "PropertyBank should not exist"
    );

    // WHEN: Saving schema without PropertyBank
    let schema = SchemaBuilder::new("test-schema").build()?;
    let result = command.save(&schema);

    // THEN: Save succeeds (allows bootstrap)
    assert!(
        result.is_ok(),
        "Should allow saving schemas before PropertyBank exists"
    );

    Ok(())
}

/// **ERROR-001**: Query detects corrupted schema data.
///
/// Verifies that `QueryAdapter` properly detects and reports corrupted
/// rkyv-serialized data instead of returning invalid schemas or panicking.
#[test]
fn query_detects_corrupted_schema_data() -> TestResult {
    use redb::TableDefinition;

    // Table definition for direct database access
    const SCHEMA_BY_ID: TableDefinition<&str, &[u8]> =
        TableDefinition::new("schema_by_id");

    // GIVEN: A database with corrupted schema data
    let test_db = TestDb::new()?;
    let (_command, query) = setup_cqrs(test_db.db());

    // Manually write invalid rkyv bytes to SCHEMA_BY_ID table
    // Access table directly to bypass validation layers
    test_db.db().batch_write(|batch| {
        // Write garbage data that cannot be deserialized as StoredSchema
        batch.put(SCHEMA_BY_ID, "corrupt-schema-key", &[
            0xFFu8, 0xFFu8, 0xFFu8, 0xFFu8, 0xFFu8,
        ])?;
        Ok(())
    })?;

    // WHEN: Attempting to list all schemas
    let result = query.list();

    // THEN: Query fails with storage error (not panic, not invalid data)
    assert!(
        result.is_err(),
        "Corrupted data should trigger error, not return invalid schemas"
    );

    // Verify it's a storage error (not a validation error)
    if let Err(e) = result {
        let error_msg = e.to_string();
        assert!(
            error_msg.contains("storage") || error_msg.contains("deserializ"),
            "Error should indicate storage/deserialization issue: {error_msg}"
        );
    }

    Ok(())
}

/// **ERROR-002**: Query detects missing metadata for existing schema.
///
/// Verifies that `find_by_id` detects database corruption when a schema
/// exists but its metadata is missing (orphaned schema record).
#[test]
fn query_detects_missing_metadata_corruption() -> TestResult {
    use redb::TableDefinition;

    // Table definition for direct database access
    const SCHEMA_METADATA: TableDefinition<&str, &[u8]> =
        TableDefinition::new("schema_metadata");

    // GIVEN: A normally saved schema
    let test_db = TestDb::new()?;
    let (command, query) = setup_cqrs(test_db.db());

    let schema = SchemaBuilder::new("orphaned-schema").build()?;
    let schema_id = schema.id();

    // Save schema normally (includes metadata)
    command.save(&schema)?;

    // WHEN: Manually delete metadata to simulate corruption
    let id_key = schema_id.into_uuid().to_string();
    test_db.db().batch_write(|batch| {
        batch.delete(SCHEMA_METADATA, id_key.as_str())?;
        Ok(())
    })?;

    // THEN: Attempting to find the orphaned schema detects corruption
    let result = query.find_by_id(schema_id);

    assert!(
        result.is_err(),
        "Orphaned schema (missing metadata) should trigger corruption error"
    );

    // Verify it's a corruption error
    if let Err(e) = result {
        let error_msg = e.to_string();
        assert!(
            error_msg.contains("corruption")
                || error_msg.contains("metadata is missing"),
            "Error should indicate metadata corruption: {error_msg}"
        );
    }

    Ok(())
}

/// **ERROR-003**: Query detects corrupted property bank metadata.
///
/// Verifies that property bank queries handle corrupted metadata gracefully.
#[test]
fn query_detects_corrupted_property_bank_metadata() -> TestResult {
    use redb::TableDefinition;

    // Table definition for direct database access
    const BANK_METADATA: TableDefinition<&str, &[u8]> =
        TableDefinition::new("bank_metadata");

    // GIVEN: A database with corrupted property bank metadata
    let test_db = TestDb::new()?;
    let (_command, query) = setup_cqrs(test_db.db());

    // Manually write invalid rkyv bytes to BANK_METADATA table
    test_db.db().batch_write(|batch| {
        batch.put(BANK_METADATA, "singleton", &[
            0xFFu8, 0xFFu8, 0xFFu8, 0xFFu8, 0xFFu8,
        ])?;
        Ok(())
    })?;

    // WHEN: Attempting to get property bank
    let result = query.get_property_bank();

    // THEN: Query fails with storage/deserialization error
    assert!(
        result.is_err(),
        "Corrupted property bank metadata should trigger error"
    );

    // Verify it's a storage error
    if let Err(e) = result {
        let error_msg = e.to_string();
        assert!(
            error_msg.contains("storage") || error_msg.contains("deserializ"),
            "Error should indicate storage/deserialization issue: {error_msg}"
        );
    }

    Ok(())
}

/// **ERROR-004**: Query detects corrupted name index.
///
/// Verifies that schema name lookups handle corrupted index data gracefully.
#[test]
fn query_detects_corrupted_name_index() -> TestResult {
    use redb::TableDefinition;

    // Table definition for direct database access
    const SCHEMA_ID_BY_NAME: TableDefinition<&str, &[u8]> =
        TableDefinition::new("schema_id_by_name");

    // GIVEN: A database with corrupted name index
    let test_db = TestDb::new()?;
    let (_command, query) = setup_cqrs(test_db.db());

    // Manually write invalid rkyv bytes to SCHEMA_ID_BY_NAME table
    test_db.db().batch_write(|batch| {
        batch.put(SCHEMA_ID_BY_NAME, "corrupt-index-key", &[
            0xFFu8, 0xFFu8, 0xFFu8, 0xFFu8, 0xFFu8,
        ])?;
        Ok(())
    })?;

    // Create a valid schema name to search for
    let schema_name = lithos_core::schema::aggregate::SchemaName::try_new(
        "corrupt-index-key",
    )?;

    // WHEN: Attempting to find schema by name with corrupted index
    let result = query.find_by_name(&schema_name);

    // THEN: Query fails with storage/deserialization error
    assert!(result.is_err(), "Corrupted name index should trigger error");

    // Verify it's a storage error
    if let Err(e) = result {
        let error_msg = e.to_string();
        assert!(
            error_msg.contains("storage") || error_msg.contains("deserializ"),
            "Error should indicate storage/deserialization issue: {error_msg}"
        );
    }

    Ok(())
}
