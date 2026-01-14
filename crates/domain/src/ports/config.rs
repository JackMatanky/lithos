//! Configuration port definitions for CQRS pattern.
//!
//! This module defines command and query trait interfaces for configuration
//! management following hexagonal architecture and CQRS principles.
//!

use async_trait::async_trait;

use crate::{
    errors::ConfigError,
    models::config::{Config, GlobalConfig, VaultConfig},
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
///     async fn save_vault_config(&self, config: VaultConfig) -> Result<(), ConfigError> {
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
    async fn save_global_config(
        &self,
        config: GlobalConfig,
    ) -> Result<(), ConfigError>;

    /// Save vault-specific configuration.
    ///
    /// # Errors
    /// Returns `ConfigError` if save operation fails.
    async fn save_vault_config(
        &self,
        config: VaultConfig,
    ) -> Result<(), ConfigError>;
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
///     async fn load_merged_config(&self) -> Result<Config, ConfigError> {
///         // Adapter implementation for loading and merging config
///         let global = self.load_global_config().await?;
///         let vault = self.load_vault_config().await?;
///         Config::merge(global, vault)
///     }
/// }
/// ```
#[async_trait]
pub trait Query: Send + Sync {
    /// Load global configuration.
    ///
    /// # Errors
    /// Returns `ConfigError` if load operation fails or config is invalid.
    async fn load_global_config(&self) -> Result<GlobalConfig, ConfigError>;

    /// Load merged configuration (Global + Vault).
    ///
    /// # Business Rules
    /// - Loads both Global and Vault configurations
    /// - Merges using `Config::merge` with Vault precedence
    /// - Validates merged result
    ///
    /// # Errors
    /// Returns `ConfigError` if:
    /// - Load operation fails
    /// - Merge operation fails
    /// - Validation fails
    async fn load_merged_config(&self) -> Result<Config, ConfigError>;

    /// Load vault-specific configuration.
    ///
    /// # Errors
    /// Returns `ConfigError` if load operation fails or config is invalid.
    async fn load_vault_config(&self) -> Result<VaultConfig, ConfigError>;
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
