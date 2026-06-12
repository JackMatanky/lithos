//! Integration tests for `BaseSchemaProcessor` stale reference handling.

#![expect(
    clippy::panic,
    reason = "Integration tests use assertions which panic on failure."
)]
#![expect(
    clippy::panic_in_result_fn,
    reason = "Integration tests use assertions which panic on failure."
)]
#![expect(
    clippy::tests_outside_test_module,
    reason = "Integration tests are top-level by default."
)]

mod common;

use std::path::Path;

use common::*;
use lithos_core::{
    fs::{DirPath, FileMetadata, FileNode, FilePath, FileReader},
    schema::{
        base_processor::{BaseSchemaProcessor, BaseSchemaResolution},
        property::PropertyName,
        property_bank_processor::PropertyBankProcessor,
        property_spec::PropertySpec,
        repository::{ReadRepository as _, WriteRepository as _},
    },
};
use tempfile::TempDir;

/// Write a file to the test directory.
fn write_file(root: &Path, relative: &str, content: &str) -> TestResult {
    let path = root.join(relative);
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    std::fs::write(&path, content)?;
    Ok(())
}

fn make_fs_file(real_path: std::path::PathBuf) -> TestResult<FileNode> {
    let path = FilePath::try_new(real_path)?;
    let std_meta = std::fs::metadata(path.as_path())?;
    let meta = FileMetadata::from(&std_meta);
    Ok(FileNode::new(path, meta))
}

mod resolution {
    use super::*;

    #[test]
    fn returns_stale_resolution_when_property_bank_changes() -> TestResult {
        let vault_dir = TempDir::new()?;
        let test_db = TestDb::new()?;
        let repository = setup_repository(test_db.store());
        let source = FileReader::new(vault_dir.path());
        let root = DirPath::try_new(vault_dir.path().to_path_buf())?;

        // 1. Cold Start
        write_file(
            vault_dir.path(),
            "schemas/property_bank.json",
            r#"{"$version": "1.0", "properties": {"status": {"type": "string"}}}"#,
        )?;
        write_file(
            vault_dir.path(),
            "schemas/task.json",
            r##"{"$version": "1.0", "properties": {"status": {"$ref": "#property_bank/status"}}}"##,
        )?;

        // Run bank processor
        let bank_file =
            make_fs_file(vault_dir.path().join("schemas/property_bank.json"))?;
        let bank_proc =
            PropertyBankProcessor::from_discovery(bank_file.clone(), &root)?;
        let bank_res = bank_proc.run(None, &source, &repository)?;

        // Run schema processor
        let task_file =
            make_fs_file(vault_dir.path().join("schemas/task.json"))?;
        let processor =
            BaseSchemaProcessor::from_discovery(task_file.clone(), &root)?;
        let res1 =
            processor.run(None, &source, &repository, Some(&bank_res))?;

        let BaseSchemaResolution::New {
            base: base1,
            ..
        } = res1
        else {
            panic!("Expected New resolution");
        };
        let schema_id = base1.id();
        let status_name = PropertyName::try_new("status")?;
        let Some(prop1) = base1.properties().get(&status_name) else {
            panic!("status property");
        };
        let prop1_id = prop1.id();

        // 2. Bank Change (Change property type)
        write_file(
            vault_dir.path(),
            "schemas/property_bank.json",
            r#"{"$version": "1.0", "properties": {"status": {"type": "boolean"}}}"#,
        )?;

        // Re-run bank processor with view
        let bank_path_key = bank_file.path().as_key(&root)?;
        let Some(bank_view) =
            repository.get_raw_property_bank_view(&bank_path_key)?
        else {
            panic!("bank view");
        };
        let bank_file_v2 =
            make_fs_file(vault_dir.path().join("schemas/property_bank.json"))?;
        let bank_proc2 =
            PropertyBankProcessor::from_discovery(bank_file_v2, &root)?;
        let bank_res2 =
            bank_proc2.run(Some(&bank_view), &source, &repository)?;

        // Re-run schema processor with view
        let task_path_key = task_file.path().as_key(&root)?;
        let Some(task_view) =
            repository.find_raw_schema_view_by_path(&task_path_key)?
        else {
            panic!("task view");
        };
        let task_file_v2 =
            make_fs_file(vault_dir.path().join("schemas/task.json"))?;
        let processor2 =
            BaseSchemaProcessor::from_discovery(task_file_v2, &root)?;
        let res2 = processor2.run(
            Some(&task_view),
            &source,
            &repository,
            Some(&bank_res2),
        )?;

        let BaseSchemaResolution::Stale {
            base: prop_base,
            id: sid,
            ..
        } = res2
        else {
            panic!("Expected Stale resolution, got {res2:?}");
        };
        assert_eq!(sid, *schema_id);
        let Some(prop2) = prop_base.properties().get(&status_name) else {
            panic!("status property");
        };
        assert_eq!(prop2.id(), prop1_id, "Property ID should be preserved");
        // Check type changed (via spec)
        assert!(matches!(
            prop2.spec(),
            lithos_core::schema::property_spec::PropertySpec::Bool(_)
        ));

        Ok(())
    }

