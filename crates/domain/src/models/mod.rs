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
/// Property and PropertySpec models.
pub mod property;
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
/// Template composition models.
pub mod template_comp;
/// Template variable models.
pub mod template_var;
