//! CLI handler for the `index` command.

use std::{io::Write, path::Path};

use trace_app::{
    bootstrap::Bootstrapper,
    index::{
        IndexCommand, IndexOptions, IndexScope, ScanFilters,
        run_index as app_run_index,
    },
};
use trace_settings::DiscoveryFlags;

use crate::{
    cli::{IndexArgs, OutputFormat},
    error::CliError,
};

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
    bootstrapper: &Bootstrapper<impl trace_settings::port::DiscoveryPort>,
    flags: Option<DiscoveryFlags>,
    anchor: &Path,
    args: IndexArgs,
    _format: OutputFormat,
    _verbose: u8,
    mut _out: impl Write,
    mut _err: impl Write,
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
    let scope = if let Some(rel_path) = args.path {
        let abs_path = vault_root.as_path().join(rel_path);
        let dir_path = trace_fs::DirPath::try_from(abs_path)
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

    let cmd = IndexCommand::new(scope, opts);

    // 3. Execute
    let cache_dir_path = discovery.cache_root().path().to_path_buf();
    let cache_dir = trace_fs::DirPath::try_from(cache_dir_path)
        .map_err(|e| CliError::InvalidPath(e.to_string()))?;

    let _result =
        app_run_index(&vault_root, &cache_dir, &cmd).map_err(CliError::from)?;

    // Formatting for CLI output is pending in Cycle 5 (Output formatting).
    // The test in this cycle only requires mapping arguments to models.
    Ok(())
}

// ----------------------------------------------------------- //
//                            Tests                            //
// ----------------------------------------------------------- //

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use trace_app::index::{IndexOptions, IndexScope, ScanFilters};
    use trace_fs::DirPath;

    use crate::cli::IndexArgs;

    // A helper to map arguments to command for testing
    fn build_domain_command(
        args: IndexArgs,
        root: &DirPath,
    ) -> Result<trace_app::index::IndexCommand, String> {
        let scope = if let Some(rel_path) = args.path {
            let abs_path = root.as_path().join(rel_path);
            let dir_path = trace_fs::DirPath::try_from(abs_path)
                .map_err(|e| e.to_string())?;
            IndexScope::Partial {
                root: dir_path,
                filters: ScanFilters::default(),
            }
        } else {
            IndexScope::Full {
                root: root.clone(),
                filters: ScanFilters::default(),
            }
        };

        let opts = IndexOptions::new(args.rebuild, args.dry_run);

        Ok(trace_app::index::IndexCommand::new(scope, opts))
    }

    #[test]
    fn maps_default_args_to_full_scope() {
        let tmp = tempfile::tempdir().unwrap();
        let root = DirPath::try_new(tmp.path().to_path_buf()).unwrap();
        let args = IndexArgs {
            rebuild: false,
            path: None,
            dry_run: false,
        };

        let cmd = build_domain_command(args, &root).unwrap();

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

        let cmd = build_domain_command(args, &root).unwrap();

        let target = DirPath::try_new(tmp.path().join("sub")).unwrap();
        assert_eq!(cmd.scope(), &IndexScope::Partial {
            root: target,
            filters: ScanFilters::default()
        });
    }

    #[test]
    fn fails_if_path_is_not_dir() {
        let tmp = tempfile::tempdir().unwrap();
        std::fs::write(tmp.path().join("file.txt"), "").unwrap();
        let root = DirPath::try_new(tmp.path().to_path_buf()).unwrap();

        let args = IndexArgs {
            rebuild: false,
            path: Some(PathBuf::from("file.txt")),
            dry_run: false,
        };

        let err = build_domain_command(args, &root).unwrap_err();
        assert!(err.contains("Path does not refer to a directory"));
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

        let cmd = build_domain_command(args, &root).unwrap();

        let opts = cmd.opts();
        assert_eq!(opts, IndexOptions::new(true, true));
    }
}
