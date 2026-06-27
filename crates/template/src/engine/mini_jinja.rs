//! MiniJinja-backed template engine adapter.
//!
//! This module confines `MiniJinja` setup and rendering mechanics behind the
//! [`TemplateEngine`] port. The adapter owns the `MiniJinja` environment and
//! uses template names as lookup keys after `compile` registers supplied
//! source.

use std::collections::HashMap;

use minijinja::{AutoEscape, Environment, UndefinedBehavior};

use super::{RenderedTemplate, TemplateEngine, TemplateEngineError};
use crate::Template;

/// MiniJinja-backed implementation of the template engine port.
///
/// The environment is owned directly because foundation rendering is
/// single-process and does not need shared mutable ownership or
/// synchronization.
pub struct MiniJinjaEngine {
    env: Environment<'static>,
}

impl MiniJinjaEngine {
    /// Creates a `MiniJinja` engine with Traces foundation rendering settings.
    ///
    /// The configured engine rejects undefined variables and disables escaping
    /// so Markdown characters render unchanged.
    #[inline]
    #[must_use]
    pub fn configured() -> Self {
        let mut env = Environment::new();
        env.set_undefined_behavior(UndefinedBehavior::Strict);
        env.set_auto_escape_callback(|_| AutoEscape::None);

        Self {
            env,
        }
    }
}

impl TemplateEngine for MiniJinjaEngine {
    #[inline]
    fn compile(
        &mut self,
        template: &Template,
    ) -> Result<(), TemplateEngineError> {
        self.env
            .add_template_owned(
                template.name().as_ref().to_owned(),
                template.body().as_ref().to_owned(),
            )
            .map_err(|source| TemplateEngineError::Compile {
                name: template.name().as_ref().to_owned(),
                source: Box::new(source),
            })
    }

    #[inline]
    fn render(
        &self,
        template: &Template,
        context: &HashMap<String, String>,
    ) -> Result<RenderedTemplate, TemplateEngineError> {
        let loaded = self.env.get_template(template.name().as_ref()).map_err(
            |source| TemplateEngineError::Render {
                name: template.name().as_ref().to_owned(),
                source: Box::new(source),
            },
        )?;
        loaded.render(context).map(RenderedTemplate::new).map_err(|source| {
            TemplateEngineError::Render {
                name: template.name().as_ref().to_owned(),
                source: Box::new(source),
            }
        })
    }
}

// ----------------------------------------------------------- //
//                            Tests                            //
// ----------------------------------------------------------- //

#[cfg(test)]
mod tests {
    use std::{collections::HashMap, path::Path};

    use trace_fs::PathKey;

    use super::{MiniJinjaEngine, TemplateEngine};
    use crate::{Template, TemplateBody, TemplateId, TemplateName};

    fn template(template_name: &str, body: &str) -> Template {
        let path = format!("templates/{template_name}.md");
        let path_key = PathKey::try_new(&path).unwrap();
        let derived_name =
            TemplateName::try_new(Path::new(&path), Path::new("templates"))
                .unwrap();
        let body = TemplateBody::try_new(body).unwrap();

        Template::new(TemplateId::new(), path_key, derived_name, body)
    }

    mod compile {
        use super::*;

        #[test]
        fn returns_ok_when_template_source_is_valid() {
            let mut engine = MiniJinjaEngine::configured();
            let template = template("greeting", "Hello {{ name }}");

            let result = engine.compile(&template);

            assert!(
                result.is_ok(),
                "expected valid template source to compile"
            );
        }

        #[test]
        fn returns_error_with_source_when_template_syntax_is_invalid() {
            let mut engine = MiniJinjaEngine::configured();
            let template = template("invalid", "Hello {{ name");

            let err = engine
                .compile(&template)
                .expect_err("expected invalid template syntax to fail");

            assert!(
                std::error::Error::source(&err).is_some(),
                "expected engine error to preserve source error"
            );
        }
    }

    mod render {
        use pretty_assertions::assert_eq;

        use super::*;

        #[test]
        fn returns_rendered_text_when_context_provides_variable() {
            let mut engine = MiniJinjaEngine::configured();
            let template = template("greeting", "Hello {{ name }}");
            engine
                .compile(&template)
                .expect("expected template source to compile");
            let context =
                HashMap::from([("name".to_owned(), "Alice".to_owned())]);

            let text = engine
                .render(&template, &context)
                .expect("expected template to render");

            assert_eq!(text.as_str(), "Hello Alice");
        }

