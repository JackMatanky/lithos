//! Versioned metadata containers for raw schema and property bank files.
//!
//! ## Purpose
//!
//! This module provides the **primary view types** ([`RawSchemaView`],
//! [`RawPropertyBankView`]) that serve as **rkyv-serializable containers** for
//! metadata extracted from **Raw\* types** ([`RawSchema`], [`RawPropertyBank`])
//! which use **serde-only** serialization.
//!
//! **Critical architectural role**: Raw\* types do **not** have `rkyv` derives
//! and **cannot be directly persisted**. View types bridge this gap by
//! extracting file identity, version history, and inheritance metadata into
//! rkyv-serializable containers, enabling staleness detection without
//! re-parsing Raw\* from files.
//!
//! ## Container Responsibilities
//!
//! ### File Identity Tracking
//!
//! Each view maintains a stable file identifier:
//! - [`RawSchemaView`][]: Vault-relative path (e.g., `"schemas/note.toml"`)
//! - [`RawPropertyBankView`][]: Filename (e.g., `"property_bank.yaml"`)
//!
//! These identifiers enable lookup by file path without scanning all views.
//!
//! ### Version History (Ring Buffer)
//!
//! Both view types maintain a **ring buffer** of up to 5 historical versions
//! using [`VecDeque`]:
//!
//! ```text
//! ┌─────────────────────────────────────────────────────────────┐
//! │  RawSchemaView (ring buffer, max 5 versions)                │
//! │                                                             │
//! │  Front (newest)                             Back (oldest)   │
//! │  ┌────────┐ ┌────────┐ ┌────────┐ ┌────────┐ ┌────────┐     │
//! │  │   v5   │ │   v4   │ │   v3   │ │   v2   │ │   v1   │     │
//! │  └────────┘ └────────┘ └────────┘ └────────┘ └────────┘     │
//! │      ▲                                            │         │
//! │      │                                            │         │
//! │  add_version()                              evicted         │
//! │  (push_front)                               (pop_back)      │
//! └─────────────────────────────────────────────────────────────┘
//! ```
//!
//! **Ring buffer properties**:
//! - **Newest first**: `current()` returns `versions.front()` (O(1) access)
//! - **Automatic eviction**: When full (5 versions), `add_version()` evicts
//!   oldest via `pop_back()`
//! - **Efficient rotation**: `VecDeque::push_front()` + `pop_back()` for O(1)
//!   amortized version rotation
//!
//! ### Inheritance Metadata Extraction
//!
//! [`RawSchemaView`] extracts inheritance metadata during ingestion:
//! - **`extends`**: Parent schema name for property inheritance
//! - **`excludes`**: Property names excluded from inheritance
//!
//! This metadata is stored in **every version** ([`SchemaVersion`]) and
//! mirrored at the view level for zero-copy queries:
//!
//! ```rust,ignore
//! // Zero-copy query via ArchivedRawSchemaView (no deserialization)
//! if let Some(parent) = archived_view.extends() {
//!     // Load parent schema before resolving child
//! }
//! ```
//!
//! ## Lifecycle Management
//!
//! ### Creation (New File Ingestion)
//!
//! When a new schema/property bank file is discovered:
//!
//! 1. Parse file to `RawSchema` or `RawPropertyBank` (serde, syntax validation)
//! 2. **Extract metadata from Raw\***: Compute [`HashRecord`], extract
//!    inheritance metadata (`extends`, `excludes`), file stats
//! 3. Create initial version: `SchemaVersion::new(stats, hashes, &raw_schema)`
//!    - Version types have rkyv derives (Raw\* do not)
//! 4. Create view container: `RawSchemaView::new(path, version)`
//! 5. Validate Raw\* → Domain: `Schema::try_from(raw_schema)`
//! 6. Persist **both separately**: view (metadata) + domain (business logic)
//!
//! ### Update (File Modified)
//!
//! When an existing file changes (hash mismatch detected):
//!
//! 1. Re-parse file to `RawSchema` or `RawPropertyBank` (serde, from file)
//! 2. **Extract updated metadata from Raw\***: New hashes, inheritance metadata
//! 3. Create new version: `SchemaVersion::new(stats, hashes, &raw_schema)`
//! 4. Load existing view, add version: `view.add_version(new_version)`
//! 5. Ring buffer automatically evicts oldest if at capacity (5 versions)
//! 6. Re-validate Raw\* → Domain: `Schema::try_from(raw_schema)`
//! 7. Persist **both**: updated view + updated domain aggregate
//!
//! ### Query (Staleness Check)
//!
//! When checking if a file needs re-parsing:
//!
//! 1. Compute current file hash ([`Blake3Hash`])
//! 2. Load view via zero-copy: `ArchivedRawSchemaView` (no allocation)
//! 3. Compare hash: `view.is_content_match(&current_hash)`
//! 4. **Match** → use cached [`Schema`] (skip parsing Raw\*)
//! 5. **Mismatch** → re-parse `RawSchema`, extract metadata, update view
//!
//! **Critical insight**: Views enable staleness checks **without** ever
//! deserializing Raw\* types from the database, because Raw\* types are
//! **never persisted** (serde-only, no rkyv derives).
//!
//! ## Types Defined
//!
//! - [`RawSchemaView`]: Versioned metadata container for schema files. Tracks
//!   path, version history (ring buffer), and inheritance metadata (`extends`,
//!   `excludes`) extracted from each version.
//!
//! - [`RawPropertyBankView`]: Versioned metadata container for property bank
//!   files. Tracks filename and version history (ring buffer) with per-property
//!   hashes for incremental resolution.
//!
//! ## Related Types
//!
//! - [`FileStats`]: File timestamp and size metadata for fast staleness checks.
//! - [`SchemaVersion`]: Version snapshot payload for schemas (includes
//!   inheritance metadata, bank references, hashes).
//! - [`PropertyBankVersion`]: Version snapshot payload for property banks
//!   (includes hashes, version string).
//! - [`HashRecord`]: Combined content hash + per-property hashes for
//!   incremental resolution.
//!
//! [`Schema`]: crate::schema::aggregate::Schema
//! [`PropertyBank`]: crate::schema::bank::PropertyBank

