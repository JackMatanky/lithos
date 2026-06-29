//! CLI handler for the `template` subcommand.
//!
//! Accepts `--input`, `--output`, `--dry-run`/`-n`, and repeated
//! `--var key=value` flags. `--var` splits on first `=` only. Maps
//! every `TemplateError` variant to a user-facing
//! `TemplateCommandError` message. Normal render prints the created
//! vault-relative path; dry-run prints the rendered content. Output
//! honours `OutputFormat::Human` (default) and `OutputFormat::Json`.

use std::{collections::HashMap, io::Write, path::Path};

use traces_app::{
    bootstrap::Bootstrapper,
    error::AppError,
    template::{
        CreateTemplateInput, CreateTemplateOutcome,
        run_template_create as app_run_template_create,
    },
};
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
    format: OutputFormat,
    _verbose: u8,
    out: &mut impl Write,
    _err: &mut impl Write,
) -> Result<(), CliError> {
    let context = parse_vars(args.vars)?;
    let bootstrap = bootstrapper.run(
        flags,
        None,
        anchor,
        traces_settings::InMemoryRepository::new(),
    )?;
    #[expect(
        deprecated,
        reason = "Currently migrating to new cache_root extraction method, \
                  but tests rely on Config::to_cache_spec()"
    )]
    let cache_spec = bootstrap
        .config
        .to_cache_spec()
        .map_err(|e| CliError::Bootstrap(AppError::Config(e)))?;
    let cache_dir = cache_spec.to_dir_path().map_err(|e| {
        TemplateCommandError::ConfigInvalid {
            detail: format!("Cache directory from config is invalid: {e}"),
        }
    })?;
    let spec = bootstrap
        .config
        .to_template_spec()
        .map_err(|e| CliError::Bootstrap(AppError::Config(e)))?;
    let template_root = spec.to_dir_path().map_err(|e| {
        TemplateCommandError::ConfigInvalid {
            detail: format!("Template directory from config is invalid: {e}"),
        }
    })?;
    let template_input = normalize_template_input(&args.input);
    let template_path =
        template_root.as_path().join(format!("{template_input}.md"));
    let name = TemplateName::try_new(&template_path, template_root.as_path())
        .map_err(|e| TemplateCommandError::OutputPathInvalid {
        detail: format!("name: {e}"),
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
        } => match format {
            OutputFormat::Human => writeln!(out, "{}", rendered.as_str())
                .map_err(crate::output::stdout_err)?,
            OutputFormat::Json => writeln!(
                out,
                "{}",
                serde_json::json!({ "preview": rendered.as_str() })
            )
            .map_err(crate::output::stdout_err)?,
        },
        CreateTemplateOutcome::Created {
            output_path: created_path,
            ..
        } => match format {
            OutputFormat::Human => {
                writeln!(out, "{}", created_path.as_path().display())
                    .map_err(crate::output::stdout_err)?;
            }
            OutputFormat::Json => writeln!(
                out,
                "{}",
                serde_json::json!({ "output": created_path.as_path().display().to_string() })
            )
            .map_err(crate::output::stdout_err)?,
        },
        // ponytail: catch-all for outcome variants not yet handled
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
            TemplateCommandError::ConfigInvalid {
                detail: format!("path: {e}"),
            }
            .into()
        }
        AppError::Template(TemplateError::Artifact(
            TemplateArtifactError::Path(e),
        )) => TemplateCommandError::OutputPathInvalid {
            detail: e.to_string(),
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
            TemplateCommandError::RenderFailed {
                detail: format!("Name derivation failed: {e}"),
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
            TemplateCommandError::ConfigInvalid {
                detail: format!(
                    "Template directory scan failed: {e}. Check directory \
                     permissions and retry."
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
    use traces_settings::{DiscoveryFlags, DiscoveryService};
    use traces_template::{
        TemplateArtifactError, TemplateBodyError, TemplateDirScanError,
        TemplateEngineError, TemplateError, TemplateName, TemplateNameError,
        TemplateReadError, TemplateRepositoryError,
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
            .expect("create .traces/cache");
        fs::create_dir_all(dir.path().join(".cache")).expect("create .cache");

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
        use pretty_assertions::assert_eq;

        use super::*;

        #[test]
        fn parses_single_var() {
            let result = parse_vars(["name=Alice".to_owned()]);
            assert!(result.is_ok(), "parse_vars should succeed");
            let parsed = result.unwrap();
            assert_eq!(
                parsed,
                HashMap::from([("name".to_owned(), "Alice".to_owned())])
            );
        }

        #[test]
        fn parses_multiple_vars() {
            let result =
                parse_vars(["name=Alice".to_owned(), "mode=cli".to_owned()]);
            assert!(result.is_ok(), "parse_vars should succeed");
            let parsed = result.unwrap();
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
            let result = parse_vars(["query=a=b".to_owned()]);
            assert!(result.is_ok(), "parse_vars should succeed");
            let parsed = result.unwrap();
            assert_eq!(parsed.get("query"), Some(&"a=b".to_owned()));
        }

        #[test]
        fn rejects_var_without_equals() {
            let err = parse_vars(["missing".to_owned()]).unwrap_err();
            assert!(
                matches!(err, TemplateCommandError::InvalidVarFormat { value } if value == "missing")
            );
        }

        #[test]
        fn duplicate_var_last_wins() {
            let result =
                parse_vars(["name=Alice".to_owned(), "name=Bob".to_owned()]);
            assert!(result.is_ok());
            let parsed = result.unwrap();
            assert_eq!(parsed.get("name"), Some(&"Bob".to_owned()));
        }
    }

    mod run_template_handler {
        use pretty_assertions::assert_eq;

        use super::*;

        #[test]
        fn renders_template_to_disk() {
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
            assert_eq!(
                fs::read_to_string(dir.path().join("notes/out.md"))
                    .expect("output file should exist"),
                "Hello Alice"
            );
        }

        #[test]
        fn prints_path_to_stdout() {
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
            let stdout = String::from_utf8(out)
                .expect("valid utf-8 output from template command");
            assert_eq!(stdout, "notes/out.md\n");
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
            let output = String::from_utf8(out)
                .expect("valid utf-8 output from template command");
            assert_eq!(output, "notes/out.md\n");
            let content = fs::read_to_string(dir.path().join("notes/out.md"))
                .expect("output file should exist");
            assert_eq!(content, "Hello Alice");
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
            let output = String::from_utf8(out)
                .expect("valid utf-8 output from template command");
            assert_eq!(output, "Hello Alice\n");
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
        }

        #[test]
        fn json_dry_run_prints_preview() {
            let (dir, bootstrapper, flags) = make_vault();
            let mut out = Vec::new();
            let mut err = Vec::new();

            let result = run_template(
                &bootstrapper,
                Some(flags),
                dir.path(),
                args(true),
                OutputFormat::Json,
                0,
                &mut out,
                &mut err,
            );

            assert!(result.is_ok(), "run_template failed: {result:?}");
            let stdout = String::from_utf8(out)
                .expect("valid utf-8 json output from template command");
            let parsed: serde_json::Value = serde_json::from_str(stdout.trim())
                .expect("stdout should be valid JSON");
            assert_eq!(
                parsed.get("preview"),
                Some(&serde_json::json!("Hello Alice"))
            );
        }

        #[test]
        fn json_created_prints_path() {
            let (dir, bootstrapper, flags) = make_vault();
            let mut out = Vec::new();
            let mut err = Vec::new();

            let result = run_template(
                &bootstrapper,
                Some(flags),
                dir.path(),
                args(false),
                OutputFormat::Json,
                0,
                &mut out,
                &mut err,
            );

            assert!(result.is_ok(), "run_template failed: {result:?}");
            let stdout = String::from_utf8(out)
                .expect("valid utf-8 json output from template command");
            let parsed: serde_json::Value = serde_json::from_str(stdout.trim())
                .expect("stdout should be valid JSON");
            assert_eq!(
                parsed.get("output"),
                Some(&serde_json::json!("notes/out.md"))
            );
        }
    }

    mod error_mapping {
        use super::*;

        fn template_command_error(
            err: AppError,
        ) -> Option<TemplateCommandError> {
            match map_template_error(err) {
                CliError::TemplateCommand(err) => Some(err),
                _ => None,
            }
        }

        #[test]
        fn maps_not_found() {
            let err = AppError::Template(TemplateError::NotFound {
                name: template_name("missing"),
            });
            assert!(
                matches!(template_command_error(err), Some(TemplateCommandError::TemplateNotFound { name }) if name == "missing")
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
                matches!(template_command_error(err), Some(TemplateCommandError::RenderFailed { detail }) if detail.contains("daily"))
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
                Some(TemplateCommandError::OutputPathInvalid { .. })
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
                matches!(template_command_error(err), Some(TemplateCommandError::DestinationExists { path }) if path == "notes/out.md")
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
                matches!(template_command_error(err), Some(TemplateCommandError::WriteFailed { detail }) if detail == "notes/out.md: disk full")
            );
        }

        #[test]
        fn maps_name_error_without_raw_template_display() {
            let err = AppError::Template(TemplateError::Name(
                TemplateNameError::Derivation,
            ));

            assert!(
                matches!(template_command_error(err), Some(TemplateCommandError::RenderFailed { detail })                 if detail.starts_with("Name derivation failed:"))
            );
        }

        #[test]
        fn maps_body_error() {
            let err = AppError::Template(TemplateError::Body(
                TemplateBodyError::Empty,
            ));
            assert!(matches!(
                template_command_error(err),
                Some(TemplateCommandError::RenderFailed { .. })
            ));
        }

        #[test]
        fn maps_read_error() {
            let path = std::path::PathBuf::from("test.md");
            let source = std::io::Error::other("permission denied");
            let read_err = TemplateReadError::from(traces_fs::ReadError::Io {
                path,
                source,
            });
            let err = AppError::Template(TemplateError::Read(read_err));
            assert!(
                matches!(template_command_error(err), Some(TemplateCommandError::RenderFailed { detail }) if detail.contains("Could not read"))
            );
        }

        #[test]
        fn maps_repository_error() {
            let err = AppError::Template(TemplateError::Repository(
                TemplateRepositoryError::NotFoundByPath(
                    traces_fs::PathKey::try_new("templates/test.md")
                        .expect("valid PathKey"),
                ),
            ));
            assert!(
                matches!(template_command_error(err), Some(TemplateCommandError::RenderFailed { detail }) if detail.contains("Run 'traces index'"))
            );
        }

        #[test]
        fn maps_scan_error() {
            let scan_err = TemplateDirScanError::from(
                traces_fs::error::ScanError::Traversal {
                    path: std::path::PathBuf::from("templates"),
                    source: std::io::Error::other("permission denied"),
                },
            );
            let err = AppError::Template(TemplateError::Scan(scan_err));
            assert!(
                matches!(template_command_error(err), Some(TemplateCommandError::ConfigInvalid { detail }) if detail.contains("Template directory scan failed:"))
            );
        }

        #[test]
        fn non_template_error_wraps_in_bootstrap() {
            let err = AppError::Config(
                traces_settings::error::ConfigError::Ingestion(
                    "bad config".into(),
                ),
            );
            let result = map_template_error(err);
            assert!(matches!(result, CliError::Bootstrap(AppError::Config(_))));
        }
    }
}
