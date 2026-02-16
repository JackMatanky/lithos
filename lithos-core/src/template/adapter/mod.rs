//! MiniJinja adapter layer for template compilation and rendering.

#![expect(clippy::pub_use, reason = "Module re-exports adapter types")]

/// Emits MiniJinja source code from template metadata.
pub mod emitter;
/// Template engine wrapper.
pub mod engine;
/// Custom filters for input constraints.
pub mod filters;

/// Redb command adapter.
pub mod command;
/// Redb query adapter.
pub mod query;

pub use command::CommandAdapter;
pub use emitter::Emitter;
pub use engine::TemplateEngine;
pub use filters::FilterRegistry;
pub use query::QueryAdapter;
