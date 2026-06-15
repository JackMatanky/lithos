//! App-layer error types.

use crate::{config::error::ConfigError, discovery::error::DiscoveryError};

/// App-owned bootstrap error boundary.
#[derive(Debug, thiserror::Error)]
#[allow(
    dead_code,
    reason = "CLI wiring pending; variants used via run() which has no \
              production caller yet"
)]
pub(crate) enum BootstrapError {
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
