//! Pre-config filesystem discovery for Lithos.
//!
//! This context owns the first phase of the `Discovery -> Config -> Indexer`
//! pipeline. It locates the vault root and the root marker file on disk and
//! returns typed path/format metadata, including the resolved cache root. It
//! does **not** parse, merge, validate, or hash config contents — those
//! responsibilities belong to [`Config`].
//!
//! # Boundary Invariants
//!
//! - **Metadata Only**: Discovery returns path, source, and format metadata
//!   only. It never reads file contents beyond existence checks.
//! - **One-way Flow**: Config consumes Discovery outputs; Discovery never
//!   imports Config types.
//!
//! # Modules
//!
//! - **`context`** — Per-invocation inputs: [`DiscoveryEnv`],
//!   [`DiscoveryFlags`].
//! - **`location`** — Cache root types: [`CacheRoot`], [`CacheLocation`],
//!   [`LocalCacheLocation`], [`GlobalCacheLocation`].
//! - **`port`** — Inbound port trait ([`DiscoveryPort`]).
//! - **`report`** — Non-fatal diagnostic output ([`DiscoveryReport`]).
//! - **`service`** — Concrete service ([`DiscoveryService`]) and boundary data
//!   ([`DiscoveryResult`], [`CandidatePath`]).
//! - **`error`** — Fatal error types ([`DiscoveryError`]).
//! - **`processor`** — Internal typestate pipeline (crate-private).
//! - **`probe`** / **`walk`** — Internal filesystem helpers (crate-private).
//!
//! # Usage Example
//!
//! ```rust,ignore
//! use lithos_core::discovery::{
//!     DiscoveryService, DiscoveryFlags, DiscoveryEnv,
//!     port::DiscoveryPort,
//!     service::DiscoveryServiceConfig,
//! };
//! use lithos_core::discovery::context::DiscoveryContext;
//!
//! let service = DiscoveryService::new(DiscoveryServiceConfig::default())
//!     .expect("valid config");
//! let ctx = DiscoveryContext::new(std::path::Path::new("."))
//!     .expect("valid anchor");
//! let (result, report) = service.discover(&ctx).expect("discovery succeeded");
//! println!("cache root: {:?}", result.cache_root().path());
//! ```
//!
//! [`Config`]: crate::config
//! [`DiscoveryPort`]: crate::discovery::port::DiscoveryPort
//! [`DiscoveryError`]: crate::discovery::error::DiscoveryError

pub(crate) mod context;
pub(crate) mod engine;
pub mod error;
pub mod location;
pub(crate) mod policy;
pub mod port;
pub(super) mod probe;
pub(super) mod processor;
pub mod report;
pub(super) mod selector;
pub mod service;
pub(super) mod walk;

pub use context::{DiscoveryEnv, DiscoveryFlags};
pub use location::{
    CacheLocation, CacheRoot, GlobalCacheLocation, LocalCacheLocation,
};
pub use report::{
    DiscoveryReport, GlobalResolutionSkipReason, LocalTraversalStopReason,
    SkippedCeiling, SkippedCeilingReason,
};
pub use service::{CandidatePath, DiscoveryResult, DiscoveryService};

/// Diagnostics and warnings during discovery.
pub(super) mod diagnostics;

#[cfg(test)]
pub(crate) mod tests;
