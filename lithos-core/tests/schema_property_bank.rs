//! Integration tests for `PropertyBank` CQRS persistence.
//!
//! Verifies `PropertyBank` singleton identity, persistence, and version
//! tracking across database transactions.

#![expect(
    clippy::panic_in_result_fn,
    reason = "Integration tests use assertions which panic on failure."
)]
#![expect(
    clippy::tests_outside_test_module,
    reason = "Integration tests are top-level by default."
)]

use lithos_core::{
    db::Database,
    schema::{
        RedbSchemaCommand, RedbSchemaQuery,
        adapter::{command::CommandAdapter, query::QueryAdapter},
        bank::{PropertyBank, PropertyBankId},
        property::{
            Cardinality, Multiplicity, Property, PropertyId, PropertyName,
        },
        property_spec::{BoolSpec, PropertySpec, StringSpec},
    },
};
use tempfile::tempdir;
use uuid::Uuid;

type TestResult<T = ()> = Result<T, Box<dyn std::error::Error>>;

const TEST_PROPERTY_ID_A: Uuid =
    Uuid::from_u128(0x018C_0000_0000_7000_8000_0000_0000_0A01);
const TEST_PROPERTY_ID_B: Uuid =
    Uuid::from_u128(0x018C_0000_0000_7000_8000_0000_0000_0A02);

fn setup_db() -> TestResult<(tempfile::TempDir, Database)> {
    let dir = tempdir()?;
    let db_path = dir.path().join("lithos.redb");
    let db = Database::open(&db_path)?;
    Ok((dir, db))
}

fn create_sample_property(
    id: Uuid,
    name: &str,
    spec: PropertySpec,
) -> TestResult<Property> {
    let property = Property::new(
        PropertyId::from_uuid(id),
        PropertyName::new(name)?,
        Cardinality::Required,
        Multiplicity::Single,
        spec,
    );
    Ok(property)
}

/// **3.4-INT-001**: `PropertyBank` singleton identity persists correctly.
/// Priority: P0.
///
/// Verifies that:
/// - `PropertyBank` uses singleton ID
/// - Saved bank has correct singleton identity
/// - Loaded bank preserves singleton ID
#[test]
fn property_bank_singleton_identity_persists() -> TestResult {
    // GIVEN: A database and PropertyBank with singleton ID
    let (_dir, db) = setup_db()?;
    let command = RedbSchemaCommand::new(CommandAdapter::new(&db));
    let query = RedbSchemaQuery::new(QueryAdapter::new(&db));

    let bank = PropertyBank::new();
    assert_eq!(
        bank.id(),
        PropertyBankId::singleton(),
        "New PropertyBank should have singleton ID"
    );

    // WHEN: Saving the PropertyBank
    command.save_property_bank(&bank)?;

    // THEN: Loaded bank has singleton ID
    let loaded = query.find_property_bank()?;
    assert!(loaded.is_some(), "PropertyBank should be retrievable after save");

    let loaded_bank = loaded.expect("just verified bank exists");
    assert_eq!(
        loaded_bank.id(),
        PropertyBankId::singleton(),
        "Loaded PropertyBank should preserve singleton ID"
    );

    Ok(())
}

/// **3.4-INT-002**: `PropertyBank` save creates singleton when missing.
/// Priority: P0.
///
/// Verifies that:
/// - Empty database returns None for `PropertyBank`
/// - First save creates singleton entry
/// - Subsequent find returns the saved bank
#[test]
fn property_bank_save_creates_singleton() -> TestResult {
    // GIVEN: An empty database
    let (_dir, db) = setup_db()?;
    let command = RedbSchemaCommand::new(CommandAdapter::new(&db));
    let query = RedbSchemaQuery::new(QueryAdapter::new(&db));

    let initial = query.find_property_bank()?;
    assert!(initial.is_none(), "Fresh database should have no PropertyBank");

    // WHEN: Saving a new PropertyBank
    let mut bank = PropertyBank::new();
    let prop = create_sample_property(
        TEST_PROPERTY_ID_A,
        "status",
        PropertySpec::Bool(BoolSpec::default()),
    )?;
    bank.register(prop)?;
    command.save_property_bank(&bank)?;

    // THEN: PropertyBank is retrievable
    let loaded = query.find_property_bank()?;
    assert!(loaded.is_some(), "PropertyBank should exist after save");

    let loaded_bank = loaded.expect("just verified bank exists");
    assert_eq!(
        loaded_bank.all().count(),
        1,
        "Loaded PropertyBank should contain registered property"
    );

    Ok(())
}

