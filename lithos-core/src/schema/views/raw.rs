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

use crate::schema::{aggregate::SchemaName, property::PropertyName};

/// Maximum number of versions to retain per file.
const MAX_VERSIONS: usize = 5;

/// Raw schema file with version history and inheritance metadata.
///
/// Tracks up to 5 versions of a schema file, plus inheritance information
/// to enable incremental resolution when only `extends` or `excludes` changes.
///
/// # Examples
///
/// ```ignore
/// use lithos_core::schema::views::RawSchemaView;
///
/// let view = RawSchemaView::new(
///     "schemas/note.toml".into(),
///     Some(SchemaName::try_new("base-note")?),
///     vec![PropertyName::try_new("internal_id")?],
/// );
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
#[derive(Debug, Clone, PartialEq, Archive, Serialize, Deserialize)]
pub struct RawSchemaView {
    /// File path relative to vault root.
    file_path: Box<str>,

    /// Parent schema name (from `extends` field).
    ///
    /// Stored here to enable incremental resolution when only inheritance
    /// changes. None for root schemas.
    extends: Option<SchemaName>,

    /// Property names to exclude from parent (from `excludes` field).
    ///
    /// Stored here to detect inheritance changes without full re-parse.
    excludes: Vec<PropertyName>,

    /// Version history (ring buffer, max 5 versions, newest first).
    ///
    /// Using `VecDeque` allows efficient `push_front`/`pop_back` for version
    /// rotation.
    versions: VecDeque<RawFileVersion>,
}

