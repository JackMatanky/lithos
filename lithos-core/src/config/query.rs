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
pub struct Query<'db> {
    db: &'db Database,
}

impl<'db> Query<'db> {
    /// Create a new `Query` with a database reference.
    #[inline]
    #[must_use]
    pub const fn new(db: &'db Database) -> Self {
        Self {
            db,
        }
    }
}

impl super::ports::Query for Query<'_> {
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
    #[inline]
    fn load(&self) -> Result<Config, ConfigError> {
        let global = self.load_global()?;
        let vault = self.load_vault()?.unwrap_or_default();

        Config::build(global.as_ref(), "vault", vault)
    }

    /// Load global configuration.
    ///
    /// # Errors
    /// Returns `ConfigError` if load operation fails or config is invalid.
    #[inline]
    fn load_global(&self) -> Result<Option<Global>, ConfigError> {
        self.db.get_owned("config", "global").map_err(
            |e: crate::db::DbError| ConfigError::Storage(e.to_string().into()),
        )
    }

    /// Load vault-specific configuration.
    ///
    /// # Errors
    /// Returns `ConfigError` if load operation fails or config is invalid.
    #[inline]
    fn load_vault(&self) -> Result<Option<Vault>, ConfigError> {
        self.db.get_owned("config", "vault").map_err(|e: crate::db::DbError| {
            ConfigError::Storage(e.to_string().into())
        })
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
    use crate::config::{command, ports::Query as _};

    fn test_db() -> (TempDir, Database) {
        let dir = tempdir().unwrap();
        let path = dir.path().join("config.redb");
        let db = Database::open(&path).unwrap();

        // Initialize config table (without writing unrelated data)
        let dummy = Global::default();
        db.put("config", "_init", &dummy).unwrap();
        db.delete("config", "_init").unwrap();

        (dir, db)
    }

    #[test]
    fn load_global_returns_none_when_missing() {
        let (_dir, db) = test_db();
        let qry = Query::new(&db);

        let global = qry.load_global().unwrap();
        assert!(global.is_none(), "Missing global config should return None");
    }

    #[test]
    fn load_vault_returns_none_when_missing() {
        let (_dir, db) = test_db();
        let qry = Query::new(&db);

        let vault = qry.load_vault().unwrap();
        assert!(vault.is_none(), "Missing vault config should return None");
    }

    #[test]
    fn load_merges_global_and_vault_config() {
        let (_dir, db) = test_db();

        let cmd = command::Command::new(&db);
        let mut global = Global::default();
        global.filesystem.template.templates_dir =
            "global_templates".to_owned();
        cmd.save_global(&global).unwrap();

        let mut vault = Vault::default();
        vault.filesystem.template.templates_dir = "vault_templates".to_owned();
        cmd.save_vault(&vault).unwrap();

        let qry = Query::new(&db);
        let config = qry.load().unwrap();

        assert_eq!(
            config.vault_metadata.path, "vault",
            "Query load should use fixed vault path"
        );
        assert_eq!(
            config.vault_filesystem.template.templates_dir, "vault_templates",
            "Vault templates_dir should take precedence"
        );
        assert_eq!(
            config.global_filesystem.template.templates_dir, "global_templates",
            "Global templates_dir should be preserved"
        );
    }
}
