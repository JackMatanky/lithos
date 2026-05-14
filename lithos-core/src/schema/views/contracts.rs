//! Trait contracts for schema view persistence.
//!
//! ## Purpose
//!
//! This module defines the trait boundaries that enable **polymorphic access**
//! to view types in both their **owned** (runtime) and **archived** (storage)
//! representations. This design ensures staleness detection, version history,
//! and metadata queries work identically whether operating on deserialized
//! views or zero-copy archived views directly from the database.
//!
//! ## Design Philosophy
//!
//! ### Why Shared Traits Between Owned and Archived Types?
//!
//! Lithos uses `rkyv` for zero-copy deserialization, which generates separate
//! `Archived*` types (e.g., `ArchivedRawSchemaView`) that live in mapped memory
//! and cannot be modified. To avoid code duplication, we define shared trait
//! boundaries:
//!
//! - **Read traits** ([`RawViewRead`], [`VersionRead`]): Implemented by
//!   **both** owned types (e.g., `RawSchemaView`) and archived types (e.g.,
//!   `ArchivedRawSchemaView`). Enable polymorphic read-only access for
//!   staleness checks, metadata queries, and version traversal.
//!
//! - **Mutation traits** ([`RawView`], [`Version`]): Implemented **only** by
//!   owned types. Provide version rotation ([`RawView::add_version`]), metadata
//!   updates, and other mutation operations that require ownership.
//!
//! ### Zero-Copy Access Pattern
//!
//! ```text
//! ┌─────────────────────────────────────────────────────────────┐
//! │  HOT PATH (Read-Only Queries)                               │
//! │  • Staleness checks: view.is_content_match(&hash)           │
//! │  • Metadata queries: view.extends(), view.file_path()       │
//! │  • Version traversal: view.versions()                       │
//! │                                                              │
//! │  Uses: ArchivedRawSchemaView (zero-copy, no allocation)     │
//! │  Trait: RawViewRead (shared read-only interface)            │
//! └─────────────────────────────────────────────────────────────┘
//!
//! ┌─────────────────────────────────────────────────────────────┐
//! │  COLD PATH (Mutations)                                       │
//! │  • Add new version: view.add_version(version)               │
//! │  • Update metadata: view.current_mut().set_expanded(...)    │
//! │  • Ring buffer rotation (evict oldest when full)            │
//! │                                                              │
//! │  Uses: RawSchemaView (owned, heap-allocated)                │
//! │  Trait: RawView (owned mutation interface)                  │
//! └─────────────────────────────────────────────────────────────┘
//! ```
//!
//! ## Trait Hierarchy
//!
//! - [`VersionRead`]: Read-only access to version snapshot metadata (hashes,
//!   timestamps, recorded time). Implemented by [`SchemaVersion`],
//!   [`PropertyBankVersion`], and their `Archived*` counterparts.
//!
//! - [`Version`]: Mutation operations for version snapshots. Extends
//!   [`VersionRead`]. Implemented **only** by owned types.
//!
//! - [`RawViewRead`]: Read-only access to versioned view containers (file path,
//!   version history, inheritance metadata). Implemented by [`RawSchemaView`],
//!   [`RawPropertyBankView`], and their `Archived*` counterparts.
//!
//! - [`RawView`]: Mutation operations for versioned containers (add version,
//!   update current). Extends [`RawViewRead`]. Implemented **only** by owned
//!   types.
//!
//! ## Performance Implications
//!
//! By implementing read traits for both owned and archived types, we enable:
//!
//! 1. **Zero-copy staleness checks**: Compare hashes directly in mapped memory
//!    without deserializing the full view structure.
//!
//! 2. **Lazy deserialization**: Only deserialize (allocate) when mutations are
//!    required (e.g., adding a new version). Read-only operations stay in
//!    mapped memory.
//!
//! 3. **Consistent API**: Code that queries view metadata works identically
//!    whether operating on cached (owned) or database (archived) views.
//!
//! ## Types Referenced
//!
//! - [`FileMetadata`]: File timestamp and size metadata for fast staleness
//!   checks.
//! - [`Blake3Hash`]: Cryptographic content hash for accurate staleness
//!   detection.
//! - [`HashRecord`]: Combined content hash + per-property hashes for
//!   incremental resolution.
//!
//! [`RawSchemaView`]: super::raw::RawSchemaView
//! [`RawPropertyBankView`]: super::raw::RawPropertyBankView
//! [`SchemaVersion`]: super::snapshots::SchemaVersion
//! [`PropertyBankVersion`]: super::snapshots::PropertyBankVersion
//! [`FileMetadata`]: crate::fs::metadata::FileMetadata

use std::time::SystemTime;

use super::HashRecord;
use crate::{
    fs::metadata::{FileMetadata, FsTimes},
    schema::error::SchemaStorageError,
    support::hash::Blake3Hash,
};

/// Defines the mutable container contract for versioned raw-file views.
///
/// Implemented by [`RawSchemaView`] and [`RawPropertyBankView`] to provide
/// consistent version rotation, staleness checks, and metadata refresh helpers.
#[expect(
    dead_code,
    reason = "Trait surface used selectively by view pipelines"
)]
pub(crate) trait RawView: RawViewRead {
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

    /// Updates complete file metadata for the current version, if present.
    #[inline]
    fn update_metadata(&mut self, metadata: FileMetadata) {
        let _version_count: usize = RawViewRead::version_count(self);
        if let Some(current) = self.current_mut() {
            current.set_metadata(metadata);
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
            let old_metadata = Version::metadata(current);
            let size = old_metadata.size();
            let is_symlink = old_metadata.is_symlink();
            let times = FsTimes::new(created_at, modified_at);
            current.set_metadata(FileMetadata::new(times, size, is_symlink));
        }
    }
}

/// Defines the read-only contract for owned and archived raw-file views.
///
/// This keeps staleness checks available on zero-copy archived values without
/// requiring mutable access or allocation.
pub(crate) trait RawViewRead {
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
#[expect(
    dead_code,
    reason = "Trait surface used selectively by snapshot pipelines"
)]
pub(crate) trait Version: VersionRead + Sized {
    /// Returns file metadata for this version.
    fn metadata(&self) -> &FileMetadata;

    /// Returns hash metadata for staleness and incremental resolution.
    fn hashes(&self) -> &HashRecord;

    /// Returns when this version was recorded in storage.
    fn recorded_at(&self) -> SystemTime;

    /// Updates file metadata in-place.
    fn set_metadata(&mut self, metadata: FileMetadata);

    /// Clones this version with replacement metadata.
    ///
    /// Resets cached data (e.g., expanded properties) to maintain
    /// consistency with the new metadata.
    #[must_use]
    fn with_metadata(&self, metadata: FileMetadata, hashes: HashRecord)
    -> Self;
}

/// Defines the read-only contract shared by snapshot payloads.
///
/// Exposes minimal staleness and format information needed by view containers
/// and archived access paths.
#[expect(
    dead_code,
    reason = "Trait surface used selectively by snapshot pipelines"
)]
pub(crate) trait VersionRead {
    /// Returns file metadata for this version.
    ///
    /// # Panics
    /// Default implementation panics - archived types must override
    /// `is_timestamp_match()` directly instead of calling this method.
    #[inline]
    #[expect(
        clippy::panic,
        reason = "Intentional panic for unimplemented default - not reachable \
                  in production"
    )]
    fn metadata(&self) -> &FileMetadata {
        panic!(
            "Archived types must override is_timestamp_match() instead of \
             using metadata()"
        )
    }

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
