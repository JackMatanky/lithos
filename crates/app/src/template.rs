//! Template pipeline wiring and composition.

use std::sync::Arc;

use traces_db::Store;
use traces_fs::{DirPath, FsWriter};
use traces_settings::Config;
pub use traces_template::{
    CreateTemplateInput, CreateTemplateOutcome, storage::TEMPLATE_DB_FILENAME,
};
use traces_template::{
    MiniJinjaEngine, TemplateError, TemplateRepositoryError, TemplateService,
    storage::RedbRepository,
};

use crate::error::AppError;

/// Run the template create pipeline.
///
/// # Errors
/// Returns [`AppError::Config`] when config projection fails, or
/// [`AppError::Template`] when storage, rendering, or writing fails.
#[inline]
pub fn run_template_create(
    config: &Config,
    cache_dir: &DirPath,
    input: &CreateTemplateInput,
) -> Result<CreateTemplateOutcome, AppError> {
    let spec = config.to_template_spec()?;
    let db_path = cache_dir.as_path().join(TEMPLATE_DB_FILENAME);
    let store = Store::open(&db_path).map_err(|e| {
        AppError::Template(TemplateError::Repository(
            TemplateRepositoryError::Storage(e),
        ))
    })?;
    let repo = RedbRepository::new(Arc::new(store));
    let writer = FsWriter::new(spec.root().as_path());
    let engine = MiniJinjaEngine::configured();
    let mut service = TemplateService::new(repo, writer, engine, spec);

    service.create(input).map_err(AppError::Template)
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
    use traces_template::{
        CreateTemplateInput, CreateTemplateOutcome, TemplateName,
    };

    use super::*;
    use crate::error::AppError;

    fn config(root: &DirPath) -> traces_settings::Config {
        build_from_layers(
            None,
            None,
            VaultId::new(),
            VaultRoot::from_dir_path(root.clone()),
            Version::initial(),
        )
        .expect("expected test config to build")
    }

    fn input(name: TemplateName, dry_run: bool) -> CreateTemplateInput {
        CreateTemplateInput {
            name,
            output_path: "notes/out.md".to_owned(),
            context: HashMap::from([("name".to_owned(), "Alice".to_owned())]),
            dry_run,
        }
    }

    fn template_name(root: &std::path::Path) -> TemplateName {
        TemplateName::try_new(
            &root.join("templates").join("greeting.md"),
            &root.join("templates"),
        )
        .expect("expected template name")
    }

    mod run_template_create {
        use pretty_assertions::assert_eq;

        use super::*;

        #[test]
        fn returns_created_outcome_for_valid_input() {
            let vault = tempfile::tempdir().expect("expected vault tempdir");
            let cache = tempfile::tempdir().expect("expected cache tempdir");
            let templates = vault.path().join("templates");
            std::fs::create_dir_all(&templates)
                .expect("expected templates dir");
            std::fs::write(templates.join("greeting.md"), "Hello {{ name }}")
                .expect("expected template write");
            let root = DirPath::try_new(vault.path().to_path_buf())
                .expect("expected vault dir");
            let cache_dir = DirPath::try_new(cache.path().to_path_buf())
                .expect("expected cache dir");
            let config = config(&root);

            let result = crate::template::run_template_create(
                &config,
                &cache_dir,
                &input(template_name(vault.path()), false),
            );

            assert!(
                matches!(result, Ok(CreateTemplateOutcome::Created { .. })),
                "expected created outcome, got: {result:?}"
            );
        }

        #[test]
        fn writes_rendered_content_to_vault_root_for_valid_input() {
            let vault = tempfile::tempdir().expect("expected vault tempdir");
            let cache = tempfile::tempdir().expect("expected cache tempdir");
            let templates = vault.path().join("templates");
            std::fs::create_dir_all(&templates)
                .expect("expected templates dir");
            std::fs::write(templates.join("greeting.md"), "Hello {{ name }}")
                .expect("expected template write");
            let root = DirPath::try_new(vault.path().to_path_buf())
                .expect("expected vault dir");
            let cache_dir = DirPath::try_new(cache.path().to_path_buf())
                .expect("expected cache dir");
            let config = config(&root);

            let result = crate::template::run_template_create(
                &config,
                &cache_dir,
                &input(template_name(vault.path()), false),
            );
            assert!(result.is_ok(), "expected create to succeed: {result:?}");

            let rendered =
                std::fs::read_to_string(vault.path().join("notes/out.md"))
                    .expect("expected rendered file");
            assert_eq!(rendered, "Hello Alice");
        }

        #[test]
        fn returns_template_error_when_store_open_fails() {
            let vault = tempfile::tempdir().expect("expected vault tempdir");
            let cache = tempfile::tempdir().expect("expected cache tempdir");
            let root = DirPath::try_new(vault.path().to_path_buf())
                .expect("expected vault dir");
            std::fs::create_dir(root.as_path().join("templates"))
                .expect("expected templates dir");
            let cache_dir = DirPath::try_new(cache.path().to_path_buf())
                .expect("expected cache dir");
            std::fs::create_dir(cache_dir.as_path().join(TEMPLATE_DB_FILENAME))
                .expect("expected db path directory");
            let config = config(&root);

            let err = crate::template::run_template_create(
                &config,
                &cache_dir,
                &input(template_name(vault.path()), true),
            )
            .expect_err("expected store open to fail");

            assert!(matches!(err, AppError::Template(_)));
        }

        #[test]
        fn returns_template_error_when_template_directory_is_missing() {
            let cache = tempfile::tempdir().expect("expected cache tempdir");
            let cache_dir = DirPath::try_new(cache.path().to_path_buf())
                .expect("expected cache dir");
            let config = build_from_layers(
                None,
                None,
                VaultId::new(),
                VaultRoot::default(),
                Version::initial(),
            )
            .expect("expected root config to build");

            let err = crate::template::run_template_create(
                &config,
                &cache_dir,
                &input(template_name(std::path::Path::new("/")), true),
            )
            .expect_err("expected invalid config to fail");

            assert!(matches!(err, AppError::Template(_)));
        }
    }
}
