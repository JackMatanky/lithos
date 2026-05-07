//! Domain-centric configuration management for Lithos.
//!
//! This module provides the domain entities, validation logic, and storage
//! for Lithos configuration. It ensures that configuration is
//! "Always Valid" by performing strict validation during ingestion and
//! construction.
//!
//! Once a domain type like [`Config`] is constructed, it is guaranteed to be
//! internally consistent and valid for use throughout the system.
//!
//! # Features
//!
//! - **Typestate Processing**: Single-file processors with compile-time state
//!   transitions for global and vault configs.
//! - **Field-Level Change Detection**: BLAKE3 hashing of individual config
//!   fields for incremental updates.
//! - **Hybrid Staleness Detection**: Combines timestamp checks (fast) with
//!   content hashing (accurate) to detect configuration changes.
//! - **Precedence Merging**: Vault configs override global configs override
//!   defaults, with field-level granularity.
//! - **Always Valid Invariants**: Strict type-driven validation at the domain
//!   boundary.
//! - **Unified Repository**: Single trait for all persistence operations.
//! - **Zero-Copy Persistence**: Optimized storage using `rkyv` and `redb`.
//!
//! # Usage
//!
//! ```rust,no_run
//! # use lithos_core::config::{
//! #     loader::Loader,
//! #     storage::RedbStorage,
//! #     vault::VaultRoot,
//! # };
//! # use lithos_core::db::Database;
//! # fn main() -> Result<(), Box<dyn std::error::Error>> {
//! # let vault_root = std::path::Path::new("/path/to/vault");
//! # let db_path = std::path::Path::new("/tmp/test.redb");
//! # let db = Database::open(db_path)?;
//! // 1. Create loader with repository
//! let storage = RedbStorage::new(&db);
//! let loader = Loader::new(vault_root, storage);
//!
//! // 2. Load configuration (with automatic staleness detection)
//! let config = loader.load()?;
//!
//! // 3. Use the validated configuration
//! assert!(config.paths().cache.cache_dir().as_path().is_relative());
//! # Ok(())
//! # }
//! ```
//!
//! # Architecture
//!
//! The configuration context follows a clean layered architecture:
//!
//! ## Core Domain
//! - [`aggregate`] - The [`Config`] aggregate root with versioning
//! - [`global`] - Global-level configuration settings
//! - [`vault`] - Vault-specific overrides and metadata
//! - [`paths`] - Validated path configurations
//!
//! ## Processing Pipeline
//! - [`processor`] - Typestate processor for single config files
//! - [`merger`] - Combines processor outcomes with precedence rules
//! - [`loader`] - Orchestrates: ingest → process → merge → persist
//! - [`ingestor`] - File discovery and TOML parsing
//!
//! ## Storage & Views
//! - [`storage`] - Unified Repository trait and redb implementation
//! - [`views`] - Raw config views for staleness tracking with version history
//!
//! ## Supporting Modules
//! - [`task`] - Task-specific schema and validation
//! - [`logging`] / [`frontmatter`] - Focused domain building blocks
//! - [`error`] - Structured error types
//! - [`events`] - Domain events

#![allow(clippy::module_name_repetitions, reason = "Namespaced types")]

// ----------------------------------------------------------- //
//                   Core Aggregate Modules                    //
// ----------------------------------------------------------- //

/// Configuration aggregate root.
pub mod aggregate;
/// Global configuration types and validation.
pub mod global;
/// Path configuration types.
pub mod paths;
/// Vault-scoped configuration types.
pub mod vault;

// ----------------------------------------------------------- //
//               Logic & Infrastructure Modules                //
// ----------------------------------------------------------- //

/// Unified repository for configuration persistence.
pub mod storage;

// ----------------------------------------------------------- //
//                  Supporting Domain Modules                  //
// ----------------------------------------------------------- //

/// Configuration error types.
pub mod error;
/// Configuration domain events.
pub mod events;
/// Frontmatter configuration types.
pub mod frontmatter;
/// Configuration file ingestion (Figment-based parsing).
pub mod ingestor;
/// Configuration loading orchestration with hybrid staleness detection.
pub mod loader;
/// Logging configuration types.
pub mod logging;
/// Config merging orchestration for processor outcomes.
pub mod merger;
/// Single-file typestate processor for config processing.
pub mod processor;
/// Raw (serde) configuration input types.
pub mod raw;
/// Task configuration schema and validation.
pub mod task;
/// Testing utilities (InMemoryRepository).
#[cfg(test)]
pub mod testing;
/// Field specification and value validation types.
pub mod value;
/// View types for config staleness tracking.
pub mod views;

// ----------------------------------------------------------- //
//               Concrete Implementation Aliases               //
// ----------------------------------------------------------- //

pub(crate) mod db_table {
    use redb::TableDefinition;

    /// Versioned global configuration.
    ///
    /// Keys: `"{version}"` → `Global`.
    /// Example: `"1"` → `Global { ... }`.
    pub(crate) const GLOBAL_CONFIG: TableDefinition<&str, &[u8]> =
        TableDefinition::new("global_config");

    /// Versioned vault-specific configuration.
    ///
    /// Keys: `"{vault_id}:{version}"` → `Vault`.
    /// Example: `"abc123:1"` → `Vault { ... }`.
    pub(crate) const VAULT_CONFIG: TableDefinition<&str, &[u8]> =
        TableDefinition::new("vault_config");

    /// Versioned final configuration (result of merging global + vault).
    ///
    /// Keys: `"{vault_id}:{version}"` → `Config`.
    /// Example: `"abc123:1"` → `Config { ... }`.
    pub(crate) const CONFIG_VERSIONS: TableDefinition<&str, &[u8]> =
        TableDefinition::new("config_versions");

    /// Vault path bidirectional mapping.
    ///
    /// Keys: `vault_root.as_key()` → `VaultId`.
    pub(crate) const VAULT_ID_BY_PATH: TableDefinition<&str, &[u8]> =
        TableDefinition::new("vault_id_by_path");

    /// Vault ID to path reverse mapping.
    ///
    /// Keys: `vault_id.to_string()` → `VaultRoot`.
    pub(crate) const VAULT_PATH_BY_ID: TableDefinition<&str, &[u8]> =
        TableDefinition::new("vault_path_by_id");

    /// Raw global config view with version history.
    ///
    /// Keys: `"global"` → `RawGlobalConfigView`.
    pub(crate) const RAW_GLOBAL_CONFIG_VIEW: TableDefinition<&str, &[u8]> =
        TableDefinition::new("raw_global_config_view");

    /// Raw vault config views with version history.
    ///
    /// Keys: `vault_id.to_string()` → `RawVaultConfigView`.
    pub(crate) const RAW_VAULT_CONFIG_VIEW: TableDefinition<&str, &[u8]> =
        TableDefinition::new("raw_vault_config_view");
}
