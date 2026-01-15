//! Domain models module.
//!
//! This module contains all domain entities and value objects.

pub mod config;
/// Frontmatter value objects and logic.
pub mod frontmatter;
/// Link subentity for Note aggregate.
pub mod link;
/// Note bounded context aggregate and main entities.
pub mod note;
/// Schema bounded context models.
pub mod schema;
/// Document structure subentities (Heading and Section) for Note aggregate.
pub mod structure;
/// Tag subentity for Note aggregate.
pub mod tag;
/// Task subentity for Note aggregate.
pub mod task;
/// Template bounded context models.
pub mod template;
