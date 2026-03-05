//! Config application service — orchestrates config ingestion with staleness
//! detection.
//!
//! # Pipeline Flow
//!
//! The service uses **staleness detection** to decide whether to load from
//! files or reuse cached data from the database:
//!
//! 1. **Check if vault exists in DB**:
//!    - `Query::find_vault_id_by_path()` → get existing vault ID if present
//!    - If not found, create new vault ID
//!
//! 2. **Global config staleness check**:
//!    - Load global config file (if exists) with metadata
//!    - `Query::is_global_stale()` checks file timestamps vs DB metadata
//!    - Stale or missing → **reload from file**
//!    - Fresh → **skip reload**
//!
//! 3. **Vault config staleness check**:
//!    - Load vault config file (if exists) with metadata
//!    - `Query::is_vault_stale()` checks file timestamps vs DB metadata
//!    - Stale or missing → **reload from file**
//!    - Fresh → **skip reload**
//!
//! 4. **Merge and persist** (only if any config is stale):
//!    - Build merged `Config` from global + vault
//!    - `Command::record_global()` → save global + metadata
//!    - `Command::record_vault()` → save vault + metadata
//!    - `Command::record_merged()` → save merged snapshot
//!
//! 5. **Return merged config**:
//!    - Fetch active merged config from DB
//!
//! **Key optimization**: Lightweight staleness checks (timestamps only)
//! avoid parsing/processing unchanged files.

#![allow(clippy::module_name_repetitions, reason = "Namespaced types")]

use crate::config::{
    adapter::ingest::Ingestor,
    aggregate::{Config, Timestamp},
    command::Command,
    error::ConfigCommandError,
    global::Global,
    query::Query,
    raw::RawConfig,
    vault::{Vault, VaultId, VaultRoot},
};

// ─────────────────────────────────────────────────────────────────────────────
//  Type Aliases
// ─────────────────────────────────────────────────────────────────────────────

/// Return type for staleness check methods.
///
/// Tuple: `(raw_config, created_at, modified_at, is_stale)`.
type ConfigWithStaleness = (RawConfig, Option<Timestamp>, Timestamp, bool);

// ─────────────────────────────────────────────────────────────────────────────
//  ConfigServiceError
//  ─────────────────────────────────────────────────────────────────────────────

