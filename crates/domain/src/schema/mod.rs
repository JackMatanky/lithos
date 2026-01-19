//! Schema bounded context domain models.
//!
//! This module contains all entities and services for the schema system,
//! organized into focused modules for better maintainability.

pub(crate) mod aggregate;
pub(crate) mod events;
pub(crate) mod graph;
pub(crate) mod property;
pub(crate) mod property_spec;
pub(crate) mod raw;
pub(crate) mod resolver;

// Re-export main types for convenience
pub use aggregate::{PropertyBank, Schema, SchemaName};
pub use events::SchemaEvents;
pub use graph::Graph as SchemaGraph;
pub use resolver::Resolver as SchemaResolver;
