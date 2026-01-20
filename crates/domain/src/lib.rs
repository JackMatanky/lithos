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

// Re-export commonly used types for convenience.
// This provides a simplified public API for external crates without requiring
// deep module path knowledge (e.g., `lithos_domain::Config` vs
// `lithos_domain::config::Config`).

// Config context re-exports
pub use config::{
    aggregate::Config,
    events::{ConfigEvents, ConfigUpdated},
    global::{
        Filesystem as GlobalFilesystemConfig, Global as GlobalConfig,
        TrustedVaults as TrustedVaultsConfig,
    },
    types::{
        Frontmatter as FrontmatterConfig, Logging as LoggingConfig,
        Schema as SchemaConfig, SettingValue as ConfigValue,
        Template as TemplateConfig,
    },
    vault::{
        Filesystem as VaultFilesystem, Metadata as VaultMetadata,
        Vault as VaultConfig,
    },
};
pub use errors::{ConfigError, DomainError};
// Note context re-exports
pub use note::{
    Note,
    events::{FrontmatterValidated, NoteCreated, NoteEvents},
    frontmatter::{FieldValue, FromFieldValue, Frontmatter},
    link::{
        Anchor as LinkAnchor, EmbedType, Link, Style as LinkStyle,
        Target as LinkTarget,
    },
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
    PropertyBank, Schema, SchemaEvents, SchemaGraph, SchemaName,
    SchemaResolver,
    events::{PropertyBankUpdated, SchemaCreated},
    property::{Property, PropertyName},
    property_spec::{
        BoolSpec, DateSpec, FileSpec, NumberSpec, PropertySpec,
        PropertySpecTrait, PropertySpecType, StringSpec,
    },
    raw::{RawProperty, RawPropertyInline, RawPropertyRef, RawSchema},
};
// Template context re-exports
pub use template::{
    Composition as TemplateComposition, InsertionPosition,
    Metadata as TemplateMetadata, PlaceholderSyntax,
    Section as TemplateSection, Template, VariableDefinition,
    events::{TemplateCreated, TemplateEvents},
};
