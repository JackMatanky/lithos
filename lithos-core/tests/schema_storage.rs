//! Integration test suite for Schema storage via Repository trait.
//!
//! # Summary
//! - Validates the unified Repository trait for schema persistence.
//! - Tests roundtrip (save/load), lookup (by ID, by name, list), durability
//!   (restart), batching, and regression scenarios.
//! - Covers `PropertyBank` and Schema storage through redb backend.
//! - Excludes: File I/O (tested in loader), inheritance resolution (tested in
//!   loader), CQRS patterns (migrated to Repository).
//!
//! # Setup
//! - Uses `TestDb` fixture for isolated database instances (tempfile-backed).
//! - Uses `PropertyBuilder` and `SchemaBuilder` from `common` module.
//! - Each test creates fresh database to ensure isolation.
//!
//! # Data Model
//! - Inputs: `Schema` aggregates with properties, `PropertyBank` with
//!   registered properties.
//! - Outputs: Same types retrieved from storage via Repository queries.
//! - Assumptions: Valid UUIDs, lowercase schema names, well-formed property
//!   specs.
//!
//! # Scenarios
//! - Happy path: Save schemas/banks, retrieve by ID/name, list all, batch
//!   operations.
//! - Edge cases: Empty schemas, separate saves (non-batch), database restarts.
//! - Error paths: Missing schemas return None, deleted schemas verified removed
//!   (deletion deferred).

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
    reason = "Test file organization prioritizes readability over arbitrary \
              ordering rules."
)]

mod common;

use std::collections::HashMap;

use common::*;
use lithos_core::schema::{
    aggregate::Schema,
    bank::PropertyBank,
    identifier::{SchemaId, SchemaName},
    property::{PropertyId, PropertyMap},
    storage::Repository as _,
};
use uuid::Uuid;

// ========================================================================
//                          Roundtrip Tests
// ========================================================================

mod roundtrip_tests {
    use super::*;

    // Test fixture UUIDs (pub for use in other test modules)
    pub(super) const TEST_PROPERTY_ID_A: Uuid =
        Uuid::from_u128(0x018C_0000_0000_7000_8000_0000_0000_0A01);

    /// Integration test for `PropertyBank` save/load roundtrip.
    ///
    /// # Purpose
    /// Validates that `PropertyBank` can be persisted and retrieved without
    /// data loss.
    ///
    /// # Inputs
    /// `PropertyBank` with one boolean property ("status").
    ///
    /// # Expected Behavior
    /// - `save_property_bank()` persists bank to redb.
    /// - `get_property_bank()` returns `Some(bank)` after save.
    ///
    /// # Failure Modes
    /// - Bank not found after save (returns `None`).
    ///
    /// # Observability
    /// Asserts `PropertyBank` exists after retrieval.
    #[test]
    fn property_bank_roundtrip() -> TestResult {
        let test_db = TestDb::new()?;
        let repository = setup_repository(test_db.db());

        // Create and save property bank
        let (prop_name, prop) = PropertyBuilder::new("status")
            .id(PropertyId::from_uuid(TEST_PROPERTY_ID_A))
            .build_bool()?;
        let mut properties = PropertyMap::new();
        properties.insert(prop_name.clone(), prop);
        let bank = PropertyBank::from(properties);
        repository.save_property_bank(&bank)?;

        // Retrieve and verify
        let loaded = repository.get_property_bank()?;
        assert!(loaded.is_some(), "PropertyBank should exist");

        Ok(())
    }

