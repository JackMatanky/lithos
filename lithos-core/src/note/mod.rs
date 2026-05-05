//! Note bounded context for markdown-based document modeling and persistence.
//!
//! This module provides the core domain entities, value objects, and the
//! ingestion pipeline for Obsidian-compatible markdown notes.
//! It follows the File → Raw → Domain → Storage pipeline with a unified
//! repository interface.
//!
//! # Features
//!
//! - **Obsidian Compatibility**: Wiki-links, markdown links, YAML/TOML
//!   frontmatter, and hierarchical inline tags.
//! - **Unified Repository**: Read and write operations live on a single trait.
//! - **File Source of Truth**: Raw notes are ingest artifacts; stored facts are
//!   rebuildable caches.
//! - **Zero-Copy Serialization**: Optimized performance using `rkyv` for
//!   database storage and retrieval.
//! - **Rich Task Modeling**: Tasks are captured during ingestion and stored
//!   alongside note projections.
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
//! - [`aggregate`] - Stable note identifiers and normalized facts.
//! - [`parser`] - Markdown parsing boundary (SAX event sink + metadata
//!   capture).
//! - `processor` - File ingestion and persistence pipeline (File → Raw →
//!   Domain).
//! - [`raw`] - Raw note types.
//! - [`storage`] - Unified repository + redb adapter.
//! - [`task`], [`tag`], [`link`], [`list`] - Domain entities derived during
//!   conversion.

#![expect(
    clippy::module_name_repetitions,
    reason = "Public API names include module prefix for clarity"
)]

/// Note aggregate and identity types.
pub mod aggregate;
/// Markdown parser boundary.
pub mod parser;
/// Note processing pipeline.
pub mod processor;
/// Raw note types and helpers.
pub(crate) mod raw;
/// Unified repository storage.
pub mod storage;

/// Frontmatter value objects and logic.
pub mod frontmatter;
/// Heading value objects.
pub mod heading;
/// Inline field value objects.
pub mod inline_fields;
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
/// Query-optimized view projections.
pub mod views;

/// Note errors.
pub mod error;
/// Note domain events.
pub mod events;
/// Path value objects for the Note context.
pub mod paths;
/// Shared position primitives for the Note context.
pub mod position;
/// Unified scanning utilities.
pub mod scanner;
/// Shared primitive for dynamic note values.
pub mod value;

/// Database table definitions for note storage.
///
/// `NOTES_BY_ID` stores serialized [`Note`][crate::note::aggregate::Note]
/// values keyed by UUID v7 strings.
pub(crate) const NOTES_BY_ID: redb::TableDefinition<&str, &[u8]> =
    redb::TableDefinition::new("notes_by_id");

/// `NOTE_ID_BY_PATH` stores serialized
/// [`NoteId`][crate::note::aggregate::NoteId] values keyed by vault-relative
/// note paths.
pub(crate) const NOTE_ID_BY_PATH: redb::TableDefinition<&str, &[u8]> =
    redb::TableDefinition::new("note_id_by_path");

/// `LIST_VIEWS_BY_NOTE_ID` stores serialized
/// [`ListView`][crate::note::views::ListView] projections keyed by note UUID
/// strings. This is a rebuildable cache of query-optimized hierarchical list
/// representations.
pub(crate) const LIST_VIEWS_BY_NOTE_ID: redb::TableDefinition<&str, &[u8]> =
    redb::TableDefinition::new("list_views_by_note_id");
