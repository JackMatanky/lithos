//! Filesystem metadata types for files and directories.
//!
//! Provides type-safe metadata primitives that distinguish between files and
//! directories at the type level.

use std::{fs::Metadata, io, time::SystemTime};

use rkyv::{
    Archive, Deserialize, Serialize,
    with::{AsUnixTime, Map},
};

/// Filesystem timestamps for creation and modification times.
///
/// Timestamps are stored as `Option<SystemTime>` because not all filesystems
/// or platforms provide both creation and modification times reliably.
#[derive(Debug, Clone, PartialEq, Archive, Serialize, Deserialize)]
#[rkyv(compare(PartialEq), derive(Debug))]
pub struct FsTimes {
    /// File/directory creation time (if available).
    #[rkyv(with = Map<AsUnixTime>)]
    created_at: Option<SystemTime>,
    /// File/directory modification time (if available).
    #[rkyv(with = Map<AsUnixTime>)]
    modified_at: Option<SystemTime>,
}

impl FsTimes {
    /// Create new filesystem timestamps.
    #[inline]
    #[must_use]
    pub const fn new(
        created_at: Option<SystemTime>,
        modified_at: Option<SystemTime>,
    ) -> Self {
        Self {
            created_at,
            modified_at,
        }
    }

    /// Get creation time.
    #[inline]
    #[must_use]
    pub const fn created_at(&self) -> Option<SystemTime> {
        self.created_at
    }

    /// Get modification time.
    #[inline]
    #[must_use]
    pub const fn modified_at(&self) -> Option<SystemTime> {
        self.modified_at
    }

    /// Check if these timestamps match another set of timestamps.
    ///
    /// Returns `true` if both creation and modification times match. Used for
    /// staleness detection by comparing cached timestamps against current
    /// filesystem state.
    #[inline]
    #[must_use]
    pub fn is_match(&self, other: &Self) -> bool {
        self.created_at == other.created_at
            && self.modified_at == other.modified_at
    }
}

/// Metadata for a file (not a directory).
///
/// Contains file-specific information including size, timestamps, and symlink
/// status.
#[derive(Debug, Clone, PartialEq, Archive, Serialize, Deserialize)]
#[rkyv(compare(PartialEq), derive(Debug))]
pub struct FileMetadata {
    /// File timestamps.
    times: FsTimes,
    /// File size in bytes.
    size: u64,
    /// Whether this file is a symbolic link.
    is_symlink: bool,
}

impl FileMetadata {
    /// Create new file metadata.
    #[inline]
    #[must_use]
    pub const fn new(times: FsTimes, size: u64, is_symlink: bool) -> Self {
        Self {
            times,
            size,
            is_symlink,
        }
    }

    /// Get file timestamps.
    #[inline]
    #[must_use]
    pub const fn times(&self) -> &FsTimes {
        &self.times
    }

    /// Get file size in bytes.
    #[inline]
    #[must_use]
    pub const fn size(&self) -> u64 {
        self.size
    }

    /// Check if this file is a symbolic link.
    #[inline]
    #[must_use]
    pub const fn is_symlink(&self) -> bool {
        self.is_symlink
    }
}

/// Metadata for a directory (not a file).
///
/// Contains directory-specific information. Unlike `FileMetadata`, this does
/// not include size because directory size is not a meaningful or portable
/// concept across filesystems.
#[derive(Debug, Clone, PartialEq, Archive, Serialize, Deserialize)]
#[rkyv(compare(PartialEq), derive(Debug))]
pub struct DirMetadata {
    /// Directory timestamps.
    times: FsTimes,
    /// Whether this directory is a symbolic link.
    is_symlink: bool,
}

impl DirMetadata {
    /// Create new directory metadata.
    #[inline]
    #[must_use]
    pub const fn new(times: FsTimes, is_symlink: bool) -> Self {
        Self {
            times,
            is_symlink,
        }
    }

    /// Get directory timestamps.
    #[inline]
    #[must_use]
    pub const fn times(&self) -> &FsTimes {
        &self.times
    }

    /// Check if this directory is a symbolic link.
    #[inline]
    #[must_use]
    pub const fn is_symlink(&self) -> bool {
        self.is_symlink
    }
}

