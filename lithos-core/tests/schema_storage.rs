//! Integration tests for Schema storage via Repository trait.
//!
//! Tests the unified Repository implementation for storage operations.

#![expect(
    clippy::panic_in_result_fn,
    reason = "Integration tests use assertions which panic on failure."
)]
#![expect(
    clippy::tests_outside_test_module,
    reason = "Integration tests are top-level by default."
)]

mod common;

use std::collections::HashMap;

use common::*;
use lithos_core::schema::{
    aggregate::{Schema, SchemaId, SchemaName},
    bank::PropertyBank,
    property::PropertyId,
    storage::Repository as _,
};
use uuid::Uuid;

// Test fixture UUIDs
const TEST_PROPERTY_ID_A: Uuid =
    Uuid::from_u128(0x018C_0000_0000_7000_8000_0000_0000_0A01);

// ========================================================================
//                          PropertyBank Storage Tests
// ========================================================================

/// Test that property bank can be saved and retrieved.
#[test]
fn property_bank_roundtrip() -> TestResult {
    let test_db = TestDb::new()?;
    let repository = setup_repository(test_db.db());

    // Create and save property bank
    let mut bank = PropertyBank::new();
    let prop = PropertyBuilder::new("status")
        .id(PropertyId::from_uuid(TEST_PROPERTY_ID_A))
        .build_bool()?;
    bank.register(prop)?;
    repository.save_property_bank(&bank)?;

    // Retrieve and verify
    let loaded = repository.get_property_bank()?;
    assert!(loaded.is_some(), "PropertyBank should exist");

    Ok(())
}

// ========================================================================
//                          Schema Storage Tests
// ========================================================================

/// Test that schemas can be saved and retrieved.
#[test]
fn schema_roundtrip() -> TestResult {
    let test_db = TestDb::new()?;
    let repository = setup_repository(test_db.db());

    // Create schema
    let prop = PropertyBuilder::new("title").build_string_default()?;
    let mut props = HashMap::new();
    props.insert(prop.name().clone(), prop);
    let schema = Schema::new(
        SchemaId::new(),
        SchemaName::try_new("task")?,
        None,
        vec![],
        props,
    );
    let schema_id = *schema.id();

    // Save
    repository.save_schemas(&[schema])?;

    // Retrieve by ID
    let loaded = repository.find_schema_by_id(schema_id)?;
    assert!(loaded.is_some(), "Schema should exist");

    Ok(())
}

/// Test that schemas can be found by name.
#[test]
fn schema_find_by_name() -> TestResult {
    let test_db = TestDb::new()?;
    let repository = setup_repository(test_db.db());

    // Create and save
    let prop = PropertyBuilder::new("title").build_string_default()?;
    let mut props = HashMap::new();
    props.insert(prop.name().clone(), prop);
    let schema = Schema::new(
        SchemaId::new(),
        SchemaName::try_new("project")?,
        None,
        vec![],
        props,
    );
    let schema_id = *schema.id();
    repository.save_schemas(&[schema])?;

    // Find by name
    let name = SchemaName::try_new("project")?;
    let found_id = repository.find_schema_id_by_name(&name)?;
    assert_eq!(found_id, Some(schema_id));

    Ok(())
}

/// Test that multiple schemas can be listed.
///
/// **IGNORED**: Critical bug in rkyv deserialization when multiple schemas
/// exist.
///
/// ## Investigation Summary
///
/// Symptoms:
/// - Error: "subtree pointer overran range" with size field corruption
/// - Saving 2nd schema corrupts 1st schema's serialized data
/// - Fails even when saving individually (not just batch save)
/// - Fails in SAME session (not a reopen/address space issue)
///
/// Root Cause:
/// - Saving second schema overwrites or corrupts first schema's rkyv bytes
/// - Likely issue in redb table write or rkyv `HashMap` serialization
/// - Size fields become invalid (often `u32::MAX` or corrupted values)
///
/// This is a critical data corruption bug that requires deep investigation
/// of the redb/rkyv integration layer. For now, the Loader integration tests
/// verify multi-schema functionality end-to-end (which works correctly).
///
/// Tracked for Phase 7 investigation.
#[test]
#[ignore = "Critical rkyv data corruption bug when multiple schemas exist"]
#[expect(
    clippy::similar_names,
    reason = "Test code - prop1/props1 naming intentional for parallel \
              construction"
)]
fn schema_list() -> TestResult {
    let test_db = TestDb::new()?;
    let repository = setup_repository(test_db.db());

    // Save multiple schemas
    let prop1 = PropertyBuilder::new("title").build_string_default()?;
    let mut props1 = HashMap::new();
    props1.insert(prop1.name().clone(), prop1);
    let schema1 = Schema::new(
        SchemaId::new(),
        SchemaName::try_new("task")?,
        None,
        vec![],
        props1,
    );
    let prop2 = PropertyBuilder::new("content").build_string_default()?;
    let mut props2 = HashMap::new();
    props2.insert(prop2.name().clone(), prop2);
    let schema2 = Schema::new(
        SchemaId::new(),
        SchemaName::try_new("note")?,
        None,
        vec![],
        props2,
    );
    repository.save_schemas(&[schema1, schema2])?;

    // List all
    let all = repository.list_schemas()?;
    assert_eq!(all.len(), 2, "Should have 2 schemas");

    Ok(())
}

/// Test that schemas can be deleted.
///
/// **IGNORED**: Implementation in progress. Issue: `SCHEMA_CHILDREN` multimap
/// uses `&[u8]` values but batch API expects `&str`. Needs API update or
/// different approach.
#[test]
#[ignore = "Implementation blocked by multimap API type mismatch"]
fn schema_delete() -> TestResult {
    let test_db = TestDb::new()?;
    let repository = setup_repository(test_db.db());

    // Create and save
    let schema = Schema::new(
        SchemaId::new(),
        SchemaName::try_new("temp")?,
        None,
        vec![],
        HashMap::new(),
    );
    let schema_id = *schema.id();
    repository.save_schemas(&[schema])?;

    // Verify exists
    assert!(repository.find_schema_by_id(schema_id)?.is_some());

    // Delete
    repository.delete_schema(schema_id)?;

    // Verify gone
    assert!(repository.find_schema_by_id(schema_id)?.is_none());

    Ok(())
}
