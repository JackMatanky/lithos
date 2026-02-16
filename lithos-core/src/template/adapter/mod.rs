//! MiniJinja adapter layer for template compilation and rendering.

#![expect(clippy::pub_use, reason = "Module re-exports adapter types")]

/// Template engine wrapper.
pub mod engine;
/// Custom filters for input constraints.
pub mod filters;
/// Generates MiniJinja source code from template metadata.
pub mod source_generator;

pub use engine::TemplateEngine;
pub use filters::FilterRegistry;
pub use source_generator::SourceGenerator;
