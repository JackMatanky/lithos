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
    /// # Constraints
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
    clippy::arbitrary_source_item_ordering,
    reason = "Test module groups fixtures and submodules for readability."
)]
mod tests {
    mod fixtures {
        use super::*;

        pub fn test_db() -> Result<(TempDir, Database), String> {
            let dir = tempdir().map_err(|e| e.to_string())?;
            let path = dir.path().join("config.redb");
            let db = Database::open(&path).map_err(|e| e.to_string())?;

            // Initialize config table (without writing unrelated data)
            let dummy = Global::default();
            db.put("config", "_init", &dummy).map_err(|e| e.to_string())?;
            db.delete("config", "_init").map_err(|e| e.to_string())?;

            Ok((dir, db))
        }
    }

    use tempfile::{TempDir, tempdir};

    use super::*;
    use crate::config::{command, ports::Query as _};

    mod load {
        use super::*;

        fn merged_config() -> Result<Config, String> {
            let (_dir, db) = fixtures::test_db()?;

            let cmd = command::Command::new(&db);
            let mut global = Global::default();
            global.filesystem.template.templates_dir =
                "global_templates".to_owned();
            cmd.save_global(&global).map_err(|e| e.to_string())?;

            let mut vault = Vault::default();
            vault.filesystem.template.templates_dir =
                "vault_templates".to_owned();
            cmd.save_vault(&vault).map_err(|e| e.to_string())?;

            let qry = Query::new(&db);
            qry.load().map_err(|e| e.to_string())
        }

        #[test]
        fn load_global_returns_none_when_missing() -> Result<(), String> {
            let (_dir, db) = fixtures::test_db()?;
            let qry = Query::new(&db);

            let global = qry.load_global().map_err(|e| e.to_string())?;
            if global.is_some() {
                return Err(
                    "Missing global config should return None".to_owned()
                );
            }
            Ok(())
        }

        #[test]
        fn load_vault_returns_none_when_missing() -> Result<(), String> {
            let (_dir, db) = fixtures::test_db()?;
            let qry = Query::new(&db);

            let vault = qry.load_vault().map_err(|e| e.to_string())?;
            if vault.is_some() {
                return Err(
                    "Missing vault config should return None".to_owned()
                );
            }
            Ok(())
        }

        #[test]
        fn load_uses_fixed_vault_path() -> Result<(), String> {
            let config = merged_config()?;

            if config.vault_metadata.path != "vault" {
                return Err("Query load should use fixed vault path".to_owned());
            }
            Ok(())
        }

        #[test]
        fn load_prefers_vault_templates_dir() -> Result<(), String> {
            let config = merged_config()?;

            if config.vault_filesystem.template.templates_dir
                != "vault_templates"
            {
                return Err(
                    "Vault templates_dir should take precedence".to_owned()
                );
            }
            Ok(())
        }

        #[test]
        fn load_preserves_global_templates_dir() -> Result<(), String> {
            let config = merged_config()?;

            if config.global_filesystem.template.templates_dir
                != "global_templates"
            {
                return Err(
                    "Global templates_dir should be preserved".to_owned()
                );
            }
            Ok(())
        }
    }
}
