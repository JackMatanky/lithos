use std::collections::HashMap;

use traces_fs::{FileWriter, WriteTarget};
use traces_settings::config::template::TemplateConfigSpec;

use crate::{
    engine::{TemplateEngine, mini_jinja::MiniJinjaEngine},
    error::{TemplateArtifactError, TemplateError},
    name::TemplateName,
};

/// Input to [`TemplateService::render`].
#[derive(Debug, Clone)]
pub struct CreateTemplateInput {
    /// Name of the template to render.
    pub name: TemplateName,
    /// Relative output path for the rendered artifact.
    pub output_path: String,
    /// Template variables (name → value).
    pub context: HashMap<String, String>,
    /// If `true`, returns a preview instead of writing to disk.
    pub dry_run: bool,
}

/// Outcome of [`TemplateService::render`].
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum CreateTemplateOutcome {
    /// Dry-run preview — rendered content shown, nothing written.
    Preview {
        /// The target output path.
        output_path: String,
        /// The rendered template content.
        rendered: String,
    },
    /// Rendered content was written to disk.
    Created {
        /// The output path written to.
        output_path: String,
        /// Number of bytes written.
        bytes_written: u64,
    },
}

/// Two-phase template service: render then write.
///
/// Generic over the engine (`E`) and writer (`W`). Use
/// [`TemplateService::from_spec`] for the convenience constructor backed by
/// [`MiniJinjaEngine`] and [`traces_fs::FsWriter`].
pub struct TemplateService<E, W> {
    engine: E,
    writer: W,
}

impl<E, W> TemplateService<E, W>
where
    E: TemplateEngine,
    W: FileWriter,
{
    /// Creates a new service from an engine and writer.
    #[inline]
    #[must_use]
    pub fn new(engine: E, writer: W) -> Self {
        Self {
            engine,
            writer,
        }
    }
}

impl TemplateService<MiniJinjaEngine, traces_fs::FsWriter> {
    /// Creates a service from a template configuration spec.
    ///
    /// # Errors
    ///
    /// Returns [`TemplateError::Config`] if the spec's directory path is
    /// invalid.
    #[inline]
    pub fn from_spec(spec: &TemplateConfigSpec) -> Result<Self, TemplateError> {
        let template_root = spec
            .to_dir_path()
            .map_err(|e| TemplateError::Config(format!("{e}")))?;
        let engine = MiniJinjaEngine::new(template_root.as_path());
        let writer = traces_fs::FsWriter::new(spec.root().as_path());
        Ok(Self {
            engine,
            writer,
        })
    }
}

impl<E, W> TemplateService<E, W>
where
    E: TemplateEngine,
    W: FileWriter,
{
    /// Renders a template and writes the output or returns a preview.
    ///
    /// When [`CreateTemplateInput::dry_run`] is `true`, returns
    /// [`CreateTemplateOutcome::Preview`] without writing. Otherwise writes
    /// the rendered content and returns [`CreateTemplateOutcome::Created`].
    ///
    /// # Errors
    ///
    /// Returns [`TemplateError::Engine`] when the engine fails to load or
    /// render the template, or [`TemplateError::Artifact`] when the write
    /// pipeline fails.
    #[inline]
    #[allow(clippy::as_conversions, reason = "len fits in u64 on all targets")]
    pub fn render(
        &self,
        input: &CreateTemplateInput,
    ) -> Result<CreateTemplateOutcome, TemplateError> {
        let rendered = self
            .engine
            .render(input.name.as_ref(), &input.context)
            .map_err(TemplateError::Engine)?;

        let target = resolve_target(&input.output_path)?;

        if input.dry_run {
            return Ok(CreateTemplateOutcome::Preview {
                output_path: input.output_path.clone(),
                rendered: rendered.into_inner(),
            });
        }

        let content = rendered.into_inner();
        let bytes_written = content.len() as u64;
        commit(&target, &content, &self.writer)?;

        Ok(CreateTemplateOutcome::Created {
            output_path: input.output_path.clone(),
            bytes_written,
        })
    }
}

#[inline]
fn resolve_target(raw: &str) -> Result<WriteTarget, TemplateArtifactError> {
    WriteTarget::try_new(raw).map_err(TemplateArtifactError::Path)
}

