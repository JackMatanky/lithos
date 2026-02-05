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

    /// Variable error.
    #[error("variable error: {0}")]
    Variable(String),

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

    /// Empty variable name.
    #[error("variable name cannot be empty")]
    EmptyVariableName,

    /// Variable name too long.
    #[error("variable name too long: {0} characters (max 32)")]
    VariableNameTooLong(usize),

    /// Invalid variable name.
    #[error("invalid variable name: {0}")]
    InvalidVariableName(String),

    /// Too many variables.
    #[error("maximum of 50 variables exceeded: {0}")]
    MaxVariablesExceeded(usize),

    /// Composition depth exceeded.
    #[error("maximum composition depth exceeded: {0}")]
    CompositionDepthExceeded(usize),

    /// Circular composition detected.
    #[error("circular composition detected: {0}")]
    CircularComposition(String),

    /// Variable not found.
    #[error("variable not found: {0}")]
    VariableNotFound(String),

    /// Variable type mismatch.
    #[error(
        "variable '{name}' type mismatch: expected {expected}, got {actual}"
    )]
    VariableTypeMismatch {
        /// Variable name.
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
    #[case(TemplateError::Variable("undefined var".into()))]
    #[case(TemplateError::Storage("io error".into()))]
    fn template_error_display_is_non_empty(#[case] error: TemplateError) {
        assert!(
            !error.to_string().is_empty(),
            "Error {error:?} should have non-empty display message"
        );
    }
}
