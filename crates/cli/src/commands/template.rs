//! CLI handler for the `template` command.

use std::{collections::HashMap, io::Write, path::Path};

use traces_app::{
    bootstrap::Bootstrapper,
    error::AppError,
    template::{
        CreateTemplateInput, CreateTemplateOutcome,
        run_template_create as app_run_template_create,
    },
};
use traces_fs::DirPath;
use traces_settings::{DiscoveryFlags, port::DiscoveryPort};
use traces_template::{TemplateArtifactError, TemplateError, TemplateName};

use crate::{
    cli::{OutputFormat, TemplateArgs},
    error::{CliError, TemplateCommandError},
};

/// Executes the `template` subcommand.
///
/// # Errors
///
/// Returns [`CliError`] when bootstrap/config loading fails, variable parsing
/// fails, template creation fails, or writing command output fails.
#[expect(
    clippy::too_many_arguments,
    reason = "handler signature matches the CLI dispatch protocol: \
              bootstrapper, discovery flags, anchor, output format, \
              verbosity, stdout, stderr"
)]
pub(crate) fn run_template<D: DiscoveryPort>(
    bootstrapper: &Bootstrapper<D>,
    flags: Option<DiscoveryFlags>,
    anchor: &Path,
    args: TemplateArgs,
    _format: OutputFormat,
    _verbose: u8,
    out: &mut impl Write,
    _err: &mut impl Write,
) -> Result<(), CliError> {
    let context = parse_vars(args.vars)?;
    let bootstrap = bootstrapper.run(
        flags.clone(),
        None,
        anchor,
        traces_settings::InMemoryRepository::new(),
    )?;
    let discovery = bootstrapper.run_discovery_only(flags, None, anchor)?;
    let cache_dir =
        DirPath::try_from(discovery.cache_root().path().to_path_buf())
            .map_err(|e| CliError::InvalidPath(e.to_string()))?;
    let spec = bootstrap
        .config
        .to_template_spec()
        .map_err(|e| CliError::Bootstrap(AppError::Config(e)))?;
    let template_root = spec.to_dir_path().map_err(|e| {
        TemplateCommandError::InvalidOutputPath {
            reason: e.to_string(),
        }
    })?;
    let template_input = normalize_template_input(&args.input);
    let template_path =
        template_root.as_path().join(format!("{template_input}.md"));
    let name = TemplateName::try_new(&template_path, template_root.as_path())
        .map_err(|e| TemplateCommandError::InvalidOutputPath {
        reason: e.to_string(),
    })?;
    let output_path =
        args.output.unwrap_or_else(|| format!("{template_input}.md"));

    let input = CreateTemplateInput {
        name,
        output_path,
        context,
        dry_run: args.dry_run,
    };
    let outcome =
        app_run_template_create(&bootstrap.config, &cache_dir, &input)
            .map_err(map_template_error)?;

    match outcome {
        CreateTemplateOutcome::Preview {
            rendered,
            ..
        } => writeln!(out, "{}", rendered.as_str())
            .map_err(crate::output::stdout_err)?,
        CreateTemplateOutcome::Created {
            output_path: created_path,
            ..
        } => writeln!(out, "{}", created_path.as_path().display())
            .map_err(crate::output::stdout_err)?,
        _ => {}
    }

    Ok(())
}

fn parse_vars(
    vars: impl IntoIterator<Item = String>,
) -> Result<HashMap<String, String>, TemplateCommandError> {
    let mut parsed = HashMap::new();
    for var in vars {
        let Some((key, value)) = var.split_once('=') else {
            return Err(TemplateCommandError::InvalidVarFormat {
                value: var,
            });
        };
        parsed.insert(key.to_owned(), value.to_owned());
    }
    Ok(parsed)
}

fn normalize_template_input(input: &str) -> &str {
    input.strip_suffix(".md").unwrap_or(input)
}

