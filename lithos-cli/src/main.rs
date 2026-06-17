//! # Lithos CLI Binary
//!
//! This binary provides the command-line interface for Lithos, a CLI-first
//! templating and schema system for Obsidian vaults. It handles argument
//! parsing, configuration loading, and delegates to application services.
//!
//! ## Architectural Invariants
//!
//! - **Rich Diagnostics**: Uses `miette` for high-fidelity error reporting with
//!   source snippets and help messages.
//! - **Sync-First Design**: Core logic is synchronous per the architecture
//!   proposal. Async is only used at edges if needed for concurrency.
//! - **Configuration Hierarchy**: Loads settings from global config, vault
//!   config, and command-line overrides in precedence order.
//!
//! ## Usage
//!
//! Run the CLI with `--help` for available commands and options.
//!
//! ## Example
//!
//! ```bash
//! lithos --version
//! ```
//!
//! # Errors
//!
//! Returns an error if configuration loading fails, vault initialization
//! encounters issues, or template processing errors occur. Errors are reported
//! via `miette` with contextual information.

mod cli;
mod commands;
mod error;

use std::process::ExitCode;

use clap::Parser as _;
use lithos_core::{app::bootstrap::Bootstrapper, discovery::DiscoveryFlags};

use crate::{
    cli::{Command, ConfigSubcommand},
    commands::{
        config::run_config, config_files::run_config_files, doctor::run_doctor,
    },
    error::CliError,
};

/// Runs the Lithos CLI application and returns the appropriate exit code.
///
/// Parses CLI arguments, constructs the bootstrap runner, builds discovery
/// flags, and dispatches to the appropriate command handler.  Errors from
/// handlers are rendered to stderr and mapped to POSIX exit codes.
///
/// # Exit Codes
///
/// | Code | Meaning                                              |
/// |------|------------------------------------------------------|
/// | `0`  | Success                                              |
/// | `1`  | Vault not found — no valid anchor directory          |
/// | `2`  | Invalid explicit path or configuration error         |
/// | `3`  | Filesystem permission denied or directory unreadable |
fn main() -> ExitCode {
    match run_main() {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            let code = e.exit_code();
            let report = miette::Report::new(e);
            eprintln!("{report:?}");
            ExitCode::from(u8::try_from(code).unwrap_or(2))
        }
    }
}

/// Core CLI logic extracted from `main()` for testability and clean error
/// propagation.
///
/// # Errors
///
/// Returns [`CliError`] if:
/// - The working directory cannot be read.
/// - The bootstrapper cannot be created from platform configuration.
/// - Discovery or config loading fails.
/// - A handler returns an error.
#[expect(
    clippy::disallowed_methods,
    reason = "std::env::current_dir is required at the CLI entry point to \
              resolve the invocation anchor directory. The disallowed-methods \
              lint targets accidental CWD reads in domain logic; this is an \
              intentional, one-time read at the process boundary."
)]
fn run_main() -> Result<(), CliError> {
    let cli = cli::Cli::parse();

    // Resolve the current working directory as the discovery anchor.
    let anchor = std::env::current_dir().map_err(|source| {
        CliError::Bootstrap(
            lithos_core::app::bootstrap::BootstrapError::Discovery(
                lithos_core::discovery::error::DiscoveryError::CurrentDirectoryCanonicalize {
                    path: std::path::PathBuf::from("."),
                    source,
                },
            ),
        )
    })?;

    // Create the bootstrapper from platform-specific global config directories.
    let bootstrapper = Bootstrapper::from_platform().map_err(CliError::from)?;

    // Build discovery flags from top-level CLI overrides (if any were set).
    let flags = build_discovery_flags(&cli)?;

    let format = cli.format;
    let verbose = cli.verbose;
    let stdout = std::io::stdout();
    let stderr = std::io::stderr();
    let mut out = stdout.lock();
    let mut err = stderr.lock();

    match cli.command {
        Command::Config(config_args) => {
            match config_args.subcommand {
                None => {
                    // `lithos config` — show resolved configuration summary.
                    run_config(
                        &bootstrapper,
                        flags,
                        &anchor,
                        format,
                        verbose,
                        &mut out,
                        &mut err,
                    )
                }
                Some(ConfigSubcommand::Files) => {
                    // `lithos config files` — list discovered config file
                    // candidates.
                    run_config_files(
                        &bootstrapper,
                        flags,
                        &anchor,
                        format,
                        &mut out,
                    )
                }
            }
        }
        Command::Doctor => run_doctor(
            &bootstrapper,
            flags,
            &anchor,
            format,
            verbose,
            &mut out,
            &mut err,
        ),
    }
}

/// Builds [`DiscoveryFlags`] from top-level CLI arguments.
///
/// Returns `Some(flags)` when at least one override flag was set, or `None`
/// when no overrides were provided (the default no-override case).
///
/// # Errors
///
/// Returns [`CliError`] if any provided path is invalid (non-existent or
/// wrong filesystem type).
fn build_discovery_flags(
    cli: &cli::Cli,
) -> Result<Option<DiscoveryFlags>, CliError> {
    let has_overrides =
        cli.vault.is_some() || cli.config.is_some() || cli.no_global_config;

    if !has_overrides {
        return Ok(None);
    }

    let flags = DiscoveryFlags::new(
        cli.config.as_deref(),
        cli.vault.as_deref(),
        cli.no_global_config,
    )
    .map_err(|e| {
        CliError::Bootstrap(
            lithos_core::app::bootstrap::BootstrapError::Discovery(e),
        )
    })?;

    Ok(Some(flags))
}

#[cfg(test)]
#[expect(
    clippy::assertions_on_result_states,
    reason = "Result assertions in tests avoid unwrap/expect while keeping \
              intent clear."
)]
mod tests {
    use clap::Parser as _;

    use crate::cli::Cli;

    #[test]
    fn main_parses_doctor_subcommand_successfully() {
        let result = Cli::try_parse_from(["lithos", "doctor"]);
        assert!(result.is_ok());
    }

    #[test]
    fn main_parses_config_subcommand_successfully() {
        let result = Cli::try_parse_from(["lithos", "config"]);
        assert!(result.is_ok());
    }

    #[test]
    fn main_parses_config_files_subcommand_successfully() {
        let result = Cli::try_parse_from(["lithos", "config", "files"]);
        assert!(result.is_ok());
    }
}
