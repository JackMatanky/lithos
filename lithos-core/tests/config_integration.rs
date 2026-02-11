//! Integration tests for the Config Context CQRS flow.

#![expect(
    clippy::panic_in_result_fn,
    reason = "Integration tests use assertions which panic on failure."
)]
#![expect(
    clippy::tests_outside_test_module,
    reason = "Integration tests are top-level by default."
)]

use lithos_core::{
    config::{
        RedbConfigCommand, RedbConfigQuery,
        vault::{VaultId, VaultRoot},
    },
    db::Database,
};
use tempfile::tempdir;

#[test]
fn config_cqrs_integration_flow() -> Result<(), Box<dyn std::error::Error>> {
    // 1. Setup DB
    let dir = tempdir()?;
    let db_path = dir.path().join("lithos.redb");
    let db = Database::open(&db_path)?;

    // 2. Setup Services using convenience wrappers
    let command = RedbConfigCommand::new_redb(&db);
    let query = RedbConfigQuery::new_redb(&db);

    // 3. Define Inputs
    let vault_id = VaultId::new();
    let vault_root = VaultRoot::try_new(dir.path().join("my_vault"))?;

    // Create dummy vault directory so ingestion doesn't fail
    std::fs::create_dir_all(vault_root.as_path())?;

    // 4. Execute Command: Rebuild Merged Config
    let version = command.rebuild_merged(vault_id, &vault_root)?;

    assert_eq!(version.value(), 1, "First version should be 1");

    // 5. Execute Query: Get Active Config
    let config = query.get(vault_id)?;

    assert!(config.is_some(), "Should return active config");
    // # LINT_DISABLE_REASON: Test assertion context
    #[expect(
        clippy::disallowed_methods,
        reason = "Test assertion uses unwrap on Option."
    )]
    let config = config.unwrap();

    // 6. Verify Content
    assert_eq!(
        config.vault_metadata().id(),
        vault_id,
        "Config should belong to the requested vault"
    );
    // Default log level is Info
    assert_eq!(config.logging().log_level_str(), "info");

    Ok(())
}
