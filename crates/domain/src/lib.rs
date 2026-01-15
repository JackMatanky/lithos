//! Lithos Domain Crate.
//!
//! This crate contains the pure business logic, domain entities, and port definitions
//! for the Lithos system. It has no dependencies on external I/O or frameworks.
//!
//! # Architecture
//! - **Models**: Domain entities and value objects (pure business logic)
//! - **Ports**: Trait interfaces for adapters (hexagonal architecture)
//! - **Errors**: Domain-specific error types
//! - **Events**: Domain events for event-driven architecture
//!
//! # Hexagonal Architecture Compliance
//! - NO external I/O dependencies
//! - NO framework dependencies
//! - Pure business logic only
//! - Adapters implement ports

#![allow(clippy::pub_use, reason = "Simplified public API for external crates")]

pub mod errors;
pub mod events;
pub mod models;
pub mod ports;

// Re-export commonly used types for convenience.
// This provides a simplified public API for external crates without requiring
// deep module path knowledge (e.g., `lithos_domain::Config` vs `lithos_domain::models::config::Config`).
pub use errors::{ConfigError, DomainError};
pub use events::{
    ConfigUpdated, NoteCreated, NoteFrontmatterValidated, TemplateCreated,
};
pub use models::{
    config::{
        Config, FileSystem as FileSystemConfig,
        Frontmatter as FrontmatterConfig, Global as GlobalConfig,
        SettingValue as ConfigValue, Vault as VaultConfig,
    },
    note::{
        Embed, EmbedType, Frontmatter, FrontmatterValue, Heading, Link,
        LinkType, Note, Section, Tag, Task, TaskStatus,
    },
    schema::{
        BoolSpec, DateSpec, FileSpec, NumberSpec, Property, PropertyBank,
        PropertySpec, Schema, StringSpec,
    },
    template::{
        Composition as TemplateComposition, InsertionPosition,
        Metadata as TemplateMetadata, Section as TemplateSection, Template,
        VariableDefinition,
    },
};
pub use ports::{
    config::{Command as ConfigCommand, Query as ConfigQuery},
    note::{Command as NoteCommand, Query as NoteQuery},
    schema::{Command as SchemaCommand, Query as SchemaQuery},
    template::{Command as TemplateCommand, Query as TemplateQuery},
};
