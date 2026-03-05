//! Domain-centric configuration management for Lithos.
//!
//! This module provides the domain entities, validation logic, and storage
//! ports for Lithos configuration. It ensures that configuration is
//! "Always Valid" by performing strict validation during ingestion and
//! construction.
//!
//! Once a domain type like [`Config`] is constructed, it is guaranteed to be
//! internally consistent and valid for use throughout the system.
//!
//! # Features
//!
//! - **Layered Ingestion**: Merges defaults, global settings, and vault
//!   overrides using Figment.
//! - **Always Valid Invariants**: Strict type-driven validation at the domain
//!   boundary.
//! - **CQRS Architecture**: Separate Command and Query implementations
//!   decoupled via Ports.
//! - **Zero-Copy Persistence**: Optimized storage using `rkyv` and `redb`.
//!
//! # Usage
//!
//! ```rust
//! # use std::path::Path;
//! # use lithos_core::config::{
//! #     aggregate::{Config, Version},
//! #     vault::{VaultId, VaultRoot},
//! #     adapter::ingest::Ingestor
//! # };
//! # fn main() -> Result<(), Box<dyn std::error::Error>> {
//! let vault_root = Path::new("/path/to/vault");
//! let vault_id = VaultId::new();
//!
//! // 1. Ingest raw configuration from files
//! let ingestor = Ingestor::new(vault_root);
//! let raw = ingestor.build_merged_raw(vault_root)?;
//!
//! // 2. Transform into a validated domain aggregate
//! let config = Config::build(
//!     &raw,
//!     vault_id,
//!     VaultRoot::try_new(vault_root.to_path_buf())?,
//!     Version::initial(),
//! )?;
//!
//! // 3. Use the validated configuration
//! assert!(config.paths().cache.cache_dir().as_path().is_relative());
//! # Ok(())
//! # }
//! ```
//!
//! # Layout
//!
//! The configuration context is organized into three logical areas:
//!
//! ### Core Aggregates
//! These modules define the primary domain models and their invariants:
//! - [`aggregate`] - The [`Config`] aggregate root.
//! - [`global`] - Global-level configuration settings.
//! - [`vault`] - Vault-specific overrides and metadata.
//! - [`paths`] - Validated path configurations.
//!
//! ### CQRS Infrastructure
//! Implementation of write and read operations:
//! - [`command`] - Mutations and state changes (saving/rebuilding).
//! - [`query`] - Read-only access to configuration snapshots.
//! - [`ports`] - Trait definitions for storage decoupling.
//!
//! ### Supporting Modules
//! - [`ingest`] - Figment-based adapter for file loading.
//! - [`task`] - Task-specific schema and validation.
//! - [`logging`] / [`frontmatter`] - Focused domain building blocks.

#![allow(clippy::module_name_repetitions, reason = "Namespaced types")]

// ----------------------------------------------------------- //
//                   Core Aggregate Modules                    //
// ----------------------------------------------------------- //

/// Configuration storage adapters.
pub mod adapter;
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

/// Configuration command implementations (CQRS write operations).
pub mod command;
/// Configuration ports for CQRS.
pub mod ports;
/// Configuration query implementations (CQRS read operations).
pub mod query;

// ----------------------------------------------------------- //
//                  Supporting Domain Modules                  //
// ----------------------------------------------------------- //

/// Configuration error types.
pub mod error;
/// Configuration domain events.
pub mod events;
/// Frontmatter configuration types.
pub mod frontmatter;
/// Logging configuration types.
pub mod logging;
/// Raw (serde) configuration input types.
pub mod raw;
/// Task configuration schema and validation.
pub mod task;
/// Field specification and value validation types.
pub mod value;

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

    /// Stores metadata for config staleness checking.
    ///
    /// Keys:
    /// - `"global:{version}"` → Global config metadata
    /// - `"{vault_id}:{version}"` → Vault config metadata
    pub(crate) const CONFIG_METADATA: TableDefinition<&str, &[u8]> =
        TableDefinition::new("config_metadata");
}

use self::adapter::{command::CommandAdapter, query::QueryAdapter};

/// Redb-backed config command alias.
pub type RedbConfigCommand<'db> = command::Command<CommandAdapter<'db>>;

/// Redb-backed config query alias.
pub type RedbConfigQuery<'db> = query::Query<QueryAdapter<'db>>;

/// Redb-backed config service alias.
///
/// This is the recommended entry point for config operations. It provides
/// staleness detection and automatic reloading when configs change.
pub type RedbConfigService<'db> =
    crate::application::config::ConfigService<'db>;
