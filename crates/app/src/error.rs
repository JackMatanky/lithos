//! App-layer error types and aggregation.
//!
//! This module defines the top-level error boundaries for the application's
//! composition root. It aggregates errors from various bounded contexts
//! (like discovery, config, and indexing) into a unified `AppError` type,
//! which is subsequently wrapped and surfaced by executable adapters (e.g.,
//! CLI).

use trace_settings::{DiscoveryError, config::error::ConfigError};

/// App-owned bootstrap error boundary.
#[derive(Debug, thiserror::Error)]
pub enum AppError {
    /// Discovery setup or execution failed.
    ///
    /// Covers all [`DiscoveryError`] variants including
    /// [`DiscoveryError::InvalidAnchorDirectory`] (anchor does not exist),
    /// service configuration errors, and traversal failures.
    #[error(transparent)]
    Discovery(#[from] DiscoveryError),

    /// Configuration building failed.
    #[error(transparent)]
    Config(#[from] ConfigError),

    /// Indexing pipeline failed.
    #[error(transparent)]
    Indexer(#[from] trace_indexer::IndexerError),
}

#[cfg(test)]
mod tests {
    use super::*;

    mod conversions {
        use super::*;

        #[test]
        fn converts_indexer_error_to_app_error() {
            let inner = trace_indexer::IndexerError::Scanner(
                trace_indexer::ScannerError::Traversal {
                    path: std::path::PathBuf::from("/"),
                    source: std::io::Error::from(std::io::ErrorKind::NotFound),
                },
            );
            let app_err = AppError::from(inner);
            assert!(matches!(app_err, AppError::Indexer(_)));
        }

        #[test]
        fn preserves_discovery_error_variant() {
            let inner = trace_settings::DiscoveryError::Env(
                trace_settings::discovery::error::EnvironmentOverrideError::GlobalConfigPathNotFound { path: std::path::PathBuf::from("/") }
            );
            let app_err = AppError::from(inner);
            assert!(matches!(app_err, AppError::Discovery(_)));
        }

        #[test]
        fn preserves_config_error_variant() {
            let inner = trace_settings::config::error::ConfigError::DependencyViolation {
                field: "foo".into(),
                depends_on: "bar".into(),
            };
            let app_err = AppError::from(inner);
            assert!(matches!(app_err, AppError::Config(_)));
        }
    }
}
