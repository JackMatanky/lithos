//! Template rendering port.

#![allow(dead_code, reason = "template service wiring lands in a later slice")]

use std::collections::HashMap;

use super::{
    Template,
    artifact::{Rendered, TemplateArtifact},
};

mod error;
pub(crate) mod mini_jinja;

pub use error::TemplateEngineError;

/// Rendering-engine boundary for checking and rendering supplied templates.
pub(crate) trait TemplateEngine {
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

    /// Renders a compiled template with a flat string context.
    ///
    /// # Errors
    ///
    /// Returns [`TemplateEngineError`] when the template was not registered or
    /// rendering fails.
    fn render(
        &self,
        template: &Template,
        context: &HashMap<String, String>,
    ) -> Result<TemplateArtifact<Rendered>, TemplateEngineError>;
}
