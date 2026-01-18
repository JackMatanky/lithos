//! Template domain models re-exports.

mod composition;
mod core;
mod validation;
mod variable;

pub use core::{DomainEvent, Metadata, Template};

pub use composition::{Composition, InsertionPosition, Section};
pub use variable::VariableDefinition;