    /// Integration test for Schema save/load roundtrip.
    ///
    /// # Purpose
    /// Validates that Schema aggregates can be persisted and retrieved by ID.
    ///
    /// # Inputs
    /// Schema named "task" with one string property ("title").
    ///
    /// # Expected Behavior
    /// - `save_schemas()` persists schema to redb.
    /// - `find_schema_by_id()` returns `Some(schema)` after save.
    ///
    /// # Failure Modes
    /// - Schema not found by ID after save (returns `None`).
    ///
    /// # Observability
    /// Asserts Schema exists after retrieval by ID.
    #[test]
    fn schema_roundtrip() -> TestResult {
        let test_db = TestDb::new()?;
        let repository = setup_repository(test_db.db());

        // Create schema
        let (prop_name, prop) =
            PropertyBuilder::new("title").build_string_default()?;
        let mut props = HashMap::new();
        props.insert(prop_name, prop);
        let schema = Schema::new(
            SchemaId::new(),
            SchemaName::try_new("task")?,
            Vec::new(),
            vec![],
            PropertyMap::from(props),
        );
        let schema_id = *schema.id();

        // Save
        repository.save_schemas(&[schema])?;

        // Retrieve by ID
        let loaded = repository.find_schema_by_id(schema_id)?;
        assert!(loaded.is_some(), "Schema should exist");

        Ok(())
    }
}

// ========================================================================
//                          Lookup Tests
// ========================================================================

mod lookup_tests {
    use super::*;

    /// Integration test for Schema lookup by name.
    ///
    /// # Purpose
    /// Validates that schemas can be retrieved by `SchemaName` via name index.
    ///
    /// # Inputs
    /// Schema named "project" saved to storage.
    ///
    /// # Expected Behavior
    /// - `find_schema_id_by_name()` returns `Some(schema_id)` for existing
    ///   schema.
    /// - Name index (`SCHEMA_ID_BY_NAME` table) correctly maps name to ID.
    ///
    /// # Failure Modes
    /// - Name not found in index (returns `None`).
    ///
    /// # Observability
    /// Asserts found ID matches original schema ID.
    #[test]
    fn schema_find_by_name() -> TestResult {
        let test_db = TestDb::new()?;
        let repository = setup_repository(test_db.db());

        // Create and save
        let (prop_name, prop) =
            PropertyBuilder::new("title").build_string_default()?;
        let mut props = HashMap::new();
        props.insert(prop_name, prop);
        let schema = Schema::new(
            SchemaId::new(),
            SchemaName::try_new("project")?,
            Vec::new(),
            vec![],
            PropertyMap::from(props),
        );
        let schema_id = *schema.id();
        repository.save_schemas(&[schema])?;

        // Find by name
        let name = SchemaName::try_new("project")?;
        let found_id = repository.find_schema_id_by_name(&name)?;
        assert_eq!(found_id, Some(schema_id));

        Ok(())
    }

    /// Integration test for listing all schemas.
    ///
    /// # Purpose
    /// Validates that multiple schemas can be retrieved as a list.
    ///
    /// # Inputs
    /// Two schemas saved with different properties ("item" and "person").
    ///
    /// # Expected Behavior
    /// - `list_schemas()` returns all saved schemas (count=2).
    /// - Deserialization works correctly for multiple entries.
    /// - Previously failed with "subtree pointer overran range" due to wrong
    ///   API call.
    ///
    /// # Failure Modes
    /// - Wrong count returned.
    /// - Deserialization error when multiple schemas exist.
    ///
    /// # Observability
    /// Asserts exact count of 2 schemas returned.
    ///
    /// **Note**: This test previously exposed a critical bug (wrong use of
    /// `list_owned` vs `list_key_value_pairs`). Kept as regression test.
    #[test]
    fn schema_list() -> TestResult {
        let test_db = TestDb::new()?;
        let repository = setup_repository(test_db.db());

        // Save multiple schemas
        let (title_name, title_prop) =
            PropertyBuilder::new("title").build_string_default()?;
        let mut task_props = HashMap::new();
        task_props.insert(title_name, title_prop);
        let schema1 = Schema::new(
            SchemaId::new(),
            SchemaName::try_new("task")?,
            Vec::new(),
            vec![],
            PropertyMap::from(task_props),
        );
        let (content_name, content_prop) =
            PropertyBuilder::new("content").build_string_default()?;
        let mut note_props = HashMap::new();
        note_props.insert(content_name, content_prop);
        let schema2 = Schema::new(
            SchemaId::new(),
            SchemaName::try_new("note")?,
            Vec::new(),
            vec![],
            PropertyMap::from(note_props),
        );
        repository.save_schemas(&[schema1, schema2])?;

        // List all
        let all = repository.list_schemas()?;
        assert_eq!(all.len(), 2, "Should have 2 schemas");

        Ok(())
    }
}

