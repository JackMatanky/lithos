//! Schema bounded context domain models.
//!
//! This module contains all entities and services for the schema system,
//! organized into focused modules for better maintainability.

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

pub type PropertyBank = aggregate::PropertyBank;
pub type Schema = aggregate::Schema;
pub type SchemaName = aggregate::SchemaName;
pub type SchemaError = error::SchemaError;
pub type Events = events::Events;
pub type PropertyBankUpdated = events::PropertyBankUpdated;
pub type SchemaCreated = events::SchemaCreated;
pub type Graph = graph::Graph;

pub trait Command: ports::Command {}
impl<T> Command for T where T: ports::Command + ?Sized {}

pub trait Query: ports::Query {}
impl<T> Query for T where T: ports::Query + ?Sized {}

pub type Property = property::Property;
pub type PropertyName = property::PropertyName;

pub type BoolSpec = property_spec::BoolSpec;
pub type DateSpec = property_spec::DateSpec;
pub type FileSpec = property_spec::FileSpec;
pub type NumberSpec = property_spec::NumberSpec;
pub type PropertySpec = property_spec::PropertySpec;
pub trait PropertySpecTrait: property_spec::PropertySpecTrait {}
impl<T> PropertySpecTrait for T where
    T: property_spec::PropertySpecTrait + ?Sized
{
}
pub type PropertySpecType = property_spec::PropertySpecType;
pub type StringSpec = property_spec::StringSpec;

pub type RawProperty = raw::RawProperty;
pub type RawPropertyInline = raw::RawPropertyInline;
pub type RawPropertyRef = raw::RawPropertyRef;
pub type RawSchema = raw::RawSchema;
pub type Resolver = resolver::Resolver;
