//! Integration test for the property bank processor pipeline.

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
    config::{
        aggregate::Config,
        builder,
        vault::{VaultId, VaultRoot},
    },
    fs::{FsReader, RelativePath},
    schema::{builder::Builder, repository::SchemaReadRepository as _},
};
use tempfile::TempDir;

/// Write a file to the test directory.
fn write_file(root: &Path, relative: &Path, content: &str) -> TestResult {
    let path = root.join(relative);
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    std::fs::write(&path, content)?;
    Ok(())
}

/// Create a test config for a vault root.
fn test_config(root: &Path) -> TestResult<Config> {
    let root = VaultRoot::try_new(root.to_path_buf())?;
    let config = builder::build_from_layers(
        None,
        None,
        VaultId::new(),
        root,
        lithos_core::config::aggregate::Version::initial(),
    )?;
    Ok(config)
}

#[test]
fn loads_and_persists_property_bank() -> TestResult {
    let vault_dir = TempDir::new()?;
    let test_db = TestDb::new()?;

    let config = test_config(vault_dir.path())?;
    let property_path = config.paths().property_bank_path();

    write_file(
        vault_dir.path(),
        &property_path,
        r#"{"$version": "1.0", "properties": {
            "title": {"type": "string"},
            "status": {"type": "string"}
        }}"#,
    )?;

    let repository = setup_repository(test_db.store());
    let source = FsReader::new(vault_dir.path());
    let mut builder = Builder::new(repository, source, &config);

    let _schemas = builder.load_all()?;
    drop(builder);

    let repository2 = setup_repository(test_db.store());
    let saved_bank = repository2
        .get_property_bank()?
        .expect("Expected bank to be persisted");
    assert!(saved_bank.has(&"title".try_into()?), "Expected title property");
    assert!(saved_bank.has(&"status".try_into()?), "Expected status property");

    let _source2 = FsReader::new(vault_dir.path());
    let bank_path = RelativePath::try_from(property_path)?;
    let view = repository2.get_raw_property_bank_view(&bank_path)?;
    assert!(view.is_some(), "Expected raw view to be persisted");

    Ok(())
}
