//! Template artifact pipeline states.
//!
//! This module defines the template artifact type-state pipeline. The initial
//! [`Rendered`] state holds rendered content tied back to the source
//! [`TemplateName`]; later write-pipeline states ([`TargetResolved`],
//! [`Committed`]) carry the resolved write target as the
//! artifact moves toward a committed vault file.

use trace_fs::{FileWriter, WriteTarget};

use super::TemplateName;
use crate::error::TemplateArtifactError;

/// Marker state for an artifact that has rendered text but no output target.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct Rendered;

/// State for an artifact whose write target has been resolved.
///
/// Holds the validated vault-relative path the artifact will be written to.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct TargetResolved(WriteTarget);

impl TargetResolved {
    /// Returns the resolved vault-relative write target.
    #[inline]
    #[must_use]
    pub(crate) const fn path(&self) -> &WriteTarget {
        &self.0
    }
}

/// State for an artifact that has been committed to the vault.
///
/// Carries the validated [`WriteTarget`] the artifact was written to so the
/// committed path can be returned to the caller without re-resolving the
/// target.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct Committed(WriteTarget);

impl Committed {
    /// Returns the resolved vault-relative write target the artifact was
    /// committed to.
    #[inline]
    #[must_use]
    pub(crate) const fn path(&self) -> &WriteTarget {
        &self.0
    }
}

/// Template output artifact in a typed pipeline state.
///
/// The generic state marker makes invalid write-pipeline transitions
/// unrepresentable as later artifact states are added. The `state` field
/// carries any per-state data (such as the resolved write target).
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct TemplateArtifact<S> {
    template: TemplateName,
    content: String,
    state: S,
}

impl TemplateArtifact<Rendered> {
    /// Creates a rendered template artifact from source template name and text.
    #[inline]
    #[must_use]
    pub(crate) fn rendered(template: TemplateName, content: String) -> Self {
        Self {
            template,
            content,
            state: Rendered,
        }
    }

    /// Returns the source template name used to render this artifact.
    #[cfg(test)]
    #[inline]
    #[must_use]
    pub(crate) const fn template(&self) -> &TemplateName {
        &self.template
    }

    /// Returns the rendered artifact content.
    #[cfg(test)]
    #[inline]
    #[must_use]
    pub(crate) fn content(&self) -> &str {
        &self.content
    }

    /// Resolves and validates the output target path, advancing the artifact
    /// from [`Rendered`] to [`TargetResolved`].
    ///
    /// Validation is performed by [`WriteTarget::try_new`], which rejects
    /// absolute paths, `..` traversal, empty paths, current-dir (`.`), and
    /// hidden components (leading `.`).
    ///
    /// # Errors
    ///
    /// Returns [`TemplateArtifactError::Path`] wrapping the underlying
    /// [`trace_fs::error::WriteTargetError`] when the path fails validation.
    pub(crate) fn try_resolve_target(
        self,
        path: &str,
    ) -> Result<TemplateArtifact<TargetResolved>, TemplateArtifactError> {
        let target = WriteTarget::try_new(path)?;
        Ok(TemplateArtifact {
            template: self.template,
            content: self.content,
            state: TargetResolved(target),
        })
    }
}

impl TemplateArtifact<TargetResolved> {
    /// Returns the resolved vault-relative write target.
    #[inline]
    #[must_use]
    pub(crate) const fn target_path(&self) -> &WriteTarget {
        self.state.path()
    }

    /// Consumes the artifact and returns the owned rendered content.
    #[inline]
    #[must_use]
    pub(crate) fn into_content(self) -> String {
        self.content
    }

    /// Returns the byte length of the rendered content.
    ///
    /// String lengths are always `usize`. On any supported platform `usize`
    /// fits in `u64`, so the saturating conversion preserves the value.
    #[inline]
    #[must_use]
    pub(crate) fn content_len(&self) -> u64 {
        u64::try_from(self.content.len()).unwrap_or(u64::MAX)
    }

