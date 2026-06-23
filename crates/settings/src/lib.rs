#![feature(trivial_bounds)]
//! Consolidated settings adapter for Trace.
//!
//! This crate unifies discovery (locating configuration files) and config
//! (parsing, validating, and merging them) into a single inbound adapter.

pub mod config;
pub mod discovery;

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
    aggregate::Config,
    builder::Builder,
    repository::{ReadRepository, Repository, WriteRepository},
    storage::{RedbRepository, RedbStorage},
};
pub use discovery::{
    context,
    context::{DiscoveryContext, DiscoveryEnv, DiscoveryFlags},
    dirs, env,
    env::EnvVars,
    error::DiscoveryError,
    location,
    location::{
        CacheLocation, CacheRoot, GlobalCacheLocation, LocalCacheLocation,
    },
    port, report,
    report::DiscoveryReport,
    service,
    service::{
        CandidatePath, DiscoveryResult, DiscoveryService,
        DiscoveryServiceConfig,
    },
};
