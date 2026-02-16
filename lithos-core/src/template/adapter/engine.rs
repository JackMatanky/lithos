use std::sync::Arc;

use minijinja::{AutoEscape, Environment, UndefinedBehavior};

use crate::template::error::TemplateError;

/// `MiniJinja` wrapper for template compilation and rendering.
///
/// # Architecture
/// - Owns Arc<Environment> for shared compiled templates
/// - Configures strict undefined behavior (fail on missing variables)
/// - Registers custom filters via `FilterRegistry`
/// - Caches compiled templates (compile once, render many)
pub struct TemplateEngine {
    env: Arc<Environment<'static>>,
}

impl TemplateEngine {
    /// Constructs engine with default configuration.
    ///
    /// Configuration:
    /// - Strict undefined behavior (fail on {{ `undefined_var` }})
    /// - Max template depth: 10 (prevent infinite recursion)
    /// - Auto-escape: None (we render Markdown, not HTML)
    /// - Custom filters registered (`validate_length`, `validate_pattern`,
    ///   etc.)
    #[must_use]
    #[inline]
    pub fn new() -> Self {
        let mut env = Environment::new();
        env.set_undefined_behavior(UndefinedBehavior::Strict);
        env.set_recursion_limit(10);
        env.set_auto_escape_callback(|_| AutoEscape::None);

        // Register custom filters
        super::FilterRegistry::register_all(&mut env);

        Self {
            env: Arc::new(env),
        }
    }

    /// Validates template syntax without compiling.
    ///
    /// # Errors
    /// - `TemplateError::Syntax`: Invalid `MiniJinja` syntax
    #[inline]
    pub fn validate_syntax(&self, source: &str) -> Result<(), TemplateError> {
        let mut env = Environment::new();
        env.set_undefined_behavior(UndefinedBehavior::Strict);
        env.set_recursion_limit(10);
        env.set_auto_escape_callback(|_| AutoEscape::None);
        super::FilterRegistry::register_all(&mut env);

        // TODO: This leaks memory. Use custom Source to avoid 'static
        // requirement or ensure minijinja::Environment supports dynamic
        // templates without leak.
        let source_static: &'static str =
            Box::leak(source.to_owned().into_boxed_str());

        env.add_template("__validate_temp__", source_static)
            .map_err(|e: minijinja::Error| TemplateError::Syntax(e.to_string()))
    }

    /// Compiles a template and adds to cache.
    ///
    /// # Errors
    /// - `TemplateError::Syntax`: Invalid `MiniJinja` syntax
    ///
    /// # Panics
    /// If Environment is not exclusively owned (should only happen during
    /// setup).
    #[inline]
    pub fn compile(
        &mut self,
        name: &str,
        source: &str,
    ) -> Result<(), TemplateError> {
        // Leak strings to make them static (templates are permanent application
        // state) This is necessary because Environment<'static>
        // requires &'static str
        let name_static: &'static str =
            Box::leak(name.to_owned().into_boxed_str());
        let source_static: &'static str =
            Box::leak(source.to_owned().into_boxed_str());

        #[expect(
            clippy::expect_used,
            reason = "Environment is exclusively owned during compilation"
        )]
        Arc::get_mut(&mut self.env)
            .expect("Environment should be exclusively owned during compile")
            .add_template(name_static, source_static)
            .map_err(|e: minijinja::Error| TemplateError::Syntax(e.to_string()))
    }

    /// Renders a compiled template with context.
    ///
    /// # Errors
    /// - `TemplateError::NotFound`: Template not compiled
    /// - `TemplateError::Render`: Rendering failed (undefined var, filter
    ///   error, etc.)
    #[inline]
    pub fn render<S: serde::Serialize>(
        &self,
        name: &str,
        context: S,
    ) -> Result<String, TemplateError> {
        let tmpl = self
            .env
            .get_template(name)
            .map_err(|_e| TemplateError::NotFound(name.into()))?;

        tmpl.render(context)
            .map_err(|e: minijinja::Error| TemplateError::Render(e.to_string()))
    }
}

impl Default for TemplateEngine {
    #[inline]
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
#[expect(clippy::disallowed_methods, reason = "Tests use unwrap")]
mod tests {
    use super::*;

    #[test]
    fn validates_template_syntax() {
        let engine = TemplateEngine::new();

        // Valid syntax
        engine.validate_syntax("Hello {{ name }}").unwrap();

        // Invalid syntax (unclosed tag)
        assert!(engine.validate_syntax("{{ unclosed").is_err());

        // Invalid syntax (unknown tag)
        assert!(engine.validate_syntax("{% unknown %}").is_err());
    }

    #[test]
    fn compiles_and_renders_simple_template() {
        let mut engine = TemplateEngine::new();

        engine.compile("test", "Hello {{ name }}!").unwrap();

        let output = engine
            .render("test", minijinja::context! { name => "World" })
            .unwrap();
        assert_eq!(output, "Hello World!");
    }

    #[test]
    fn renders_with_filter() {
        let mut engine = TemplateEngine::new();

        engine.compile("test", "{{ text | upper }}").unwrap();

        let output = engine
            .render("test", minijinja::context! { text => "hello" })
            .unwrap();
        assert_eq!(output, "HELLO");
    }

    #[test]
    fn fails_on_undefined_variable_strict_mode() {
        let mut engine = TemplateEngine::new();

        engine.compile("test", "Hello {{ undefined }}!").unwrap();

        let result = engine.render("test", minijinja::context! {});
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("undefined"));
    }
}
