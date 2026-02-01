//! Template bounded context models.

#![allow(clippy::pub_use, reason = "Re-exports provide clean public API")]
#![allow(clippy::module_name_repetitions, reason = "Namespaced types")]

pub(crate) mod aggregate;
pub(crate) mod composition;
pub(crate) mod error;
pub(crate) mod events;
pub(crate) mod ports;
pub(crate) mod syntax;
pub(crate) mod validation;
pub(crate) mod variable;

// --- Public API & Re-exports ---

pub use aggregate::{Metadata, Template};
pub use composition::{Composition, InsertionPosition, Section};
pub use error::TemplateError;
pub use events::{Events, TemplateCreated};
pub use ports::{Command, Query};
pub use syntax::PlaceholderSyntax;
pub use variable::VariableDefinition;
