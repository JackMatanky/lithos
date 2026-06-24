//! # CLI Argument Definitions
//!
//! Defines the command-line interface structure for Traces using the clap
//! derive API. All argument types, subcommands, and output format variants
//! are declared here and consumed by `main.rs`.

use std::path::PathBuf;

use clap::{ArgAction, Args, Parser, Subcommand, ValueEnum};

/// Top-level CLI arguments for the Traces binary.
#[derive(Debug, Parser)]
#[command(
    name = "traces",
    version,
    about = "A CLI-first templating and schema system for Obsidian vaults"
)]
pub(crate) struct Cli {
    /// Bootstrap/loading flags shared by all commands.
    #[command(flatten)]
    pub(crate) bootstrap: BootstrapArgs,

    /// Output format for command results.
    #[arg(long, global = true, default_value = "human")]
    pub(crate) format: OutputFormat,

    /// Increase verbosity (pass multiple times for more detail).
    #[arg(short = 'v', long, global = true, action = ArgAction::Count)]
    pub(crate) verbose: u8,

    /// Subcommand to execute.
    #[command(subcommand)]
    pub(crate) command: Command,
}

/// Bootstrap/loading flags shared by all commands.
///
/// These flags control how vault and config file discovery is performed before
/// any command runs. They are flattened into the top-level [`Cli`] struct so
/// they appear as top-level options in the CLI.
#[derive(Debug, Args)]
pub(crate) struct BootstrapArgs {
    /// Override the vault root directory for this invocation.
    #[arg(long, global = true)]
    pub(crate) vault: Option<PathBuf>,
    /// Override the config file path for this invocation.
    #[arg(long, global = true)]
    pub(crate) config: Option<PathBuf>,
    /// Suppress loading the global config file.
    #[arg(long, global = true)]
    pub(crate) no_global_config: bool,
}

/// Top-level subcommands available in the Traces CLI.
#[derive(Debug, Subcommand)]
pub(crate) enum Command {
    /// Inspect the effective bootstrap and configuration loading state.
    Config {
        /// Optional sub-subcommand under `config`.
        #[command(subcommand)]
        command: Option<ConfigSubcommand>,
    },
    /// Run health checks on the current environment.
    Doctor,
}

/// Sub-subcommands available under `config`.
#[derive(Debug, Subcommand)]
pub(crate) enum ConfigSubcommand {
    /// List all config files that Traces resolves for the current context.
    Files,
}

/// Output format requested by the caller.
#[derive(Clone, Copy, Debug, Default, PartialEq, ValueEnum)]
pub(crate) enum OutputFormat {
    /// Human-readable, coloured terminal output.
    #[default]
    Human,
    /// Machine-readable JSON output.
    Json,
}

// ----------------------------------------------------------- //
//                            Tests                            //
// ----------------------------------------------------------- //

#[cfg(test)]
mod arg_parsing {
    use std::path::PathBuf;

    use clap::Parser;

    use super::{Cli, Command, ConfigSubcommand, OutputFormat};

    fn parse(args: &[&str]) -> Result<Cli, clap::Error> {
        Cli::try_parse_from(args)
    }

    #[test]
    fn returns_vault_flag_value_when_provided() {
        let cli = parse(&["traces", "--vault", "/tmp/vault", "doctor"])
            .expect("valid args");
        assert_eq!(cli.bootstrap.vault, Some(PathBuf::from("/tmp/vault")));
    }

    #[test]
    fn returns_config_flag_value_when_provided() {
        let cli = parse(&["traces", "--config", "/tmp/traces.toml", "doctor"])
            .expect("valid args");
        assert_eq!(
            cli.bootstrap.config,
            Some(PathBuf::from("/tmp/traces.toml"))
        );
    }

    #[test]
    fn returns_no_global_config_when_flag_present() {
        let cli = parse(&["traces", "--no-global-config", "doctor"])
            .expect("valid args");
        assert!(cli.bootstrap.no_global_config);
    }

    #[test]
    fn returns_format_human_by_default() {
        let cli = parse(&["traces", "doctor"]).expect("valid args");
        assert_eq!(cli.format, OutputFormat::Human);
    }

    #[test]
    fn returns_format_json_when_explicitly_set() {
        let cli = parse(&["traces", "--format", "json", "doctor"])
            .expect("valid args");
        assert_eq!(cli.format, OutputFormat::Json);
    }

    #[test]
    fn rejects_unknown_format_value() {
        let result = parse(&["traces", "--format", "xml", "doctor"]);
        assert!(result.is_err());
    }

    #[test]
    fn returns_verbose_count_zero_by_default() {
        let cli = parse(&["traces", "doctor"]).expect("valid args");
        assert_eq!(cli.verbose, 0);
    }

    #[test]
    fn returns_verbose_count_incremented_per_flag() {
        let cli =
            parse(&["traces", "-v", "-v", "-v", "doctor"]).expect("valid args");
        assert_eq!(cli.verbose, 3);
    }

    #[test]
    fn routes_config_subcommand() {
        let cli = parse(&["traces", "config"]).expect("valid args");
        assert!(matches!(cli.command, Command::Config { .. }));
    }

    #[test]
    fn routes_config_files_subcommand() {
        let cli = parse(&["traces", "config", "files"]).expect("valid args");
        let is_config = matches!(cli.command, Command::Config { .. });
        assert!(is_config, "expected Config command variant");
        let Command::Config {
            command,
        } = cli.command
        else {
            return;
        };
        assert!(matches!(command, Some(ConfigSubcommand::Files)));
    }

    #[test]
    fn routes_doctor_subcommand() {
        let cli = parse(&["traces", "doctor"]).expect("valid args");
        assert!(matches!(cli.command, Command::Doctor));
    }

    #[test]
    fn vault_flag_is_available_to_config_files_subcommand() {
        let cli = parse(&["traces", "--vault", "/my/vault", "config", "files"])
            .expect("valid args");
        assert_eq!(cli.bootstrap.vault, Some(PathBuf::from("/my/vault")));
    }

    #[test]
    fn vault_flag_is_available_to_doctor_subcommand() {
        let cli = parse(&["traces", "--vault", "/my/vault", "doctor"])
            .expect("valid args");
        assert_eq!(cli.bootstrap.vault, Some(PathBuf::from("/my/vault")));
    }
}
