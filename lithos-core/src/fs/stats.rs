use std::time::{SystemTime, UNIX_EPOCH};

use rkyv::{
    Archive, Deserialize, Serialize,
    with::{AsUnixTime, Map},
};

/// Filesystem statistics for a file.
///
/// Centralises file metadata retrieval (creation, modification, size)
/// to ensure consistent policy across the project.
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, Archive, Serialize, Deserialize,
)]
#[non_exhaustive]
#[expect(
    clippy::module_name_repetitions,
    reason = "FileStats is the standard naming for this type"
)]
#[rkyv(attr(expect(
    clippy::exhaustive_structs,
    reason = "rkyv-generated types are exhaustive"
)))]
pub struct FileStats {
    /// File creation timestamp (birthtime).
    ///
    /// None if the filesystem does not support birthtime.
    #[rkyv(with = Map<AsUnixTime>)]
    pub created_at: Option<SystemTime>,

    /// File modification timestamp (mtime).
    #[rkyv(with = Map<AsUnixTime>)]
    pub modified_at: Option<SystemTime>,

    /// File size in bytes.
    pub size: u64,
}

impl FileStats {
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
