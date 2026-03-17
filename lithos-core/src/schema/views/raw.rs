//! Raw schema and property bank views for persistence.
//!
//! These types track version history for schemas and property banks,
//! enabling staleness detection and incremental updates.

use std::collections::VecDeque;

use rkyv::{Archive, Deserialize, Serialize};

use super::{FilePath, PropertyBankVersion, SchemaVersion};
use crate::schema::{
    aggregate::SchemaName,
    property::PropertyName,
    raw::{RawPropertyBank, RawSchema},
};

/// Maximum number of versions to retain per file.
const MAX_VERSIONS: usize = 5;

/// Raw schema file with version history.
///
/// Tracks up to 5 versions of a schema file. Each version includes inheritance
/// metadata (`extends`, `excludes`) to enable incremental resolution.
///
/// # Examples
///
/// ```ignore
/// use lithos_core::schema::views::{RawSchemaView, FilePath, SchemaVersion};
///
/// let file_path = FilePath::new("schemas/note.toml".into());
/// let version = SchemaVersion::new(/* ... */)?;
/// let view = RawSchemaView::new(file_path, version);
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
#[derive(Debug, Clone, PartialEq, Archive, Serialize, Deserialize)]
pub struct RawSchemaView {
    /// File path relative to vault root.
    file_path: FilePath,

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
    pub fn new(file_path: FilePath, version: SchemaVersion) -> Self {
        let mut versions = VecDeque::with_capacity(MAX_VERSIONS);
        versions.push_front(version);

        Self {
            file_path,
            versions,
        }
    }

    /// Returns the file path.
    #[inline]
    #[must_use]
    pub fn file_path(&self) -> &FilePath {
        &self.file_path
    }

    /// Returns the schema name (derived from file basename).
    ///
    /// # Examples
    /// ```ignore
    /// let view = RawSchemaView::new(
    ///     FilePath::new("schemas/note.toml".into()),
    ///     version,
    /// );
    /// assert_eq!(view.name(), "note");
    /// ```
    #[inline]
    #[must_use]
    pub fn name(&self) -> &str {
        self.file_path.basename()
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
        if self.versions.len() >= MAX_VERSIONS {
            self.versions.pop_back(); // Remove oldest
        }

        self.versions.push_front(version);
    }

    /// Reconstructs `RawSchema` from the current version.
    ///
    /// Returns `None` if no current version exists.
    ///
    /// The schema name is derived from the file path basename.
    ///
    /// # Errors
    /// Returns error if deserialization of properties fails.
    #[inline]
    pub fn to_raw(
        &self,
    ) -> Result<Option<RawSchema>, crate::schema::error::SchemaIngestionError>
    {
        let Some(version) = self.current() else {
            return Ok(None);
        };
        let name = self.name().into();
        version.to_raw(name).map(Some)
    }

