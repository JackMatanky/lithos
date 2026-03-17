//! Raw file version views for schema persistence.
//!
//! These types track raw file version history with hashing,
//! enabling staleness detection without full re-parsing.

use std::{
    collections::{BTreeMap, VecDeque},
    time::SystemTime,
};

use rkyv::{
    Archive, Deserialize, Serialize,
    with::{AsUnixTime, Map},
};

use super::{FilePath, PropertyBankVersion, SchemaVersion};
use crate::schema::{
    aggregate::SchemaName, property::PropertyName, raw::RawSchema,
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
        Option<crate::schema::raw::RawPropertyBank>,
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
        raw: &super::super::raw::RawPropertyBank,
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

/// A single version of a raw file with hash and metadata.
///
/// Stores content hash and per-property hashes for staleness detection.
/// Timestamps are optional (files might not have filesystem metadata).
///
/// # Examples
///
/// ```ignore
/// use lithos_core::schema::views::RawFileVersion;
///
/// let version = RawFileVersion::new(
///     Some(created_at),
///     Some(modified_at),
/// )?;
/// # Ok::<(), std::io::Error>(())
/// ```
#[derive(Debug, Clone, PartialEq, Archive, Serialize, Deserialize)]
pub struct RawFileVersion {
    /// Blake3 hash of uncompressed content.
    content_hash: [u8; 32],
    /// Per-property Blake3 hashes for incremental resolution.
    ///
    /// Maps property name to the hash of its definition.
    /// Enables detecting which specific properties changed, allowing
    /// incremental re-resolution of only affected properties instead of
    /// full schema re-resolution.
    property_hashes: BTreeMap<PropertyName, [u8; 32]>,
    /// Compressed original file content (zstd level 3) - enables exact
    /// reconstruction.
    ///
    /// Stored as `Option<Vec<u8>>` to support legacy versions without content,
    /// but all new versions should include this for cache reconstruction.
    compressed_content: Option<Vec<u8>>,
    /// File creation timestamp (from filesystem).
    #[rkyv(with = Map<AsUnixTime>)]
    created_at: Option<SystemTime>,
    /// File modification timestamp (from filesystem).
    #[rkyv(with = Map<AsUnixTime>)]
    modified_at: Option<SystemTime>,
    /// When this version was recorded in the database.
    #[rkyv(with = AsUnixTime)]
    recorded_at: SystemTime,
}

impl RawFileVersion {
    /// Creates a new file version from metadata.
    ///
    /// # Parameters
    /// - `content_hash`: Blake3 hash of file content (computed by caller)
    /// - `property_hashes`: Per-property hashes for incremental resolution
    ///   (computed by caller)
    /// - `created_at`: File creation timestamp
    /// - `modified_at`: File modification timestamp
    /// - `compressed_content`: Optional zstd-compressed file content for
    ///   reconstruction
    #[inline]
    #[must_use]
    pub fn new(
        content_hash: [u8; 32],
        property_hashes: BTreeMap<PropertyName, [u8; 32]>,
        compressed_content: Option<Vec<u8>>,
        created_at: Option<SystemTime>,
        modified_at: Option<SystemTime>,
    ) -> Self {
        let recorded_at = SystemTime::now();

        Self {
            content_hash,
            property_hashes,
            compressed_content,
            created_at,
            modified_at,
            recorded_at,
        }
    }

    /// Returns the Blake3 hash of the content.
    #[inline]
    #[must_use]
    pub fn content_hash(&self) -> &[u8; 32] {
        &self.content_hash
    }

    /// Returns file creation timestamp.
    #[inline]
    #[must_use]
    pub fn created_at(&self) -> Option<SystemTime> {
        self.created_at
    }

    /// Returns file modification timestamp.
    #[inline]
    #[must_use]
    pub fn modified_at(&self) -> Option<SystemTime> {
        self.modified_at
    }

    /// Returns database recording timestamp.
    #[inline]
    #[must_use]
    pub fn recorded_at(&self) -> SystemTime {
        self.recorded_at
    }

    /// Returns per-property hashes.
    ///
    /// Used for incremental resolution - compare with new hashes to detect
    /// which specific properties changed.
    #[inline]
    #[must_use]
    pub fn property_hashes(&self) -> &BTreeMap<PropertyName, [u8; 32]> {
        &self.property_hashes
    }

    /// Computes which properties changed by comparing hashes.
    ///
    /// Returns a list of property names that have different hashes,
    /// enabling incremental re-resolution of only affected properties.
    #[inline]
    #[must_use]
    pub fn changed_properties(
        &self,
        new_hashes: &BTreeMap<PropertyName, [u8; 32]>,
    ) -> Vec<PropertyName> {
        let mut changed = Vec::new();

        // Check for modified or added properties
        for (name, new_hash) in new_hashes {
            if self.property_hashes.get(name) != Some(new_hash) {
                changed.push(name.clone());
            }
        }

        // Check for removed properties
        for name in self.property_hashes.keys() {
            if !new_hashes.contains_key(name) {
                changed.push(name.clone());
            }
        }

        changed
    }

    /// Checks if timestamps match (fast staleness check).
    ///
    /// Returns `true` if both `created_at` and `modified_at` match exactly.
    /// This is the fast path for staleness detection.
    #[inline]
    #[must_use]
    pub fn is_timestamp_match(
        &self,
        created_at: Option<SystemTime>,
        modified_at: Option<SystemTime>,
    ) -> bool {
        self.created_at == created_at && self.modified_at == modified_at
    }

    /// Checks if content matches via hash (accurate staleness check).
    ///
    /// Returns `true` if the provided Blake3 hash matches the stored hash.
    /// This is the accurate staleness check that handles timestamp edge cases
    /// (e.g., file restored from backup, git checkout).
    ///
    /// # Examples
    ///
    /// ```ignore
    /// use lithos_core::schema::views::raw::RawFileVersion;
    /// use std::time::SystemTime;
    ///
    /// let content = "schema content";
    /// let version = RawFileVersion::new(/* ... */);
    /// let hash = blake3::hash(content.as_bytes());
    ///
    /// assert!(version.is_content_match(hash.as_bytes()));
    /// ```
    #[inline]
    #[must_use]
    pub fn is_content_match(&self, content_hash: &[u8; 32]) -> bool {
        &self.content_hash == content_hash
    }

    /// Decompresses the stored content.
    ///
    /// Returns `None` if no content is stored, or an error if decompression
    /// fails.
    ///
    /// # Errors
    /// Returns error if zstd decompression fails.
    #[inline]
    #[must_use]
    pub fn decompress_content(&self) -> Option<Result<String, std::io::Error>> {
        let compressed = self.compressed_content.as_ref()?;

        match zstd::decode_all(compressed.as_slice()) {
            Ok(bytes) => {
                // Convert decompressed bytes to UTF-8 string
                match String::from_utf8(bytes) {
                    Ok(s) => Some(Ok(s)),
                    Err(e) => Some(Err(std::io::Error::new(
                        std::io::ErrorKind::InvalidData,
                        format!("UTF-8 decode failed: {e}"),
                    ))),
                }
            }
            Err(e) => Some(Err(e)),
        }
    }

    /// Compresses content using zstd level 3 (balanced speed/size).
    ///
    /// # Errors
    /// Returns error if zstd compression fails.
    #[inline]
    #[cfg_attr(
        not(test),
        expect(
            dead_code,
            reason = "Used in tests; prepared for future loader implementation"
        )
    )]
    pub(crate) fn compress_content(content: &str) -> std::io::Result<Vec<u8>> {
        const COMPRESSION_LEVEL: i32 = 3;
        zstd::encode_all(content.as_bytes(), COMPRESSION_LEVEL)
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn compress_content_succeeds_for_valid_string() {
        let content = "Test content for compression";
        let result = RawFileVersion::compress_content(content);
        assert!(result.is_ok());
        let compressed = result.unwrap();
        assert!(!compressed.is_empty());
        // Compressed data should be smaller or similar size for short strings
        // (zstd might not compress very short strings well)
    }

    #[test]
    fn compress_content_handles_empty_string() {
        let content = "";
        let result = RawFileVersion::compress_content(content);
        result.unwrap();
    }

    #[test]
    fn compress_content_handles_unicode() {
        let content = "Hello \u{4e16}\u{754c} \u{1f980}";
        let result = RawFileVersion::compress_content(content);
        result.unwrap();
    }

    #[test]
    fn decompress_content_returns_none_when_no_content_stored() {
        let version = RawFileVersion::new(
            [0; 32],
            BTreeMap::new(),
            None,
            None,
            None, // No compressed content
        );
        assert!(version.decompress_content().is_none());
    }

    #[test]
    fn decompress_content_roundtrip_succeeds() {
        let original = "Test content for compression roundtrip";
        let compressed = RawFileVersion::compress_content(original)
            .expect("compression failed");

        let version = RawFileVersion::new(
            [0; 32],
            BTreeMap::new(),
            Some(compressed),
            None,
            None,
        );

        let decompressed = version
            .decompress_content()
            .expect("should have content")
            .expect("decompression should succeed");

        assert_eq!(decompressed, original);
    }

    #[test]
    fn decompress_content_handles_unicode_roundtrip() {
        let original = "Hello \u{4e16}\u{754c} \u{1f980} with special chars: \
                        \n\t\"quotes\"";
        let compressed = RawFileVersion::compress_content(original)
            .expect("compression failed");

        let version = RawFileVersion::new(
            [0; 32],
            BTreeMap::new(),
            Some(compressed),
            None,
            None,
        );

        let decompressed = version
            .decompress_content()
            .expect("should have content")
            .expect("decompression should succeed");

        assert_eq!(decompressed, original);
    }

    #[test]
    fn decompress_content_fails_for_invalid_compressed_data() {
        let invalid_data = vec![0xFF, 0xFF, 0xFF, 0xFF];
        let version = RawFileVersion::new(
            [0; 32],
            BTreeMap::new(),
            Some(invalid_data),
            None,
            None,
        );

        let result = version.decompress_content().expect("should have content");
        result.unwrap_err();
    }

    #[test]
    fn raw_schema_view_to_raw_reconstructs_schema() {
        use std::collections::HashMap;

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

    #[test]
    fn raw_file_version_stores_compressed_content() {
        let content = "Test content";
        let compressed = RawFileVersion::compress_content(content)
            .expect("compression failed");

        let version = RawFileVersion::new(
            [1; 32],
            BTreeMap::new(),
            Some(compressed.clone()),
            None,
            None,
        );

        // Verify the compressed content is stored
        let decompressed = version
            .decompress_content()
            .expect("should have content")
            .expect("decompression should succeed");

        assert_eq!(decompressed, content);
    }
}
