//! Shared contracts for schema persistence views.

use std::time::SystemTime;

use super::HashRecord;
use crate::{
    fs::FileStats, schema::error::SchemaStorageError, support::hash::Blake3Hash,
};

/// Shared behavior for raw file views with version history.
#[expect(
    clippy::arbitrary_source_item_ordering,
    reason = "Trait items are grouped by lifecycle semantics"
)]
pub trait RawView {
    /// Concrete version payload type.
    type Version: Version;
    /// Concrete path/filename identifier type.
    type FilePath;

    /// Maximum number of historical versions retained.
    const MAX_VERSIONS: usize = 5;

    /// Returns the file identifier (path or filename).
    fn file_path(&self) -> &Self::FilePath;

    /// Returns the most recent version, if any.
    fn current(&self) -> Option<&Self::Version>;

    /// Returns mutable access to the most recent version, if any.
    fn current_mut(&mut self) -> Option<&mut Self::Version>;

    /// Returns the number of tracked versions.
    fn version_count(&self) -> usize;

    /// Adds a new version, evicting the oldest if needed.
    fn add_version(&mut self, version: Self::Version);

    /// Returns true when filesystem timestamps match current version metadata.
    #[inline]
    fn is_timestamp_match(
        &self,
        created_at: Option<SystemTime>,
        modified_at: Option<SystemTime>,
    ) -> bool {
        self.current()
            .is_some_and(|v| v.is_timestamp_match(created_at, modified_at))
    }

    /// Returns true when content hash matches current version metadata.
    #[inline]
    fn is_content_match(&self, content_hash: &Blake3Hash) -> bool {
        self.current().is_some_and(|v| v.is_content_match(content_hash))
    }

    /// Updates timestamps for the current version, if present.
    ///
    /// This preserves the current file size and refreshes recorded metadata.
    #[inline]
    fn update_timestamps(
        &mut self,
        created_at: Option<SystemTime>,
        modified_at: Option<SystemTime>,
    ) {
        if let Some(current) = self.current_mut() {
            let size = current.file_stats().size();
            current.set_file_stats(FileStats::new(
                created_at,
                modified_at,
                size,
            ));
        }
    }

    /// Updates complete file stats for the current version, if present.
    #[inline]
    fn update_file_stats(&mut self, file_stats: FileStats) {
        if let Some(current) = self.current_mut() {
            current.set_file_stats(file_stats);
        }
    }

    /// Adds a new version with updated content hash.
    ///
    /// # Errors
    ///
    /// Returns [`SchemaStorageError`] when the current version is unavailable
    /// or when replacement metadata cannot be constructed.
    fn update_content_hash(
        &mut self,
        content_hash: Blake3Hash,
    ) -> Result<(), SchemaStorageError>;
}

/// Zero-copy read contract for archived and owned raw views.
pub trait RawViewRead {
    /// Returns true when content hash matches current version metadata.
    fn is_content_match(&self, content_hash: &Blake3Hash) -> bool;

    /// Returns true when filesystem timestamps match current version metadata.
    fn is_timestamp_match(
        &self,
        created_at: Option<SystemTime>,
        modified_at: Option<SystemTime>,
    ) -> bool;

    /// Returns number of retained historical versions.
    fn version_count(&self) -> usize;
}
/// Mutable version contract for persisted raw views.
pub trait Version: VersionRead + Sized {
    /// Returns file statistics metadata.
    fn file_stats(&self) -> &FileStats;

    /// Returns hash metadata.
    fn hashes(&self) -> &HashRecord;

    /// Returns when this version was recorded.
    fn recorded_at(&self) -> SystemTime;

    /// Updates file statistics metadata in-place.
    fn set_file_stats(&mut self, file_stats: FileStats);

    /// Clones this version with replacement metadata.
    #[must_use]
    fn with_metadata(&self, file_stats: FileStats, hashes: HashRecord) -> Self;
}

/// Read-only access contract shared by version payloads.
pub trait VersionRead {
    /// Checks whether the content hash matches this version.
    fn is_content_match(&self, hash: &Blake3Hash) -> bool;

    /// Checks whether filesystem timestamps match this version's metadata.
    fn is_timestamp_match(
        &self,
        created_at: Option<SystemTime>,
        modified_at: Option<SystemTime>,
    ) -> bool;

    /// Returns the format version string.
    fn version(&self) -> &str;
}
