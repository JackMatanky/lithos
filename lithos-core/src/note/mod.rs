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
//! let mut note = Note::try_new(id, path)?;
//!
//! // Add domain components
//! note.add_tag(Tag::try_new("#research")?);
//! # Ok(())
//! # }
//! ```
//!
//! # Layout
//!
//! The Note context is organized around the [`aggregate`] root:
//!
//! - [`aggregate`] - Legacy ingest artifact (`aggregate::Note`) used during
//!   parsing; projections are the source of truth.
//! - [`ports`] - Command and Query trait definitions for CQRS.
//! - [`command`] & [`query`] - CQRS facades for application use.
//! - [`reader`] - Markdown ingestion parser for note parsing.
//! - [`stored`] - Projection read models (`StoredNote`, `StoredTask`).
//! - [`task`], [`tag`], [`link`], [`list`] - Sub-entities extracted during
//!   ingestion and stored in projections.

#![expect(
    clippy::module_name_repetitions,
    reason = "Public API names include module prefix for clarity"
)]

/// Core Note aggregate root and main entities.
pub mod aggregate;
/// Note command implementations (CQRS write operations).
pub mod command;
/// Note storage adapters.
pub mod db_command;
/// Note storage query adapters.
pub mod db_query;
/// Note storage table definitions.
pub(crate) mod db_tables;
mod extract_frontmatter;
mod extract_heading;
mod extract_link;
mod extract_list;
mod extract_section;
mod extract_tag;
/// Note file ingestor.
pub mod ingestor;
/// Note loader orchestration.
pub mod loader;
/// Note ports for CQRS.
pub mod ports;
/// Note query implementations (CQRS read operations).
pub mod query;
/// Markdown note reader.
pub mod reader;
/// Stored note projections.
pub mod stored;

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
/// Path value objects for the Note context.
pub mod paths;
/// Shared position primitives for the Note context.
pub mod position;
/// Shared primitive for dynamic note values.
pub mod value;

use self::{
    command::Command, db_command::CommandAdapter, db_query::QueryAdapter,
    query::Query,
};

/// Note command type alias (storage-agnostic).
pub type NoteCommand<'db, 'config> = Command<CommandAdapter<'db, 'config>>;

/// Note query type alias (storage-agnostic).
pub type NoteQuery = Query<QueryAdapter>;
