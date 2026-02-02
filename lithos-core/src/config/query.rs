//! Config query implementations (CQRS read operations).
//!
//! This module implements the Query port trait for Config read operations,
//! using the Database layer for zero-copy reads.

use super::{
    aggregate::Config, error::ConfigError, global::Global, vault::Vault,
};
use crate::db::Database;

/// Query implementation for Config read operations.
///
/// Implements the Query port trait using the Database layer.
pub struct ConfigQuery<'db> {
    db: &'db Database,
}

impl<'db> ConfigQuery<'db> {
    /// Create a new `ConfigQuery` with a database reference.
    #[inline]
    #[must_use]
    pub const fn new(db: &'db Database) -> Self {
        Self {
            db,
        }
    }

    /// Load configuration (Global + Vault merged).
    ///
    /// # Business Rules
    /// - Loads both Global and Vault configurations
    /// - Merges using `Config::build` with Vault precedence
    /// - Validates merged result
    ///
    /// # Errors
    /// Returns `ConfigError` if:
    /// - Load operation fails
    /// - Merge operation fails
    /// - Validation fails
    ///
    /// # Phase 4 Note
    /// This is a stub implementation. Phase 6 will implement:
    /// 1. Load global config using ``db.get()``
    /// 2. Load vault config using ``db.get()``
    /// 3. Merge using `Config::build()`
    /// 4. Validate and return
    #[inline]
    #[expect(
        clippy::todo,
        reason = "Phase 6 stub - will implement load merged config"
    )]
    pub fn load(&self) -> Result<Config, ConfigError> {
        let _: &Database = self.db;
        todo!("Implement in Phase 6: Load and merge global + vault config")
    }

    /// Load global configuration.
    ///
    /// # Errors
    /// Returns `ConfigError` if load operation fails or config is invalid.
    ///
    /// # Phase 4 Note
    /// This is a stub implementation. Phase 6 will implement:
    /// 1. Use `db.get("global_config", ...)` for retrieval
    /// 2. Return Global or default if not found
    #[inline]
    #[expect(
        clippy::todo,
        reason = "Phase 6 stub - will implement load global config"
    )]
    pub fn load_global(&self) -> Result<Global, ConfigError> {
        let _: &Database = self.db;
        todo!("Implement in Phase 6: Load global config from database")
    }

    /// Load vault-specific configuration.
    ///
    /// # Errors
    /// Returns `ConfigError` if load operation fails or config is invalid.
    ///
    /// # Phase 4 Note
    /// This is a stub implementation. Phase 6 will implement:
    /// 1. Use `db.get("vault_config", ...)` for retrieval
    /// 2. Return Vault or default if not found
    #[inline]
    #[expect(
        clippy::todo,
        reason = "Phase 6 stub - will implement load vault config"
    )]
    pub fn load_vault(&self) -> Result<Vault, ConfigError> {
        let _: &Database = self.db;
        todo!("Implement in Phase 6: Load vault config from database")
    }
}