#[inline]
fn commit(
    target: &WriteTarget,
    content: &str,
    writer: &impl FileWriter,
) -> Result<(), TemplateArtifactError> {
    writer.create_new(target, content.as_bytes())?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    mod render {
        use std::collections::HashMap;

        use tempfile::TempDir;

        use super::*;

        fn setup()
        -> (TempDir, TemplateService<MiniJinjaEngine, traces_fs::FsWriter>)
        {
            let dir = TempDir::new().unwrap();
            std::fs::create_dir_all(dir.path().join("templates")).unwrap();
            std::fs::write(
                dir.path().join("templates/greeting.md"),
                "Hello {{ name }}",
            )
            .unwrap();
            let engine =
                MiniJinjaEngine::new(dir.path().join("templates").as_path());
            let writer = traces_fs::FsWriter::new(dir.path());
            let service = TemplateService::new(engine, writer);
            (dir, service)
        }

        #[test]
        fn creates_output_file() {
            let (dir, service) = setup();
            let input = CreateTemplateInput {
                name: TemplateName::unchecked("greeting.md"),
                output_path: "notes/out.md".to_owned(),
                context: HashMap::from([(
                    "name".to_owned(),
                    "Alice".to_owned(),
                )]),
                dry_run: false,
            };

            let result = service.render(&input).unwrap();

            assert!(matches!(result, CreateTemplateOutcome::Created { .. }));
            let content =
                std::fs::read_to_string(dir.path().join("notes/out.md"))
                    .unwrap();
            assert_eq!(content, "Hello Alice");
        }

        #[test]
        fn returns_preview_in_dry_run() {
            let (_dir, service) = setup();
            let input = CreateTemplateInput {
                name: TemplateName::unchecked("greeting.md"),
                output_path: "notes/out.md".to_owned(),
                context: HashMap::from([(
                    "name".to_owned(),
                    "Alice".to_owned(),
                )]),
                dry_run: true,
            };

            let result = service.render(&input).unwrap();

            assert!(matches!(result, CreateTemplateOutcome::Preview { .. }));
        }

        #[test]
        fn errors_when_template_not_found() {
            let (_dir, service) = setup();
            let input = CreateTemplateInput {
                name: TemplateName::unchecked("missing.md"),
                output_path: "notes/out.md".to_owned(),
                context: HashMap::new(),
                dry_run: false,
            };

            let result = service.render(&input);

            assert!(
                result.is_err(),
                "expected error for missing template, got: {result:?}"
            );
        }
    }

    mod resolve_target {
        use std::path::PathBuf;

        use pretty_assertions::assert_eq;

        use super::*;

        #[test]
        fn resolves_valid_relative_path() {
            let target = resolve_target("notes/out.md").unwrap();
            assert_eq!(target.as_path(), PathBuf::from("notes/out.md"));
        }

        #[test]
        fn rejects_absolute_path() {
            let err = resolve_target("/abs/out.md").unwrap_err();
            assert!(matches!(err, TemplateArtifactError::Path(_)));
        }

        #[test]
        fn rejects_traversal_path() {
            let err = resolve_target("../escape.md").unwrap_err();
            assert!(matches!(err, TemplateArtifactError::Path(_)));
        }
    }

    mod commit {
        use pretty_assertions::assert_eq;

        use super::*;

        #[test]
        fn writes_content_through_writer() {
            let dir = tempfile::tempdir().unwrap();
            let writer = traces_fs::FsWriter::new(dir.path());
            let target = resolve_target("out.md").unwrap();

            commit(&target, "hello", &writer).unwrap();

            let content =
                std::fs::read_to_string(dir.path().join("out.md")).unwrap();
            assert_eq!(content, "hello");
        }

        #[test]
        fn rejects_commit_to_existing_file() {
            let dir = tempfile::tempdir().unwrap();
            std::fs::write(dir.path().join("out.md"), "existing").unwrap();
            let writer = traces_fs::FsWriter::new(dir.path());
            let target = resolve_target("out.md").unwrap();

            let err = commit(&target, "new", &writer).unwrap_err();

            assert!(matches!(err, TemplateArtifactError::Write(_)));
        }

        #[test]
        fn creates_intermediate_directory() {
            let dir = tempfile::tempdir().unwrap();
            let writer = traces_fs::FsWriter::new(dir.path());
            let target = resolve_target("subdir/out.md").unwrap();

            commit(&target, "content", &writer).unwrap();

            let path = dir.path().join("subdir/out.md");
            assert!(path.exists());
            let content = std::fs::read_to_string(path).unwrap();
            assert_eq!(content, "content");
        }
    }
}