#![expect(
    clippy::same_name_method,
    reason = "Trait contracts intentionally mirror existing view API names"
)]

use std::{collections::VecDeque, time::SystemTime};

use rkyv::{Archive, Deserialize, Serialize};

use super::{
    HashRecord, PropertyBankVersion, RawView, RawViewRead, SchemaVersion,
    Version as _, VersionRead as _,
};
use crate::{
    fs::{FileStats, RelativePath},
    schema::{
        error::{SchemaIngestionError, SchemaStorageError},
        identifier::SchemaName,
        property::PropertyName,
        raw::{RawPropertyBank, RawSchema},
    },
    support::hash::Blake3Hash,
};

/// Represents a raw schema file with version history.
///
/// Tracks up to 5 versions of a schema file. Each version includes inheritance
/// metadata (`extends`, `excludes`) to enable incremental resolution.
#[derive(Debug, Clone, PartialEq, Archive, Serialize, Deserialize)]
pub struct RawSchemaView {
    /// Vault-relative schema path (e.g., "schemas/note.toml").
    path: RelativePath,

    /// Version history (ring buffer, max 5 versions, newest first).
    ///
    /// Using `VecDeque` allows efficient `push_front`/`pop_back` for version
    /// rotation. Each version contains extends/excludes metadata.
    versions: VecDeque<SchemaVersion>,
}

impl RawSchemaView {
    /// Creates a new schema view with an initial version.
    ///
    /// # Examples
    ///
    /// ```
    /// # use lithos_core::schema::views::{RawSchemaView, SchemaVersion};
    /// # use lithos_core::fs::RelativePath;
    /// #
    /// # // Mock setup
    /// # let path = RelativePath::try_from("schemas/note.json").unwrap();
    /// # // let version = ...
    /// ```
    #[inline]
    #[must_use]
    pub fn new(path: RelativePath, version: SchemaVersion) -> Self {
        let mut versions =
            VecDeque::with_capacity(<Self as RawView>::MAX_VERSIONS);
        versions.push_front(version);

        Self {
            path,
            versions,
        }
    }

    /// Returns the vault-relative schema path.
    ///
    /// # Examples
    ///
    /// ```
    /// # use lithos_core::schema::views::RawSchemaView;
    /// # use lithos_core::fs::RelativePath;
    /// #
    /// # // let view = ...
    /// # // assert_eq!(view.file_path().as_str(), "schemas/note.json");
    /// ```
    #[inline]
    #[must_use]
    pub fn file_path(&self) -> &RelativePath {
        &self.path
    }

