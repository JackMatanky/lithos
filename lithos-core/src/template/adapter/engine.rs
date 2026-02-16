use std::sync::Arc;

use minijinja::{AutoEscape, Environment, UndefinedBehavior};

use crate::template::error::TemplateError;

/// `MiniJinja` wrapper for template compilation and rendering.
///
/// # Architecture
/// - Owns Arc<Environment> for shared compiled templates
/// - Configures strict undefined behavior (fail on missing inputs)
/// - Registers custom filters via `FilterRegistry`
/// - Caches compiled templates (compile once, render many)
pub struct TemplateEngine {
    env: Arc<Environment<'static>>,
}

impl TemplateEngine {
    /// Constructs engine with default configuration.
    ///
    /// Configuration:
    /// - Strict undefined behavior (fail on {{ `undefined_input` }})
    /// - Max template depth: 10 (prevent infinite recursion)
    /// - Auto-escape: None (we render Markdown, not HTML)
    /// - Registers custom filters registered (`validate_length`,
    ///   `validate_pattern`, etc.)
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

    /// Compiles a template from its domain aggregate.
    ///
    /// This method uses the `SourceGenerator` to convert the template metadata
    /// into `MiniJinja` source code before compiling.
    ///
    /// # Errors
    /// - `TemplateError::Syntax`: Invalid `MiniJinja` syntax
    #[inline]
    pub fn compile_from_template(
        &mut self,
        template: &crate::template::aggregate::Template,
    ) -> Result<(), TemplateError> {
        let source = super::SourceGenerator::generate(template);
        self.compile(template.name().as_str(), &source)
    }

    /// Renders a compiled template with context.
    ///
    /// # Errors
    /// - `TemplateError::NotFound`: Template not compiled
    /// - `TemplateError::Render`: Rendering failed (undefined input, filter
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

    mod integration {
        use std::collections::HashMap;

        use super::*;
        use crate::template::{
            aggregate::{Template, TemplateName},
            block::{BlockStrategy, TemplateBlock},
        };

        #[test]
        fn multi_level_inheritance() {
            let mut engine = TemplateEngine::new();

            // 1. Grandparent
            let gp_name = TemplateName::try_from("grandparent").unwrap();
            let grandparent = Template::new(
                &gp_name,
                None,
                vec![TemplateBlock::new(
                    "base",
                    "GP:[{% block content %}{% endblock %}]",
                    BlockStrategy::Replace,
                )],
                HashMap::new(),
            )
            .unwrap();
            engine.compile_from_template(&grandparent).unwrap();

            // 2. Parent
            let p_name = TemplateName::try_from("parent").unwrap();
            let parent = Template::new(
                &p_name,
                Some(TemplateName::try_from("grandparent").unwrap()),
                vec![TemplateBlock::new(
                    "content",
                    "P({% block inner %}{% endblock %})",
                    BlockStrategy::Replace,
                )],
                HashMap::new(),
            )
            .unwrap();
            engine.compile_from_template(&parent).unwrap();

            // 3. Child
            let c_name = TemplateName::try_from("child").unwrap();
            let child = Template::new(
                &c_name,
                Some(TemplateName::try_from("parent").unwrap()),
                vec![TemplateBlock::new("inner", "C", BlockStrategy::Replace)],
                HashMap::new(),
            )
            .unwrap();
            engine.compile_from_template(&child).unwrap();

            let output =
                engine.render("child", minijinja::context! {}).unwrap();
            // Remove any newlines that might be generated by the generator
            let output_clean = output.replace('\n', "");
            assert_eq!(output_clean, "GP:[P(C)]");
        }

        #[test]
        fn block_strategies() {
            let mut engine = TemplateEngine::new();

            // Base
            let b_name = TemplateName::try_from("base").unwrap();
            let base = Template::new(
                &b_name,
                None,
                vec![
                    TemplateBlock::new("b1", "Base1", BlockStrategy::Replace),
                    TemplateBlock::new("b2", "Base2", BlockStrategy::Replace),
                    TemplateBlock::new("b3", "Base3", BlockStrategy::Replace),
                ],
                HashMap::new(),
            )
            .unwrap();
            engine.compile_from_template(&base).unwrap();

            // Child
            let c_name = TemplateName::try_from("child").unwrap();
            let child = Template::new(
                &c_name,
                Some(TemplateName::try_from("base").unwrap()),
                vec![
                    TemplateBlock::new("b1", "-Over-", BlockStrategy::Replace),
                    TemplateBlock::new("b2", "-Ext-", BlockStrategy::Extend),
                    TemplateBlock::new("b3", "-Pre-", BlockStrategy::Prepend),
                ],
                HashMap::new(),
            )
            .unwrap();
            engine.compile_from_template(&child).unwrap();

            let output =
                engine.render("child", minijinja::context! {}).unwrap();

            // b1: Replace "Base1" -> "-Over-"
            // b2: Extend "Base2" -> "Base2-Ext-"
            // b3: Prepend "Base3" -> "-Pre-Base3"
            assert!(output.contains("-Over-"));
            assert!(output.contains("Base2-Ext-"));
            assert!(output.contains("-Pre-Base3"));
        }
    }

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
    fn fails_on_undefined_input_strict_mode() {
        let mut engine = TemplateEngine::new();

        engine.compile("test", "Hello {{ undefined }}!").unwrap();

        let result = engine.render("test", minijinja::context! {});
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("undefined"));
    }
}
