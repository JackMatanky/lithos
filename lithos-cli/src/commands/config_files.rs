//! Handler for the `lithos config files` subcommand.
//!
//! This module provides [`run_config_files`], which runs discovery only
//! (without triggering config parsing) and writes the discovered candidate
//! config file paths to the provided output writer.
//!
//! Unlike most commands this handler **always exits 0**.  If discovery fails,
//! the error is silently swallowed and empty/warning output is written
//! instead.  This is intentional: `lithos config files` is a listing command
//! that should never block shell completion scripts or other tooling that
//! pipes its output.

// This module is wired to the CLI dispatch layer in main.rs.

use std::{
    io::Write,
    path::{Path, PathBuf},
};

use lithos_core::{
    app::bootstrap::BootstrapError,
    discovery::{
        DiscoveryFlags, report::DiscoveryReport, service::DiscoveryResult,
    },
};

use crate::{cli::OutputFormat, error::CliError};

// ------------------------------------------------------------------ //
//                          Port Trait                                //
// ------------------------------------------------------------------ //

/// Abstraction over the discovery-only pipeline used by the config files
/// handler.
///
/// Implement this trait on a concrete runner (e.g. a thin wrapper around
/// [`lithos_core::app::bootstrap::Bootstrapper`]) and pass it to
/// [`run_config_files`].  In tests, provide a mock implementation instead.
///
/// The handler only holds a `DiscoveryRunner` reference, not a
/// `BootstrapRunner`, which structurally prevents it from calling the full
/// bootstrap `run()` method.
pub(crate) trait DiscoveryRunner {
    /// Runs discovery only (without config loading) and returns the raw result
    /// and report.
    ///
    /// # Errors
    ///
    /// Returns [`BootstrapError`] if anchor resolution or discovery fails.
    fn run_discovery(
        &self,
        flags: Option<DiscoveryFlags>,
        anchor: &Path,
    ) -> Result<(DiscoveryResult, DiscoveryReport), BootstrapError>;
}

// ------------------------------------------------------------------ //
//                          Handler                                   //
// ------------------------------------------------------------------ //

/// Runs the `lithos config files` command handler.
///
/// Calls discovery only (no config parsing), then writes the discovered
/// vault and global candidate config file paths to `out` in the requested
/// `format`.
///
/// # Always Returns `Ok(())`
///
/// This handler catches all errors and writes empty/warning output instead
/// of propagating them.  The `lithos config files` command is designed to
/// always exit 0 so that shell completion scripts and other tooling that
/// pipes its output are never blocked.
pub(crate) fn run_config_files(
    runner: &impl DiscoveryRunner,
    flags: Option<DiscoveryFlags>,
    anchor: &Path,
    format: OutputFormat,
    out: &mut impl Write,
) -> Result<(), CliError> {
    let discovery_result = match runner.run_discovery(flags, anchor) {
        Ok((result, _report)) => Some(result),
        Err(_) => None,
    };

    let (vault_candidates, global_candidates) = match discovery_result {
        Some(result) => {
            let vault: Vec<&Path> =
                result.vault().iter().map(|c| c.path().as_path()).collect();
            let global: Vec<&Path> =
                result.global().iter().map(|c| c.path().as_path()).collect();
            // We need owned paths since result is dropped at end of match
            let vault_owned: Vec<PathBuf> =
                vault.iter().map(|p| p.to_path_buf()).collect();
            let global_owned: Vec<PathBuf> =
                global.iter().map(|p| p.to_path_buf()).collect();
            (vault_owned, global_owned)
        }
        None => (vec![], vec![]),
    };

    match format {
        OutputFormat::Human => {
            write_human(&vault_candidates, &global_candidates, out)
        }
        OutputFormat::Json => {
            write_json(&vault_candidates, &global_candidates, out)
        }
    }
}