        #[test]
        fn returns_error_when_variable_is_undefined() {
            let mut engine = MiniJinjaEngine::configured();
            let template = template("greeting", "Hello {{ name }}");
            engine
                .compile(&template)
                .expect("expected template source to compile");

            let err = engine
                .render(&template, &HashMap::new())
                .expect_err("expected undefined variable to fail");

            assert!(
                std::error::Error::source(&err).is_some(),
                "expected render error to preserve source error"
            );
        }

        #[test]
        fn preserves_markdown_characters_when_auto_escape_is_disabled() {
            let mut engine = MiniJinjaEngine::configured();
            let template = template("markdown", "{{ markdown }}");
            engine
                .compile(&template)
                .expect("expected template source to compile");
            let markdown = "# Title *bold* [link]";
            let context =
                HashMap::from([("markdown".to_owned(), markdown.to_owned())]);

            let text = engine
                .render(&template, &context)
                .expect("expected markdown to render");

            assert_eq!(text.as_str(), markdown);
        }

        #[test]
        fn uses_compiled_source_instead_of_supplied_template_body() {
            let mut engine = MiniJinjaEngine::configured();
            let compiled = template("daily", "Hello {{ name }}");
            engine
                .compile(&compiled)
                .expect("expected template source to compile");
            let changed = template("daily", "Changed {{ name }}");
            let context =
                HashMap::from([("name".to_owned(), "Alice".to_owned())]);

            let text = engine
                .render(&changed, &context)
                .expect("expected compiled source to render");

            assert_eq!(text.as_str(), "Hello Alice");
        }

        #[test]
        fn returns_error_when_template_was_not_compiled() {
            let engine = MiniJinjaEngine::configured();
            let template = template("missing", "Hello");

            let err = engine
                .render(&template, &HashMap::new())
                .expect_err("expected uncompiled template to fail");

            assert!(
                std::error::Error::source(&err).is_some(),
                "expected lookup error to preserve source error"
            );
        }
    }

    mod template_engine_error {
        use std::error::Error;

        use minijinja::Environment;

        use crate::engine::TemplateEngineError;

        fn compile_source_error() -> minijinja::Error {
            let mut env = Environment::new();
            env.add_template_owned(
                "invalid".to_owned(),
                "Hello {{ name".to_owned(),
            )
            .expect_err("expected invalid template syntax to fail")
        }

        fn render_source_error() -> minijinja::Error {
            let env = Environment::new();
            env.get_template("missing")
                .expect_err("expected missing template lookup to fail")
        }

        mod compile {
            use pretty_assertions::assert_eq;

            use super::*;

            #[test]
            fn displays_template_name() {
                let source = compile_source_error();
                let err = TemplateEngineError::Compile {
                    name: "daily".to_owned(),
                    source: Box::new(source),
                };

                assert_eq!(
                    err.to_string(),
                    "failed to compile template `daily`"
                );
            }

            #[test]
            fn preserves_source_error() {
                let source = compile_source_error();
                let err = TemplateEngineError::Compile {
                    name: "daily".to_owned(),
                    source: Box::new(source),
                };

                assert!(
                    err.source().is_some(),
                    "expected compile error to preserve MiniJinja source"
                );
            }

            #[test]
            fn stores_template_name() {
                let source = compile_source_error();
                let err = TemplateEngineError::Compile {
                    name: "daily".to_owned(),
                    source: Box::new(source),
                };

                assert!(matches!(
                    err,
                    TemplateEngineError::Compile { ref name, .. } if name == "daily"
                ));
            }
        }

        mod render {
            use pretty_assertions::assert_eq;

            use super::*;

            #[test]
            fn displays_template_name() {
                let source = render_source_error();
                let err = TemplateEngineError::Render {
                    name: "daily".to_owned(),
                    source: Box::new(source),
                };

                assert_eq!(
                    err.to_string(),
                    "failed to render template `daily`"
                );
            }

            #[test]
            fn preserves_source_error() {
                let source = render_source_error();
                let err = TemplateEngineError::Render {
                    name: "daily".to_owned(),
                    source: Box::new(source),
                };

                assert!(
                    err.source().is_some(),
                    "expected render error to preserve MiniJinja source"
                );
            }

            #[test]
            fn stores_template_name() {
                let source = render_source_error();
                let err = TemplateEngineError::Render {
                    name: "daily".to_owned(),
                    source: Box::new(source),
                };

                assert!(matches!(
                    err,
                    TemplateEngineError::Render { ref name, .. } if name == "daily"
                ));
            }
        }
    }
}
