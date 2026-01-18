//! Note bounded context module.
//!
//! This module contains all entities, value objects, and logic related to the Note aggregate
//! and its subentities in the domain layer.

/// Core Note aggregate root and main entities.
pub mod core;

/// Frontmatter value objects and logic.
pub mod frontmatter;
/// Link subentity for Note aggregate.
pub mod link;
/// Document structure subentities (Heading and Section) for Note aggregate.
pub mod structure;
/// Tag subentity for Note aggregate.
pub mod tag;
/// Task subentity for Note aggregate.
pub mod task;
