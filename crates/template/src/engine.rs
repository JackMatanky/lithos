use std::collections::HashMap;

pub mod mini_jinja;
mod rendered;

pub use rendered::RenderedTemplate;

use crate::error::TemplateEngineError;

pub trait TemplateEngine {
    fn render(
        &self,
        name: &str,
        variables: &HashMap<String, String>,
    ) -> Result<RenderedTemplate, TemplateEngineError>;
}
