//! Verification tests for common test infrastructure.
//!
//! These tests ensure that the shared test utilities work correctly before
//! using them in actual integration tests.

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
use lithos_core::schema::property::{Cardinality, Multiplicity};

/// Verify `TestDb` creates a working database.
#[test]
fn db_creates_working_database() -> TestResult {
    let test_db = TestDb::new()?;
    let _db = test_db.db();
    assert!(test_db.path().exists(), "Database file should exist");
    Ok(())
}

/// Verify `setup_cqrs` creates valid CQRS adapters.
#[test]
fn setup_cqrs_creates_valid_adapters() -> TestResult {
    let test_db = TestDb::new()?;
    let (_command, _query) = setup_cqrs(test_db.db());
    Ok(())
}

/// Verify `PropertyBuilder` creates valid properties.
#[test]
fn property_builder_creates_bool_property() -> TestResult {
    let prop = PropertyBuilder::new("is_active")
        .cardinality(Cardinality::Optional)
        .build_bool()?;

    assert_eq!(prop.name().as_str(), "is_active");
    assert_eq!(prop.cardinality(), Cardinality::Optional);
    assert_eq!(prop.multiplicity(), Multiplicity::Single);
    Ok(())
}

/// Verify `PropertyBuilder` creates string properties.
#[test]
fn property_builder_creates_string_property() -> TestResult {
    let prop = PropertyBuilder::new("title")
        .multiplicity(Multiplicity::Many)
        .build_string_default()?;

    assert_eq!(prop.name().as_str(), "title");
    assert_eq!(prop.cardinality(), Cardinality::Required);
    assert_eq!(prop.multiplicity(), Multiplicity::Many);
    Ok(())
}

/// Verify helper functions work.
#[test]
fn helper_functions_create_properties() -> TestResult {
    let bool_prop = bool_property("active")?;
    assert_eq!(bool_prop.name().as_str(), "active");

    let string_prop = string_property("name")?;
    assert_eq!(string_prop.name().as_str(), "name");

    Ok(())
}

/// Verify `SchemaBuilder` creates valid schemas.
#[test]
fn schema_builder_creates_schema_with_properties() -> TestResult {
    let prop1 = bool_property("is_done")?;
    let prop2 = string_property("title")?;

    let schema =
        SchemaBuilder::new("task").property(prop1).property(prop2).build()?;

    assert_eq!(schema.name().as_str(), "task");
    assert_eq!(schema.properties().count(), 2);
    Ok(())
}

/// Verify `SchemaBuilder` with properties vec.
#[test]
fn schema_builder_accepts_properties_vec() -> TestResult {
    let props = vec![bool_property("flag")?, string_property("text")?];

    let schema = SchemaBuilder::new("test").properties(props).build()?;

    assert_eq!(schema.properties().count(), 2);
    Ok(())
}

/// Verify assertion helper: `assert_has_property`.
#[test]
fn assert_has_property_works() -> TestResult {
    let schema =
        SchemaBuilder::new("test").property(bool_property("flag")?).build()?;

    assert_has_property(&schema, "flag", "Test context");
    Ok(())
}

/// Verify assertion helper: `assert_not_has_property`.
#[test]
fn assert_not_has_property_works() -> TestResult {
    let schema =
        SchemaBuilder::new("test").property(bool_property("flag")?).build()?;

    assert_not_has_property(&schema, "nonexistent", "Test context");
    Ok(())
}

/// Verify assertion helper: `assert_properties_sorted`.
#[test]
fn assert_properties_sorted_works() -> TestResult {
    // BTreeMap guarantees sorted order by PropertyName
    let schema = SchemaBuilder::new("test")
        .property(string_property("zebra")?)
        .property(bool_property("alpha")?)
        .build()?;

    // Should be sorted: alpha, zebra
    assert_properties_sorted(&schema, "Properties should be sorted");
    Ok(())
}