// ========================================================================
//                          Durability Tests (Phase 7.1)
// ========================================================================

mod durability_tests {
    use super::*;
    use crate::roundtrip_tests::TEST_PROPERTY_ID_A;

    /// Integration test for `PropertyBank` durability across restarts.
    ///
    /// # Purpose
    /// Validates that `PropertyBank` persists correctly across database
    /// close/reopen.
    ///
    /// # Inputs
    /// `PropertyBank` with one boolean property, saved before database reopen.
    ///
    /// # Expected Behavior
    /// - Data survives database close/reopen cycle (simulates process restart).
    /// - Property count and IDs match after reload.
    /// - rkyv serialization stable across sessions.
    ///
    /// # Failure Modes
    /// - `PropertyBank` not found after restart (data loss).
    /// - Property count mismatch (corruption).
    ///
    /// # Observability
    /// Asserts property count=1 and property ID matches original.
    #[test]
    fn property_bank_survives_restart() -> TestResult {
        let mut test_db = TestDb::new()?;
        let repository = setup_repository(test_db.db());

        // Create and save property bank
        let (prop_name, prop) = PropertyBuilder::new("status")
            .id(PropertyId::from_uuid(TEST_PROPERTY_ID_A))
            .build_bool()?;
        let prop_id = prop.id();
        let mut properties = PropertyMap::new();
        properties.insert(prop_name.clone(), prop);
        let bank = PropertyBank::from(properties);
        repository.save_property_bank(&bank)?;

        // Drop repository to release Arc reference before reopen
        drop(repository);

        // Reopen database (simulates process restart)
        let db = test_db.reopen()?;
        let repository_after_restart = setup_repository(&db);

        // Verify PropertyBank survived restart
        let loaded = repository_after_restart
            .get_property_bank()?
            .expect("PropertyBank should exist after restart");

        // Verify property count
        let loaded_count = loaded.all().count();
        assert_eq!(loaded_count, 1, "Should have exactly 1 property");

        // Verify specific property by name
        let loaded_prop =
            loaded.get(&prop_name).expect("Property should exist by name");
        assert_eq!(loaded_prop.id(), prop_id, "Property ID should match");

        Ok(())
    }

    /// Integration test for Schema durability across restarts.
    ///
    /// # Purpose
    /// Validates that Schema aggregates persist correctly across database
    /// close/reopen.
    ///
    /// # Inputs
    /// Schema named "person" with one property, saved before database reopen.
    ///
    /// # Expected Behavior
    /// - Schema survives database close/reopen cycle.
    /// - Name, properties, and name index all persist correctly.
    /// - rkyv-serialized `HashMap` stable across sessions.
    ///
    /// # Failure Modes
    /// - Schema not found after restart (data loss).
    /// - Name or property count mismatch (corruption).
    ///
    /// # Observability
    /// Asserts name matches, property count=1, and name lookup works after
    /// restart.
    #[test]
    fn schema_survives_restart() -> TestResult {
        let mut test_db = TestDb::new()?;
        let repository = setup_repository(test_db.db());

        // Create schema with properties
        let (prop_name, prop) =
            PropertyBuilder::new("title").build_string_default()?;
        let mut props = HashMap::new();
        props.insert(prop_name, prop);
        let expected_prop_count = 1;

        let schema = Schema::new(
            SchemaId::new(),
            SchemaName::try_new("person")?,
            Vec::new(),
            vec![],
            PropertyMap::from(props),
        );
        let schema_id = *schema.id();
        let schema_name = schema.name().clone();

        // Save schema
        repository.save_schemas(&[schema])?;

        // Drop repository to release Arc reference before reopen
        drop(repository);

        // Reopen database (simulates process restart)
        let db = test_db.reopen()?;
        let repository_after_restart = setup_repository(&db);

        // Verify schema survived restart
        let loaded = repository_after_restart
            .find_schema_by_id(schema_id)?
            .expect("Schema should exist after restart");
        assert_eq!(
            loaded.name().as_str(),
            schema_name.as_str(),
            "Schema name should match"
        );
        assert_eq!(
            loaded.properties().len(),
            expected_prop_count,
            "Property count should match"
        );
        assert!(loaded.parents().is_empty(), "Parents should be empty");

        // Verify can still find by name
        let found_id =
            repository_after_restart.find_schema_id_by_name(&schema_name)?;
        assert_eq!(
            found_id,
            Some(schema_id),
            "Should find schema by name after restart"
        );

        Ok(())
    }
}

