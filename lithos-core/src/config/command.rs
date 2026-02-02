//! Config command implementations (CQRS write operations).
//!
//! This module implements the Command port trait for Config write operations,
//! using the Database layer for persistence.

#![allow(
    clippy::same_name_method,
    clippy::missing_inline_in_public_items,
    clippy::elidable_lifetime_names,
    reason = "CQRS pattern: public methods intentionally delegate to trait \
              impls with same names"
)]

use super::{error::ConfigError, global::Global, vault::Vault};
use crate::db::Database;

/// Command implementation for Config write operations.
///
/// Implements the Command port trait using the Database layer.
pub struct Command<'db> {
    db: &'db Database,
}

impl<'db> Command<'db> {
    /// Create a new `Command` with a database reference.
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
    #[inline]
    pub fn save_global(&self, config: &Global) -> Result<(), ConfigError> {
        self.db
            .put("config", "global", config)
            .map_err(|e| ConfigError::Storage(e.to_string().into()))
    }

    /// Save vault-specific configuration.
    ///
    /// # Errors
    /// Returns `ConfigError` if save operation fails.
    #[inline]
    pub fn save_vault(&self, config: &Vault) -> Result<(), ConfigError> {
        self.db
            .put("config", "vault", config)
            .map_err(|e| ConfigError::Storage(e.to_string().into()))
    }
}

impl<'db> super::ports::Command for Command<'db> {
    fn save_global(&self, config: &Global) -> Result<(), ConfigError> {
        self.save_global(config)
    }

    fn save_vault(&self, config: &Vault) -> Result<(), ConfigError> {
        self.save_vault(config)
    }
}
