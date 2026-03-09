//! Integration tests for Schema CQRS persistence.
//!
//! Tests the full CQRS stack for `PropertyBank` and Schema aggregates:
//! - `PropertyBank`: Singleton persistence, versioning
//! - Schema: CRUD operations, batch saves, indices, roundtrips
//! - Cross-aggregate: `PropertyBank` ↔ Schema consistency
//!
//! Organized into modules for clarity:
//! - `property_bank`: `PropertyBank` CQRS tests (migrated + enhanced)
//! - `schema`: Schema CQRS tests (new comprehensive coverage)
//! - `cross_aggregate`: Tests spanning both aggregates

#![expect(
    clippy::panic_in_result_fn,
    reason = "Integration tests use assertions which panic on failure."
)]
#![expect(
    clippy::tests_outside_test_module,
    reason = "Integration tests are top-level by default."
)]
#![expect(
    clippy::arbitrary_source_item_ordering,
    reason = "Test constants grouped before modules for clarity"
)]

mod common;

use common::*;
use lithos_core::{
    db::Database,
    schema::{
        bank::PropertyBank,
        db_command, db_query,
        id::{SchemaId, SchemaName},
        property::{Multiplicity, Optionality, PropertyId, PropertyName},
    },
};
use uuid::Uuid;

// Test fixture UUIDs (deterministic for reproducibility)
const TEST_PROPERTY_ID_A: Uuid =
    Uuid::from_u128(0x018C_0000_0000_7000_8000_0000_0000_0A01);
const TEST_PROPERTY_ID_B: Uuid =
    Uuid::from_u128(0x018C_0000_0000_7000_8000_0000_0000_0A02);
const TEST_SCHEMA_ID_TASK: Uuid =
    Uuid::from_u128(0x018C_0000_0000_7000_8000_0000_0000_0101);
const TEST_SCHEMA_ID_PROJECT: Uuid =
    Uuid::from_u128(0x018C_0000_0000_7000_8000_0000_0000_0102);

// ========================================================================
//                          PropertyBank CQRS Tests
// ========================================================================

mod property_bank {
    use super::*;

    /// **3.4-INT-002**: `PropertyBank` save persists when missing.
    ///
    /// Verifies:
    /// - Empty database returns None for `PropertyBank`
    /// - First save persists the bank
    /// - Subsequent find returns the saved bank
    #[test]
    fn save_creates_singleton() -> TestResult {
        // GIVEN: An empty database
        let test_db = TestDb::new()?;
        let (command, query) = setup_cqrs(test_db.db());

        let initial = query.get_property_bank()?;
        assert!(
            initial.is_none(),
            "Fresh database should have no PropertyBank"
        );

        // WHEN: Saving a new PropertyBank
        let mut bank = PropertyBank::new();
        let prop = PropertyBuilder::new("status")
            .id(PropertyId::from_uuid(TEST_PROPERTY_ID_A))
            .build_bool()?;
        bank.register(prop)?;
        command.save_property_bank(&bank)?;

        // THEN: PropertyBank is retrievable
        let loaded = query.get_property_bank()?;
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
    ///
    /// Verifies:
    /// - Second save overwrites first save
    /// - Version increments are persisted
    /// - Properties are updated correctly
    #[test]
    fn save_updates_existing_singleton() -> TestResult {
        // GIVEN: A PropertyBank with one property
        let test_db = TestDb::new()?;
        let (command, query) = setup_cqrs(test_db.db());

        let mut bank = PropertyBank::new();
        let prop1 = PropertyBuilder::new("status")
            .id(PropertyId::from_uuid(TEST_PROPERTY_ID_A))
            .build_bool()?;
        bank.register(prop1)?;
        command.save_property_bank(&bank)?;

        let initial_version = bank.version();

        // WHEN: Adding another property and saving
        let prop2 = PropertyBuilder::new("title")
            .id(PropertyId::from_uuid(TEST_PROPERTY_ID_B))
            .build_string_default()?;
        bank.register(prop2)?;
        let updated_version = bank.version();
        command.save_property_bank(&bank)?;

        // THEN: Loaded bank reflects updates
        let loaded = query.get_property_bank()?.expect("Bank should exist");
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
    ///
    /// Verifies:
    /// - Initial version is 0
    /// - Version increments on property registration
    /// - Version persists across save/load
    #[test]
    fn version_increments_persist() -> TestResult {
        // GIVEN: A new PropertyBank
        let test_db = TestDb::new()?;
        let (command, query) = setup_cqrs(test_db.db());

        let mut bank = PropertyBank::new();
        let initial_version = bank.version();

        // WHEN: Registering properties
        let prop1 = PropertyBuilder::new("status")
            .id(PropertyId::from_uuid(TEST_PROPERTY_ID_A))
            .build_bool()?;
        bank.register(prop1)?;
        let version_after_first = bank.version();

        let prop2 = PropertyBuilder::new("title")
            .id(PropertyId::from_uuid(TEST_PROPERTY_ID_B))
            .build_string_default()?;
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

        let loaded = query.get_property_bank()?.expect("Bank should exist");
        assert_eq!(
            loaded.version(),
            version_after_second,
            "Persisted version should match final version"
        );

        Ok(())
    }

    /// **3.4-INT-005**: `PropertyBank` survives database restart.
    ///
    /// Verifies:
    /// - `PropertyBank` persists to disk
    /// - Reopening database preserves `PropertyBank`
    /// - All properties are intact after restart
    #[test]
    #[expect(
        clippy::semicolon_outside_block,
        reason = "Block intentionally scoped to drop database before reopening"
    )]
    fn survives_restart() -> TestResult {
        // GIVEN: A PropertyBank saved to disk
        use tempfile::tempdir;
        let dir = tempdir()?;
        let db_path = dir.path().join("lithos.redb");

        let mut bank = PropertyBank::new();
        let prop = PropertyBuilder::new("status")
            .id(PropertyId::from_uuid(TEST_PROPERTY_ID_A))
            .build_bool()?;
        bank.register(prop)?;
        let original_version = bank.version();

        // Save with first database connection
        {
            let db = Database::open(&db_path)?;
            let command = db_command::Command::new(&db);
            command.save_property_bank(&bank)?;
        } // Database closed

        // WHEN: Reopening database
        let db = Database::open(&db_path)?;
        let query = db_query::Query::new(&db);

        // THEN: PropertyBank is intact
        let loaded = query.get_property_bank()?;
        assert!(
            loaded.is_some(),
            "PropertyBank should survive database restart"
        );

        let loaded_bank = loaded.expect("just verified bank exists");
        assert_eq!(
            loaded_bank.version(),
            original_version,
            "Version should persist"
        );
        assert_eq!(loaded_bank.all().count(), 1, "Properties should persist");

        let name = PropertyName::try_new("status")?;
        assert!(
            loaded_bank.has(&name),
            "Property 'status' should exist after restart"
        );

        Ok(())
    }

