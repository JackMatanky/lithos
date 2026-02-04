//! Config command implementations (CQRS write operations).
//!
//! This module implements the Command port trait for Config write operations,
//! using the Database layer for persistence.

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
    #[expect(
        clippy::same_name_method,
        reason = "Inherent convenience method intentionally matches the port \
                  trait method name"
    )]
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
    #[expect(
        clippy::same_name_method,
        reason = "Inherent convenience method intentionally matches the port \
                  trait method name"
    )]
    pub fn save_vault(&self, config: &Vault) -> Result<(), ConfigError> {
        self.db
            .put("config", "vault", config)
            .map_err(|e| ConfigError::Storage(e.to_string().into()))
    }
}

impl super::ports::Command for Command<'_> {
    #[inline]
    fn save_global(&self, config: &Global) -> Result<(), ConfigError> {
        self.save_global(config)
    }

    #[inline]
    fn save_vault(&self, config: &Vault) -> Result<(), ConfigError> {
        self.save_vault(config)
    }
}

#[cfg(test)]
mod tests {
    use tempfile::{TempDir, tempdir};

    use super::*;

    fn test_db() -> Result<(TempDir, Database), String> {
        let dir = tempdir().map_err(|e| e.to_string())?;
        let path = dir.path().join("config.redb");
        let db = Database::open(&path).map_err(|e| e.to_string())?;
        Ok((dir, db))
    }

    #[test]
    fn save_global_persists_configuration() -> Result<(), String> {
        let (_dir, db) = test_db()?;
        let cmd = Command::new(&db);

        let global = Global::default();
        cmd.save_global(&global).map_err(|e| e.to_string())?;

        let stored = db
            .get_owned::<Global>("config", "global")
            .map_err(|e| e.to_string())?;
        let stored_global = stored
            .ok_or_else(|| "Stored global config should exist".to_owned())?;
        if stored_global != global {
            return Err("Stored global config should match input".to_owned());
        }
        Ok(())
    }

    #[test]
    fn save_vault_persists_configuration() -> Result<(), String> {
        let (_dir, db) = test_db()?;
        let cmd = Command::new(&db);

        let vault = Vault::default();
        cmd.save_vault(&vault).map_err(|e| e.to_string())?;

        let stored = db
            .get_owned::<Vault>("config", "vault")
            .map_err(|e| e.to_string())?;
        let stored_vault = stored
            .ok_or_else(|| "Stored vault config should exist".to_owned())?;
        if stored_vault != vault {
            return Err("Stored vault config should match input".to_owned());
        }
        Ok(())
    }
}
