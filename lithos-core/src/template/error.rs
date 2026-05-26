//! Template error types.
//!
//! This module defines template-specific errors using thiserror for
//! structured error handling.

/// Template-related errors.
#[derive(Debug, thiserror::Error, Clone, PartialEq)]
#[non_exhaustive]
pub enum TemplateError {
    /// Template not found.
    #[error("template not found: {0}")]
    NotFound(String),

    /// Template already exists.
    #[error("template already exists: {0}")]
    AlreadyExists(String),

    /// Template validation failed.
    #[error("template validation failed: {0}")]
    ValidationFailed(String),

    /// Composition error.
    #[error("composition error: {0}")]
    Composition(String),

    /// Syntax error in template.
    #[error("syntax error: {0}")]
    Syntax(String),

    /// Rendering error.
    #[error("render error: {0}")]
    Render(String),

    /// Input error.
    #[error("input error: {0}")]
    Input(String),

    /// Storage error.
    #[error("storage error: {0}")]
    Storage(String),

    /// Empty template name.
    #[error("template name cannot be empty")]
    EmptyTemplateName,

    /// Template name too long.
    #[error("template name too long: {0} characters (max 64)")]
    TemplateNameTooLong(usize),

    /// Invalid template name.
    #[error("invalid template name: {0}")]
    InvalidTemplateName(String),

    /// Empty input name.
    #[error("input name cannot be empty")]
    EmptyInputName,

    /// Input name too long.
    #[error("input name too long: {0} characters (max 32)")]
    InputNameTooLong(usize),

    /// Invalid input name.
    #[error("invalid input name: {0}")]
    InvalidInputName(String),

    /// Too many inputs.
    #[error("maximum of 50 inputs exceeded: {0}")]
    MaxInputsExceeded(usize),

    /// Composition depth exceeded.
    #[error("maximum composition depth exceeded: {0}")]
    CompositionDepthExceeded(usize),

    /// Circular composition detected.
    #[error("circular composition detected: {0}")]
    CircularComposition(String),

    /// Input spec not found.
    #[error("input spec not found: {0}")]
    InputNotFound(String),

    /// Input type mismatch.
    #[error("input '{name}' type mismatch: expected {expected}, got {actual}")]
    InputTypeMismatch {
        /// Input name.
        name: String,
        /// Expected type.
        expected: String,
        /// Actual value/type.
        actual: String,
    },

    /// Invalid value type.
    #[error("invalid type for value '{value}': expected {expected}")]
    InvalidType {
        /// Actual value.
        value: String,
        /// Expected type description.
        expected: String,
    },

    /// Invalid regex pattern.
    #[error("invalid regex pattern: {0}")]
    InvalidRegex(String),

    /// Invalid date format.
    #[error("invalid date format: {0}")]
    InvalidDateFormat(String),

    /// Invalid file class/extension.
    #[error("invalid file: {0}")]
    InvalidFileClass(String),

    /// Template content too large.
    #[error("template content too large: {0} bytes (max {1})")]
    TemplateContentTooLarge(usize, usize),

    /// Repository error.
    #[error(transparent)]
    Repository(Box<TemplateRepositoryError>),
}

impl From<TemplateRepositoryError> for TemplateError {
    #[inline]
    fn from(error: TemplateRepositoryError) -> Self {
        Self::Repository(Box::new(error))
    }
}

/// Errors returned by template repository implementations.
#[derive(Debug, thiserror::Error, Clone, PartialEq)]
#[non_exhaustive]
pub enum TemplateRepositoryError {
    /// Returned when the underlying storage layer fails.
    #[error("storage error: {0}")]
    Storage(Box<str>),

    /// Returned when domain validation fails while saving or loading.
    #[error(transparent)]
    Domain(Box<TemplateError>),

    /// Returned when an expected entity is missing by ID.
    #[error("template not found: {0}")]
    NotFoundById(crate::template::aggregate::TemplateId),

    /// Returned when an expected entity is missing by name.
    #[error("template name not found: {0}")]
    NotFoundByName(crate::template::aggregate::TemplateName),
}

impl From<crate::db::DbError> for TemplateRepositoryError {
    #[inline]
    fn from(error: crate::db::DbError) -> Self {
        Self::Storage(error.to_string().into_boxed_str())
    }
}

impl From<TemplateError> for TemplateRepositoryError {
    #[inline]
    fn from(error: TemplateError) -> Self {
        Self::Domain(Box::new(error))
    }
}

#[cfg(test)]
mod tests {
    use rstest::rstest;

    use super::*;

    #[test]
    fn template_error_is_send_and_sync() {
        fn is_send_sync<T: Send + Sync>() {}
        is_send_sync::<TemplateError>();
    }

    #[rstest]
    #[case(TemplateError::NotFound("tpl".into()))]
    #[case(TemplateError::AlreadyExists("tpl".into()))]
    #[case(TemplateError::ValidationFailed("invalid".into()))]
    #[case(TemplateError::Composition("missing part".into()))]
    #[case(TemplateError::Syntax("invalid braces".into()))]
    #[case(TemplateError::Render("failed to render".into()))]
    #[case(TemplateError::Input("undefined input".into()))]
    #[case(TemplateError::Storage("io error".into()))]
    fn template_error_display_is_non_empty(#[case] error: TemplateError) {
        assert!(
            !error.to_string().is_empty(),
            "Error {error:?} should have non-empty display message"
        );
    }
}
