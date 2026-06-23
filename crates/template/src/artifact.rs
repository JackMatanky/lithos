//! Template artifact pipeline states.
//!
//! This module defines the template artifact type-state pipeline. The initial
//! [`Rendered`] state holds rendered content tied back to the source
//! [`TemplateName`]; later write-pipeline states ([`TargetResolved`],
//! [`ReadyToCommit`], [`Committed`]) carry the resolved write target as the
//! artifact moves toward a committed vault file.

#![allow(dead_code, reason = "rendered artifacts are wired by issue-06")]

use std::path::Path;

use trace_fs::{FsWriter, RelativeFilePath, error::PathError};

use super::TemplateName;
use crate::error::{
    TemplateArtifactError, TemplatePathError, TemplateWriteError,
};

/// Marker state for an artifact that has rendered text but no output target.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct Rendered;

/// State for an artifact whose write target has been resolved.
///
/// Holds the validated vault-relative path the artifact will be written to.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct TargetResolved(RelativeFilePath);

impl TargetResolved {
    /// Returns the resolved vault-relative write target.
    #[inline]
    #[must_use]
    pub(crate) const fn path(&self) -> &RelativeFilePath {
        &self.0
    }
}

/// State for an artifact whose target directory has been ensured and is ready
/// for the atomic commit.
///
/// Holds the validated vault-relative path whose parent directory now exists,
/// so the artifact can be written in a single atomic create.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct ReadyToCommit(RelativeFilePath);

impl ReadyToCommit {
    /// Returns the vault-relative write target ready for commit.
    #[inline]
    #[must_use]
    pub(crate) const fn path(&self) -> &RelativeFilePath {
        &self.0
    }
}

/// Marker state for an artifact that has been committed to the vault.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct Committed;

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
    #[inline]
    #[must_use]
    pub(crate) const fn template(&self) -> &TemplateName {
        &self.template
    }

    /// Returns the rendered artifact content.
    #[inline]
    #[must_use]
    pub(crate) fn content(&self) -> &str {
        &self.content
    }

    /// Resolves and validates the output target path, advancing the artifact
    /// from [`Rendered`] to [`TargetResolved`].
    ///
    /// Validation is performed by [`RelativeFilePath::try_new`], which rejects
    /// absolute paths and `..` traversal. Rejections are mapped to named
    /// [`TemplateArtifactError`] variants by [`map_path_error`]; any other
    /// validation failure surfaces through
    /// [`TemplateArtifactError::InvalidPath`].
    ///
    /// # Errors
    ///
    /// Returns [`TemplateArtifactError::AbsolutePathRejected`] for absolute
    /// paths, [`TemplateArtifactError::TraversalRejected`] for `..` traversal,
    /// and [`TemplateArtifactError::InvalidPath`] for any other invalid path.
    pub(crate) fn try_resolve_target(
        &self,
        path: &str,
    ) -> Result<TemplateArtifact<TargetResolved>, TemplateArtifactError> {
        match RelativeFilePath::try_new(path) {
            Ok(target) => Ok(TemplateArtifact {
                template: self.template.clone(),
                content: self.content.clone(),
                state: TargetResolved(target),
            }),
            Err(err) => Err(map_path_error(err)),
        }
    }
}

impl TemplateArtifact<TargetResolved> {
    /// Ensures the parent directory of the resolved target exists, advancing
    /// the artifact from [`TargetResolved`] to [`ReadyToCommit`].
    ///
    /// This does NOT check whether the target file already exists — the
    /// conflict guard is the atomic `File::create_new` performed at commit
    /// time, which eliminates the TOCTOU race. Directory creation is
    /// idempotent, so an already-existing parent is not an error, and a
    /// top-level target whose parent is the vault root skips creation
    /// because the vault root already exists.
    ///
    /// # Errors
    ///
    /// Returns [`TemplateArtifactError::Write`] if the parent directory
    /// cannot be created.
    pub(crate) fn try_check_conflict(
        &self,
        fs_writer: &FsWriter,
    ) -> Result<TemplateArtifact<ReadyToCommit>, TemplateArtifactError> {
        let rel = self.state.path();
        let rel_str = rel.as_str();
        let target = Path::new(rel_str);

        if let Some(parent) = target.parent()
            && !parent.as_os_str().is_empty()
        {
            fs_writer
                .create_dir_all(parent)
                .map_err(|source| write_io_error(parent, source))?;
        }

        Ok(TemplateArtifact {
            template: self.template.clone(),
            content: self.content.clone(),
            state: ReadyToCommit(rel.clone()),
        })
    }
}

/// Maps a directory-creation [`std::io::Error`] into the template error
/// hierarchy via [`trace_fs::error::WriteError::Io`].
fn write_io_error(
    path: &Path,
    source: std::io::Error,
) -> TemplateArtifactError {
    TemplateArtifactError::Write(TemplateWriteError::from(
        trace_fs::error::WriteError::Io {
            path: path.to_path_buf(),
            source,
        },
    ))
}

