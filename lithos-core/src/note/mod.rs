//! Note bounded context module.
//!
//! This module contains all entities, value objects, and logic related to the
//! Note aggregate and its subentities in the domain layer.

#![allow(clippy::pub_use, reason = "Re-exports provide clean public API")]
#![allow(clippy::module_name_repetitions, reason = "Namespaced types")]

/// Core Note aggregate root and main entities.
pub(crate) mod aggregate;
/// Note errors.
pub mod error;
/// Note domain events.
pub(crate) mod events;
/// Frontmatter value objects and logic.
pub(crate) mod frontmatter;
/// Link subentity for Note aggregate.
pub(crate) mod link;
/// Note ports for CQRS.
pub mod ports;
/// Document structure subentities (Heading and Section) for Note aggregate.
pub(crate) mod structure;
/// Tag subentity for Note aggregate.
pub(crate) mod tag;
/// Task subentity for Note aggregate.
pub(crate) mod task;
// --- Public API & Re-exports ---
pub use aggregate::Note;
pub use error::NoteError;
pub use events::{FrontmatterValidated, NoteCreated, NoteEvents};
pub use frontmatter::{FieldValue, FromFieldValue, Frontmatter};
pub use link::{Anchor, EmbedType, Link, Style, Target};
pub use ports::{Command, Query};
pub use structure::{Heading, Section};
pub use tag::Tag;
pub use task::{Task, TaskStatus};
