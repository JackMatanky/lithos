//! Template bounded context models.

/// Composition and modular template assembly logic.
pub mod composition;
/// Core Template aggregate root and identity.
pub mod core;
/// Template domain events.
pub mod events;
/// Placeholder syntax and wrap logic.
pub mod syntax;
/// Domain-level structure and content validation.
pub mod validation;
/// Variable definition and type-safe validation.
pub mod variable;

pub use core::{Metadata, Template};

pub use composition::{Composition, InsertionPosition, Section};
#[expect(
    clippy::module_name_repetitions,
    reason = "Context-specific event enum naming"
)]
pub use events::TemplateEvents;
pub use syntax::PlaceholderSyntax;
pub use variable::VariableDefinition;