/// Errors that can occur during config service operations.
#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub enum ConfigServiceError {
    /// Ingestion (file I/O or parsing) failed.
    #[error("ingestion error: {0}")]
    Ingestion(#[from] crate::config::error::ConfigIngestError),

    /// Domain validation failed.
    #[error("domain error: {0}")]
    Domain(#[from] crate::config::error::ConfigError),

    /// Storage query failed.
    #[error("query error: {0}")]
    Query(#[from] crate::config::error::ConfigQueryError),

    /// Storage command failed.
    #[error("command error: {0}")]
    Command(#[from] ConfigCommandError),
}

// ─────────────────────────────────────────────────────────────────────────────
//  ConfigService
// ─────────────────────────────────────────────────────────────────────────────

/// Thin orchestration service for config ingestion with staleness detection.
///
/// Uses concrete redb adapters for production use.
pub struct ConfigService<'db> {
    query: Query<crate::config::adapter::query::QueryAdapter<'db>>,
    command: Command<crate::config::adapter::command::CommandAdapter<'db>>,
}

impl<'db> ConfigService<'db> {
    /// Creates a new config service with the given database adapters.
    #[inline]
    #[must_use]
    pub const fn new(
        query: Query<crate::config::adapter::query::QueryAdapter<'db>>,
        command: Command<crate::config::adapter::command::CommandAdapter<'db>>,
    ) -> Self {
        Self {
            query,
            command,
        }
    }

    /// Loads and merges configuration for a vault with staleness detection.
    ///
    /// Only reloads configs that have changed since last ingestion, avoiding
    /// unnecessary file I/O and parsing for unchanged configs.
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - File I/O fails (missing files, permission errors)
    /// - Config parsing fails (invalid TOML)
    /// - Domain validation fails (invalid config values)
    /// - Database operations fail
    #[inline]
    pub fn load(
        &self,
        vault_root: &VaultRoot,
    ) -> Result<Config, ConfigServiceError> {
        // ── Step 1: Get or create vault ID ──────────────────────────────────
        let vault_id = self
            .query
            .find_vault_id_by_path(vault_root)?
            .unwrap_or_else(VaultId::new);

        // ── Step 2: Check global config staleness ───────────────────────────
        let (global_raw, global_created, global_modified, global_stale) =
            self.load_global_with_staleness()?;

        // ── Step 3: Check vault config staleness ────────────────────────────
        let (vault_raw, vault_created, vault_modified, vault_stale) =
            self.load_vault_with_staleness(vault_root, vault_id)?;

        // ── Step 4: Merge and persist if anything changed ───────────────────
        let needs_rebuild = global_stale || vault_stale;

        if needs_rebuild {
            // Save global config + metadata (if stale)
            if global_stale {
                let global = Global::try_from(&global_raw)?;
                self.command.record_global(
                    &global,
                    global_created,
                    global_modified,
                )?;
            }

            // Save vault config + metadata (if stale)
            if vault_stale {
                let vault = Vault::try_from(&vault_raw)?;
                self.command.record_vault(
                    vault_id,
                    &vault,
                    vault_created,
                    vault_modified,
                )?;
            }

            // Build merged config from files
            let ingestor = Ingestor::new(vault_root.as_path());
            let raw_merged = ingestor.build_merged_raw(vault_root.as_path())?;
            let merged_config = Config::build(
                &raw_merged,
                vault_id,
                vault_root.clone(),
                crate::config::aggregate::Version::initial(), /* Placeholder
                                                               * -
                                                               * real version
                                                               * assigned
                                                               * atomically */
            )?;

            // Record vault path mapping and merged config
            self.command.record_vault_path_mapping(vault_id, vault_root)?;
            let _version =
                self.command.record_config(vault_id, &merged_config)?;
        }

        // Return active merged config (either newly built or cached)
        self.query.find(vault_id)?.ok_or_else(|| {
            ConfigServiceError::Query(
                crate::config::error::ConfigQueryError::Corruption(
                    "merged config not found in database".into(),
                ),
            )
        })
    }

    /// Load global config with staleness check.
    ///
    /// Returns: `(raw_config, created_at, modified_at, is_stale)`.
    fn load_global_with_staleness(
        &self,
    ) -> Result<ConfigWithStaleness, ConfigServiceError> {
        // Ingestor creates both vault-scoped and system-wide readers internally
        // We pass a dummy vault root since global config uses system-wide
        // reader
        let ingestor = Ingestor::new(std::env::temp_dir());
        if let Some((raw, created_at, modified_at)) =
            ingestor.load_global_config()?
        {
            // File exists - check if stale
            // If modified_at is None, use current time (file system doesn't
            // support it)
            let modified = modified_at.unwrap_or_else(Timestamp::now);
            let is_stale = self.query.is_global_stale(created_at, modified)?;
            Ok((raw, created_at, modified, is_stale))
        } else {
            // No file - use defaults
            // Only mark as stale if we haven't saved defaults yet
            // (no metadata with created_at = None exists)
            // Use a fixed timestamp (epoch) to check if defaults were saved
            let is_stale =
                self.query.is_global_stale(None, Timestamp::from_secs(0))?;
            Ok((RawConfig::default(), None, Timestamp::from_secs(0), is_stale))
        }
    }

    /// Load vault config with staleness check.
    ///
    /// Returns: `(raw_config, created_at, modified_at, is_stale)`.
    fn load_vault_with_staleness(
        &self,
        vault_root: &VaultRoot,
        vault_id: VaultId,
    ) -> Result<ConfigWithStaleness, ConfigServiceError> {
        let ingestor = Ingestor::new(vault_root.as_path());
        if let Some((raw, created_at, modified_at)) =
            ingestor.load_vault_config(vault_root)?
        {
            // File exists - check if stale
            // If modified_at is None, use current time (file system doesn't
            // support it)
            let modified = modified_at.unwrap_or_else(Timestamp::now);
            let is_stale =
                self.query.is_vault_stale(vault_id, created_at, modified)?;
            Ok((raw, created_at, modified, is_stale))
        } else {
            // No file - use defaults
            // Only mark as stale if we haven't saved defaults yet
            // (no metadata with created_at = None exists)
            // Use a fixed timestamp (epoch) to check if defaults were saved
            let is_stale = self.query.is_vault_stale(
                vault_id,
                None,
                Timestamp::from_secs(0),
            )?;
            Ok((RawConfig::default(), None, Timestamp::from_secs(0), is_stale))
        }
    }

    /// Merge global and vault configs into a single raw config.
    ///
    /// This is unnecessary since `rebuild_config` handles merging via Figment.
    /// Keeping as placeholder for future enhancement.
    #[expect(
        dead_code,
        reason = "Reserved for future manual merge control - currently \
                  rebuild_config handles merging via Figment"
    )]
    #[expect(
        clippy::unused_self,
        reason = "Method signature kept for future enhancement"
    )]
    fn merge_configs(
        &self,
        _global_raw: &RawConfig,
        _vault_raw: &RawConfig,
    ) -> RawConfig {
        // NOTE: This method is not currently used because rebuild_config
        // handles the merging via Figment. If we want manual control over
        // merging in the future, we can implement shallow field-level merging
        // here.
        RawConfig::default()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn config_service_error_displays_correctly() {
        // Test that error display works
        let error = ConfigServiceError::Domain(
            crate::config::error::ConfigError::DependencyViolation {
                field: "test".into(),
                depends_on: "other".into(),
            },
        );
        assert!(error.to_string().contains("domain error"));
    }
}
