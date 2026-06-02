//! Pre-config filesystem discovery for Lithos.
//!
//! This context owns the first phase of the `Discovery -> Config -> Indexer`
//! pipeline. It locates the vault root and the root marker file on disk and
//! returns typed path/format metadata. It does **not** parse, merge, validate,
//! or hash config contents — those responsibilities belong to `Config`.
//!
//! # Boundary invariants
//!
//! - Discovery returns path, source, and format metadata only.
//! - Discovery does not import `ConfigLocation`, `LocalConfigLocation`, or any
//!   other Config-owned classification type.
//! - Config consumes Discovery outputs; Discovery never imports Config types.
//!
//! # Exports
//!
//! - [`resolver`] — [`RootResolver`] and supporting types for vault root
//!   resolution via explicit flag, environment variable, or ascending search.
//! - [`marker`] — [`FoundRootMarker`], the typed handoff from resolver to
//!   Config.
//! - [`diagnostics`] — Non-fatal warning types emitted during discovery.

/// Diagnostics and warnings during discovery.
pub mod diagnostics;
/// Root marker contract: the typed output of vault root resolution.
pub mod marker;
/// Root resolution logic.
pub mod resolver;

pub(crate) use diagnostics::*;
pub(crate) use marker::*;
