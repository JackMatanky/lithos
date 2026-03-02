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

    command.save_one(&schema)?;
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
    let result = command.save_batch(&[schema1, schema2]);

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
    command.save_one(&schema)?;

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
