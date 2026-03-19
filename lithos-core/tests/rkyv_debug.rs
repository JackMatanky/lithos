//! Debug test to investigate rkyv corruption bug.
//!
//! This test isolates the Schema serialization issue to determine root cause.

#![expect(
    clippy::panic_in_result_fn,
    reason = "Test code uses assertions which panic on failure."
)]
#![expect(
    clippy::similar_names,
    reason = "Test code intentionally uses prop1/props1 naming for parallel \
              construction"
)]
#![expect(
    clippy::print_stdout,
    reason = "Debug test - println! used for visibility"
)]
#![expect(
    clippy::doc_markdown,
    reason = "Test documentation - backticks not critical"
)]
#![expect(
    clippy::doc_paragraphs_missing_punctuation,
    reason = "Test documentation - terminal punctuation not critical"
)]
#![expect(
    clippy::uninlined_format_args,
    reason = "Test code - format string style not critical"
)]
#![expect(
    clippy::redundant_test_prefix,
    reason = "Test functions - prefix makes purpose explicit"
)]
#![expect(clippy::useless_vec, reason = "Test code - vec! used for clarity")]
#![expect(
    clippy::tests_outside_test_module,
    reason = "Integration tests are top-level by default."
)]

mod common;

use std::collections::HashMap;

use common::*;
use lithos_core::schema::{
    aggregate::{Schema, SchemaId, SchemaName},
    property::PropertyName,
    storage::Repository as _,
};

/// Test 1: Can we save and load 2 SIMPLE schemas (no properties)?
#[test]
fn two_simple_schemas_no_properties() -> TestResult {
    let test_db = TestDb::new()?;
    let repository = setup_repository(test_db.db());

    // Create 2 schemas with NO properties
    let schema1 = Schema::new(
        SchemaId::new(),
        SchemaName::try_new("schema1")?,
        None,
        vec![],
        HashMap::new(), // Empty properties
    );
    let id1 = *schema1.id();

    let schema2 = Schema::new(
        SchemaId::new(),
        SchemaName::try_new("schema2")?,
        None,
        vec![],
        HashMap::new(), // Empty properties
    );
    let id2 = *schema2.id();

    // Save both in batch
    repository.save_schemas(&[schema1, schema2])?;

    // Try to load first schema
    let loaded1 = repository.find_schema_by_id(id1)?;
    assert!(loaded1.is_some(), "Schema 1 should exist");

    // Try to load second schema
    let loaded2 = repository.find_schema_by_id(id2)?;
    assert!(loaded2.is_some(), "Schema 2 should exist");

    Ok(())
}

/// Test 2: Can we save and load 2 schemas with SINGLE property each?
#[test]
fn two_schemas_with_one_property() -> TestResult {
    let test_db = TestDb::new()?;
    let repository = setup_repository(test_db.db());

    // Create schema 1 with 1 property
    let prop1 = PropertyBuilder::new("title").build_string_default()?;
    let mut props1 = HashMap::new();
    props1.insert(prop1.name().clone(), prop1);

    let schema1 = Schema::new(
        SchemaId::new(),
        SchemaName::try_new("schema1")?,
        None,
        vec![],
        props1,
    );
    let id1 = *schema1.id();

    // Create schema 2 with DIFFERENT property
    let prop2 = PropertyBuilder::new("content").build_string_default()?;
    let mut props2 = HashMap::new();
    props2.insert(prop2.name().clone(), prop2);

    let schema2 = Schema::new(
        SchemaId::new(),
        SchemaName::try_new("schema2")?,
        None,
        vec![],
        props2,
    );
    let id2 = *schema2.id();

    // Save both in batch
    repository.save_schemas(&[schema1, schema2])?;

    // Try to load first schema
    let loaded1 = repository.find_schema_by_id(id1)?;
    assert!(loaded1.is_some(), "Schema 1 should exist");

    // Try to load second schema
    let loaded2 = repository.find_schema_by_id(id2)?;
    assert!(loaded2.is_some(), "Schema 2 should exist");

    Ok(())
}