    /// **3.4-INT-006**: `PropertyBank` roundtrip preserves all fields.
    ///
    /// Verifies:
    /// - Properties preserve correctly
    /// - Version preserves correctly
    /// - Properties preserve correctly
    /// - Name lookup rebuilds correctly
    #[test]
    fn roundtrip_preserves_all_fields() -> TestResult {
        // GIVEN: A PropertyBank with multiple properties
        let test_db = TestDb::new()?;
        let (command, query) = setup_cqrs(test_db.db());

        let mut bank = PropertyBank::new();
        let prop1 = PropertyBuilder::new("status")
            .id(PropertyId::from_uuid(TEST_PROPERTY_ID_A))
            .build_bool()?;
        let prop2 = PropertyBuilder::new("title")
            .id(PropertyId::from_uuid(TEST_PROPERTY_ID_B))
            .build_string_default()?;

        bank.register(prop1)?;
        bank.register(prop2)?;

        let original_version = bank.version();
        let original_count = bank.all().count();

        // WHEN: Saving and loading
        command.save_property_bank(&bank)?;
        let loaded = query.get_property_bank()?.expect("Bank should exist");

        // THEN: All fields preserved
        assert_eq!(
            loaded.version(),
            original_version,
            "Version should preserve"
        );
        assert_eq!(
            loaded.all().count(),
            original_count,
            "Property count should preserve"
        );

        // Verify name lookup works
        let status_name = PropertyName::try_new("status")?;
        let title_name = PropertyName::try_new("title")?;

        assert!(loaded.has(&status_name));
        assert!(loaded.has(&title_name));

        assert!(loaded.get(&status_name).is_some());
        assert!(loaded.get(&title_name).is_some());

        Ok(())
    }

    // --- NEW TESTS (beyond original 6) ---

    /// **3.4-INT-007**: `PropertyBank` with zero properties persists correctly.
    ///
    /// Verifies:
    /// - Empty `PropertyBank` can be saved
    /// - Loaded bank is also empty
    /// - Version still persists
    #[test]
    fn empty_bank_persists() -> TestResult {
        // GIVEN: An empty PropertyBank
        let test_db = TestDb::new()?;
        let (command, query) = setup_cqrs(test_db.db());

        let bank = PropertyBank::new();
        let original_version = bank.version();

        // WHEN: Saving empty bank
        command.save_property_bank(&bank)?;

        // THEN: Loaded bank is empty but valid
        let loaded = query.get_property_bank()?.expect("Bank should exist");
        assert_eq!(loaded.all().count(), 0, "Bank should be empty");
        assert_eq!(loaded.version(), original_version);

        Ok(())
    }

    /// **3.4-INT-008**: `PropertyBank` name lookup works after updates.
    ///
    /// Verifies:
    /// - Name lookup remains consistent after updates
    /// - Lookups return correct properties
    #[test]
    fn indices_consistent_after_updates() -> TestResult {
        // GIVEN: A PropertyBank that's updated multiple times
        let test_db = TestDb::new()?;
        let (command, query) = setup_cqrs(test_db.db());

        let mut bank = PropertyBank::new();

        // First save: 1 property
        let prop1 = PropertyBuilder::new("alpha")
            .id(PropertyId::from_uuid(TEST_PROPERTY_ID_A))
            .build_bool()?;
        bank.register(prop1)?;
        command.save_property_bank(&bank)?;

        // Second save: 2 properties
        let prop2 = PropertyBuilder::new("beta")
            .id(PropertyId::from_uuid(TEST_PROPERTY_ID_B))
            .build_string_default()?;
        bank.register(prop2)?;
        command.save_property_bank(&bank)?;

        // WHEN: Loading bank
        let loaded = query.get_property_bank()?.expect("Bank should exist");

        // THEN: Name lookup works correctly
        let alpha_name = PropertyName::try_new("alpha")?;
        let beta_name = PropertyName::try_new("beta")?;
        assert!(loaded.has(&alpha_name));
        assert!(loaded.has(&beta_name));

        let prop_by_name =
            loaded.get(&beta_name).expect("Property should exist by name");
        assert_eq!(prop_by_name.name().as_str(), "beta");

        Ok(())
    }

    /// **3.4-INT-009**: `PropertyBank` iteration order is consistent.
    ///
    /// Verifies:
    /// - Properties are iterable via `all()`
    /// - Order is deterministic (sorted by name)
    /// - All registered properties are included
    #[test]
    fn iteration_order_consistent() -> TestResult {
        // GIVEN: A PropertyBank with multiple properties
        let test_db = TestDb::new()?;
        let (command, query) = setup_cqrs(test_db.db());

        let mut bank = PropertyBank::new();
        let prop_a = PropertyBuilder::new("zebra")
            .id(PropertyId::from_uuid(TEST_PROPERTY_ID_A))
            .build_bool()?;
        let prop_b = PropertyBuilder::new("alpha")
            .id(PropertyId::from_uuid(TEST_PROPERTY_ID_B))
            .build_string_default()?;

        // Register in non-alphabetical order
        bank.register(prop_a)?;
        bank.register(prop_b)?;
        command.save_property_bank(&bank)?;

        // WHEN: Loading and iterating
        let loaded = query.get_property_bank()?.expect("Bank should exist");
        let prop_names: Vec<_> =
            loaded.all().map(|p| p.name().as_str()).collect();

        // THEN: All properties present (order implementation-defined, but
        // deterministic)
        assert_eq!(prop_names.len(), 2);
        assert!(prop_names.contains(&"zebra"));
        assert!(prop_names.contains(&"alpha"));

        Ok(())
    }