fn map_template_error(err: AppError) -> CliError {
    match err {
        AppError::Template(TemplateError::NotFound {
            name,
        }) => TemplateCommandError::TemplateNotFound {
            name: name.to_string(),
        }
        .into(),
        AppError::Template(TemplateError::Engine(e)) => {
            TemplateCommandError::RenderFailed {
                detail: e.to_string(),
            }
            .into()
        }
        AppError::Template(TemplateError::Path(e)) => {
            TemplateCommandError::InvalidOutputPath {
                reason: e.to_string(),
            }
            .into()
        }
        AppError::Template(TemplateError::Artifact(
            TemplateArtifactError::Path(e),
        )) => TemplateCommandError::InvalidOutputPath {
            reason: e.to_string(),
        }
        .into(),
        AppError::Template(TemplateError::Artifact(
            TemplateArtifactError::Write(
                traces_fs::error::WriteError::AlreadyExists {
                    path,
                },
            ),
        )) => TemplateCommandError::DestinationExists {
            path: path.display().to_string(),
        }
        .into(),
        AppError::Template(TemplateError::Artifact(
            TemplateArtifactError::Write(traces_fs::error::WriteError::Io {
                path,
                source,
            }),
        )) => TemplateCommandError::WriteFailed {
            detail: format!("{}: {source}", path.display()),
        }
        .into(),
        AppError::Template(TemplateError::Name(e)) => {
            TemplateCommandError::InvalidOutputPath {
                reason: format!("Template name is invalid: {e}"),
            }
            .into()
        }
        AppError::Template(TemplateError::Body(e)) => {
            TemplateCommandError::RenderFailed {
                detail: format!("Template body is invalid: {e}"),
            }
            .into()
        }
        AppError::Template(TemplateError::Read(e)) => {
            TemplateCommandError::RenderFailed {
                detail: format!("Could not read template: {e}"),
            }
            .into()
        }
        AppError::Template(TemplateError::Repository(e)) => {
            TemplateCommandError::RenderFailed {
                detail: format!(
                    "Template repository failed: {e}. Run 'traces index' to \
                     rebuild template metadata."
                ),
            }
            .into()
        }
        AppError::Template(TemplateError::Scan(e)) => {
            TemplateCommandError::RenderFailed {
                detail: format!(
                    "Could not scan template directory: {e}. Check template \
                     directory permissions and retry."
                ),
            }
            .into()
        }
        other => CliError::Bootstrap(other),
    }
}

#[cfg(test)]
mod tests {
    use std::{collections::HashMap, fs, path::PathBuf};

    use traces_app::{bootstrap::Bootstrapper, error::AppError};
    use traces_fs::error::{WriteError, WriteTargetError};
    use traces_settings::{DiscoveryFlags, service::DiscoveryService};
    use traces_template::{
        TemplateArtifactError, TemplateEngineError, TemplateError,
        TemplateName, TemplateNameError,
    };

    use super::{map_template_error, parse_vars, run_template};
    use crate::{
        cli::{OutputFormat, TemplateArgs},
        error::{CliError, TemplateCommandError},
    };

    fn template_name(name: &str) -> TemplateName {
        TemplateName::try_new(
            std::path::Path::new(&format!("templates/{name}.md")),
            std::path::Path::new("templates"),
        )
        .expect("template name")
    }

    fn make_vault()
    -> (tempfile::TempDir, Bootstrapper<DiscoveryService>, DiscoveryFlags) {
        let dir = tempfile::tempdir().expect("vault dir");
        let config_path = dir.path().join("traces.toml");
        fs::write(&config_path, "[template]\ndirectory = \"templates\"")
            .expect("write traces.toml");
        fs::create_dir_all(dir.path().join("templates"))
            .expect("create templates");
        fs::write(dir.path().join("templates/greeting.md"), "Hello {{ name }}")
            .expect("write template");
        fs::create_dir_all(dir.path().join(".traces/cache"))
            .expect("create cache");

        let flags = DiscoveryFlags::new(
            Some(config_path.as_path()),
            Some(dir.path()),
            true,
        )
        .expect("valid flags");
        let bootstrapper = Bootstrapper::with_global_directories(vec![])
            .expect("bootstrapper");
        (dir, bootstrapper, flags)
    }

    fn args(dry_run: bool) -> TemplateArgs {
        args_with_input("greeting", dry_run)
    }