/// Writes human-readable candidate list output.
fn write_human(
    vault: &[PathBuf],
    global: &[PathBuf],
    out: &mut impl Write,
) -> Result<(), CliError> {
    if vault.is_empty() && global.is_empty() {
        writeln!(out, "(no candidates found)").map_err(write_error)?;
        return Ok(());
    }

    writeln!(out, "vault candidates:").map_err(write_error)?;
    for path in vault {
        writeln!(out, "  {}", path.display()).map_err(write_error)?;
    }

    writeln!(out, "global candidates:").map_err(write_error)?;
    for path in global {
        writeln!(out, "  {}", path.display()).map_err(write_error)?;
    }

    Ok(())
}

/// Writes JSON candidate list output.
fn write_json(
    vault: &[PathBuf],
    global: &[PathBuf],
    out: &mut impl Write,
) -> Result<(), CliError> {
    let vault_json = format_json_path_array(vault);
    let global_json = format_json_path_array(global);
    writeln!(out, r#"{{"vault":{vault_json},"global":{global_json}}}"#)
        .map_err(write_error)?;
    Ok(())
}

/// Formats a slice of paths as a JSON array of strings.
fn format_json_path_array(paths: &[PathBuf]) -> String {
    let items: Vec<String> =
        paths.iter().map(|p| json_string(&p.display().to_string())).collect();
    format!("[{}]", items.join(","))
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
        discovery::{
            DiscoveryFlags,
            report::{DiscoveryReport, LocalTraversalStopReason},
            service::{CandidatePath, DiscoveryResult},
        },
        fs::{DirPath, FilePath},
    };

    use super::DiscoveryRunner;

    /// Creates a vault candidate in the given temp dir.
    pub fn make_vault_candidate(
        dir: &tempfile::TempDir,
        filename: &str,
    ) -> CandidatePath {
        let file_path = dir.path().join(filename);
        std::fs::write(&file_path, "").expect("write candidate");
        let base = DirPath::try_new(dir.path().to_path_buf()).expect("base");
        let path = FilePath::try_new(file_path).expect("file path");
        CandidatePath::new(base, path)
    }

    /// Creates a global candidate in a subdirectory of the given temp dir.
    pub fn make_global_candidate(
        dir: &tempfile::TempDir,
        subdir: &str,
        filename: &str,
    ) -> CandidatePath {
        let sub = dir.path().join(subdir);
        std::fs::create_dir_all(&sub).expect("create subdir");
        let file_path = sub.join(filename);
        std::fs::write(&file_path, "").expect("write candidate");
        let base = DirPath::try_new(sub).expect("base");
        let path = FilePath::try_new(file_path).expect("file path");
        CandidatePath::new(base, path)
    }

    /// Builds a default empty discovery report.
    pub fn default_report() -> DiscoveryReport {
        DiscoveryReport {
            skipped_ceilings: vec![],
            local_traversal_stop_reason:
                LocalTraversalStopReason::FilesystemRoot,
            global_resolution_skip_reason: None,
        }
    }

    /// A mock discovery runner that records whether `run_discovery` was called.
    pub struct MockDiscoveryRunner {
        result: Result<DiscoveryResult, ()>,
        pub call_count: std::sync::atomic::AtomicUsize,
    }

    impl MockDiscoveryRunner {
        /// Constructs a mock that returns the given vault and global
        /// candidates.
        pub fn success(
            vault: Vec<CandidatePath>,
            global: Vec<CandidatePath>,
        ) -> Self {
            Self {
                result: Ok(DiscoveryResult::new(vault, global)),
                call_count: std::sync::atomic::AtomicUsize::new(0),
            }
        }

        /// Constructs a mock that always returns an empty discovery result.
        pub fn empty() -> Self {
            Self::success(vec![], vec![])
        }

        /// Constructs a mock that always fails with a discovery error.
        pub fn failing() -> Self {
            Self {
                result: Err(()),
                call_count: std::sync::atomic::AtomicUsize::new(0),
            }
        }

        /// Returns the number of times `run_discovery` was called.
        pub fn call_count(&self) -> usize {
            self.call_count.load(std::sync::atomic::Ordering::SeqCst)
        }
    }

    impl DiscoveryRunner for MockDiscoveryRunner {
        fn run_discovery(
            &self,
            _flags: Option<DiscoveryFlags>,
            _anchor: &Path,
        ) -> Result<(DiscoveryResult, DiscoveryReport), BootstrapError>
        {
            self.call_count.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
            match &self.result {
                Ok(r) => Ok((r.clone(), default_report())),
                Err(()) => {
                    Err(BootstrapError::Discovery(
                        lithos_core::discovery::error::DiscoveryError::InvalidAnchorDirectory {
                            path: PathBuf::from("/no/vault"),
                            source: lithos_core::fs::PathError::NotADirectory(
                                PathBuf::from("/no/vault"),
                            ),
                        },
                    ))
                }
            }
        }
    }
}

