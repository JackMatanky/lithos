#![feature(trivial_bounds)]
//! Crate-private implementation support for `traces-core`.
//!
//! This module is the internal "engine room" for helpers that are useful
//! across multiple internal modules but are not part of the public contract
//! surface. Items here may change freely as implementation needs evolve.
//!
//! Boundary rule:
//! - Public, stable contracts belong in `trace_utils`.
//! - `crate` stays crate-private and must not be exposed to external consumers.

/// BLAKE3 content hashing utilities and types.
mod content_hash;
/// Indexed hash map for change detection.
mod hash_index;
/// Path hashing for symlink-based tracking and trust.
mod hash_path;

#[doc(hidden)]
pub use content_hash::{
    Blake3Hash, HasContentHash, HasContentHashMut, HashInput, hash_structured,
};
#[doc(hidden)]
pub use hash_index::{Blake3HashIndex, HasHashIndex, HasHashIndexMut};
#[doc(hidden)]
pub use hash_path::hash_path_to_str;
