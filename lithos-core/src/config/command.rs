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
#[expect(
    clippy::disallowed_methods,
    reason = "Test code uses unwrap/expect for clarity"
)]
mod tests {
    use tempfile::{TempDir, tempdir};

    use super::*;

    fn test_db() -> (TempDir, Database) {
        let dir = tempdir().unwrap();
        let path = dir.path().join("config.redb");
        let db = Database::open(&path).unwrap();
        (dir, db)
    }

    #[test]
    fn save_global_persists_configuration() {
        let (_dir, db) = test_db();
        let cmd = Command::new(&db);

        let global = Global::default();
        cmd.save_global(&global).unwrap();

        let stored = db.get_owned::<Global>("config", "global").unwrap();
        let stored_global = stored.expect("Stored global config should exist");
        assert_eq!(
            stored_global, global,
            "Stored global config should match input"
        );
    }

    #[test]
    fn save_vault_persists_configuration() {
        let (_dir, db) = test_db();
        let cmd = Command::new(&db);

        let vault = Vault::default();
        cmd.save_vault(&vault).unwrap();

        let stored = db.get_owned::<Vault>("config", "vault").unwrap();
        let stored_vault = stored.expect("Stored vault config should exist");
        assert_eq!(
            stored_vault, vault,
            "Stored vault config should match input"
        );
    }
}
