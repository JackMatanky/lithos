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
//! # use std::sync::Arc;
//! # use lithos_core::config::{
//! #     builder::Builder,
//! #     storage::RedbStorage,
//! #     vault::VaultRoot,
//! # };
//! # use lithos_core::db::Store;
//! # fn main() -> Result<(), Box<dyn std::error::Error>> {
//! # let vault_root = std::path::Path::new("/path/to/vault");
//! # let db_path = std::path::Path::new("/tmp/test.redb");
//! # let store = Arc::new(Store::open(db_path)?);
//! // 1. Create builder with repository
//! let storage = RedbStorage::new(store);
//! let builder = Builder::new(vault_root, storage);
//!
//! // 2. Load configuration (with automatic staleness detection)
//! let config = builder.load()?;
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
//! - [`builder`] - Orchestrates: discover → process → merge → persist
//! - [`discovery`] - File discovery and cached-view lookup
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

/// Configuration storage traits.
pub mod repository;
/// Unified repository for configuration persistence.
pub mod storage;

pub use repository::{ReadRepository, Repository, WriteRepository};
#[cfg(any(test, feature = "testing"))]
pub use storage::{RedbRepository, RedbStorage};

// ----------------------------------------------------------- //

//                  Supporting Domain Modules                  //
// ----------------------------------------------------------- //

/// Configuration build orchestration with hybrid staleness detection.
pub mod builder;
/// Consolidated discovery logic for config files.
pub(crate) mod discovery;
/// Configuration error types.
pub mod error;
/// Configuration domain events.
pub mod events;
/// Frontmatter configuration types.
pub mod frontmatter;
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
/// Field specification and value validation types.
pub mod value;
/// View types for config staleness tracking.
pub mod views;

// ----------------------------------------------------------- //
//               Concrete Implementation Aliases               //
// ----------------------------------------------------------- //

// Removed legacy db_table definitions. Use crate::config::storage::tables
// instead.
