//! Path hashing for symlink-based config tracking and trust.
//!
//! Hashes a canonicalized path to a hex string suitable for use as a
//! filename in TRACKED_CONFIGS, TRUSTED_CONFIGS, and IGNORED_CONFIGS.
//! This is distinct from content hashing ([`Blake3Hash`]) — we hash the
//! path string, not the file contents.

use std::path::Path;

/// Hash a path to a hex string for use as a symlink filename.
///
/// Canonicalizes the path first so the same file has the same hash
/// regardless of symlinks, `.`, `..`, or relative path variations.
///
/// # Panics
///
/// Does not panic. If canonicalization fails, falls back to the raw path.
#[inline]
#[must_use]
#[allow(
    clippy::disallowed_methods,
    reason = "canonicalize ensures path consistency for persistent \
              trust/tracking symlinks"
)]
pub fn hash_path_to_str(path: &Path) -> String {
    let target =
        std::fs::canonicalize(path).unwrap_or_else(|_| path.to_path_buf());
    blake3::hash(target.as_os_str().as_encoded_bytes()).to_hex().to_string()
}
