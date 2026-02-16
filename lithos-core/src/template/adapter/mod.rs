//! MiniJinja adapter layer for template compilation and rendering.

#![expect(clippy::pub_use, reason = "Module re-exports adapter types")]

/// Template engine wrapper.
pub mod engine;
/// Custom filters for variable constraints.
pub mod filters;
// pub mod source_generator; // Added in Phase 3

pub use engine::TemplateEngine;
pub use filters::FilterRegistry;
// pub use source_generator::SourceGenerator; // Added in Phase 3