// ========================================================================
//                          Batch Operations Tests (Phase 7.1)
// ========================================================================

mod batch_operations {
    use super::*;

    /// Integration test for batch save atomicity (happy path).
    ///
    /// # Purpose
    /// Validates that multiple schemas can be saved atomically in one batch.
    ///
    /// # Inputs
    /// Three schemas ("task", "project", "note") saved in single batch call.
    ///
    /// # Expected Behavior
    /// - All 3 schemas persist in single redb transaction.
    /// - All schemas retrievable after batch save.
    /// - Previously failed with "subtree pointer overran range" due to wrong
    ///   API.
    ///
    /// # Failure Modes
    /// - Partial save (only some schemas persist) - violates atomicity.
    /// - Deserialization error when multiple schemas exist.
    ///
    /// # Observability
    /// Asserts all 3 schemas exist individually and in list (count=3).
    ///
    /// **Note**: This test verifies happy path only. Negative testing (rollback
    /// on error) would require database mocking or intentional corruption.
    #[test]
    fn batch_save_is_atomic() -> TestResult {
        let test_db = TestDb::new()?;
        let repository = setup_repository(test_db.db());

        // Prepare batch with 3 valid schemas
        let (prop_name, prop) =
            PropertyBuilder::new("title").build_string_default()?;
        let mut props = HashMap::new();
        props.insert(prop_name, prop);

        let schema_a = Schema::new(
            SchemaId::new(),
            SchemaName::try_new("task")?,
            Vec::new(),
            vec![],
            PropertyMap::from(props.clone()),
        );
        let id_a = *schema_a.id();

        let schema_b = Schema::new(
            SchemaId::new(),
            SchemaName::try_new("project")?,
            Vec::new(),
            vec![],
            PropertyMap::from(props.clone()),
        );
        let id_b = *schema_b.id();

        let schema_c = Schema::new(
            SchemaId::new(),
            SchemaName::try_new("note")?,
            Vec::new(),
            vec![],
            PropertyMap::from(props),
        );
        let id_c = *schema_c.id();

        // Save batch
        repository.save_schemas(&[schema_a, schema_b, schema_c])?;

        // Verify ALL schemas were saved (atomic commit)
        let loaded_a = repository.find_schema_by_id(id_a)?;
        assert!(loaded_a.is_some(), "Schema A should be saved");

        let loaded_b = repository.find_schema_by_id(id_b)?;
        assert!(loaded_b.is_some(), "Schema B should be saved");

        let loaded_c = repository.find_schema_by_id(id_c)?;
        assert!(loaded_c.is_some(), "Schema C should be saved");

        // Verify all schemas are in the database
        let all_schemas = repository.list_schemas()?;
        assert_eq!(
            all_schemas.len(),
            3,
            "All 3 schemas should be saved atomically"
        );

        Ok(())
    }
}

// ========================================================================
//                        Regression Tests
// ========================================================================

/// Regression tests for specific bugs that were fixed.
///
/// These tests ensure previously fixed bugs don't resurface.
mod regression_tests {
    use super::*;