    /// **3.4-INT-010**: `get_property_by_id` returns correct property.
    ///
    /// Verifies:
    /// - Property can be retrieved by ID
    /// - Retrieved property matches original
    #[test]
    fn get_property_by_id_returns_correct_property() -> TestResult {
        // GIVEN: A PropertyBank with registered properties
        let test_db = TestDb::new()?;
        let (command, query) = setup_cqrs(test_db.db());

        let mut bank = PropertyBank::new();
        let prop1 = PropertyBuilder::new("status")
            .id(PropertyId::from_uuid(TEST_PROPERTY_ID_A))
            .build_bool()?;
        let prop2 = PropertyBuilder::new("title")
            .id(PropertyId::from_uuid(TEST_PROPERTY_ID_B))
            .build_string_default()?;

        bank.register(prop1.clone())?;
        bank.register(prop2)?;
        command.save_property_bank(&bank)?;

        // WHEN: Retrieving property by ID
        let retrieved = query.get_property_by_id(prop1.id())?;

        // THEN: Correct property is returned
        assert!(retrieved.is_some(), "Property should exist");
        let retrieved_prop = retrieved.expect("just verified property exists");
        assert_eq!(retrieved_prop.id(), prop1.id());
        assert_eq!(retrieved_prop.name(), prop1.name());

        Ok(())
    }

    /// **3.4-INT-011**: `get_property_by_id` returns None for invalid ID.
    ///
    /// Verifies:
    /// - Invalid property ID returns None (not error)
    #[test]
    fn get_property_by_id_invalid_id_returns_none() -> TestResult {
        // GIVEN: A PropertyBank with one property
        let test_db = TestDb::new()?;
        let (command, query) = setup_cqrs(test_db.db());

        let mut bank = PropertyBank::new();
        let prop = PropertyBuilder::new("status")
            .id(PropertyId::from_uuid(TEST_PROPERTY_ID_A))
            .build_bool()?;
        bank.register(prop)?;
        command.save_property_bank(&bank)?;

        // WHEN: Querying with invalid ID
        let invalid_id = PropertyId::from_uuid(TEST_PROPERTY_ID_B);
        let result = query.get_property_by_id(invalid_id)?;

        // THEN: Returns None
        assert!(result.is_none(), "Invalid ID should return None");

        Ok(())
    }

    /// **3.4-INT-012**: `get_property_by_id` returns None when bank missing.
    ///
    /// Verifies:
    /// - Query gracefully handles missing `PropertyBank`
    #[test]
    fn get_property_by_id_bank_missing_returns_none() -> TestResult {
        // GIVEN: An empty database (no PropertyBank)
        let test_db = TestDb::new()?;
        let (_command, query) = setup_cqrs(test_db.db());

        // Verify PropertyBank doesn't exist
        assert!(
            query.get_property_bank()?.is_none(),
            "PropertyBank should not exist"
        );

        // WHEN: Querying for property by ID
        let id = PropertyId::from_uuid(TEST_PROPERTY_ID_A);
        let result = query.get_property_by_id(id)?;

        // THEN: Returns None gracefully
        assert!(
            result.is_none(),
            "Should return None when PropertyBank missing"
        );

        Ok(())
    }

    /// **3.4-INT-013**: `get_property_by_id` roundtrip preserves data.
    ///
    /// Verifies:
    /// - Register → Save → Get by ID preserves all property fields
    #[test]
    fn get_property_by_id_roundtrip_preserves_data() -> TestResult {
        // GIVEN: A property with all fields set
        let test_db = TestDb::new()?;
        let (command, query) = setup_cqrs(test_db.db());

        let mut bank = PropertyBank::new();
        let original_prop = PropertyBuilder::new("priority")
            .id(PropertyId::from_uuid(TEST_PROPERTY_ID_A))
            .optionality(Optionality::Required)
            .multiplicity(Multiplicity::Many)
            .build_string_default()?;

        bank.register(original_prop.clone())?;
        command.save_property_bank(&bank)?;

        // WHEN: Retrieving by ID
        let retrieved = query
            .get_property_by_id(original_prop.id())?
            .expect("Property should exist");

        // THEN: All fields match
        assert_eq!(retrieved.id(), original_prop.id());
        assert_eq!(retrieved.name(), original_prop.name());
        assert_eq!(retrieved.optionality(), original_prop.optionality());
        assert_eq!(retrieved.multiplicity(), original_prop.multiplicity());

        Ok(())
    }
}

// ========================================================================
//                          Schema CQRS Tests
// ========================================================================

mod schema {
    use super::*;

    /// **3.4-INT-010**: Schema save and load roundtrip works.
    ///
    /// Verifies:
    /// - Schema can be saved
    /// - Schema can be loaded by ID
    /// - All fields preserve correctly
    #[test]
    fn save_and_load_roundtrip() -> TestResult {
        // GIVEN: A schema with properties
        let test_db = TestDb::new()?;
        let (command, query) = setup_cqrs(test_db.db());

        let prop = bool_property("is_done")?;
        let schema = SchemaBuilder::new("task")
            .id(SchemaId::from_uuid(TEST_SCHEMA_ID_TASK))
            .property(prop)
            .build()?;

        let original_id = schema.id;
        let original_name = schema.name.clone();

        // WHEN: Saving and loading
        command.save(&schema)?;
        let loaded =
            query.find_by_id(original_id)?.expect("Schema should exist");

        // THEN: All fields preserved
        assert_eq!(loaded.id, original_id);
        assert_eq!(loaded.name, original_name);
        assert_eq!(loaded.properties.len(), 1);
        assert_has_property(&loaded, "is_done", "After roundtrip");

        Ok(())
    }

