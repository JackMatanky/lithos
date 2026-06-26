//! Template rendering engine port.
//!
//! The port accepts already-loaded Template domain values and already-supplied
//! render context data. It owns only engine-level source checking/loading and
//! rendering, leaving repository lookup, target resolution, conflict checks,
//! and file commits to the Template service/write pipeline.

use std::collections::HashMap;

use super::Template;

mod error;
pub mod mini_jinja;
mod rendered;

pub use error::TemplateEngineError;
pub use rendered::RenderedTemplate;

/// Rendering-engine boundary for checking and rendering supplied templates.
///
/// The trait intentionally has no `Clone`, `Send`, `Sync`, or `'static` bounds;
/// concrete orchestration needs should drive future bounds. `render` returns a
/// [`RenderedTemplate`] — the [`crate::TemplateService`] feeds it into a
/// [`crate::artifact::TemplateArtifact`] before driving the write pipeline,
/// keeping the artifact typestate confined to the crate.
pub trait TemplateEngine {
    /// Registers and checks the supplied template source in the engine.
    ///
    /// # Errors
    ///
    /// Returns [`TemplateEngineError`] when the engine rejects the template
    /// source.
    fn compile(
        &mut self,
        template: &Template,
    ) -> Result<(), TemplateEngineError>;

    /// Renders a compiled template with a flat string context, returning the
    /// rendered text wrapped in a [`RenderedTemplate`].
    ///
    /// # Errors
    ///
    /// Returns [`TemplateEngineError`] when the template was not registered or
    /// rendering fails.
    fn render(
        &self,
        template: &Template,
        context: &HashMap<String, String>,
    ) -> Result<RenderedTemplate, TemplateEngineError>;
}