    /// Returns the schema name derived from the filename without extension.
    ///
    /// # Examples
    ///
    /// ```
    /// # use lithos_core::schema::views::RawSchemaView;
    /// #
    /// # // let view = ...
    /// # // assert_eq!(view.name(), "note");
    /// ```
    #[inline]
    #[must_use]
    pub fn name(&self) -> &str {
        self.path
            .as_path()
            .file_stem()
            .and_then(|stem| stem.to_str())
            .unwrap_or("")
    }

    /// Returns the parent schema name from the current version, if any.
    ///
    /// Extracted from the `extends` field during schema ingestion.
    ///
    /// # Examples
    ///
    /// ```
    /// # use lithos_core::schema::views::RawSchemaView;
    /// #
    /// # // let view = ...
    /// # // let parent = view.extends();
    /// ```
    #[inline]
    #[must_use]
    pub fn extends(&self) -> Option<&SchemaName> {
        let v = self.current()?;
        v.extends()
    }

    /// Returns excluded property names from the current version.
    ///
    /// Extracted from the `excludes` field during schema ingestion.
    ///
    /// # Examples
    ///
    /// ```
    /// # use lithos_core::schema::views::RawSchemaView;
    /// #
    /// # // let view = ...
    /// # // let excluded = view.excludes();
    /// ```
    #[inline]
    #[must_use]
    pub fn excludes(&self) -> &[PropertyName] {
        self.current().map_or(&[], super::SchemaVersion::excludes)
    }

    /// Returns the most recent version, if any.
    ///
    /// # Examples
    ///
    /// ```
    /// # use lithos_core::schema::views::RawSchemaView;
    /// #
    /// # // let view = ...
    /// # // if let Some(current) = view.current() { ... }
    /// ```
    #[inline]
    #[must_use]
    pub fn current(&self) -> Option<&SchemaVersion> {
        self.versions.front()
    }

    /// Returns mutable access to the most recent version, if any.
    ///
    /// Used for updating cached expanded properties after [`RefExpander`] runs.
    ///
    /// # Examples
    ///
    /// ```
    /// # use lithos_core::schema::views::RawSchemaView;
    /// #
    /// # // let mut view = ...
    /// # // if let Some(current) = view.current_mut() { ... }
    /// ```
    #[inline]
    #[must_use]
    pub fn current_mut(&mut self) -> Option<&mut SchemaVersion> {
        self.versions.front_mut()
    }

    /// Returns all tracked versions (newest first).
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use lithos_core::schema::views::RawSchemaView;
    /// #
    /// # let view: RawSchemaView = todo!();
    /// for version in view.versions() {
    ///     // ...
    /// }
    /// ```
    #[inline]
    #[must_use]
    pub fn versions(&self) -> &VecDeque<SchemaVersion> {
        &self.versions
    }

    /// Returns the number of tracked versions.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use lithos_core::schema::views::RawSchemaView;
    /// #
    /// # let view: RawSchemaView = todo!();
    /// assert!(view.version_count() > 0);
    /// ```
    #[inline]
    #[must_use]
    pub fn version_count(&self) -> usize {
        self.versions.len()
    }

    /// Adds a new version, evicting the oldest if at capacity.
    ///
    /// # Examples
    ///
    /// ```
    /// # use lithos_core::schema::views::{RawSchemaView, SchemaVersion};
    /// #
    /// # // let mut view = ...
    /// # // let new_version = ...
    /// # // view.add_version(new_version);
    /// ```
    #[inline]
    pub fn add_version(&mut self, version: SchemaVersion) {
        if self.versions.len() >= <Self as RawView>::MAX_VERSIONS {
            self.versions.pop_back(); // Remove oldest
        }

        self.versions.push_front(version);
    }