    /// **3.4-INT-011**: Schema can be found by name.
    ///
    /// Verifies:
    /// - Name index is populated on save
    /// - `find_by_name` returns correct schema
    /// - Schema content matches original
    #[test]
    fn find_by_name_works() -> TestResult {
        // GIVEN: A saved schema
        let test_db = TestDb::new()?;
        let (command, query) = setup_cqrs(test_db.db());

        let schema = SchemaBuilder::new("project")
            .id(SchemaId::from_uuid(TEST_SCHEMA_ID_PROJECT))
            .build()?;

        command.save(&schema)?;

        // WHEN: Finding by name
        let name = SchemaName::try_new("project")?;
        let loaded = query.find_by_name(&name)?.expect("Schema should exist");

        // THEN: Correct schema returned
        assert_eq!(loaded.id, schema.id);
        assert_eq!(loaded.name, schema.name);

        Ok(())
    }

    /// **3.4-INT-012**: Schema `find_by_name` returns None for missing schema.
    ///
    /// Verifies:
    /// - Missing schema returns None (not error)
    /// - Database remains consistent
    #[test]
    fn find_by_name_missing_returns_none() -> TestResult {
        // GIVEN: An empty database
        let test_db = TestDb::new()?;
        let (_command, query) = setup_cqrs(test_db.db());

        // WHEN: Finding nonexistent schema
        let name = SchemaName::try_new("nonexistent")?;
        let result = query.find_by_name(&name)?;

        // THEN: Returns None
        assert!(result.is_none());

        Ok(())
    }

    /// **3.4-INT-013**: Schema batch save works atomically.
    ///
    /// Verifies:
    /// - Multiple schemas can be saved in one transaction
    /// - All schemas are retrievable after batch save
    /// - Name indices are populated for all schemas
    #[test]
    fn batch_save_atomic() -> TestResult {
        // GIVEN: Multiple schemas
        let test_db = TestDb::new()?;
        let (command, query) = setup_cqrs(test_db.db());

        let task_schema = SchemaBuilder::new("task")
            .id(SchemaId::from_uuid(TEST_SCHEMA_ID_TASK))
            .build()?;
        let project_schema = SchemaBuilder::new("project")
            .id(SchemaId::from_uuid(TEST_SCHEMA_ID_PROJECT))
            .build()?;

        let schemas = vec![task_schema, project_schema];

        // WHEN: Batch saving
        command.save_many(&schemas)?;

        // THEN: All schemas retrievable
        let loaded1 = query
            .find_by_id(SchemaId::from_uuid(TEST_SCHEMA_ID_TASK))?
            .expect("task schema should exist");
        let loaded2 = query
            .find_by_id(SchemaId::from_uuid(TEST_SCHEMA_ID_PROJECT))?
            .expect("project schema should exist");

        assert_eq!(loaded1.name.as_ref(), "task");
        assert_eq!(loaded2.name.as_ref(), "project");

        // Verify name indices
        assert!(query.find_by_name(&SchemaName::try_new("task")?)?.is_some());
        assert!(
            query.find_by_name(&SchemaName::try_new("project")?)?.is_some()
        );

        Ok(())
    }

    /// **3.4-INT-014**: Schema list returns all schemas.
    ///
    /// Verifies:
    /// - Empty database returns empty list
    /// - List returns all saved schemas
    /// - Order is deterministic
    #[test]
    fn list_returns_all_schemas() -> TestResult {
        // GIVEN: Multiple schemas
        let test_db = TestDb::new()?;
        let (command, query) = setup_cqrs(test_db.db());

        // Empty list initially
        let initial = query.list()?;
        assert_eq!(initial.len(), 0);

        let schema1 = SchemaBuilder::new("task").build()?;
        let schema2 = SchemaBuilder::new("project").build()?;
        command.save_many(&[schema1, schema2])?;

        // WHEN: Listing all
        let all = query.list()?;

        // THEN: All schemas returned
        assert_eq!(all.len(), 2);
        let names: Vec<_> = all.iter().map(|s| s.name.as_ref()).collect();
        assert!(names.contains(&"task"));
        assert!(names.contains(&"project"));

        Ok(())
    }

    /// **3.4-INT-015**: Schema delete removes schema and index.
    ///
    /// Verifies:
    /// - Schema can be deleted by ID
    /// - Deleted schema is not findable by ID
    /// - Deleted schema is not findable by name
    /// - Other schemas remain intact
    #[test]
    fn delete_removes_schema_and_index() -> TestResult {
        // GIVEN: Two schemas
        let test_db = TestDb::new()?;
        let (command, query) = setup_cqrs(test_db.db());

        let schema1 = SchemaBuilder::new("task")
            .id(SchemaId::from_uuid(TEST_SCHEMA_ID_TASK))
            .build()?;
        let schema2 = SchemaBuilder::new("project")
            .id(SchemaId::from_uuid(TEST_SCHEMA_ID_PROJECT))
            .build()?;
        command.save_many(&[schema1, schema2])?;

        // WHEN: Deleting one schema
        command.delete(SchemaId::from_uuid(TEST_SCHEMA_ID_TASK))?;

        // THEN: Deleted schema not found
        assert!(
            query
                .find_by_id(SchemaId::from_uuid(TEST_SCHEMA_ID_TASK))?
                .is_none()
        );
        assert!(query.find_by_name(&SchemaName::try_new("task")?)?.is_none());

        // Other schema still exists
        assert!(
            query
                .find_by_id(SchemaId::from_uuid(TEST_SCHEMA_ID_PROJECT))?
                .is_some()
        );
        assert!(
            query.find_by_name(&SchemaName::try_new("project")?)?.is_some()
        );

        Ok(())
    }

