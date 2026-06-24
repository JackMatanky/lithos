//! Template engine error type.
//!
//! Engine errors preserve the underlying `MiniJinja` source error for
//! diagnostics while keeping the Template engine port signatures Traces-shaped.

/// Error returned by template engine operations.
///
/// Each variant stores the template name for user-facing diagnostics and keeps
/// the `MiniJinja` error in the standard source chain.
#[derive(Debug, thiserror::Error)]
pub enum TemplateEngineError {
    /// The engine rejected template source during compile/load.
    #[error("failed to compile template `{name}`")]
    Compile {
        /// Name of the template that failed to compile.
        name: String,
        /// Source error returned by `MiniJinja`.
        #[source]
        source: minijinja::Error,
    },
    /// The engine failed to look up or render a compiled template.
    #[error("failed to render template `{name}`")]
    Render {
        /// Name of the template that failed to render.
        name: String,
        /// Source error returned by `MiniJinja`.
        #[source]
        source: minijinja::Error,
    },
}

// ----------------------------------------------------------- //
//                            Tests                            //
// ----------------------------------------------------------- //

#[cfg(test)]
mod tests {
    use std::error::Error;

    use minijinja::Environment;

    use super::*;

    mod compile {
        use pretty_assertions::assert_eq;

        use super::*;

        #[test]
        fn displays_template_name() {
            let source = compile_source_error();
            let err = TemplateEngineError::Compile {
                name: "daily".to_owned(),
                source,
            };

            assert_eq!(err.to_string(), "failed to compile template `daily`");
        }

        #[test]
        fn preserves_source_error() {
            let source = compile_source_error();
            let err = TemplateEngineError::Compile {
                name: "daily".to_owned(),
                source,
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
                source,
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
                source,
            };

            assert_eq!(err.to_string(), "failed to render template `daily`");
        }

        #[test]
        fn preserves_source_error() {
            let source = render_source_error();
            let err = TemplateEngineError::Render {
                name: "daily".to_owned(),
                source,
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
                source,
            };

            assert!(matches!(
                err,
                TemplateEngineError::Render { ref name, .. } if name == "daily"
            ));
        }
    }

    fn compile_source_error() -> minijinja::Error {
        let mut env = Environment::new();
        env.add_template_owned("invalid".to_owned(), "Hello {{ name".to_owned())
            .expect_err("expected invalid template syntax to fail")
    }

    fn render_source_error() -> minijinja::Error {
        let env = Environment::new();
        env.get_template("missing")
            .expect_err("expected missing template lookup to fail")
    }
}
