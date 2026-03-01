//! Note bounded context for markdown-based document modeling and persistence.
//!
//! This module provides the core domain entities, value objects, and ports
//! required to model, parse, and persist Obsidian-compatible markdown notes.
//! It follows a Port-Based CQRS architecture, isolating the business logic
//! from the underlying storage layer.
//!
//! # Features
//!
//! - **Obsidian Compatibility**: Wiki-links, markdown links, YAML/TOML
//!   frontmatter, and hierarchical inline tags.
//! - **Port-Based CQRS**: Explicit separation of read ([`query::Query`]) and
//!   write ([`command::Command`]) operations.
//! - **Zero-Copy Serialization**: Optimized performance using `rkyv` for
//!   database storage and retrieval.
//! - **Rich Task Modeling**: Integrated task management with 7 specialized
//!   indexes for efficient querying.
//!
//! ## Notes
//!
//! - Tag extraction scans inline `#tags` in the markdown body; frontmatter tags
//!   remain available through the frontmatter API.
//! - Markdown task list markers only expose checked/unchecked states, so custom
//!   status symbols are not currently representable by the parser.
//!
//! # Usage
//!
//! ```
//! # use lithos_core::note::{aggregate::{Note, NoteId}, tag::Tag};
//! # fn main() -> Result<(), Box<dyn std::error::Error>> {
//! // Create a new note identity and path
//! let id = NoteId::new();
//! let path = "projects/lithos.md";
//!
//! // Construct the aggregate root
//! let mut note = Note::new(id, path)?;
//!
//! // Add domain components
//! note.add_tag(Tag::new("#research")?);
//! # Ok(())
//! # }
//! ```
//!
//! # Layout
//!
//! The Note context is organized around the [`aggregate`] root:
//!
//! - [`aggregate`] - The [`aggregate::Note`] root and primary domain entities.
//! - [`ports`] - Command and Query trait definitions for CQRS.
//! - [`command`] & [`query`] - Concrete implementations of the CQRS ports.
//! - [`adapter::reader`] - Markdown ingestion adapter for note parsing.
//! - [`task`], [`tag`], [`link`], [`list`] - Sub-entities owned by the
//!   [`aggregate::Note`].

#![allow(clippy::module_name_repetitions, reason = "Namespaced types")]

/// Note storage adapters.
pub mod adapter;
/// Core Note aggregate root and main entities.
pub mod aggregate;
/// Note command implementations (CQRS write operations).
pub mod command;
/// Note ports for CQRS.
pub mod ports;
/// Note query implementations (CQRS read operations).
pub mod query;

/// Frontmatter value objects and logic.
pub mod frontmatter;
/// Link subentity for Note aggregate.
pub mod link;
/// List subentities for Note aggregate.
pub mod list;
/// Document structure subentities (Heading and Section) for Note aggregate.
pub mod structure;
/// Tag subentity for Note aggregate.
pub mod tag;
/// Task subentity for Note aggregate.
pub mod task;

/// Note errors.
pub mod error;
/// Note domain events.
pub mod events;
/// Shared domain types for the Note context.
pub mod types;
/// Shared primitive for dynamic note values.
pub mod value;

pub(crate) mod db_table {
    use redb::{MultimapTableDefinition, TableDefinition};

    pub(crate) const NOTES: TableDefinition<&str, &[u8]> =
        TableDefinition::new("notes");

    pub(crate) const PATH_TO_ID: MultimapTableDefinition<&str, &str> =
        MultimapTableDefinition::new("path_to_id");
    pub(crate) const TAGS_TO_NOTES: MultimapTableDefinition<&str, &str> =
        MultimapTableDefinition::new("tags_to_notes");
    pub(crate) const ALIAS_TO_ID: MultimapTableDefinition<&str, &str> =
        MultimapTableDefinition::new("alias_to_id");
    pub(crate) const FILE_CLASS_TO_ID: MultimapTableDefinition<&str, &str> =
        MultimapTableDefinition::new("file_class_to_id");
    pub(crate) const FOLDER_TO_ID: MultimapTableDefinition<&str, &str> =
        MultimapTableDefinition::new("folder_to_id");
    pub(crate) const TASKS_BY_COMPLETED_DATE: MultimapTableDefinition<
        &str,
        &str,
    > = MultimapTableDefinition::new("tasks_by_completed_date");
    pub(crate) const TASKS_BY_CREATED_DATE: MultimapTableDefinition<
        &str,
        &str,
    > = MultimapTableDefinition::new("tasks_by_created_date");
    pub(crate) const TASKS_BY_DUE_DATE: MultimapTableDefinition<&str, &str> =
        MultimapTableDefinition::new("tasks_by_due_date");
    pub(crate) const TASKS_BY_PRIORITY: MultimapTableDefinition<&str, &str> =
        MultimapTableDefinition::new("tasks_by_priority");
    pub(crate) const TASKS_BY_PROJECT: MultimapTableDefinition<&str, &str> =
        MultimapTableDefinition::new("tasks_by_project");
    pub(crate) const TASKS_BY_REMINDER_DATE: MultimapTableDefinition<
        &str,
        &str,
    > = MultimapTableDefinition::new("tasks_by_reminder_date");
    pub(crate) const TASKS_BY_STATUS: MultimapTableDefinition<&str, &str> =
        MultimapTableDefinition::new("tasks_by_status");
    pub(crate) const FRONTMATTER_KV: MultimapTableDefinition<&str, &str> =
        MultimapTableDefinition::new("frontmatter_kv");
}

use self::{
    adapter::{command::CommandAdapter, query::QueryAdapter},
    command::Command,
    query::Query,
};

/// Redb-backed note command alias.
pub type RedbNoteCommand<'db, 'config> = Command<CommandAdapter<'db, 'config>>;

/// Redb-backed note query alias.
pub type RedbNoteQuery = Query<QueryAdapter>;
