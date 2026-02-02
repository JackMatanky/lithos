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