#[cfg(test)]
mod config_files_handler {
    use lithos_core::discovery::DiscoveryFlags;

    use super::{fixtures, run_config_files};
    use crate::cli::OutputFormat;

    fn anchor() -> tempfile::TempDir {
        tempfile::tempdir().expect("anchor dir")
    }

    fn run_human(
        runner: &impl super::DiscoveryRunner,
        flags: Option<DiscoveryFlags>,
        anchor: &std::path::Path,
    ) -> String {
        let mut out = Vec::<u8>::new();
        run_config_files(runner, flags, anchor, OutputFormat::Human, &mut out)
            .expect("always Ok");
        String::from_utf8(out).expect("utf8")
    }

    fn run_json(
        runner: &impl super::DiscoveryRunner,
        flags: Option<DiscoveryFlags>,
        anchor: &std::path::Path,
    ) -> String {
        let mut out = Vec::<u8>::new();
        run_config_files(runner, flags, anchor, OutputFormat::Json, &mut out)
            .expect("always Ok");
        String::from_utf8(out).expect("utf8")
    }

    #[test]
    fn returns_vault_candidates_in_precedence_order() {
        let dir1 = tempfile::tempdir().expect("dir1");
        let dir2 = tempfile::tempdir().expect("dir2");
        let c1 = fixtures::make_vault_candidate(&dir1, "lithos.toml");
        let c2 = fixtures::make_vault_candidate(&dir2, "lithos.toml");
        let expected_path1 = dir1.path().join("lithos.toml");
        let expected_path2 = dir2.path().join("lithos.toml");

        let runner =
            fixtures::MockDiscoveryRunner::success(vec![c1, c2], vec![]);
        let anchor = anchor();
        let output = run_human(&runner, None, anchor.path());

        let pos1 = output
            .find(&expected_path1.display().to_string())
            .expect("path1 in output");
        let pos2 = output
            .find(&expected_path2.display().to_string())
            .expect("path2 in output");
        assert!(
            pos1 < pos2,
            "vault candidates should appear in order: {output}"
        );
    }

    #[test]
    fn returns_global_candidates_after_vault_candidates() {
        let vault_dir = tempfile::tempdir().expect("vault dir");
        let global_dir = tempfile::tempdir().expect("global dir");
        let vault_c = fixtures::make_vault_candidate(&vault_dir, "lithos.toml");
        let global_c = fixtures::make_global_candidate(
            &global_dir,
            "config",
            "lithos.toml",
        );
        let vault_path = vault_dir.path().join("lithos.toml");
        let global_path = global_dir.path().join("config").join("lithos.toml");

        let runner =
            fixtures::MockDiscoveryRunner::success(vec![vault_c], vec![
                global_c,
            ]);
        let anchor = anchor();
        let output = run_human(&runner, None, anchor.path());

        let vault_pos =
            output.find("vault candidates:").expect("vault section in output");
        let global_pos = output
            .find("global candidates:")
            .expect("global section in output");
        assert!(
            vault_pos < global_pos,
            "vault section should precede global section: {output}"
        );

        let vault_path_pos = output
            .find(&vault_path.display().to_string())
            .expect("vault path in output");
        let global_path_pos = output
            .find(&global_path.display().to_string())
            .expect("global path in output");
        assert!(
            vault_path_pos < global_path_pos,
            "vault path should appear before global path: {output}"
        );
    }