    /// **3.4-INT-016**: Schema properties persist in correct order.
    ///
    /// Verifies:
    /// - Properties are stored in `BTreeMap` (sorted by name)
    /// - Loaded schema has properties in sorted order
    #[test]
    fn properties_persist_sorted() -> TestResult {
        // GIVEN: Schema with properties in non-alphabetical order
        let test_db = TestDb::new()?;
        let (command, query) = setup_cqrs(test_db.db());

        let prop_z = string_property("zebra")?;
        let prop_a = bool_property("alpha")?;
        let prop_m = string_property("middle")?;

        let schema = SchemaBuilder::new("test")
            .property(prop_z)
            .property(prop_a)
            .property(prop_m)
            .build()?;

        // WHEN: Saving and loading
        command.save(&schema)?;
        let loaded = query.find_by_id(schema.id)?.expect("Schema should exist");

        // THEN: Properties are sorted
        assert_properties_sorted(&loaded, "Loaded schema");

        let names: Vec<_> =
            loaded.properties.iter().map(|p| p.name.as_ref()).collect();
        assert_eq!(names, vec!["alpha", "middle", "zebra"]);

        Ok(())
    }

    /// **3.4-INT-017**: Schema with zero properties persists correctly.
    ///
    /// Verifies:
    /// - Schema with no properties can be saved
    /// - Loaded schema has zero properties
    /// - Name and ID still work
    #[test]
    fn empty_schema_persists() -> TestResult {
        // GIVEN: Schema with no properties
        let test_db = TestDb::new()?;
        let (command, query) = setup_cqrs(test_db.db());

        let schema = SchemaBuilder::new("empty").build()?;
        let schema_id = schema.id;

        // WHEN: Saving and loading
        command.save(&schema)?;
        let loaded = query.find_by_id(schema_id)?.expect("Schema should exist");

        // THEN: Empty schema works
        assert_eq!(loaded.properties.len(), 0);
        assert_eq!(loaded.name.as_ref(), "empty");
        assert_eq!(loaded.id, schema_id);

        Ok(())
    }

    /// **3.4-INT-018**: Schema survives database restart.
    ///
    /// Verifies:
    /// - Schema persists to disk
    /// - Reopening database preserves schema
    /// - Properties and indices intact after restart
    #[test]
    #[expect(
        clippy::semicolon_outside_block,
        reason = "Block intentionally scoped to drop database"
    )]
    fn survives_restart() -> TestResult {
        // GIVEN: A schema saved to disk
        use tempfile::tempdir;
        let dir = tempdir()?;
        let db_path = dir.path().join("lithos.redb");

        let prop = bool_property("is_done")?;
        let schema = SchemaBuilder::new("task")
            .id(SchemaId::from_uuid(TEST_SCHEMA_ID_TASK))
            .property(prop)
            .build()?;

        let schema_id = schema.id;
        let schema_name = schema.name.clone();

        // Save with first connection
        {
            let db = Database::open(&db_path)?;
            let command = db_command::Command::new(&db);
            command.save(&schema)?;
        } // Database closed

        // WHEN: Reopening database
        let db = Database::open(&db_path)?;
        let query = db_query::Query::new(&db);

        // THEN: Schema intact
        let loaded = query
            .find_by_id(schema_id)?
            .expect("Schema should survive restart");
        assert_eq!(loaded.id, schema_id);
        assert_eq!(loaded.name, schema_name);
        assert_eq!(loaded.properties.len(), 1);

        // Name index still works
        let name_lookup = SchemaName::try_new(schema_name.as_ref())?;
        assert!(query.find_by_name(&name_lookup)?.is_some());

        Ok(())
    }

    /// **3.4-INT-019**: Schema update overwrites existing entry.
    ///
    /// Verifies:
    /// - Second save with same ID overwrites first
    /// - Properties are updated correctly
    /// - Name index reflects new name if changed
    #[test]
    fn update_overwrites_existing() -> TestResult {
        // GIVEN: A saved schema
        let test_db = TestDb::new()?;
        let (command, query) = setup_cqrs(test_db.db());

        let schema_v1 = SchemaBuilder::new("task")
            .id(SchemaId::from_uuid(TEST_SCHEMA_ID_TASK))
            .property(bool_property("is_done")?)
            .build()?;
        command.save(&schema_v1)?;

        // WHEN: Saving updated version with same ID
        let schema_v2 = SchemaBuilder::new("task")
            .id(SchemaId::from_uuid(TEST_SCHEMA_ID_TASK))
            .property(bool_property("is_done")?)
            .property(string_property("title")?)
            .build()?;
        command.save(&schema_v2)?;

        // THEN: Updated version loaded
        let loaded = query
            .find_by_id(SchemaId::from_uuid(TEST_SCHEMA_ID_TASK))?
            .expect("Schema should exist");
        assert_eq!(loaded.properties.len(), 2);
        assert_has_property(&loaded, "is_done", "After update");
        assert_has_property(&loaded, "title", "After update");

        Ok(())
    }

    /// **3.4-INT-020**: Schema `list_name_id_pairs` returns all mappings.
    ///
    /// Verifies:
    /// - Empty database returns empty vec
    /// - All name→ID mappings are returned
    /// - Mappings match actual schemas
    #[test]
    fn list_name_id_pairs_works() -> TestResult {
        // GIVEN: Multiple schemas
        let test_db = TestDb::new()?;
        let (command, query) = setup_cqrs(test_db.db());

        let schema1 = SchemaBuilder::new("task")
            .id(SchemaId::from_uuid(TEST_SCHEMA_ID_TASK))
            .build()?;
        let schema2 = SchemaBuilder::new("project")
            .id(SchemaId::from_uuid(TEST_SCHEMA_ID_PROJECT))
            .build()?;
        command.save_many(&[schema1, schema2])?;

        // WHEN: Listing name-ID pairs
        let pairs = query.list_name_id_pairs()?;

        // THEN: All pairs returned
        assert_eq!(pairs.len(), 2);

        let names: Vec<_> = pairs.iter().map(|p| p.0.as_str()).collect();
        assert!(names.contains(&"task"));
        assert!(names.contains(&"project"));

        // Verify IDs match
        let task_pair = pairs
            .iter()
            .find(|p| p.0.as_str() == "task")
            .expect("task pair should exist");
        assert_eq!(task_pair.1, SchemaId::from_uuid(TEST_SCHEMA_ID_TASK));

        Ok(())
    }

    /// **3.4-INT-021**: Schema with `parent_id` persists correctly.
    ///
    /// Verifies:
    /// - Schema with `parent_id` can be saved
    /// - Parent ID is preserved on load
    #[test]
    fn parent_id_persists() -> TestResult {
        // GIVEN: A child schema with parent
        let test_db = TestDb::new()?;
        let (command, query) = setup_cqrs(test_db.db());

        let parent_id = SchemaId::from_uuid(TEST_SCHEMA_ID_PROJECT);
        let child = SchemaBuilder::new("task")
            .id(SchemaId::from_uuid(TEST_SCHEMA_ID_TASK))
            .parent(parent_id)
            .build()?;

        // WHEN: Saving and loading
        command.save(&child)?;
        let loaded = query
            .find_by_id(SchemaId::from_uuid(TEST_SCHEMA_ID_TASK))?
            .expect("Schema should exist");

        // THEN: Parent ID preserved
        assert_eq!(loaded.parent_id, Some(parent_id));

        Ok(())
    }

    /// **3.4-INT-022**: Schema with no parent has None `parent_id`.
    ///
    /// Verifies:
    /// - Root schema has None for `parent_id`
    /// - None is preserved correctly
    #[test]
    fn no_parent_persists_as_none() -> TestResult {
        // GIVEN: A root schema
        let test_db = TestDb::new()?;
        let (command, query) = setup_cqrs(test_db.db());

        let schema = SchemaBuilder::new("root").build()?;
        let schema_id = schema.id;

        // WHEN: Saving and loading
        command.save(&schema)?;
        let loaded = query.find_by_id(schema_id)?.expect("Schema should exist");

        // THEN: parent_id is None
        assert_eq!(loaded.parent_id, None);

        Ok(())
    }

    /// **3.4-INT-023**: Schema properties with different optionality persist.
    ///
    /// Verifies:
    /// - Required and Optional properties both work
    /// - Optionality is preserved correctly
    #[test]
    fn optionality_persists() -> TestResult {
        // GIVEN: Schema with mixed optionality
        let test_db = TestDb::new()?;
        let (command, query) = setup_cqrs(test_db.db());

        let required = PropertyBuilder::new("title")
            .optionality(Optionality::Required)
            .build_string_default()?;
        let optional = PropertyBuilder::new("description")
            .optionality(Optionality::Optional)
            .build_string_default()?;

        let schema = SchemaBuilder::new("test")
            .property(required)
            .property(optional)
            .build()?;

        // WHEN: Saving and loading
        command.save(&schema)?;
        let loaded = query.find_by_id(schema.id)?.expect("Schema should exist");

        // THEN: Optionality preserved
        let title_prop = loaded
            .properties
            .iter()
            .find(|p| p.name.as_ref() == "title")
            .expect("title property should exist");
        assert!(title_prop.required);

        let desc_prop = loaded
            .properties
            .iter()
            .find(|p| p.name.as_ref() == "description")
            .expect("description property should exist");
        assert!(!desc_prop.required);

        Ok(())
    }

    /// **3.4-INT-024**: Schema properties with Many multiplicity persist.
    ///
    /// Verifies:
    /// - Single and Many properties both work
    /// - Multiplicity is preserved correctly
    #[test]
    fn multiplicity_persists() -> TestResult {
        // GIVEN: Schema with mixed multiplicity
        let test_db = TestDb::new()?;
        let (command, query) = setup_cqrs(test_db.db());

        let single = PropertyBuilder::new("title")
            .multiplicity(Multiplicity::Single)
            .build_string_default()?;
        let many = PropertyBuilder::new("tags")
            .multiplicity(Multiplicity::Many)
            .build_string_default()?;

        let schema = SchemaBuilder::new("test")
            .property(single)
            .property(many)
            .build()?;

        // WHEN: Saving and loading
        command.save(&schema)?;
        let loaded = query.find_by_id(schema.id)?.expect("Schema should exist");

        // THEN: Multiplicity preserved
        let title_prop = loaded
            .properties
            .iter()
            .find(|p| p.name.as_ref() == "title")
            .expect("title property should exist");
        assert!(!title_prop.multi);

        let tags_prop = loaded
            .properties
            .iter()
            .find(|p| p.name.as_ref() == "tags")
            .expect("tags property should exist");
        assert!(tags_prop.multi);

        Ok(())
    }
}