/// Test 3: Save schemas SEPARATELY (not in batch)
#[test]
fn two_schemas_saved_separately() -> TestResult {
    let test_db = TestDb::new()?;
    let repository = setup_repository(test_db.db());

    // Create schema 1 with property
    let prop1 = PropertyBuilder::new("title").build_string_default()?;
    let mut props1 = HashMap::new();
    props1.insert(prop1.name().clone(), prop1);

    let schema1 = Schema::new(
        SchemaId::new(),
        SchemaName::try_new("schema1")?,
        None,
        vec![],
        props1,
    );
    let id1 = *schema1.id();

    // Save schema 1 FIRST
    repository.save_schemas(&[schema1])?;

    // Verify schema 1 loads
    let loaded1 = repository.find_schema_by_id(id1)?;
    assert!(loaded1.is_some(), "Schema 1 should exist after first save");

    // Create schema 2 with property
    let prop2 = PropertyBuilder::new("content").build_string_default()?;
    let mut props2 = HashMap::new();
    props2.insert(prop2.name().clone(), prop2);

    let schema2 = Schema::new(
        SchemaId::new(),
        SchemaName::try_new("schema2")?,
        None,
        vec![],
        props2,
    );
    let id2 = *schema2.id();

    // Save schema 2 SECOND
    repository.save_schemas(&[schema2])?;

    // Try to load FIRST schema again (this is where corruption happens)
    let loaded1_again = repository.find_schema_by_id(id1)?;
    assert!(
        loaded1_again.is_some(),
        "Schema 1 should still exist after saving schema 2"
    );

    // Try to load second schema
    let loaded2 = repository.find_schema_by_id(id2)?;
    assert!(loaded2.is_some(), "Schema 2 should exist");

    Ok(())
}

/// Test 4: Direct rkyv serialization test (outside redb)
#[test]
fn direct_rkyv_serialization_test() -> TestResult {
    use rkyv::access;

    // Create 2 schemas
    let prop1 = PropertyBuilder::new("title").build_string_default()?;
    let mut props1 = HashMap::new();
    props1.insert(prop1.name().clone(), prop1);

    let schema1 = Schema::new(
        SchemaId::new(),
        SchemaName::try_new("schema1")?,
        None,
        vec![],
        props1,
    );

    let prop2 = PropertyBuilder::new("content").build_string_default()?;
    let mut props2 = HashMap::new();
    props2.insert(prop2.name().clone(), prop2);

    let schema2 = Schema::new(
        SchemaId::new(),
        SchemaName::try_new("schema2")?,
        None,
        vec![],
        props2,
    );

    // Serialize both independently
    let bytes1 = rkyv::to_bytes::<rkyv::rancor::Error>(&schema1)
        .map_err(|e| format!("Schema 1 serialization failed: {}", e))?;
    println!("Schema 1: {} bytes", bytes1.len());

    let bytes2 = rkyv::to_bytes::<rkyv::rancor::Error>(&schema2)
        .map_err(|e| format!("Schema 2 serialization failed: {}", e))?;
    println!("Schema 2: {} bytes", bytes2.len());

    // Try to deserialize both
    let _archived1 =
        access::<rkyv::Archived<Schema>, rkyv::rancor::Error>(&bytes1)
            .map_err(|e| format!("Schema 1 deserialization failed: {}", e))?;
    println!("Schema 1 deserialized successfully");

    let _archived2 =
        access::<rkyv::Archived<Schema>, rkyv::rancor::Error>(&bytes2)
            .map_err(|e| format!("Schema 2 deserialization failed: {}", e))?;
    println!("Schema 2 deserialized successfully");

    Ok(())
}

/// Test 5: Check if PropertyName HashMap key is the issue
#[test]
fn test_property_name_as_hashmap_key() -> TestResult {
    use rkyv::access;

    // Create HashMap with PropertyName keys
    let mut map = HashMap::new();
    let name1 = PropertyName::try_new("prop1")?;
    let prop1 = PropertyBuilder::new("prop1").build_string_default()?;
    map.insert(name1, prop1);

    // Serialize
    let bytes = rkyv::to_bytes::<rkyv::rancor::Error>(&map)
        .map_err(|e| format!("HashMap serialization failed: {}", e))?;
    println!("HashMap: {} bytes", bytes.len());

    // Deserialize
    let archived = access::<
        rkyv::Archived<
            HashMap<PropertyName, lithos_core::schema::property::Property>,
        >,
        rkyv::rancor::Error,
    >(&bytes)
    .map_err(|e| format!("HashMap deserialization failed: {}", e))?;
    println!("HashMap deserialized: {} entries", archived.len());

    Ok(())
}

