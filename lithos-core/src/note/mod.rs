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
//! - [`aggregate`] - Stable note identifiers and normalized facts.
//! - [`parser`] - Markdown parsing boundary (AST + frontmatter capture).
//! - [`raw`] - Raw extraction helpers (AST → Raw*).
//! - [`storage`] - Unified repository + redb adapter.
//! - [`task`], [`tag`], [`link`], [`list`] - Domain entities derived during
//!   conversion.

#![expect(
    clippy::module_name_repetitions,
    reason = "Public API names include module prefix for clarity"
)]

/// Note aggregate and identity types.
pub mod aggregate;
/// Note loader orchestration.
pub mod loader;
/// Markdown parser boundary.
pub mod parser;
/// Raw extraction helpers.
pub mod raw;
/// Unified repository storage.
pub mod storage;

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

/// Database table definitions for note storage.
pub(crate) const STORED_NOTES: redb::TableDefinition<&str, &[u8]> =
    redb::TableDefinition::new("stored_notes");
pub(crate) const NOTE_EVENTS: redb::TableDefinition<&str, &[u8]> =
    redb::TableDefinition::new("note_events");

pub(crate) const PATH_TO_ID: redb::MultimapTableDefinition<&str, &str> =
    redb::MultimapTableDefinition::new("path_to_id");
pub(crate) const TAGS_TO_NOTES: redb::MultimapTableDefinition<&str, &str> =
    redb::MultimapTableDefinition::new("tags_to_notes");
pub(crate) const ALIAS_TO_ID: redb::MultimapTableDefinition<&str, &str> =
    redb::MultimapTableDefinition::new("alias_to_id");
pub(crate) const FILE_CLASS_TO_ID: redb::MultimapTableDefinition<&str, &str> =
    redb::MultimapTableDefinition::new("file_class_to_id");
pub(crate) const FOLDER_TO_ID: redb::MultimapTableDefinition<&str, &str> =
    redb::MultimapTableDefinition::new("folder_to_id");
pub(crate) const TASKS_BY_COMPLETED_DATE: redb::MultimapTableDefinition<
    &str,
    &str,
> = redb::MultimapTableDefinition::new("tasks_by_completed_date");
pub(crate) const TASKS_BY_CREATED_DATE: redb::MultimapTableDefinition<
    &str,
    &str,
> = redb::MultimapTableDefinition::new("tasks_by_created_date");
pub(crate) const TASKS_BY_DUE_DATE: redb::MultimapTableDefinition<&str, &str> =
    redb::MultimapTableDefinition::new("tasks_by_due_date");
pub(crate) const TASKS_BY_REMINDER_DATE: redb::MultimapTableDefinition<
    &str,
    &str,
> = redb::MultimapTableDefinition::new("tasks_by_reminder_date");
pub(crate) const TASKS_BY_STATUS: redb::MultimapTableDefinition<&str, &str> =
    redb::MultimapTableDefinition::new("tasks_by_status");
#[expect(dead_code, reason = "Reserved for per-note task indexing")]
pub(crate) const TASKS_BY_NOTE: redb::MultimapTableDefinition<&str, &str> =
    redb::MultimapTableDefinition::new("tasks_by_note");
#[expect(dead_code, reason = "Reserved for future task table usage")]
pub(crate) const TASKS: redb::TableDefinition<&str, &[u8]> =
    redb::TableDefinition::new("tasks");
pub(crate) const TASKS_BY_METADATA: redb::MultimapTableDefinition<&str, &str> =
    redb::MultimapTableDefinition::new("tasks_by_metadata");
pub(crate) const TASKS_BY_DEPENDS_ON: redb::MultimapTableDefinition<
    &str,
    &str,
> = redb::MultimapTableDefinition::new("tasks_by_depends_on");
pub(crate) const FRONTMATTER_KV: redb::MultimapTableDefinition<&str, &str> =
    redb::MultimapTableDefinition::new("frontmatter_kv");
