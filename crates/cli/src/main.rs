//! Lithos CLI Binary.
//!
//! The entry point for the Lithos command-line interface.

// # LINT_DISABLE_REASON: Main entry point requires disallowed methods for initialization
// | Options tried: None
// | Justification: Initial setup and signal handling often require methods disallowed in business logic.
#[expect(
    clippy::disallowed_methods,
    reason = "Main entry point requires disallowed methods for initialization"
)]
#[tokio::main]
/// The main entry point for the Lithos application.
async fn main() -> miette::Result<()> {
    #[expect(
        clippy::let_underscore_untyped,
        reason = "Command line matches are ignored for now as we only use this to trigger --help/--version"
    )]
    let _ = clap::Command::new("lithos")
        .version("0.1.0")
        .about("A CLI-first templating and schema system for Obsidian vaults")
        .ignore_errors(true) // Ignore test flags when running as a unit test
        .get_matches();

    println!("Hello, Lithos!");
    tracing::info!("Hello, Lithos!");
    Ok(())
}

#[cfg(test)]
#[expect(
    clippy::assertions_on_result_states,
    reason = "Result assertions in tests avoid unwrap/expect while keeping intent clear."
)]
mod tests {
    use super::main;

    #[test]
    fn main_runs_successfully() {
        let result = main();
        assert!(result.is_ok());
    }
}
