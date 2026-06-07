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
    fs::{DirPath, FileMetadata, FilePath, FileReader, FsFile},
    schema::{
        base_processor::{BaseSchemaProcessor, BaseSchemaResolution},
        property::PropertyName,
        property_bank_processor::PropertyBankProcessor,
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

fn make_fs_file(real_path: std::path::PathBuf) -> TestResult<FsFile> {
    let path = FilePath::try_new(real_path)?;
    let std_meta = std::fs::metadata(path.as_path())?;
    let meta = FileMetadata::from(&std_meta);
    Ok(FsFile::new(path, meta))
}

#[test]
fn cold_start_and_bank_change() -> TestResult {
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
    let task_file = make_fs_file(vault_dir.path().join("schemas/task.json"))?;
    let processor =
        BaseSchemaProcessor::from_discovery(task_file.clone(), &root)?;
    let res1 = processor.run(None, &source, &repository, Some(&bank_res))?;

    let BaseSchemaResolution::New {
        base_schema: base1,
        ..
    } = res1
    else {
        panic!("Expected New resolution");
    };
    let schema_id = base1.id();
    let status_name = PropertyName::try_new("status")?;
    let prop1 = base1.properties().get(&status_name).expect("status property");
    let prop1_id = prop1.id();

    // 2. Bank Change (Change property type)
    write_file(
        vault_dir.path(),
        "schemas/property_bank.json",
        r#"{"$version": "1.0", "properties": {"status": {"type": "boolean"}}}"#,
    )?;

    // Re-run bank processor with view
    let bank_path_key = bank_file.path().as_key(&root)?;
    let bank_view = repository
        .get_raw_property_bank_view(&bank_path_key)?
        .expect("bank view");
    let bank_file_v2 =
        make_fs_file(vault_dir.path().join("schemas/property_bank.json"))?;
    let bank_proc2 =
        PropertyBankProcessor::from_discovery(bank_file_v2, &root)?;
    let bank_res2 = bank_proc2.run(Some(&bank_view), &source, &repository)?;

    // Re-run schema processor with view
    let task_path_key = task_file.path().as_key(&root)?;
    let task_view = repository
        .find_raw_schema_view_by_path(&task_path_key)?
        .expect("task view");
    let task_file_v2 =
        make_fs_file(vault_dir.path().join("schemas/task.json"))?;
    let processor2 = BaseSchemaProcessor::from_discovery(task_file_v2, &root)?;
    let res2 = processor2.run(
        Some(&task_view),
        &source,
        &repository,
        Some(&bank_res2),
    )?;

    let BaseSchemaResolution::Stale {
        base_schema: prop_base,
        schema_id: sid,
        ..
    } = res2
    else {
        panic!("Expected Stale resolution, got {res2:?}");
    };
    assert_eq!(sid, *schema_id);
    let prop2 =
        prop_base.properties().get(&status_name).expect("status property");
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
fn multiple_schemas_shared_bank_target() -> TestResult {
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
    let bank_res = PropertyBankProcessor::from_discovery(
        bank_file.clone(),
        &root,
    )?
    .run(None, &source, &repository)?;

    let s1_file = make_fs_file(vault_dir.path().join("schemas/schema1.json"))?;
    let res1 = BaseSchemaProcessor::from_discovery(s1_file.clone(), &root)?
        .run(None, &source, &repository, Some(&bank_res))?;

    let s2_file = make_fs_file(vault_dir.path().join("schemas/schema2.json"))?;
    let res2 = BaseSchemaProcessor::from_discovery(s2_file.clone(), &root)?
        .run(None, &source, &repository, Some(&bank_res))?;

    let BaseSchemaResolution::New {
        base_schema: s1_base,
        ..
    } = res1
    else {
        panic!("Expected New for schema 1");
    };
    let id1 = *s1_base.id();

    let BaseSchemaResolution::New {
        base_schema: s2_base,
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
    let bank_view = repository
        .get_raw_property_bank_view(&bank_path_key)?
        .expect("bank view");
    let bank_file_v2 =
        make_fs_file(vault_dir.path().join("schemas/property_bank.json"))?;
    let bank_proc2 =
        PropertyBankProcessor::from_discovery(bank_file_v2, &root)?;
    let bank_res2 = bank_proc2.run(Some(&bank_view), &source, &repository)?;

    // Verify both are Stale
    let s1_path_key = s1_file.path().as_key(&root)?;
    let s1_view = repository
        .find_raw_schema_view_by_path(&s1_path_key)?
        .expect("s1 view");
    let s1_file_v2 =
        make_fs_file(vault_dir.path().join("schemas/schema1.json"))?;
    let res1_v2 = BaseSchemaProcessor::from_discovery(s1_file_v2, &root)?.run(
        Some(&s1_view),
        &source,
        &repository,
        Some(&bank_res2),
    )?;

    let BaseSchemaResolution::Stale {
        schema_id: sid1,
        ..
    } = res1_v2
    else {
        panic!("Schema 1 should be stale, got {res1_v2:?}");
    };
    assert_eq!(sid1, id1);

    let s2_path_key_v2 = s2_file.path().as_key(&root)?;
    let s2_view_v2 = repository
        .find_raw_schema_view_by_path(&s2_path_key_v2)?
        .expect("s2 view");
    let s2_file_v2 =
        make_fs_file(vault_dir.path().join("schemas/schema2.json"))?;
    let res2_v2 = BaseSchemaProcessor::from_discovery(s2_file_v2, &root)?.run(
        Some(&s2_view_v2),
        &source,
        &repository,
        Some(&bank_res2),
    )?;

    let BaseSchemaResolution::Stale {
        schema_id: sid2,
        ..
    } = res2_v2
    else {
        panic!("Schema 2 should be stale, got {res2_v2:?}");
    };
    assert_eq!(sid2, id2);

    Ok(())
}

#[test]
fn caller_deletes_base_schema_and_constructs_resolution() -> TestResult {
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

    let task_file = make_fs_file(vault_dir.path().join("schemas/task.json"))?;
    let processor =
        BaseSchemaProcessor::from_discovery(task_file.clone(), &root)?;
    let resolution = processor.run(None, &source, &repository, None)?;

    let BaseSchemaResolution::New {
        schema_id,
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
        schema_id,
    };
    assert_eq!(deleted.schema_id(), schema_id);
    assert!(matches!(deleted, BaseSchemaResolution::Deleted { .. }));

    Ok(())
}
