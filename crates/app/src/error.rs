//! App-layer error types and aggregation.
//!
//! This module defines the top-level error boundaries for the application's
//! composition root. It aggregates errors from various bounded contexts
//! (like discovery, config, indexing, and template rendering) into a unified
//! `AppError` type,
//! which is subsequently wrapped and surfaced by executable adapters (e.g.,
//! CLI).

use traces_settings::{DiscoveryError, config::error::ConfigError};

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
    Indexer(#[from] traces_indexer::IndexerError),

    /// Template pipeline failed.
    #[error(transparent)]
    Template(#[from] traces_template::TemplateError),
}

// ----------------------------------------------------------- //
//                            Tests                            //
// ----------------------------------------------------------- //

#[cfg(test)]
mod tests {
    use super::*;

    mod conversions {
        use super::*;

        #[test]
        fn converts_indexer_error_to_app_error() {
            let inner = traces_indexer::IndexerError::Scanner(
                traces_indexer::ScannerError::Traversal {
                    path: std::path::PathBuf::from("/"),
                    source: std::io::Error::from(std::io::ErrorKind::NotFound),
                },
            );
            let app_err = AppError::from(inner);
            assert!(matches!(app_err, AppError::Indexer(_)));
        }

        #[test]
        fn preserves_discovery_error_variant() {
            let inner = traces_settings::DiscoveryError::Env(
                traces_settings::discovery::error::EnvironmentOverrideError::GlobalConfigPathNotFound { path: std::path::PathBuf::from("/") }
            );
            let app_err = AppError::from(inner);
            assert!(matches!(app_err, AppError::Discovery(_)));
        }

        #[test]
        fn preserves_config_error_variant() {
            let inner = traces_settings::config::error::ConfigError::DependencyViolation {
                field: "foo".into(),
                depends_on: "bar".into(),
            };
            let app_err = AppError::from(inner);
            assert!(matches!(app_err, AppError::Config(_)));
        }

        #[test]
        fn converts_template_error_to_app_error() {
            let name = traces_template::TemplateName::try_new(
                std::path::Path::new("templates/missing.md"),
                std::path::Path::new("templates"),
            )
            .expect("expected template name");
            let inner = traces_template::TemplateError::NotFound {
                name,
            };
            let app_err = AppError::from(inner);
            assert!(matches!(app_err, AppError::Template(_)));
        }
    }
}