    /// Creates a view from a parsed schema and its file content.
    ///
    /// Computes the content hash from `content` and extracts metadata
    /// from `raw` to build the initial [`SchemaVersion`].
    ///
    /// # Parameters
    ///
    /// - `raw`: The parsed [`RawSchema`].
    /// - `path`: The vault-relative path (e.g., `"schemas/note.json"`).
    /// - `content`: The raw file content (used to compute the content hash).
    ///
    /// # Errors
    ///
    /// Returns [`SchemaIngestionError`] if metadata extraction fails.
    #[inline]
    pub fn try_from_with_content(
        raw: &RawSchema,
        path: RelativePath,
        content: &str,
    ) -> Result<Self, SchemaIngestionError> {
        use super::HashRecord;

        // Compute content hash from raw file content
        let content_hash = Blake3Hash::compute(content.as_bytes());

        // Compute per-property hashes
        let property_hashes = raw.properties().compute_hashes();

        let file_stats = *raw.file_stats();
        let hashes = HashRecord::new(content_hash, property_hashes);

        let version = SchemaVersion::new(file_stats, hashes, raw)?;

        Ok(Self::new(path, version))
    }
}

/// Implements [`RawView`] for [`RawSchemaView`].
impl RawView for RawSchemaView {
    type FilePath = RelativePath;
    type Version = SchemaVersion;

    #[inline]
    fn file_path(&self) -> &Self::FilePath {
        &self.path
    }

    #[inline]
    fn current(&self) -> Option<&Self::Version> {
        self.versions.front()
    }

    #[inline]
    fn current_mut(&mut self) -> Option<&mut Self::Version> {
        self.versions.front_mut()
    }

    #[inline]
    fn version_count(&self) -> usize {
        self.versions.len()
    }

    #[inline]
    fn add_version(&mut self, version: Self::Version) {
        RawSchemaView::add_version(self, version);
    }

    #[inline]
    fn is_content_match(&self, content_hash: &Blake3Hash) -> bool {
        self.current().is_some_and(|v| v.is_content_match(content_hash))
    }

    #[inline]
    fn is_timestamp_match(
        &self,
        created_at: Option<SystemTime>,
        modified_at: Option<SystemTime>,
    ) -> bool {
        self.current()
            .is_some_and(|v| v.is_timestamp_match(created_at, modified_at))
    }

    #[inline]
    fn update_file_stats(&mut self, file_stats: FileStats) {
        if let Some(current) = self.current_mut() {
            current.set_file_stats(file_stats);
        }
    }

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

    /// Adds a new version with updated content hash.
    ///
    /// Keeps property hashes from the current version.
    #[inline]
    fn update_content_hash(
        &mut self,
        content_hash: Blake3Hash,
    ) -> Result<(), SchemaStorageError> {
        let current = self.current().ok_or(SchemaStorageError::NotFound {
            name: "current schema version".into(),
        })?;
        let file_stats = *current.file_stats();
        let hashes = HashRecord::new(
            content_hash,
            current.hashes().properties().clone(),
        );
        let version = SchemaVersion::with_metadata(current, file_stats, hashes);
        self.add_version(version);
        Ok(())
    }
}

/// Implements [`RawViewRead`] for [`RawSchemaView`].
impl RawViewRead for RawSchemaView {
    #[inline]
    fn is_content_match(&self, content_hash: &Blake3Hash) -> bool {
        RawView::is_content_match(self, content_hash)
    }

    #[inline]
    fn is_timestamp_match(
        &self,
        created_at: Option<SystemTime>,
        modified_at: Option<SystemTime>,
    ) -> bool {
        RawView::is_timestamp_match(self, created_at, modified_at)
    }

    #[inline]
    fn version_count(&self) -> usize {
        self.version_count()
    }
}

/// Implements [`RawViewRead`] for [`ArchivedRawSchemaView`] (zero-copy).
impl RawViewRead for ArchivedRawSchemaView {
    #[inline]
    fn is_content_match(&self, content_hash: &Blake3Hash) -> bool {
        self.versions
            .as_slice()
            .first()
            .is_some_and(|version| version.is_content_match(content_hash))
    }

    #[inline]
    fn is_timestamp_match(
        &self,
        created_at: Option<SystemTime>,
        modified_at: Option<SystemTime>,
    ) -> bool {
        self.versions.as_slice().first().is_some_and(|version| {
            version.is_timestamp_match(created_at, modified_at)
        })
    }

    #[inline]
    fn version_count(&self) -> usize {
        self.versions.len()
    }
}

/// Represents a raw property bank file with version history.
///
/// Tracks up to 5 versions of the property bank file for staleness detection.
#[derive(Debug, Clone, PartialEq, Archive, Serialize, Deserialize)]
pub struct RawPropertyBankView {
    /// Relative path to the property bank file (e.g.,
    /// "`schemas/property_bank.json`").
    path: RelativePath,

