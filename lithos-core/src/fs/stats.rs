use std::time::SystemTime;

/// Filesystem statistics for a file.
///
/// Centralises file metadata retrieval (creation, modification, size)
/// to ensure consistent policy across the project.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
#[expect(
    clippy::module_name_repetitions,
    reason = "FileStats is the standard naming for this type"
)]
pub struct FileStats {
    /// File creation timestamp (birthtime).
    ///
    /// None if the filesystem does not support birthtime.
    pub created_at: Option<SystemTime>,

    /// File modification timestamp (mtime).
    pub modified_at: Option<SystemTime>,

    /// File size in bytes.
    pub size: u64,
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