    /// Atomically writes the rendered content to the resolved target,
    /// advancing the artifact from [`TargetResolved`] to the terminal
    /// [`Committed`] state.
    ///
    /// Uses `File::create_new` via the FS writer, which fails with
    /// [`trace_fs::error::WriteError::AlreadyExists`] if the destination
    /// already exists. Performing the existence check and creation in one
    /// atomic operation eliminates the TOCTOU race a separate pre-check would
    /// introduce. Parent directories are automatically created if missing.
    ///
    /// # Errors
    ///
    /// Returns [`TemplateArtifactError::Write`] if the destination already
    /// exists or the file cannot be written.
    pub(crate) fn commit(
        self,
        writer: &impl FileWriter,
    ) -> Result<TemplateArtifact<Committed>, TemplateArtifactError> {
        writer.create_new(self.state.path(), self.content.as_bytes())?;

        let TemplateArtifact {
            template,
            content,
            state,
        } = self;
        Ok(TemplateArtifact {
            template,
            content,
            state: Committed(state.0),
        })
    }
}

impl TemplateArtifact<Committed> {
    /// Returns the validated write target the artifact was committed to.
    #[inline]
    #[must_use]
    pub(crate) const fn committed_path(&self) -> &WriteTarget {
        self.state.path()
    }
}

// ----------------------------------------------------------- //
//                            Tests                            //
// ----------------------------------------------------------- //

#[cfg(test)]
mod tests {
    use std::path::Path;

    use super::{
        Committed, Rendered, TargetResolved, TemplateArtifact, TemplateName,
    };

    fn template_name(name: &str) -> TemplateName {
        let path = format!("templates/{name}.md");
        TemplateName::try_new(Path::new(&path), Path::new("templates"))
            .expect("expected derivable template name")
    }

    mod state {
        use pretty_assertions::assert_eq;
        use trace_fs::WriteTarget;

        use super::{
            Committed, Rendered, TargetResolved, TemplateArtifact,
            template_name,
        };

        #[test]
        fn target_resolved_holds_write_target() {
            let target = WriteTarget::try_new("notes/x.md")
                .expect("expected valid write target");
            let state = TargetResolved(target.clone());

            assert_eq!(state.path(), &target);
        }

        #[test]
        fn committed_holds_write_target() {
            let target = WriteTarget::try_new("notes/x.md")
                .expect("expected valid write target");
            let state = Committed(target.clone());
            assert_eq!(state.path(), &target);
        }

        #[test]
        fn rendered_state_is_rendered() {
            let artifact = TemplateArtifact::rendered(
                template_name("greeting"),
                "Hello Alice".to_owned(),
            );

            assert_eq!(artifact.state, Rendered);
        }
    }

    mod constructor {
        use pretty_assertions::assert_eq;

        use super::{TemplateArtifact, template_name};

        #[test]
        fn stores_template_and_content() {
            let name = template_name("greeting");
            let artifact = TemplateArtifact::rendered(
                name.clone(),
                "Hello Alice".to_owned(),
            );

            assert_eq!(artifact.template(), &name);
            assert_eq!(artifact.content(), "Hello Alice");
        }
    }

    mod validation {
        use pretty_assertions::assert_eq;
        use trace_fs::error::WriteTargetError;

        use super::{
            super::super::error::TemplateArtifactError, TemplateArtifact,
            template_name,
        };

        #[test]
        fn returns_target_resolved_when_path_valid() {
            let name = template_name("greeting");
            let artifact = TemplateArtifact::rendered(
                name.clone(),
                "Hello Alice".to_owned(),
            );

            let resolved = artifact
                .try_resolve_target("notes/x.md")
                .expect("expected valid relative path to resolve");

            assert_eq!(
                resolved.state.path().as_path().to_str().unwrap(),
                "notes/x.md"
            );
            assert_eq!(resolved.template, name);
            assert_eq!(resolved.content.as_str(), "Hello Alice");
        }

