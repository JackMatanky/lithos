//! Template bounded context models.

/// Core Template aggregate root and identity.
pub(crate) mod aggregate;
/// Composition and modular template assembly logic.
pub(crate) mod composition;
/// Template domain events.
pub(crate) mod events;
/// Placeholder syntax and wrap logic.
pub(crate) mod syntax;
/// Domain-level structure and content validation.
pub(crate) mod validation;
/// Variable definition and type-safe validation.
pub(crate) mod variable;

pub use aggregate::{Metadata, Template};
pub use composition::{Composition, InsertionPosition, Section};
pub use syntax::PlaceholderSyntax;
pub use variable::VariableDefinition;
