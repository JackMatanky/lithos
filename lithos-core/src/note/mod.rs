//! Note bounded context module.
//!
//! This module contains all entities, value objects, and logic related to the
//! Note aggregate and its subentities in the domain layer.

#![allow(clippy::module_name_repetitions, reason = "Namespaced types")]

/// Core Note aggregate root and main entities.
pub mod aggregate;
/// Note command implementations (CQRS write operations).
pub mod command;
/// Note errors.
pub mod error;
/// Note domain events.
pub mod events;
/// Frontmatter value objects and logic.
pub mod frontmatter;
/// Link subentity for Note aggregate.
pub mod link;
/// Note ports for CQRS.
pub mod ports;
/// Note query implementations (CQRS read operations).
pub mod query;
/// Document structure subentities (Heading and Section) for Note aggregate.
pub mod structure;
/// Tag subentity for Note aggregate.
pub mod tag;
/// Task subentity for Note aggregate.
pub mod task;
/// Shared domain types for the Note context.
pub mod types;
/// Shared primitive for dynamic note values.
pub mod value;
// --- Public API ---
