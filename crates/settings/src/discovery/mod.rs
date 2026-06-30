//! Pre-config filesystem discovery for Traces.
//!
//! This context owns the first phase of the `Discovery -> Config -> Indexer`
//! pipeline. It locates the vault root and the root marker file on disk and
//! returns typed path/format metadata, including the resolved cache root. It
//! does **not** parse, merge, validate, or hash config contents — those
//! responsibilities belong to `Config`.
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
//! - **`port`** — Inbound port trait (`DiscoveryPort`).
//! - **`report`** — Non-fatal diagnostic output ([`DiscoveryReport`]).
//! - **`service`** — Concrete service ([`DiscoveryService`]) and discovery
//!   result boundary data ([`DiscoveryResult`]).
//! - **`error`** — Fatal error types (`DiscoveryError`).
//! - **`processor`** — Internal typestate pipeline (crate-private).
//! - **`probe`** / **`walk`** — Internal filesystem helpers (crate-private).
//!
//! # Usage Example
//!
//! ```rust,ignore
//! use traces_settings::config::{
//!     DiscoveryService, DiscoveryFlags, DiscoveryEnv,
//!     port::DiscoveryPort,
//!     service::DiscoveryServiceConfig,
//! };
//! use traces_settings::config::context::DiscoveryContext;
//!
//! let service = DiscoveryService::new(DiscoveryServiceConfig::default())
//!     .expect("valid config");
//! let ctx = DiscoveryContext::new(std::path::Path::new("."))
//!     .expect("valid anchor");
//! let (result, report) = service.discover(&ctx).expect("discovery succeeded");
//! println!("cache root: {:?}", result.cache_root().path());
//! ```
//!
//! `Config`

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