    /// Regression test for empty schemas batch save and list.
    ///
    /// # Purpose
    /// Prevents regression of rkyv deserialization bug with minimal schemas.
    ///
    /// # Inputs
    /// Two schemas with NO properties (empty `HashMap`) saved in batch.
    ///
    /// # Expected Behavior
    /// - Batch save succeeds for empty schemas.
    /// - `list_schemas()` works correctly (returns count=2).
    /// - Individual lookups by ID succeed.
    ///
    /// # Failure Modes
    /// - "subtree pointer overran range" error (original bug).
    /// - Wrong API call (`list_owned` trying to deserialize tuples from
    ///   values).
    ///
    /// # Observability
    /// Asserts list count=2 and both schemas findable by ID.
    ///
    /// **Bug Context**: Previously failed due to wrong API (`list_owned` vs
    /// `list_key_value_pairs`). This edge case exposed the issue most clearly.
    #[test]
    fn empty_schemas_batch_save_and_list() -> TestResult {
        let test_db = TestDb::new()?;
        let repository = setup_repository(test_db.db());

        // Create 2 schemas with NO properties (edge case)
        let schema1 = Schema::new(
            SchemaId::new(),
            SchemaName::try_new("empty_schema1")?,
            Vec::new(),
            vec![],
            PropertyMap::new(),
        );
        let id1 = *schema1.id();

        let schema2 = Schema::new(
            SchemaId::new(),
            SchemaName::try_new("empty_schema2")?,
            Vec::new(),
            vec![],
            PropertyMap::new(),
        );
        let id2 = *schema2.id();

        // Save both in batch
        repository.save_schemas(&[schema1, schema2])?;

        // List all schemas (this is where the bug manifested)
        let all_schemas = repository.list_schemas()?;
        assert_eq!(all_schemas.len(), 2, "Should list both empty schemas");

        // Verify individual loads still work
        let loaded1 = repository.find_schema_by_id(id1)?;
        assert!(loaded1.is_some(), "Schema 1 should exist");

        let loaded2 = repository.find_schema_by_id(id2)?;
        assert!(loaded2.is_some(), "Schema 2 should exist");

        Ok(())
    }

    /// Regression test for separate (non-batch) saves then list.
    ///
    /// # Purpose
    /// Ensures separate save operations don't cause deserialization issues.
    ///
    /// # Inputs
    /// Two schemas saved in separate `save_schemas()` calls (not batched).
    ///
    /// # Expected Behavior
    /// - First save succeeds, schema1 retrievable.
    /// - Second save succeeds, schema2 retrievable.
    /// - `list_schemas()` returns both (count=2) after separate saves.
    ///
    /// # Failure Modes
    /// - Second save corrupts first schema's data.
    /// - List fails to deserialize after separate saves.
    ///
    /// # Observability
    /// Asserts schema1 exists after first save, both exist after second save,
    /// and list returns count=2.
    ///
    /// **Bug Context**: Tests different transaction pattern than batch saves,
    /// ensuring the fix works for incremental saves too.
    #[test]
    fn separate_saves_then_list() -> TestResult {
        let test_db = TestDb::new()?;
        let repository = setup_repository(test_db.db());

        // Create and save schema 1
        let (field1_name, field1_prop) =
            PropertyBuilder::new("field1").build_string_default()?;
        let mut schema1_props = HashMap::new();
        schema1_props.insert(field1_name, field1_prop);

        let schema1 = Schema::new(
            SchemaId::new(),
            SchemaName::try_new("separate_schema1")?,
            Vec::new(),
            vec![],
            PropertyMap::from(schema1_props),
        );
        let id1 = *schema1.id();
        repository.save_schemas(&[schema1])?;

        // Verify schema 1 loads
        let loaded_schema1 = repository.find_schema_by_id(id1)?;
        assert!(
            loaded_schema1.is_some(),
            "Schema 1 should exist after first save"
        );

        // Create and save schema 2 separately
        let (field2_name, field2_prop) =
            PropertyBuilder::new("field2").build_string_default()?;
        let mut schema2_props = HashMap::new();
        schema2_props.insert(field2_name, field2_prop);

        let schema2 = Schema::new(
            SchemaId::new(),
            SchemaName::try_new("separate_schema2")?,
            Vec::new(),
            vec![],
            PropertyMap::from(schema2_props),
        );
        let id2 = *schema2.id();
        repository.save_schemas(&[schema2])?;

        // Verify both schemas can be listed
        let all_schemas = repository.list_schemas()?;
        assert_eq!(
            all_schemas.len(),
            2,
            "Should list both separately saved schemas"
        );

        // Verify both load individually
        let reloaded_schema1 = repository.find_schema_by_id(id1)?;
        assert!(reloaded_schema1.is_some(), "Schema 1 should still exist");

        let reloaded_schema2 = repository.find_schema_by_id(id2)?;
        assert!(reloaded_schema2.is_some(), "Schema 2 should exist");

        Ok(())
    }

