//! Lithos Adapters Crate.
//!
//! This crate contains the infrastructure implementations (SPI) for
//! persistence, filesystem access, and other external services.

#![expect(clippy::pub_use, reason = "Intended public API re-exports")]

pub mod spi;

// Re-export common types
pub use spi::errors::ParseError;
// Re-export parser utilities for convenience
pub use spi::{JsonParser, ParserDispatcher, TomlParser, YamlParser};

#[cfg(test)]
mod tests {
    #[test]
    fn adapters_compilation_works() {
        // Simple test to verify the crate is being checked by the test harness
        let ready = true;
        assert!(ready);
    }
}
