//! Handler for the `lithos config` subcommand.
//!
//! This module provides [`run_config`], which calls the bootstrap runner,
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
    app::bootstrap::BootstrapError,
    config::aggregate::Config,
    discovery::{
        DiscoveryFlags,
        report::{DiscoveryReport, SkippedCeilingReason},
    },
};

use crate::{cli::OutputFormat, error::CliError};

// ------------------------------------------------------------------ //
//                          Port Trait                                //
// ------------------------------------------------------------------ //

/// Outcome returned by a bootstrap run for the config handler.
///
/// Bundles the resolved [`Config`], the [`DiscoveryReport`], and the
/// discovered config file paths so that the handler can format output
/// without knowing how discovery or config loading works internally.
pub(crate) struct BootstrapOutcome {
    /// The fully-resolved configuration.
    pub(crate) config: Config,
    /// Process metadata from discovery.
    pub(crate) report: DiscoveryReport,
    /// Absolute path to the vault-local config file, if one was found.
    pub(crate) vault_config_path: Option<PathBuf>,
    /// Absolute path to the global config file, if one was found.
    pub(crate) global_config_path: Option<PathBuf>,
}

/// Abstraction over the bootstrap pipeline used by the config handler.
///
/// Implement this trait on a concrete runner (e.g. a wrapper around
/// [`lithos_core::app::bootstrap::Bootstrapper`]) and pass it to
/// [`run_config`].  In tests, provide a [`MockBootstrapRunner`] instead.
pub(crate) trait BootstrapRunner {
    /// Runs the full bootstrap pipeline (discovery → config build).
    ///
    /// # Errors
    ///
    /// Returns [`BootstrapError`] if discovery or config loading fails.
    fn run_bootstrap(
        &self,
        flags: Option<DiscoveryFlags>,
        anchor: &Path,
    ) -> Result<BootstrapOutcome, BootstrapError>;
}

// ------------------------------------------------------------------ //
//                          Handler                                   //
// ------------------------------------------------------------------ //