    fn args_with_input(input: &str, dry_run: bool) -> TemplateArgs {
        args_with_output(input, (!dry_run).then_some("notes/out.md"), dry_run)
    }

    fn args_with_output(
        input: &str,
        output: Option<&str>,
        dry_run: bool,
    ) -> TemplateArgs {
        TemplateArgs {
            input: input.to_owned(),
            output: output.map(str::to_owned),
            dry_run,
            vars: vec!["name=Alice".to_owned()],
        }
    }

    mod parse_vars {
        use super::*;

        #[test]
        fn parses_single_var() {
            let parsed = parse_vars(["name=Alice".to_owned()]).unwrap();
            assert_eq!(
                parsed,
                HashMap::from([("name".to_owned(), "Alice".to_owned())])
            );
        }

        #[test]
        fn parses_multiple_vars() {
            let parsed =
                parse_vars(["name=Alice".to_owned(), "mode=cli".to_owned()])
                    .unwrap();
            assert_eq!(
                parsed,
                HashMap::from([
                    ("name".to_owned(), "Alice".to_owned()),
                    ("mode".to_owned(), "cli".to_owned()),
                ])
            );
        }

        #[test]
        fn parses_var_with_equals_in_value() {
            let parsed = parse_vars(["query=a=b".to_owned()]).unwrap();
            assert_eq!(parsed.get("query"), Some(&"a=b".to_owned()));
        }

        #[test]
        fn rejects_var_without_equals() {
            let err = parse_vars(["missing".to_owned()]).unwrap_err();
            assert!(
                matches!(err, TemplateCommandError::InvalidVarFormat { value } if value == "missing")
            );
        }
    }

    mod run_template_handler {
        use super::*;

        #[test]
        fn renders_template_and_prints_path() {
            let (dir, bootstrapper, flags) = make_vault();
            let mut out = Vec::new();
            let mut err = Vec::new();

            let result = run_template(
                &bootstrapper,
                Some(flags),
                dir.path(),
                args(false),
                OutputFormat::Human,
                0,
                &mut out,
                &mut err,
            );

            assert!(result.is_ok(), "run_template failed: {result:?}");
            assert_eq!(String::from_utf8(out).unwrap(), "notes/out.md\n");
            assert_eq!(
                fs::read_to_string(dir.path().join("notes/out.md")).unwrap(),
                "Hello Alice"
            );
        }

        #[test]
        fn renders_template_when_input_has_md_suffix() {
            let (dir, bootstrapper, flags) = make_vault();
            let mut out = Vec::new();
            let mut err = Vec::new();

            let result = run_template(
                &bootstrapper,
                Some(flags),
                dir.path(),
                args_with_input("greeting.md", false),
                OutputFormat::Human,
                0,
                &mut out,
                &mut err,
            );

            assert!(result.is_ok(), "run_template failed: {result:?}");
            assert_eq!(String::from_utf8(out).unwrap(), "notes/out.md\n");
            assert_eq!(
                fs::read_to_string(dir.path().join("notes/out.md")).unwrap(),
                "Hello Alice"
            );
        }

        #[test]
        fn dry_run_prints_content() {
            let (dir, bootstrapper, flags) = make_vault();
            let mut out = Vec::new();
            let mut err = Vec::new();

            let result = run_template(
                &bootstrapper,
                Some(flags),
                dir.path(),
                args(true),
                OutputFormat::Human,
                0,
                &mut out,
                &mut err,
            );

            assert!(result.is_ok(), "run_template failed: {result:?}");
            assert_eq!(String::from_utf8(out).unwrap(), "Hello Alice\n");
            assert!(!dir.path().join("notes/out.md").exists());
        }

        #[test]
        fn missing_template_returns_user_facing_error() {
            let (dir, bootstrapper, flags) = make_vault();
            let mut out = Vec::new();
            let mut err = Vec::new();

            let result = run_template(
                &bootstrapper,
                Some(flags),
                dir.path(),
                args_with_input("missing", false),
                OutputFormat::Human,
                0,
                &mut out,
                &mut err,
            );

            let message = result.unwrap_err().to_string();
            assert!(message.starts_with("Template 'missing' not found."));
            assert!(message.contains("Run 'traces index'"));
        }

