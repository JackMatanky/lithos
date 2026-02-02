//! Config command implementations (CQRS write operations).
//!
//! This module implements the Command port trait for Config write operations,
//! using the Database layer for persistence.

use super::{error::ConfigError, global::Global, vault::Vault};
use crate::db::Database;

/// Command implementation for Config write operations.
///
/// Implements the Command port trait using the Database layer.
pub struct ConfigCommand<'db> {
    db: &'db Database,
}

impl<'db> ConfigCommand<'db> {
    /// Create a new `ConfigCommand` with a database reference.
    #[inline]
    #[must_use]
    pub const fn new(db: &'db Database) -> Self {
        Self {
            db,
        }
    }

    /// Save global configuration.
    ///
    /// # Errors
    /// Returns `ConfigError` if save operation fails.
    ///
    /// # Phase 4 Note
    /// This is a stub implementation. Phase 6 will implement:
    /// 1. Validate Global config
    /// 2. Persist to database using `db.put("global_config", ...)`
    /// 3. Emit `ConfigUpdated` event
    #[inline]
    #[expect(
        clippy::todo,
        reason = "Phase 6 stub - will implement save global config"
    )]
    pub fn save_global(&self, _config: Global) -> Result<(), ConfigError> {
        let _: &Database = self.db;
        todo!("Implement in Phase 6: Save global config to database")
    }

    /// Save vault-specific configuration.
    ///
    /// # Errors
    /// Returns `ConfigError` if save operation fails.
    ///
    /// # Phase 4 Note
    /// This is a stub implementation. Phase 6 will implement:
    /// 1. Validate Vault config
    /// 2. Persist to database using `db.put("vault_config", ...)`
    /// 3. Emit `ConfigUpdated` event
    #[inline]
    #[expect(
        clippy::todo,
        reason = "Phase 6 stub - will implement save vault config"
    )]
    pub fn save_vault(&self, _config: Vault) -> Result<(), ConfigError> {
        let _: &Database = self.db;
        todo!("Implement in Phase 6: Save vault config to database")
    }
}
