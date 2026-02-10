//! Note bounded context for markdown-based document modeling and persistence.
//!
//! This module provides the core domain entities, value objects, and ports
//! required to model, parse, and persist Obsidian-compatible markdown notes.
//! It follows a Port-Based CQRS architecture, isolating the business logic
//! from the underlying storage layer.
//!
//! # Features
//!
//! - **Obsidian Compatibility**: Full support for wiki-links, frontmatter, and
//!   hierarchical tags.
//! - **Port-Based CQRS**: Explicit separation of read ([`query::Query`]) and
//!   write ([`command::Command`]) operations.
//! - **Zero-Copy Serialization**: Optimized performance using `rkyv` for
//!   database storage and retrieval.
//! - **Rich Task Modeling**: Integrated task management with 7 specialized
//!   indexes for efficient querying.
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
//! - [`parser`] - Markdown parsing adapter for extracting domain entities.
//! - [`task`], [`tag`], [`link`], [`list`] - Sub-entities owned by the
//!   [`aggregate::Note`].

#![allow(clippy::module_name_repetitions, reason = "Namespaced types")]

/// Core Note aggregate root and main entities.
pub mod aggregate;
/// Note command implementations (CQRS write operations).
pub mod command;
/// Note ports for CQRS.
pub mod ports;
/// Note query implementations (CQRS read operations).
pub mod query;

/// Markdown parsing adapter for extracting domain entities.
pub mod parser;

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
