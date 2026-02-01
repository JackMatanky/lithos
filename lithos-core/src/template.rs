//! Template management for note generation.
//!
//! Templates use `MiniJinja` syntax to generate notes from structured data.
//! This module provides template composition, variables, and validation.

/// Template error types.
#[expect(
    clippy::module_name_repetitions,
    reason = "Context-specific error name is intentional"
)]
#[non_exhaustive]
#[derive(Debug, thiserror::Error)]
pub enum TemplateError {
    /// Template is invalid.
    #[error("invalid template: {0}")]
    Invalid(String),
    /// Rendering failed.
    #[error("render error: {0}")]
    Render(String),
}

/// Stub template aggregate root.
///
/// Phase 3 will implement real template types with `MiniJinja` integration.
#[non_exhaustive]
pub struct Template;
