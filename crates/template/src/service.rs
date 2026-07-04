use std::collections::HashMap;

use traces_fs::FileWriter;

use crate::{
    artifact::{commit, resolve_target},
    engine::TemplateEngine,
    error::TemplateError,
    name::TemplateName,
};

#[derive(Debug, Clone)]
pub struct CreateTemplateInput {
    pub name: TemplateName,
    pub output_path: String,
    pub context: HashMap<String, String>,
    pub dry_run: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum CreateTemplateOutcome {
    Preview {
        output_path: String,
        rendered: String,
    },
    Created {
        output_path: String,
        bytes_written: u64,
    },
}

pub struct TemplateService<E, W> {
    engine: E,
    writer: W,
}

impl<E, W> TemplateService<E, W>
where
    E: TemplateEngine,
    W: FileWriter,
{
    #[inline]
    #[must_use]
    pub fn new(engine: E, writer: W) -> Self {
        Self {
            engine,
            writer,
        }
    }

    #[inline]
    pub fn render(
        &self,
        input: &CreateTemplateInput,
    ) -> Result<CreateTemplateOutcome, TemplateError> {
        let rendered = self
            .engine
            .render(input.name.as_ref(), &input.context)
            .map_err(TemplateError::Engine)?;

        let target = resolve_target(&input.output_path)?;
        let output_path = target.as_path().display().to_string();

        if input.dry_run {
            return Ok(CreateTemplateOutcome::Preview {
                output_path,
                rendered: rendered.into_inner(),
            });
        }

        let content = rendered.into_inner();
        let bytes_written = content.len() as u64;
        commit(target, &content, &self.writer)?;

        Ok(CreateTemplateOutcome::Created {
            output_path,
            bytes_written,
        })
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use tempfile::TempDir;

    use super::*;
    use crate::engine::mini_jinja::MiniJinjaEngine;

    fn setup()
    -> (TempDir, TemplateService<MiniJinjaEngine, traces_fs::FsWriter>) {
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
    fn render_creates_output_file() {
        let (dir, service) = setup();
        let input = CreateTemplateInput {
            name: TemplateName::unchecked("greeting.md"),
            output_path: "notes/out.md".to_owned(),
            context: HashMap::from([("name".to_owned(), "Alice".to_owned())]),
            dry_run: false,
        };

        let result = service.render(&input).unwrap();

        assert!(matches!(result, CreateTemplateOutcome::Created { .. }));
        let content =
            std::fs::read_to_string(dir.path().join("notes/out.md")).unwrap();
        assert_eq!(content, "Hello Alice");
    }

    #[test]
    fn dry_run_returns_preview() {
        let (_dir, service) = setup();
        let input = CreateTemplateInput {
            name: TemplateName::unchecked("greeting.md"),
            output_path: "notes/out.md".to_owned(),
            context: HashMap::from([("name".to_owned(), "Alice".to_owned())]),
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

        assert!(result.is_err());
    }
}
