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

/// Newtype for `MiniJinja` filter names to ensure consistency.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
#[non_exhaustive]
pub struct FilterName(pub &'static str);

impl FilterName {
    /// Formatter filter for dates.
    pub const DATE_FORMAT: Self = Self("date_format");
    /// Validation filter for file extensions.
    pub const VALIDATE_FILE_TYPE: Self = Self("validate_file_type");
    /// Validation filter for string length.
    pub const VALIDATE_LENGTH: Self = Self("validate_length");
    /// Validation filter for regex patterns.
    pub const VALIDATE_PATTERN: Self = Self("validate_pattern");
    /// Validation filter for numeric ranges.
    pub const VALIDATE_RANGE: Self = Self("validate_range");
    /// Validation filter for vault paths.
    pub const VAULT_PATH: Self = Self("vault_path");

    /// Returns the filter name as a string slice.
    #[must_use]
    #[inline]
    pub const fn as_str(&self) -> &'static str {
        self.0
    }
}

impl std::fmt::Display for FilterName {
    #[inline]
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}
