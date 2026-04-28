//! Raw schema and property bank views for persistence.
//!
//! These types track version history for schemas and property banks,
//! enabling staleness detection and incremental updates.

#![expect(
    clippy::same_name_method,
    reason = "Trait contracts intentionally mirror existing view API names"
)]

use std::collections::VecDeque;

use rkyv::{Archive, Deserialize, Serialize};

use super::{
    FileTimesMetadata, HashMetadata, PropertyBankVersion, SchemaVersion,
    version::{Version as _, VersionRead as _},
};
use crate::{
    schema::{
        aggregate::SchemaName, error::SchemaStorageError,
        property::PropertyName, raw::RawPropertyBank,
    },
    support::hash::Blake3Hash,
};

/// Shared behavior for raw file views with version history.
#[expect(
    clippy::arbitrary_source_item_ordering,
    reason = "Trait items are grouped by lifecycle semantics"
)]
pub trait RawView {
    /// Concrete version payload type.
    type Version: super::version::Version;
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
        created_at: Option<std::time::SystemTime>,
        modified_at: Option<std::time::SystemTime>,
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
    #[inline]
    fn update_timestamps(&mut self, file_times: FileTimesMetadata) {
        if let Some(current) = self.current_mut() {
            current.set_file_times(file_times);
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
        created_at: Option<std::time::SystemTime>,
        modified_at: Option<std::time::SystemTime>,
    ) -> bool;

    /// Returns number of retained historical versions.
    fn version_count(&self) -> usize;
}

/// Raw schema file with version history.
///
/// Tracks up to 5 versions of a schema file. Each version includes inheritance
/// metadata (`extends`, `excludes`) to enable incremental resolution.
///
/// # Examples
///
/// ```ignore
/// use lithos_core::schema::views::{RawSchemaView, SchemaVersion};
/// use lithos_core::fs::Filename;
///
/// let filename = Filename::new("note.toml".into());
/// let version = SchemaVersion::new(/* ... */)?;
/// let view = RawSchemaView::new(filename, version);
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
#[derive(Debug, Clone, PartialEq, Archive, Serialize, Deserialize)]
pub struct RawSchemaView {
    /// Vault-relative schema path (e.g., "schemas/note.toml").
    path: crate::fs::RelativePath,

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
    pub fn new(path: crate::fs::RelativePath, version: SchemaVersion) -> Self {
        let mut versions =
            VecDeque::with_capacity(<Self as RawView>::MAX_VERSIONS);
        versions.push_front(version);

        Self {
            path,
            versions,
        }
    }

    /// Returns the filename.
    #[inline]
    #[must_use]
    pub fn file_path(&self) -> &crate::fs::RelativePath {
        &self.path
    }

    /// Returns the schema name (derived from filename without extension).
    ///
    /// # Examples
    /// ```ignore
    /// let view = RawSchemaView::new(
    ///     Filename::new("note.toml".into()),
    ///     version,
    /// );
    /// assert_eq!(view.name(), "note");
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

    /// Returns the parent schema name (`extends`) from current version, if any.
    #[inline]
    #[must_use]
    pub fn extends(&self) -> Option<&SchemaName> {
        let v = self.current()?;
        v.extends()
    }

    /// Returns the excluded property names from current version.
    #[inline]
    #[must_use]
    pub fn excludes(&self) -> &[PropertyName] {
        self.current().map_or(&[], super::version::SchemaVersion::excludes)
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

    /// Creates a view from a raw schema with content.
    ///
    /// This method bridges the old ingestor API with the new
    /// SchemaVersion-based storage. It will be simplified in Phase 3 when
    /// the ingestor is updated.
    ///
    /// # Parameters
    /// - `raw`: The parsed raw schema
    /// - `filename`: The filename with extension (e.g., "note.toml")
    /// - `content`: The uncompressed file content (unused - for API
    ///   compatibility)
    ///
    /// # Errors
    /// Returns error if metadata is missing or validation fails.
    #[inline]
    pub fn try_from_with_content(
        raw: &super::super::raw::RawSchema,
        path: crate::fs::RelativePath,
        content: &str,
    ) -> Result<Self, crate::schema::error::SchemaIngestionError> {
        use super::{FileTimesMetadata, HashMetadata};

        // Compute content hash from raw file content
        let content_hash = Blake3Hash::compute(content.as_bytes());

        // Compute per-property hashes
        let property_hashes =
            HashMetadata::compute_property_hashes(raw.properties());

        let file_times = FileTimesMetadata::new(
            raw.file_stats().created_at(),
            raw.file_stats().modified_at(),
        );
        let hashes = HashMetadata::new(content_hash, property_hashes);

        let version = SchemaVersion::new(file_times, hashes, raw)?;

        Ok(Self::new(path, version))
    }
}

#[expect(
    clippy::missing_trait_methods,
    reason = "Default trait methods are intentionally reused"
)]
impl RawView for RawSchemaView {
    type FilePath = crate::fs::RelativePath;
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
    fn update_content_hash(
        &mut self,
        content_hash: Blake3Hash,
    ) -> Result<(), SchemaStorageError> {
        let current = self.current().ok_or(SchemaStorageError::NotFound {
            name: "current schema version".into(),
        })?;
        let file_times = current.file_times().clone();
        let hashes = HashMetadata::new(
            content_hash,
            current.hashes().properties().clone(),
        );
        let version = SchemaVersion::with_metadata(current, file_times, hashes);
        self.add_version(version);
        Ok(())
    }
}

impl RawViewRead for RawSchemaView {
    #[inline]
    fn is_timestamp_match(
        &self,
        created_at: Option<std::time::SystemTime>,
        modified_at: Option<std::time::SystemTime>,
    ) -> bool {
        RawView::is_timestamp_match(self, created_at, modified_at)
    }

