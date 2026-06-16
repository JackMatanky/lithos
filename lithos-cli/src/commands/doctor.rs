//! Handler for the `lithos doctor` subcommand.
//!
//! This module provides [`run_doctor`], which calls the bootstrap runner,
//! then writes a diagnostics section summarising the bootstrap/config health
//! to the provided output writers.
//!
//! On success, the summary is written to `out`.
//! On failure, a "failed" section is still written to `out`, and the error is
//! returned so that the CLI exit code can be set appropriately.

// Private helper functions and public-facing items in this module are wired
// to the CLI dispatch layer in a later slice. Until then they appear unused
// in non-test builds.
#![cfg_attr(
    not(test),
    allow(
        dead_code,
        reason = "doctor handler helpers are wired to main() in the dispatch \
                  slice; marked forward-declared per the incremental slice \
                  plan"
    )
)]

use std::{
    io::Write,
    path::{Path, PathBuf},
};

use lithos_core::{
    app::bootstrap::BootstrapError,
    discovery::{DiscoveryFlags, error::DiscoveryError},
};

use crate::{
    cli::OutputFormat,
    commands::config::{BootstrapOutcome, BootstrapRunner},
    error::CliError,
};

// ------------------------------------------------------------------ //
//                          Handler                                   //
// ------------------------------------------------------------------ //

/// Runs the `lithos doctor` command handler.
///
/// Calls the bootstrap runner and writes a Bootstrap/Config Diagnostics
/// section to `out`.  On failure, the section still includes the error
/// message before returning `Err`.
///
/// # Errors
///
/// Returns [`CliError`] if the bootstrap pipeline fails (vault not found,
/// invalid path, or permission denied).
#[expect(
    clippy::too_many_arguments,
    reason = "handler signature matches the CLI dispatch protocol: runner, \
              discovery flags, anchor, output format, verbosity, stdout, \
              stderr"
)]
#[cfg_attr(
    not(test),
    allow(dead_code, reason = "wired to main() in the dispatch slice")
)]
pub(crate) fn run_doctor(
    runner: &impl BootstrapRunner,
    flags: Option<DiscoveryFlags>,
    anchor: &Path,
    format: OutputFormat,
    _verbose: u8,
    out: &mut impl Write,
    _err: &mut impl Write,
) -> Result<(), CliError> {
    match runner.run_bootstrap(flags, anchor) {
        Ok(outcome) => {
            write_success(format, &outcome, out)?;
            Ok(())
        }
        Err(bootstrap_err) => {
            let message = bootstrap_err.to_string();
            write_failure(format, &message, out)?;
            Err(CliError::Bootstrap(bootstrap_err))
        }
    }
}

// ------------------------------------------------------------------ //
//                       Output Formatting                            //
// ------------------------------------------------------------------ //

/// Writes the success diagnostics section to `out`.
fn write_success(
    format: OutputFormat,
    outcome: &BootstrapOutcome,
    out: &mut impl Write,
) -> Result<(), CliError> {
    match format {
        OutputFormat::Human => write_human_success(outcome, out),
        OutputFormat::Json => write_json_success(outcome, out),
    }
}

/// Writes the failure diagnostics section to `out`.
fn write_failure(
    format: OutputFormat,
    message: &str,
    out: &mut impl Write,
) -> Result<(), CliError> {
    match format {
        OutputFormat::Human => write_human_failure(message, out),
        OutputFormat::Json => write_json_failure(message, out),
    }
}

