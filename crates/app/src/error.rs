//! App-layer error types.

use trace_config::error::ConfigError;
use trace_discovery::error::DiscoveryError;

/// App-owned bootstrap error boundary.
#[derive(Debug, thiserror::Error)]
pub enum BootstrapError {
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
}