    /// Version history (ring buffer, max 5 versions, newest first).
    versions: VecDeque<PropertyBankVersion>,
}

impl RawPropertyBankView {
    /// Creates a new property bank view with an initial version.
    ///
    /// # Examples
    ///
    /// ```
    /// # use lithos_core::schema::views::{RawPropertyBankView, PropertyBankVersion};
    /// # use lithos_core::fs::Filename;
    /// #
    /// # // let filename = Filename::new("properties.yaml".into());
    /// # // let version = ...
    /// # // let view = RawPropertyBankView::new(path, version);
    /// ```
    #[inline]
    #[must_use]
    pub fn new(path: RelativePath, version: PropertyBankVersion) -> Self {
        let mut versions =
            VecDeque::with_capacity(<Self as RawView>::MAX_VERSIONS);
        versions.push_front(version);

        Self {
            path,
            versions,
        }
    }

    /// Returns the property bank relative path.
    ///
    /// # Examples
    ///
    /// ```
    /// # use lithos_core::schema::views::RawPropertyBankView;
    /// #
    /// # // let view = ...
    /// # // assert_eq!(view.file_path().as_str(), "schemas/property_bank.json");
    /// ```
    #[inline]
    #[must_use]
    pub fn file_path(&self) -> &RelativePath {
        &self.path
    }

    /// Returns the most recent version, if any.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use lithos_core::schema::views::RawPropertyBankView;
    /// #
    /// # let view: RawPropertyBankView = todo!();
    /// if let Some(current) = view.current() {
    ///     // ...
    /// }
    /// ```
    #[inline]
    #[must_use]
    pub fn current(&self) -> Option<&PropertyBankVersion> {
        self.versions.front()
    }

    /// Returns mutable access to the most recent version, if any.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use lithos_core::schema::views::RawPropertyBankView;
    /// #
    /// # let mut view: RawPropertyBankView = todo!();
    /// if let Some(current) = view.current_mut() {
    ///     // ...
    /// }
    /// ```
    #[inline]
    #[must_use]
    pub fn current_mut(&mut self) -> Option<&mut PropertyBankVersion> {
        self.versions.front_mut()
    }

    /// Returns all tracked versions (newest first).
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use lithos_core::schema::views::RawPropertyBankView;
    /// #
    /// # let view: RawPropertyBankView = todo!();
    /// for version in view.versions() {
    ///     // ...
    /// }
    /// ```
    #[inline]
    #[must_use]
    pub fn versions(&self) -> &VecDeque<PropertyBankVersion> {
        &self.versions
    }

    /// Returns the number of tracked versions.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use lithos_core::schema::views::RawPropertyBankView;
    /// #
    /// # let view: RawPropertyBankView = todo!();
    /// assert!(view.version_count() > 0);
    /// ```
    #[inline]
    #[must_use]
    pub fn version_count(&self) -> usize {
        self.versions.len()
    }

    /// Adds a new version, evicting the oldest if at capacity.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use lithos_core::schema::views::{RawPropertyBankView, PropertyBankVersion};
    /// #
    /// # let mut view: RawPropertyBankView = todo!();
    /// # let new_version: PropertyBankVersion = todo!();
    /// view.add_version(new_version);
    /// ```
    #[inline]
    pub fn add_version(&mut self, version: PropertyBankVersion) {
        if self.versions.len() >= <Self as RawView>::MAX_VERSIONS {
            self.versions.pop_back(); // Remove oldest
        }

        self.versions.push_front(version);
    }

    /// Updates file timestamps of the current version.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use lithos_core::schema::views::RawPropertyBankView;
    /// # use std::time::SystemTime;
    /// #
    /// # let mut view: RawPropertyBankView = todo!();
    /// view.update_timestamps(Some(SystemTime::now()), None);
    /// ```
    #[inline]
    pub fn update_timestamps(
        &mut self,
        created_at: Option<SystemTime>,
        modified_at: Option<SystemTime>,
    ) {
        if let Some(current) = self.versions.front_mut() {
            let size = current.file_stats().size();
            current.set_file_stats(FileStats::new(
                created_at,
                modified_at,
                size,
            ));
        }
    }

