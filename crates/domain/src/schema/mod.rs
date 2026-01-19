//! Schema bounded context domain models.
//!
//! This module contains all entities and services for the schema system,
//! organized into focused modules for better maintainability.

pub mod core;
pub mod events;
pub mod graph;
pub mod patterns;
pub mod property;
pub mod property_bank;
pub mod property_spec;
pub mod resolver;

// Re-export main types for convenience
#[expect(
    clippy::module_name_repetitions,
    reason = "SchemaName and DomainEvent follow domain naming conventions"
)]
pub use core::{DomainEvent, RawSchema, Schema, SchemaName};

#[expect(
    clippy::module_name_repetitions,
    reason = "SchemaGraph follows domain service naming conventions"
)]
pub use graph::SchemaGraph;
pub use property::{Property, PropertyName};
pub use property_bank::PropertyBank;
pub use property_spec::{
    BoolSpec, DateSpec, FileSpec, NumberSpec, PropertySpec, PropertySpecTrait,
    PropertySpecType, StringSpec,
};
#[expect(
    clippy::module_name_repetitions,
    reason = "SchemaResolver follows domain service naming conventions"
)]
pub use resolver::SchemaResolver;