    /// Regression test for 2-phase deserialization pattern.
    ///
    /// # Purpose
    /// Validates the defensive fix separating redb `AccessGuard` iteration from
    /// rkyv deserialization.
    ///
    /// # Inputs
    /// Two schemas serialized to rkyv bytes, then deserialized sequentially.
    ///
    /// # Expected Behavior
    /// - Phase 1: Collect all byte buffers into `Vec`.
    /// - Phase 2: Deserialize after `AccessGuards` dropped.
    /// - Both schemas deserialize successfully.
    ///
    /// # Failure Modes
    /// - Deserialization fails in loop (`AccessGuard`/rkyv conflict).
    /// - Memory corruption from mixing iteration and deserialization.
    ///
    /// # Observability
    /// Asserts successful deserialization of 2 schemas (count=2).
    ///
    /// **Bug Context**: This pattern was applied defensively to `db/reader.rs`
    /// functions (`scan_table`, `scan_table_key_value`, `scan_range`) to
    /// prevent potential `AccessGuard` lifetime issues, though the actual
    /// bug was wrong API usage.
    #[test]
    fn sequential_deserialization_pattern() -> TestResult {
        use rkyv::{access, deserialize};

        // Create 2 schemas
        let (seq_field1_name, seq_field1) =
            PropertyBuilder::new("seq_field1").build_string_default()?;
        let mut seq_schema1_props = HashMap::new();
        seq_schema1_props.insert(seq_field1_name, seq_field1);
        let schema1 = Schema::new(
            SchemaId::new(),
            SchemaName::try_new("seq_schema1")?,
            Vec::new(),
            vec![],
            PropertyMap::from(seq_schema1_props),
        );

        let (seq_field2_name, seq_field2) =
            PropertyBuilder::new("seq_field2").build_string_default()?;
        let mut seq_schema2_props = HashMap::new();
        seq_schema2_props.insert(seq_field2_name, seq_field2);
        let schema2 = Schema::new(
            SchemaId::new(),
            SchemaName::try_new("seq_schema2")?,
            Vec::new(),
            vec![],
            PropertyMap::from(seq_schema2_props),
        );

        // Serialize both
        let bytes1 = rkyv::to_bytes::<rkyv::rancor::Error>(&schema1)?;
        let bytes2 = rkyv::to_bytes::<rkyv::rancor::Error>(&schema2)?;

        // Simulate table scan: collect all bytes first
        let all_bytes = vec![bytes1, bytes2];
        let mut results = Vec::new();

        // Then deserialize sequentially (2-phase pattern)
        for bytes in &all_bytes {
            let archived =
                access::<rkyv::Archived<Schema>, rkyv::rancor::Error>(bytes)?;
            let deserialized =
                deserialize::<Schema, rkyv::rancor::Error>(archived)?;
            results.push(deserialized);
        }

        assert_eq!(results.len(), 2, "Should deserialize both schemas");

        Ok(())
    }
}
