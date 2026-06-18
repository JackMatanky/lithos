//! Error types for the Template context.
//!
//! Provides domain-level error types for template validation and persistence:
//! - [`TemplateError`] — top-level error embedding all domain and repository
//!   errors
//! - [`TemplateNameError`] — stem derivation failures
//! - [`TemplateBodyError`] — empty content rejection
//! - [`TemplateRepositoryError`] — persistence/storage errors

use thiserror::Error;

use crate::{db::DbError, fs::PathKey};

// ============================================================================
// TemplateError
// ============================================================================

/// Top-level error type for the Template context.
///
/// Embeds domain validation errors and repository errors via `#[from]`
/// conversions, following the `SchemaError` composition pattern.
#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub enum TemplateError {
    /// A template name derivation error.
    #[error("template name error: {0}")]
    Name(#[from] TemplateNameError),
    /// A template body validation error.
    #[error("template body error: {0}")]
    Body(#[from] TemplateBodyError),
    /// A template file read error.
    #[error(transparent)]
    Read(#[from] TemplateReadError),
    /// A template repository persistence error.
    #[error(transparent)]
    Repository(#[from] TemplateRepositoryError),
    /// A path validation error.
    #[error(transparent)]
    Path(#[from] TemplatePathError),
    /// A filesystem scanning error.
    #[error(transparent)]
    Scan(#[from] TemplateDirScanError),
}

// ============================================================================
// TemplateNameError
// ============================================================================

/// Errors returned when deriving a [`super::TemplateName`].
///
/// The only validation `TemplateName::try_new` performs is stem derivation.
/// Non-`.md` filtering and root-scope checks belong to the Template Processor.
#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum TemplateNameError {
    /// The file path produced an empty stem when stripped and normalized.
    ///
    /// This occurs when the path has no file stem, or when the stripped
    /// relative path is empty.
    #[error("could not derive a template name from the given path")]
    Derivation,
}

// ============================================================================
// TemplateBodyError
// ============================================================================

/// Errors returned when constructing a [`super::TemplateBody`].
#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum TemplateBodyError {
    /// The provided string was empty.
    ///
    /// Template body must contain at least one character.
    #[error("template body must not be empty")]
    Empty,
}

// ============================================================================
// TemplateReadError
// ============================================================================

/// Errors returned when reading a template file from the filesystem.
///
/// Wraps [`crate::fs::ReadError`] so file I/O failures surface through the
/// template error hierarchy without leaking `fs` internals into call sites.
#[derive(Debug, thiserror::Error)]
pub enum TemplateReadError {
    #[error(transparent)]
    Read(#[from] crate::fs::ReadError),
}

// ============================================================================
// TemplatePathError
// ============================================================================

/// Errors returned during path validation or resolution.
///
/// Wraps [`crate::fs::error::PathError`] so path failures surface through the
/// template error hierarchy without leaking `fs` internals into call sites.
#[derive(Debug, thiserror::Error)]
pub enum TemplatePathError {
    #[error(transparent)]
    Path(#[from] crate::fs::error::PathError),
}

// ============================================================================
// TemplateDirScanError
// ============================================================================

/// Errors returned during directory scanning.
///
/// Wraps [`crate::fs::error::ScanError`] so file discovery failures surface
/// through the template error hierarchy without leaking `fs` internals into
/// call sites.
#[derive(Debug, thiserror::Error)]
pub enum TemplateDirScanError {
    #[error(transparent)]
    Scan(#[from] crate::fs::error::ScanError),
}

// ============================================================================
// TemplateRepositoryError
// ============================================================================

/// Errors returned by template repository implementations.
#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub enum TemplateRepositoryError {
    /// Returned when the underlying storage layer fails.
    #[error(transparent)]
    Storage(#[from] DbError),

    /// Returned when a template is not found by its ID.
    #[error("template not found: {0}")]
    NotFoundById(super::aggregate::TemplateId),

    /// Returned when a template is not found by its path.
    #[error("template path not found: {0}")]
    NotFoundByPath(PathKey),
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

    mod template_read_error {
        use std::io;

        use super::*;

        #[test]
        fn template_read_error_wraps_fs_read_error_via_from() {
            let io_err =
                io::Error::new(io::ErrorKind::NotFound, "file not found");
            let fs_err = crate::fs::ReadError::Io {
                path: std::path::PathBuf::from("test.md"),
                source: io_err,
            };
            let read_err: TemplateReadError = fs_err.into();
            assert!(matches!(read_err, TemplateReadError::Read(_)));
        }
    }

    mod template_error {
        use super::*;

        #[test]
        fn from_name_error_wraps_correctly() {
            let name_err = TemplateNameError::Derivation;
            let template_err: TemplateError = name_err.clone().into();
            assert!(matches!(template_err, TemplateError::Name(_)));
        }

        #[test]
        fn from_body_error_wraps_correctly() {
            let body_err = TemplateBodyError::Empty;
            let template_err: TemplateError = body_err.clone().into();
            assert!(matches!(template_err, TemplateError::Body(_)));
        }

        #[test]
        fn from_repository_error_wraps_correctly() {
            let repo_err =
                crate::template::error::TemplateRepositoryError::NotFoundById(
                    crate::template::aggregate::TemplateId::new(),
                );
            let template_err: TemplateError = repo_err.into();
            assert!(matches!(template_err, TemplateError::Repository(_)));
        }

        #[test]
        fn template_error_read_variant_wraps_template_read_error_via_from() {
            use std::io;
            let io_err =
                io::Error::new(io::ErrorKind::NotFound, "file not found");
            let fs_err = crate::fs::ReadError::Io {
                path: std::path::PathBuf::from("test.md"),
                source: io_err,
            };
            let read_err = TemplateReadError::from(fs_err);
            let template_err: TemplateError = read_err.into();
            assert!(matches!(template_err, TemplateError::Read(_)));
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
