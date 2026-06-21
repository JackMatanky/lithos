//! Handler for the `lithos doctor` subcommand.
//!
//! This module provides [`run_doctor`], which calls the full bootstrap
//! pipeline and discovery only, then writes a diagnostics section
//! summarising the bootstrap/config health to the provided output writers.
//!
//! On success, the summary is written to `out`.
//! On failure, a "failed" section is still written to `out`, and the error
//! is returned so that the CLI exit code can be set appropriately.

// This module is wired to the CLI dispatch layer in main.rs.

use std::{io::Write, path::Path};

use lithos_core::{
    app::bootstrap::Bootstrapper,
    config::InMemoryRepository,
    discovery::{DiscoveryFlags, port::DiscoveryPort},
};

use crate::{cli::OutputFormat, error::CliError, output};

// ----------------------------------------------------------- //
//                       Command Handler                       //
// ----------------------------------------------------------- //

/// Runs the `lithos doctor` command handler.
///
/// Calls the full bootstrap pipeline to verify config health, then runs
/// discovery only to obtain candidate paths for display. Writes a
/// Bootstrap/Config Diagnostics section to `out`. On failure, the section
/// still includes the error message before returning `Err`.
///
/// # Errors
///
/// Returns [`CliError`] if the bootstrap pipeline fails (vault not found,
/// invalid path, or permission denied).
#[expect(
    clippy::too_many_arguments,
    reason = "handler signature matches the CLI dispatch protocol: \
              bootstrapper, discovery flags, anchor, output format, \
              verbosity, stdout, stderr"
)]
pub(crate) fn run_doctor<D: DiscoveryPort>(
    bootstrapper: &Bootstrapper<D>,
    flags: Option<DiscoveryFlags>,
    anchor: &Path,
    format: OutputFormat,
    _verbose: u8,
    out: &mut impl Write,
    _err: &mut impl Write,
) -> Result<(), CliError> {
    // Full bootstrap: verifies config parses correctly.
    let bootstrap_result = bootstrapper.run(
        flags.clone(),
        None,
        anchor,
        InMemoryRepository::new(),
    );

    match bootstrap_result {
        Ok(_) => {
            // Discovery only: captures candidate paths for display.
            let discovery = bootstrapper
                .run_discovery_only(flags, None, anchor)
                .map_err(CliError::Bootstrap)?;
            let report = discovery.report().clone();

            let vault_root = discovery
                .vault()
                .first()
                .map(|c| c.base().as_path().to_path_buf());
            let vault_config_path = discovery
                .vault()
                .first()
                .map(|c| c.path().as_path().to_path_buf());
            let global_config_path = discovery
                .global()
                .first()
                .map(|c| c.path().as_path().to_path_buf());

            write_success(
                format,
                vault_root.as_deref(),
                vault_config_path.as_deref(),
                global_config_path.as_deref(),
                &report,
                out,
            )?;
            Ok(())
        }
        Err(bootstrap_err) => {
            let message = bootstrap_err.to_string();
            write_failure(format, &message, out)?;
            Err(CliError::Bootstrap(bootstrap_err))
        }
    }
}

// ----------------------------------------------------------- //
//                      Output Formatting                      //
// ----------------------------------------------------------- //

