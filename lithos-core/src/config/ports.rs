//! Configuration port definitions for CQRS pattern.
//!
//! This module defines command and query trait interfaces for configuration
//! management. Note: Per proposal, async is removed - sync-first approach.

use super::{
    aggregate::Config, error::ConfigError, global::Global, vault::Vault,
};

/// Command port for configuration write operations.
///
/// # Invariants
/// - All operations return Result for error handling
/// - Commands may modify state (write operations)
///
/// # Note on Sync-First
/// This trait is synchronous per the architecture proposal (Phase 3).
/// Async wrappers should be added at the CLI/LSP boundary if needed.
pub trait Command: Send + Sync {
    /// Save global configuration.
    ///
    /// # Errors
    /// Returns `ConfigError` if save operation fails.
    fn save_global(&self, config: &Global) -> Result<(), ConfigError>;

    /// Save vault-specific configuration.
    ///
    /// # Errors
    /// Returns `ConfigError` if save operation fails.
    fn save_vault(&self, config: &Vault) -> Result<(), ConfigError>;
}

/// Query port for configuration read operations.
///
/// # Invariants
/// - All operations return Result for error handling
/// - Queries must NOT modify state (read-only operations)
///
/// # Note on Sync-First
/// This trait is synchronous per the architecture proposal (Phase 3).
/// Async wrappers should be added at the CLI/LSP boundary if needed.
pub trait Query: Send + Sync {
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
    fn load(&self) -> Result<Config, ConfigError>;

    /// Load global configuration.
    ///
    /// # Errors
    /// Returns `ConfigError` if load operation fails or config is invalid.
    fn load_global(&self) -> Result<Option<Global>, ConfigError>;

    /// Load vault-specific configuration.
    ///
    /// # Errors
    /// Returns `ConfigError` if load operation fails or config is invalid.
    fn load_vault(&self) -> Result<Option<Vault>, ConfigError>;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn command_trait_is_object_safe() {
        // GIVEN the Command trait
        fn _assert_object_safe(_: &dyn Command) {}

        // WHEN using it as a trait object
        // THEN it remains object-safe
    }

    #[test]
    fn query_trait_is_object_safe() {
        // GIVEN the Query trait
        fn _assert_object_safe(_: &dyn Query) {}

        // WHEN using it as a trait object
        // THEN it remains object-safe
    }

    #[test]
    fn traits_are_send_and_sync() {
        // GIVEN Command and Query trait objects
        fn is_send_sync<T: Send + Sync>() {}

        // WHEN checking Send + Sync bounds
        is_send_sync::<Box<dyn Command>>();
        is_send_sync::<Box<dyn Query>>();

        // THEN trait objects satisfy Send + Sync
    }
}
