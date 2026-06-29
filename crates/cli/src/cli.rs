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

/// Arguments for the `index` subcommand.
///
/// EXAMPLES:
///   $ traces index
///   $ traces index --rebuild
///   $ traces index --path templates/ --format json
#[derive(Debug, Args, PartialEq, Eq)]
pub(crate) struct IndexArgs {
    /// Discard current cache and perform a full rebuild.
    #[arg(long)]
    pub(crate) rebuild: bool,

    /// Only index files under this path (relative to vault root).
    #[arg(long, short)]
    pub(crate) path: Option<PathBuf>,

    /// Run the index process but do not save results to cache.
    #[arg(long)]
    pub(crate) dry_run: bool,
}

/// Arguments for the `template` subcommand.
#[derive(Debug, Args)]
pub(crate) struct TemplateArgs {
    /// Template input path.
    #[arg(short = 'i', long)]
    pub(crate) input: String,

    /// Rendered output path.
    #[arg(short = 'o', long, required_unless_present = "dry_run")]
    pub(crate) output: Option<String>,

    /// Render without writing output.
    #[arg(short = 'n', long, conflicts_with = "output")]
    pub(crate) dry_run: bool,

    /// Template variable assignment.
    #[arg(long = "var", action = clap::ArgAction::Append)]
    pub(crate) vars: Vec<String>,
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
    /// Index templates and layouts in the current vault.
    Index(IndexArgs),
    /// Render a template.
    Template(TemplateArgs),
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

    #[expect(
        clippy::panic,
        reason = "let-else panic signals test invariant violation, not \
                  production code"
    )]
    #[test]
    fn routes_config_files_subcommand() {
        let cli = parse(&["traces", "config", "files"]).expect("valid args");
        let is_config = matches!(cli.command, Command::Config { .. });
        assert!(is_config, "expected Config command variant");
        let Command::Config {
            command,
        } = cli.command
        else {
            panic!("expected Config variant, got {:?}", cli.command);
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

    #[test]
    fn parses_index_subcommand() {
        let cli = parse(&["traces", "index"]).expect("valid args");
        assert!(matches!(cli.command, Command::Index(_)));
    }

    #[expect(
        clippy::panic,
        reason = "match panic signals test invariant violation, not \
                  production code"
    )]
    #[test]
    fn parses_template_subcommand() {
        let cli = parse(&[
            "traces",
            "template",
            "--input",
            "note.md",
            "--output",
            "rendered.md",
            "--var",
            "name=traces",
            "--var",
            "mode=cli",
        ])
        .expect("valid args");

        let Command::Template(args) = cli.command else {
            panic!("expected Template variant, got {:?}", cli.command);
        };
        assert_eq!(args.input, "note.md");
        assert_eq!(args.output, Some("rendered.md".to_owned()));
        assert!(!args.dry_run);
        assert_eq!(args.vars, ["name=traces", "mode=cli"]);
    }

    #[expect(
        clippy::panic,
        reason = "match panic signals test invariant violation, not \
                  production code"
    )]
    #[test]
    fn template_accepts_short_input_and_output_flags() {
        let cli = parse(&[
            "traces",
            "template",
            "-i",
            "greeting",
            "-o",
            "notes/out.md",
        ])
        .expect("valid args");

        let Command::Template(args) = cli.command else {
            panic!("expected Template variant, got {:?}", cli.command);
        };

        assert_eq!(args.input, "greeting");
        assert_eq!(args.output, Some("notes/out.md".to_owned()));
        assert!(!args.dry_run);
    }

    #[expect(
        clippy::panic,
        reason = "match panic signals test invariant violation, not \
                  production code"
    )]
    #[test]
    fn template_accepts_long_dry_run_flag() {
        let cli =
            parse(&["traces", "template", "--input", "greeting", "--dry-run"])
                .expect("valid args");

        let Command::Template(args) = cli.command else {
            panic!("expected Template variant, got {:?}", cli.command);
        };

        assert!(args.dry_run);
        assert_eq!(args.output, None);
    }

    #[expect(
        clippy::panic,
        reason = "match panic signals test invariant violation, not \
                  production code"
    )]
    #[test]
    fn short_flag_n_sets_dry_run() {
        let cli = parse(&["traces", "template", "--input", "note", "-n"])
            .expect("valid args");

        let Command::Template(args) = cli.command else {
            panic!("expected Template variant, got {:?}", cli.command);
        };
        assert!(args.dry_run);
    }

    #[test]
    fn rejects_missing_input() {
        let err = parse(&["traces", "template", "--output", "out.md"])
            .expect_err("missing input should fail");

        assert_eq!(err.kind(), clap::error::ErrorKind::MissingRequiredArgument);
        assert!(err.to_string().contains("--input <INPUT>"));
    }

    #[test]
    fn rejects_conflicting_output_and_dry_run() {
        let err = parse(&[
            "traces",
            "template",
            "--input",
            "note",
            "--output",
            "out.md",
            "--dry-run",
        ])
        .expect_err("conflicting output and dry-run should fail");

        assert_eq!(err.kind(), clap::error::ErrorKind::ArgumentConflict);
        assert!(err.to_string().contains("--output <OUTPUT>"));
        assert!(err.to_string().contains("--dry-run"));
    }

    #[test]
    fn index_subcommand_fails_under_config() {
        let result = parse(&["traces", "config", "index"]);
        assert!(result.is_err());
    }

    #[expect(
        clippy::panic,
        reason = "let-else panic signals test invariant violation, not \
                  production code"
    )]
    #[test]
    fn parses_index_args_all_flags() {
        let cli = parse(&[
            "traces",
            "index",
            "--rebuild",
            "--path",
            "sub/dir",
            "--dry-run",
        ])
        .expect("valid args");
        assert!(
            matches!(cli.command, Command::Index(_)),
            "Expected Index subcommand"
        );
        let Command::Index(args) = cli.command else {
            panic!("expected Index variant, got {:?}", cli.command);
        };
        assert!(args.rebuild);
        assert_eq!(args.path, Some(PathBuf::from("sub/dir")));
        assert!(args.dry_run);
    }

    #[expect(
        clippy::panic,
        reason = "let-else panic signals test invariant violation, not \
                  production code"
    )]
    #[test]
    fn index_args_defaults() {
        let cli = parse(&["traces", "index"]).expect("valid args");
        assert!(
            matches!(cli.command, Command::Index(_)),
            "Expected Index subcommand"
        );
        let Command::Index(args) = cli.command else {
            panic!("expected Index variant, got {:?}", cli.command);
        };
        assert!(!args.rebuild);
        assert_eq!(args.path, None);
        assert!(!args.dry_run);
    }

    #[test]
    fn index_rejects_unknown_flags() {
        let result = parse(&["traces", "index", "--unknown"]);
        assert!(result.is_err());
    }
}