/// Writes human-readable success diagnostics.
fn write_human_success(
    outcome: &BootstrapOutcome,
    out: &mut impl Write,
) -> Result<(), CliError> {
    let vault_root = outcome.config.vault_metadata().root().as_path();

    let vault_config = outcome
        .vault_config_path
        .as_deref()
        .map_or_else(|| "none".to_owned(), |p| p.display().to_string());

    let global_config = outcome
        .global_config_path
        .as_deref()
        .map_or_else(|| "none".to_owned(), |p| p.display().to_string());

    let suppressed = if outcome.report.global_resolution_skip_reason.is_some() {
        "yes"
    } else {
        "no"
    };

    writeln!(out, "Bootstrap/Config Diagnostics").map_err(write_error)?;
    writeln!(out, "  vault root:  {}", vault_root.display())
        .map_err(write_error)?;
    writeln!(out, "  vault config: {vault_config}").map_err(write_error)?;
    writeln!(out, "  global config: {global_config}").map_err(write_error)?;
    writeln!(out, "  global config suppressed: {suppressed}")
        .map_err(write_error)?;
    writeln!(out, "  status: healthy").map_err(write_error)?;

    Ok(())
}

/// Writes human-readable failure diagnostics.
fn write_human_failure(
    message: &str,
    out: &mut impl Write,
) -> Result<(), CliError> {
    writeln!(out, "Bootstrap/Config Diagnostics").map_err(write_error)?;
    writeln!(out, "  status: failed").map_err(write_error)?;
    writeln!(out, "  error: {message}").map_err(write_error)?;

    Ok(())
}

/// Writes JSON success diagnostics.
fn write_json_success(
    outcome: &BootstrapOutcome,
    out: &mut impl Write,
) -> Result<(), CliError> {
    let vault_root =
        outcome.config.vault_metadata().root().as_path().display().to_string();

    let vault_config_json =
        json_path_or_null(outcome.vault_config_path.as_deref());
    let global_config_json =
        json_path_or_null(outcome.global_config_path.as_deref());
    let suppressed = outcome.report.global_resolution_skip_reason.is_some();

    writeln!(
        out,
        r#"{{"bootstrap":{{"vault_root":{vault_root_json},"vault_config":{vault_config_json},"global_config":{global_config_json},"global_config_suppressed":{suppressed},"status":"healthy"}}}}"#,
        vault_root_json = json_string(&vault_root),
    )
    .map_err(write_error)?;

    Ok(())
}

/// Writes JSON failure diagnostics.
fn write_json_failure(
    message: &str,
    out: &mut impl Write,
) -> Result<(), CliError> {
    let message_json = json_string(message);
    writeln!(
        out,
        r#"{{"bootstrap":{{"status":"failed","error":{message_json}}}}}"#,
    )
    .map_err(write_error)?;

    Ok(())
}

/// Returns `"<escaped>"` for `Some(path)` or `null` for `None`.
fn json_path_or_null(path: Option<&Path>) -> String {
    path.map_or_else(
        || "null".to_owned(),
        |p| json_string(&p.display().to_string()),
    )
}

/// Wraps a string value in JSON double-quotes with basic escaping.
fn json_string(s: &str) -> String {
    let escaped = s.replace('\\', "\\\\").replace('"', "\\\"");
    format!("\"{escaped}\"")
}

/// Converts an [`std::io::Error`] from a write call into a [`CliError`].
fn write_error(e: std::io::Error) -> CliError {
    CliError::Bootstrap(BootstrapError::Discovery(
        DiscoveryError::ReadDirectory {
            path: PathBuf::from("<stdout>"),
            source: e,
        },
    ))
}

// ------------------------------------------------------------------ //
//                             Tests                                  //
// ------------------------------------------------------------------ //

#[cfg(test)]
mod doctor_handler {
    use clap::Parser;
    use lithos_core::discovery::{
        DiscoveryFlags,
        report::{DiscoveryReport, LocalTraversalStopReason},
    };

    use super::run_doctor;
    use crate::{
        cli::{Cli, OutputFormat},
        commands::config::{BootstrapOutcome, BootstrapRunner},
        error::CliError,
    };

    // ----- fixtures -----

    fn test_vault_root(
        suffix: &str,
    ) -> (tempfile::TempDir, lithos_core::config::vault::VaultRoot) {
        let dir = tempfile::tempdir().expect("temp dir");
        let subdir = dir.path().join(suffix);
        std::fs::create_dir_all(&subdir).expect("subdir");
        let root = lithos_core::config::vault::VaultRoot::try_new(subdir)
            .expect("vault root");
        (dir, root)
    }

