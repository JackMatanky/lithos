//! Template view types for freshness/cache tracking.
//!
//! Provides [`RawTemplateView`], a flat struct carrying the path key, content
//! hash, file metadata, and recorded time for a cached template. This is a
//! flat struct — NOT a versioned ring buffer (templates have no inheritance
//! metadata that would justify the ring buffer pattern).

use std::time::SystemTime;

use rkyv::{Archive, Deserialize, Serialize, with::AsUnixTime};

use crate::{
    fs::{PathKey, metadata::FileMetadata},
    support::content_hash::{Blake3Hash, HasContentHash, HasContentHashMut},
};

// ============================================================================
// RawTemplateView
// ============================================================================

/// Flat freshness/cache struct for a raw template file.
///
/// Carries the minimum data needed for staleness detection:
/// - `path` — vault-relative storage key
/// - `content_hash` — BLAKE3 hash of the file content
/// - `metadata` — file size and timestamps for fast pre-hash check
/// - `recorded_at` — time this view was recorded
///
/// This is a flat struct. Unlike `RawSchemaView` (which uses a versioned ring
/// buffer for incremental inheritance resolution), templates have no
/// inheritance metadata and need no versioning.
///
/// Implements [`HasContentHash`] and [`HasContentHashMut`] for integration
/// with the content-hash-based staleness detection infrastructure.
#[derive(Debug, Clone, Archive, Serialize, Deserialize)]
pub struct RawTemplateView {
    /// Vault-relative storage path.
    path: PathKey,
    /// BLAKE3 hash of the template file content.
    content_hash: Blake3Hash,
    /// File metadata for staleness detection.
    metadata: FileMetadata,
    /// Time this view was recorded.
    #[rkyv(with = AsUnixTime)]
    recorded_at: SystemTime,
}

impl RawTemplateView {
    /// Constructs a new `RawTemplateView` with the given path, hash, metadata,
    /// and recorded timestamp.
    #[inline]
    #[must_use]
    #[allow(dead_code, reason = "used by Template Processor (Issue 04)")]
    pub(crate) fn new(
        path: PathKey,
        content_hash: Blake3Hash,
        metadata: FileMetadata,
        recorded_at: SystemTime,
    ) -> Self {
        Self {
            path,
            content_hash,
            metadata,
            recorded_at,
        }
    }

    /// Returns the vault-relative storage path.
    #[inline]
    #[must_use]
    pub const fn path(&self) -> &PathKey {
        &self.path
    }

    /// Returns a reference to the content hash.
    #[inline]
    #[must_use]
    #[allow(
        dead_code,
        reason = "used by staleness detection in Template Processor (Issue 04)"
    )]
    pub(crate) const fn content_hash(&self) -> &Blake3Hash {
        &self.content_hash
    }

    /// Returns a reference to the file metadata.
    #[inline]
    #[must_use]
    pub const fn metadata(&self) -> &FileMetadata {
        &self.metadata
    }

    /// Returns the recorded time.
    #[inline]
    #[must_use]
    pub const fn recorded_at(&self) -> SystemTime {
        self.recorded_at
    }

    /// Updates the view metadata to the provided new metadata and records the
    /// update time.
    #[inline]
    pub(crate) fn update_metadata(&mut self, metadata: FileMetadata) {
        self.metadata = metadata;
        self.recorded_at = SystemTime::now();
    }
}

impl HasContentHash for RawTemplateView {
    #[inline]
    fn content_hash(&self) -> &Blake3Hash {
        &self.content_hash
    }
}

