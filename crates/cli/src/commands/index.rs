//! CLI handler for the `index` command.

use std::{io::Write, path::Path};

use traces_app::{
    bootstrap::BootstrapRunner,
    index::{
        IndexOptions, IndexScope, ScanFilters, run_index as app_run_index,
    },
};
use traces_settings::DiscoveryFlags;

use crate::{
    cli::{IndexArgs, OutputFormat},
    error::CliError,
};

/// Builds a domain [`IndexCommand`] from CLI [`IndexArgs`].
///
/// Converts `--path` from a vault-relative path to a [`DirPath`] for
/// [`IndexScope::Partial`], or defaults to [`IndexScope::Full`] when
/// `--path` is omitted.
///
/// # Errors
///
/// Returns [`CliError::InvalidPath`] if the provided `--path` cannot be
/// resolved to a valid directory path.
pub(crate) fn build_index_command(
    args: IndexArgs,
    vault_root: &traces_fs::DirPath,
) -> Result<traces_app::index::IndexCommand, CliError> {
    let scope = if let Some(rel_path) = args.path {
        let abs_path = vault_root.as_path().join(rel_path);
        let dir_path = traces_fs::DirPath::try_from(abs_path)
            .map_err(|e| CliError::InvalidPath(e.to_string()))?;
        IndexScope::Partial {
            root: dir_path,
            filters: ScanFilters::default(),
        }
    } else {
        IndexScope::Full {
            root: vault_root.clone(),
            filters: ScanFilters::default(),
        }
    };

    let opts = IndexOptions::new(args.rebuild, args.dry_run);

    Ok(traces_app::index::IndexCommand::new(scope, opts))
}

/// Executes the `index` subcommand.
///
/// Builds the index domain command from CLI arguments, executes the application
/// indexing logic, and formats the output.
///
/// # Errors
///
/// Returns [`CliError`] if building the command fails, configuration loading
/// fails, or the underlying index operation fails.
#[expect(
    clippy::too_many_arguments,
    reason = "handler signature matches the CLI dispatch protocol: \
              bootstrapper, discovery flags, anchor, output format, \
              verbosity, stdout, stderr"
)]
pub(crate) fn run_index(
    bootstrapper: &BootstrapRunner<
        impl traces_settings::discovery::port::DiscoveryPort,
    >,
    flags: Option<DiscoveryFlags>,
    anchor: &Path,
    args: IndexArgs,
    format: OutputFormat,
    _verbose: u8,
    out: &mut impl Write,
    _err: &mut impl Write,
) -> Result<(), CliError> {
    // 1. Resolve configuration
    let discovery = bootstrapper
        .run_discovery_only(flags, None, anchor)
        .map_err(CliError::Bootstrap)?;
    let vault_root = discovery
        .vault()
        .first()
        .map(|c| c.base().clone())
        .ok_or_else(|| {
            CliError::InvalidPath("No vault found during discovery".to_owned())
        })?;

    // 2. Build domain command
    let cmd = build_index_command(args, &vault_root)?;

    // 3. Execute
    let cache_dir_path = discovery.cache_root().path().to_path_buf();
    let cache_dir = traces_fs::DirPath::try_from(cache_dir_path)
        .map_err(|e| CliError::InvalidPath(e.to_string()))?;

    let result =
        app_run_index(&vault_root, &cache_dir, &cmd).map_err(|e| match e {
            traces_app::error::AppError::Indexer(idx_err) => {
                CliError::Index(crate::error::IndexCommandError::from(idx_err))
            }
            other => CliError::Bootstrap(other),
        })?;

    // 4. Write output
    let report = result.report();
    match format {
        OutputFormat::Human => write_report_human(report, out)?,
        OutputFormat::Json => write_report_json(report, out)?,
    }

    Ok(())
}

/// Writes a human-readable index report to `out`.
fn write_report_human(
    report: &traces_indexer::IndexReport,
    out: &mut impl Write,
) -> Result<(), CliError> {
    writeln!(out, "  scanned: {}", report.scanned())
        .map_err(crate::output::stdout_err)?;
    writeln!(out, "      new: {}", report.new_count())
        .map_err(crate::output::stdout_err)?;
    writeln!(out, "    fresh: {}", report.fresh_count())
        .map_err(crate::output::stdout_err)?;
    writeln!(out, "    stale: {}", report.stale_count())
        .map_err(crate::output::stdout_err)?;
    writeln!(out, "  deleted: {}", report.deleted_count())
        .map_err(crate::output::stdout_err)?;
    writeln!(out, "   failed: {}", report.failures().len())
        .map_err(crate::output::stdout_err)?;
    Ok(())
}

/// Serializable payload for JSON index reports.
#[derive(serde::Serialize)]
struct IndexReportPayload {
    scanned: usize,
    new: usize,
    fresh: usize,
    stale: usize,
    deleted: usize,
    failed: usize,
}

