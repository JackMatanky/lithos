//! Vault-level services and policies.

#![expect(
    clippy::pub_use,
    reason = "Re-exporting service components for ergonomic use in \
              application layer"
)]

pub mod service;
pub mod staleness;

pub use service::{Service, ServiceError};
pub use staleness::StalenessPolicy;
