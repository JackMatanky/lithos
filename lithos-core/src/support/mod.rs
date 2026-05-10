//! Crate-private implementation support for `lithos-core`.
//!
//! This module is the internal "engine room" for helpers that are useful
//! across multiple internal modules but are not part of the public contract
//! surface. Items here may change freely as implementation needs evolve.
//!
//! Boundary rule:
//! - Public, stable contracts belong in `crate::utils`.
//! - `crate::support` stays crate-private and must not be exposed to external
//!   consumers.

/// BLAKE3 hashing utilities and types.
pub(crate) mod hash;

#[expect(
    unused_imports,
    reason = "crate-private support facade keeps ergonomic internal imports"
)]
pub(crate) use hash::{
    Blake3Hash, Blake3HashIndex, HashInput, hash_structured,
};
