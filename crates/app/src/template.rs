use traces_fs::FsWriter;
use traces_settings::AppConfig;
pub use traces_template::{CreateTemplateInput, CreateTemplateOutcome};
use traces_template::{MiniJinjaEngine, TemplateService};

use crate::error::AppError;

/// Run the template create pipeline.
///
/// # Errors
/// Returns [`AppError::Config`] when config projection fails, or
/// [`AppError::Template`] when rendering or writing fails.
#[inline]
pub fn run_template_create(
    config: &AppConfig,
    input: &CreateTemplateInput,
) -> Result<CreateTemplateOutcome, AppError> {
    let spec = config.to_template_spec().map_err(AppError::Config)?;
    let template_root = spec.to_dir_path().map_err(|e| {
        AppError::Config(traces_settings::error::ConfigError::Ingestion(
            format!("Invalid template directory: {e}").into(),
        ))
    })?;
    let engine = MiniJinjaEngine::new(template_root.as_path());
    let writer = FsWriter::new(spec.root().as_path());
    let service = TemplateService::new(engine, writer);
    service.render(input).map_err(AppError::Template)
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use traces_fs::DirPath;
    use traces_settings::{
        builder::build_from_layers,
        config::{
            aggregate::Version,
            vault::{VaultId, VaultRoot},
        },
    };
    use traces_template::TemplateName;

    use super::*;

    fn config(root: &DirPath) -> traces_settings::AppConfig {
        build_from_layers(
            None,
            None,
            VaultId::new(),
            VaultRoot::from_dir_path(root.clone()),
            Version::initial(),
        )
        .expect("expected test config to build")
    }

    fn input(name: &str, dry_run: bool) -> CreateTemplateInput {
        CreateTemplateInput {
            name: TemplateName::unchecked(name),
            output_path: "notes/out.md".to_owned(),
            context: HashMap::from([("name".to_owned(), "Alice".to_owned())]),
            dry_run,
        }
    }

    #[test]
    fn returns_created_outcome_for_valid_input() {
        let vault = tempfile::tempdir().expect("expected vault tempdir");
        let templates = vault.path().join("templates");
        std::fs::create_dir_all(&templates).expect("expected templates dir");
        std::fs::write(templates.join("greeting.md"), "Hello {{ name }}")
            .expect("expected template write");
        let root = DirPath::try_new(vault.path().to_path_buf())
            .expect("expected vault dir");
        let config = config(&root);

        let result = run_template_create(&config, &input("greeting.md", false));

        assert!(
            matches!(result, Ok(CreateTemplateOutcome::Created { .. })),
            "expected created outcome, got: {result:?}"
        );
    }

    #[test]
    fn writes_rendered_content_to_vault_root() {
        let vault = tempfile::tempdir().expect("expected vault tempdir");
        let templates = vault.path().join("templates");
        std::fs::create_dir_all(&templates).expect("expected templates dir");
        std::fs::write(templates.join("greeting.md"), "Hello {{ name }}")
            .expect("expected template write");
        let root = DirPath::try_new(vault.path().to_path_buf())
            .expect("expected vault dir");
        let config = config(&root);

        let result = run_template_create(&config, &input("greeting.md", false));
        assert!(result.is_ok(), "expected create to succeed: {result:?}");

        let rendered =
            std::fs::read_to_string(vault.path().join("notes/out.md"))
                .expect("expected rendered file");
        assert_eq!(rendered, "Hello Alice");
    }

    #[test]
    fn returns_template_error_when_template_not_found() {
        let vault = tempfile::tempdir().expect("expected vault tempdir");
        let templates = vault.path().join("templates");
        std::fs::create_dir_all(&templates).expect("expected templates dir");
        let root = DirPath::try_new(vault.path().to_path_buf())
            .expect("expected vault dir");
        let config = config(&root);

        let result =
            run_template_create(&config, &input("nonexistent.md", true));

        assert!(matches!(result, Err(AppError::Template(_))));
    }
}
