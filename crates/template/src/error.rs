use crate::name::TemplateName;

/// Top-level template error.
///
/// Wraps all domain-specific errors (name, engine, artifact) into a single
/// enum for service-level callers.
#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub enum TemplateError {
    /// A template name failed validation.
    #[error("template name error: {0}")]
    Name(#[from] TemplateNameError),
    /// The engine failed to render a template.
    #[error(transparent)]
    Engine(#[from] TemplateEngineError),
    /// Writing the rendered artifact to disk failed.
    #[error(transparent)]
    Artifact(#[from] TemplateArtifactError),
    /// The requested template was not found.
    #[error("template not found: {name}")]
    NotFound {
        /// The name that was searched for.
        name: TemplateName,
    },
    /// Template configuration is invalid.
    #[error("template configuration error: {0}")]
    Config(String),
}

/// Template name validation error.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
#[non_exhaustive]
pub enum TemplateNameError {
    /// The provided name was an empty string.
    #[error("template name must not be empty")]
    Empty,
}

/// Template engine rendering error.
#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub enum TemplateEngineError {
    /// Rendering a specific template failed.
    #[error("failed to render template `{name}`")]
    Render {
        /// The template name that caused the failure.
        name: String,
        /// The underlying engine error.
        #[source]
        source: Box<dyn std::error::Error + Send + Sync>,
    },
}

/// Template artifact write error.
///
/// Wraps underlying filesystem errors from target-path resolution and
/// file writing.
#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub enum TemplateArtifactError {
    /// The output path could not be resolved.
    #[error(transparent)]
    Path(#[from] traces_fs::error::WriteTargetError),
    /// Writing the artifact file failed.
    #[error(transparent)]
    Write(#[from] traces_fs::error::WriteError),
}

#[cfg(test)]
mod tests {
    use super::*;

    mod display {
        use super::*;

        #[test]
        fn displays_message_when_empty() {
            let err = TemplateNameError::Empty;
            assert!(err.to_string().contains("empty"));
        }

        #[test]
        fn displays_name_when_not_found() {
            let name = TemplateName::unchecked("missing");
            let err = TemplateError::NotFound {
                name,
            };
            assert!(err.to_string().contains("missing"));
        }

        #[test]
        fn displays_config_message() {
            let err = TemplateError::Config("bad template dir".to_owned());
            let msg = err.to_string();
            assert!(msg.contains("template configuration error"));
            assert!(msg.contains("bad template dir"));
        }
    }

    mod conversions {
        use super::*;

        #[test]
        fn wraps_write_target_error() {
            let inner = traces_fs::error::WriteTargetError::Absolute(
                std::path::PathBuf::from("/abs/x.md"),
            );
            let err: TemplateArtifactError = inner.into();
            assert!(matches!(err, TemplateArtifactError::Path(_)));
        }
    }
}