    /// Updates full file stats of the current version.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use lithos_core::schema::views::RawPropertyBankView;
    /// # use lithos_core::fs::FileStats;
    /// #
    /// # let mut view: RawPropertyBankView = todo!();
    /// # let stats = FileStats::new(None, None, 1024);
    /// view.update_file_stats(stats);
    /// ```
    #[inline]
    pub fn update_file_stats(&mut self, file_stats: FileStats) {
        if let Some(current) = self.versions.front_mut() {
            current.set_file_stats(file_stats);
        }
    }

    /// Updates the content hash while preserving property hashes.
    ///
    /// Adds a new version with the updated content hash and existing file
    /// times.
    ///
    /// # Errors
    ///
    /// Returns [`SchemaStorageError`] if the current version is unavailable.
    #[inline]
    pub fn update_content_hash(
        &mut self,
        content_hash: Blake3Hash,
    ) -> Result<(), SchemaStorageError> {
        let current =
            self.current().ok_or(SchemaStorageError::PropertyBankNotFound)?;
        let file_stats = *current.file_stats();

        let hashes = HashRecord::new(
            content_hash,
            current.hashes().properties().clone(),
        );
        let version =
            PropertyBankVersion::new(file_stats, hashes, current.version())
                .map_err(|_error| SchemaStorageError::Corruption {
                    reason: "failed to rebuild property bank version".into(),
                })?;
        self.add_version(version);
        Ok(())
    }

    /// Creates a view from a raw property bank with hashes.
    ///
    /// Use this when you have computed hashes and want to persist a new
    /// versioned view for staleness checks.
    ///
    /// # Errors
    ///
    /// Returns [`SchemaIngestionError`] if metadata is missing or validation
    /// fails.
    #[inline]
    pub fn try_from_raw_with_hashes(
        raw: &RawPropertyBank,
        path: RelativePath,
        raw_hash: HashRecord,
    ) -> Result<Self, SchemaIngestionError> {
        let file_stats = *raw.file_stats();

        let version = PropertyBankVersion::new(
            file_stats,
            raw_hash,
            raw.version().as_str(),
        )?;

        Ok(Self::new(path, version))
    }
}

/// Implements [`RawView`] for [`RawPropertyBankView`].
impl RawView for RawPropertyBankView {
    type FilePath = RelativePath;
    type Version = PropertyBankVersion;

    #[inline]
    fn file_path(&self) -> &Self::FilePath {
        &self.path
    }

    #[inline]
    fn current(&self) -> Option<&Self::Version> {
        self.versions.front()
    }

    #[inline]
    fn current_mut(&mut self) -> Option<&mut Self::Version> {
        self.versions.front_mut()
    }

    #[inline]
    fn version_count(&self) -> usize {
        self.versions.len()
    }

    #[inline]
    fn add_version(&mut self, version: Self::Version) {
        RawPropertyBankView::add_version(self, version);
    }

    /// Checks content hash against the current version.
    #[inline]
    fn is_content_match(&self, content_hash: &Blake3Hash) -> bool {
        self.current().is_some_and(|v| v.is_content_match(content_hash))
    }

    /// Checks filesystem timestamps against the current version.
    #[inline]
    fn is_timestamp_match(
        &self,
        created_at: Option<SystemTime>,
        modified_at: Option<SystemTime>,
    ) -> bool {
        self.current()
            .is_some_and(|v| v.is_timestamp_match(created_at, modified_at))
    }

    /// Updates file stats on the current version, if present.
    #[inline]
    fn update_file_stats(&mut self, file_stats: FileStats) {
        if let Some(current) = self.current_mut() {
            current.set_file_stats(file_stats);
        }
    }

    /// Updates timestamps on the current version, preserving file size.
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

    /// Adds a new version with updated content hash.
    ///
    /// Preserves property hashes from the current version.
    /// Delegates to [`RawPropertyBankView::update_content_hash`].
    #[inline]
    fn update_content_hash(
        &mut self,
        content_hash: Blake3Hash,
    ) -> Result<(), SchemaStorageError> {
        RawPropertyBankView::update_content_hash(self, content_hash)
    }
}

impl RawViewRead for RawPropertyBankView {
    #[inline]
    fn is_content_match(&self, content_hash: &Blake3Hash) -> bool {
        RawView::is_content_match(self, content_hash)
    }

    #[inline]
    fn is_timestamp_match(
        &self,
        created_at: Option<SystemTime>,
        modified_at: Option<SystemTime>,
    ) -> bool {
        RawView::is_timestamp_match(self, created_at, modified_at)
    }

