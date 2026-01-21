//! Lithos Adapters Crate.
//!
//! This crate contains the infrastructure implementations (SPI) for
//! persistence, filesystem access, and other external services.

pub mod spi;

/// Re-exported adapter for file reading.
pub type FileReaderAdapter = crate::spi::fs::FileReaderAdapter;

#[cfg(test)]
mod tests {
    #[test]
    fn adapters_compilation_works() {
        // Simple test to verify the crate is being checked by the test harness
        let ready = true;
        assert!(ready);
    }
}
