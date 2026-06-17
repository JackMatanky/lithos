//! Handler for the `lithos config` subcommand.
//!
//! This module provides [`run_config`], which calls the bootstrapper's full
//! pipeline (discovery → config build) and its discovery-only pipeline,
//! then formats and writes the resolved configuration summary to the provided
//! output writers.
//!
//! Structured output is written to `out`; verbose diagnostics (skipped
//! ceilings, stop reason) are written to `err`.

// This module is wired to the CLI dispatch layer in main.rs.

use std::{
    io::Write,
    path::{Path, PathBuf},
};

use lithos_core::{
    app::bootstrap::{BootstrapError, Bootstrapper},
    config::InMemoryRepository,
    discovery::{
        DiscoveryFlags,
        port::DiscoveryPort,
        report::{DiscoveryReport, SkippedCeilingReason},
    },
};

use crate::{cli::OutputFormat, error::CliError};

// ------------------------------------------------------------------ //
//                          Handler                                   //
// ------------------------------------------------------------------ //

/// Runs the `lithos config` command handler.
///
/// Calls the full bootstrap pipeline to verify config can be parsed, then
/// runs discovery only to obtain candidate paths for display. Writes the
/// resolved configuration summary to `out` in the requested `format`.
/// Verbose diagnostics (skipped ceilings, stop reason) are written to `err`
/// when `verbose > 0`.
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
pub(crate) fn run_config<D: DiscoveryPort>(
    bootstrapper: &Bootstrapper<D>,
    flags: Option<DiscoveryFlags>,
    anchor: &Path,
    format: OutputFormat,
    verbose: u8,
    out: &mut impl Write,
    err: &mut impl Write,
) -> Result<(), CliError> {
    // Full bootstrap: verifies config parses correctly.
    bootstrapper
        .run(flags.clone(), None, anchor, InMemoryRepository::new())
        .map_err(CliError::Bootstrap)?;

    // Discovery only: captures candidate paths for display.
    let (discovery, report) = bootstrapper
        .run_discovery_only(flags, None, anchor)
        .map_err(CliError::Bootstrap)?;

    write_diagnostics(verbose, &report, err)?;
    write_output(
        format,
        discovery.vault().first().map(|c| c.base().as_path()),
        discovery.vault().first().map(|c| c.path().as_path()),
        discovery.global().first().map(|c| c.path().as_path()),
        &report,
        out,
    )?;

    Ok(())
}

/// Writes verbose diagnostics to `err` when `verbose > 0`.
fn write_diagnostics(
    verbose: u8,
    report: &DiscoveryReport,
    err: &mut impl Write,
) -> Result<(), CliError> {
    if verbose == 0 {
        return Ok(());
    }

    for ceiling in &report.skipped_ceilings {
        let reason = match ceiling.reason {
            SkippedCeilingReason::EmptySegment => "empty segment",
            SkippedCeilingReason::InvalidPath => "invalid path",
        };
        writeln!(
            err,
            "warning: skipped ceiling {}: {reason}",
            ceiling.segment.display()
        )
        .map_err(write_error)?;
    }

    let stop = format_stop_reason(&report.local_traversal_stop_reason);
    writeln!(err, "stop reason: {stop}").map_err(write_error)?;

    if report.global_resolution_skip_reason.is_some() {
        writeln!(err, "global resolution skipped: suppressed by flag")
            .map_err(write_error)?;
    }

    Ok(())
}

/// Formats the local traversal stop reason as a short string.
fn format_stop_reason(
    reason: &lithos_core::discovery::report::LocalTraversalStopReason,
) -> &'static str {
    use lithos_core::discovery::report::LocalTraversalStopReason;
    match reason {
        LocalTraversalStopReason::FilesystemRoot => "filesystem root",
        LocalTraversalStopReason::ExplicitConfigFile => "explicit config file",
        LocalTraversalStopReason::ProjectBoundaryMarker {
            ..
        } => "project boundary marker",
        LocalTraversalStopReason::CeilingEnforced {
            ..
        } => "ceiling enforced",
    }
}