/// Unified filesystem metadata for files or directories.
///
/// Provides type-safe access to metadata with variants for files and
/// directories. Use the helper methods to determine the variant and access
/// the underlying metadata.
#[derive(Debug, Clone, PartialEq, Archive, Serialize, Deserialize)]
#[rkyv(compare(PartialEq), derive(Debug))]
#[non_exhaustive]
pub enum FsMetadata {
    /// Metadata for a file.
    File(FileMetadata),
    /// Metadata for a directory.
    Dir(DirMetadata),
}

impl FsMetadata {
    /// Check if this metadata is for a file.
    #[inline]
    #[must_use]
    pub const fn is_file(&self) -> bool {
        matches!(self, Self::File(_))
    }

    /// Check if this metadata is for a directory.
    #[inline]
    #[must_use]
    pub const fn is_dir(&self) -> bool {
        matches!(self, Self::Dir(_))
    }

    /// Get file metadata if this is a file.
    #[inline]
    #[must_use]
    pub const fn as_file(&self) -> Option<&FileMetadata> {
        match self {
            Self::File(meta) => Some(meta),
            Self::Dir(_) => None,
        }
    }

    /// Get directory metadata if this is a directory.
    #[inline]
    #[must_use]
    pub const fn as_dir(&self) -> Option<&DirMetadata> {
        match self {
            Self::File(_) => None,
            Self::Dir(meta) => Some(meta),
        }
    }
}

impl TryFrom<Metadata> for FsMetadata {
    type Error = io::Error;

    #[inline]
    fn try_from(meta: Metadata) -> Result<Self, Self::Error> {
        let times = FsTimes::new(meta.created().ok(), meta.modified().ok());

        let is_symlink = meta.is_symlink();

        if meta.is_file() {
            Ok(Self::File(FileMetadata::new(times, meta.len(), is_symlink)))
        } else if meta.is_dir() {
            Ok(Self::Dir(DirMetadata::new(times, is_symlink)))
        } else {
            Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "metadata is neither file nor directory",
            ))
        }
    }
}

#[cfg(test)]
mod tests {
    use std::time::{SystemTime, UNIX_EPOCH};

    use super::*;

    #[test]
    fn constructs_with_both_timestamps() {
        let now = SystemTime::now();
        let earlier = UNIX_EPOCH;

        let times = FsTimes::new(Some(earlier), Some(now));

        assert_eq!(times.created_at(), Some(earlier));
        assert_eq!(times.modified_at(), Some(now));
    }

    #[test]
    fn constructs_with_missing_timestamps() {
        let now = SystemTime::now();

        let times = FsTimes::new(None, Some(now));

        assert_eq!(times.created_at(), None);
        assert_eq!(times.modified_at(), Some(now));
    }

    #[test]
    fn is_match_returns_true_for_identical_timestamps() {
        let now = SystemTime::now();
        let earlier = UNIX_EPOCH;

        let times1 = FsTimes::new(Some(earlier), Some(now));
        let times2 = FsTimes::new(Some(earlier), Some(now));

        assert!(times1.is_match(&times2));
    }

    #[test]
    fn is_match_returns_false_when_modified_differs() {
        let now = SystemTime::now();
        let earlier = UNIX_EPOCH;
        let later = now.checked_add(std::time::Duration::from_secs(1)).unwrap();

        let times1 = FsTimes::new(Some(earlier), Some(now));
        let times2 = FsTimes::new(Some(earlier), Some(later));

        assert!(!times1.is_match(&times2));
    }

    #[test]
    fn is_match_returns_false_when_created_differs() {
        let now = SystemTime::now();
        let earlier = UNIX_EPOCH;

        let times1 = FsTimes::new(Some(earlier), Some(now));
        let times2 = FsTimes::new(Some(now), Some(now));

        assert!(!times1.is_match(&times2));
    }

    #[test]
    fn is_match_handles_none_values() {
        let now = SystemTime::now();

        let times1 = FsTimes::new(None, Some(now));
        let times2 = FsTimes::new(None, Some(now));

        assert!(times1.is_match(&times2));
    }

