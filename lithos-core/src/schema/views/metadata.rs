//! Shared metadata types for schema and property bank versions.

use std::{collections::HashMap, time::SystemTime};

use rkyv::{
    Archive, Deserialize, Serialize,
    with::{AsUnixTime, Map},
};

use crate::schema::{
    property::PropertyName,
    raw::property::{RawProperty, RawPropertyBankEntry},
};

// ─────────────────────────────────────────────────────────────────────────────
//  FileTimesMetadata
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
pub struct FileTimesMetadata {
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

impl FileTimesMetadata {
    /// Create new file times metadata.
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
    pub fn is_timestamp_match(
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
    content: [u8; 32],

    /// Per-property Blake3 hashes for incremental updates/resolution.
    properties: HashMap<PropertyName, [u8; 32]>,
}

impl HashMetadata {
    /// Create new hash metadata.
    #[inline]
    #[must_use]
    pub fn new(
        content: [u8; 32],
        properties: HashMap<PropertyName, [u8; 32]>,
    ) -> Self {
        Self {
            content,
            properties,
        }
    }

    /// Get content hash.
    #[inline]
    #[must_use]
    pub fn content(&self) -> &[u8; 32] {
        &self.content
    }

    /// Get property hashes.
    #[inline]
    #[must_use]
    pub fn properties(&self) -> &HashMap<PropertyName, [u8; 32]> {
        &self.properties
    }

    /// Check if content hash matches (for staleness detection).
    #[inline]
    #[must_use]
    pub fn is_content_match(&self, hash: &[u8; 32]) -> bool {
        self.content == *hash
    }

    /// Compute property hashes from raw properties (for schemas).
    ///
    /// This is the canonical hash computation used by schemas.
    #[inline]
    #[must_use]
    pub fn compute_property_hashes(
        properties: &std::collections::HashMap<Box<str>, RawProperty>,
    ) -> HashMap<PropertyName, [u8; 32]> {
        properties
            .iter()
            .filter_map(|(name, prop)| {
                let hash = Self::hash_property(prop);
                PropertyName::try_new(name.as_ref()).ok().map(|pn| (pn, hash))
            })
            .collect()
    }

    /// Compute property hashes from raw property bank entries.
    ///
    /// This is the canonical hash computation used by property banks.
    #[inline]
    #[must_use]
    pub fn compute_property_hashes_for_bank(
        properties: &std::collections::HashMap<Box<str>, RawPropertyBankEntry>,
    ) -> HashMap<PropertyName, [u8; 32]> {
        properties
            .iter()
            .filter_map(|(name, entry)| {
                let hash = Self::hash_property_bank_entry(entry);
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
        new_hashes: &HashMap<PropertyName, [u8; 32]>,
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

    /// Hash a single property bank entry.
    ///
    /// Uses JSON serialization to ensure consistent hashing.
    fn hash_property_bank_entry(entry: &RawPropertyBankEntry) -> [u8; 32] {
        let mut hasher = blake3::Hasher::new();

        // Serialize to JSON for consistent hashing
        if let Ok(json) = serde_json::to_string(entry) {
            hasher.update(json.as_bytes());
        } else {
            // Fallback: use debug representation
            hasher.update(format!("{entry:?}").as_bytes());
        }

        *hasher.finalize().as_bytes()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn file_times_matches_same_timestamps() {
        let now = SystemTime::now();
        let metadata = FileTimesMetadata::new(Some(now), Some(now));

        assert!(metadata.is_timestamp_match(Some(now), Some(now)));
    }

    #[test]
    fn file_times_no_match_different_timestamps() {
        let now = SystemTime::now();
        let later = now + std::time::Duration::from_secs(1);
        let metadata = FileTimesMetadata::new(Some(now), Some(now));

        assert!(!metadata.is_timestamp_match(Some(later), Some(now)));
    }

    #[test]
    fn hash_metadata_content_matches() {
        let hash = [1u8; 32];
        let metadata = HashMetadata::new(hash, HashMap::new());

        assert!(metadata.is_content_match(&hash));
        assert!(!metadata.is_content_match(&[2u8; 32]));
    }

    #[test]
    fn hash_metadata_changed_properties_detects_added() {
        let current = HashMetadata::new([0u8; 32], HashMap::new());
        let mut new_hashes = HashMap::new();
        let prop_name = PropertyName::try_new("title").unwrap();
        new_hashes.insert(prop_name.clone(), [1u8; 32]);

        let changed = current.changed_properties(&new_hashes);

        assert_eq!(changed.len(), 1);
        assert_eq!(changed.first(), Some(&prop_name));
    }

    #[test]
    fn hash_metadata_changed_properties_detects_removed() {
        let mut current_hashes = HashMap::new();
        let prop_name = PropertyName::try_new("title").unwrap();
        current_hashes.insert(prop_name.clone(), [1u8; 32]);
        let current = HashMetadata::new([0u8; 32], current_hashes);

        let changed = current.changed_properties(&HashMap::new());

        assert_eq!(changed.len(), 1);
        assert_eq!(changed.first(), Some(&prop_name));
    }

    #[test]
    fn hash_metadata_changed_properties_detects_modified() {
        let prop_name = PropertyName::try_new("title").unwrap();
        let mut current_hashes = HashMap::new();
        current_hashes.insert(prop_name.clone(), [1u8; 32]);
        let current = HashMetadata::new([0u8; 32], current_hashes);

        let mut new_hashes = HashMap::new();
        new_hashes.insert(prop_name.clone(), [2u8; 32]);

        let changed = current.changed_properties(&new_hashes);

        assert_eq!(changed.len(), 1);
        assert_eq!(changed.first(), Some(&prop_name));
    }
}
