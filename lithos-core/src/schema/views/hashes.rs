//! Hash-based content integrity tracking for staleness detection.
//!
//! ## Purpose
//!
//! This module provides [`HashRecord`], a dual-level hashing structure that
//! enables both **file-level staleness detection** (content hash) and
//! **incremental property resolution** (per-property hashes). This design
//! allows Lithos to answer two critical questions efficiently:
//!
//! 1. **"Has this file changed?"** (content hash comparison, O(1))
//! 2. **"Which properties changed?"** (per-property hash diff, O(k) where k =
//!    changed properties)
//!
//! ## Staleness Detection Strategy
//!
//! ### File-Level Staleness (Content Hash)
//!
//! The **content hash** ([`Blake3Hash`] of entire file contents) enables fast
//! staleness detection without parsing:
//!
//! ```text
//! Fast Path (file unchanged):
//! 1. Compute Blake3Hash of file contents (fast: ~3µs for 1KB file)
//! 2. Compare with stored content hash: record.is_content_match(&hash)
//! 3. Match → skip parsing, use cached domain aggregate
//! 4. Mismatch → proceed to slow path (re-parse + re-validate)
//! ```
//!
//! **Why Blake3?** Chosen for:
//! - **Speed**: 10x faster than SHA-256 on modern CPUs
//! - **Cryptographic quality**: Collision-resistant (256-bit output)
//! - **Streaming support**: Can hash files larger than RAM incrementally
//! - **Standard**: Wide adoption in modern tools (Nix, IPFS)
//!
//! ### Property-Level Incremental Resolution (Per-Property Hashes)
//!
//! The **property hashes** (map of `PropertyName` → `Blake3Hash`) enable
//! **incremental updates** when property bank changes:
//!
//! ```text
//! Property Bank Update Scenario:
//! 1. Property bank file changes (content hash mismatch)
//! 2. Compare property_hashes maps:
//!    old_record.properties vs. new_record.properties
//! 3. Identify changed properties: diff.keys()
//! 4. Query schemas: which reference changed properties?
//! 5. Re-expand ONLY affected schemas (not all)
//! ```
//!
//! **Example**:
//! - Property bank has 100 properties
//! - User edits 1 property definition
//! - 10 schemas reference the changed property
//! - **With per-property hashing**: Re-expand 10 schemas
//! - **Without**: Re-expand all schemas using property bank (wasteful)
//!
//! ## Content Hash vs. Per-Property Hash
//!
//! ### When to Use Content Hash
//!
//! - **Staleness checks**: "Has this file changed at all?"
//! - **Cache invalidation**: "Can I skip re-parsing?"
//! - **First-level filter**: Fast rejection of unchanged files
//!
//! ### When to Use Per-Property Hashes
//!
//! - **Incremental resolution**: "Which properties need re-expansion?"
//! - **Dependency tracking**: "Which schemas are affected by this change?"
//! - **Second-level analysis**: After content hash confirms file changed
//!
//! ## Hash Computation Strategy
//!
//! ### For Schemas
//!
//! - **Content hash**: `Blake3Hash::compute(file_contents.as_bytes())`
//! - **Property hashes**: Map each property definition to
//!   `Blake3Hash::compute(property_toml.as_bytes())`
//!
//! ### For Property Banks
//!
//! - **Content hash**: `Blake3Hash::compute(file_contents.as_bytes())`
//! - **Property hashes**: Map each registered property to
//!   `Blake3Hash::compute(property_definition.as_bytes())`
//!
//! ## Performance Characteristics
//!
//! - **Hash computation**: ~3µs per 1KB file (Blake3 on modern CPU)
//! - **Hash comparison**: O(1) memory comparison (32 bytes)
//! - **Per-property diff**: O(k) where k = number of properties
//! - **Zero-copy access**: Via `ArchivedHashRecord` (no deserialization
//!   overhead)
//!
//! ## Zero-Copy Access
//!
//! [`HashRecord`] is stored via `rkyv` serialization, enabling zero-copy
//! access in hot paths:
//!
//! ```rust,ignore
//! // Hot path: zero-copy staleness check (no allocation)
//! let archived: &ArchivedHashRecord = view.current().hashes();
//! if archived.is_content_match(&current_hash) {
//!     // File unchanged, use cached aggregate
//! }
//! ```
//!
//! The archived type ([`ArchivedHashRecord`]) implements the same API as the
//! owned type, ensuring consistent behavior across runtime and storage.
//!
//! [`ArchivedHashRecord`]: hashes::ArchivedHashRecord

use std::collections::HashMap;

use rkyv::{Archive, Deserialize, Serialize};

use crate::{schema::property::PropertyName, support::hash::Blake3Hash};

// ─────────────────────────────────────────────────────────────────────────────
//  HashRecord
// ─────────────────────────────────────────────────────────────────────────────

