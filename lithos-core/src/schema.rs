//! Schema management for structured note properties.
//!
//! Schemas define the shape of note frontmatter properties, enabling
//! validation and type checking across the vault.

/// Schema error types.
#[expect(
    clippy::module_name_repetitions,
    reason = "Context-specific error name is intentional"
)]
#[non_exhaustive]
#[derive(Debug, thiserror::Error)]
pub enum SchemaError {
    /// Schema is invalid.
    #[error("invalid schema: {0}")]
    Invalid(String),
    /// Property validation failed.
    #[error("property error: {0}")]
    Property(String),
}

/// Stub schema aggregate root.
///
/// Phase 3 will implement real schema types with properties and validation.
#[non_exhaustive]
pub struct Schema;
