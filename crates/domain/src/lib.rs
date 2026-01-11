//! Lithos Domain Crate
//!
//! This crate contains the pure business logic, domain entities, and port definitions
//! for the Lithos system. It has no dependencies on external I/O or frameworks.

/// Placeholder domain error for initialization verification
#[derive(Debug, thiserror::Error)]
pub enum DomainError {
    /// Initial placeholder error
    #[error("Initialization error")]
    Initialize,
}