/// Writes the formatted config output to `out`.
#[expect(
    clippy::too_many_arguments,
    reason = "Internal output helper passes individual fields extracted from \
              discovery results; grouping them into a struct would create a \
              one-use type with no other consumers."
)]
fn write_output(
    format: OutputFormat,
    vault_root: Option<&Path>,
    vault_config_path: Option<&Path>,
    global_config_path: Option<&Path>,
    report: &DiscoveryReport,
    out: &mut impl Write,
) -> Result<(), CliError> {
    match format {
        OutputFormat::Human => write_human(
            vault_root,
            vault_config_path,
            global_config_path,
            report,
            out,
        ),
        OutputFormat::Json => write_json(
            vault_root,
            vault_config_path,
            global_config_path,
            report,
            out,
        ),
    }
}

/// Writes human-readable config output.
fn write_human(
    vault_root: Option<&Path>,
    vault_config: Option<&Path>,
    global_config: Option<&Path>,
    report: &DiscoveryReport,
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

    writeln!(out, "vault root:  {vault_root_str}").map_err(write_error)?;
    writeln!(out, "vault config: {vault_config_str}").map_err(write_error)?;
    writeln!(out, "global config: {global_config_str}").map_err(write_error)?;
    writeln!(out, "global config suppressed: {suppressed}")
        .map_err(write_error)?;

    Ok(())
}

/// Writes JSON config output.
fn write_json(
    vault_root: Option<&Path>,
    vault_config: Option<&Path>,
    global_config: Option<&Path>,
    report: &DiscoveryReport,
    out: &mut impl Write,
) -> Result<(), CliError> {
    let vault_root_json = vault_root.map_or_else(
        || "null".to_owned(),
        |p| json_string(&p.display().to_string()),
    );

    let vault_config_json = json_path_or_null(vault_config);
    let global_config_json = json_path_or_null(global_config);
    let suppressed = report.global_resolution_skip_reason.is_some();

    writeln!(
        out,
        r#"{{"vault_root":{vault_root_json},"vault_config":{vault_config_json},"global_config":{global_config_json},"global_config_suppressed":{suppressed}}}"#,
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
        lithos_core::discovery::error::DiscoveryError::ReadDirectory {
            path: PathBuf::from("<stdout>"),
            source: e,
        },
    ))
}

// ------------------------------------------------------------------ //
//                             Tests                                  //
// ------------------------------------------------------------------ //

#[cfg(test)]
mod config_handler {
    use lithos_core::{
        app::bootstrap::Bootstrapper,
        discovery::{DiscoveryFlags, service::DiscoveryService},
    };

    use super::run_config;
    use crate::cli::OutputFormat;

    // ----- helpers -----

