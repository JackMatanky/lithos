//! Template artifact pipeline states.
//!
//! This module defines the template artifact type-state pipeline. The initial
//! [`Rendered`] state holds rendered content tied back to the source
//! [`TemplateName`]; later write-pipeline states ([`TargetResolved`],
//! [`ReadyToCommit`], [`Committed`]) carry the resolved write target as the
//! artifact moves toward a committed vault file.

#![allow(dead_code, reason = "rendered artifacts are wired by issue-06")]

use trace_fs::RelativeFilePath;

use super::TemplateName;

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

/// State for an artifact validated and ready to commit to its target.
///
/// Holds the validated vault-relative path the artifact will be written to.
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