    #[inline]
    fn is_content_match(&self, content_hash: &Blake3Hash) -> bool {
        RawView::is_content_match(self, content_hash)
    }

    #[inline]
    fn version_count(&self) -> usize {
        self.version_count()
    }
}

/// Raw property bank file with version history.
///
/// Tracks up to 5 versions of the property bank file for staleness detection.
///
/// # Examples
///
/// ```ignore
/// use lithos_core::schema::views::RawPropertyBankView;
///
/// let view = RawPropertyBankView::new(filename, version);
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
#[derive(Debug, Clone, PartialEq, Archive, Serialize, Deserialize)]
pub struct RawPropertyBankView {
    /// Filename with extension (e.g., "properties.yaml").
    filename: crate::fs::Filename,

    /// Version history (ring buffer, max 5 versions, newest first).
    versions: VecDeque<PropertyBankVersion>,
}

impl RawPropertyBankView {
    /// Creates a new property bank view with initial version.
    #[inline]
    #[must_use]
    pub fn new(
        filename: crate::fs::Filename,
        version: PropertyBankVersion,
    ) -> Self {
        let mut versions =
            VecDeque::with_capacity(<Self as RawView>::MAX_VERSIONS);
        versions.push_front(version);

        Self {
            filename,
            versions,
        }
    }

    /// Returns the filename.
    #[inline]
    #[must_use]
    pub fn file_path(&self) -> &crate::fs::Filename {
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
    ///
    /// Takes a `FileTimesMetadata` to automatically get a fresh `recorded_at`
    /// timestamp.
    #[inline]
    pub fn update_timestamps(&mut self, file_times: FileTimesMetadata) {
        if let Some(current) = self.versions.front_mut() {
            current.set_file_times(file_times);
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
        let file_times = current.file_times().clone();

        let hashes = HashMetadata::new(
            content_hash,
            current.hashes().properties().clone(),
        );
        let version =
            PropertyBankVersion::new(file_times, hashes, current.version())
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
        filename: crate::fs::Filename,
        raw_hash: HashMetadata,
    ) -> Result<Self, crate::schema::error::SchemaIngestionError> {
        use super::FileTimesMetadata;

        let file_times = FileTimesMetadata::new(
            raw.file_stats().created_at(),
            raw.file_stats().modified_at(),
        );

        let version = PropertyBankVersion::new(
            file_times,
            raw_hash,
            raw.version().as_str(),
        )?;

        Ok(Self::new(filename, version))
    }
}

#[expect(
    clippy::missing_trait_methods,
    reason = "Default trait methods are intentionally reused"
)]
impl RawView for RawPropertyBankView {
    type FilePath = crate::fs::Filename;
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
    fn is_timestamp_match(
        &self,
        created_at: Option<std::time::SystemTime>,
        modified_at: Option<std::time::SystemTime>,
    ) -> bool {
        RawView::is_timestamp_match(self, created_at, modified_at)
    }

    #[inline]
    fn is_content_match(&self, content_hash: &Blake3Hash) -> bool {
        RawView::is_content_match(self, content_hash)
    }

    #[inline]
    fn version_count(&self) -> usize {
        self.version_count()
    }
}

impl RawViewRead for ArchivedRawSchemaView {
    #[inline]
    fn is_timestamp_match(
        &self,
        created_at: Option<std::time::SystemTime>,
        modified_at: Option<std::time::SystemTime>,
    ) -> bool {
        self.versions.as_slice().first().is_some_and(|version| {
            version.is_timestamp_match(created_at, modified_at)
        })
    }

    #[inline]
    fn is_content_match(&self, content_hash: &Blake3Hash) -> bool {
        self.versions
            .as_slice()
            .first()
            .is_some_and(|version| version.is_content_match(content_hash))
    }

    #[inline]
    fn version_count(&self) -> usize {
        self.versions.len()
    }
}

impl RawViewRead for ArchivedRawPropertyBankView {
    #[inline]
    fn is_timestamp_match(
        &self,
        created_at: Option<std::time::SystemTime>,
        modified_at: Option<std::time::SystemTime>,
    ) -> bool {
        self.versions.as_slice().first().is_some_and(|version| {
            version.is_timestamp_match(created_at, modified_at)
        })
    }

    #[inline]
    fn is_content_match(&self, content_hash: &Blake3Hash) -> bool {
        self.versions
            .as_slice()
            .first()
            .is_some_and(|version| version.is_content_match(content_hash))
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
    use crate::schema::{
        property::{PropertyMap, PropertyName},
        raw::{RawPropertyBank, RawSchema},
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
            FileTimesMetadata::new(None, None),
            HashMetadata::new(content_hash, HashMap::new()),
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
            FileTimesMetadata::new(None, None),
            HashMetadata::new(content_hash, property_hashes),
            "1.0",
        )
        .expect("property bank version should build")
    }

    #[test]
    fn property_bank_view_rotation_uses_trait_max_versions() {
        let filename = crate::fs::Filename::new("property_bank.json".into());
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
        let filename = crate::fs::Filename::new("property_bank.json".into());
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
        let schema_path =
            crate::fs::RelativePath::try_from("schemas/note.json").unwrap();
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
    fn property_bank_update_content_hash_errors_without_current_version() {
        let filename = crate::fs::Filename::new("property_bank.json".into());
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
            crate::fs::Filename::new("property_bank.json".into()),
            HashMetadata::new(Blake3Hash::new([1; 32]), HashMap::new()),
        )
        .expect("view creation should succeed");

        assert_eq!(view.version_count(), 1);
    }

    #[test]
    fn archived_raw_property_bank_view_supports_zero_copy_staleness_checks() {
        let filename = crate::fs::Filename::new("property_bank.json".into());
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
