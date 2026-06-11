//! Error types for the Template context.
//!
//! Provides domain-level error types for template validation:
//! - [`TemplateNameError`] — stem derivation failures
//! - [`TemplateBodyError`] — empty content rejection
//! - [`TemplateError`] — top-level error embedding both via `#[from]`

use std::fmt;

// ============================================================================
// TemplateNameError
// ============================================================================

/// Errors returned when deriving a [`super::TemplateName`].
///
/// The only validation `TemplateName::try_new` performs is stem derivation.
/// Non-`.md` filtering and root-scope checks belong to the Template Processor.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TemplateNameError {
    /// The file path produced an empty stem when stripped and normalized.
    ///
    /// This occurs when the path has no file stem, or when the stripped
    /// relative path is empty.
    Derivation,
}

impl fmt::Display for TemplateNameError {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Derivation => {
                write!(
                    f,
                    "could not derive a template name from the given path"
                )
            }
        }
    }
}

impl std::error::Error for TemplateNameError {}

// ============================================================================
// TemplateBodyError
// ============================================================================

/// Errors returned when constructing a [`super::TemplateBody`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TemplateBodyError {
    /// The provided string was empty.
    ///
    /// Template body must contain at least one character.
    Empty,
}

impl fmt::Display for TemplateBodyError {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Empty => write!(f, "template body must not be empty"),
        }
    }
}

impl std::error::Error for TemplateBodyError {}

// ============================================================================
// TemplateError
// ============================================================================

/// Top-level error type for the Template context.
///
/// Embeds both [`TemplateNameError`] and [`TemplateBodyError`] via `#[from]`
/// conversions, following the `FsError` composition pattern.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TemplateError {
    /// A template name derivation error.
    Name(TemplateNameError),
    /// A template body validation error.
    Body(TemplateBodyError),
}

impl fmt::Display for TemplateError {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Name(e) => write!(f, "template name error: {e}"),
            Self::Body(e) => write!(f, "template body error: {e}"),
        }
    }
}

impl std::error::Error for TemplateError {
    #[inline]
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Name(e) => Some(e),
            Self::Body(e) => Some(e),
        }
    }
}

impl From<TemplateNameError> for TemplateError {
    #[inline]
    fn from(e: TemplateNameError) -> Self {
        Self::Name(e)
    }
}

impl From<TemplateBodyError> for TemplateError {
    #[inline]
    fn from(e: TemplateBodyError) -> Self {
        Self::Body(e)
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    mod template_name_error {
        use super::*;

        #[test]
        fn derivation_displays_meaningful_message() {
            let err = TemplateNameError::Derivation;
            let msg = err.to_string();
            assert!(!msg.is_empty(), "Display message should not be empty");
            assert!(
                msg.contains("template name"),
                "Display message should mention template name, got: {msg}"
            );
        }

        #[test]
        fn derivation_implements_error_trait() {
            let err = TemplateNameError::Derivation;
            let _: &dyn std::error::Error = &err;
        }
    }

    mod template_body_error {
        use super::*;

        #[test]
        fn empty_displays_meaningful_message() {
            let err = TemplateBodyError::Empty;
            let msg = err.to_string();
            assert!(!msg.is_empty(), "Display message should not be empty");
            assert!(
                msg.contains("empty"),
                "Display message should mention empty, got: {msg}"
            );
        }

        #[test]
        fn empty_implements_error_trait() {
            let err = TemplateBodyError::Empty;
            let _: &dyn std::error::Error = &err;
        }
    }

    mod template_error {
        use super::*;

        #[test]
        fn from_name_error_wraps_correctly() {
            let name_err = TemplateNameError::Derivation;
            let template_err: TemplateError = name_err.clone().into();
            assert_eq!(template_err, TemplateError::Name(name_err));
        }

        #[test]
        fn from_body_error_wraps_correctly() {
            let body_err = TemplateBodyError::Empty;
            let template_err: TemplateError = body_err.clone().into();
            assert_eq!(template_err, TemplateError::Body(body_err));
        }

        #[test]
        fn name_variant_displays_prefixed_message() {
            let err = TemplateError::Name(TemplateNameError::Derivation);
            let msg = err.to_string();
            assert!(
                msg.contains("template name error"),
                "Should contain 'template name error', got: {msg}"
            );
        }

        #[test]
        fn body_variant_displays_prefixed_message() {
            let err = TemplateError::Body(TemplateBodyError::Empty);
            let msg = err.to_string();
            assert!(
                msg.contains("template body error"),
                "Should contain 'template body error', got: {msg}"
            );
        }

        #[test]
        fn source_returns_inner_error_for_name_variant() {
            use std::error::Error;
            let err = TemplateError::Name(TemplateNameError::Derivation);
            assert!(
                err.source().is_some(),
                "source() should return Some for Name variant"
            );
        }

        #[test]
        fn source_returns_inner_error_for_body_variant() {
            use std::error::Error;
            let err = TemplateError::Body(TemplateBodyError::Empty);
            assert!(
                err.source().is_some(),
                "source() should return Some for Body variant"
            );
        }
    }
}
