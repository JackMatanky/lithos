//! Raw schema and property bank views for persistence.
//!
//! These types track version history for schemas and property banks,
//! enabling staleness detection and incremental updates.
//!
//! Types defined in this module:
//! - [`RawSchemaView`] — Versioned view of a schema file
//! - [`RawPropertyBankView`] — Versioned view of a property bank file
//!
//! See [`FileStats`], [`SchemaVersion`], [`PropertyBankVersion`] for
//! related metadata types.

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
    fs::{FileStats, Filename, RelativePath},
    schema::{
        aggregate::SchemaName,
        error::{SchemaIngestionError, SchemaStorageError},
        property::PropertyName,
        raw::{RawPropertyBank, RawSchema},
    },
    support::hash::Blake3Hash,
};

/// Raw schema file with version history.
///
/// Tracks up to 5 versions of a schema file. Each version includes inheritance
/// metadata (`extends`, `excludes`) to enable incremental resolution.
///
/// Raw schema file with version history.
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
    /// Creates a new schema view with initial version.
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
    #[inline]
    #[must_use]
    pub fn file_path(&self) -> &RelativePath {
        &self.path
    }

    /// Returns the schema name derived from filename without extension.
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
    #[inline]
    #[must_use]
    pub fn extends(&self) -> Option<&SchemaName> {
        let v = self.current()?;
        v.extends()
    }

    /// Returns excluded property names from the current version.
    ///
    /// Extracted from the `excludes` field during schema ingestion.
    #[inline]
    #[must_use]
    pub fn excludes(&self) -> &[PropertyName] {
        self.current().map_or(&[], super::SchemaVersion::excludes)
    }

    /// Returns the most recent version, if any.
    #[inline]
    #[must_use]
    pub fn current(&self) -> Option<&SchemaVersion> {
        self.versions.front()
    }

    /// Returns mutable access to the most recent version, if any.
    ///
    /// Used for updating cached expanded properties after `RefExpander` runs.
    #[inline]
    #[must_use]
    pub fn current_mut(&mut self) -> Option<&mut SchemaVersion> {
        self.versions.front_mut()
    }

    /// Returns all tracked versions (newest first).
    #[inline]
    #[must_use]
    pub fn versions(&self) -> &VecDeque<SchemaVersion> {
        &self.versions
    }

    /// Returns the number of tracked versions.
    #[inline]
    #[must_use]
    pub fn version_count(&self) -> usize {
        self.versions.len()
    }

    /// Adds a new version (evicts oldest if at capacity).
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
    /// - `raw`: The parsed [`RawSchema`]
    /// - `path`: The vault-relative path (e.g., `"schemas/note.json"`)
    /// - `content`: The raw file content (used to compute content hash)
    ///
    /// # Errors
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

/// Raw property bank file with version history.
///
/// Tracks up to 5 versions of the property bank file for staleness detection.
#[derive(Debug, Clone, PartialEq, Archive, Serialize, Deserialize)]
pub struct RawPropertyBankView {
    /// Filename with extension (e.g., "properties.yaml").
    filename: Filename,

    /// Version history (ring buffer, max 5 versions, newest first).
    versions: VecDeque<PropertyBankVersion>,
}

impl RawPropertyBankView {
    /// Creates a new property bank view with initial version.
    #[inline]
    #[must_use]
    pub fn new(filename: Filename, version: PropertyBankVersion) -> Self {
        let mut versions =
            VecDeque::with_capacity(<Self as RawView>::MAX_VERSIONS);
        versions.push_front(version);

        Self {
            filename,
            versions,
        }
    }

    /// Returns the property bank filename.
    #[inline]
    #[must_use]
    pub fn file_path(&self) -> &Filename {
        &self.filename
    }

    /// Returns the most recent version, if any.
    #[inline]
    #[must_use]
    pub fn current(&self) -> Option<&PropertyBankVersion> {
        self.versions.front()
    }

