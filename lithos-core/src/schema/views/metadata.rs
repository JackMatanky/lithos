//! Shared metadata types for schema and property bank versions.

use std::{
    collections::HashMap,
    time::{SystemTime, UNIX_EPOCH},
};

use rkyv::{
    Archive, Deserialize, Serialize,
    with::{AsUnixTime, Map},
};

use crate::{
    schema::{property::PropertyName, raw::property::RawPropertyMap},
    support::hash::Blake3Hash,
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

    /// Convert a [`SystemTime`] into Unix seconds used by archived metadata.
    #[inline]
    #[must_use]
    fn system_time_to_unix_seconds(time: SystemTime) -> Option<i64> {
        let duration = time.duration_since(UNIX_EPOCH).ok()?;
        i64::try_from(duration.as_secs()).ok()
    }
}

impl ArchivedFileTimesMetadata {
    /// Check whether archived timestamps match provided filesystem times.
    #[inline]
    #[must_use]
    pub fn is_timestamp_match(
        &self,
        created_at: Option<SystemTime>,
        modified_at: Option<SystemTime>,
    ) -> bool {
        self.created_at
            .as_ref()
            .and_then(|time| i64::try_from(time.as_secs()).ok())
            == created_at
                .and_then(FileTimesMetadata::system_time_to_unix_seconds)
            && self
                .modified_at
                .as_ref()
                .and_then(|time| i64::try_from(time.as_secs()).ok())
                == modified_at
                    .and_then(FileTimesMetadata::system_time_to_unix_seconds)
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
