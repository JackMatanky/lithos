//! Internal settings discovery for Traces.
//!
//! New settings-service code normalizes
//! [`DiscoveryOptions`](crate::DiscoveryOptions)
//! plus internal env capture, collects vault-local and global candidate config
//! paths, filters/deduplicates them, and returns a
//! [`DiscoveryOutcome`](crate::DiscoveryOutcome). Discovery does not parse
//! config contents or resolve cache directories.
//!
//! Old `DiscoveryPort`/`DiscoveryService` modules remain for migration slices
//! that still compile against the previous bootstrap path.

pub mod context;
pub mod dirs;
pub mod env;
pub mod error;
pub(crate) mod filter;
pub(crate) mod global;
#[allow(
    dead_code,
    reason = "internal linear discovery slice is still wiring callers"
)]
pub(crate) mod input;
pub mod location;
pub mod outcome;
pub(crate) mod policy;
pub mod port;
pub(crate) mod probe;
#[allow(
    dead_code,
    reason = "internal linear discovery slice is still wiring callers"
)]
pub(crate) mod processor;
pub(crate) mod processor_old;
pub mod report;
pub mod service;
pub mod targets;
pub(crate) mod walk;

pub use context::{DiscoveryContext, DiscoveryEnv, DiscoveryFlags};
pub use env::EnvVars;
pub use location::{
    CacheLocation, CacheRoot, GlobalCacheLocation, LocalCacheLocation,
};
pub use report::{
    DiscoveryReport, GlobalResolutionSkipReason, LocalTraversalStopReason,
    SkippedCeiling, SkippedCeilingReason,
};
pub use service::{DiscoveryResult, DiscoveryService, DiscoveryServiceConfig};
