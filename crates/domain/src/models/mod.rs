//! Domain models module.
//!
//! This module contains all domain entities and value objects.

pub mod config;
/// Note bounded context - contains all Note aggregate entities and subentities.
pub mod note;
/// Schema bounded context models.
pub mod schema;
/// Template bounded context models.
pub mod template;
