//! Shared metadata types for schema and property bank versions.

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
#[derive(Debug, Clone, PartialEq, Eq, Archive, Serialize, Deserialize)]
pub struct HashRecord {
    /// Blake3 hash of file content for staleness detection.
    content: Blake3Hash,

    /// Per-property Blake3 hashes for incremental updates/resolution.
    properties: HashMap<PropertyName, Blake3Hash>,
}

impl HashRecord {
    /// Create new hash metadata.
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

    /// Get content hash.
    #[inline]
    #[must_use]
    pub fn content(&self) -> &Blake3Hash {
        &self.content
    }

    /// Get property hashes.
    #[inline]
    #[must_use]
    pub fn properties(&self) -> &HashMap<PropertyName, Blake3Hash> {
        &self.properties
    }

    /// Check if content hash matches (for staleness detection).
    #[inline]
    #[must_use]
    pub fn is_content_match(&self, hash: &Blake3Hash) -> bool {
        self.content.is_match(hash)
    }
}

impl ArchivedHashRecord {
    /// Check if archived content hash matches (for zero-copy staleness checks).
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