/// Maps a [`PathError`] from target-path validation to a named
/// [`TemplateArtifactError`] variant.
///
/// The catch-all arm handles [`PathError::Empty`],
/// [`PathError::CurrentDirComponent`], and any future variants of the
/// `#[non_exhaustive]` [`PathError`] enum.
fn map_path_error(err: PathError) -> TemplateArtifactError {
    match err {
        PathError::NotRelative(path) => {
            TemplateArtifactError::AbsolutePathRejected(path)
        }
        PathError::ParentTraversal(path) => {
            TemplateArtifactError::TraversalRejected(path)
        }
        other => {
            TemplateArtifactError::InvalidPath(TemplatePathError::from(other))
        }
    }
}

#[cfg(test)]
mod tests {
    use std::path::Path;

    use super::{
        Committed, ReadyToCommit, Rendered, TargetResolved, TemplateArtifact,
        TemplateName,
    };

    fn template_name(name: &str) -> TemplateName {
        let path = format!("templates/{name}.md");
        TemplateName::try_new(Path::new(&path), Path::new("templates"))
            .expect("expected derivable template name")
    }

    mod state {
        use pretty_assertions::assert_eq;
        use trace_fs::RelativeFilePath;

        use super::{Committed, ReadyToCommit, TargetResolved};

        #[test]
        fn target_resolved_holds_path() {
            let state = TargetResolved(
                RelativeFilePath::try_new("notes/x.md")
                    .expect("expected valid relative file path"),
            );

            assert_eq!(state.path().as_str(), "notes/x.md");
        }

        #[test]
        fn ready_to_commit_holds_path() {
            let state = ReadyToCommit(
                RelativeFilePath::try_new("notes/x.md")
                    .expect("expected valid relative file path"),
            );

            assert_eq!(state.path().as_str(), "notes/x.md");
        }

        #[test]
        fn committed_is_zero_sized() {
            assert_eq!(std::mem::size_of::<Committed>(), 0);
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

    mod try_resolve_target {
        use pretty_assertions::assert_eq;

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

            assert_eq!(resolved.state.path().as_str(), "notes/x.md");
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
                Err(TemplateArtifactError::AbsolutePathRejected(_))
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
                Err(TemplateArtifactError::TraversalRejected(_))
            ));
        }

        #[test]
        fn rejects_invalid_path() {
            let artifact = TemplateArtifact::rendered(
                template_name("greeting"),
                "Hello Alice".to_owned(),
            );

            let result = artifact.try_resolve_target("");

            assert!(matches!(
                result,
                Err(TemplateArtifactError::InvalidPath(_))
            ));
        }
    }

    mod try_check_conflict {
        use pretty_assertions::assert_eq;
        use tempfile::TempDir;
        use trace_fs::FsWriter;

        use super::{TemplateArtifact, template_name};

        #[test]
        fn returns_ready_to_commit_when_dir_created() {
            let dir = TempDir::new().expect("expected temp dir");
            let fs_writer = FsWriter::new(dir.path());
            let resolved = TemplateArtifact::rendered(
                template_name("greeting"),
                "body".to_owned(),
            )
            .try_resolve_target("sub/x.md")
            .expect("expected valid relative path to resolve");

            let ready = resolved
                .try_check_conflict(&fs_writer)
                .expect("expected parent directory creation to succeed");

            assert_eq!(ready.state.path().as_str(), "sub/x.md");
            assert!(
                dir.path().join("sub").exists(),
                "expected parent directory to be created on disk"
            );
        }

        #[test]
        fn returns_ready_to_commit_when_dir_exists() {
            let dir = TempDir::new().expect("expected temp dir");
            std::fs::create_dir_all(dir.path().join("sub"))
                .expect("expected pre-created parent directory");
            let fs_writer = FsWriter::new(dir.path());
            let resolved = TemplateArtifact::rendered(
                template_name("greeting"),
                "body".to_owned(),
            )
            .try_resolve_target("sub/x.md")
            .expect("expected valid relative path to resolve");

            let ready = resolved
                .try_check_conflict(&fs_writer)
                .expect("expected idempotent directory creation to succeed");

            assert_eq!(ready.state.path().as_str(), "sub/x.md");
        }

        #[test]
        fn returns_ready_to_commit_for_top_level_target() {
            let dir = TempDir::new().expect("expected temp dir");
            let fs_writer = FsWriter::new(dir.path());
            let resolved = TemplateArtifact::rendered(
                template_name("greeting"),
                "body".to_owned(),
            )
            .try_resolve_target("x.md")
            .expect("expected valid relative path to resolve");

            let ready = resolved
                .try_check_conflict(&fs_writer)
                .expect("expected top-level target to skip directory creation");

            assert_eq!(ready.state.path().as_str(), "x.md");
        }
    }

    mod accessors {
        use pretty_assertions::assert_eq;

        use super::{Rendered, TemplateArtifact, template_name};

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

        #[test]
        fn rendered_state_is_rendered() {
            let artifact = TemplateArtifact::rendered(
                template_name("greeting"),
                "Hello Alice".to_owned(),
            );

            assert_eq!(artifact.state, Rendered);
        }
    }
}