impl HasContentHashMut for RawTemplateView {
    #[inline]
    fn set_content_hash(&mut self, hash: Blake3Hash) {
        self.content_hash = hash;
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use crate::fs::metadata::FsTimes;

    fn make_view() -> RawTemplateView {
        let path = PathKey::try_new("templates/standup.md").unwrap();
        let hash = Blake3Hash::from_bytes(b"test content");
        let metadata = FileMetadata::new(FsTimes::new(None, None), 100, false);
        let recorded_at = SystemTime::now();
        RawTemplateView::new(path, hash, metadata, recorded_at)
    }

    mod raw_template_view {
        use super::*;

        mod constructor {
            use super::*;

            #[test]
            fn new_stores_all_fields() {
                let path = PathKey::try_new("templates/note.md").unwrap();
                let hash = Blake3Hash::from_bytes(b"data");
                let times = FsTimes::new(Some(SystemTime::UNIX_EPOCH), None);
                let metadata = FileMetadata::new(times, 42, false);
                let recorded_at = SystemTime::now();

                let view = RawTemplateView::new(
                    path.clone(),
                    hash,
                    metadata.clone(),
                    recorded_at,
                );

                assert_eq!(view.path(), &path);
                assert_eq!(view.content_hash(), &hash);
                assert_eq!(view.metadata(), &metadata);
            }

            #[test]
            fn recorded_at_accessor_returns_stored_time() {
                let view = make_view();
                let epoch = SystemTime::UNIX_EPOCH;
                assert!(
                    view.recorded_at() > epoch,
                    "recorded_at should be after Unix epoch"
                );
            }
        }

        mod has_content_hash {
            use super::*;

            #[test]
            fn content_hash_returns_stored_hash() {
                let view = make_view();
                let expected = Blake3Hash::from_bytes(b"test content");
                assert_eq!(view.content_hash(), &expected);
            }

            #[test]
            fn is_content_match_returns_true_for_same_hash() {
                let view = make_view();
                let same = Blake3Hash::from_bytes(b"test content");
                assert!(
                    view.is_content_match(&same),
                    "is_content_match should return true for same hash"
                );
            }

            #[test]
            fn is_content_match_returns_false_for_different_hash() {
                let view = make_view();
                let different = Blake3Hash::from_bytes(b"different content");
                assert!(
                    !view.is_content_match(&different),
                    "is_content_match should return false for different hash"
                );
            }
        }

        mod has_content_hash_mut {
            use super::*;

            #[test]
            fn set_content_hash_updates_hash() {
                let mut view = make_view();
                let new_hash = Blake3Hash::from_bytes(b"updated content");
                view.set_content_hash(new_hash);
                assert_eq!(view.content_hash(), &new_hash);
            }

            #[test]
            fn is_content_match_true_after_update() {
                let mut view = make_view();
                let new_hash = Blake3Hash::from_bytes(b"new data");
                view.set_content_hash(new_hash);
                assert!(view.is_content_match(&new_hash));
            }

            #[test]
            fn is_content_match_false_for_old_hash_after_update() {
                let mut view = make_view();
                let original = *view.content_hash();
                let new_hash = Blake3Hash::from_bytes(b"new data");
                view.set_content_hash(new_hash);
                assert!(!view.is_content_match(&original));
            }
        }

        mod serialization {
            use super::*;

            #[test]
            fn rkyv_round_trip() {
                let path = PathKey::try_new("templates/note.md").unwrap();
                let hash = Blake3Hash::from_bytes(b"serialized content");
                let times = FsTimes::new(Some(SystemTime::UNIX_EPOCH), None);
                let metadata = FileMetadata::new(times, 256, false);
                let recorded_at = SystemTime::UNIX_EPOCH;

                let view = RawTemplateView::new(
                    path.clone(),
                    hash,
                    metadata.clone(),
                    recorded_at,
                );

                let bytes = rkyv::to_bytes::<rkyv::rancor::Error>(&view)
                    .expect("Failed to serialize RawTemplateView");
                let deserialized: RawTemplateView = rkyv::from_bytes::<
                    RawTemplateView,
                    rkyv::rancor::Error,
                >(&bytes)
                .expect("Failed to deserialize RawTemplateView");

                assert_eq!(deserialized.path(), &path);
                assert_eq!(deserialized.content_hash(), &hash);
                assert_eq!(deserialized.metadata(), &metadata);
                // recorded_at serialized as unix seconds — check second
                // precision.
                let orig_secs = recorded_at
                    .duration_since(SystemTime::UNIX_EPOCH)
                    .unwrap()
                    .as_secs();
                let deser_secs = deserialized
                    .recorded_at()
                    .duration_since(SystemTime::UNIX_EPOCH)
                    .unwrap()
                    .as_secs();
                assert_eq!(
                    orig_secs, deser_secs,
                    "recorded_at should round-trip as unix seconds"
                );
            }
        }
    }
}
