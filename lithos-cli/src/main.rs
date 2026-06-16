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

use clap::Parser as _;

/// The main entry point for the Lithos application.
///
/// Initializes the CLI, parses arguments, and runs the application logic.
///
/// # Errors
///
/// Returns an error if initialization fails or unhandled exceptions occur.
///
/// # Phase 5 Note
///
/// This is a sync-first implementation per Proposal 6. Core logic is
/// synchronous. If async is needed later (e.g., for parallel indexing),
/// use `tokio::task::spawn_blocking` to keep async at the edges.
#[expect(
    clippy::unnecessary_wraps,
    reason = "Result return type reserved for future error propagation from \
              command handlers"
)]
fn main() -> miette::Result<()> {
    let _cli = cli::Cli::parse();
    tracing::info!("Hello, Lithos!");
    Ok(())
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
}
