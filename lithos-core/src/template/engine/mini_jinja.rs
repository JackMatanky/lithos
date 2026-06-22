//! MiniJinja-backed template engine adapter.

#![allow(dead_code, reason = "template service wiring lands in a later slice")]

use std::collections::HashMap;

use minijinja::{AutoEscape, Environment, UndefinedBehavior};

use super::{TemplateEngine, TemplateEngineError};
use crate::template::{
    Template,
    artifact::{Rendered, TemplateArtifact},
};

/// MiniJinja-backed implementation of the template engine port.
pub(crate) struct MiniJinjaEngine {
    env: Environment<'static>,
}

impl MiniJinjaEngine {
    /// Creates a `MiniJinja` engine with Lithos foundation rendering settings.
    #[inline]
    #[must_use]
    pub(crate) fn configured() -> Self {
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
                source,
            })
    }

    #[inline]
    fn render(
        &self,
        template: &Template,
        context: &HashMap<String, String>,
    ) -> Result<TemplateArtifact<Rendered>, TemplateEngineError> {
        let loaded = self.env.get_template(template.name().as_ref()).map_err(
            |source| TemplateEngineError::Render {
                name: template.name().as_ref().to_owned(),
                source,
            },
        )?;
        let rendered_text = loaded.render(context).map_err(|source| {
            TemplateEngineError::Render {
                name: template.name().as_ref().to_owned(),
                source,
            }
        })?;

        Ok(TemplateArtifact::rendered(template.name().clone(), rendered_text))
    }
}

#[cfg(test)]
mod tests {
    use std::{collections::HashMap, path::Path};

    use super::{MiniJinjaEngine, TemplateEngine};
    use crate::{
        fs::PathKey,
        template::{Template, TemplateBody, TemplateId, TemplateName},
    };

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
        fn returns_rendered_artifact_when_context_provides_variable() {
            let mut engine = MiniJinjaEngine::configured();
            let template = template("greeting", "Hello {{ name }}");
            engine
                .compile(&template)
                .expect("expected template source to compile");
            let context =
                HashMap::from([("name".to_owned(), "Alice".to_owned())]);

            let artifact = engine
                .render(&template, &context)
                .expect("expected template to render");

            assert_eq!(artifact.content(), "Hello Alice");
        }

        #[test]
        fn returns_artifact_with_template_name_when_context_provides_variable()
        {
            let mut engine = MiniJinjaEngine::configured();
            let template = template("greeting", "Hello {{ name }}");
            engine
                .compile(&template)
                .expect("expected template source to compile");
            let context =
                HashMap::from([("name".to_owned(), "Alice".to_owned())]);

            let artifact = engine
                .render(&template, &context)
                .expect("expected template to render");

            assert_eq!(artifact.template(), template.name());
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

            let artifact = engine
                .render(&template, &context)
                .expect("expected markdown to render");

            assert_eq!(artifact.content(), markdown);
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

            let artifact = engine
                .render(&changed, &context)
                .expect("expected compiled source to render");

            assert_eq!(artifact.content(), "Hello Alice");
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
}
