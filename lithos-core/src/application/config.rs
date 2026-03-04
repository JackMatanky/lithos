//! Config application service — orchestrates config ingestion with staleness
//! detection.
//!
//! This is a placeholder implementation. The full staleness detection logic
//! will be implemented in Phase 6.

#![allow(clippy::module_name_repetitions, reason = "Namespaced types")]

use crate::config::{
    aggregate::Config, command::Command, error::ConfigCommandError,
    query::Query, vault::VaultRoot,
};

// ─────────────────────────────────────────────────────────────────────────────
//  ConfigServiceError
// ─────────────────────────────────────────────────────────────────────────────

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

    /// Loads and merges configuration for a vault.
    ///
    /// This is a placeholder that uses the existing `rebuild_merged` logic.
    /// Full staleness detection will be implemented in Phase 6.
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
        // Use existing rebuild_merged for now
        let vault_id = crate::config::vault::VaultId::new();
        let _version = self.command.rebuild_merged(vault_id, vault_root)?;

        // Fetch the merged config
        let config = self.query.find(vault_id)?.ok_or_else(|| {
            ConfigServiceError::Query(
                crate::config::error::ConfigQueryError::Corruption(
                    "merged config not found after rebuild".into(),
                ),
            )
        })?;

        Ok(config)
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