/// Writes the success diagnostics section to `out`.
#[expect(
    clippy::too_many_arguments,
    reason = "Internal output helper passes individual fields extracted from \
              discovery results; grouping them into a struct would create a \
              one-use type with no other consumers."
)]
fn write_success(
    format: OutputFormat,
    vault_root: Option<&Path>,
    vault_config: Option<&Path>,
    global_config: Option<&Path>,
    report: &lithos_core::discovery::report::DiscoveryReport,
    out: &mut impl Write,
) -> Result<(), CliError> {
    match format {
        OutputFormat::Human => write_human_success(
            vault_root,
            vault_config,
            global_config,
            report,
            out,
        ),
        OutputFormat::Json => write_json_success(
            vault_root,
            vault_config,
            global_config,
            report,
            out,
        ),
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
    vault_root: Option<&Path>,
    vault_config: Option<&Path>,
    global_config: Option<&Path>,
    report: &lithos_core::discovery::report::DiscoveryReport,
    out: &mut impl Write,
) -> Result<(), CliError> {
    let vault_root_str = vault_root
        .map_or_else(|| "none".to_owned(), |p| p.display().to_string());

    let vault_config_str = vault_config
        .map_or_else(|| "none".to_owned(), |p| p.display().to_string());

    let global_config_str = global_config
        .map_or_else(|| "none".to_owned(), |p| p.display().to_string());

    let suppressed = if report.global_resolution_skip_reason.is_some() {
        "yes"
    } else {
        "no"
    };

    writeln!(out, "Bootstrap/Config Diagnostics")
        .map_err(output::stdout_err)?;
    writeln!(out, "  vault root:  {vault_root_str}")
        .map_err(output::stdout_err)?;
    writeln!(out, "  vault config: {vault_config_str}")
        .map_err(output::stdout_err)?;
    writeln!(out, "  global config: {global_config_str}")
        .map_err(output::stdout_err)?;
    writeln!(out, "  global config suppressed: {suppressed}")
        .map_err(output::stdout_err)?;
    writeln!(out, "  status: healthy").map_err(output::stdout_err)?;

    Ok(())
}

/// Writes human-readable failure diagnostics.
fn write_human_failure(
    message: &str,
    out: &mut impl Write,
) -> Result<(), CliError> {
    writeln!(out, "Bootstrap/Config Diagnostics")
        .map_err(output::stdout_err)?;
    writeln!(out, "  status: failed").map_err(output::stdout_err)?;
    writeln!(out, "  error: {message}").map_err(output::stdout_err)?;

    Ok(())
}

/// Serialisable representation of the bootstrap section in the success case.
#[derive(serde::Serialize)]
struct DoctorBootstrapSection<'a> {
    /// Resolved vault root directory path, or `null` if not found.
    vault_root: Option<&'a str>,
    /// Resolved vault config file path, or `null` if not found.
    vault_config: Option<&'a str>,
    /// Resolved global config file path, or `null` if not found.
    global_config: Option<&'a str>,
    /// Whether global config resolution was suppressed by a CLI flag.
    global_config_suppressed: bool,
    /// Bootstrap health status (`"healthy"`).
    status: &'static str,
}

/// Serialisable representation of the `doctor` command's JSON success output.
#[derive(serde::Serialize)]
struct DoctorOutput<'a> {
    /// Bootstrap and config diagnostics section.
    bootstrap: DoctorBootstrapSection<'a>,
}

/// Writes JSON success diagnostics using [`serde_json`].
fn write_json_success(
    vault_root: Option<&Path>,
    vault_config: Option<&Path>,
    global_config: Option<&Path>,
    report: &lithos_core::discovery::report::DiscoveryReport,
    out: &mut impl Write,
) -> Result<(), CliError> {
    let payload = DoctorOutput {
        bootstrap: DoctorBootstrapSection {
            vault_root: vault_root.and_then(Path::to_str),
            vault_config: vault_config.and_then(Path::to_str),
            global_config: global_config.and_then(Path::to_str),
            global_config_suppressed: report
                .global_resolution_skip_reason
                .is_some(),
            status: "healthy",
        },
    };
    output::write_json_line(out, &payload)
}

/// Serialisable representation of the bootstrap section in the failure case.
#[derive(serde::Serialize)]
struct DoctorFailureSection {
    /// Bootstrap health status (`"failed"`).
    status: &'static str,
    /// Human-readable error message from the bootstrap pipeline.
    error: String,
}

/// Serialisable representation of the `doctor` command's JSON failure output.
#[derive(serde::Serialize)]
struct DoctorFailureOutput {
    /// Bootstrap and config diagnostics section (failure variant).
    bootstrap: DoctorFailureSection,
}