    /// Creates a temp directory with a minimal vault (`lithos.toml`) and
    /// returns a `Bootstrapper` wired to discover it via explicit flags.
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
            true, /* suppress global — no accidental global config reads in
                   * tests */
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
    ) -> (String, String) {
        let mut out = Vec::<u8>::new();
        let mut err = Vec::<u8>::new();
        run_config(
            bootstrapper,
            flags,
            anchor,
            OutputFormat::Human,
            0,
            &mut out,
            &mut err,
        )
        .expect("run_config should succeed");
        (
            String::from_utf8(out).expect("stdout utf8"),
            String::from_utf8(err).expect("stderr utf8"),
        )
    }

    fn run_json(
        bootstrapper: &Bootstrapper<DiscoveryService>,
        flags: Option<DiscoveryFlags>,
        anchor: &std::path::Path,
    ) -> (String, String) {
        let mut out = Vec::<u8>::new();
        let mut err = Vec::<u8>::new();
        run_config(
            bootstrapper,
            flags,
            anchor,
            OutputFormat::Json,
            0,
            &mut out,
            &mut err,
        )
        .expect("run_config should succeed");
        (
            String::from_utf8(out).expect("stdout utf8"),
            String::from_utf8(err).expect("stderr utf8"),
        )
    }

    fn run_verbose(
        bootstrapper: &Bootstrapper<DiscoveryService>,
        flags: Option<DiscoveryFlags>,
        anchor: &std::path::Path,
    ) -> (String, String) {
        let mut out = Vec::<u8>::new();
        let mut err = Vec::<u8>::new();
        run_config(
            bootstrapper,
            flags,
            anchor,
            OutputFormat::Human,
            1,
            &mut out,
            &mut err,
        )
        .expect("run_config should succeed");
        (
            String::from_utf8(out).expect("stdout utf8"),
            String::from_utf8(err).expect("stderr utf8"),
        )
    }

    // ----- tests -----

    #[test]
    fn run_config_writes_vault_root_in_human_format() {
        let (dir, bootstrapper, flags) = make_vault();

        let (stdout, _) = run_human(&bootstrapper, Some(flags), dir.path());

        assert!(
            stdout.contains("vault root:"),
            "expected vault root label in stdout, got: {stdout}"
        );
        assert!(
            stdout.contains(&dir.path().display().to_string()),
            "expected vault root path in stdout, got: {stdout}"
        );
    }

    #[test]
    fn run_config_writes_json_format() {
        let (dir, bootstrapper, flags) = make_vault();

        let (stdout, _) = run_json(&bootstrapper, Some(flags), dir.path());

        assert!(
            stdout.contains("vault_root"),
            "expected vault_root key in JSON, got: {stdout}"
        );
        assert!(
            stdout.contains(&dir.path().display().to_string()),
            "expected vault root path in JSON, got: {stdout}"
        );
        assert!(
            stdout.contains("\"vault_config\""),
            "expected vault_config key in JSON, got: {stdout}"
        );
        assert!(
            stdout.contains("\"global_config\""),
            "expected global_config key in JSON, got: {stdout}"
        );
        assert!(
            stdout.contains("\"global_config_suppressed\""),
            "expected global_config_suppressed key in JSON, got: {stdout}"
        );
    }

    #[test]
    fn writes_structured_output_to_stdout_writer() {
        let (dir, bootstrapper, flags) = make_vault();

        let (stdout, _) = run_human(&bootstrapper, Some(flags), dir.path());

        assert!(
            stdout.contains("vault root:"),
            "expected vault root label in stdout, got: {stdout}"
        );
        assert!(
            stdout.contains("vault config:"),
            "expected vault config label in stdout, got: {stdout}"
        );
        assert!(
            stdout.contains("global config:"),
            "expected global config label in stdout, got: {stdout}"
        );
        assert!(
            stdout.contains("global config suppressed:"),
            "expected suppression label in stdout, got: {stdout}"
        );
    }

    #[test]
    fn includes_global_config_suppression_status_no_when_no_global_dirs() {
        // With no global directories configured, suppress_global has no
        // effect on the report — there is nothing to suppress, so
        // global_resolution_skip_reason stays None and the output shows "no".
        let (dir, bootstrapper, flags) = make_vault();

        let (stdout, _) = run_human(&bootstrapper, Some(flags), dir.path());

        assert!(
            stdout.contains("global config suppressed:"),
            "expected suppression label in output, got: {stdout}"
        );
        assert!(
            stdout.contains("global config suppressed: no"),
            "expected suppressed=no (no global dirs configured), got: {stdout}"
        );
    }

    #[test]
    fn writes_verbose_diagnostics_to_stderr_writer() {
        let (dir, bootstrapper, flags) = make_vault();

        let (stdout, stderr) =
            run_verbose(&bootstrapper, Some(flags), dir.path());

        assert!(
            stdout.contains("vault root:"),
            "structured output should go to stdout, got: {stdout}"
        );
        assert!(
            !stderr.is_empty(),
            "verbose diagnostics should go to stderr, got: {stderr}"
        );
        assert!(
            !stdout.contains("stop reason:"),
            "stop reason must not appear in stdout, got: {stdout}"
        );
        // stop reason must be in stderr when verbose
        assert!(
            stderr.contains("stop reason:"),
            "expected stop reason in stderr, got: {stderr}"
        );
    }

    #[test]
    fn json_output_includes_vault_config_path_when_found() {
        let (dir, bootstrapper, flags) = make_vault();
        let config_path = dir.path().join("lithos.toml");

        let (stdout, _) = run_json(&bootstrapper, Some(flags), dir.path());

        assert!(
            stdout.contains(&config_path.display().to_string()),
            "expected vault config path in JSON, got: {stdout}"
        );
    }

    #[test]
    fn json_output_includes_suppression_false_when_no_global_dirs() {
        // With no global directories configured, suppress_global has no
        // effect on the report — global_resolution_skip_reason stays None,
        // so the JSON field is false.
        let (dir, bootstrapper, flags) = make_vault();

        let (stdout, _) = run_json(&bootstrapper, Some(flags), dir.path());

        assert!(
            stdout.contains(r#""global_config_suppressed":false"#),
            "expected suppression=false (no global dirs configured), got: \
             {stdout}"
        );
    }
}