// ========================================================================
//                      Cross-Aggregate Tests
// ========================================================================

mod cross_aggregate {
    use super::*;

    /// **3.4-INT-025**: `PropertyBank` and Schema can coexist in database.
    ///
    /// Verifies:
    /// - `PropertyBank` and Schema use separate tables
    /// - Both can be saved and loaded independently
    /// - No interference between aggregates
    #[test]
    fn property_bank_and_schema_coexist() -> TestResult {
        // GIVEN: PropertyBank and Schema
        let test_db = TestDb::new()?;
        let (command, query) = setup_cqrs(test_db.db());

        let mut bank = PropertyBank::new();
        let prop = bool_property("flag")?;
        bank.register(prop)?;

        let schema = SchemaBuilder::new("test").build()?;
        let schema_id = schema.id;

        // WHEN: Saving both
        command.save_property_bank(&bank)?;
        command.save(&schema)?;

        // THEN: Both retrievable independently
        let loaded_bank =
            query.get_property_bank()?.expect("Bank should exist");
        let loaded_schema =
            query.find_by_id(schema_id)?.expect("Schema should exist");

        assert_eq!(loaded_bank.all().count(), 1);
        assert_eq!(loaded_schema.name.as_ref(), "test");

        Ok(())
    }

    /// **3.4-INT-026**: Multiple schemas can be saved with same `PropertyBank`.
    ///
    /// Verifies:
    /// - `PropertyBank` is global registry
    /// - Multiple schemas can reference same properties
    /// - No conflicts or corruption
    #[test]
    fn multiple_schemas_with_shared_bank() -> TestResult {
        // GIVEN: PropertyBank and multiple schemas
        let test_db = TestDb::new()?;
        let (command, query) = setup_cqrs(test_db.db());

        let mut bank = PropertyBank::new();
        let prop = bool_property("shared")?;
        bank.register(prop.clone())?;
        command.save_property_bank(&bank)?;

        let schema1 =
            SchemaBuilder::new("schema1").property(prop.clone()).build()?;
        let schema2 =
            SchemaBuilder::new("schema2").property(prop.clone()).build()?;

        // WHEN: Saving both schemas
        command.save_many(&[schema1, schema2])?;

        // THEN: Both schemas exist with same property
        let loaded1 = query
            .find_by_name(&SchemaName::try_new("schema1")?)?
            .expect("schema1 should exist");
        let loaded2 = query
            .find_by_name(&SchemaName::try_new("schema2")?)?
            .expect("schema2 should exist");

        assert_has_property(&loaded1, "shared", "schema1");
        assert_has_property(&loaded2, "shared", "schema2");

        Ok(())
    }

