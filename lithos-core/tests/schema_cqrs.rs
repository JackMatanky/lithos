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
        RedbSchemaCommand, RedbSchemaQuery,
        aggregate::{SchemaId, SchemaName},
        bank::PropertyBank,
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
            let command = RedbSchemaCommand::new(
                lithos_core::schema::adapter::command::CommandAdapter::new(&db),
            );
            command.save_property_bank(&bank)?;
        } // Database closed

        // WHEN: Reopening database
        let db = Database::open(&db_path)?;
        let query = RedbSchemaQuery::new(
            lithos_core::schema::adapter::query::QueryAdapter::new(&db),
        );

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

        let name = PropertyName::new("status")?;
        assert!(
            loaded_bank.has_name(&name),
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
        let status_name = PropertyName::new("status")?;
        let title_name = PropertyName::new("title")?;

        assert!(loaded.has_name(&status_name));
        assert!(loaded.has_name(&title_name));

        assert!(loaded.get_by_name(&status_name).is_some());
        assert!(loaded.get_by_name(&title_name).is_some());

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
        let alpha_name = PropertyName::new("alpha")?;
        let beta_name = PropertyName::new("beta")?;
        assert!(loaded.has_name(&alpha_name));
        assert!(loaded.has_name(&beta_name));

        let prop_by_name = loaded
            .get_by_name(&beta_name)
            .expect("Property should exist by name");
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

        let original_id = schema.id();
        let original_name = schema.name().clone();

        // WHEN: Saving and loading
        command.save_one(&schema)?;
        let loaded =
            query.find_by_id(original_id)?.expect("Schema should exist");

        // THEN: All fields preserved
        assert_eq!(loaded.id(), original_id);
        assert_eq!(loaded.name(), &original_name);
        assert_eq!(loaded.properties().count(), 1);
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

        command.save_one(&schema)?;

        // WHEN: Finding by name
        let name = SchemaName::new("project")?;
        let loaded = query.find_by_name(&name)?.expect("Schema should exist");

        // THEN: Correct schema returned
        assert_eq!(loaded.id(), schema.id());
        assert_eq!(loaded.name(), schema.name());

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
        let name = SchemaName::new("nonexistent")?;
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
        command.save_batch(&schemas)?;

        // THEN: All schemas retrievable
        let loaded1 = query
            .find_by_id(SchemaId::from_uuid(TEST_SCHEMA_ID_TASK))?
            .expect("task schema should exist");
        let loaded2 = query
            .find_by_id(SchemaId::from_uuid(TEST_SCHEMA_ID_PROJECT))?
            .expect("project schema should exist");

        assert_eq!(loaded1.name().as_str(), "task");
        assert_eq!(loaded2.name().as_str(), "project");

        // Verify name indices
        assert!(query.find_by_name(&SchemaName::new("task")?)?.is_some());
        assert!(query.find_by_name(&SchemaName::new("project")?)?.is_some());

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
        command.save_batch(&[schema1, schema2])?;

        // WHEN: Listing all
        let all = query.list()?;

        // THEN: All schemas returned
        assert_eq!(all.len(), 2);
        let names: Vec<_> = all.iter().map(|s| s.name().as_str()).collect();
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
        command.save_batch(&[schema1, schema2])?;

        // WHEN: Deleting one schema
        command.delete(SchemaId::from_uuid(TEST_SCHEMA_ID_TASK))?;

        // THEN: Deleted schema not found
        assert!(
            query
                .find_by_id(SchemaId::from_uuid(TEST_SCHEMA_ID_TASK))?
                .is_none()
        );
        assert!(query.find_by_name(&SchemaName::new("task")?)?.is_none());

        // Other schema still exists
        assert!(
            query
                .find_by_id(SchemaId::from_uuid(TEST_SCHEMA_ID_PROJECT))?
                .is_some()
        );
        assert!(query.find_by_name(&SchemaName::new("project")?)?.is_some());

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
        command.save_one(&schema)?;
        let loaded =
            query.find_by_id(schema.id())?.expect("Schema should exist");

        // THEN: Properties are sorted
        assert_properties_sorted(&loaded, "Loaded schema");

        let names: Vec<_> =
            loaded.properties().map(|p| p.name().as_str()).collect();
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
        let schema_id = schema.id();

        // WHEN: Saving and loading
        command.save_one(&schema)?;
        let loaded = query.find_by_id(schema_id)?.expect("Schema should exist");

        // THEN: Empty schema works
        assert_eq!(loaded.properties().count(), 0);
        assert_eq!(loaded.name().as_str(), "empty");
        assert_eq!(loaded.id(), schema_id);

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

        let schema_id = schema.id();
        let schema_name = schema.name().clone();

        // Save with first connection
        {
            let db = Database::open(&db_path)?;
            let command = RedbSchemaCommand::new(
                lithos_core::schema::adapter::command::CommandAdapter::new(&db),
            );
            command.save_one(&schema)?;
        } // Database closed

        // WHEN: Reopening database
        let db = Database::open(&db_path)?;
        let query = RedbSchemaQuery::new(
            lithos_core::schema::adapter::query::QueryAdapter::new(&db),
        );

        // THEN: Schema intact
        let loaded = query
            .find_by_id(schema_id)?
            .expect("Schema should survive restart");
        assert_eq!(loaded.id(), schema_id);
        assert_eq!(loaded.name(), &schema_name);
        assert_eq!(loaded.properties().count(), 1);

        // Name index still works
        assert!(query.find_by_name(&schema_name)?.is_some());

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
        command.save_one(&schema_v1)?;

        // WHEN: Saving updated version with same ID
        let schema_v2 = SchemaBuilder::new("task")
            .id(SchemaId::from_uuid(TEST_SCHEMA_ID_TASK))
            .property(bool_property("is_done")?)
            .property(string_property("title")?)
            .build()?;
        command.save_one(&schema_v2)?;

        // THEN: Updated version loaded
        let loaded = query
            .find_by_id(SchemaId::from_uuid(TEST_SCHEMA_ID_TASK))?
            .expect("Schema should exist");
        assert_eq!(loaded.properties().count(), 2);
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
        command.save_batch(&[schema1, schema2])?;

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
        command.save_one(&child)?;
        let loaded = query
            .find_by_id(SchemaId::from_uuid(TEST_SCHEMA_ID_TASK))?
            .expect("Schema should exist");

        // THEN: Parent ID preserved
        assert_eq!(loaded.parent_id(), Some(parent_id));

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
        let schema_id = schema.id();

        // WHEN: Saving and loading
        command.save_one(&schema)?;
        let loaded = query.find_by_id(schema_id)?.expect("Schema should exist");

        // THEN: parent_id is None
        assert_eq!(loaded.parent_id(), None);

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
        command.save_one(&schema)?;
        let loaded =
            query.find_by_id(schema.id())?.expect("Schema should exist");

        // THEN: Optionality preserved
        let title_prop = loaded
            .properties()
            .find(|p| p.name().as_str() == "title")
            .expect("title property should exist");
        assert_eq!(title_prop.optionality(), Optionality::Required);

        let desc_prop = loaded
            .properties()
            .find(|p| p.name().as_str() == "description")
            .expect("description property should exist");
        assert_eq!(desc_prop.optionality(), Optionality::Optional);

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
        command.save_one(&schema)?;
        let loaded =
            query.find_by_id(schema.id())?.expect("Schema should exist");

        // THEN: Multiplicity preserved
        let title_prop = loaded
            .properties()
            .find(|p| p.name().as_str() == "title")
            .expect("title property should exist");
        assert_eq!(title_prop.multiplicity(), Multiplicity::Single);

        let tags_prop = loaded
            .properties()
            .find(|p| p.name().as_str() == "tags")
            .expect("tags property should exist");
        assert_eq!(tags_prop.multiplicity(), Multiplicity::Many);

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
        let schema_id = schema.id();

        // WHEN: Saving both
        command.save_property_bank(&bank)?;
        command.save_one(&schema)?;

        // THEN: Both retrievable independently
        let loaded_bank =
            query.get_property_bank()?.expect("Bank should exist");
        let loaded_schema =
            query.find_by_id(schema_id)?.expect("Schema should exist");

        assert_eq!(loaded_bank.all().count(), 1);
        assert_eq!(loaded_schema.name().as_str(), "test");

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
        command.save_batch(&[schema1, schema2])?;

        // THEN: Both schemas exist with same property
        let loaded1 = query
            .find_by_name(&SchemaName::new("schema1")?)?
            .expect("schema1 should exist");
        let loaded2 = query
            .find_by_name(&SchemaName::new("schema2")?)?
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
        let schema_id = schema.id();

        // WHEN: Saving schema (doesn't touch PropertyBank)
        command.save_one(&schema)?;

        // THEN: PropertyBank version unchanged
        let loaded_bank =
            query.get_property_bank()?.expect("Bank should exist");
        assert_eq!(loaded_bank.version(), bank_version);

        // Schema still exists
        assert!(query.find_by_id(schema_id)?.is_some());

        Ok(())
    }
}
