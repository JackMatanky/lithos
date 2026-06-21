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

use std::{io::Write, path::PathBuf};

use lithos_core::{
    app::bootstrap::Bootstrapper,
    discovery::{DiscoveryFlags, port::DiscoveryPort},
};

use crate::{cli::OutputFormat, error::CliError, output};

// ----------------------------------------------------------- //
//                       Command Handler                       //
// ----------------------------------------------------------- //

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
pub(crate) fn run_config_files<D: DiscoveryPort>(
    bootstrapper: &Bootstrapper<D>,
    flags: Option<DiscoveryFlags>,
    anchor: &std::path::Path,
    format: OutputFormat,
    out: &mut impl Write,
) -> Result<(), CliError> {
    let discovery_result =
        match bootstrapper.run_discovery_only(flags, None, anchor) {
            Ok((result, _report)) => Some(result),
            Err(_) => None,
        };

    let (vault_candidates, global_candidates) = match discovery_result {
        Some(result) => {
            let vault_owned: Vec<PathBuf> = result
                .vault()
                .iter()
                .map(|c| c.path().as_path().to_path_buf())
                .collect();
            let global_owned: Vec<PathBuf> = result
                .global()
                .iter()
                .map(|c| c.path().as_path().to_path_buf())
                .collect();
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

// ----------------------------------------------------------- //
//                      Output Formatting                      //
// ----------------------------------------------------------- //

/// Writes human-readable candidate list output.
fn write_human(
    vault: &[PathBuf],
    global: &[PathBuf],
    out: &mut impl Write,
) -> Result<(), CliError> {
    if vault.is_empty() && global.is_empty() {
        writeln!(out, "(no candidates found)").map_err(output::stdout_err)?;
        return Ok(());
    }

    writeln!(out, "vault candidates:").map_err(output::stdout_err)?;
    for path in vault {
        writeln!(out, "  {}", path.display()).map_err(output::stdout_err)?;
    }

    writeln!(out, "global candidates:").map_err(output::stdout_err)?;
    for path in global {
        writeln!(out, "  {}", path.display()).map_err(output::stdout_err)?;
    }

    Ok(())
}

/// Serialisable representation of the `config files` command's JSON output.
#[derive(serde::Serialize)]
struct ConfigFilesOutput<'a> {
    /// Discovered vault config file candidates.
    vault: Vec<&'a str>,
    /// Discovered global config file candidates.
    global: Vec<&'a str>,
}

/// Writes JSON candidate list output using [`serde_json`].
fn write_json(
    vault: &[PathBuf],
    global: &[PathBuf],
    out: &mut impl Write,
) -> Result<(), CliError> {
    let vault_strs: Vec<&str> =
        vault.iter().map(|p| p.to_str().unwrap_or("")).collect();
    let global_strs: Vec<&str> =
        global.iter().map(|p| p.to_str().unwrap_or("")).collect();
    let payload = ConfigFilesOutput {
        vault: vault_strs,
        global: global_strs,
    };
    output::write_json_line(out, &payload)
}

// ----------------------------------------------------------- //
//                            Tests                            //
// ----------------------------------------------------------- //

#[cfg(test)]
mod config_files_handler {
    use lithos_core::{
        app::bootstrap::Bootstrapper,
        discovery::{DiscoveryFlags, service::DiscoveryService},
    };

    use super::run_config_files;
    use crate::cli::OutputFormat;

    /// Creates a temp directory with a minimal vault and returns a
    /// `Bootstrapper` plus `DiscoveryFlags` that point at it explicitly.
    fn make_vault()
    -> (tempfile::TempDir, Bootstrapper<DiscoveryService>, DiscoveryFlags) {
        let dir = tempfile::tempdir().expect("vault dir");
        let config_path = dir.path().join("lithos.toml");
        std::fs::write(&config_path, "").expect("write lithos.toml");
        let flags = DiscoveryFlags::new(
            Some(config_path.as_path()),
            Some(dir.path()),
            true, // suppress global — no accidental global reads in tests
        )
        .expect("valid flags");
        let bootstrapper = Bootstrapper::with_global_directories(vec![])
            .expect("bootstrapper");
        (dir, bootstrapper, flags)
    }

    /// Creates an empty temp directory (no vault) and a `Bootstrapper`.
    fn make_empty() -> (tempfile::TempDir, Bootstrapper<DiscoveryService>) {
        let dir = tempfile::tempdir().expect("temp dir");
        let bootstrapper = Bootstrapper::with_global_directories(vec![])
            .expect("bootstrapper");
        (dir, bootstrapper)
    }

    fn run_human(
        bootstrapper: &Bootstrapper<DiscoveryService>,
        flags: Option<DiscoveryFlags>,
        anchor: &std::path::Path,
    ) -> String {
        let mut out = Vec::<u8>::new();
        run_config_files(
            bootstrapper,
            flags,
            anchor,
            OutputFormat::Human,
            &mut out,
        )
        .expect("always Ok");
        String::from_utf8(out).expect("utf8")
    }

    fn run_json(
        bootstrapper: &Bootstrapper<DiscoveryService>,
        flags: Option<DiscoveryFlags>,
        anchor: &std::path::Path,
    ) -> String {
        let mut out = Vec::<u8>::new();
        run_config_files(
            bootstrapper,
            flags,
            anchor,
            OutputFormat::Json,
            &mut out,
        )
        .expect("always Ok");
        String::from_utf8(out).expect("utf8")
    }

    // ----- tests -----

    #[test]
    fn run_config_files_lists_vault_candidates() {
        let (dir, bootstrapper, flags) = make_vault();
        let config_path = dir.path().join("lithos.toml");

        let output = run_human(&bootstrapper, Some(flags), dir.path());

        assert!(
            output.contains("vault candidates:"),
            "expected vault candidates section in output, got: {output}"
        );
        assert!(
            output.contains(&config_path.display().to_string()),
            "expected vault config path in output, got: {output}"
        );
    }

    #[test]
    fn run_config_files_returns_ok_when_no_vault_found() {
        // Empty directory — discovery finds nothing, but must still return Ok.
        let (dir, bootstrapper) = make_empty();
        let mut out = Vec::<u8>::new();

        let result = run_config_files(
            &bootstrapper,
            None,
            dir.path(),
            OutputFormat::Human,
            &mut out,
        );

        assert!(result.is_ok(), "expected Ok when no vault found: {result:?}");
    }

    #[test]
    fn returns_empty_output_when_no_candidates_found() {
        let (dir, bootstrapper) = make_empty();

        let output = run_human(&bootstrapper, None, dir.path());

        assert!(
            output.contains("(no candidates found)"),
            "expected no-candidates message, got: {output}"
        );
    }

    #[test]
    fn returns_candidates_in_json_format() {
        let (dir, bootstrapper, flags) = make_vault();
        let config_path = dir.path().join("lithos.toml");

        let output = run_json(&bootstrapper, Some(flags), dir.path());

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
            output.contains(&config_path.display().to_string()),
            "expected vault path in JSON, got: {output}"
        );
    }

    #[test]
    fn always_returns_ok_when_discovery_error_occurs() {
        // Use a non-existent anchor to force a discovery error.
        let bootstrapper = Bootstrapper::with_global_directories(vec![])
            .expect("bootstrapper");
        let bad_anchor =
            std::path::Path::new("/this/path/does/not/exist/at/all");
        let mut out = Vec::<u8>::new();

        let result = run_config_files(
            &bootstrapper,
            None,
            bad_anchor,
            OutputFormat::Human,
            &mut out,
        );

        assert!(
            result.is_ok(),
            "expected Ok even when discovery fails: {result:?}"
        );
    }
}