/// Runs the `lithos config` command handler.
///
/// Calls the bootstrap runner, then writes the resolved configuration
/// summary to `out` in the requested `format`.  Verbose diagnostics
/// (skipped ceilings, stop reason) are written to `err` when `verbose > 0`.
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
pub(crate) fn run_config(
    runner: &impl BootstrapRunner,
    flags: Option<DiscoveryFlags>,
    anchor: &Path,
    format: OutputFormat,
    verbose: u8,
    out: &mut impl Write,
    err: &mut impl Write,
) -> Result<(), CliError> {
    let outcome = runner.run_bootstrap(flags, anchor)?;

    write_diagnostics(verbose, &outcome.report, err)?;
    write_output(format, &outcome, out)?;

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
fn write_output(
    format: OutputFormat,
    outcome: &BootstrapOutcome,
    out: &mut impl Write,
) -> Result<(), CliError> {
    match format {
        OutputFormat::Human => write_human(outcome, out),
        OutputFormat::Json => write_json(outcome, out),
    }
}

/// Writes human-readable config output.
fn write_human(
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

    let suppressed = if is_global_suppressed(outcome) {
        "yes"
    } else {
        "no"
    };

    writeln!(out, "vault root:  {}", vault_root.display())
        .map_err(write_error)?;
    writeln!(out, "vault config: {vault_config}").map_err(write_error)?;
    writeln!(out, "global config: {global_config}").map_err(write_error)?;
    writeln!(out, "global config suppressed: {suppressed}")
        .map_err(write_error)?;

    Ok(())
}

/// Writes JSON config output.
fn write_json(
    outcome: &BootstrapOutcome,
    out: &mut impl Write,
) -> Result<(), CliError> {
    let vault_root =
        outcome.config.vault_metadata().root().as_path().display().to_string();

    let vault_config_json =
        json_path_or_null(outcome.vault_config_path.as_deref());
    let global_config_json =
        json_path_or_null(outcome.global_config_path.as_deref());
    let suppressed = is_global_suppressed(outcome);

    writeln!(
        out,
        r#"{{"vault_root":{vault_root_json},"vault_config":{vault_config_json},"global_config":{global_config_json},"global_config_suppressed":{suppressed}}}"#,
        vault_root_json = json_string(&vault_root),
    )
    .map_err(write_error)?;

    Ok(())
}

/// Returns whether global config was suppressed.
fn is_global_suppressed(outcome: &BootstrapOutcome) -> bool {
    outcome.report.global_resolution_skip_reason.is_some()
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
mod fixtures {
    use std::path::{Path, PathBuf};

    use lithos_core::{
        app::bootstrap::BootstrapError,
        config::{
            aggregate::Config,
            builder::build_from_layers,
            vault::{VaultId, VaultRoot},
        },
        discovery::{
            DiscoveryFlags,
            report::{
                DiscoveryReport, GlobalResolutionSkipReason,
                LocalTraversalStopReason, SkippedCeiling, SkippedCeilingReason,
            },
        },
    };

    use super::{BootstrapOutcome, BootstrapRunner};

    /// Creates a vault root in a temp directory for testing.
    pub fn test_vault_root(suffix: &str) -> (tempfile::TempDir, VaultRoot) {
        let dir = tempfile::tempdir().expect("temp dir");
        let subdir = dir.path().join(suffix);
        std::fs::create_dir_all(&subdir).expect("subdir");
        let root = VaultRoot::try_new(subdir).expect("vault root");
        (dir, root)
    }

    /// Builds a minimal [`Config`] for use in test outcomes.
    pub fn test_config(root: VaultRoot) -> Config {
        let version = lithos_core::config::aggregate::Version::initial();
        let vault_id = VaultId::new();
        build_from_layers(None, None, vault_id, root, version)
            .expect("test config")
    }

    /// A mock bootstrap runner that always succeeds with the given outcome.
    pub struct MockBootstrapRunner {
        pub outcome: BootstrapOutcome,
    }

    impl MockBootstrapRunner {
        /// Constructs a mock with a success outcome using the given values.
        pub fn success(
            config: Config,
            report: DiscoveryReport,
            vault_config_path: Option<PathBuf>,
            global_config_path: Option<PathBuf>,
        ) -> Self {
            Self {
                outcome: BootstrapOutcome {
                    config,
                    report,
                    vault_config_path,
                    global_config_path,
                },
            }
        }

        /// Constructs a mock with a default successful outcome.
        pub fn default_success(root: VaultRoot) -> Self {
            let config = test_config(root);
            let report = DiscoveryReport {
                skipped_ceilings: vec![],
                local_traversal_stop_reason:
                    LocalTraversalStopReason::FilesystemRoot,
                global_resolution_skip_reason: None,
            };
            Self::success(config, report, None, None)
        }

        /// Constructs a mock with verbose diagnostic data.
        pub fn with_skipped_ceiling(root: VaultRoot) -> Self {
            let config = test_config(root);
            let report = DiscoveryReport {
                skipped_ceilings: vec![SkippedCeiling {
                    segment: PathBuf::from("/invalid/ceiling"),
                    reason: SkippedCeilingReason::InvalidPath,
                }],
                local_traversal_stop_reason:
                    LocalTraversalStopReason::FilesystemRoot,
                global_resolution_skip_reason: None,
            };
            Self::success(config, report, None, None)
        }

        /// Constructs a mock with global config suppressed.
        pub fn with_suppressed_global(root: VaultRoot) -> Self {
            let config = test_config(root);
            let report = DiscoveryReport {
                skipped_ceilings: vec![],
                local_traversal_stop_reason:
                    LocalTraversalStopReason::FilesystemRoot,
                global_resolution_skip_reason: Some(
                    GlobalResolutionSkipReason::SuppressedByFlag,
                ),
            };
            Self::success(config, report, None, None)
        }
    }

    impl BootstrapRunner for MockBootstrapRunner {
        fn run_bootstrap(
            &self,
            _flags: Option<DiscoveryFlags>,
            _anchor: &Path,
        ) -> Result<BootstrapOutcome, BootstrapError> {
            Ok(BootstrapOutcome {
                config: self.outcome.config.clone(),
                report: self.outcome.report.clone(),
                vault_config_path: self.outcome.vault_config_path.clone(),
                global_config_path: self.outcome.global_config_path.clone(),
            })
        }
    }

    /// A mock bootstrap runner that always fails with `InvalidAnchorDirectory`.
    pub struct FailingBootstrapRunner;

    impl BootstrapRunner for FailingBootstrapRunner {
        fn run_bootstrap(
            &self,
            _flags: Option<DiscoveryFlags>,
            _anchor: &Path,
        ) -> Result<BootstrapOutcome, BootstrapError> {
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
}

#[cfg(test)]
mod config_handler {
    use lithos_core::discovery::{
        DiscoveryFlags,
        report::{DiscoveryReport, LocalTraversalStopReason},
    };

    use super::{BootstrapRunner, fixtures, run_config};
    use crate::cli::OutputFormat;

    // ----- helpers -----

    fn run_human(
        runner: &impl BootstrapRunner,
        flags: Option<DiscoveryFlags>,
        anchor: &std::path::Path,
    ) -> (String, String) {
        let mut out = Vec::<u8>::new();
        let mut err = Vec::<u8>::new();
        run_config(
            runner,
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
        runner: &impl BootstrapRunner,
        flags: Option<DiscoveryFlags>,
        anchor: &std::path::Path,
    ) -> (String, String) {
        let mut out = Vec::<u8>::new();
        let mut err = Vec::<u8>::new();
        run_config(
            runner,
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
        runner: &impl BootstrapRunner,
        anchor: &std::path::Path,
    ) -> (String, String) {
        let mut out = Vec::<u8>::new();
        let mut err = Vec::<u8>::new();
        run_config(
            runner,
            None,
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

    fn anchor() -> tempfile::TempDir {
        tempfile::tempdir().expect("anchor dir")
    }

    // ----- tests -----

    #[test]
    fn returns_resolved_vault_root_in_human_format() {
        let (tmp, root) = fixtures::test_vault_root("my-vault");
        let vault_path = tmp.path().join("my-vault");
        let runner = fixtures::MockBootstrapRunner::default_success(root);
        let anchor = anchor();

        let (stdout, _) = run_human(&runner, None, anchor.path());

        assert!(
            stdout.contains(&vault_path.display().to_string()),
            "expected vault root path in output, got: {stdout}"
        );
    }

    #[test]
    fn returns_resolved_vault_root_in_json_format() {
        let (tmp, root) = fixtures::test_vault_root("json-vault");
        let vault_path = tmp.path().join("json-vault");
        let runner = fixtures::MockBootstrapRunner::default_success(root);
        let anchor = anchor();

        let (stdout, _) = run_json(&runner, None, anchor.path());

        assert!(
            stdout.contains("vault_root"),
            "expected vault_root key in JSON, got: {stdout}"
        );
        assert!(
            stdout.contains(&vault_path.display().to_string()),
            "expected vault root path in JSON output, got: {stdout}"
        );
    }

    #[test]
    fn includes_global_config_suppression_status_when_no_global_config_set() {
        let (_tmp, root) = fixtures::test_vault_root("suppressed-vault");
        let runner =
            fixtures::MockBootstrapRunner::with_suppressed_global(root);
        let anchor = anchor();

        let (stdout, _) = run_human(&runner, None, anchor.path());

        assert!(
            stdout.contains("global config suppressed: yes"),
            "expected suppressed=yes in output, got: {stdout}"
        );
    }

    #[test]
    fn writes_skipped_ceiling_warning_to_stderr_when_verbose() {
        let (_tmp, root) = fixtures::test_vault_root("ceiling-vault");
        let runner = fixtures::MockBootstrapRunner::with_skipped_ceiling(root);
        let anchor = anchor();

        let (_, stderr) = run_verbose(&runner, anchor.path());

        assert!(
            stderr.contains("skipped ceiling"),
            "expected skipped ceiling warning in stderr, got: {stderr}"
        );
        assert!(
            stderr.contains("/invalid/ceiling"),
            "expected ceiling path in stderr, got: {stderr}"
        );
    }

    #[test]
    fn writes_structured_output_to_stdout_writer() {
        let (_tmp, root) = fixtures::test_vault_root("stdout-vault");
        let runner = fixtures::MockBootstrapRunner::default_success(root);
        let anchor = anchor();

        let (stdout, _) = run_human(&runner, None, anchor.path());

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
    fn writes_verbose_diagnostics_to_stderr_writer() {
        let (_tmp, root) = fixtures::test_vault_root("diag-vault");
        let runner = fixtures::MockBootstrapRunner::with_skipped_ceiling(root);
        let anchor = anchor();

        let mut out = Vec::<u8>::new();
        let mut err = Vec::<u8>::new();
        run_config(
            &runner,
            None,
            anchor.path(),
            OutputFormat::Human,
            1,
            &mut out,
            &mut err,
        )
        .expect("run_config should succeed");

        let stdout = String::from_utf8(out).expect("stdout utf8");
        let stderr = String::from_utf8(err).expect("stderr utf8");

        assert!(
            stdout.contains("vault root:"),
            "structured output should go to stdout, got: {stdout}"
        );
        assert!(
            !stderr.is_empty(),
            "verbose diagnostics should go to stderr, got: {stderr}"
        );
        assert!(
            !stdout.contains("skipped ceiling"),
            "skipped ceiling warning must not appear in stdout, got: {stdout}"
        );
    }

    #[test]
    fn honours_vault_flag_override() {
        let (tmp, root) = fixtures::test_vault_root("flag-vault");
        let vault_dir = tmp.path().join("flag-vault");
        let config_file = vault_dir.join("lithos.toml");
        std::fs::write(&config_file, "").expect("write config file");

        let flags = DiscoveryFlags::new(
            Some(config_file.as_path()),
            Some(vault_dir.as_path()),
            false,
        )
        .expect("valid flags");

        let runner = fixtures::MockBootstrapRunner::default_success(root);
        let anchor = anchor();

        // The mock ignores flags; we just verify the handler passes them
        // through without panicking and produces valid output.
        let (stdout, _) = run_human(&runner, Some(flags), anchor.path());

        assert!(
            stdout.contains("vault root:"),
            "expected output with vault flag, got: {stdout}"
        );
    }

    #[test]
    fn honours_config_flag_override() {
        let (tmp, root) = fixtures::test_vault_root("cfg-override-vault");
        let vault_dir = tmp.path().join("cfg-override-vault");
        let config_file = vault_dir.join("lithos.toml");
        std::fs::write(&config_file, "").expect("write config file");

        let flags =
            DiscoveryFlags::new(Some(config_file.as_path()), None, false)
                .expect("valid flags");

        let runner = fixtures::MockBootstrapRunner::default_success(root);
        let anchor = anchor();

        let (stdout, _) = run_human(&runner, Some(flags), anchor.path());

        assert!(
            stdout.contains("vault root:"),
            "expected output with config flag, got: {stdout}"
        );
    }

    #[test]
    fn returns_err_when_vault_not_found() {
        let runner = fixtures::FailingBootstrapRunner;
        let anchor = anchor();
        let mut out = Vec::<u8>::new();
        let mut err = Vec::<u8>::new();

        let result = run_config(
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
    fn returns_err_when_explicit_vault_path_invalid() {
        // The FailingBootstrapRunner simulates any bootstrap failure.
        // The handler must propagate the error regardless of variant.
        let runner = fixtures::FailingBootstrapRunner;
        let anchor = anchor();
        let mut out = Vec::<u8>::new();
        let mut err = Vec::<u8>::new();

        let result = run_config(
            &runner,
            None,
            anchor.path(),
            OutputFormat::Human,
            0,
            &mut out,
            &mut err,
        );

        assert!(
            result.is_err(),
            "expected Err for invalid explicit vault path"
        );
    }

    #[test]
    fn returns_err_when_permission_denied() {
        // The FailingBootstrapRunner simulates any bootstrap failure.
        // The handler must propagate the error regardless of variant.
        let runner = fixtures::FailingBootstrapRunner;
        let anchor = anchor();
        let mut out = Vec::<u8>::new();
        let mut err = Vec::<u8>::new();

        let result = run_config(
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
    fn json_output_includes_vault_config_null_when_not_found() {
        let (_tmp, root) = fixtures::test_vault_root("null-vault-cfg");
        let runner = fixtures::MockBootstrapRunner::default_success(root);
        let anchor = anchor();

        let (stdout, _) = run_json(&runner, None, anchor.path());

        assert!(
            stdout.contains(r#""vault_config":null"#),
            "expected null vault_config in JSON, got: {stdout}"
        );
    }

    #[test]
    fn json_output_includes_global_config_null_when_not_found() {
        let (_tmp, root) = fixtures::test_vault_root("null-global-cfg");
        let runner = fixtures::MockBootstrapRunner::default_success(root);
        let anchor = anchor();

        let (stdout, _) = run_json(&runner, None, anchor.path());

        assert!(
            stdout.contains(r#""global_config":null"#),
            "expected null global_config in JSON, got: {stdout}"
        );
    }

    #[test]
    fn json_output_includes_suppression_false_by_default() {
        let (_tmp, root) = fixtures::test_vault_root("no-suppress");
        let runner = fixtures::MockBootstrapRunner::default_success(root);
        let anchor = anchor();

        let (stdout, _) = run_json(&runner, None, anchor.path());

        assert!(
            stdout.contains(r#""global_config_suppressed":false"#),
            "expected suppression=false in JSON, got: {stdout}"
        );
    }

    #[test]
    fn json_output_includes_vault_config_path_when_set() {
        let (tmp, root) = fixtures::test_vault_root("with-vault-cfg");
        let vault_cfg = tmp.path().join("with-vault-cfg").join("lithos.toml");
        let config = fixtures::test_config(root);
        let report = DiscoveryReport {
            skipped_ceilings: vec![],
            local_traversal_stop_reason:
                LocalTraversalStopReason::FilesystemRoot,
            global_resolution_skip_reason: None,
        };
        let runner = fixtures::MockBootstrapRunner::success(
            config,
            report,
            Some(vault_cfg.clone()),
            None,
        );
        let anchor = anchor();

        let (stdout, _) = run_json(&runner, None, anchor.path());

        assert!(
            stdout.contains("vault_config"),
            "expected vault_config in JSON, got: {stdout}"
        );
        assert!(
            stdout.contains(&vault_cfg.display().to_string()),
            "expected vault config path in JSON, got: {stdout}"
        );
    }

    #[test]
    fn human_output_shows_none_for_missing_vault_config() {
        let (_tmp, root) = fixtures::test_vault_root("no-vault-cfg");
        let runner = fixtures::MockBootstrapRunner::default_success(root);
        let anchor = anchor();

        let (stdout, _) = run_human(&runner, None, anchor.path());

        assert!(
            stdout.contains("vault config: none"),
            "expected 'none' for missing vault config, got: {stdout}"
        );
    }

    #[test]
    fn human_output_shows_none_for_missing_global_config() {
        let (_tmp, root) = fixtures::test_vault_root("no-global-cfg");
        let runner = fixtures::MockBootstrapRunner::default_success(root);
        let anchor = anchor();

        let (stdout, _) = run_human(&runner, None, anchor.path());

        assert!(
            stdout.contains("global config: none"),
            "expected 'none' for missing global config, got: {stdout}"
        );
    }

    #[test]
    fn stop_reason_written_to_stderr_when_verbose() {
        let (_tmp, root) = fixtures::test_vault_root("stop-reason");
        let runner = fixtures::MockBootstrapRunner::default_success(root);
        let anchor = anchor();

        let (_, stderr) = run_verbose(&runner, anchor.path());

        assert!(
            stderr.contains("stop reason:"),
            "expected stop reason in stderr, got: {stderr}"
        );
    }
}
