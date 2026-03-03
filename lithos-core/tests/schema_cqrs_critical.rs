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