/// Writes a JSON index report to `out`.
fn write_report_json(
    report: &traces_indexer::IndexReport,
    out: &mut impl Write,
) -> Result<(), CliError> {
    let payload = IndexReportPayload {
        scanned: report.scanned(),
        new: report.new_count(),
        fresh: report.fresh_count(),
        stale: report.stale_count(),
        deleted: report.deleted_count(),
        failed: report.failures().len(),
    };
    crate::output::write_json_line(out, &payload)
}

// ----------------------------------------------------------- //
//                            Tests                            //
// ----------------------------------------------------------- //

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use traces_app::index::{IndexOptions, IndexScope, ScanFilters};
    use traces_fs::DirPath;
    use traces_indexer::{IndexNodeFailure, IndexReport};

    use super::{build_index_command, write_report_human, write_report_json};
    use crate::{cli::IndexArgs, error::CliError};

    #[expect(
        clippy::too_many_arguments,
        reason = "test helper: one counter per IndexReport field"
    )]
    fn make_report(
        scanned: usize,
        new: usize,
        fresh: usize,
        stale: usize,
        deleted: usize,
        failed: usize,
    ) -> IndexReport {
        let failures: Vec<IndexNodeFailure> = (0..failed)
            .map(|i| {
                IndexNodeFailure::new_for_test(
                    PathBuf::from(format!("failed-{i}.md")),
                    format!("err-{i}").into_boxed_str(),
                )
            })
            .collect();
        IndexReport::new_for_test(
            scanned,
            new,
            fresh,
            stale,
            deleted,
            Box::new([]),
            failures.into_boxed_slice(),
        )
    }

    mod build_index_command {
        use pretty_assertions::assert_eq;

        use super::*;

        #[test]
        fn maps_default_args_to_full_scope() {
            let tmp = tempfile::tempdir().unwrap();
            let root = DirPath::try_new(tmp.path().to_path_buf()).unwrap();
            let args = IndexArgs {
                rebuild: false,
                path: None,
                dry_run: false,
            };

            let cmd = build_index_command(args, &root).unwrap();

            assert_eq!(cmd.scope(), &IndexScope::Full {
                root: root.clone(),
                filters: ScanFilters::default()
            });
            assert_eq!(cmd.opts(), IndexOptions::new(false, false));
        }

        #[test]
        fn maps_path_to_partial_scope() {
            let tmp = tempfile::tempdir().unwrap();
            std::fs::create_dir(tmp.path().join("sub")).unwrap();
            let root = DirPath::try_new(tmp.path().to_path_buf()).unwrap();

            let args = IndexArgs {
                rebuild: false,
                path: Some(PathBuf::from("sub")),
                dry_run: false,
            };

            let cmd = build_index_command(args, &root).unwrap();

            let target = DirPath::try_new(tmp.path().join("sub")).unwrap();
            assert_eq!(cmd.scope(), &IndexScope::Partial {
                root: target,
                filters: ScanFilters::default()
            });
        }

        #[test]
        fn returns_invalid_path_error_when_path_is_file_not_dir() {
            let tmp = tempfile::tempdir().unwrap();
            std::fs::write(tmp.path().join("file.txt"), "").unwrap();
            let root = DirPath::try_new(tmp.path().to_path_buf()).unwrap();

            let args = IndexArgs {
                rebuild: false,
                path: Some(PathBuf::from("file.txt")),
                dry_run: false,
            };

            let err = build_index_command(args, &root).unwrap_err();
            assert!(
                matches!(err, CliError::InvalidPath(_)),
                "expected CliError::InvalidPath, got {err:?}"
            );
        }

        #[test]
        fn maps_rebuild_and_dry_run_flags() {
            let tmp = tempfile::tempdir().unwrap();
            let root = DirPath::try_new(tmp.path().to_path_buf()).unwrap();
            let args = IndexArgs {
                rebuild: true,
                path: None,
                dry_run: true,
            };

            let cmd = build_index_command(args, &root).unwrap();

            let opts = cmd.opts();
            assert_eq!(opts, IndexOptions::new(true, true));
        }
    }

    mod write_report_human {
        use super::*;

        #[test]
        fn contains_all_labels_and_values() {
            let report = make_report(42, 12, 28, 2, 0, 1);
            let mut buf = Vec::new();
            write_report_human(&report, &mut buf).unwrap();
            let output = String::from_utf8(buf).unwrap();

            assert!(output.contains("  scanned: 42"), "got: {output}");
            assert!(output.contains("      new: 12"), "got: {output}");
            assert!(output.contains("    fresh: 28"), "got: {output}");
            assert!(output.contains("    stale: 2"), "got: {output}");
            assert!(output.contains("  deleted: 0"), "got: {output}");
            assert!(output.contains("   failed: 1"), "got: {output}");
        }

        #[test]
        fn shows_zero_for_all_when_empty() {
            let report = make_report(0, 0, 0, 0, 0, 0);
            let mut buf = Vec::new();
            write_report_human(&report, &mut buf).unwrap();
            let output = String::from_utf8(buf).unwrap();

            assert!(output.contains("  scanned: 0"), "got: {output}");
            assert!(output.contains("   failed: 0"), "got: {output}");
        }
    }

    mod write_report_json {
        use pretty_assertions::assert_eq;

        use super::*;

        #[test]
        fn is_valid_json_with_correct_values() {
            let report = make_report(10, 5, 3, 1, 1, 0);
            let mut buf = Vec::new();
            write_report_json(&report, &mut buf).unwrap();
            let output = String::from_utf8(buf).unwrap();

            let parsed: serde_json::Value =
                serde_json::from_str(&output).expect("valid JSON");
            assert_eq!(
                parsed.get("scanned").and_then(serde_json::Value::as_u64),
                Some(10)
            );
            assert_eq!(
                parsed.get("new").and_then(serde_json::Value::as_u64),
                Some(5)
            );
            assert_eq!(
                parsed.get("fresh").and_then(serde_json::Value::as_u64),
                Some(3)
            );
            assert_eq!(
                parsed.get("stale").and_then(serde_json::Value::as_u64),
                Some(1)
            );
            assert_eq!(
                parsed.get("deleted").and_then(serde_json::Value::as_u64),
                Some(1)
            );
            assert_eq!(
                parsed.get("failed").and_then(serde_json::Value::as_u64),
                Some(0)
            );
        }
    }

    mod run_index_handler {
        use std::fs;

        use tempfile::tempdir;
        use traces_app::bootstrap::BootstrapRunner;
        use traces_settings::{DiscoveryFlags, DiscoveryService};

        use super::super::run_index;
        use crate::cli::{IndexArgs, OutputFormat};

        fn make_vault() -> (
            tempfile::TempDir,
            BootstrapRunner<DiscoveryService>,
            DiscoveryFlags,
        ) {
            let dir = tempdir().expect("vault dir");
            let config_path = dir.path().join("traces.toml");
            fs::write(&config_path, "[template]\ndirectory = \"templates\"")
                .expect("write traces.toml");

            // Create some files for the indexer to discover. The vault root
            // dir is never indexed, and only directories count as indexed
            // "dirs". We create file1.md at root and file2.md in sub/ so
            // assertions mirror those in crates/app/tests/index.rs:
            //   1 dir + 2 files = 3 indexed nodes.
            fs::write(dir.path().join("file1.md"), "c1").expect("write file1");
            fs::create_dir(dir.path().join("sub")).expect("create sub");
            fs::write(dir.path().join("sub/file2.md"), "c2")
                .expect("write file2");

            // Pre-create the cache directory so Store::open succeeds.
            fs::create_dir_all(dir.path().join(".traces/cache"))
                .expect("create .traces/cache");

            let flags = DiscoveryFlags::new(
                Some(config_path.as_path()),
                Some(dir.path()),
                true, // suppress global
            )
            .expect("valid flags");
            let bootstrapper = BootstrapRunner::with_global_directories(vec![])
                .expect("bootstrapper");
            (dir, bootstrapper, flags)
        }

        #[test]
        fn indexes_vault_and_prints_human_report() {
            let (dir, bootstrapper, flags) = make_vault();

            let args = IndexArgs {
                rebuild: false,
                path: None,
                dry_run: false,
            };

            let mut out = Vec::<u8>::new();
            let mut err = Vec::<u8>::new();
            let result = run_index(
                &bootstrapper,
                Some(flags),
                dir.path(),
                args,
                OutputFormat::Human,
                0,
                &mut out,
                &mut err,
            );

            assert!(result.is_ok(), "run_index failed: {result:?}");
            let stdout = String::from_utf8(out).expect("stdout utf8");
            assert!(stdout.contains("scanned:"), "stdout:\n{stdout}");
            assert!(stdout.contains("new:"), "stdout:\n{stdout}");
        }

        #[test]
        fn indexes_vault_and_prints_json_report() {
            let (dir, bootstrapper, flags) = make_vault();

            let args = IndexArgs {
                rebuild: false,
                path: None,
                dry_run: false,
            };

            let mut out = Vec::<u8>::new();
            let mut err = Vec::<u8>::new();
            let result = run_index(
                &bootstrapper,
                Some(flags),
                dir.path(),
                args,
                OutputFormat::Json,
                0,
                &mut out,
                &mut err,
            );

            assert!(result.is_ok(), "run_index failed: {result:?}");
            let stdout = String::from_utf8(out).expect("stdout utf8");
            let parsed: serde_json::Value =
                serde_json::from_str(&stdout).expect("valid JSON");
            // Must contain all report fields with non-zero scanned count.
            // The exact scanned values include cache artifacts
            // (.traces/cache/*.db), so assert structure, not exact count.
            let scanned = parsed
                .get("scanned")
                .and_then(serde_json::Value::as_u64)
                .expect("scanned field as u64");
            assert!(scanned > 0, "expected scanned > 0, got: {stdout}");
            for key in &["new", "fresh", "stale", "deleted", "failed"] {
                assert!(
                    parsed.get(*key).is_some(),
                    "missing field '{key}', got: {stdout}"
                );
            }
        }
    }
}