    #[inline]
    fn version_count(&self) -> usize {
        self.version_count()
    }
}

impl RawViewRead for ArchivedRawPropertyBankView {
    #[inline]
    fn is_content_match(&self, content_hash: &Blake3Hash) -> bool {
        self.versions
            .as_slice()
            .first()
            .is_some_and(|version| version.is_content_match(content_hash))
    }

    #[inline]
    fn is_timestamp_match(
        &self,
        created_at: Option<SystemTime>,
        modified_at: Option<SystemTime>,
    ) -> bool {
        self.versions.as_slice().first().is_some_and(|version| {
            version.is_timestamp_match(created_at, modified_at)
        })
    }

    #[inline]
    fn version_count(&self) -> usize {
        self.versions.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        fs::FileStats,
        schema::{
            property::{PropertyMap, PropertyName},
            raw::{RawPropertyBank, RawSchema},
            views::RawPropertyMapHash,
        },
    };

    fn make_schema_version(content_hash: Blake3Hash) -> SchemaVersion {
        let raw = serde_json::from_value::<RawSchema>(serde_json::json!({
            "$version": "1.0",
            "properties": {}
        }))
        .expect("valid schema fixture")
        .with_name("note".into());

        SchemaVersion::new(
            FileStats::new(None, None, 0),
            HashRecord::new(content_hash, RawPropertyMapHash::default()),
            &raw,
        )
        .expect("schema version should build")
    }

    fn make_property_bank_version(
        content_hash: Blake3Hash,
    ) -> PropertyBankVersion {
        let mut property_hashes = RawPropertyMapHash::default();
        property_hashes.insert(
            PropertyName::try_new("title").expect("valid property name"),
            Blake3Hash::new([9; 32]),
        );

        PropertyBankVersion::new(
            FileStats::new(None, None, 0),
            HashRecord::new(content_hash, property_hashes),
            "1.0",
        )
        .expect("property bank version should build")
    }

    #[test]
    fn property_bank_view_rotation_uses_trait_max_versions() {
        let path =
            RelativePath::try_from("schemas/property_bank.json").unwrap();
        let mut view = RawPropertyBankView::new(
            path,
            make_property_bank_version(Blake3Hash::new([1; 32])),
        );

        for i in 2..=7 {
            view.add_version(make_property_bank_version(Blake3Hash::new(
                [i; 32],
            )));
        }

        assert_eq!(
            view.version_count(),
            <RawPropertyBankView as RawView>::MAX_VERSIONS
        );
        assert_eq!(
            view.current().map(|v| v.hashes().content()),
            Some(&Blake3Hash::new([7; 32]))
        );
    }

    #[test]
    fn property_bank_update_content_hash_preserves_property_hashes() {
        let path =
            RelativePath::try_from("schemas/property_bank.json").unwrap();
        let mut view = RawPropertyBankView::new(
            path,
            make_property_bank_version(Blake3Hash::new([1; 32])),
        );

        let expected_properties = view
            .current()
            .map(|v| v.hashes().properties().clone())
            .expect("current version should exist");

        view.update_content_hash(Blake3Hash::new([2; 32]))
            .expect("content hash update should succeed");

        let current = view.current().expect("new version should exist");
        assert_eq!(current.hashes().content(), &Blake3Hash::new([2; 32]));
        assert_eq!(current.hashes().properties(), &expected_properties);
    }

    #[test]
    fn schema_view_update_content_hash_clears_expanded_properties_cache() {
        let schema_path = RelativePath::try_from("schemas/note.json").unwrap();
        let mut view = RawSchemaView::new(
            schema_path,
            make_schema_version(Blake3Hash::new([1; 32])),
        );

        let expanded = PropertyMap::new();
        view.current_mut()
            .expect("current version should exist")
            .set_expanded_properties(expanded);

        <RawSchemaView as RawView>::update_content_hash(
            &mut view,
            Blake3Hash::new([3; 32]),
        )
        .expect("content hash update should succeed");

        let current = view.current().expect("new version should exist");
        assert_eq!(current.hashes().content(), &Blake3Hash::new([3; 32]));
        assert!(current.expanded_properties().is_none());
    }