/// **3.4-INT-003**: `PropertyBank` save updates existing singleton.
/// Priority: P0.
///
/// Verifies that:
/// - Second save overwrites first save
/// - Version increments are persisted
/// - Properties are updated correctly
#[test]
fn property_bank_save_updates_existing_singleton() -> TestResult {
    // GIVEN: A PropertyBank with one property
    let (_dir, db) = setup_db()?;
    let command = RedbSchemaCommand::new(CommandAdapter::new(&db));
    let query = RedbSchemaQuery::new(QueryAdapter::new(&db));

    let mut bank = PropertyBank::new();
    let prop1 = create_sample_property(
        TEST_PROPERTY_ID_A,
        "status",
        PropertySpec::Bool(BoolSpec::default()),
    )?;
    bank.register(prop1)?;
    command.save_property_bank(&bank)?;

    let initial_version = bank.version();

    // WHEN: Adding another property and saving
    let prop2 = create_sample_property(
        TEST_PROPERTY_ID_B,
        "title",
        PropertySpec::String(StringSpec::default()),
    )?;
    bank.register(prop2)?;
    let updated_version = bank.version();
    command.save_property_bank(&bank)?;

    // THEN: Loaded bank reflects updates
    let loaded = query.find_property_bank()?.expect("Bank should exist");
    assert_eq!(
        loaded.all().count(),
        2,
        "Updated PropertyBank should contain both properties"
    );
    assert_eq!(
        loaded.version(),
        updated_version,
        "Loaded version should match updated version"
    );
    assert!(
        initial_version.is_older_than(loaded.version()),
        "Version should have incremented"
    );

    Ok(())
}

/// **3.4-INT-004**: `PropertyBank` version increments persist correctly.
/// Priority: P0.
///
/// Verifies that:
/// - Initial version is 0
/// - Version increments on property registration
/// - Version persists across save/load
#[test]
fn property_bank_version_increments_persist() -> TestResult {
    // GIVEN: A new PropertyBank
    let (_dir, db) = setup_db()?;
    let command = RedbSchemaCommand::new(CommandAdapter::new(&db));
    let query = RedbSchemaQuery::new(QueryAdapter::new(&db));

    let mut bank = PropertyBank::new();
    let initial_version = bank.version();

    // WHEN: Registering properties
    let prop1 = create_sample_property(
        TEST_PROPERTY_ID_A,
        "status",
        PropertySpec::Bool(BoolSpec::default()),
    )?;
    bank.register(prop1)?;
    let version_after_first = bank.version();

    let prop2 = create_sample_property(
        TEST_PROPERTY_ID_B,
        "title",
        PropertySpec::String(StringSpec::default()),
    )?;
    bank.register(prop2)?;
    let version_after_second = bank.version();

    command.save_property_bank(&bank)?;

    // THEN: Versions increment correctly and persist
    assert!(
        initial_version.is_older_than(version_after_first),
        "Version should increment after first property"
    );
    assert!(
        version_after_first.is_older_than(version_after_second),
        "Version should increment after second property"
    );

    let loaded = query.find_property_bank()?.expect("Bank should exist");
    assert_eq!(
        loaded.version(),
        version_after_second,
        "Persisted version should match final version"
    );

    Ok(())
}