/// Writes JSON failure diagnostics using [`serde_json`].
fn write_json_failure(
    message: &str,
    out: &mut impl Write,
) -> Result<(), CliError> {
    let payload = DoctorFailureOutput {
        bootstrap: DoctorFailureSection {
            status: "failed",
            error: message.to_owned(),
        },
    };
    output::write_json_line(out, &payload)
}

// ----------------------------------------------------------- //
//                            Tests                            //
// ----------------------------------------------------------- //

#[cfg(test)]
mod doctor_handler {
    use clap::Parser;
    use lithos_core::{
        app::bootstrap::Bootstrapper,
        discovery::{DiscoveryFlags, service::DiscoveryService},
    };

    use super::run_doctor;
    use crate::{
        cli::{Cli, OutputFormat},
        error::CliError,
    };

    /// Creates a temp vault dir with `lithos.toml` and returns bootstrapper
    /// plus flags pointing at it.
    fn make_vault()
    -> (tempfile::TempDir, Bootstrapper<DiscoveryService>, DiscoveryFlags) {
        let dir = tempfile::tempdir().expect("vault dir");
        let config_path = dir.path().join("lithos.toml");
        // Must contain a non-default field so `InMemoryRepository` takes the
        // Rebuild path rather than UseCached (empty TOML → "No active config
        // version found").
        std::fs::write(&config_path, "[template]\ndirectory = \"templates\"")
            .expect("write lithos.toml");
        let flags = DiscoveryFlags::new(
            Some(config_path.as_path()),
            Some(dir.path()),
            true, // suppress global
        )
        .expect("valid flags");
        let bootstrapper = Bootstrapper::with_global_directories(vec![])
            .expect("bootstrapper");
        (dir, bootstrapper, flags)
    }

    fn run_human(
        bootstrapper: &Bootstrapper<DiscoveryService>,
        flags: Option<DiscoveryFlags>,
        anchor: &std::path::Path,
    ) -> (String, String, Result<(), CliError>) {
        let mut out = Vec::<u8>::new();
        let mut err = Vec::<u8>::new();
        let result = run_doctor(
            bootstrapper,
            flags,
            anchor,
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
        bootstrapper: &Bootstrapper<DiscoveryService>,
        flags: Option<DiscoveryFlags>,
        anchor: &std::path::Path,
    ) -> (String, String, Result<(), CliError>) {
        let mut out = Vec::<u8>::new();
        let mut err = Vec::<u8>::new();
        let result = run_doctor(
            bootstrapper,
            flags,
            anchor,
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
    fn run_doctor_reports_healthy_when_bootstrap_succeeds() {
        let (dir, bootstrapper, flags) = make_vault();

        let (stdout, _, result) =
            run_human(&bootstrapper, Some(flags), dir.path());

        assert!(result.is_ok(), "expected Ok when bootstrap succeeds");
        assert!(
            stdout.contains("status: healthy"),
            "expected 'status: healthy' in output, got: {stdout}"
        );
    }

    #[test]
    fn writes_bootstrap_section_to_output() {
        let (dir, bootstrapper, flags) = make_vault();

        let (stdout, _, _) = run_human(&bootstrapper, Some(flags), dir.path());

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
    fn reports_failed_when_no_vault_found() {
        // Use a non-existent anchor to force discovery failure.
        let bootstrapper = Bootstrapper::with_global_directories(vec![])
            .expect("bootstrapper");
        let bad_anchor =
            std::path::Path::new("/this/path/does/not/exist/at/all");
        let mut out = Vec::<u8>::new();
        let mut err = Vec::<u8>::new();

        let result = run_doctor(
            &bootstrapper,
            None,
            bad_anchor,
            OutputFormat::Human,
            0,
            &mut out,
            &mut err,
        );

        let stdout = String::from_utf8(out).expect("utf8");
        assert!(
            stdout.contains("status: failed"),
            "expected 'status: failed' section in output, got: {stdout}"
        );
        assert!(result.is_err(), "expected Err when vault not found, got Ok");
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
        let (dir, bootstrapper, flags) = make_vault();
        let (stdout, _, json_result) =
            run_json(&bootstrapper, Some(flags), dir.path());

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
