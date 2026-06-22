//! Template artifact domain state.

#![allow(dead_code, reason = "rendered artifacts are wired by issue-06")]

use std::marker::PhantomData;

use super::TemplateName;

/// Rendered artifact state produced by a template engine.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct Rendered;

/// Template output artifact in a typed pipeline state.
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
