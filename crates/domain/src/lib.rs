//! Lithos Domain Crate.
//!
//! This crate contains the pure business logic, domain entities, and port definitions
//! for the Lithos system. It has no dependencies on external I/O or frameworks.
//!
//! # Architecture
//! - **Bounded Contexts**: Config, Note, Schema, Template (each owns models, events)
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

// Bounded Contexts
pub mod config;
pub mod note;
pub mod schema;
pub mod template;

// Cross-cutting concerns
pub mod errors;
pub mod ports;

// Internal validation utilities (not part of public API)
pub(crate) mod validation;

// Re-export commonly used types for convenience.
// This provides a simplified public API for external crates without requiring
// deep module path knowledge (e.g., `lithos_domain::Config` vs `lithos_domain::config::Config`).
// Config context re-exports
pub use config::{
    Config, FileSystem as FileSystemConfig, Frontmatter as FrontmatterConfig,
    Global as GlobalConfig, SettingValue as ConfigValue, Vault as VaultConfig,
    events::ConfigUpdated,
};
pub use errors::{ConfigError, DomainError};
// Note context re-exports
pub use note::{
    core::Note,
    events::{FrontmatterValidated, NoteCreated},
    frontmatter::{FieldValue, FromFieldValue, Frontmatter},
    link::{EmbedType, Link, LinkType},
    structure::{Heading, Section},
    tag::Tag,
    task::{Task, TaskStatus},
};
// Port re-exports
pub use ports::{
    config::{Command as ConfigCommand, Query as ConfigQuery},
    note::{Command as NoteCommand, Query as NoteQuery},
    schema::{Command as SchemaCommand, Query as SchemaQuery},
    template::{Command as TemplateCommand, Query as TemplateQuery},
};
// Schema context re-exports
pub use schema::{
    core::{DomainEvent as SchemaDomainEvent, Schema},
    events::{PropertyBankUpdated, SchemaCreated},
    property::{Property, PropertyName, RawProperty},
    property_bank::PropertyBank,
    property_spec::{
        BoolSpec, DateSpec, FileSpec, NumberSpec, PropertySpec,
        PropertySpecTrait, PropertySpecType, StringSpec,
    },
};
// Template context re-exports
pub use template::{
    Composition as TemplateComposition, DomainEvent as TemplateDomainEvent,
    InsertionPosition, Metadata as TemplateMetadata, PlaceholderSyntax,
    Section as TemplateSection, Template, VariableDefinition,
    events::TemplateCreated,
};
