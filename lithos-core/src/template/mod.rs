//! Template bounded context models.

#![allow(clippy::module_name_repetitions, reason = "Namespaced types")]
#![allow(clippy::pub_use, reason = "Re-exporting for convenience")]

/// MiniJinja adapter layer.
pub mod adapter;
/// Template aggregate root and main entities.
pub mod aggregate;
/// Template composition strategies.
pub mod block;
/// Template catalog for lifecycle orchestration.
pub mod catalog;
/// Template command implementations (Generic CQRS wrapper).
pub mod command;
/// Template errors.
pub mod error;
/// Template domain events.
pub mod events;
/// Template query implementations (Generic CQRS wrapper).
pub mod query;
/// Raw template input definitions.
pub mod raw;
/// Template repository traits and errors.
pub mod repository;
/// Template storage implementations.
pub mod storage;
/// Template input specifications.
pub mod value;

pub use aggregate::{InputName, Metadata, Template, TemplateId, TemplateName};
pub use block::{BlockStrategy, TemplateBlock};
pub use catalog::TemplateCatalog;
pub use value::InputSpec;

/// Generic command type alias to remove path stuttering: `template::Command` vs
/// `template::command::Command`.
pub type Command<R> = command::Command<R>;

/// Generic query type alias to remove path stuttering: `template::Query` vs
/// `template::query::Query`.
pub type Query<R> = query::Query<R>;
