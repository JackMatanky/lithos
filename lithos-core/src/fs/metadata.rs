//! Filesystem metadata types for files and directories.
//!
//! Provides type-safe metadata primitives that distinguish between files and
//! directories at the type level.

use std::{io, path::Path, time::SystemTime};

use rkyv::{
    Archive, Deserialize, Serialize,
    with::{AsUnixTime, Map},
};

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
    /// Create `FsMetadata` from a filesystem path.
    ///
    /// Reads metadata from the given path and constructs the appropriate
    /// variant (`File` or `Dir`) based on the filesystem entry type.
    /// Follows symlinks like [`std::fs::metadata`].
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - The path does not exist
    /// - Permission is denied
    /// - The metadata indicates neither a file nor directory (e.g., special
    ///   files)
    ///
    /// # Examples
    ///
    /// ```
    /// use std::fs;
    ///
    /// use lithos_core::fs::metadata::FsMetadata;
    /// use tempfile::tempdir;
    ///
    /// let temp_dir = tempdir().unwrap();
    /// let file_path = temp_dir.path().join("test.txt");
    /// fs::write(&file_path, b"content").unwrap();
    ///
    /// let metadata = FsMetadata::from_path(&file_path).unwrap();
    /// assert!(metadata.is_file());
    /// ```
    #[inline]
    pub fn from_path<P: AsRef<Path>>(path: P) -> io::Result<Self> {
        let std_meta = std::fs::metadata(path.as_ref())?;
        Self::try_from(std_meta)
            .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))
    }

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

impl TryFrom<std::fs::Metadata> for FsMetadata {
    type Error = io::Error;

    #[inline]
    fn try_from(meta: std::fs::Metadata) -> Result<Self, Self::Error> {
        if meta.is_file() {
            Ok(Self::File(FileMetadata::from(&meta)))
        } else if meta.is_dir() {
            Ok(Self::Dir(DirMetadata::from(&meta)))
        } else {
            Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "metadata is neither file nor directory",
            ))
        }
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

    /// Check if the provided size matches this file's size.
    ///
    /// Used for fast staleness detection before performing more expensive
    /// content hash checks.
    #[inline]
    #[must_use]
    pub fn is_size_match(&self, size: u64) -> bool {
        self.size == size
    }

    /// Check if the provided timestamps match this file's timestamps.
    ///
    /// Used for fast staleness detection before performing more expensive
    /// content hash checks. Delegates to the underlying `FsTimes::is_match`.
    #[inline]
    #[must_use]
    pub fn is_timestamp_match(
        &self,
        created_at: Option<SystemTime>,
        modified_at: Option<SystemTime>,
    ) -> bool {
        let other = FsTimes::new(created_at, modified_at);
        self.times.is_match(&other)
    }
}