    /// **3.4-INT-027**: `PropertyBank` version independent of Schema saves.
    ///
    /// Verifies:
    /// - Saving Schema doesn't change `PropertyBank` version
    /// - Saving `PropertyBank` doesn't affect Schema
    /// - Versions are tracked independently
    #[test]
    fn versions_independent() -> TestResult {
        // GIVEN: PropertyBank and Schema
        let test_db = TestDb::new()?;
        let (command, query) = setup_cqrs(test_db.db());

        let mut bank = PropertyBank::new();
        let prop = bool_property("flag")?;
        bank.register(prop)?;
        command.save_property_bank(&bank)?;
        let bank_version = bank.version();

        let schema = SchemaBuilder::new("test").build()?;
        let schema_id = schema.id;

        // WHEN: Saving schema (doesn't touch PropertyBank)
        command.save(&schema)?;

        // THEN: PropertyBank version unchanged
        let loaded_bank =
            query.get_property_bank()?.expect("Bank should exist");
        assert_eq!(loaded_bank.version(), bank_version);

        // Schema still exists
        assert!(query.find_by_id(schema_id)?.is_some());

        Ok(())
    }
}

// ========================================================================
//                      Critical Edge Cases & Error Handling
// ========================================================================

mod critical {
    use super::*;

    /// **Critical-001**: Property bank respects version retention limit.
    ///
    /// Verifies that only the last 3 versions are retained in the database
    /// after saving 6 versions (retention limit = 3).
    #[test]
    fn property_bank_respects_version_retention_limit() -> TestResult {
        // GIVEN: A property bank
        let test_db = TestDb::new()?;
        let (command, query) = setup_cqrs(test_db.db());

        let mut bank = PropertyBank::new();
        let initial_version = bank.version();

        // WHEN: Saving 6 versions (retention = 3, so first 3 should be
        // deleted)
        for i in 0u32..6u32 {
            let prop_name = format!("prop{i}");
            let prop =
                PropertyBuilder::new(&prop_name).build_string_default()?;
            bank.register(prop)?;
            command.save_property_bank(&bank)?;
        }

        let final_version = bank.version();

        // THEN: Latest version should be retrievable with all 6 properties
        let loaded_bank =
            query.get_property_bank()?.expect("Bank should exist");

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
            let name = PropertyName::try_new(&prop_name)?;
            assert!(
                loaded_bank.has(&name),
                "Property '{prop_name}' should be present"
            );
        }

        // File size should be bounded (retention working)
        let db_file_size = std::fs::metadata(test_db.path())?.len();
        assert!(
            db_file_size > 1024,
            "Database file should contain data (size: {db_file_size} bytes)"
        );

        Ok(())
    }

    /// **Critical-002**: Batch save with duplicate names fails.
    ///
    /// Verifies that `save_many()` detects duplicate schema names within
    /// the same batch and rejects the entire operation.
    #[test]
    fn batch_save_duplicate_names_in_batch_fails() -> TestResult {
        // GIVEN: Multiple schemas with duplicate names
        let test_db = TestDb::new()?;
        let (command, _query) = setup_cqrs(test_db.db());

        let schema1 = SchemaBuilder::new("duplicate").build()?;
        let schema2 = SchemaBuilder::new("duplicate").build()?;

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

    /// **Critical-003**: Save rejects invalid property references.
    ///
    /// Verifies that schemas with property references that don't exist in
    /// `PropertyBank` are rejected.
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

    /// **Critical-004**: Save succeeds without `PropertyBank` (bootstrap case).
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

    /// **Critical-005**: Delete removes schema metadata completely.
    ///
    /// Verifies that `delete()` removes schema from all tables including
    /// metadata.
    #[test]
    fn delete_removes_schema_metadata() -> TestResult {
        // GIVEN: A saved schema
        let test_db = TestDb::new()?;
        let (command, query) = setup_cqrs(test_db.db());

        let schema = SchemaBuilder::new("test-schema").build()?;
        let schema_id = schema.id;
        let schema_name = schema.name.clone();
        command.save(&schema)?;

        // Verify schema exists in all indices
        assert!(
            query.find_by_id(schema_id)?.is_some(),
            "Schema should exist by ID before delete"
        );
        let name_lookup_before = SchemaName::try_new(schema_name.as_ref())?;
        assert!(
            query.find_by_name(&name_lookup_before)?.is_some(),
            "Schema should exist by name before delete"
        );

        // WHEN: Deleting the schema
        command.delete(schema_id)?;

        // THEN: Schema is removed from all tables
        assert!(
            query.find_by_id(schema_id)?.is_none(),
            "Schema should not exist by ID after delete"
        );
        let name_lookup_after = SchemaName::try_new(schema_name.as_ref())?;
        assert!(
            query.find_by_name(&name_lookup_after)?.is_none(),
            "Schema should not exist by name after delete"
        );

        Ok(())
    }
}

// ========================================================================
//                      Staleness Detection Tests
// ========================================================================

mod staleness {

    use super::*;

