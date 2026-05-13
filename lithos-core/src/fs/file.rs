//! Type-safe wrappers for files and their metadata.
//!
//! Provides the [`FileName`], [`FileInfo`], and [`FileEntry`] types for
//! capturing and processing file information in a way that is compatible with
//! zero-copy storage.
//!
//! ## Usage
//!
//! These types are primarily used by:
//! - [`crate::fs::scanner::DirScanner`]: Returns `Vec<FileEntry>` from
//!   directory scans
//! - [`crate::fs::reader::Reader`]: Uses `FileEntry` in `list_entries()` method
//! - Domain contexts: Store and query file metadata with zero-copy access via
//!   rkyv

#![expect(
    clippy::module_name_repetitions,
    reason = "File* names are intentional and clear in this file-specific \
              module"
)]

use std::time::{SystemTime, UNIX_EPOCH};

use rkyv::{
    Archive, Deserialize, Serialize,
    with::{AsUnixTime, Map},
};

pub use super::name::FileName;

/// Filesystem information for a file.
///
/// Centralises file metadata retrieval (creation, modification, size)
/// to ensure consistent policy across the project. This type is modeled
/// after Obsidian's `FileInfo` API.
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, Archive, Serialize, Deserialize,
)]
#[rkyv(derive(Debug))]
#[non_exhaustive]
pub struct FileInfo {
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

impl FileInfo {
    /// Create new file information.
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

    /// Checks if the provided size matches this information.
    #[inline]
    #[must_use]
    pub fn is_size_match(&self, size: u64) -> bool {
        self.size == size
    }

    /// Checks if the provided timestamps match this information.
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

impl ArchivedFileInfo {
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
            == created_at.and_then(FileInfo::system_time_to_unix_seconds)
            && self
                .modified_at
                .as_ref()
                .and_then(|time| i64::try_from(time.as_secs()).ok())
                == modified_at.and_then(FileInfo::system_time_to_unix_seconds)
    }
}

impl From<std::fs::Metadata> for FileInfo {
    #[inline]
    fn from(meta: std::fs::Metadata) -> Self {
        Self {
            created_at: meta.created().ok(),
            modified_at: meta.modified().ok(),
            size: meta.len(),
        }
    }
}

impl From<super::metadata::FileMetadata> for FileInfo {
    #[inline]
    fn from(meta: super::metadata::FileMetadata) -> Self {
        Self {
            created_at: meta.times().created_at(),
            modified_at: meta.times().modified_at(),
            size: meta.size(),
        }
    }
}

impl From<&super::metadata::FileMetadata> for FileInfo {
    #[inline]
    fn from(meta: &super::metadata::FileMetadata) -> Self {
        Self {
            created_at: meta.times().created_at(),
            modified_at: meta.times().modified_at(),
            size: meta.size(),
        }
    }
}

/// A general-purpose filesystem entry.
///
/// Captures path, metadata, and filename in a single structure, suitable for
/// unified filesystem processing.
#[derive(Debug, Clone)]
#[non_exhaustive]
pub struct FileEntry {
    /// The path to the entry.
    pub path: std::path::PathBuf,
    /// The entry's filename.
    pub filename: FileName,
    /// The entry's information/metadata.
    pub info: FileInfo,
}

#[cfg(test)]
mod tests {
    use std::{
        path::Path,
        time::{SystemTime, UNIX_EPOCH},
    };

    use super::*;

    #[test]
    fn basename_extracts_filename_without_extension() {
        let filename = FileName::new("note.toml".into());
        assert_eq!(filename.basename_str(), "note");
    }

    #[test]
    fn basename_handles_hyphens() {
        let filename = FileName::new("base-note.toml".into());
        assert_eq!(filename.basename_str(), "base-note");
    }

    #[test]
    fn extension_returns_file_extension() {
        let filename = FileName::new("note.toml".into());
        assert_eq!(filename.extension(), Some("toml"));
    }

