//! Config application service — orchestrates config ingestion with staleness
//! detection.
//!
//! **NOTE**: This module is deprecated. New code should use
//! [`crate::config::loader::Loader`] directly instead of `ConfigService`.
//!
//! # Migration Guide
//!
//! Old code:
//! ```rust,ignore
//! let service = ConfigService::new(query, command);
//! let config = service.load(&vault_root)?;
//! ```
//!
//! New code:
//! ```rust,ignore
//! let loader = Loader::new(vault_root, repository);
//! let config = loader.load()?;
//! ```

#![allow(clippy::module_name_repetitions, reason = "Namespaced types")]

use crate::config::{
    aggregate::Config, error::ConfigError, loader::Loader,
    storage::RedbStorage, vault::VaultRoot,
};

// ─────────────────────────────────────────────────────────────────────────────
//  ConfigServiceError
//  ─────────────────────────────────────────────────────────────────────────────

/// Errors that can occur during config service operations.
#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub enum ConfigServiceError {
    /// Configuration error (domain, storage, or ingestion).
    #[error("config error: {0}")]
    Config(#[from] ConfigError),
}

// ─────────────────────────────────────────────────────────────────────────────
//  ConfigService (Deprecated - Use Loader)
// ─────────────────────────────────────────────────────────────────────────────

/// **DEPRECATED**: Thin wrapper around [`Loader`] for backward compatibility.
///
/// New code should use [`crate::config::loader::Loader`] directly.
#[deprecated(
    since = "0.1.0",
    note = "Use config::loader::Loader directly. This wrapper exists only for \
            backward compatibility."
)]
pub struct ConfigService<'db> {
    db: &'db crate::db::Database,
}

#[expect(deprecated, reason = "Implementation of deprecated type")]
impl<'db> ConfigService<'db> {
    /// Creates a new config service.
    ///
    /// **DEPRECATED**: Use [`Loader::new`] instead.
    #[inline]
    #[must_use]
    pub const fn new(db: &'db crate::db::Database) -> Self {
        Self {
            db,
        }
    }

    /// Loads and merges configuration for a vault with staleness detection.
    ///
    /// **DEPRECATED**: Use [`Loader::load`] instead.
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
        // Delegate to Loader (new architecture)
        let storage = RedbStorage::new(self.db);
        let loader = Loader::new(vault_root.as_path(), storage);
        Ok(loader.load()?)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn config_service_error_displays_correctly() {
        // Test that error display works
        let error = ConfigServiceError::Config(
            crate::config::error::ConfigError::DependencyViolation {
                field: "test".into(),
                depends_on: "other".into(),
            },
        );
        assert!(error.to_string().contains("config error"));
    }
}