    #[test]
    fn raw_schema_view_update_timestamps_preserves_existing_size() {
        let schema_path =
            crate::fs::RelativePath::try_from("schemas/note.json").unwrap();
        let mut version = make_schema_version(Blake3Hash::new([1; 32]));
        let created_at = Some(std::time::UNIX_EPOCH);
        let modified_at =
            Some(std::time::UNIX_EPOCH + std::time::Duration::from_secs(5));
        super::super::Version::set_file_stats(
            &mut version,
            FileStats::new(created_at, modified_at, 128),
        );

        let mut view = RawSchemaView::new(schema_path, version);

        let next_created_at =
            Some(std::time::UNIX_EPOCH + std::time::Duration::from_secs(30));
        let next_modified_at =
            Some(std::time::UNIX_EPOCH + std::time::Duration::from_secs(45));
        view.update_timestamps(next_created_at, next_modified_at);

        let current = view.current().expect("current version should exist");
        assert_eq!(current.file_stats().created_at(), next_created_at);
        assert_eq!(current.file_stats().modified_at(), next_modified_at);
        assert_eq!(current.file_stats().size(), 128);
    }

    #[test]
    fn raw_schema_view_update_file_stats_replaces_full_metadata() {
        let schema_path =
            crate::fs::RelativePath::try_from("schemas/note.json").unwrap();
        let mut view = RawSchemaView::new(
            schema_path,
            make_schema_version(Blake3Hash::new([1; 32])),
        );

        let replacement = FileStats::new(
            Some(std::time::UNIX_EPOCH + std::time::Duration::from_secs(60)),
            Some(std::time::UNIX_EPOCH + std::time::Duration::from_secs(75)),
            4096,
        );
        view.update_file_stats(replacement);

        let current = view.current().expect("current version should exist");
        assert_eq!(current.file_stats(), &replacement);
    }

    #[test]
    fn property_bank_update_content_hash_errors_without_current_version() {
        let path =
            RelativePath::try_from("schemas/property_bank.json").unwrap();
        let mut view = RawPropertyBankView::new(
            path,
            make_property_bank_version(Blake3Hash::new([1; 32])),
        );

        view.versions.clear();

        let err = view
            .update_content_hash(Blake3Hash::new([4; 32]))
            .expect_err("missing version should error");

        assert!(matches!(err, SchemaStorageError::PropertyBankNotFound));
    }

    #[test]
    fn raw_property_bank_view_try_from_raw_with_hashes_builds_view() {
        let raw: RawPropertyBank = serde_json::from_value(serde_json::json!({
            "$version": "1.0",
            "properties": {}
        }))
        .expect("valid property bank fixture");

        let view = RawPropertyBankView::try_from_raw_with_hashes(
            &raw,
            RelativePath::try_from("property_bank.json").unwrap(),
            HashRecord::new(
                Blake3Hash::new([1; 32]),
                RawPropertyMapHash::default(),
            ),
        )
        .expect("view creation should succeed");

        assert_eq!(view.version_count(), 1);
    }

    #[test]
    fn archived_raw_property_bank_view_supports_zero_copy_staleness_checks() {
        let path =
            RelativePath::try_from("schemas/property_bank.json").unwrap();
        let view = RawPropertyBankView::new(
            path,
            make_property_bank_version(Blake3Hash::new([9; 32])),
        );

        let bytes = rkyv::to_bytes::<rkyv::rancor::Error>(&view)
            .expect("serialize test view");
        let archived = rkyv::access::<
            rkyv::Archived<RawPropertyBankView>,
            rkyv::rancor::Error,
        >(&bytes)
        .expect("access archived view");

        assert!(archived.is_content_match(&Blake3Hash::new([9; 32])));
        assert_eq!(archived.version_count(), 1);
    }

    #[test]
    fn archived_raw_schema_view_supports_zero_copy_staleness_checks() {
        let schema_path =
            crate::fs::RelativePath::try_from("schemas/note.json").unwrap();
        let view = RawSchemaView::new(
            schema_path,
            make_schema_version(Blake3Hash::new([5; 32])),
        );

        let bytes = rkyv::to_bytes::<rkyv::rancor::Error>(&view)
            .expect("serialize test view");
        let archived = rkyv::access::<
            rkyv::Archived<RawSchemaView>,
            rkyv::rancor::Error,
        >(&bytes)
        .expect("access archived view");

        assert!(archived.is_content_match(&Blake3Hash::new([5; 32])));
        assert_eq!(archived.version_count(), 1);
    }
}