impl RawSchemaView {
    /// Creates a new schema view with initial version.
    ///
    /// # Errors
    /// Returns error if compression fails.
    #[expect(
        clippy::too_many_arguments,
        reason = "Builder pattern refactor tracked - need to preserve all \
                  file metadata for staleness detection"
    )]
    #[inline]
    #[must_use]
    pub fn new(
        file_path: Box<str>,
        extends: Option<SchemaName>,
        excludes: Vec<PropertyName>,
        content_hash: [u8; 32],
        property_hashes: BTreeMap<PropertyName, [u8; 32]>,
        created_at: Option<SystemTime>,
        modified_at: Option<SystemTime>,
        compressed_content: Option<Vec<u8>>,
    ) -> Self {
        let mut versions = VecDeque::with_capacity(MAX_VERSIONS);
        let version = RawFileVersion::new(
            content_hash,
            property_hashes,
            created_at,
            modified_at,
            compressed_content,
        );
        versions.push_front(version);

        Self {
            file_path,
            extends,
            excludes,
            versions,
        }
    }

    /// Returns the file path.
    #[inline]
    #[must_use]
    pub fn file_path(&self) -> &str {
        &self.file_path
    }

    /// Returns the parent schema name (`extends`), if any.
    #[inline]
    #[must_use]
    pub fn extends(&self) -> Option<&SchemaName> {
        self.extends.as_ref()
    }

    /// Returns the excluded property names.
    #[inline]
    #[must_use]
    pub fn excludes(&self) -> &[PropertyName] {
        &self.excludes
    }

    /// Returns the most recent version, if any.
    #[inline]
    #[must_use]
    pub fn current(&self) -> Option<&RawFileVersion> {
        self.versions.front()
    }

    /// Returns all tracked versions (newest first).
    #[inline]
    #[must_use]
    pub fn versions(&self) -> &VecDeque<RawFileVersion> {
        &self.versions
    }

    /// Returns the number of tracked versions.
    #[inline]
    #[must_use]
    pub fn version_count(&self) -> usize {
        self.versions.len()
    }

    /// Adds a new version (evicts oldest if at capacity).
    #[expect(
        clippy::too_many_arguments,
        reason = "Consistent with RawFileVersion::new signature - represents \
                  file metadata"
    )]
    #[inline]
    pub fn add_version(
        &mut self,
        content_hash: [u8; 32],
        property_hashes: BTreeMap<PropertyName, [u8; 32]>,
        created_at: Option<SystemTime>,
        modified_at: Option<SystemTime>,
        compressed_content: Option<Vec<u8>>,
    ) {
        let version = RawFileVersion::new(
            content_hash,
            property_hashes,
            created_at,
            modified_at,
            compressed_content,
        );

        if self.versions.len() >= MAX_VERSIONS {
            self.versions.pop_back(); // Remove oldest
        }

        self.versions.push_front(version);
    }

    /// Checks if the current version is fresh (matches provided metadata).
    ///
    /// Returns `true` if timestamps match OR (content hash matches AND property
    /// hashes match). This is a hybrid staleness check optimizing for the
    /// common case (timestamp check) while being accurate for edge cases
    /// (hash check).
    ///
    /// # Examples
    ///
    /// ```ignore
    /// use lithos_core::schema::raw::RawSchemaMetadata;
    ///
    /// if view.is_fresh(&raw_schema.metadata) {
    ///     // Skip re-resolution
    /// }
    /// ```
    #[inline]
    #[must_use]
    pub fn is_fresh(
        &self,
        metadata: &super::super::raw::RawSchemaMetadata,
    ) -> bool {
        self.current().is_some_and(|_current| {
            self.is_timestamp_match(metadata)
                || (self.is_content_match(metadata)
                    && self.is_properties_match(metadata))
        })
    }

    /// Checks if timestamps match the current version.
    #[inline]
    #[must_use]
    pub fn is_timestamp_match(
        &self,
        metadata: &super::super::raw::RawSchemaMetadata,
    ) -> bool {
        self.current().is_some_and(|current| {
            current
                .is_timestamp_match(metadata.created_at, metadata.modified_at)
        })
    }

    /// Checks if content hash matches the current version.
    #[inline]
    #[must_use]
    pub fn is_content_match(
        &self,
        metadata: &super::super::raw::RawSchemaMetadata,
    ) -> bool {
        self.current().is_some_and(|current| {
            metadata
                .content_hash
                .is_some_and(|hash| hash == current.content_hash)
        })
    }

    /// Checks if all property hashes match the current version.
    #[inline]
    #[must_use]
    pub fn is_properties_match(
        &self,
        metadata: &super::super::raw::RawSchemaMetadata,
    ) -> bool {
        self.current().is_some_and(|current| {
            // All properties in metadata must match current version
            // Need to convert Box<str> keys to PropertyName for lookup
            metadata.property_hashes.iter().all(|(name, hash)| {
                PropertyName::try_new(name.as_ref())
                    .ok()
                    .and_then(|prop_name| {
                        current.property_hashes.get(&prop_name)
                    })
                    .is_some_and(|current_hash| current_hash == hash)
            })
        })
    }

    /// Returns the list of properties that changed between the current version
    /// and provided metadata.
    ///
    /// Returns property names that:
    /// - Were added (in metadata but not in current version)
    /// - Were removed (in current version but not in metadata)
    /// - Were modified (different hash)
    ///
    /// # Examples
    ///
    /// ```ignore
    /// let changed = view.filter_changed_properties(&raw_schema.metadata);
    /// if !changed.is_empty() {
    ///     // Re-resolve only these properties
    /// }
    /// ```
    #[must_use]
    #[inline]
    pub fn filter_changed_properties(
        &self,
        metadata: &super::super::raw::RawSchemaMetadata,
    ) -> Vec<PropertyName> {
        let Some(current) = self.current() else {
            return Vec::new();
        };

        let mut changed = Vec::new();

        // Check for added or modified properties
        for (name, new_hash) in &metadata.property_hashes {
            let Ok(prop_name) = PropertyName::try_new(name.as_ref()) else {
                continue;
            };

            let is_changed = match current.property_hashes.get(&prop_name) {
                Some(old_hash) if old_hash != new_hash => true, // Modified
                None => true,                                   // Added
                _ => false,                                     // Unchanged
            };

            if is_changed {
                changed.push(prop_name);
            }
        }

        // Check for removed properties (in current but not in metadata)
        for prop_name in current.property_hashes.keys() {
            let name_str: &str = prop_name.as_ref();
            if !metadata.property_hashes.contains_key(name_str) {
                changed.push(prop_name.clone());
            }
        }

        changed
    }

    /// Reconstructs `RawSchema` from cached compressed content.
    ///
    /// This enables reusing cached schema data without re-reading files.
    /// Returns `None` if no compressed content is stored, or if
    /// decompression/parsing fails.
    ///
    /// Reconstructs the raw schema from cached compressed content.
    ///
    /// Returns `None` if no current version exists, no compressed content is
    /// stored, or if decompression/parsing fails.
    ///
    /// The format (JSON/TOML/YAML) is detected from the `file_path` extension.
    ///
    /// This enables the Fresh optimization - returning cached data without
    /// re-reading or re-parsing the file.
    #[inline]
    #[must_use]
    pub fn to_raw(&self) -> Option<super::super::raw::RawSchema> {
        let version = self.current()?;
        let content = version.decompress_content()?.ok()?;

        // Parse based on file extension
        let path = std::path::Path::new(self.file_path.as_ref());
        let mut raw: super::super::raw::RawSchema =
            crate::fs::FsReader::parse_structured_from_str(path, &content)
                .ok()?;

        // Populate the name field from filename (serde skips this field)
        // The name is the filename without extension
        let filename_stem = path.file_stem()?.to_str()?;
        raw.name = filename_stem.into();

        Some(raw)
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
    versions: VecDeque<RawFileVersion>,
}