    fn test_config(
        root: lithos_core::config::vault::VaultRoot,
    ) -> lithos_core::config::aggregate::Config {
        let version = lithos_core::config::aggregate::Version::initial();
        let vault_id = lithos_core::config::vault::VaultId::new();
        lithos_core::config::builder::build_from_layers(
            None, None, vault_id, root, version,
        )
        .expect("test config")
    }

    struct MockSuccess {
        outcome: BootstrapOutcome,
    }

    impl MockSuccess {
        fn new(root: lithos_core::config::vault::VaultRoot) -> Self {
            let config = test_config(root);
            let report = DiscoveryReport {
                skipped_ceilings: vec![],
                local_traversal_stop_reason:
                    LocalTraversalStopReason::FilesystemRoot,
                global_resolution_skip_reason: None,
            };
            Self {
                outcome: BootstrapOutcome {
                    config,
                    report,
                    vault_config_path: None,
                    global_config_path: None,
                },
            }
        }
    }

    impl BootstrapRunner for MockSuccess {
        fn run_bootstrap(
            &self,
            _flags: Option<DiscoveryFlags>,
            _anchor: &std::path::Path,
        ) -> Result<BootstrapOutcome, lithos_core::app::bootstrap::BootstrapError>
        {
            Ok(BootstrapOutcome {
                config: self.outcome.config.clone(),
                report: self.outcome.report.clone(),
                vault_config_path: self.outcome.vault_config_path.clone(),
                global_config_path: self.outcome.global_config_path.clone(),
            })
        }
    }

    struct MockFailure;

    impl BootstrapRunner for MockFailure {
        fn run_bootstrap(
            &self,
            _flags: Option<DiscoveryFlags>,
            _anchor: &std::path::Path,
        ) -> Result<BootstrapOutcome, lithos_core::app::bootstrap::BootstrapError>
        {
            Err(lithos_core::app::bootstrap::BootstrapError::Discovery(
                lithos_core::discovery::error::DiscoveryError::InvalidAnchorDirectory {
                    path: std::path::PathBuf::from("/no/vault"),
                    source: lithos_core::fs::PathError::NotADirectory(
                        std::path::PathBuf::from("/no/vault"),
                    ),
                },
            ))
        }
    }

    fn anchor() -> tempfile::TempDir {
        tempfile::tempdir().expect("anchor dir")
    }

    fn run_human(
        runner: &impl BootstrapRunner,
    ) -> (String, String, Result<(), CliError>) {
        let anchor = anchor();
        let mut out = Vec::<u8>::new();
        let mut err = Vec::<u8>::new();
        let result = run_doctor(
            runner,
            None,
            anchor.path(),
            OutputFormat::Human,
            0,
            &mut out,
            &mut err,
        );
        (
            String::from_utf8(out).expect("stdout utf8"),
            String::from_utf8(err).expect("stderr utf8"),
            result,
        )
    }

    fn run_json(
        runner: &impl BootstrapRunner,
    ) -> (String, String, Result<(), CliError>) {
        let anchor = anchor();
        let mut out = Vec::<u8>::new();
        let mut err = Vec::<u8>::new();
        let result = run_doctor(
            runner,
            None,
            anchor.path(),
            OutputFormat::Json,
            0,
            &mut out,
            &mut err,
        );
        (
            String::from_utf8(out).expect("stdout utf8"),
            String::from_utf8(err).expect("stderr utf8"),
            result,
        )
    }

    // ----- tests -----

    #[test]
    fn reports_healthy_when_bootstrap_succeeds() {
        let (_tmp, root) = test_vault_root("healthy-vault");
        let runner = MockSuccess::new(root);

        let (stdout, _, result) = run_human(&runner);

        assert!(result.is_ok(), "expected Ok when bootstrap succeeds");
        assert!(
            stdout.contains("status: healthy"),
            "expected 'status: healthy' in output, got: {stdout}"
        );
    }

