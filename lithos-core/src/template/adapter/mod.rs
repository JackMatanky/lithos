//! MiniJinja adapter layer for template compilation and rendering.

#![expect(clippy::pub_use, reason = "Module re-exports adapter types")]

/// Template engine wrapper.
pub mod engine;
/// Custom filters for input constraints.
pub mod filters;
/// Generates MiniJinja source code from template metadata.
pub mod source_generator;

/// Redb command adapter.
pub mod command;
/// Redb query adapter.
pub mod query;

pub use command::CommandAdapter;
pub use engine::TemplateEngine;
pub use filters::FilterRegistry;
pub use query::QueryAdapter;
pub use source_generator::SourceGenerator;
