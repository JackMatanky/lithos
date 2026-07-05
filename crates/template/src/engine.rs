use std::collections::HashMap;

pub mod mini_jinja;
mod rendered;

pub use rendered::RenderedTemplate;

use crate::error::TemplateEngineError;

/// Template engine port.
///
/// Abstracts over rendering backends. Implementations load named templates
/// and produce [`RenderedTemplate`] for a set of variables.
pub trait TemplateEngine {
    /// Renders a template with the given variables.
    ///
    /// # Errors
    ///
    /// Returns [`TemplateEngineError::Render`] when the template cannot be
    /// found, parsed, or rendered.
    fn render(
        &self,
        name: &str,
        variables: &HashMap<String, String>,
    ) -> Result<RenderedTemplate, TemplateEngineError>;
}
