use thiserror::Error;

use crate::name::TemplateName;

#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub enum TemplateError {
    #[error("template name error: {0}")]
    Name(#[from] TemplateNameError),
    #[error(transparent)]
    Engine(#[from] TemplateEngineError),
    #[error(transparent)]
    Artifact(#[from] TemplateArtifactError),
    #[error("template not found: {name}")]
    NotFound {
        name: TemplateName,
    },
}

#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum TemplateNameError {
    #[error("template name must not be empty")]
    Empty,
}

#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub enum TemplateEngineError {
    #[error("failed to render template `{name}`")]
    Render {
        name: String,
        #[source]
        source: Box<dyn std::error::Error + Send + Sync>,
    },
}

#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub enum TemplateArtifactError {
    #[error(transparent)]
    Path(#[from] traces_fs::error::WriteTargetError),
    #[error(transparent)]
    Write(#[from] traces_fs::error::WriteError),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn name_empty_displays_message() {
        let err = TemplateNameError::Empty;
        assert!(err.to_string().contains("empty"));
    }

    #[test]
    fn not_found_displays_name() {
        let name = TemplateName::unchecked("missing");
        let err = TemplateError::NotFound {
            name,
        };
        assert!(err.to_string().contains("missing"));
    }

    #[test]
    fn artifact_wraps_write_target_error() {
        let inner = traces_fs::error::WriteTargetError::Absolute(
            std::path::PathBuf::from("/abs/x.md"),
        );
        let err: TemplateArtifactError = inner.into();
        assert!(matches!(err, TemplateArtifactError::Path(_)));
    }
}
