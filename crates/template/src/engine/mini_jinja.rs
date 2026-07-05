use std::{collections::HashMap, path::Path};

use minijinja::{AutoEscape, Environment, UndefinedBehavior, path_loader};

use super::{RenderedTemplate, TemplateEngine};
use crate::error::TemplateEngineError;

/// Minijinja-backed [`TemplateEngine`] implementation.
///
/// Loads templates from a filesystem root using minijinja's [`path_loader`].
/// Configured with strict undefined-behaviour and no auto-escape for
/// plain-text output (markdown/code generation).
pub struct MiniJinjaEngine {
    env: Environment<'static>,
}

impl MiniJinjaEngine {
    /// Creates a new engine that loads templates from `root`.
    ///
    /// The root directory is scanned at render time — no upfront compilation.
    #[inline]
    #[must_use]
    pub fn new(root: &Path) -> Self {
        let mut env = Environment::new();
        env.set_loader(path_loader(root));
        env.set_undefined_behavior(UndefinedBehavior::Strict);
        env.set_auto_escape_callback(|_| AutoEscape::None);
        Self {
            env,
        }
    }
}

impl TemplateEngine for MiniJinjaEngine {
    #[inline]
    fn render(
        &self,
        name: &str,
        variables: &HashMap<String, String>,
    ) -> Result<RenderedTemplate, TemplateEngineError> {
        let tmpl = self.env.get_template(name).map_err(|source| {
            TemplateEngineError::Render {
                name: name.to_owned(),
                source: Box::new(source),
            }
        })?;
        let result = tmpl.render(variables).map_err(|source| {
            TemplateEngineError::Render {
                name: name.to_owned(),
                source: Box::new(source),
            }
        })?;
        Ok(RenderedTemplate::new(result))
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use super::*;

    fn engine(root: &Path) -> MiniJinjaEngine {
        MiniJinjaEngine::new(root)
    }

    fn write_template(dir: &Path, name: &str, content: &str) {
        std::fs::create_dir_all(dir).unwrap();
        std::fs::write(dir.join(name), content).unwrap();
    }

    mod render {
        use pretty_assertions::assert_eq;

        use super::*;

        #[test]
        fn renders_valid_template() {
            let dir = tempfile::tempdir().unwrap();
            write_template(dir.path(), "greeting.md", "Hello {{ name }}");
            let engine = engine(dir.path());

            let result = engine.render(
                "greeting.md",
                &HashMap::from([("name".to_owned(), "Alice".to_owned())]),
            );

            assert!(result.is_ok());
            assert_eq!(result.unwrap().as_str(), "Hello Alice");
        }

        #[test]
        fn rejects_undefined_variable() {
            let dir = tempfile::tempdir().unwrap();
            write_template(dir.path(), "test.md", "{{ missing }}");
            let engine = engine(dir.path());

            let result = engine.render("test.md", &HashMap::new());

            assert!(result.is_err());
        }

        #[test]
        fn preserves_markdown() {
            let dir = tempfile::tempdir().unwrap();
            write_template(dir.path(), "test.md", "# Title *bold* [link]");
            let engine = engine(dir.path());

            let result = engine.render("test.md", &HashMap::new());

            assert!(result.is_ok());
            assert_eq!(result.unwrap().as_str(), "# Title *bold* [link]");
        }

        #[test]
        fn error_when_template_not_found() {
            let dir = tempfile::tempdir().unwrap();
            let engine = MiniJinjaEngine::new(dir.path());

            let result = engine.render("nonexistent.md", &HashMap::new());

            assert!(result.is_err());
        }

        /// Verifies that rendering `Hello {{ name }}` with an empty variable
        /// value succeeds and produces `Hello ` (with trailing space).
        #[test]
        fn renders_empty_variable_value() {
            let dir = tempfile::tempdir().unwrap();
            write_template(dir.path(), "greeting.md", "Hello {{ name }}");
            let engine = engine(dir.path());

            let result = engine.render(
                "greeting.md",
                &HashMap::from([("name".to_owned(), String::new())]),
            );

            assert!(result.is_ok());
            assert_eq!(result.unwrap().as_str(), "Hello ");
        }
    }
}