        #[test]
        fn invalid_output_path_returns_user_facing_error() {
            let (dir, bootstrapper, flags) = make_vault();
            let mut out = Vec::new();
            let mut err = Vec::new();

            let result = run_template(
                &bootstrapper,
                Some(flags),
                dir.path(),
                args_with_output("greeting", Some("../out.md"), false),
                OutputFormat::Human,
                0,
                &mut out,
                &mut err,
            );

            let message = result.unwrap_err().to_string();
            assert!(message.starts_with("Output path is invalid:"));
            assert!(message.contains("../out.md"));
        }

        #[test]
        fn destination_exists_returns_user_facing_error() {
            let (dir, bootstrapper, flags) = make_vault();
            fs::create_dir_all(dir.path().join("notes"))
                .expect("create notes dir");
            fs::write(dir.path().join("notes/out.md"), "existing")
                .expect("write existing output");
            let mut out = Vec::new();
            let mut err = Vec::new();

            let result = run_template(
                &bootstrapper,
                Some(flags),
                dir.path(),
                args(false),
                OutputFormat::Human,
                0,
                &mut out,
                &mut err,
            );

            let cli_err = result.expect_err("existing destination should fail");
            assert!(
                cli_err.to_string().contains("Output file already exists:"),
                "got: {cli_err}"
            );
            assert!(
                cli_err.to_string().contains("Choose a different output path."),
                "got: {cli_err}"
            );
        }
    }

    mod error_mapping {
        use super::*;

        #[expect(
            clippy::panic,
            reason = "test helper panic signals unexpected error variant"
        )]
        fn template_command_error(err: AppError) -> TemplateCommandError {
            match map_template_error(err) {
                CliError::TemplateCommand(err) => err,
                other => {
                    panic!("expected TemplateCommand error, got {other:?}")
                }
            }
        }

        #[test]
        fn maps_not_found() {
            let err = AppError::Template(TemplateError::NotFound {
                name: template_name("missing"),
            });
            assert!(
                matches!(template_command_error(err), TemplateCommandError::TemplateNotFound { name } if name == "missing")
            );
        }

        #[test]
        fn maps_engine_error() {
            let source = std::io::Error::other("bad render");
            let err = AppError::Template(TemplateError::Engine(
                TemplateEngineError::Render {
                    name: "daily".to_owned(),
                    source: Box::new(source),
                },
            ));
            assert!(
                matches!(template_command_error(err), TemplateCommandError::RenderFailed { detail } if detail.contains("daily"))
            );
        }

        #[test]
        fn maps_path_error() {
            let err = AppError::Template(TemplateError::Artifact(
                TemplateArtifactError::Path(WriteTargetError::Traversal(
                    PathBuf::from("../out.md"),
                )),
            ));
            assert!(matches!(
                template_command_error(err),
                TemplateCommandError::InvalidOutputPath { .. }
            ));
        }

        #[test]
        fn maps_already_exists() {
            let err = AppError::Template(TemplateError::Artifact(
                TemplateArtifactError::Write(WriteError::AlreadyExists {
                    path: PathBuf::from("notes/out.md"),
                }),
            ));
            assert!(
                matches!(template_command_error(err), TemplateCommandError::DestinationExists { path } if path == "notes/out.md")
            );
        }

        #[test]
        fn maps_write_io() {
            let err = AppError::Template(TemplateError::Artifact(
                TemplateArtifactError::Write(WriteError::Io {
                    path: PathBuf::from("notes/out.md"),
                    source: std::io::Error::other("disk full"),
                }),
            ));
            assert!(
                matches!(template_command_error(err), TemplateCommandError::WriteFailed { detail } if detail == "notes/out.md: disk full")
            );
        }

        #[test]
        fn maps_name_error_without_raw_template_display() {
            let err = AppError::Template(TemplateError::Name(
                TemplateNameError::Derivation,
            ));

            assert!(
                matches!(template_command_error(err), TemplateCommandError::InvalidOutputPath { reason } if reason.contains("Template name is invalid"))
            );
        }
    }
}
