//! Error types for the Template context.
//!
//! Provides domain-level error types for template validation and persistence:
//! - [`TemplateError`] — top-level error embedding all domain and repository
//!   errors
//! - [`TemplateNameError`] — stem derivation failures
//! - [`TemplateBodyError`] — empty content rejection
//! - [`TemplateRepositoryError`] — persistence/storage errors

use thiserror::Error;
use traces_db::DbError;
use traces_fs::PathKey;

use super::{TemplateEngineError, aggregate::TemplateName};

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
    /// A template engine error.
    #[error(transparent)]
    Engine(#[from] TemplateEngineError),
    /// A template artifact pipeline error.
    #[error(transparent)]
    Artifact(#[from] TemplateArtifactError),
    /// A requested template name was not found in the repository.
    #[error("template not found: {name}")]
    NotFound {
        /// Name of the missing template.
        name: TemplateName,
    },
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
/// Wraps [`traces_fs::ReadError`] so file I/O failures surface through the
/// template error hierarchy without leaking `fs` internals into call sites.
#[derive(Debug, thiserror::Error)]
pub enum TemplateReadError {
    /// Embedded filesystem read error.
    #[error(transparent)]
    Read(#[from] traces_fs::ReadError),
}

// ============================================================================
// TemplatePathError
// ============================================================================

/// Errors returned during path validation or resolution.
///
/// Wraps [`traces_fs::error::PathError`] so path failures surface through the
/// template error hierarchy without leaking `fs` internals into call sites.
#[derive(Debug, thiserror::Error)]
pub enum TemplatePathError {
    /// Embedded filesystem path validation error.
    #[error(transparent)]
    Path(#[from] traces_fs::error::PathError),
}

// ============================================================================
// TemplateArtifactError
// ============================================================================

/// Errors returned while resolving and committing a template artifact through
/// the write pipeline.
///
/// `Path` covers all target-validation rejections from
/// [`traces_fs::error::WriteTargetError`] — absolute, traversal, hidden, empty,
/// and current-dir. `Write` covers commit-time write failures from
/// [`traces_fs::error::WriteError`] — `AlreadyExists` and `Io`.
#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub enum TemplateArtifactError {
    /// A target path was rejected during target resolution.
    #[error(transparent)]
    Path(#[from] traces_fs::error::WriteTargetError),
    /// A filesystem write failed while committing the artifact.
    #[error(transparent)]
    Write(#[from] traces_fs::error::WriteError),
}

// ============================================================================
// TemplateDirScanError
// ============================================================================

/// Errors returned during directory scanning.
///
/// Wraps [`traces_fs::error::ScanError`] so file discovery failures surface
/// through the template error hierarchy without leaking `fs` internals into
/// call sites.
#[derive(Debug, thiserror::Error)]
pub enum TemplateDirScanError {
    /// Embedded directory scanning error.
    #[error(transparent)]
    Scan(#[from] traces_fs::error::ScanError),
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

    /// Returned when a template is not found by its derived name.
    #[error("template name not found: {0}")]
    NotFoundByName(super::aggregate::TemplateName),
}

// ----------------------------------------------------------- //
//                            Tests                            //
// ----------------------------------------------------------- //

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

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
            let fs_err = traces_fs::ReadError::Io {
                path: std::path::PathBuf::from("test.md"),
                source: io_err,
            };
            let read_err: TemplateReadError = fs_err.into();
            assert!(matches!(read_err, TemplateReadError::Read(_)));
        }
    }

    mod template_artifact_error {
        use std::error::Error;

        use super::*;

        #[test]
        fn path_variant_wraps_write_target_error() {
            let target_err = traces_fs::error::WriteTargetError::Absolute(
                PathBuf::from("/abs/x.md"),
            );
            let err: TemplateArtifactError = target_err.into();
            assert!(matches!(
                err,
                TemplateArtifactError::Path(
                    traces_fs::error::WriteTargetError::Absolute(_)
                )
            ));
        }

        #[test]
        fn write_variant_wraps_write_error() {
            let write_err = traces_fs::error::WriteError::AlreadyExists {
                path: PathBuf::from("note.md"),
            };
            let err: TemplateArtifactError = write_err.into();
            assert!(matches!(
                err,
                TemplateArtifactError::Write(
                    traces_fs::error::WriteError::AlreadyExists { .. }
                )
            ));
        }

        #[test]
        fn path_variant_preserves_display() {
            let inner = traces_fs::error::WriteTargetError::Absolute(
                PathBuf::from("/abs/x.md"),
            );
            let inner_msg = inner.to_string();
            let err = TemplateArtifactError::Path(inner);
            assert_eq!(
                err.to_string(),
                inner_msg,
                "transparent Path variant should forward inner Display"
            );
        }

        #[test]
        fn write_variant_preserves_source() {
            let io_err = std::io::Error::new(
                std::io::ErrorKind::PermissionDenied,
                "permission denied",
            );
            let err = TemplateArtifactError::Write(
                traces_fs::error::WriteError::Io {
                    path: PathBuf::from("note.md"),
                    source: io_err,
                },
            );
            assert!(
                err.source().is_some(),
                "transparent Write variant should forward its source"
            );
        }

        #[test]
        fn implements_error_trait() {
            let err = TemplateArtifactError::Path(
                traces_fs::error::WriteTargetError::Absolute(PathBuf::from(
                    "/abs/x.md",
                )),
            );
            let _: &dyn std::error::Error = &err;
        }
    }

    mod template_repository_error {
        use std::path::Path;

        use super::*;
        use crate::aggregate::TemplateName;

        #[test]
        fn not_found_by_name_displays_name() {
            let name = TemplateName::try_new(
                Path::new("templates/daily.md"),
                Path::new("templates"),
            )
            .expect("derivable template name");
            let err = TemplateRepositoryError::NotFoundByName(name);
            let msg = err.to_string();
            assert!(
                msg.contains("daily"),
                "Display should contain the template name, got: {msg}"
            );
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
            let repo_err = crate::error::TemplateRepositoryError::NotFoundById(
                crate::aggregate::TemplateId::new(),
            );
            let template_err: TemplateError = repo_err.into();
            assert!(matches!(template_err, TemplateError::Repository(_)));
        }

        #[test]
        fn template_error_read_variant_wraps_template_read_error_via_from() {
            use std::io;
            let io_err =
                io::Error::new(io::ErrorKind::NotFound, "file not found");
            let fs_err = traces_fs::ReadError::Io {
                path: std::path::PathBuf::from("test.md"),
                source: io_err,
            };
            let read_err = TemplateReadError::from(fs_err);
            let template_err: TemplateError = read_err.into();
            assert!(matches!(template_err, TemplateError::Read(_)));
        }

        #[test]
        fn from_engine_error_wraps_correctly() {
            let engine_err = template_engine_error();

            let template_err: TemplateError = engine_err.into();

            assert!(matches!(template_err, TemplateError::Engine(_)));
        }

        #[test]
        fn source_returns_inner_error_for_engine_variant() {
            use std::error::Error;

            let engine_err = template_engine_error();

            let template_err: TemplateError = engine_err.into();

            assert!(
                template_err.source().is_some(),
                "expected TemplateError::Engine to preserve source chain"
            );
        }

        fn template_engine_error() -> TemplateEngineError {
            let mut env = minijinja::Environment::new();
            let source = env
                .add_template_owned(
                    "invalid".to_owned(),
                    "Hello {{ name".to_owned(),
                )
                .expect_err("expected invalid template syntax to fail");

            TemplateEngineError::Compile {
                name: "invalid".to_owned(),
                source: Box::new(source),
            }
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

        #[test]
        fn not_found_displays_name() {
            let name = crate::aggregate::TemplateName::try_new(
                std::path::Path::new("templates/daily.md"),
                std::path::Path::new("templates"),
            )
            .expect("derivable template name");
            let err = TemplateError::NotFound {
                name,
            };
            let msg = err.to_string();
            assert!(
                msg.contains("daily"),
                "Display should contain the missing template name, got: {msg}"
            );
        }

        #[test]
        fn artifact_variant_wraps_template_artifact_error() {
            let artifact_err = TemplateArtifactError::Path(
                traces_fs::error::WriteTargetError::Absolute(PathBuf::from(
                    "/abs/x.md",
                )),
            );
            let template_err: TemplateError = artifact_err.into();
            assert!(matches!(template_err, TemplateError::Artifact(_)));
        }
    }
}