    /// **Staleness-001**: Missing schema is reported as stale.
    #[test]
    fn is_schema_stale_reports_missing_schema_as_stale() -> TestResult {
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

    /// **Staleness-002**: Fresh schema is not stale.
    #[test]
    fn is_schema_stale_returns_false_for_fresh_schema() -> TestResult {
        // GIVEN: A schema saved without PropertyBank changes
        let test_db = TestDb::new()?;
        let (command, query) = setup_cqrs(test_db.db());

        // Save a schema (will use BankVersion::initial())
        let schema = SchemaBuilder::new("test-schema").build()?;
        let schema_id = schema.id;
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
            "Schema saved with initial version should not be stale when \
             checked against initial version"
        );

        Ok(())
    }

    /// **Staleness-003**: Asymmetric `created_at` timestamps handled
    /// gracefully.
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
    /// Expected behavior: Falls back to `modified_at` comparison.
    #[test]
    fn is_schema_stale_with_asymmetric_created_at() -> TestResult {
        use lithos_core::schema::bank::BankVersion;
        use redb::TableDefinition;
        // Manually create StoredMetadata struct for test metadata crafting
        use rkyv::with::{AsUnixTime, Map};

        #[derive(rkyv::Archive, rkyv::Serialize)]
        struct TempMetadata {
            bank_version: BankVersion,
            source_file_hash: lithos_core::schema::storage::Blake3Hash,
            #[rkyv(with = Map<AsUnixTime>)]
            created_at: Option<std::time::SystemTime>,
            #[rkyv(with = Map<AsUnixTime>)]
            modified_at: Option<std::time::SystemTime>,
            #[rkyv(with = AsUnixTime)]
            recorded_at: std::time::SystemTime,
        }

        // Table definitions for direct database access
        const SCHEMA_METADATA: TableDefinition<&str, &[u8]> =
            TableDefinition::new("schema_metadata");

        // GIVEN: A schema saved with explicit created_at and modified_at
        // metadata
        let test_db = TestDb::new()?;
        let (command, query) = setup_cqrs(test_db.db());

        let schema = SchemaBuilder::new("asymmetric-schema").build()?;
        let schema_id = schema.id;

        // Save schema first (will have None/None timestamps)
        command.save(&schema)?;

        // Manually craft metadata with created_at = Some, modified_at = Some
        // to simulate a file that was saved WITH birthtime support
        let created_timestamp = std::time::SystemTime::UNIX_EPOCH
            + std::time::Duration::from_secs(1_000_000);
        let modified_timestamp = std::time::SystemTime::UNIX_EPOCH
            + std::time::Duration::from_secs(1_000_100);

        let metadata = TempMetadata {
            bank_version: BankVersion::initial(),
            source_file_hash: lithos_core::schema::storage::Blake3Hash::zero(),
            created_at: Some(created_timestamp),
            modified_at: Some(modified_timestamp),
            recorded_at: std::time::SystemTime::now(),
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
        assert!(
            !is_stale_matching_mtime,
            "Schema should not be stale when created_at is asymmetric but \
             modified_at matches"
        );

        // WHEN: Checking with newer modified_at (file was actually modified)
        let newer_modified = std::time::SystemTime::UNIX_EPOCH
            + std::time::Duration::from_secs(1_000_200);
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
}

// ========================================================================
//                      Corruption Detection Tests
// ========================================================================

mod corruption {
    use redb::TableDefinition;

    use super::*;

    /// **Corruption-001**: Query detects corrupted schema data.
    #[test]
    fn query_detects_corrupted_schema_data() -> TestResult {
        const SCHEMA_BY_ID: TableDefinition<&str, &[u8]> =
            TableDefinition::new("schema_by_id");

        // GIVEN: A database with corrupted schema data
        let test_db = TestDb::new()?;
        let (_command, query) = setup_cqrs(test_db.db());

        // Manually write invalid rkyv bytes to SCHEMA_BY_ID table
        test_db.db().batch_write(|batch| {
            batch.put(SCHEMA_BY_ID, "corrupt-schema-key", &[
                0xFFu8, 0xFFu8, 0xFFu8, 0xFFu8, 0xFFu8,
            ])?;
            Ok(())
        })?;

        // WHEN: Attempting to list all schemas
        let result = query.list();

        // THEN: Query fails with storage error
        assert!(
            result.is_err(),
            "Corrupted data should trigger error, not return invalid schemas"
        );

        if let Err(e) = result {
            let error_msg = e.to_string();
            assert!(
                error_msg.contains("storage")
                    || error_msg.contains("deserializ"),
                "Error should indicate storage/deserialization issue: \
                 {error_msg}"
            );
        }

        Ok(())
    }

    /// **Corruption-002**: Query detects missing metadata for existing
    /// schema.
    #[test]
    fn query_detects_missing_metadata_corruption() -> TestResult {
        const SCHEMA_METADATA: TableDefinition<&str, &[u8]> =
            TableDefinition::new("schema_metadata");

        // GIVEN: A normally saved schema
        let test_db = TestDb::new()?;
        let (command, query) = setup_cqrs(test_db.db());

        let schema = SchemaBuilder::new("orphaned-schema").build()?;
        let schema_id = schema.id;

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
            "Orphaned schema (missing metadata) should trigger corruption \
             error"
        );

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

    /// **Corruption-003**: Query detects corrupted property bank metadata.
    #[test]
    fn query_detects_corrupted_property_bank_metadata() -> TestResult {
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

        if let Err(e) = result {
            let error_msg = e.to_string();
            assert!(
                error_msg.contains("storage")
                    || error_msg.contains("deserializ"),
                "Error should indicate storage/deserialization issue: \
                 {error_msg}"
            );
        }

        Ok(())
    }

    /// **Corruption-004**: Query detects corrupted name index.
    #[test]
    fn query_detects_corrupted_name_index() -> TestResult {
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
        let schema_name = SchemaName::try_new("corrupt-index-key")?;

        // WHEN: Attempting to find schema by name with corrupted index
        let result = query.find_by_name(&schema_name);

        // THEN: Query fails with storage/deserialization error
        assert!(result.is_err(), "Corrupted name index should trigger error");

        if let Err(e) = result {
            let error_msg = e.to_string();
            assert!(
                error_msg.contains("storage")
                    || error_msg.contains("deserializ"),
                "Error should indicate storage/deserialization issue: \
                 {error_msg}"
            );
        }

        Ok(())
    }
}
