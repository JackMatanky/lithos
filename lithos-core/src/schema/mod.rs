//! Schema bounded context domain models.
//!
//! This module contains all entities and services for the schema system,
//! organized into focused modules for better maintainability.

#![allow(clippy::pub_use, reason = "Re-exports provide clean public API")]
#![allow(clippy::module_name_repetitions, reason = "Namespaced types")]

pub(crate) mod aggregate;
pub mod error;
pub(crate) mod events;
pub(crate) mod graph;
pub mod ports;
pub(crate) mod property;
pub(crate) mod property_spec;
pub(crate) mod raw;
pub(crate) mod resolver;

// --- Public API & Re-exports ---

pub use aggregate::{PropertyBank, Schema, SchemaName};
pub use error::SchemaError;
pub use events::{Events, PropertyBankUpdated, SchemaCreated};
pub use graph::Graph;
pub use ports::{Command, Query};
pub use property::{Property, PropertyName};
pub use property_spec::{
    BoolSpec, DateSpec, FileSpec, NumberSpec, PropertySpec, PropertySpecTrait,
    PropertySpecType, StringSpec,
};
pub use raw::{RawProperty, RawPropertyInline, RawPropertyRef, RawSchema};
pub use resolver::Resolver;
