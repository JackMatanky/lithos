//! Contracts for schema persistence view types.
//!
//! This module defines trait boundaries used by the schema view layer:
//! - [`VersionRead`] and [`Version`] for snapshot payloads,
//! - [`RawViewRead`] and [`RawView`] for versioned raw-file view containers.
//!
//! The contracts are shared by owned and archived (`rkyv`) representations so
//! staleness checks and version-history behavior stay consistent across storage
//! and runtime access paths.

use std::time::SystemTime;

use super::HashRecord;
use crate::{
    fs::FileStats, schema::error::SchemaStorageError, support::hash::Blake3Hash,
};

/// Mutable container contract for versioned raw-file views.
///
/// Implemented by `RawSchemaView` and `RawPropertyBankView` to provide
/// consistent version rotation, staleness checks, and metadata refresh helpers.
pub trait RawView {
    /// Maximum number of historical versions retained.
    const MAX_VERSIONS: usize = 5;

    /// Concrete path/filename identifier type.
    type FilePath;
    /// Concrete version payload type.
    type Version: Version;

    /// Adds a new version, evicting the oldest if needed.
    fn add_version(&mut self, version: Self::Version);

    /// Returns the most recent version, if any.
    fn current(&self) -> Option<&Self::Version>;

    /// Returns mutable access to the most recent version, if any.
    fn current_mut(&mut self) -> Option<&mut Self::Version>;

    /// Returns the file identifier (path or filename).
    fn file_path(&self) -> &Self::FilePath;

    /// Returns true when content hash matches current version metadata.
    #[inline]
    fn is_content_match(&self, content_hash: &Blake3Hash) -> bool {
        self.current().is_some_and(|v| v.is_content_match(content_hash))
    }

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

    /// Updates complete file stats for the current version, if present.
    #[inline]
    fn update_file_stats(&mut self, file_stats: FileStats) {
        if let Some(current) = self.current_mut() {
            current.set_file_stats(file_stats);
        }
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

    /// Returns the number of tracked versions.
    fn version_count(&self) -> usize;
}

/// Read-only contract for owned and archived raw-file views.
///
/// This keeps staleness checks available on zero-copy archived values without
/// requiring mutable access or allocation.
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
/// Mutable contract for persisted snapshot payloads.
///
/// Implemented by `SchemaVersion` and `PropertyBankVersion`.
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

/// Read-only contract shared by snapshot payloads.
///
/// Exposes minimal staleness and format information needed by view containers
/// and archived access paths.
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
