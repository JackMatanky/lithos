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
    tracing::info!("Hello, Lithos!");
    Ok(())
}
