//! Pre-config filesystem discovery for Lithos.
//!
//! This context owns the first phase of the `Discovery -> Config -> Indexer`
//! pipeline. It locates the vault root and the root marker file on disk and
//! returns typed path/format metadata. It does **not** parse, merge, validate,
//! or hash config contents — those responsibilities belong to [`Config`].
//!
//! # Boundary Invariants
//!
//! - **Metadata Only**: Discovery returns path, source, and format metadata
//!   only. It never reads file contents beyond existence checks.
//! - **One-way Flow**: Config consumes Discovery outputs; Discovery never
//!   imports Config types.
//!
//! # Usage Example
//!
//! ```rust,ignore
//! use lithos_core::discovery::{DiscoveryEngine, DiscoveryPolicy, DiscoveryInput};
//! use std::path::Path;
//!
//! let policy = DiscoveryPolicy::default();
//! let engine = DiscoveryEngine::new(policy);
//!
//! let input = DiscoveryInput {
//!     flag_path: None,
//!     env_path: None,
//!     cwd: Path::new("."),
//!     ceiling_dirs_raw: None,
//! };
//!
//! let result = engine.find_vault(&input)?;
//! if let Some(root) = result.root {
//!     println!("Found vault root at: {:?}", root);
//! }
//! ```
//!
//! # Architecture
//!
//! - **`engine`** — The main entry point ([`DiscoveryEngine`]).
//! - **`policy`** — Precedence and behavioral configuration.
//! - **`probe`** — Directory examination for marker patterns.
//! - **`walk`** — Ascending directory traversal with boundary enforcement.
//! - **`selector`** — Tie-breaking logic for multiple discovered markers.
//! - **`marker`** — Shared metadata types ([`DiscoveredMarker`]).
//! - **`diagnostics`** — Structured non-fatal warnings
//!   ([`VaultDiscoveryWarning`]).
//! - **`error`** — Fatal error types ([`DiscoveryError`]).
//!
//! [`Config`]: crate::config
//! [`DiscoveryEngine`]: crate::discovery::engine::DiscoveryEngine
//! [`DiscoveredMarker`]: crate::discovery::engine::DiscoveredMarker
//! [`VaultDiscoveryWarning`]: crate::discovery::diagnostics::VaultDiscoveryWarning
//! [`DiscoveryError`]: crate::discovery::error::DiscoveryError

pub(crate) mod context;
pub(crate) mod engine;
pub(crate) mod error;
pub(crate) mod policy;
pub(crate) mod port;
pub(super) mod probe;
pub(super) mod processor;
pub(crate) mod report;
pub(super) mod selector;
pub(crate) mod service;
pub(super) mod walk;

/// Diagnostics and warnings during discovery.
pub(super) mod diagnostics;

#[cfg(test)]
pub(crate) mod tests;