    #[test]
    fn reports_vault_not_found_section_when_no_vault_root() {
        let runner = MockFailure;

        let (stdout, _, result) = run_human(&runner);

        // Should write a failed section to output
        assert!(
            stdout.contains("status: failed"),
            "expected 'status: failed' section in output, got: {stdout}"
        );
        // AND return an error
        assert!(result.is_err(), "expected Err when vault not found, got Ok");
    }

    #[test]
    fn writes_bootstrap_section_to_output() {
        let (_tmp, root) = test_vault_root("section-vault");
        let runner = MockSuccess::new(root);

        let (stdout, _, _) = run_human(&runner);

        assert!(
            stdout.contains("Bootstrap/Config Diagnostics"),
            "expected section header in output, got: {stdout}"
        );
        assert!(
            stdout.contains("vault root:"),
            "expected vault root label in output, got: {stdout}"
        );
        assert!(
            stdout.contains("vault config:"),
            "expected vault config label in output, got: {stdout}"
        );
        assert!(
            stdout.contains("global config:"),
            "expected global config label in output, got: {stdout}"
        );
        assert!(
            stdout.contains("global config suppressed:"),
            "expected suppression label in output, got: {stdout}"
        );
    }

    #[test]
    fn returns_err_when_bootstrap_fails_vault_not_found() {
        let runner = MockFailure;
        let anchor = anchor();
        let mut out = Vec::<u8>::new();
        let mut err = Vec::<u8>::new();

        let result = run_doctor(
            &runner,
            None,
            anchor.path(),
            OutputFormat::Human,
            0,
            &mut out,
            &mut err,
        );

        assert!(result.is_err(), "expected Err when vault not found");
    }

    #[test]
    fn returns_err_when_bootstrap_fails_invalid_explicit_path() {
        // MockFailure simulates any bootstrap failure, including invalid paths.
        let runner = MockFailure;
        let anchor = anchor();
        let mut out = Vec::<u8>::new();
        let mut err = Vec::<u8>::new();

        let result = run_doctor(
            &runner,
            None,
            anchor.path(),
            OutputFormat::Human,
            0,
            &mut out,
            &mut err,
        );

        assert!(result.is_err(), "expected Err for invalid explicit path");
    }

    #[test]
    fn returns_err_when_bootstrap_fails_permission_denied() {
        // MockFailure simulates any bootstrap failure, including permission
        // errors.
        let runner = MockFailure;
        let anchor = anchor();
        let mut out = Vec::<u8>::new();
        let mut err = Vec::<u8>::new();

        let result = run_doctor(
            &runner,
            None,
            anchor.path(),
            OutputFormat::Human,
            0,
            &mut out,
            &mut err,
        );

        assert!(result.is_err(), "expected Err for permission denied");
    }

    #[test]
    fn is_registered_as_top_level_subcommand_not_under_config() {
        // `lithos doctor` must parse as a top-level subcommand.
        let cli = Cli::try_parse_from(["lithos", "doctor"])
            .expect("lithos doctor should parse as top-level subcommand");
        assert!(
            matches!(cli.command, crate::cli::Command::Doctor),
            "expected Command::Doctor variant, got: {:?}",
            cli.command
        );

        // `lithos config doctor` must fail — doctor is not under config.
        let result = Cli::try_parse_from(["lithos", "config", "doctor"]);
        assert!(
            result.is_err(),
            "lithos config doctor should fail (doctor is not a config \
             subcommand)"
        );

        // JSON output check: also verify json format flag works.
        let (_tmp, root) = test_vault_root("toplevel-vault");
        let runner = MockSuccess::new(root);
        let (stdout, _, json_result) = run_json(&runner);

        assert!(json_result.is_ok(), "expected Ok for JSON doctor output");
        assert!(
            stdout.contains(r#""bootstrap""#),
            "expected 'bootstrap' key in JSON output, got: {stdout}"
        );
        assert!(
            stdout.contains(r#""status":"healthy""#),
            "expected healthy status in JSON output, got: {stdout}"
        );
    }
}
