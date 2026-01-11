//! Lithos Domain Crate
//!
//! This crate contains the pure business logic, domain entities, and port definitions
//! for the Lithos system. It has no dependencies on external I/O or frameworks.

/// Placeholder domain error for initialization verification
#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub enum DomainError {
    /// Initial placeholder error
    #[error("Initialization error")]
    Initialize,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn domain_error_is_send_and_sync() {
        fn is_send_sync<T: Send + Sync>() {}
        is_send_sync::<DomainError>();
    }
}
