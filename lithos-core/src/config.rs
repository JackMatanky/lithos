//! Configuration management for Lithos vaults.
//!
//! This module handles vault configuration, global settings, and config
//! merging. Configuration is loaded from TOML/JSON/YAML files in the vault
//! root.

/// Configuration error types.
#[expect(
    clippy::module_name_repetitions,
    reason = "Context-specific error name is intentional"
)]
#[non_exhaustive]
#[derive(Debug, thiserror::Error)]
pub enum ConfigError {
    /// Configuration is invalid.
    #[error("invalid configuration: {0}")]
    Invalid(String),
}

/// Stub configuration aggregate root.
///
/// Phase 3 will implement real configuration types.
#[non_exhaustive]
pub struct Config;