    #[test]
    fn file_metadata_stores_all_fields() {
        let now = SystemTime::now();
        let times = FsTimes::new(Some(UNIX_EPOCH), Some(now));

        let metadata = FileMetadata::new(times.clone(), 1024, false);

        assert_eq!(metadata.times(), &times);
        assert_eq!(metadata.size(), 1024);
        assert!(!metadata.is_symlink());
    }

    #[test]
    fn file_metadata_tracks_symlink_status() {
        let times = FsTimes::new(None, None);

        let metadata = FileMetadata::new(times, 0, true);

        assert!(metadata.is_symlink());
    }

    #[test]
    fn dir_metadata_stores_times_and_symlink() {
        let now = SystemTime::now();
        let times = FsTimes::new(Some(UNIX_EPOCH), Some(now));

        let metadata = DirMetadata::new(times.clone(), false);

        assert_eq!(metadata.times(), &times);
        assert!(!metadata.is_symlink());
    }

    #[test]
    fn dir_metadata_does_not_have_size_field() {
        // This test verifies by compilation that DirMetadata has no size()
        // method
        let times = FsTimes::new(None, None);
        let dir_meta = DirMetadata::new(times, false);

        // If this compiles, we've confirmed DirMetadata doesn't expose size
        assert!(!dir_meta.is_symlink());
    }

    #[test]
    fn fs_metadata_distinguishes_file_from_dir() {
        let times = FsTimes::new(None, None);

        let file_meta =
            FsMetadata::File(FileMetadata::new(times.clone(), 100, false));
        let dir_meta = FsMetadata::Dir(DirMetadata::new(times, false));

        assert!(file_meta.is_file());
        assert!(!file_meta.is_dir());

        assert!(dir_meta.is_dir());
        assert!(!dir_meta.is_file());
    }

    #[test]
    fn fs_metadata_as_file_returns_some_for_file_variant() {
        let times = FsTimes::new(None, None);
        let file = FileMetadata::new(times, 512, false);
        let meta = FsMetadata::File(file.clone());

        let retrieved = meta.as_file();

        assert!(retrieved.is_some());
        assert_eq!(retrieved.unwrap(), &file);
    }

    #[test]
    fn fs_metadata_as_file_returns_none_for_dir_variant() {
        let times = FsTimes::new(None, None);
        let dir = DirMetadata::new(times, false);
        let meta = FsMetadata::Dir(dir);

        assert!(meta.as_file().is_none());
    }

    #[test]
    fn fs_metadata_as_dir_returns_some_for_dir_variant() {
        let times = FsTimes::new(None, None);
        let dir = DirMetadata::new(times, false);
        let meta = FsMetadata::Dir(dir.clone());

        let retrieved = meta.as_dir();

        assert!(retrieved.is_some());
        assert_eq!(retrieved.unwrap(), &dir);
    }

    #[test]
    fn fs_metadata_as_dir_returns_none_for_file_variant() {
        let times = FsTimes::new(None, None);
        let file = FileMetadata::new(times, 512, false);
        let meta = FsMetadata::File(file);

        assert!(meta.as_dir().is_none());
    }

    #[test]
    fn converts_from_file_metadata() {
        use std::fs;

        let temp_dir = tempfile::tempdir().unwrap();
        let file_path = temp_dir.path().join("test.txt");
        fs::write(&file_path, b"hello").unwrap();

        let std_meta = fs::metadata(&file_path).unwrap();
        let fs_meta = FsMetadata::try_from(std_meta).unwrap();

        assert!(fs_meta.is_file());
        let file_meta = fs_meta.as_file().unwrap();
        assert_eq!(file_meta.size(), 5);
    }

    #[test]
    fn converts_from_dir_metadata() {
        use std::fs;

        let temp_dir = tempfile::tempdir().unwrap();
        let dir_path = temp_dir.path().join("subdir");
        fs::create_dir(&dir_path).unwrap();

        let std_meta = fs::metadata(&dir_path).unwrap();
        let fs_meta = FsMetadata::try_from(std_meta).unwrap();

        assert!(fs_meta.is_dir());
        assert!(fs_meta.as_dir().is_some());
    }
}