impl RawPropertyBankView {
    /// Creates a new property bank view with initial version.
    #[inline]
    #[must_use]
    pub fn new(
        content_hash: [u8; 32],
        property_hashes: BTreeMap<PropertyName, [u8; 32]>,
        created_at: Option<SystemTime>,
        modified_at: Option<SystemTime>,
        compressed_content: Option<Vec<u8>>,
    ) -> Self {
        let mut versions = VecDeque::with_capacity(MAX_VERSIONS);
        let version = RawFileVersion::new(
            content_hash,
            property_hashes,
            created_at,
            modified_at,
            compressed_content,
        );
        versions.push_front(version);

        Self {
            versions,
        }
    }

    /// Returns the most recent version, if any.
    #[inline]
    #[must_use]
    pub fn current(&self) -> Option<&RawFileVersion> {
        self.versions.front()
    }

    /// Returns all tracked versions (newest first).
    #[inline]
    #[must_use]
    pub fn versions(&self) -> &VecDeque<RawFileVersion> {
        &self.versions
    }

    /// Returns the number of tracked versions.
    #[inline]
    #[must_use]
    pub fn version_count(&self) -> usize {
        self.versions.len()
    }

    /// Adds a new version (evicts oldest if at capacity).
    #[expect(
        clippy::too_many_arguments,
        reason = "Consistent with RawFileVersion::new signature - represents \
                  file metadata"
    )]
    #[inline]
    pub fn add_version(
        &mut self,
        content_hash: [u8; 32],
        property_hashes: BTreeMap<PropertyName, [u8; 32]>,
        created_at: Option<SystemTime>,
        modified_at: Option<SystemTime>,
        compressed_content: Option<Vec<u8>>,
    ) {
        let version = RawFileVersion::new(
            content_hash,
            property_hashes,
            created_at,
            modified_at,
            compressed_content,
        );

        if self.versions.len() >= MAX_VERSIONS {
            self.versions.pop_back(); // Remove oldest
        }

        self.versions.push_front(version);
    }

    /// Checks if the current version is fresh (matches provided metadata).
    ///
    /// Returns `true` if timestamps match OR (content hash matches AND property
    /// hashes match). This is a hybrid staleness check optimizing for the
    /// common case (timestamp check) while being accurate for edge cases
    /// (hash check).
    #[inline]
    #[must_use]
    pub fn is_fresh(
        &self,
        metadata: &super::super::raw::RawSchemaMetadata,
    ) -> bool {
        self.current().is_some_and(|_current| {
            self.is_timestamp_match(metadata)
                || (self.is_content_match(metadata)
                    && self.is_properties_match(metadata))
        })
    }

    /// Checks if timestamps match the current version.
    #[inline]
    #[must_use]
    pub fn is_timestamp_match(
        &self,
        metadata: &super::super::raw::RawSchemaMetadata,
    ) -> bool {
        self.current().is_some_and(|current| {
            current
                .is_timestamp_match(metadata.created_at, metadata.modified_at)
        })
    }

    /// Checks if content hash matches the current version.
    #[inline]
    #[must_use]
    pub fn is_content_match(
        &self,
        metadata: &super::super::raw::RawSchemaMetadata,
    ) -> bool {
        self.current().is_some_and(|current| {
            metadata
                .content_hash
                .is_some_and(|hash| hash == current.content_hash)
        })
    }

    /// Checks if all property hashes match the current version.
    #[inline]
    #[must_use]
    pub fn is_properties_match(
        &self,
        metadata: &super::super::raw::RawSchemaMetadata,
    ) -> bool {
        self.current().is_some_and(|current| {
            // All properties in metadata must match current version
            // Need to convert Box<str> keys to PropertyName for lookup
            metadata.property_hashes.iter().all(|(name, hash)| {
                PropertyName::try_new(name.as_ref())
                    .ok()
                    .and_then(|prop_name| {
                        current.property_hashes.get(&prop_name)
                    })
                    .is_some_and(|current_hash| current_hash == hash)
            })
        })
    }

    /// Returns the list of properties that changed between the current version
    /// and provided metadata.
    ///
    /// Returns property names that:
    /// - Were added (in metadata but not in current version)
    /// - Were removed (in current version but not in metadata)
    /// - Were modified (different hash)
    #[must_use]
    #[inline]
    pub fn filter_changed_properties(
        &self,
        metadata: &super::super::raw::RawSchemaMetadata,
    ) -> Vec<PropertyName> {
        let Some(current) = self.current() else {
            return Vec::new();
        };

        let mut changed = Vec::new();

        // Check for added or modified properties
        for (name, new_hash) in &metadata.property_hashes {
            let Ok(prop_name) = PropertyName::try_new(name.as_ref()) else {
                continue;
            };

            let is_changed = match current.property_hashes.get(&prop_name) {
                Some(old_hash) if old_hash != new_hash => true, // Modified
                None => true,                                   // Added
                _ => false,                                     // Unchanged
            };

            if is_changed {
                changed.push(prop_name);
            }
        }

        // Check for removed properties (in current but not in metadata)
        for prop_name in current.property_hashes.keys() {
            let name_str: &str = prop_name.as_ref();
            if !metadata.property_hashes.contains_key(name_str) {
                changed.push(prop_name.clone());
            }
        }

        changed
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
    #[inline]
    #[must_use]
    pub fn to_raw(
        &self,
        path: &std::path::Path,
    ) -> Option<super::super::raw::RawPropertyBank> {
        let version = self.current()?;
        let content = version.decompress_content()?.ok()?;

        // Parse based on file extension
        crate::fs::FsReader::parse_structured_from_str(path, &content).ok()
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
    /// File creation timestamp (from filesystem).
    #[rkyv(with = Map<AsUnixTime>)]
    created_at: Option<SystemTime>,
    /// File modification timestamp (from filesystem).
    #[rkyv(with = Map<AsUnixTime>)]
    modified_at: Option<SystemTime>,
    /// When this version was recorded in the database.
    #[rkyv(with = AsUnixTime)]
    recorded_at: SystemTime,
    /// Compressed original file content (zstd level 3) - enables exact
    /// reconstruction.
    ///
    /// Stored as `Option<Vec<u8>>` to support legacy versions without content,
    /// but all new versions should include this for cache reconstruction.
    compressed_content: Option<Vec<u8>>,
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
        created_at: Option<SystemTime>,
        modified_at: Option<SystemTime>,
        compressed_content: Option<Vec<u8>>,
    ) -> Self {
        let recorded_at = SystemTime::now();

        Self {
            content_hash,
            property_hashes,
            created_at,
            modified_at,
            recorded_at,
            compressed_content,
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
    pub(crate) fn compress_content(content: &str) -> std::io::Result<Vec<u8>> {
        const COMPRESSION_LEVEL: i32 = 3;
        zstd::encode_all(content.as_bytes(), COMPRESSION_LEVEL)
    }
}

// ============================================================================
// TryFrom implementations for convenient conversion
// ============================================================================

impl TryFrom<&super::super::raw::RawPropertyBank> for RawPropertyBankView {
    type Error = crate::schema::error::SchemaIngestionError;

    /// Convert from a raw property bank, using its metadata for the view.
    #[inline]
    fn try_from(
        raw: &super::super::raw::RawPropertyBank,
    ) -> Result<Self, Self::Error> {
        let content_hash = raw.metadata.content_hash.ok_or_else(|| {
            crate::schema::error::SchemaIngestionError::Io {
                path: "property bank".into(),
                reason: "missing content hash".into(),
            }
        })?;

        let property_hashes: BTreeMap<PropertyName, [u8; 32]> = raw
            .metadata
            .property_hashes
            .iter()
            .filter_map(|(k, v)| {
                PropertyName::try_new(k.as_ref()).ok().map(|name| (name, *v))
            })
            .collect();

        Ok(Self::new(
            content_hash,
            property_hashes,
            raw.metadata.created_at,
            raw.metadata.modified_at,
            None, /* TryFrom doesn't have access to compressed content (only
                   * Ingestor does) */
        ))
    }
}

impl TryFrom<&super::super::raw::RawSchema> for RawSchemaView {
    type Error = crate::schema::error::SchemaIngestionError;

    /// Convert from a raw schema, using its metadata for the view.
    #[inline]
    fn try_from(
        raw: &super::super::raw::RawSchema,
    ) -> Result<Self, Self::Error> {
        let content_hash = raw.metadata.content_hash.ok_or_else(|| {
            crate::schema::error::SchemaIngestionError::Io {
                path: format!("schema {}", raw.name).into(),
                reason: "missing content hash".into(),
            }
        })?;

        let property_hashes: BTreeMap<PropertyName, [u8; 32]> = raw
            .metadata
            .property_hashes
            .iter()
            .filter_map(|(k, v)| {
                PropertyName::try_new(k.as_ref()).ok().map(|name| (name, *v))
            })
            .collect();

        let extends = raw
            .extends
            .as_ref()
            .and_then(|name| SchemaName::try_new(name.as_ref()).ok());

        let excludes: Vec<PropertyName> = raw
            .excludes
            .iter()
            .filter_map(|name| PropertyName::try_new(name.as_ref()).ok())
            .collect();

        Ok(Self::new(
            format!("schemas/{}.toml", raw.name).into_boxed_str(),
            extends,
            excludes,
            content_hash,
            property_hashes,
            raw.metadata.created_at,
            raw.metadata.modified_at,
            None, /* TryFrom doesn't have access to compressed content (only
                   * Ingestor does) */
        ))
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
            None,
            None,
            Some(compressed),
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
            None,
            None,
            Some(compressed),
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
            None,
            None,
            Some(invalid_data),
        );

        let result = version.decompress_content().expect("should have content");
        result.unwrap_err();
    }

    #[test]
    fn raw_schema_view_to_raw_returns_none_without_content() {
        let view = RawSchemaView::new(
            "schemas/test.toml".into(),
            None,
            vec![],
            [0; 32],
            BTreeMap::new(),
            None,
            None,
            None, // No compressed content
        );

        assert!(view.to_raw().is_none());
    }

    #[test]
    fn raw_property_bank_view_to_raw_returns_none_without_content() {
        let view = RawPropertyBankView::new(
            [0; 32],
            BTreeMap::new(),
            None,
            None,
            None, // No compressed content
        );

        let path = std::path::Path::new("schemas/property_bank.json");
        assert!(view.to_raw(path).is_none());
    }

    #[test]
    fn raw_file_version_stores_compressed_content() {
        let content = "Test content";
        let compressed = RawFileVersion::compress_content(content)
            .expect("compression failed");

        let version = RawFileVersion::new(
            [1; 32],
            BTreeMap::new(),
            None,
            None,
            Some(compressed.clone()),
        );

        // Verify the compressed content is stored
        let decompressed = version
            .decompress_content()
            .expect("should have content")
            .expect("decompression should succeed");

        assert_eq!(decompressed, content);
    }
}
