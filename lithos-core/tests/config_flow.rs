//! Integration tests for the Config Context CQRS flow.

#![expect(
    clippy::panic_in_result_fn,
    reason = "Integration tests use assertions which panic on failure."
)]
#![expect(
    clippy::tests_outside_test_module,
    reason = "Integration tests are top-level by default."
)]

use std::path::{Path, PathBuf};

use lithos_core::{
    application::ConfigService,
    bounds::Bounds,
    config::{
        self as config_mod, adapter, error::ConfigError, value::FieldSpec,
        vault::VaultRoot,
    },
    db::Database,
};
use tempfile::tempdir;

type TestResult<T = ()> = Result<T, Box<dyn std::error::Error>>;

const VAULT_CONFIG_TOML: &str = r##"
vault_path = "/vault"

[logging]
log_level = "debug"

[paths]
templates_dir = "vault-templates"
property_bank_file = "bank.json"

[frontmatter]
title_key = "headline"
date_created_key = "created_at"

[task]
enabled = false
task_tags = ["#work"]

[task.status.todo]
symbol = ' '
status_type = "Todo"

[task.status.done]
symbol = 'x'
status_type = "Done"

[task.status.blocked]
symbol = '!'
status_type = "OnHold"

[task.dates.due]
keyword = "due"
format = "%Y-%m-%d"

[task.fields.priority]
type = "integer"
min = 1
max = 5

[task.fields.label]
type = "string"
pattern = "^[a-z]+$"

[task.indexing]
indexed_fields = ["priority"]
"##;

fn setup_db() -> TestResult<(tempfile::TempDir, Database)> {
    let dir = tempdir()?;
    let db_path = dir.path().join("lithos.redb");
    let db = Database::open(&db_path)?;
    Ok((dir, db))
}

fn write_vault_config(
    dir: &tempfile::TempDir,
    content: &str,
) -> TestResult<VaultRoot> {
    let vault_root = VaultRoot::try_new(dir.path().join("vault"))?;
    std::fs::create_dir_all(vault_root.as_path().join(".lithos"))?;
    std::fs::write(
        vault_root.as_path().join(".lithos").join("lithos.toml"),
        content,
    )?;
    Ok(vault_root)
}

fn load_config(
    service: &ConfigService,
    vault_root: &VaultRoot,
) -> TestResult<lithos_core::config::aggregate::Config> {
    let config = service.load(vault_root)?;
    Ok(config)
}

fn assert_merged_config(
    config: &lithos_core::config::aggregate::Config,
) -> TestResult {
    assert_eq!(
        config.logging().level_str(),
        "debug",
        "Expected logging to reflect vault override"
    );
    assert_eq!(
        config.paths().template.templates_dir().as_path(),
        Path::new("vault-templates"),
        "Expected templates_dir override from vault config"
    );
    assert_eq!(
        config.paths().schema.schemas_dir().as_path(),
        Path::new("schemas"),
        "Expected schemas_dir default when not overridden"
    );
    assert_eq!(
        config.paths().property_bank.as_str(),
        "bank.json",
        "Expected property bank file override"
    );
    assert_eq!(
        config.paths().property_bank_path(),
        PathBuf::from("schemas").join("bank.json"),
        "Expected property bank path to resolve under schemas dir"
    );
    assert_eq!(
        config.frontmatter().title().as_str(),
        "headline",
        "Expected frontmatter title override"
    );
    assert_eq!(
        config.frontmatter().date_created().as_str(),
        "created_at",
        "Expected frontmatter created date override"
    );
    assert_eq!(
        config.frontmatter().alias().as_str(),
        "aliases",
        "Expected frontmatter alias default"
    );
    assert!(!config.task().enabled(), "Expected task config to disable tasks");

    let tags = config.task().tags();
    let first_tag = tags.first().ok_or_else(|| {
        std::io::Error::other("Expected at least one task tag")
    })?;
    assert_eq!(first_tag.as_str(), "#work", "Expected task tag override");

    let due = config.task().due().ok_or_else(|| {
        std::io::Error::other("Expected due date spec to be set")
    })?;
    assert_eq!(due.keyword().as_str(), "due", "Expected due keyword override");
    assert_eq!(due.format(), "%Y-%m-%d", "Expected due date format override");

    let priority = config.task().field_spec("priority").ok_or_else(|| {
        std::io::Error::other("Expected priority field spec to exist")
    })?;
    #[expect(
        clippy::pattern_type_mismatch,
        reason = "Matching &FieldSpec by reference"
    )]
    if let FieldSpec::Integer {
        bounds,
        ..
    } = priority
    {
        assert_eq!(
            *bounds,
            Bounds::Range {
                min: 1,
                max: 5
            },
            "Expected priority bounds override"
        );
    } else {
        return Err(std::io::Error::other(
            "Expected priority field to be integer",
        )
        .into());
    }

    assert!(
        config.task().indexed().iter().any(|name| name.as_ref() == "priority"),
        "Expected priority to be indexed"
    );

    Ok(())
}

