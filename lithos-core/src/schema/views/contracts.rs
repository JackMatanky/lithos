//! Trait contracts for schema view persistence.
//!
//! This module defines trait boundaries used by the schema view layer:
//! - [`VersionRead`] and [`Version`] for snapshot payloads.
//! - [`RawViewRead`] and [`RawView`] for versioned raw-file view containers.
//!
//! These contracts are shared by owned and archived (`rkyv`) representations so
//! staleness checks and version-history behavior stay consistent across storage
//! and runtime access paths.
//!
//! Types referenced by these traits:
//! - [`FileStats`] — File timestamp and size metadata.
//! - [`Blake3Hash`] — Content hash for staleness detection.

use std::time::SystemTime;

use super::HashRecord;
use crate::{
    fs::FileStats, schema::error::SchemaStorageError, support::hash::Blake3Hash,
};

/// Defines the mutable container contract for versioned raw-file views.
///
/// Implemented by [`RawSchemaView`] and [`RawPropertyBankView`] to provide
/// consistent version rotation, staleness checks, and metadata refresh helpers.
pub trait RawView {
    /// Represents the maximum number of historical versions retained.
    const MAX_VERSIONS: usize = 5;

    /// Specifies the concrete path or filename identifier type.
    type FilePath;

    /// Specifies the concrete version payload type.
    type Version: Version;

    /// Adds a new version, evicting the oldest if at capacity.
    ///
    /// When the version history reaches [`Self::MAX_VERSIONS`], the oldest
    /// version is removed to make room for the new one.
    fn add_version(&mut self, version: Self::Version);

    /// Returns the most recent version, if any.
    fn current(&self) -> Option<&Self::Version>;

    /// Returns mutable access to the most recent version, if any.
    fn current_mut(&mut self) -> Option<&mut Self::Version>;

    /// Returns the file identifier (path or filename).
    fn file_path(&self) -> &Self::FilePath;

    /// Returns `true` if the content hash matches the current version metadata.
    #[inline]
    fn is_content_match(&self, content_hash: &Blake3Hash) -> bool {
        self.current().is_some_and(|v| v.is_content_match(content_hash))
    }

    /// Returns `true` if filesystem timestamps match the current version
    /// metadata.
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

/// Defines the read-only contract for owned and archived raw-file views.
///
/// This keeps staleness checks available on zero-copy archived values without
/// requiring mutable access or allocation.
pub trait RawViewRead {
    /// Returns `true` if the content hash matches the current version metadata.
    fn is_content_match(&self, content_hash: &Blake3Hash) -> bool;

    /// Returns `true` if filesystem timestamps match the current version
    /// metadata.
    fn is_timestamp_match(
        &self,
        created_at: Option<SystemTime>,
        modified_at: Option<SystemTime>,
    ) -> bool;

    /// Returns the number of retained historical versions.
    fn version_count(&self) -> usize;
}

/// Defines the mutable contract for persisted snapshot payloads.
///
/// Implemented by [`SchemaVersion`] and [`PropertyBankVersion`].
pub trait Version: VersionRead + Sized {
    /// Returns file statistics metadata for this version.
    fn file_stats(&self) -> &FileStats;

    /// Returns hash metadata for staleness and incremental resolution.
    fn hashes(&self) -> &HashRecord;

    /// Returns when this version was recorded in storage.
    fn recorded_at(&self) -> SystemTime;

    /// Updates file statistics metadata in-place.
    fn set_file_stats(&mut self, file_stats: FileStats);

    /// Clones this version with replacement metadata.
    ///
    /// Resets cached data (e.g., expanded properties) to maintain
    /// consistency with the new metadata.
    #[must_use]
    fn with_metadata(&self, file_stats: FileStats, hashes: HashRecord) -> Self;
}

/// Defines the read-only contract shared by snapshot payloads.
///
/// Exposes minimal staleness and format information needed by view containers
/// and archived access paths.
pub trait VersionRead {
    /// Returns `true` if the content hash matches this version.
    fn is_content_match(&self, hash: &Blake3Hash) -> bool;

    /// Returns `true` if filesystem timestamps match this version's metadata.
    fn is_timestamp_match(
        &self,
        created_at: Option<SystemTime>,
        modified_at: Option<SystemTime>,
    ) -> bool;

    /// Returns the format version string (e.g., `"1.0"`).
    fn version(&self) -> &str;
}
