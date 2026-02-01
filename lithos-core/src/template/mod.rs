//! Template bounded context models.

#![allow(clippy::module_name_repetitions, reason = "Namespaced types")]

/// Template aggregate root and main entities.
pub mod aggregate;
/// Template composition logic.
pub mod composition;
/// Template errors.
pub mod error;
/// Template domain events.
pub mod events;
/// Template ports for CQRS.
pub mod ports;
/// Template placeholder syntax.
pub mod syntax;
/// Template validation logic.
pub mod validation;
/// Template variable definitions.
pub mod variable;

// --- Public API ---
