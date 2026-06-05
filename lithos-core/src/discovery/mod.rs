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
//! - [`engine`] — [`DiscoveryEngine`] orchestrating vault discovery via
//!   explicit flag, environment variable, or ascending walk.
//! - [`error`] — [`DiscoveryError`] and [`VaultDiscoveryWarning`] types.
//! - [`policy`] — [`DiscoveryPolicy`], [`VaultSourceType`],
//!   [`GlobalSourceType`].
//! - [`marker`] — [`FoundRootMarker`], the typed handoff to Config.
//! - [`diagnostics`] — Non-fatal warning types emitted during discovery.
//! - [`probe`] — [`VaultRootProbe`], [`GlobalConfigProbe`], [`DiscoveryProbe`]
//!   trait, marker patterns.
//! - [`selector`] — Candidate selection and format promotion functions.
//! - [`walk`] — [`AscendingWalker`], [`DiscoveryBoundaries`], ceiling parsing.

pub(crate) mod engine;
pub(crate) mod error;
pub(crate) mod policy;
pub(crate) mod probe;
pub(crate) mod selector;
pub(crate) mod walk;

/// Diagnostics and warnings during discovery.
pub(crate) mod diagnostics;