/// Test 6: Full deserialization (not just access) - THIS MIGHT FAIL
#[test]
fn test_full_deserialization() -> TestResult {
    use rkyv::{access, deserialize};

    // Create 2 schemas
    let prop1 = PropertyBuilder::new("title").build_string_default()?;
    let mut props1 = HashMap::new();
    props1.insert(prop1.name().clone(), prop1);

    let schema1 = Schema::new(
        SchemaId::new(),
        SchemaName::try_new("schema1")?,
        None,
        vec![],
        props1,
    );

    let prop2 = PropertyBuilder::new("content").build_string_default()?;
    let mut props2 = HashMap::new();
    props2.insert(prop2.name().clone(), prop2);

    let schema2 = Schema::new(
        SchemaId::new(),
        SchemaName::try_new("schema2")?,
        None,
        vec![],
        props2,
    );

    // Serialize both
    let bytes1 = rkyv::to_bytes::<rkyv::rancor::Error>(&schema1)?;
    let bytes2 = rkyv::to_bytes::<rkyv::rancor::Error>(&schema2)?;

    // Try FULL DESERIALIZATION (not just access)
    let archived1 =
        access::<rkyv::Archived<Schema>, rkyv::rancor::Error>(&bytes1)?;
    let _deserialized1 = deserialize::<Schema, rkyv::rancor::Error>(archived1)
        .map_err(|e| format!("Schema 1 full deserialization failed: {}", e))?;
    println!("Schema 1 fully deserialized successfully");

    let archived2 =
        access::<rkyv::Archived<Schema>, rkyv::rancor::Error>(&bytes2)?;
    let _deserialized2 = deserialize::<Schema, rkyv::rancor::Error>(archived2)
        .map_err(|e| format!("Schema 2 full deserialization failed: {}", e))?;
    println!("Schema 2 fully deserialized successfully");

    Ok(())
}

/// Test 7: Replicate list_schemas behavior - iterate and deserialize ALL
#[test]
fn test_list_schemas_behavior() -> TestResult {
    let test_db = TestDb::new()?;
    let repository = setup_repository(test_db.db());

    // Create 2 schemas with properties
    let prop1 = PropertyBuilder::new("title").build_string_default()?;
    let mut props1 = HashMap::new();
    props1.insert(prop1.name().clone(), prop1);

    let schema1 = Schema::new(
        SchemaId::new(),
        SchemaName::try_new("schema1")?,
        None,
        vec![],
        props1,
    );

    let prop2 = PropertyBuilder::new("content").build_string_default()?;
    let mut props2 = HashMap::new();
    props2.insert(prop2.name().clone(), prop2);

    let schema2 = Schema::new(
        SchemaId::new(),
        SchemaName::try_new("schema2")?,
        None,
        vec![],
        props2,
    );

    // Save both
    repository.save_schemas(&[schema1, schema2])?;

    // NOW TRY TO LIST (this is where the bug might be)
    let all_schemas = repository
        .list_schemas()
        .map_err(|e| format!("list_schemas failed: {}", e))?;

    println!("Successfully listed {} schemas", all_schemas.len());
    assert_eq!(all_schemas.len(), 2, "Should have 2 schemas");

    Ok(())
}

/// Test 8: Deserialize both schemas sequentially (not in parallel)
#[test]
fn test_sequential_deserialization() -> TestResult {
    use rkyv::{access, deserialize};

    // Create 2 schemas
    let prop1 = PropertyBuilder::new("title").build_string_default()?;
    let mut props1 = HashMap::new();
    props1.insert(prop1.name().clone(), prop1);
    let schema1 = Schema::new(
        SchemaId::new(),
        SchemaName::try_new("schema1")?,
        None,
        vec![],
        props1,
    );

    let prop2 = PropertyBuilder::new("content").build_string_default()?;
    let mut props2 = HashMap::new();
    props2.insert(prop2.name().clone(), prop2);
    let schema2 = Schema::new(
        SchemaId::new(),
        SchemaName::try_new("schema2")?,
        None,
        vec![],
        props2,
    );

    // Serialize both
    let bytes1 = rkyv::to_bytes::<rkyv::rancor::Error>(&schema1)?;
    let bytes2 = rkyv::to_bytes::<rkyv::rancor::Error>(&schema2)?;

    // Put them in a vec and iterate (simulating table scan)
    let all_bytes = vec![bytes1, bytes2];
    let mut results = Vec::new();

    for (i, bytes) in all_bytes.iter().enumerate() {
        println!("Deserializing schema {}", i + 1);

        let archived =
            access::<rkyv::Archived<Schema>, rkyv::rancor::Error>(bytes)
                .map_err(|e| {
                    format!("Schema {} access failed: {}", i + 1, e)
                })?;

        let deserialized = deserialize::<Schema, rkyv::rancor::Error>(archived)
            .map_err(|e| {
                format!("Schema {} deserialization failed: {}", i + 1, e)
            })?;

        results.push(deserialized);
    }

    println!("Successfully deserialized {} schemas", results.len());
    assert_eq!(results.len(), 2);

    Ok(())
}
