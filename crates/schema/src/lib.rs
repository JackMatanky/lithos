#![feature(trivial_bounds)]
//! Schema bounded context domain models.
//!
//! This module contains all entities and services for the schema system,
//! organized into focused modules for better maintainability.

#![expect(
    clippy::module_name_repetitions,
    reason = "Schema* types are namespaced intentionally for clarity"
)]

/// Schema aggregate and identifier types.
pub mod aggregate;
/// Base schema domain type (name-based extends list, pre-resolution).
pub mod base;
/// Core identity types for the schema system.
pub mod identifier;

/// Repository traits for the new schema seam (read, write, unified).
pub mod repository;
/// Redb-backed repository implementation for the new seam.
pub mod storage;

/// View types for staleness detection and versioned metadata tracking.
///
/// Provides versioned metadata containers ([`RawSchemaView`],
/// [`RawPropertyBankView`]) that enable incremental updates by tracking content
/// hashes, file timestamps, and version history. Views persist alongside domain
/// aggregates to answer "Has this file changed?" without re-parsing.
///
/// [`RawSchemaView`]: views::RawSchemaView
/// [`RawPropertyBankView`]: views::RawPropertyBankView
pub mod views;

/// PropertyBank domain aggregate for centralized property registration.
pub mod bank;
/// Typed processing pipeline for base schemas.
///
/// **Pipeline utility**: This module is `#[doc(hidden)] pub` to allow
/// tests to use the base processor directly.
#[doc(hidden)]
pub mod base_processor;
/// Property-bank reference expansion pipeline stage.
///
/// **Benchmark access**: This module is `#[doc(hidden)] pub` to allow
/// benchmarks to measure individual pipeline stages while hiding from public
/// documentation.
#[doc(hidden)]
pub mod expander;
/// PropertyBank state machine for incremental loading and staleness detection.
///
/// **Pipeline utility**: This module is `#[doc(hidden)] pub` to allow
/// tests to use the property bank processor directly.
#[doc(hidden)]
pub mod property_bank_processor;

/// Batch-based schema processor pipeline.
///
/// **Pipeline utility**: This module is `#[doc(hidden)] pub` to allow
/// builder and tests to use the new batch processor.
#[doc(hidden)]
pub mod schema_processor;

/// Core inheritance graph types.
pub mod inheritance;

/// Schema index types for efficient lookups.
pub(crate) mod index;

/// Atomic schema discovery engine.
pub(crate) mod discovery;

/// Shared delta computation utilities for schema ingestion.
///
/// **Pipeline utility**: This module is `#[doc(hidden)] pub` to allow
/// tests to use delta types directly.
#[doc(hidden)]
pub mod delta;
/// Schema errors.
pub mod error;
/// Schema domain events, pipeline events, and event handlers.
pub mod events;
/// Directed graph primitives used by schema inheritance resolution.
pub mod graph;

/// Facade for schema orchestration.
pub mod builder;
/// Property domain entities.
pub mod property;
/// Property specification variants.
pub mod property_spec;
/// Raw schema input definitions.
pub mod raw;

/// Schema-level property merging for inheritance.
///
/// **Benchmark access**: This module is `#[doc(hidden)] pub` to allow
/// benchmarks to measure individual pipeline stages while hiding from public
/// documentation.
#[doc(hidden)]
pub mod merger;

pub use graph::{
    DagGraph, Graph, GraphBuilder, GraphError, GraphNode, GraphNodeMut, Node,
};