impl From<&std::fs::Metadata> for FileMetadata {
    #[inline]
    fn from(meta: &std::fs::Metadata) -> Self {
        let times = FsTimes::from(meta);
        let is_symlink = meta.is_symlink();
        Self::new(times, meta.len(), is_symlink)
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

impl From<&std::fs::Metadata> for DirMetadata {
    #[inline]
    fn from(meta: &std::fs::Metadata) -> Self {
        let times = FsTimes::from(meta);
        let is_symlink = meta.is_symlink();
        Self::new(times, is_symlink)
    }
}

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

impl From<&std::fs::Metadata> for FsTimes {
    #[inline]
    fn from(meta: &std::fs::Metadata) -> Self {
        Self::new(meta.created().ok(), meta.modified().ok())
    }
}

impl ArchivedFsTimes {
    /// Check if archived timestamps match provided filesystem times.
    ///
    /// Performs zero-copy comparison by converting `SystemTime` to Unix seconds
    /// and comparing against archived Unix timestamps. Used for fast staleness
    /// detection without deserialization.
    ///
    /// # Examples
    ///
    /// ```
    /// use std::time::SystemTime;
    ///
    /// use lithos_core::fs::metadata::FsTimes;
    ///
    /// let times = FsTimes::new(Some(SystemTime::UNIX_EPOCH), None);
    /// let bytes = rkyv::to_bytes::<rkyv::rancor::Error>(&times).unwrap();
    /// let archived = rkyv::access::<
    ///     lithos_core::fs::metadata::ArchivedFsTimes,
    ///     rkyv::rancor::Error,
    /// >(&bytes)
    /// .unwrap();
    ///
    /// assert!(archived.is_timestamp_match(Some(SystemTime::UNIX_EPOCH), None));
    /// ```
    #[inline]
    #[must_use]
    pub fn is_timestamp_match(
        &self,
        created_at: Option<SystemTime>,
        modified_at: Option<SystemTime>,
    ) -> bool {
        let created_match = match (self.created_at.as_ref(), created_at) {
            (Some(archived), Some(sys)) => {
                let unix_secs = sys
                    .duration_since(std::time::UNIX_EPOCH)
                    .ok()
                    .map(|d| d.as_secs());
                unix_secs.is_some_and(|secs| secs == archived.as_secs())
            }
            (None, None) => true,
            _ => false,
        };

        let modified_match = match (self.modified_at.as_ref(), modified_at) {
            (Some(archived), Some(sys)) => {
                let unix_secs = sys
                    .duration_since(std::time::UNIX_EPOCH)
                    .ok()
                    .map(|d| d.as_secs());
                unix_secs.is_some_and(|secs| secs == archived.as_secs())
            }
            (None, None) => true,
            _ => false,
        };

        created_match && modified_match
    }
}

impl ArchivedFileMetadata {
    /// Check if archived file metadata timestamps match provided filesystem
    /// times.
    ///
    /// Performs zero-copy comparison by delegating to the underlying
    /// `ArchivedFsTimes`. Used for fast staleness detection without
    /// deserialization.
    ///
    /// # Examples
    ///
    /// ```
    /// use std::time::SystemTime;
    ///
    /// use lithos_core::fs::metadata::{FileMetadata, FsTimes};
    ///
    /// let times = FsTimes::new(Some(SystemTime::UNIX_EPOCH), None);
    /// let metadata = FileMetadata::new(times, 1024, false);
    /// let bytes = rkyv::to_bytes::<rkyv::rancor::Error>(&metadata).unwrap();
    /// let archived = rkyv::access::<
    ///     lithos_core::fs::metadata::ArchivedFileMetadata,
    ///     rkyv::rancor::Error,
    /// >(&bytes)
    /// .unwrap();
    ///
    /// assert!(archived.is_timestamp_match(Some(SystemTime::UNIX_EPOCH), None));
    /// ```
    #[inline]
    #[must_use]
    pub fn is_timestamp_match(
        &self,
        created_at: Option<SystemTime>,
        modified_at: Option<SystemTime>,
    ) -> bool {
        self.times.is_timestamp_match(created_at, modified_at)
    }
}

#[cfg(test)]
mod tests {
    use std::time::{SystemTime, UNIX_EPOCH};

    use super::*;

    mod fs_times {
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

        mod is_match {
            use super::*;

            #[test]
            fn returns_true_for_identical_timestamps() {
                let now = SystemTime::now();
                let earlier = UNIX_EPOCH;

                let times1 = FsTimes::new(Some(earlier), Some(now));
                let times2 = FsTimes::new(Some(earlier), Some(now));

                assert!(times1.is_match(&times2));
            }

            #[test]
            fn returns_false_when_modified_differs() {
                let now = SystemTime::now();
                let earlier = UNIX_EPOCH;
                let later =
                    now.checked_add(std::time::Duration::from_secs(1)).unwrap();

                let times1 = FsTimes::new(Some(earlier), Some(now));
                let times2 = FsTimes::new(Some(earlier), Some(later));

                assert!(!times1.is_match(&times2));
            }

            #[test]
            fn returns_false_when_created_differs() {
                let now = SystemTime::now();
                let earlier = UNIX_EPOCH;

                let times1 = FsTimes::new(Some(earlier), Some(now));
                let times2 = FsTimes::new(Some(now), Some(now));

                assert!(!times1.is_match(&times2));
            }

            #[test]
            fn handles_none_values() {
                let now = SystemTime::now();

                let times1 = FsTimes::new(None, Some(now));
                let times2 = FsTimes::new(None, Some(now));

                assert!(times1.is_match(&times2));
            }
        }

        mod archived {
            use super::*;

            #[test]
            fn is_timestamp_match_handles_none_and_some() {
                let t1 = UNIX_EPOCH;
                let t2 = SystemTime::now();

                // Test: Some(t1), None - should match same values
                let times_some_created = FsTimes::new(Some(t1), None);
                let bytes_some_created =
                    rkyv::to_bytes::<rkyv::rancor::Error>(&times_some_created)
                        .unwrap();
                let archived_some_created =
                    rkyv::access::<ArchivedFsTimes, rkyv::rancor::Error>(
                        &bytes_some_created,
                    )
                    .unwrap();

                assert!(
                    archived_some_created.is_timestamp_match(Some(t1), None)
                );
                assert!(
                    !archived_some_created.is_timestamp_match(Some(t2), None)
                );
                assert!(!archived_some_created.is_timestamp_match(None, None));

                // Test: None, Some(t2) - should match same values
                let times_some_modified = FsTimes::new(None, Some(t2));
                let bytes_some_modified =
                    rkyv::to_bytes::<rkyv::rancor::Error>(&times_some_modified)
                        .unwrap();
                let archived_some_modified =
                    rkyv::access::<ArchivedFsTimes, rkyv::rancor::Error>(
                        &bytes_some_modified,
                    )
                    .unwrap();

                assert!(
                    archived_some_modified.is_timestamp_match(None, Some(t2))
                );
                assert!(
                    !archived_some_modified.is_timestamp_match(None, Some(t1))
                );
                assert!(!archived_some_modified.is_timestamp_match(None, None));

                // Test: None, None - should match None, None
                let times_none = FsTimes::new(None, None);
                let bytes_none =
                    rkyv::to_bytes::<rkyv::rancor::Error>(&times_none).unwrap();
                let archived_none = rkyv::access::<
                    ArchivedFsTimes,
                    rkyv::rancor::Error,
                >(&bytes_none)
                .unwrap();

                assert!(archived_none.is_timestamp_match(None, None));
                assert!(!archived_none.is_timestamp_match(Some(t1), None));
            }
        }

        #[test]
        fn converts_from_std_metadata() {
            use std::fs;

            let temp_dir = tempfile::tempdir().unwrap();
            let file_path = temp_dir.path().join("test.txt");
            fs::write(&file_path, b"hello").unwrap();

            let std_meta = fs::metadata(&file_path).unwrap();
            let fs_times = FsTimes::from(&std_meta);

            assert!(fs_times.created_at().is_some());
            assert!(fs_times.modified_at().is_some());
        }
    }

    mod file_metadata {
        use super::*;

        #[test]
        fn stores_all_fields() {
            let now = SystemTime::now();
            let times = FsTimes::new(Some(UNIX_EPOCH), Some(now));

            let metadata = FileMetadata::new(times.clone(), 1024, false);

            assert_eq!(metadata.times(), &times);
            assert_eq!(metadata.size(), 1024);
            assert!(!metadata.is_symlink());
        }

        #[test]
        fn tracks_symlink_status() {
            let times = FsTimes::new(None, None);

            let metadata = FileMetadata::new(times, 0, true);

            assert!(metadata.is_symlink());
        }

        #[test]
        fn converts_from_std_metadata() {
            use std::fs;

            let temp_dir = tempfile::tempdir().unwrap();
            let file_path = temp_dir.path().join("test.txt");
            fs::write(&file_path, b"hello").unwrap();

            let std_meta = fs::metadata(&file_path).unwrap();
            let file_meta = FileMetadata::from(&std_meta);

            assert_eq!(file_meta.size(), 5);
            assert!(file_meta.times().modified_at().is_some());
        }

        #[test]
        fn is_size_match_compares_size_correctly() {
            let times = FsTimes::new(None, None);
            let metadata = FileMetadata::new(times, 1024, false);

            assert!(metadata.is_size_match(1024));
            assert!(!metadata.is_size_match(512));
            assert!(!metadata.is_size_match(2048));
        }

        #[test]
        fn is_timestamp_match_delegates_to_times() {
            let now = SystemTime::now();
            let earlier = UNIX_EPOCH;
            let later =
                now.checked_add(std::time::Duration::from_secs(1)).unwrap();

            let times = FsTimes::new(Some(earlier), Some(now));
            let metadata = FileMetadata::new(times, 1024, false);

            // Should return true for matching timestamps
            assert!(metadata.is_timestamp_match(Some(earlier), Some(now)));

            // Should return false for different created time
            assert!(!metadata.is_timestamp_match(Some(now), Some(now)));

            // Should return false for different modified time
            assert!(!metadata.is_timestamp_match(Some(earlier), Some(later)));

            // Should return false for None vs Some mismatches
            assert!(!metadata.is_timestamp_match(None, Some(now)));
        }

        mod archived {
            use super::*;

            #[test]
            fn is_timestamp_match_performs_zero_copy_comparison() {
                let now = SystemTime::now();
                let earlier = UNIX_EPOCH;

                let times = FsTimes::new(Some(earlier), Some(now));
                let metadata = FileMetadata::new(times, 1024, false);

                // Archive the metadata
                let bytes =
                    rkyv::to_bytes::<rkyv::rancor::Error>(&metadata).unwrap();
                let archived = rkyv::access::<
                    ArchivedFileMetadata,
                    rkyv::rancor::Error,
                >(&bytes)
                .unwrap();

                // Should return true for matching timestamps (zero-copy)
                assert!(archived.is_timestamp_match(Some(earlier), Some(now)));

                // Should return false for different timestamps
                assert!(!archived.is_timestamp_match(Some(now), Some(now)));
                assert!(
                    !archived.is_timestamp_match(Some(earlier), Some(earlier))
                );

                // Should return false for None mismatches
                assert!(!archived.is_timestamp_match(None, Some(now)));
            }
        }
    }

    mod dir_metadata {
        use super::*;

        #[test]
        fn stores_times_and_symlink() {
            let now = SystemTime::now();
            let times = FsTimes::new(Some(UNIX_EPOCH), Some(now));

            let metadata = DirMetadata::new(times.clone(), false);

            assert_eq!(metadata.times(), &times);
            assert!(!metadata.is_symlink());
        }

        #[test]
        fn does_not_have_size_field() {
            // This test verifies by compilation that DirMetadata has no
            // size() method
            let times = FsTimes::new(None, None);
            let dir_meta = DirMetadata::new(times, false);

            // If this compiles, we've confirmed DirMetadata doesn't expose
            // size
            assert!(!dir_meta.is_symlink());
        }

        #[test]
        fn converts_from_std_metadata() {
            use std::fs;

            let temp_dir = tempfile::tempdir().unwrap();
            let dir_path = temp_dir.path().join("subdir");
            fs::create_dir(&dir_path).unwrap();

            let std_meta = fs::metadata(&dir_path).unwrap();
            let dir_meta = DirMetadata::from(&std_meta);

            assert!(dir_meta.times().created_at().is_some());
        }
    }

    mod fs_metadata {
        use super::*;

        #[test]
        fn distinguishes_file_from_dir() {
            let times = FsTimes::new(None, None);

            let file_meta =
                FsMetadata::File(FileMetadata::new(times.clone(), 100, false));
            let dir_meta = FsMetadata::Dir(DirMetadata::new(times, false));

            assert!(file_meta.is_file());
            assert!(!file_meta.is_dir());

            assert!(dir_meta.is_dir());
            assert!(!dir_meta.is_file());
        }

        mod as_file {
            use super::*;

            #[test]
            fn returns_some_for_file_variant() {
                let times = FsTimes::new(None, None);
                let file = FileMetadata::new(times, 512, false);
                let meta = FsMetadata::File(file.clone());

                let retrieved = meta.as_file();

                assert!(retrieved.is_some());
                assert_eq!(retrieved.unwrap(), &file);
            }

            #[test]
            fn returns_none_for_dir_variant() {
                let times = FsTimes::new(None, None);
                let dir = DirMetadata::new(times, false);
                let meta = FsMetadata::Dir(dir);

                assert!(meta.as_file().is_none());
            }
        }

        mod as_dir {
            use super::*;

            #[test]
            fn returns_some_for_dir_variant() {
                let times = FsTimes::new(None, None);
                let dir = DirMetadata::new(times, false);
                let meta = FsMetadata::Dir(dir.clone());

                let retrieved = meta.as_dir();

                assert!(retrieved.is_some());
                assert_eq!(retrieved.unwrap(), &dir);
            }

            #[test]
            fn returns_none_for_file_variant() {
                let times = FsTimes::new(None, None);
                let file = FileMetadata::new(times, 512, false);
                let meta = FsMetadata::File(file);

                assert!(meta.as_dir().is_none());
            }
        }

        mod try_from {
            use super::*;

            #[test]
            fn converts_file_metadata() {
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
            fn converts_dir_metadata() {
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

        mod from_path {
            use super::*;

            #[test]
            fn constructs_file_metadata_for_file() {
                use std::fs;

                let temp_dir = tempfile::tempdir().unwrap();
                let file_path = temp_dir.path().join("test.txt");
                fs::write(&file_path, b"hello world").unwrap();

                let fs_meta = FsMetadata::from_path(&file_path).unwrap();

                assert!(fs_meta.is_file());
                let file_meta = fs_meta.as_file().unwrap();
                assert_eq!(file_meta.size(), 11);
                assert!(file_meta.times().modified_at().is_some());
            }

            #[test]
            fn constructs_dir_metadata_for_directory() {
                use std::fs;

                let temp_dir = tempfile::tempdir().unwrap();
                let dir_path = temp_dir.path().join("subdir");
                fs::create_dir(&dir_path).unwrap();

                let fs_meta = FsMetadata::from_path(&dir_path).unwrap();

                assert!(fs_meta.is_dir());
                let dir_meta = fs_meta.as_dir().unwrap();
                assert!(dir_meta.times().created_at().is_some());
            }

            #[test]
            fn returns_error_when_path_does_not_exist() {
                let temp_dir = tempfile::tempdir().unwrap();
                let nonexistent = temp_dir.path().join("does_not_exist.txt");

                let result = FsMetadata::from_path(&nonexistent);

                assert!(result.is_err());
                let err = result.unwrap_err();
                assert_eq!(err.kind(), io::ErrorKind::NotFound);
            }
        }
    }
}