    #[test]
    #[expect(
        clippy::too_many_lines,
        reason = "Integration test setup and multi-stage verification."
    )]
    fn returns_multiple_stale_resolutions_when_shared_bank_changes()
    -> TestResult {
        let vault_dir = TempDir::new()?;
        let test_db = TestDb::new()?;
        let repository = setup_repository(test_db.store());
        let source = FileReader::new(vault_dir.path());
        let root = DirPath::try_new(vault_dir.path().to_path_buf())?;

        // Setup
        write_file(
            vault_dir.path(),
            "schemas/property_bank.json",
            r#"{"$version": "1.0", "properties": {"shared": {"type": "string"}}}"#,
        )?;
        write_file(
            vault_dir.path(),
            "schemas/schema1.json",
            r##"{"$version": "1.0", "properties": {"p1": {"$ref": "#property_bank/shared"}}}"##,
        )?;
        write_file(
            vault_dir.path(),
            "schemas/schema2.json",
            r##"{"$version": "1.0", "properties": {"p2": {"$ref": "#property_bank/shared"}}}"##,
        )?;

        // Initial Load
        let bank_file =
            make_fs_file(vault_dir.path().join("schemas/property_bank.json"))?;
        let bank_res =
            PropertyBankProcessor::from_discovery(bank_file.clone(), &root)?
                .run(None, &source, &repository)?;

        let s1_file =
            make_fs_file(vault_dir.path().join("schemas/schema1.json"))?;
        let res1 = BaseSchemaProcessor::from_discovery(s1_file.clone(), &root)?
            .run(None, &source, &repository, Some(&bank_res))?;

        let s2_file =
            make_fs_file(vault_dir.path().join("schemas/schema2.json"))?;
        let res2 = BaseSchemaProcessor::from_discovery(s2_file.clone(), &root)?
            .run(None, &source, &repository, Some(&bank_res))?;

        let BaseSchemaResolution::New {
            base: s1_base,
            ..
        } = res1
        else {
            panic!("Expected New for schema 1");
        };
        let id1 = *s1_base.id();

        let BaseSchemaResolution::New {
            base: s2_base,
            ..
        } = res2
        else {
            panic!("Expected New for schema 2");
        };
        let id2 = *s2_base.id();

        // Change Bank
        write_file(
            vault_dir.path(),
            "schemas/property_bank.json",
            r#"{"$version": "1.0", "properties": {"shared": {"type": "number"}}}"#,
        )?;
        let bank_path_key = bank_file.path().as_key(&root)?;
        let Some(bank_view) =
            repository.get_raw_property_bank_view(&bank_path_key)?
        else {
            panic!("bank view");
        };
        let bank_file_v2 =
            make_fs_file(vault_dir.path().join("schemas/property_bank.json"))?;
        let bank_proc2 =
            PropertyBankProcessor::from_discovery(bank_file_v2, &root)?;
        let bank_res2 =
            bank_proc2.run(Some(&bank_view), &source, &repository)?;

        // Verify both are Stale
        let s1_path_key = s1_file.path().as_key(&root)?;
        let Some(s1_view) =
            repository.find_raw_schema_view_by_path(&s1_path_key)?
        else {
            panic!("s1 view");
        };
        let s1_file_v2 =
            make_fs_file(vault_dir.path().join("schemas/schema1.json"))?;
        let res1_v2 = BaseSchemaProcessor::from_discovery(s1_file_v2, &root)?
            .run(
            Some(&s1_view),
            &source,
            &repository,
            Some(&bank_res2),
        )?;

        let BaseSchemaResolution::Stale {
            id: sid1,
            ..
        } = res1_v2
        else {
            panic!("Schema 1 should be stale, got {res1_v2:?}");
        };
        assert_eq!(sid1, id1, "Schema 1 ID should match original ID");

        let s2_path_key_v2 = s2_file.path().as_key(&root)?;
        let Some(s2_view_v2) =
            repository.find_raw_schema_view_by_path(&s2_path_key_v2)?
        else {
            panic!("s2 view");
        };
        let s2_file_v2 =
            make_fs_file(vault_dir.path().join("schemas/schema2.json"))?;
        let res2_v2 = BaseSchemaProcessor::from_discovery(s2_file_v2, &root)?
            .run(
            Some(&s2_view_v2),
            &source,
            &repository,
            Some(&bank_res2),
        )?;

        let BaseSchemaResolution::Stale {
            id: sid2,
            ..
        } = res2_v2
        else {
            panic!("Schema 2 should be stale, got {res2_v2:?}");
        };
        assert_eq!(sid2, id2, "Schema 2 ID should match original ID");

        Ok(())
    }

    #[test]
    fn constructs_deleted_resolution_when_base_schema_removed() -> TestResult {
        let vault_dir = TempDir::new()?;
        let test_db = TestDb::new()?;
        let repository = setup_repository(test_db.store());
        let source = FileReader::new(vault_dir.path());
        let root = DirPath::try_new(vault_dir.path().to_path_buf())?;

        write_file(
            vault_dir.path(),
            "schemas/task.json",
            r#"{"$version": "1.0", "properties": {}}"#,
        )?;

        let task_file =
            make_fs_file(vault_dir.path().join("schemas/task.json"))?;
        let processor =
            BaseSchemaProcessor::from_discovery(task_file.clone(), &root)?;
        let resolution = processor.run(None, &source, &repository, None)?;

        let BaseSchemaResolution::New {
            id: schema_id,
            ..
        } = resolution
        else {
            panic!("Expected New resolution, got {resolution:?}");
        };

        assert!(
            repository.find_base_schema_by_id(schema_id)?.is_some(),
            "base schema should exist before deletion"
        );

        // Caller-level deletion: remove persistence then construct event.
        repository.delete_base_schema(schema_id)?;
        repository.delete_schema(schema_id)?;

        assert!(
            repository.find_base_schema_by_id(schema_id)?.is_none(),
            "base schema should be removed after deletion"
        );

        let deleted = BaseSchemaResolution::Deleted {
            id: schema_id,
        };
        assert_eq!(deleted.schema_id(), schema_id);
        assert!(matches!(deleted, BaseSchemaResolution::Deleted { .. }));

        Ok(())
    }

    #[test]
    #[expect(
        clippy::too_many_lines,
        reason = "Integration test setup and mixed outcome verification."
    )]
    fn returns_mixed_resolutions_when_inputs_vary() -> TestResult {
        let vault_dir = TempDir::new()?;
        let test_db = TestDb::new()?;
        let repository = setup_repository(test_db.store());
        let source = FileReader::new(vault_dir.path());
        let root = DirPath::try_new(vault_dir.path().to_path_buf())?;

        write_file(
            vault_dir.path(),
            "schemas/property_bank.json",
            r#"{"$version": "1.0", "properties": {"status": {"type": "string"}}}"#,
        )?;
        write_file(
            vault_dir.path(),
            "schemas/fresh_inline.json",
            r#"{"$version": "1.0", "properties": {"title": {"type": "string"}}}"#,
        )?;
        write_file(
            vault_dir.path(),
            "schemas/stale_ref.json",
            r##"{"$version": "1.0", "properties": {"status": {"$ref": "#property_bank/status"}}}"##,
        )?;

        let bank_file =
            make_fs_file(vault_dir.path().join("schemas/property_bank.json"))?;
        let bank_res =
            PropertyBankProcessor::from_discovery(bank_file.clone(), &root)?
                .run(None, &source, &repository)?;

        let fresh_file =
            make_fs_file(vault_dir.path().join("schemas/fresh_inline.json"))?;
        let fresh_res =
            BaseSchemaProcessor::from_discovery(fresh_file.clone(), &root)?
                .run(None, &source, &repository, Some(&bank_res))?;
        let fresh_id = fresh_res.schema_id();

        let stale_file =
            make_fs_file(vault_dir.path().join("schemas/stale_ref.json"))?;
        let stale_res =
            BaseSchemaProcessor::from_discovery(stale_file.clone(), &root)?
                .run(None, &source, &repository, Some(&bank_res))?;
        let BaseSchemaResolution::New {
            base: stale_base,
            id: stale_id,
        } = stale_res
        else {
            panic!("Expected initial New resolution for stale_ref");
        };
        let status_name = PropertyName::try_new("status")?;
        let Some(stale_property) = stale_base.properties().get(&status_name)
        else {
            panic!("status property");
        };
        let stale_property_id = stale_property.id();

        write_file(
            vault_dir.path(),
            "schemas/property_bank.json",
            r#"{"$version": "1.0", "properties": {"status": {"type": "boolean"}}}"#,
        )?;
        write_file(
            vault_dir.path(),
            "schemas/new_schema.json",
            r#"{"$version": "1.0", "properties": {"note": {"type": "string"}}}"#,
        )?;

        let bank_path_key = bank_file.path().as_key(&root)?;
        let Some(bank_view) =
            repository.get_raw_property_bank_view(&bank_path_key)?
        else {
            panic!("bank view");
        };
        let bank_file_v2 =
            make_fs_file(vault_dir.path().join("schemas/property_bank.json"))?;
        let bank_res_v2 = PropertyBankProcessor::from_discovery(
            bank_file_v2,
            &root,
        )?
        .run(Some(&bank_view), &source, &repository)?;

        let fresh_path_key = fresh_file.path().as_key(&root)?;
        let Some(fresh_view) =
            repository.find_raw_schema_view_by_path(&fresh_path_key)?
        else {
            panic!("fresh view");
        };
        let stale_path_key = stale_file.path().as_key(&root)?;
        let Some(stale_view) =
            repository.find_raw_schema_view_by_path(&stale_path_key)?
        else {
            panic!("stale view");
        };
        let new_file =
            make_fs_file(vault_dir.path().join("schemas/new_schema.json"))?;

        let outcomes = [
            BaseSchemaProcessor::from_discovery(new_file, &root)?.run(
                None,
                &source,
                &repository,
                Some(&bank_res_v2),
            )?,
            BaseSchemaProcessor::from_discovery(stale_file, &root)?.run(
                Some(&stale_view),
                &source,
                &repository,
                Some(&bank_res_v2),
            )?,
            BaseSchemaProcessor::from_discovery(fresh_file, &root)?.run(
                Some(&fresh_view),
                &source,
                &repository,
                Some(&bank_res_v2),
            )?,
        ];

        let mut saw_fresh = false;
        let mut saw_stale = false;
        let mut saw_new = false;
        for outcome in outcomes {
            match outcome {
                BaseSchemaResolution::Fresh {
                    id: schema_id,
                    ..
                } => {
                    assert_eq!(
                        schema_id, fresh_id,
                        "fresh schema ID should match"
                    );
                    saw_fresh = true;
                }
                BaseSchemaResolution::Stale {
                    id: schema_id,
                    base: base_schema,
                    ..
                } => {
                    assert_eq!(
                        schema_id, stale_id,
                        "stale schema ID should match"
                    );
                    let Some(property) =
                        base_schema.properties().get(&status_name)
                    else {
                        panic!("status property");
                    };
                    assert_eq!(
                        property.id(),
                        stale_property_id,
                        "stale re-expansion should preserve the property ID"
                    );
                    assert!(
                        matches!(property.spec(), PropertySpec::Bool(_)),
                        "stale re-expansion should use the changed Property \
                         Spec"
                    );
                    saw_stale = true;
                }
                BaseSchemaResolution::New {
                    base: base_schema,
                    ..
                } => {
                    assert_eq!(
                        base_schema.name().as_str(),
                        "new_schema",
                        "new schema should be created in the same logical run"
                    );
                    saw_new = true;
                }
                BaseSchemaResolution::Deleted {
                    id: schema_id,
                } => panic!("Unexpected Deleted resolution for {schema_id}"),
                _ => panic!("Unexpected unknown BaseSchemaResolution variant"),
            }
        }

        assert!(saw_fresh, "run should include a Fresh outcome");
        assert!(saw_stale, "run should include a Stale outcome");
        assert!(saw_new, "run should include a New outcome");

        Ok(())
    }
}
