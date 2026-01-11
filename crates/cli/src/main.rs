//! Lithos CLI Binary
//!
//! The entry point for the Lithos command-line interface.

#[allow(clippy::print_stdout, clippy::disallowed_methods)]
#[tokio::main]
async fn main() -> miette::Result<()> {
    println!("Hello, Lithos!");
    Ok(())
}