    #[test]
    fn extension_returns_none_for_no_extension() {
        let filename = FileName::new("note".into());
        assert_eq!(filename.extension(), None);
    }

    #[test]
    fn as_str_returns_full_filename() {
        let filename = FileName::new("note.toml".into());
        assert_eq!(filename.as_str(), "note.toml");
    }

    #[test]
    fn try_from_path_extracts_filename() {
        let path = Path::new("schemas/user.json");
        let filename = FileName::try_from(path).unwrap();
        assert_eq!(filename.as_str(), "user.json");
        assert_eq!(filename.basename_str(), "user");
    }

    #[test]
    fn try_from_path_rejects_empty_filename() {
        let path = Path::new("schemas/..");
        let result: Result<FileName, _> = FileName::try_from(path);
        assert!(result.is_err());
        assert_eq!(
            result.unwrap_err().kind(),
            std::io::ErrorKind::InvalidInput
        );
    }

    mod constructor {
        use super::*;

        #[test]
        fn should_create_new_info_with_provided_values() {
            let now = SystemTime::now();
            let info = FileInfo::new(Some(now), Some(now), 1024);

            assert_eq!(info.created_at(), Some(now), "Created time mismatch");
            assert_eq!(info.modified_at(), Some(now), "Modified time mismatch");
            assert_eq!(info.size(), 1024, "Size mismatch");
        }
    }

    mod validation {
        use super::*;

        #[test]
        fn is_timestamp_match_should_return_true_when_identical() {
            let now = SystemTime::now();
            let info = FileInfo::new(Some(now), Some(now), 1024);

            assert!(
                info.is_timestamp_match(Some(now), Some(now)),
                "Should match identical timestamps"
            );
        }

        #[test]
        fn is_timestamp_match_should_return_false_when_different() {
            let now = SystemTime::now();
            let later = now + std::time::Duration::from_secs(1);
            let info = FileInfo::new(Some(now), Some(now), 1024);

            assert!(
                !info.is_timestamp_match(Some(later), Some(now)),
                "Should not match different created_at"
            );
            assert!(
                !info.is_timestamp_match(Some(now), Some(later)),
                "Should not match different modified_at"
            );
        }

        #[test]
        fn is_size_match_should_return_true_when_identical() {
            let info = FileInfo::new(None, None, 1024);

            assert!(info.is_size_match(1024), "Should match identical size");
        }

        #[test]
        fn is_size_match_should_return_false_when_different() {
            let info = FileInfo::new(None, None, 1024);

            assert!(
                !info.is_size_match(2048),
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
            let info = FileInfo::from(metadata.clone());

            assert_eq!(
                info.size(),
                metadata.len(),
                "Size from metadata mismatch"
            );
            assert_eq!(
                info.modified_at(),
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

            let info = FileInfo::new(Some(rounded), Some(rounded), 1024);
            let bytes = rkyv::to_bytes::<rkyv::rancor::Error>(&info)
                .expect("Failed to serialize");
            let archived =
                rkyv::access::<ArchivedFileInfo, rkyv::rancor::Error>(&bytes)
                    .expect("Failed to access archived info");

            assert!(
                archived.is_timestamp_match(Some(rounded), Some(rounded)),
                "Archived info should match identical timestamps"
            );
        }

        #[test]
        fn archived_should_not_match_different_timestamps() {
            let now = SystemTime::now();
            let later = now + std::time::Duration::from_secs(1);
            let info = FileInfo::new(Some(now), Some(now), 1024);
            let bytes = rkyv::to_bytes::<rkyv::rancor::Error>(&info)
                .expect("Failed to serialize");
            let archived =
                rkyv::access::<ArchivedFileInfo, rkyv::rancor::Error>(&bytes)
                    .expect("Failed to access archived info");

            assert!(
                !archived.is_timestamp_match(Some(later), Some(now)),
                "Archived info should not match different created_at"
            );
        }
    }
}