#[test]
fn config_cqrs_integration_flow() -> TestResult {
    // 1. Setup DB
    let dir = tempdir()?;
    let db_path = dir.path().join("lithos.redb");
    let db = Database::open(&db_path)?;

    // 2. Setup Services using convenience wrappers
    let command = config_mod::Command::new(adapter::Command::new(&db));
    let query = config_mod::Query::new(adapter::Query::new(&db));
    let service = ConfigService::new(query, command);

    // 3. Define Inputs
    let vault_root = VaultRoot::try_new(dir.path().join("my_vault"))?;

    // Create dummy vault directory so ingestion doesn't fail
    std::fs::create_dir_all(vault_root.as_path())?;

    // 4. Load Config (with staleness detection)
    let config = service.load(&vault_root)?;

    // 5. Verify Content
    assert_eq!(
        config.vault_metadata().root(),
        &vault_root,
        "Config should belong to the requested vault"
    );
    // Default log level is Info
    assert_eq!(
        config.logging().level_str(),
        "info",
        "Expected default log level to be info"
    );

    Ok(())
}

#[test]
fn config_ingestion_parsing_and_merge_from_vault_file() -> TestResult {
    let (dir, db) = setup_db()?;

    let command = config_mod::Command::new(adapter::Command::new(&db));
    let query = config_mod::Query::new(adapter::Query::new(&db));
    let service = ConfigService::new(query, command);

    let vault_root = write_vault_config(&dir, VAULT_CONFIG_TOML)?;
    let config = load_config(&service, &vault_root)?;
    assert_merged_config(&config)?;

    Ok(())
}

#[test]
fn config_ingestion_rejects_invalid_toml() -> TestResult {
    let (dir, db) = setup_db()?;

    let command = config_mod::Command::new(adapter::Command::new(&db));
    let query = config_mod::Query::new(adapter::Query::new(&db));
    let service = ConfigService::new(query, command);

    let vault_root =
        write_vault_config(&dir, "[logging]\nlog_level = \"debug\n")?;

    let result = service.load(&vault_root);
    let error = result.expect_err("Expected invalid TOML to error");
    if matches!(
        error,
        lithos_core::application::config::ConfigServiceError::Ingestion(_)
    ) {
        Ok(())
    } else {
        Err(std::io::Error::other("Expected ingest error for invalid TOML")
            .into())
    }
}

#[test]
fn config_ingestion_rejects_unknown_indexed_field() -> TestResult {
    let (dir, db) = setup_db()?;

    let command = config_mod::Command::new(adapter::Command::new(&db));
    let query = config_mod::Query::new(adapter::Query::new(&db));
    let service = ConfigService::new(query, command);

    let vault_root = write_vault_config(
        &dir,
        "vault_path = \"/vault\"\n[task]\nenabled = \
         true\n\n[task.indexing]\nindexed_fields = [\"priority\"]\n",
    )?;

    let result = service.load(&vault_root);
    let error = result.expect_err("Expected invalid task indexing to error");
    if matches!(
        error,
        lithos_core::application::config::ConfigServiceError::Domain(
            ConfigError::ValidationFailed { .. }
        )
    ) {
        Ok(())
    } else {
        Err(std::io::Error::other(
            "Expected validation error for unknown indexed field",
        )
        .into())
    }
}

#[test]
fn config_ingestion_rejects_invalid_field_name() -> TestResult {
    let (dir, db) = setup_db()?;

    let command = config_mod::Command::new(adapter::Command::new(&db));
    let query = config_mod::Query::new(adapter::Query::new(&db));
    let service = ConfigService::new(query, command);

    let vault_root = write_vault_config(
        &dir,
        "vault_path = \"/vault\"\n[task.fields.\"bad name\"]\ntype = \
         \"string\"\n",
    )?;

    let result = service.load(&vault_root);
    let error = result.expect_err("Expected invalid field name to error");
    if matches!(
        error,
        lithos_core::application::config::ConfigServiceError::Domain(
            ConfigError::ValidationFailed { .. }
        )
    ) {
        Ok(())
    } else {
        Err(std::io::Error::other(
            "Expected validation error for invalid field name",
        )
        .into())
    }
}

#[test]
fn config_rebuild_is_idempotent_for_same_inputs() -> TestResult {
    let (dir, db) = setup_db()?;

    let command = config_mod::Command::new(adapter::Command::new(&db));
    let query = config_mod::Query::new(adapter::Query::new(&db));
    let service = ConfigService::new(query, command);

    let vault_root = write_vault_config(&dir, VAULT_CONFIG_TOML)?;

    // First load - should rebuild from files
    let first = service.load(&vault_root)?;

    // Second load - should use cached config (no staleness)
    let second = service.load(&vault_root)?;

    // Both loads should return identical configs (same version since cached)
    assert_eq!(
        first.version(),
        second.version(),
        "Version should be same when loading cached config"
    );
    assert_eq!(
        first.vault_metadata(),
        second.vault_metadata(),
        "Vault metadata should be stable"
    );
    assert_eq!(
        first.logging(),
        second.logging(),
        "Logging config should be stable"
    );
    assert_eq!(first.paths(), second.paths(), "Paths config should be stable");
    assert_eq!(
        first.frontmatter(),
        second.frontmatter(),
        "Frontmatter config should be stable"
    );
    assert_eq!(first.task(), second.task(), "Task config should be stable");

    Ok(())
}
