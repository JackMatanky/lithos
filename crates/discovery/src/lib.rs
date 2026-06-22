#![feature(trivial_bounds)]
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
//! use trace_discovery::{
//!     DiscoveryService, DiscoveryFlags, DiscoveryEnv,
//!     port::DiscoveryPort,
//!     service::DiscoveryServiceConfig,
//! };
//! use trace_discovery::context::DiscoveryContext;
//!
//! let service = DiscoveryService::new(DiscoveryServiceConfig::default())
//!     .expect("valid config");
//! let ctx = DiscoveryContext::new(std::path::Path::new("."))
//!     .expect("valid anchor");
//! let (result, report) = service.discover(&ctx).expect("discovery succeeded");
//! println!("cache root: {:?}", result.cache_root().path());
//! ```
//!
//! [`Config`]: trace_config
//! [`DiscoveryPort`]: crate::port::DiscoveryPort
//! [`DiscoveryError`]: crate::error::DiscoveryError

pub(crate) mod context;
pub mod dirs;
pub(crate) mod env;
pub mod error;
pub mod location;
pub(crate) mod policy;
pub mod port;
pub(crate) mod probe;
pub(crate) mod processor;
pub mod report;
pub mod service;
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
pub use service::{
    CandidatePath, DiscoveryResult, DiscoveryService, DiscoveryServiceConfig,
};
