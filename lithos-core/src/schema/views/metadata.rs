//! Shared metadata types for schema and property bank versions.

use std::collections::HashMap;

use rkyv::{Archive, Deserialize, Serialize};

use crate::{
    schema::{property::PropertyName, raw::property::RawPropertyMap},
    support::hash::Blake3Hash,
};

// ─────────────────────────────────────────────────────────────────────────────
//  HashMetadata
// ─────────────────────────────────────────────────────────────────────────────

/// Content and property hash metadata shared by schema and property bank
/// versions.
///
/// Used for staleness detection (content hash) and incremental resolution
/// (property hashes).
#[derive(Debug, Clone, PartialEq, Eq, Archive, Serialize, Deserialize)]
pub struct HashMetadata {
    /// Blake3 hash of file content for staleness detection.
    content: Blake3Hash,

    /// Per-property Blake3 hashes for incremental updates/resolution.
    properties: HashMap<PropertyName, Blake3Hash>,
}

impl HashMetadata {
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

    /// Compute property hashes from a validated property map.
    ///
    /// This is the canonical hash computation used by both schemas and
    /// property banks.
    #[inline]
    #[must_use]
    pub fn compute_property_hashes<T: serde::Serialize + std::fmt::Debug>(
        properties: &RawPropertyMap<T>,
    ) -> HashMap<PropertyName, Blake3Hash> {
        properties
            .iter()
            .map(|(name, prop)| {
                let hash = Blake3Hash::compute_json(prop);
                (name.clone(), hash)
            })
            .collect()
    }

    /// Compute changed properties by comparing with new hashes.
    ///
    /// Returns property names that were:
    /// - Added (in new but not in current)
    /// - Removed (in current but not in new)
    /// - Modified (different hash)
    #[inline]
    #[must_use]
    pub fn changed_properties(
        &self,
        new_hashes: &HashMap<PropertyName, Blake3Hash>,
    ) -> Vec<PropertyName> {
        let mut changed = Vec::new();

        // Find modified or added properties
        #[expect(
            clippy::iter_over_hash_type,
            reason = "HashMap iteration is intentional for detecting changed \
                      properties"
        )]
        for (name, new_hash) in new_hashes {
            if self.properties.get(name) != Some(new_hash) {
                changed.push(name.clone());
            }
        }

        // Find removed properties
        #[expect(
            clippy::iter_over_hash_type,
            reason = "HashMap keys iteration is intentional for detecting \
                      removed properties"
        )]
        for name in self.properties.keys() {
            if !new_hashes.contains_key(name) {
                changed.push(name.clone());
            }
        }

        changed
    }
}

impl ArchivedHashMetadata {
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
    fn hash_metadata_content_matches() {
        let hash = Blake3Hash::compute(b"test");
        let metadata = HashMetadata::new(hash, HashMap::new());

        assert!(metadata.is_content_match(&hash));
        assert!(!metadata.is_content_match(&Blake3Hash::compute(b"other")));
    }

    #[test]
    fn hash_metadata_changed_properties_detects_added() {
        let current =
            HashMetadata::new(Blake3Hash::new([0u8; 32]), HashMap::new());
        let mut new_hashes = HashMap::new();
        let prop_name = PropertyName::try_new("title").unwrap();
        let hash = Blake3Hash::new([1u8; 32]);
        new_hashes.insert(prop_name.clone(), hash);

        let changed = current.changed_properties(&new_hashes);

        assert_eq!(changed.len(), 1);
        assert_eq!(changed.first(), Some(&prop_name));
    }

    #[test]
    fn hash_metadata_changed_properties_detects_removed() {
        let mut current_hashes = HashMap::new();
        let prop_name = PropertyName::try_new("title").unwrap();
        let hash = Blake3Hash::new([1u8; 32]);
        current_hashes.insert(prop_name.clone(), hash);
        let current =
            HashMetadata::new(Blake3Hash::new([0u8; 32]), current_hashes);

        let changed = current.changed_properties(&HashMap::new());

        assert_eq!(changed.len(), 1);
        assert_eq!(changed.first(), Some(&prop_name));
    }

    #[test]
    fn hash_metadata_changed_properties_detects_modified() {
        let prop_name = PropertyName::try_new("title").unwrap();
        let mut current_hashes = HashMap::new();
        let hash1 = Blake3Hash::new([1u8; 32]);
        current_hashes.insert(prop_name.clone(), hash1);
        let current =
            HashMetadata::new(Blake3Hash::new([0u8; 32]), current_hashes);

        let mut new_hashes = HashMap::new();
        let hash2 = Blake3Hash::new([2u8; 32]);
        new_hashes.insert(prop_name.clone(), hash2);

        let changed = current.changed_properties(&new_hashes);

        assert_eq!(changed.len(), 1);
        assert_eq!(changed.first(), Some(&prop_name));
    }
}
