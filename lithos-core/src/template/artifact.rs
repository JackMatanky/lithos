//! Rendered template artifact state.
//!
//! This module starts the template artifact type-state pipeline. Issue 05 owns
//! only the initial [`Rendered`] state: rendered content tied back to the
//! source [`TemplateName`]. Later write-pipeline slices extend
//! [`TemplateArtifact`] with target resolution and commit states.

#![allow(dead_code, reason = "rendered artifacts are wired by issue-06")]

use std::marker::PhantomData;

use super::TemplateName;

/// Marker state for an artifact that has rendered text but no output target.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct Rendered;

/// Template output artifact in a typed pipeline state.
///
/// The generic state marker makes invalid write-pipeline transitions
/// unrepresentable as later artifact states are added.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct TemplateArtifact<State> {
    template: TemplateName,
    content: String,
    state: PhantomData<State>,
}

impl TemplateArtifact<Rendered> {
    /// Creates a rendered template artifact from source template name and text.
    #[inline]
    #[must_use]
    pub(crate) fn rendered(template: TemplateName, content: String) -> Self {
        Self {
            template,
            content,
            state: PhantomData,
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
