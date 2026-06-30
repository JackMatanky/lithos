#![feature(trivial_bounds)]
//! Consolidated settings adapter for Trace.
//!
//! This crate unifies discovery (locating configuration files) and config
//! (parsing, validating, and merging them) into a single inbound adapter.

pub mod candidate;
pub mod config;
pub mod discovery;
pub mod env_var;
pub mod location;
pub mod os_dirs;
pub mod service;

pub use candidate::CandidatePath;
// To satisfy the mechanical `crate::xxx -> crate::xxx`
// and `crate::xxx -> crate::xxx` import rewrites:
#[cfg(any(test, feature = "testing"))]
pub use config::storage::testing::InMemoryRepository;
pub use config::{
    aggregate, builder, cache, error, events, frontmatter, global, logging,
    merger, processor, raw, repository, schema, storage, task, template, value,
    vault,
};
// Re-export specific boundary APIs as requested
pub use config::{
    aggregate::AppConfig,
    builder::Builder,
    global::GlobalConfig,
    repository::{ReadRepository, Repository, WriteRepository},
    storage::{RedbRepository, RedbStorage},
    vault::LocalConfig,
};
pub use discovery::{
    context,
    context::{DiscoveryContext, DiscoveryEnv, DiscoveryFlags},
    dirs,
    error::DiscoveryError,
    location::{
        CacheLocation, CacheRoot, GlobalCacheLocation, LocalCacheLocation,
    },
    outcome::DiscoveryOutcome,
    port, report,
    report::DiscoveryReport,
    service::{DiscoveryResult, DiscoveryService, DiscoveryServiceConfig},
};
pub use env_var::SettingsEnvVars;
pub use service::{
    ConfigBuilderOptions, DiscoveryOptions, Service, SettingsError,
    SettingsService, TrustMode,
};
