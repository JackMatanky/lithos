//! Template rendering engine port.
//!
//! The port accepts already-loaded Template domain values and already-supplied
//! render context data. It owns only engine-level source checking/loading and
//! rendering, leaving repository lookup, target resolution, conflict checks,
//! and file commits to the Template service/write pipeline.
//!
//! [`TemplateEngineError`] carries a boxed source error so the port does not
//! depend on adapter-specific types. The adapter (`mini_jinja`) constructs
//! errors by boxing its native error type.

use std::collections::HashMap;

use super::Template;

pub mod mini_jinja;
mod rendered;

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

/// Error returned by template engine operations.
///
/// Each variant stores the template name for user-facing diagnostics and keeps
/// the adapter source error in the standard source chain via a boxed
/// `dyn Error` so the port does not depend on adapter-specific types.
#[derive(Debug, thiserror::Error)]
pub enum TemplateEngineError {
    /// The engine rejected template source during compile/load.
    #[error("failed to compile template `{name}`")]
    Compile {
        /// Name of the template that failed to compile.
        name: String,
        /// Source error returned by the rendering engine adapter.
        #[source]
        source: Box<dyn std::error::Error + Send + Sync>,
    },
    /// The engine failed to look up or render a compiled template.
    #[error("failed to render template `{name}`")]
    Render {
        /// Name of the template that failed to render.
        name: String,
        /// Source error returned by the rendering engine adapter.
        #[source]
        source: Box<dyn std::error::Error + Send + Sync>,
    },
}
