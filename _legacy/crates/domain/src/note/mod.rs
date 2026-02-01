//! Note bounded context module.
//!
//! This module contains all entities, value objects, and logic related to the
//! Note aggregate and its subentities in the domain layer.

/// Core Note aggregate root and main entities.
pub(crate) mod aggregate;
/// Note domain events.
pub(crate) mod events;

/// Frontmatter value objects and logic.
pub(crate) mod frontmatter;
/// Link subentity for Note aggregate.
pub(crate) mod link;
/// Document structure subentities (Heading and Section) for Note aggregate.
pub(crate) mod structure;
/// Tag subentity for Note aggregate.
pub(crate) mod tag;
/// Task subentity for Note aggregate.
pub(crate) mod task;

// --- Public API & Re-exports ---

pub use aggregate::Note;
pub use events::{FrontmatterValidated, NoteCreated, NoteEvents};
pub use frontmatter::{FieldValue, FromFieldValue, Frontmatter};
pub use link::{
    Anchor as LinkAnchor, EmbedType, Link, Style as LinkStyle,
    Target as LinkTarget,
};
pub use structure::{Heading, Section};
pub use tag::Tag;
pub use task::{Task, TaskStatus};