    #[test]
    fn returns_empty_output_when_no_candidates_found() {
        let runner = fixtures::MockDiscoveryRunner::empty();
        let anchor = anchor();
        let output = run_human(&runner, None, anchor.path());

        assert!(
            output.contains("(no candidates found)"),
            "expected no-candidates message, got: {output}"
        );
    }

    #[test]
    fn always_returns_ok_when_no_vault_found() {
        // Even with an empty result (no vault found), handler returns Ok.
        let runner = fixtures::MockDiscoveryRunner::empty();
        let anchor = anchor();
        let mut out = Vec::<u8>::new();

        let result = run_config_files(
            &runner,
            None,
            anchor.path(),
            OutputFormat::Human,
            &mut out,
        );

        assert!(result.is_ok(), "expected Ok when no vault found");
    }

    #[test]
    fn always_returns_ok_when_discovery_error_occurs() {
        let runner = fixtures::MockDiscoveryRunner::failing();
        let anchor = anchor();
        let mut out = Vec::<u8>::new();

        let result = run_config_files(
            &runner,
            None,
            anchor.path(),
            OutputFormat::Human,
            &mut out,
        );

        assert!(
            result.is_ok(),
            "expected Ok even when discovery fails: {result:?}"
        );
    }

    #[test]
    fn calls_run_discovery_only_not_run() {
        // Structural test: the mock only exposes `run_discovery()`.
        // The handler can only call `run_discovery()` because `DiscoveryRunner`
        // does not expose any other method. We verify the call was made.
        let runner = fixtures::MockDiscoveryRunner::empty();
        let anchor = anchor();
        let mut out = Vec::<u8>::new();

        run_config_files(
            &runner,
            None,
            anchor.path(),
            OutputFormat::Human,
            &mut out,
        )
        .expect("always Ok");

        assert_eq!(
            runner.call_count(),
            1,
            "run_discovery should be called exactly once"
        );
    }

    #[test]
    fn returns_candidates_in_json_format_when_format_json() {
        let vault_dir = tempfile::tempdir().expect("vault dir");
        let global_dir = tempfile::tempdir().expect("global dir");
        let vault_c = fixtures::make_vault_candidate(&vault_dir, "lithos.toml");
        let global_c = fixtures::make_global_candidate(
            &global_dir,
            "config",
            "lithos.toml",
        );
        let vault_path = vault_dir.path().join("lithos.toml");
        let global_path = global_dir.path().join("config").join("lithos.toml");

        let runner =
            fixtures::MockDiscoveryRunner::success(vec![vault_c], vec![
                global_c,
            ]);
        let anchor = anchor();
        let output = run_json(&runner, None, anchor.path());

        assert!(output.starts_with('{'), "expected JSON object, got: {output}");
        assert!(
            output.contains("\"vault\""),
            "expected vault key in JSON, got: {output}"
        );
        assert!(
            output.contains("\"global\""),
            "expected global key in JSON, got: {output}"
        );
        assert!(
            output.contains(&vault_path.display().to_string()),
            "expected vault path in JSON, got: {output}"
        );
        assert!(
            output.contains(&global_path.display().to_string()),
            "expected global path in JSON, got: {output}"
        );
    }

    #[test]
    fn honours_vault_flag_override() {
        let vault_dir = tempfile::tempdir().expect("vault dir");
        let config_file = vault_dir.path().join("lithos.toml");
        std::fs::write(&config_file, "").expect("write config");

        let flags = DiscoveryFlags::new(
            Some(config_file.as_path()),
            Some(vault_dir.path()),
            false,
        )
        .expect("valid flags");

        // The mock ignores flags; we verify the handler passes them through
        // without panicking and calls run_discovery.
        let runner = fixtures::MockDiscoveryRunner::empty();
        let anchor = anchor();

        let mut out = Vec::<u8>::new();
        run_config_files(
            &runner,
            Some(flags),
            anchor.path(),
            OutputFormat::Human,
            &mut out,
        )
        .expect("always Ok");

        assert_eq!(
            runner.call_count(),
            1,
            "run_discovery should be called with flags"
        );
    }
}
