//! File system utilities for parsing and validation.
//!
//! This module provides generic file system operations with no domain
//! knowledge. Dependencies flow inward: domain contexts may use fs utilities,
//! but fs has no dependencies on domain logic.

/// File system error types.
#[expect(
    clippy::module_name_repetitions,
    reason = "Context-specific error name is intentional"
)]
#[non_exhaustive]
#[derive(Debug, thiserror::Error)]
pub enum FsError {
    /// IO operation failed.
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),
}