/// Content and property hash metadata shared by schema and property bank
/// versions.
///
/// Used for staleness detection (content hash) and incremental resolution
/// (property hashes).
///
/// # Examples
///
/// ```
/// # use lithos_core::schema::views::HashRecord;
/// # use lithos_core::support::hash::Blake3Hash;
/// # use std::collections::HashMap;
/// #
/// let content_hash = Blake3Hash::compute(b"content");
/// let property_hashes = HashMap::new();
/// let record = HashRecord::new(content_hash, property_hashes);
///
/// assert!(record.is_content_match(&content_hash));
/// ```
#[derive(Debug, Clone, PartialEq, Eq, Archive, Serialize, Deserialize)]
pub struct HashRecord {
    /// Blake3 hash of content for staleness detection.
    content: Blake3Hash,

    /// Per-property Blake3 hashes for incremental resolution.
    properties: HashMap<PropertyName, Blake3Hash>,
}

impl HashRecord {
    /// Creates a new hash metadata record.
    ///
    /// # Examples
    ///
    /// ```
    /// # use lithos_core::schema::views::HashRecord;
    /// # use lithos_core::support::hash::Blake3Hash;
    /// # use std::collections::HashMap;
    /// #
    /// let content_hash = Blake3Hash::compute(b"content");
    /// let property_hashes = HashMap::new();
    /// let record = HashRecord::new(content_hash, property_hashes);
    /// ```
    #[inline]
    #[must_use]
    pub fn new(
        content: Blake3Hash,
        properties: HashMap<PropertyName, Blake3Hash>,
    ) -> Self {
        Self {
            content,
            properties,
        }
    }

    /// Returns the content hash ([`Blake3Hash`]) for staleness detection.
    ///
    /// # Examples
    ///
    /// ```
    /// # use lithos_core::schema::views::HashRecord;
    /// # use lithos_core::support::hash::Blake3Hash;
    /// # use std::collections::HashMap;
    /// #
    /// let content = Blake3Hash::compute(b"test");
    /// let record = HashRecord::new(content, HashMap::new());
    /// assert_eq!(record.content(), &content);
    /// ```
    #[inline]
    #[must_use]
    pub fn content(&self) -> &Blake3Hash {
        &self.content
    }

    /// Returns per-property hashes for incremental resolution.
    ///
    /// Returns a map of [`PropertyName`] to [`Blake3Hash`] for properties
    /// that need incremental updates.
    ///
    /// # Examples
    ///
    /// ```
    /// # use lithos_core::schema::views::HashRecord;
    /// # use lithos_core::support::hash::Blake3Hash;
    /// # use std::collections::HashMap;
    /// #
    /// let record = HashRecord::new(Blake3Hash::compute(b"c"), HashMap::new());
    /// assert!(record.properties().is_empty());
    /// ```
    #[inline]
    #[must_use]
    pub fn properties(&self) -> &HashMap<PropertyName, Blake3Hash> {
        &self.properties
    }

    /// Returns `true` if the provided content hash matches this record.
    ///
    /// Used for fast staleness checks without deserializing the full schema.
    ///
    /// # Examples
    ///
    /// ```
    /// # use lithos_core::schema::views::HashRecord;
    /// # use lithos_core::support::hash::Blake3Hash;
    /// # use std::collections::HashMap;
    /// #
    /// let content = Blake3Hash::compute(b"test");
    /// let record = HashRecord::new(content, HashMap::new());
    /// assert!(record.is_content_match(&content));
    /// ```
    #[inline]
    #[must_use]
    pub fn is_content_match(&self, hash: &Blake3Hash) -> bool {
        self.content.is_match(hash)
    }
}

impl ArchivedHashRecord {
    /// Returns `true` if the archived content hash matches (zero-copy).
    ///
    /// Used for zero-copy staleness checks without deserialization.
    ///
    /// # Examples
    ///
    /// ```
    /// # use lithos_core::schema::views::hashes::{HashRecord, ArchivedHashRecord};
    /// # use lithos_core::support::hash::Blake3Hash;
    /// # use std::collections::HashMap;
    /// # use rkyv::access;
    /// #
    /// let hash = Blake3Hash::compute(b"test");
    /// let record = HashRecord::new(hash, HashMap::new());
    /// let bytes = rkyv::to_bytes::<rkyv::rancor::Error>(&record).unwrap();
    /// let archived =
    ///     rkyv::access::<ArchivedHashRecord, rkyv::rancor::Error>(&bytes)
    ///         .unwrap();
    ///
    /// assert!(archived.is_content_match(&hash));
    /// ```
    #[inline]
    #[must_use]
    pub fn is_content_match(&self, hash: &Blake3Hash) -> bool {
        self.content.is_match(hash)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hash_record_content_matches() {
        let hash = Blake3Hash::compute(b"test");
        let record = HashRecord::new(hash, HashMap::new());

        assert!(record.is_content_match(&hash));
        assert!(!record.is_content_match(&Blake3Hash::compute(b"other")));
    }
}