        #[test]
        fn rejects_absolute_path() {
            let artifact = TemplateArtifact::rendered(
                template_name("greeting"),
                "Hello Alice".to_owned(),
            );

            let result = artifact.try_resolve_target("/abs/x.md");

            assert!(matches!(
                result,
                Err(TemplateArtifactError::Path(WriteTargetError::Absolute(_)))
            ));
        }

        #[test]
        fn rejects_traversal_path() {
            let artifact = TemplateArtifact::rendered(
                template_name("greeting"),
                "Hello Alice".to_owned(),
            );

            let result = artifact.try_resolve_target("../escape.md");

            assert!(matches!(
                result,
                Err(TemplateArtifactError::Path(WriteTargetError::Traversal(
                    _
                )))
            ));
        }

        #[test]
        fn rejects_empty_path() {
            let artifact = TemplateArtifact::rendered(
                template_name("greeting"),
                "Hello Alice".to_owned(),
            );

            let result = artifact.try_resolve_target("");

            assert!(matches!(
                result,
                Err(TemplateArtifactError::Path(WriteTargetError::Empty))
            ));
        }

        #[test]
        fn rejects_hidden_file_component() {
            let artifact = TemplateArtifact::rendered(
                template_name("greeting"),
                "Hello Alice".to_owned(),
            );

            let result = artifact.try_resolve_target(".config/x.md");

            assert!(matches!(
                result,
                Err(TemplateArtifactError::Path(WriteTargetError::Hidden(_)))
            ));
        }

        #[test]
        fn rejects_current_dir_component() {
            let artifact = TemplateArtifact::rendered(
                template_name("greeting"),
                "Hello Alice".to_owned(),
            );

            let result = artifact.try_resolve_target("./x.md");

            assert!(matches!(
                result,
                Err(TemplateArtifactError::Path(WriteTargetError::CurrentDir(
                    _
                )))
            ));
        }
    }

    mod create {
        use pretty_assertions::assert_eq;
        use tempfile::TempDir;
        use trace_fs::FsWriter;

        use super::{
            super::super::error::TemplateArtifactError, TemplateArtifact,
            template_name,
        };

        #[test]
        fn creates_file_with_content() {
            let dir = TempDir::new().expect("expected temp dir");
            let writer = FsWriter::new(dir.path());
            let artifact = TemplateArtifact::rendered(
                template_name("greeting"),
                "Hello".to_owned(),
            )
            .try_resolve_target("sub/x.md")
            .expect("resolve");

            let result = artifact.commit(&writer);
            assert!(result.is_ok(), "expected commit to succeed");

            let written = dir.path().join("sub/x.md");
            assert!(written.exists(), "expected committed file on disk");
            assert_eq!(
                std::fs::read_to_string(&written)
                    .expect("expected to read committed file"),
                "Hello"
            );
        }

        #[test]
        fn commit_preserves_target_on_committed_state() {
            let dir = TempDir::new().expect("expected temp dir");
            let writer = FsWriter::new(dir.path());
            let artifact = TemplateArtifact::rendered(
                template_name("greeting"),
                "Hello".to_owned(),
            )
            .try_resolve_target("sub/x.md")
            .expect("resolve");

            let committed = artifact.commit(&writer).expect("commit");
            assert_eq!(
                committed
                    .committed_path()
                    .as_path()
                    .to_str()
                    .expect("path utf8"),
                "sub/x.md"
            );
        }

        #[test]
        fn rejects_existing_file() {
            let dir = TempDir::new().expect("expected temp dir");
            std::fs::create_dir_all(dir.path().join("sub"))
                .expect("expected pre-created parent directory");
            std::fs::write(dir.path().join("sub/x.md"), b"existing")
                .expect("expected pre-created destination file");
            let writer = FsWriter::new(dir.path());
            let artifact = TemplateArtifact::rendered(
                template_name("greeting"),
                "Hello".to_owned(),
            )
            .try_resolve_target("sub/x.md")
            .expect("resolve");

            let result = artifact.commit(&writer);

            assert!(matches!(
                result,
                Err(TemplateArtifactError::Write(
                    trace_fs::error::WriteError::AlreadyExists { .. }
                ))
            ));
            assert_eq!(
                std::fs::read_to_string(dir.path().join("sub/x.md"))
                    .expect("expected to read original file"),
                "existing",
                "expected create_new to leave the original file unchanged"
            );
        }