/// **3.4-INT-005**: `PropertyBank` survives database restart.
/// Priority: P0.
///
/// Verifies that:
/// - `PropertyBank` persists to disk
/// - Reopening database preserves `PropertyBank`
/// - All properties are intact after restart
#[test]
#[expect(
    clippy::semicolon_outside_block,
    reason = "Block intentionally scoped to drop database before reopening"
)]
fn property_bank_survives_restart() -> TestResult {
    // GIVEN: A PropertyBank saved to disk
    let dir = tempdir()?;
    let db_path = dir.path().join("lithos.redb");

    let mut bank = PropertyBank::new();
    let prop = create_sample_property(
        TEST_PROPERTY_ID_A,
        "status",
        PropertySpec::Bool(BoolSpec::default()),
    )?;
    bank.register(prop)?;
    let original_version = bank.version();

    // Save with first database connection
    {
        let db = Database::open(&db_path)?;
        let command = RedbSchemaCommand::new(CommandAdapter::new(&db));
        command.save_property_bank(&bank)?;
    } // Database closed

    // WHEN: Reopening database
    let db = Database::open(&db_path)?;
    let query = RedbSchemaQuery::new(QueryAdapter::new(&db));

    // THEN: PropertyBank is intact
    let loaded = query.find_property_bank()?;
    assert!(loaded.is_some(), "PropertyBank should survive database restart");

    let loaded_bank = loaded.expect("just verified bank exists");
    assert_eq!(
        loaded_bank.id(),
        PropertyBankId::singleton(),
        "Singleton ID should persist"
    );
    assert_eq!(
        loaded_bank.version(),
        original_version,
        "Version should persist"
    );
    assert_eq!(loaded_bank.all().count(), 1, "Properties should persist");

    let name = PropertyName::new("status")?;
    assert!(
        loaded_bank.has_name(&name),
        "Property 'status' should exist after restart"
    );

    Ok(())
}

/// **3.4-INT-006**: `PropertyBank` roundtrip preserves all fields.
/// Priority: P1.
///
/// Verifies that:
/// - ID preserves correctly
/// - Version preserves correctly
/// - Properties preserve correctly
/// - Indices rebuild correctly
#[test]
fn property_bank_roundtrip_preserves_all_fields() -> TestResult {
    // GIVEN: A PropertyBank with multiple properties
    let (_dir, db) = setup_db()?;
    let command = RedbSchemaCommand::new(CommandAdapter::new(&db));
    let query = RedbSchemaQuery::new(QueryAdapter::new(&db));

    let mut bank = PropertyBank::new();
    let prop1 = create_sample_property(
        TEST_PROPERTY_ID_A,
        "status",
        PropertySpec::Bool(BoolSpec::default()),
    )?;
    let prop2 = create_sample_property(
        TEST_PROPERTY_ID_B,
        "title",
        PropertySpec::String(StringSpec::default()),
    )?;

    bank.register(prop1)?;
    bank.register(prop2)?;

    let original_id = bank.id();
    let original_version = bank.version();
    let original_count = bank.all().count();

    // WHEN: Saving and loading
    command.save_property_bank(&bank)?;
    let loaded = query.find_property_bank()?.expect("Bank should exist");

    // THEN: All fields preserved
    assert_eq!(loaded.id(), original_id, "ID should preserve");
    assert_eq!(loaded.version(), original_version, "Version should preserve");
    assert_eq!(
        loaded.all().count(),
        original_count,
        "Property count should preserve"
    );

    // Verify dual indices work
    let status_name = PropertyName::new("status")?;
    let title_name = PropertyName::new("title")?;

    assert!(loaded.has_id(PropertyId::from_uuid(TEST_PROPERTY_ID_A)));
    assert!(loaded.has_id(PropertyId::from_uuid(TEST_PROPERTY_ID_B)));
    assert!(loaded.has_name(&status_name));
    assert!(loaded.has_name(&title_name));

    assert!(
        loaded.get_by_id(PropertyId::from_uuid(TEST_PROPERTY_ID_A)).is_some()
    );
    assert!(
        loaded.get_by_id(PropertyId::from_uuid(TEST_PROPERTY_ID_B)).is_some()
    );
    assert!(loaded.get_by_name(&status_name).is_some());
    assert!(loaded.get_by_name(&title_name).is_some());

    Ok(())
}