    /// Creates a view from a raw schema with content.
    ///
    /// This method bridges the old ingestor API with the new
    /// SchemaVersion-based storage. It will be simplified in Phase 3 when
    /// the ingestor is updated.
    ///
    /// # Parameters
    /// - `raw`: The parsed raw schema
    /// - `file_path`: The relative path to the schema file (for view indexing)
    /// - `content`: The uncompressed file content (unused - for API
    ///   compatibility)
    ///
    /// # Errors
    /// Returns error if metadata is missing or validation fails.
    #[inline]
    pub fn try_from_with_content(
        raw: &super::super::raw::RawSchema,
        file_path: &str,
        content: &str,
    ) -> Result<Self, crate::schema::error::SchemaIngestionError> {
        use super::{FileTimesMetadata, HashMetadata};

        // Compute content hash from raw file content
        let content_hash = blake3::hash(content.as_bytes());

        // Compute per-property hashes
        let property_hashes =
            HashMetadata::compute_property_hashes(&raw.properties);

        let file_times = FileTimesMetadata::new(
            raw.metadata.created_at,
            raw.metadata.modified_at,
        );
        let hashes =
            HashMetadata::new(*content_hash.as_bytes(), property_hashes);

        let version = SchemaVersion::new(file_times, hashes, raw)?;

        Ok(Self::new(FilePath::new(file_path.into()), version))
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
/// let view = RawPropertyBankView::new(content_hash, property_hashes, created_at, modified_at);
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
#[derive(Debug, Clone, PartialEq, Archive, Serialize, Deserialize)]
pub struct RawPropertyBankView {
    /// Version history (ring buffer, max 5 versions, newest first).
    versions: VecDeque<PropertyBankVersion>,
}

impl RawPropertyBankView {
    /// Creates a new property bank view with initial version.
    #[inline]
    #[must_use]
    pub fn new(version: PropertyBankVersion) -> Self {
        let mut versions = VecDeque::with_capacity(MAX_VERSIONS);
        versions.push_front(version);

        Self {
            versions,
        }
    }

    /// Returns the most recent version, if any.
    #[inline]
    #[must_use]
    pub fn current(&self) -> Option<&PropertyBankVersion> {
        self.versions.front()
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
        if self.versions.len() >= MAX_VERSIONS {
            self.versions.pop_back(); // Remove oldest
        }

        self.versions.push_front(version);
    }

    /// Reconstructs `RawPropertyBank` from cached compressed content.
    ///
    /// This enables reusing cached property bank data without re-reading files.
    /// Returns `None` if no compressed content is stored, or if
    /// decompression/parsing fails.
    ///
    /// # Design Note
    ///
    /// Reconstructs the raw property bank from cached compressed content.
    ///
    /// Returns `None` if no current version exists, no compressed content is
    /// stored, or if decompression/parsing fails.
    ///
    /// The `path` parameter is needed to determine the file format (JSON, TOML,
    /// or YAML) for parsing.
    ///
    /// This enables the Fresh optimization - returning cached data without
    /// re-reading or re-parsing the file.
    /// Reconstructs `RawPropertyBank` from the current version.
    ///
    /// Returns `None` if no current version exists.
    ///
    /// # Errors
    /// Returns error if deserialization of properties fails.
    #[inline]
    pub fn to_raw(
        &self,
    ) -> Result<
        Option<RawPropertyBank>,
        crate::schema::error::SchemaIngestionError,
    > {
        let Some(version) = self.current() else {
            return Ok(None);
        };
        version.to_raw().map(Some)
    }

    /// Creates a view from a raw property bank with content.
    ///
    /// This is the complete version of `TryFrom` that accepts the file content
    /// and compresses it for caching. Use this when you have the content
    /// available and want to enable the Fresh optimization.
    ///
    /// # Errors
    /// Returns error if metadata is missing or validation fails.
    #[inline]
    pub fn try_from_with_content(
        raw: &RawPropertyBank,
        content: &str,
    ) -> Result<Self, crate::schema::error::SchemaIngestionError> {
        use super::{FileTimesMetadata, HashMetadata};

        // Compute content hash from raw file content
        let content_hash = blake3::hash(content.as_bytes());

        // Compute per-property hashes
        let property_hashes =
            HashMetadata::compute_property_hashes_for_bank(&raw.properties);

        let file_times = FileTimesMetadata::new(
            raw.metadata.created_at,
            raw.metadata.modified_at,
        );
        let hashes =
            HashMetadata::new(*content_hash.as_bytes(), property_hashes);

        let version = PropertyBankVersion::new(file_times, hashes, raw)?;

        Ok(Self::new(version))
    }
}

// ----------------------------------------------------------- //
//                            Tests                            //
// ----------------------------------------------------------- //

#[cfg(test)]
mod tests {
    use std::collections::{BTreeMap, HashMap};

    use super::*;

    #[test]
    fn raw_schema_view_to_raw_reconstructs_schema() {
        use super::super::{FileTimesMetadata, HashMetadata};
        use crate::schema::raw::{
            RawSchema, RawSchemaMetadata, RawSchemaVersion,
        };

        // Create a test RawSchema
        let raw = RawSchema {
            version: RawSchemaVersion::default(),
            name: "test".into(),
            extends: None,
            excludes: Vec::new(),
            properties: HashMap::new(),
            metadata: RawSchemaMetadata::default(),
        };

        let file_times = FileTimesMetadata::new(None, None);
        let hashes = HashMetadata::new([0; 32], BTreeMap::new());
        let version = SchemaVersion::new(file_times, hashes, &raw).unwrap();

        let file_path = FilePath::new("schemas/test.toml".into());
        let view = RawSchemaView::new(file_path, version);

        let reconstructed =
            view.to_raw().expect("should succeed").expect("should have value");
        assert_eq!(reconstructed.name.as_ref(), "test");
    }

    #[test]
    fn raw_property_bank_view_to_raw_reconstructs_property_bank() {
        use std::collections::HashMap;

        use super::super::{FileTimesMetadata, HashMetadata};
        use crate::schema::raw::{
            RawPropertyBank, RawSchemaMetadata, RawSchemaVersion,
        };

        // Create a test RawPropertyBank
        let raw = RawPropertyBank {
            version: RawSchemaVersion::default(),
            properties: HashMap::new(),
            metadata: RawSchemaMetadata::default(),
        };

        let file_times = FileTimesMetadata::new(None, None);
        let hashes = HashMetadata::new([0; 32], BTreeMap::new());
        let version =
            PropertyBankVersion::new(file_times, hashes, &raw).unwrap();

        let view = RawPropertyBankView::new(version);

        let reconstructed =
            view.to_raw().expect("should succeed").expect("should have value");
        assert_eq!(reconstructed.properties.len(), 0);
    }
}