        #[test]
        fn fails_with_io_error_on_non_already_exists_error() {
            // Test that a generic I/O error (like parent dir not writable)
            // surfaces as WriteError::Io.
            let dir = TempDir::new().expect("expected temp dir");
            let sub = dir.path().join("sub");
            std::fs::create_dir_all(&sub).expect("create dir");

            // On Unix, we can remove write permissions from the parent to force
            // an I/O error
            #[cfg(unix)]
            {
                use std::os::unix::fs::PermissionsExt;
                let mut perms = std::fs::metadata(&sub).unwrap().permissions();
                perms.set_mode(0o555); // read and execute, no write
                std::fs::set_permissions(&sub, perms).unwrap();
            }

            let writer = FsWriter::new(dir.path());
            let artifact = TemplateArtifact::rendered(
                template_name("greeting"),
                "Hello".to_owned(),
            )
            .try_resolve_target("sub/x.md")
            .expect("resolve");

            let result = artifact.commit(&writer);

            #[cfg(unix)]
            assert!(matches!(
                result,
                Err(TemplateArtifactError::Write(
                    trace_fs::error::WriteError::Io { .. }
                ))
            ));
        }
    }

    mod pipeline {
        use pretty_assertions::assert_eq;
        use tempfile::TempDir;
        use trace_fs::FsWriter;

        use super::{
            super::super::error::TemplateArtifactError, TemplateArtifact,
            template_name,
        };

        #[test]
        fn writes_file_end_to_end() {
            let dir = TempDir::new().expect("expected temp dir");
            let writer = FsWriter::new(dir.path());

            let result = TemplateArtifact::rendered(
                template_name("greeting"),
                "Hello, world!".to_owned(),
            )
            .try_resolve_target("notes/out.md")
            .expect("expected valid relative path to resolve")
            .commit(&writer);

            assert!(result.is_ok(), "expected commit to succeed");

            let written = dir.path().join("notes/out.md");
            assert!(written.exists(), "expected committed file on disk");
            assert_eq!(
                std::fs::read_to_string(&written)
                    .expect("expected to read committed file"),
                "Hello, world!"
            );
        }

        #[test]
        fn rejects_existing_file() {
            let dir = TempDir::new().expect("expected temp dir");
            std::fs::create_dir_all(dir.path().join("notes"))
                .expect("expected pre-created parent directory");
            std::fs::write(dir.path().join("notes/out.md"), b"existing")
                .expect("expected pre-created destination file");
            let writer = FsWriter::new(dir.path());

            let result = TemplateArtifact::rendered(
                template_name("greeting"),
                "Hello, world!".to_owned(),
            )
            .try_resolve_target("notes/out.md")
            .expect("expected valid relative path to resolve")
            .commit(&writer);

            assert!(matches!(
                result,
                Err(TemplateArtifactError::Write(
                    trace_fs::error::WriteError::AlreadyExists { .. }
                ))
            ));
            assert_eq!(
                std::fs::read_to_string(dir.path().join("notes/out.md"))
                    .expect("expected to read original file"),
                "existing",
                "expected create_new to leave the original file unchanged"
            );
        }
    }

    mod equality {
        use pretty_assertions::assert_eq;

        use super::{TemplateArtifact, template_name};

        #[test]
        fn maintains_partial_eq() {
            let name = template_name("greeting");
            let first = TemplateArtifact::rendered(
                name.clone(),
                "Hello Alice".to_owned(),
            );
            let second =
                TemplateArtifact::rendered(name, "Hello Alice".to_owned());

            assert_eq!(first, second);
        }
    }
}
