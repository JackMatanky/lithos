//! Lithos Domain Crate.
//!
//! This crate contains the pure business logic, domain entities, and port
//! definitions for the Lithos system. It has no dependencies on external I/O or
//! frameworks.
//!
//! # Architecture
//! - **Bounded Contexts**: Config, Note, Schema, Template (each owns models,
//!   events)
//! - **Ports**: Trait interfaces for adapters (hexagonal architecture)
//! - **Errors**: Domain-specific error types (shared across contexts)
//! - **Validation**: Cross-context validation utilities (internal only)
//!
//! # Hexagonal Architecture Compliance
//! - NO external I/O dependencies
//! - NO framework dependencies
//! - Pure business logic only
//! - Adapters implement ports

#![allow(clippy::pub_use, reason = "Simplified public API for external crates")]

// Internal and External Modules
pub(crate) mod config;
pub mod errors;
pub(crate) mod note;
pub(crate) mod patterns;
pub mod ports;
pub(crate) mod schema;
pub(crate) mod template;
pub(crate) mod validation;

// --- Re-exports ---
// This provides a simplified public API for external crates without requiring
// deep module path knowledge.

// Config context re-exports
pub use config::{
    Config, ConfigEvents, ConfigUpdated, ConfigValue, FrontmatterConfig,
    GlobalConfig, GlobalFilesystemConfig, LoggingConfig, SchemaConfig,
    TemplateConfig, TrustedVaultsConfig, VaultConfig, VaultFilesystemConfig,
    VaultMetadataConfig,
};
// Error re-exports
pub use errors::{ConfigError, DomainError, FileLoaderError};
// Note context re-exports
pub use note::{
    EmbedType, FieldValue, FromFieldValue, Frontmatter, FrontmatterValidated,
    Heading, Link, LinkAnchor, LinkStyle, LinkTarget, Note, NoteCreated,
    NoteEvents, Section, Tag, Task, TaskStatus,
};
// Port re-exports
pub use ports::{
    ConfigCommand, ConfigQuery, NoteCommand, NoteQuery, SchemaCommand,
    SchemaQuery, TemplateCommand, TemplateQuery,
};
// Schema context re-exports
pub use schema::{
    BoolSpec, DateSpec, FileSpec, NumberSpec, Property, PropertyBank,
    PropertyBankUpdated, PropertyName, PropertySpec, PropertySpecTrait,
    PropertySpecType, RawProperty, RawPropertyInline, RawPropertyRef,
    RawSchema, Schema, SchemaCreated, SchemaEvents, SchemaGraph, SchemaName,
    SchemaResolver, StringSpec,
};
// Template context re-exports
pub use template::{
    InsertionPosition, PlaceholderSyntax, Template, TemplateComposition,
    TemplateCreated, TemplateEvents, TemplateMetadata, TemplateSection,
    VariableDefinition,
};
