//! Configuration port definitions for CQRS pattern.
//!
//! This module defines command and query trait interfaces for configuration
//! management following hexagonal architecture and CQRS principles.
//!

use async_trait::async_trait;

use crate::{
    config::{aggregate::Config, global::Global, vault::Vault},
    errors::ConfigError,
};

/// Command port for configuration write operations.
///
/// # Invariants
/// - All methods must be async (use `#[async_trait]`)
/// - All operations return Result for error handling
/// - Commands may modify state (write operations)
///
/// # Examples
/// ```ignore
/// #[async_trait]
/// impl Command for MyConfigAdapter {
///     async fn save_vault(&self, config: Vault) -> Result<(), ConfigError> {
///         // Adapter implementation for saving vault config
///         Ok(())
///     }
/// }
/// ```
#[async_trait]
pub trait Command: Send + Sync {
    /// Save global configuration.
    ///
    /// # Errors
    /// Returns `ConfigError` if save operation fails.
    async fn save_global(&self, config: Global) -> Result<(), ConfigError>;

    /// Save vault-specific configuration.
    ///
    /// # Errors
    /// Returns `ConfigError` if save operation fails.
    async fn save_vault(&self, config: Vault) -> Result<(), ConfigError>;
}

/// Query port for configuration read operations.
///
/// # Invariants
/// - All methods must be async (use `#[async_trait]`)
/// - All operations return Result for error handling
/// - Queries must NOT modify state (read-only operations)
///
/// # Examples
/// ```ignore
/// #[async_trait]
/// impl Query for MyConfigAdapter {
///     async fn load(&self) -> Result<Config, ConfigError> {
///         // Adapter implementation for loading and merging config
///         let global = self.load_global().await?;
///         let vault = self.load_vault().await?;
///         Config::build(global, vault)
///     }
/// }
/// ```
#[async_trait]
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
    async fn load(&self) -> Result<Config, ConfigError>;

    /// Load global configuration.
    ///
    /// # Errors
    /// Returns `ConfigError` if load operation fails or config is invalid.
    async fn load_global(&self) -> Result<Global, ConfigError>;

    /// Load vault-specific configuration.
    ///
    /// # Errors
    /// Returns `ConfigError` if load operation fails or config is invalid.
    async fn load_vault(&self) -> Result<Vault, ConfigError>;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn command_trait_is_object_safe() {
        // Verify trait can be used as trait object
        fn _assert_object_safe(_: &dyn Command) {}
    }

    #[test]
    fn query_trait_is_object_safe() {
        // Verify trait can be used as trait object
        fn _assert_object_safe(_: &dyn Query) {}
    }

    #[test]
    fn traits_are_send_and_sync() {
        // Compile-time check that trait objects are Send + Sync.
        // This test documents the requirement explicitly.
        fn is_send_sync<T: Send + Sync>() {}
        is_send_sync::<Box<dyn Command>>();
        is_send_sync::<Box<dyn Query>>();
    }
}