    /// Returns mutable access to the most recent version, if any.
    #[inline]
    #[must_use]
    pub fn current_mut(&mut self) -> Option<&mut PropertyBankVersion> {
        self.versions.front_mut()
    }

    /// Returns all tracked versions (newest first).
    #[inline]
    #[must_use]
    pub fn versions(&self) -> &VecDeque<PropertyBankVersion> {
        &self.versions
    }

    /// Returns the number of tracked versions.
    #[inline]
    #[must_use]
    pub fn version_count(&self) -> usize {
        self.versions.len()
    }

    /// Adds a new version (evicts oldest if at capacity).
    #[inline]
    pub fn add_version(&mut self, version: PropertyBankVersion) {
        if self.versions.len() >= <Self as RawView>::MAX_VERSIONS {
            self.versions.pop_back(); // Remove oldest
        }

        self.versions.push_front(version);
    }

    /// Update file timestamps of the current version.
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

    /// Update full file stats of the current version.
    #[inline]
    pub fn update_file_stats(&mut self, file_stats: FileStats) {
        if let Some(current) = self.versions.front_mut() {
            current.set_file_stats(file_stats);
        }
    }

    /// Update content hash while preserving property hashes.
    ///
    /// Adds a new version with updated content hash and existing file times.
    ///
    /// # Errors
    /// This method is currently infallible; the `Result` is retained for
    /// pipeline compatibility if future validation is added.
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
    /// Returns error if metadata is missing or validation fails.
    #[inline]
    pub fn try_from_raw_with_hashes(
        raw: &RawPropertyBank,
        filename: Filename,
        raw_hash: HashRecord,
    ) -> Result<Self, SchemaIngestionError> {
        let file_stats = *raw.file_stats();

        let version = PropertyBankVersion::new(
            file_stats,
            raw_hash,
            raw.version().as_str(),
        )?;

        Ok(Self::new(filename, version))
    }
}

/// Implements [`RawView`] for [`RawPropertyBankView`].
impl RawView for RawPropertyBankView {
    type FilePath = Filename;
    type Version = PropertyBankVersion;

    #[inline]
    fn file_path(&self) -> &Self::FilePath {
        &self.filename
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
    use std::collections::HashMap;

    use super::*;
    use crate::{
        fs::{FileStats, Filename},
        schema::{
            property::{PropertyMap, PropertyName},
            raw::{RawPropertyBank, RawSchema},
        },
    };

    fn make_schema_version(content_hash: Blake3Hash) -> SchemaVersion {
        let raw: RawSchema =
            serde_json::from_value::<RawSchema>(serde_json::json!({
                "$version": "1.0",
                "properties": {}
            }))
            .expect("valid schema fixture")
            .with_name("note".into());

        SchemaVersion::new(
            FileStats::new(None, None, 0),
            HashRecord::new(content_hash, HashMap::new()),
            &raw,
        )
        .expect("schema version should build")
    }

    fn make_property_bank_version(
        content_hash: Blake3Hash,
    ) -> PropertyBankVersion {
        let mut property_hashes = HashMap::new();
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
        let filename = Filename::new("property_bank.json".into());
        let mut view = RawPropertyBankView::new(
            filename,
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
        let filename = Filename::new("property_bank.json".into());
        let mut view = RawPropertyBankView::new(
            filename,
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
        let filename = Filename::new("property_bank.json".into());
        let mut view = RawPropertyBankView::new(
            filename,
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
            Filename::new("property_bank.json".into()),
            HashRecord::new(Blake3Hash::new([1; 32]), HashMap::new()),
        )
        .expect("view creation should succeed");

        assert_eq!(view.version_count(), 1);
    }

    #[test]
    fn archived_raw_property_bank_view_supports_zero_copy_staleness_checks() {
        let filename = Filename::new("property_bank.json".into());
        let view = RawPropertyBankView::new(
            filename,
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
