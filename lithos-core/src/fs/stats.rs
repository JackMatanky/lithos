//! Filesystem statistics for vault-scoped files.
//!
//! Provides the [`FileStats`] type for capturing and comparing file metadata
//! (timestamps and size) in a way that is compatible with zero-copy storage.

use std::time::{SystemTime, UNIX_EPOCH};

use rkyv::{
    Archive, Deserialize, Serialize,
    with::{AsUnixTime, Map},
};

/// Filesystem statistics for a file.
///
/// Centralises file metadata retrieval (creation, modification, size)
/// to ensure consistent policy across the project. This type is modeled
/// after Obsidian's `FileStats` API.
///
/// # Field Policy
///
/// - `created_at`: The file's birthtime. Optional as not all filesystems
///   support it.
/// - `modified_at`: The file's last modification time.
/// - `size`: The file's size in bytes.
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, Archive, Serialize, Deserialize,
)]
#[non_exhaustive]
#[expect(
    clippy::module_name_repetitions,
    reason = "FileStats is the standard naming for this type"
)]
pub struct FileStats {
    /// File creation timestamp (birthtime).
    ///
    /// None if the filesystem does not support birthtime.
    #[rkyv(with = Map<AsUnixTime>)]
    created_at: Option<SystemTime>,

    /// File modification timestamp (mtime).
    #[rkyv(with = Map<AsUnixTime>)]
    modified_at: Option<SystemTime>,

    /// File size in bytes.
    size: u64,
}

impl FileStats {
    /// Create new file statistics.
    #[inline]
    #[must_use]
    pub const fn new(
        created_at: Option<SystemTime>,
        modified_at: Option<SystemTime>,
        size: u64,
    ) -> Self {
        Self {
            created_at,
            modified_at,
            size,
        }
    }

    /// Get file creation timestamp.
    #[inline]
    #[must_use]
    pub const fn created_at(&self) -> Option<SystemTime> {
        self.created_at
    }

    /// Get file modification timestamp.
    #[inline]
    #[must_use]
    pub const fn modified_at(&self) -> Option<SystemTime> {
        self.modified_at
    }

    /// Get file size in bytes.
    #[inline]
    #[must_use]
    pub const fn size(&self) -> u64 {
        self.size
    }

    /// Checks if the provided size matches these statistics.
    #[inline]
    #[must_use]
    pub fn is_size_match(&self, size: u64) -> bool {
        self.size == size
    }

    /// Checks if the provided timestamps match these statistics.
    ///
    /// Used for fast staleness detection before performing more expensive
    /// content hash checks.
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

impl ArchivedFileStats {
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
            == created_at.and_then(FileStats::system_time_to_unix_seconds)
            && self
                .modified_at
                .as_ref()
                .and_then(|time| i64::try_from(time.as_secs()).ok())
                == modified_at.and_then(FileStats::system_time_to_unix_seconds)
    }
}

impl From<std::fs::Metadata> for FileStats {
    #[inline]
    fn from(meta: std::fs::Metadata) -> Self {
        Self {
            created_at: meta.created().ok(),
            modified_at: meta.modified().ok(),
            size: meta.len(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    mod constructor {
        use super::*;

        #[test]
        fn should_create_new_stats_with_provided_values() {
            let now = SystemTime::now();
            let stats = FileStats::new(Some(now), Some(now), 1024);

            assert_eq!(stats.created_at(), Some(now), "Created time mismatch");
            assert_eq!(
                stats.modified_at(),
                Some(now),
                "Modified time mismatch"
            );
            assert_eq!(stats.size(), 1024, "Size mismatch");
        }
    }

    mod validation {
        use super::*;

        #[test]
        fn is_timestamp_match_should_return_true_when_identical() {
            let now = SystemTime::now();
            let stats = FileStats::new(Some(now), Some(now), 1024);

            assert!(
                stats.is_timestamp_match(Some(now), Some(now)),
                "Should match identical timestamps"
            );
        }

        #[test]
        fn is_timestamp_match_should_return_false_when_different() {
            let now = SystemTime::now();
            let later = now + std::time::Duration::from_secs(1);
            let stats = FileStats::new(Some(now), Some(now), 1024);

            assert!(
                !stats.is_timestamp_match(Some(later), Some(now)),
                "Should not match different created_at"
            );
            assert!(
                !stats.is_timestamp_match(Some(now), Some(later)),
                "Should not match different modified_at"
            );
        }

        #[test]
        fn is_size_match_should_return_true_when_identical() {
            let stats = FileStats::new(None, None, 1024);

            assert!(stats.is_size_match(1024), "Should match identical size");
        }

        #[test]
        fn is_size_match_should_return_false_when_different() {
            let stats = FileStats::new(None, None, 1024);

            assert!(
                !stats.is_size_match(2048),
                "Should not match different size"
            );
        }
    }

    mod conversions {
        use tempfile::NamedTempFile;

        use super::*;

        #[test]
        fn should_create_from_metadata() {
            let file =
                NamedTempFile::new().expect("Failed to create temp file");
            let metadata =
                file.as_file().metadata().expect("Failed to get metadata");
            let stats = FileStats::from(metadata.clone());

            assert_eq!(
                stats.size(),
                metadata.len(),
                "Size from metadata mismatch"
            );
            assert_eq!(
                stats.modified_at(),
                metadata.modified().ok(),
                "Modified time from metadata mismatch"
            );
        }
    }

    mod borrowing {
        use super::*;

        #[test]
        fn archived_should_match_identical_timestamps() {
            let now = SystemTime::now();
            // Round to seconds to match AsUnixTime precision
            let secs = now.duration_since(UNIX_EPOCH).unwrap().as_secs();
            let rounded = UNIX_EPOCH + std::time::Duration::from_secs(secs);

            let stats = FileStats::new(Some(rounded), Some(rounded), 1024);
            let bytes = rkyv::to_bytes::<rkyv::rancor::Error>(&stats)
                .expect("Failed to serialize");
            let archived =
                rkyv::access::<ArchivedFileStats, rkyv::rancor::Error>(&bytes)
                    .expect("Failed to access archived stats");

            assert!(
                archived.is_timestamp_match(Some(rounded), Some(rounded)),
                "Archived stats should match identical timestamps"
            );
        }

        #[test]
        fn archived_should_not_match_different_timestamps() {
            let now = SystemTime::now();
            let later = now + std::time::Duration::from_secs(1);
            let stats = FileStats::new(Some(now), Some(now), 1024);
            let bytes = rkyv::to_bytes::<rkyv::rancor::Error>(&stats)
                .expect("Failed to serialize");
            let archived =
                rkyv::access::<ArchivedFileStats, rkyv::rancor::Error>(&bytes)
                    .expect("Failed to access archived stats");

            assert!(
                !archived.is_timestamp_match(Some(later), Some(now)),
                "Archived stats should not match different created_at"
            );
        }
    }
}
