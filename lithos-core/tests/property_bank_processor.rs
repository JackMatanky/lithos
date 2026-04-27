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
        raw::RawConfig,
        vault::{VaultId, VaultRoot},
    },
    fs::FsReader,
    schema::{builder::Builder, storage::Repository as _},
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
    let raw = RawConfig::default();
    let root = VaultRoot::try_new(root.to_path_buf())?;
    let config = Config::build(
        &raw,
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

    let repository = setup_repository(test_db.db());
    let source = FsReader::new(vault_dir.path());
    let mut builder = Builder::new(repository, source, &config);

    let _schemas = builder.load_all()?;

    let repository2 = setup_repository(test_db.db());
    let saved_bank = repository2
        .get_property_bank()?
        .expect("Expected bank to be persisted");
    assert!(saved_bank.has(&"title".try_into()?), "Expected title property");
    assert!(saved_bank.has(&"status".try_into()?), "Expected status property");

    let source2 = FsReader::new(vault_dir.path());
    let filename = source2.filename(&property_path)?;
    let view = repository2.get_raw_property_bank_view(&filename)?;
    assert!(view.is_some(), "Expected raw view to be persisted");

    Ok(())
}
