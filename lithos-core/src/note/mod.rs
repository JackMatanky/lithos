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
//! - **Port-Based CQRS**: Explicit separation of read and write operations.
//! - **File Source of Truth**: Parsed notes are ingest artifacts; stored
//!   projections are rebuildable caches.
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
//! # Layout
//!
//! The Note context is organized around ingest artifacts and projections:
//!
//! - [`identity`] - Stable note identifiers and validated names.
//! - [`reader`] - Markdown ingestion parser for note parsing.
//! - [`stored`] - Projection read models (`StoredNote`, `StoredTask`).
//! - [`ports`] - Command and Query trait definitions for CQRS.
//! - [`task`], [`tag`], [`link`], [`list`] - Sub-entities extracted during
//!   ingestion and stored in projections.

#![expect(
    clippy::module_name_repetitions,
    reason = "Public API names include module prefix for clarity"
)]

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
/// Note identity and validated names.
pub mod identity;
/// Note loader orchestration.
pub mod loader;
/// Note ports for CQRS.
pub mod ports;
/// Markdown note reader.
pub mod reader;
/// Stored note projections.
pub mod stored;

/// Frontmatter value objects and logic.
pub mod frontmatter;
/// Heading value objects.
pub mod heading;
/// Link value object.
pub mod link;
/// List value objects.
pub mod list;
/// Document structure values (Section and block references).
pub mod structure;
/// Tag value object.
pub mod tag;
/// Task value object.
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

/// Parsed note ingest artifact.
pub type ParsedNote = reader::ParsedNote;
