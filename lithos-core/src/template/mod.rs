//! Template bounded context models.

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

pub type Metadata = aggregate::Metadata;
pub type Template = aggregate::Template;
pub type Composition = composition::Composition;
pub type InsertionPosition = composition::InsertionPosition;
pub type Section = composition::Section;
pub type TemplateError = error::TemplateError;
pub type Events = events::Events;
pub type TemplateCreated = events::TemplateCreated;

pub trait Command: ports::Command {}
impl<T> Command for T where T: ports::Command + ?Sized {}

pub trait Query: ports::Query {}
impl<T> Query for T where T: ports::Query + ?Sized {}

pub type PlaceholderSyntax = syntax::PlaceholderSyntax;
pub type VariableDefinition = variable::VariableDefinition;
