//! Shared metadata types for schema and property bank versions.

use std::{collections::BTreeMap, time::SystemTime};

use rkyv::{
    Archive, Deserialize, Serialize,
    with::{AsUnixTime, Map},
};

use crate::schema::{property::PropertyName, raw::property::RawProperty};

// ─────────────────────────────────────────────────────────────────────────────
//  FileVersionMetadata
// ─────────────────────────────────────────────────────────────────────────────

/// File timestamp metadata shared by schema and property bank versions.
///
/// Tracks when the file was created, modified, and when this version
/// was recorded in the database.
#[expect(
    clippy::struct_field_names,
    reason = "Timestamp fields (created_at, modified_at, recorded_at) are \
              semantically distinct"
)]
#[derive(Debug, Clone, PartialEq, Eq, Archive, Serialize, Deserialize)]
#[rkyv(attr(expect(
    clippy::struct_field_names,
    reason = "Archived type mirrors source struct field names"
)))]
pub struct FileVersionMetadata {
    /// File creation timestamp from filesystem.
    #[rkyv(with = Map<AsUnixTime>)]
    created_at: Option<SystemTime>,

    /// File modification timestamp from filesystem.
    #[rkyv(with = Map<AsUnixTime>)]
    modified_at: Option<SystemTime>,

    /// When this version was recorded in the database.
    #[rkyv(with = AsUnixTime)]
    recorded_at: SystemTime,
}

impl FileVersionMetadata {
    /// Create new file version metadata.
    #[inline]
    #[must_use]
    pub fn new(
        created_at: Option<SystemTime>,
        modified_at: Option<SystemTime>,
    ) -> Self {
        Self {
            created_at,
            modified_at,
            recorded_at: SystemTime::now(),
        }
    }

    /// Get file creation timestamp.
    #[inline]
    #[must_use]
    pub fn created_at(&self) -> Option<SystemTime> {
        self.created_at
    }

    /// Get file modification timestamp.
    #[inline]
    #[must_use]
    pub fn modified_at(&self) -> Option<SystemTime> {
        self.modified_at
    }

    /// Get database recording timestamp.
    #[inline]
    #[must_use]
    pub fn recorded_at(&self) -> SystemTime {
        self.recorded_at
    }

    /// Check if timestamps match (for staleness detection).
    #[inline]
    #[must_use]
    pub fn matches(
        &self,
        created_at: Option<SystemTime>,
        modified_at: Option<SystemTime>,
    ) -> bool {
        self.created_at == created_at && self.modified_at == modified_at
    }
}

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
    content_hash: [u8; 32],

    /// Per-property Blake3 hashes for incremental updates/resolution.
    property_hashes: BTreeMap<PropertyName, [u8; 32]>,
}

impl HashMetadata {
    /// Create new hash metadata.
    #[inline]
    #[must_use]
    pub fn new(
        content_hash: [u8; 32],
        property_hashes: BTreeMap<PropertyName, [u8; 32]>,
    ) -> Self {
        Self {
            content_hash,
            property_hashes,
        }
    }

    /// Get content hash.
    #[inline]
    #[must_use]
    pub fn content_hash(&self) -> &[u8; 32] {
        &self.content_hash
    }

    /// Get property hashes.
    #[inline]
    #[must_use]
    pub fn property_hashes(&self) -> &BTreeMap<PropertyName, [u8; 32]> {
        &self.property_hashes
    }

    /// Check if content hash matches (for staleness detection).
    #[inline]
    #[must_use]
    pub fn content_matches(&self, hash: &[u8; 32]) -> bool {
        self.content_hash == *hash
    }

    /// Compute property hashes from raw properties.
    ///
    /// This is the canonical hash computation used by both schemas
    /// and property banks.
    #[inline]
    #[must_use]
    pub fn compute_property_hashes(
        properties: &std::collections::HashMap<Box<str>, RawProperty>,
    ) -> BTreeMap<PropertyName, [u8; 32]> {
        properties
            .iter()
            .filter_map(|(name, prop)| {
                let hash = Self::hash_property(prop);
                PropertyName::try_new(name.as_ref()).ok().map(|pn| (pn, hash))
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
        new_hashes: &BTreeMap<PropertyName, [u8; 32]>,
    ) -> Vec<PropertyName> {
        let mut changed = Vec::new();

        // Find modified or added properties
        for (name, new_hash) in new_hashes {
            if self.property_hashes.get(name) != Some(new_hash) {
                changed.push(name.clone());
            }
        }

        // Find removed properties
        for name in self.property_hashes.keys() {
            if !new_hashes.contains_key(name) {
                changed.push(name.clone());
            }
        }

        changed
    }

    /// Hash a single property definition.
    ///
    /// Uses JSON serialization to ensure consistent hashing across all
    /// property types and variants.
    fn hash_property(prop: &RawProperty) -> [u8; 32] {
        let mut hasher = blake3::Hasher::new();

        // Serialize to JSON for consistent hashing
        // This is fine for metadata operations (not a hot path)
        if let Ok(json) = serde_json::to_string(prop) {
            hasher.update(json.as_bytes());
        } else {
            // Fallback: use debug representation
            hasher.update(format!("{prop:?}").as_bytes());
        }

        *hasher.finalize().as_bytes()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn file_metadata_matches_same_timestamps() {
        let now = SystemTime::now();
        let metadata = FileVersionMetadata::new(Some(now), Some(now));

        assert!(metadata.matches(Some(now), Some(now)));
    }

    #[test]
    fn file_metadata_no_match_different_timestamps() {
        let now = SystemTime::now();
        let later = now + std::time::Duration::from_secs(1);
        let metadata = FileVersionMetadata::new(Some(now), Some(now));

        assert!(!metadata.matches(Some(later), Some(now)));
    }

    #[test]
    fn hash_metadata_content_matches() {
        let hash = [1u8; 32];
        let metadata = HashMetadata::new(hash, BTreeMap::new());

        assert!(metadata.content_matches(&hash));
        assert!(!metadata.content_matches(&[2u8; 32]));
    }

    #[test]
    fn hash_metadata_changed_properties_detects_added() {
        let current = HashMetadata::new([0u8; 32], BTreeMap::new());
        let mut new_hashes = BTreeMap::new();
        let prop_name = PropertyName::try_new("title").unwrap();
        new_hashes.insert(prop_name.clone(), [1u8; 32]);

        let changed = current.changed_properties(&new_hashes);

        assert_eq!(changed.len(), 1);
        assert_eq!(changed.first(), Some(&prop_name));
    }

    #[test]
    fn hash_metadata_changed_properties_detects_removed() {
        let mut current_hashes = BTreeMap::new();
        let prop_name = PropertyName::try_new("title").unwrap();
        current_hashes.insert(prop_name.clone(), [1u8; 32]);
        let current = HashMetadata::new([0u8; 32], current_hashes);

        let changed = current.changed_properties(&BTreeMap::new());

        assert_eq!(changed.len(), 1);
        assert_eq!(changed.first(), Some(&prop_name));
    }

    #[test]
    fn hash_metadata_changed_properties_detects_modified() {
        let prop_name = PropertyName::try_new("title").unwrap();
        let mut current_hashes = BTreeMap::new();
        current_hashes.insert(prop_name.clone(), [1u8; 32]);
        let current = HashMetadata::new([0u8; 32], current_hashes);

        let mut new_hashes = BTreeMap::new();
        new_hashes.insert(prop_name.clone(), [2u8; 32]);

        let changed = current.changed_properties(&new_hashes);

        assert_eq!(changed.len(), 1);
        assert_eq!(changed.first(), Some(&prop_name));
    }
}
