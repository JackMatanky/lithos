//! Raw file version views for schema persistence.
//!
//! These types track raw file content history with compression and hashing,
//! enabling staleness detection without full re-parsing.

use std::{
    collections::{BTreeMap, VecDeque},
    io::Read as _,
    time::SystemTime,
};

use rkyv::{
    Archive, Deserialize, Serialize,
    with::{AsUnixTime, Map},
};

use crate::schema::{aggregate::SchemaName, property::PropertyName};

/// Maximum number of versions to retain per file.
const MAX_VERSIONS: usize = 5;

/// Zstd compression level (3 = balanced speed/ratio).
const COMPRESSION_LEVEL: i32 = 3;

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
    pub fn new(
        file_path: Box<str>,
        extends: Option<SchemaName>,
        excludes: Vec<PropertyName>,
        content: &str,
        property_hashes: BTreeMap<PropertyName, [u8; 32]>,
        created_at: Option<SystemTime>,
        modified_at: Option<SystemTime>,
    ) -> Result<Self, std::io::Error> {
        let mut versions = VecDeque::with_capacity(MAX_VERSIONS);
        let version = RawFileVersion::new(
            content,
            property_hashes,
            created_at,
            modified_at,
        )?;
        versions.push_front(version);

        Ok(Self {
            file_path,
            extends,
            excludes,
            versions,
        })
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
    ///
    /// # Errors
    /// Returns error if compression fails.
    #[inline]
    pub fn add_version(
        &mut self,
        content: &str,
        property_hashes: BTreeMap<PropertyName, [u8; 32]>,
        created_at: Option<SystemTime>,
        modified_at: Option<SystemTime>,
    ) -> Result<(), std::io::Error> {
        let version = RawFileVersion::new(
            content,
            property_hashes,
            created_at,
            modified_at,
        )?;

        if self.versions.len() >= MAX_VERSIONS {
            self.versions.pop_back(); // Remove oldest
        }

        self.versions.push_front(version); // Add newest
        Ok(())
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
        self.current().is_some_and(|current| {
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
    pub fn filter_changed_properties(
        &self,
        metadata: &super::super::raw::RawSchemaMetadata,
    ) -> Vec<PropertyName> {
        self.current().map_or_else(Vec::new, |current| {
            let mut changed = Vec::new();

            // Check for added or modified properties
            for (name, new_hash) in &metadata.property_hashes {
                if let Ok(prop_name) = PropertyName::try_new(name.as_ref()) {
                    match current.property_hashes.get(&prop_name) {
                        Some(old_hash) if old_hash != new_hash => {
                            // Property modified
                            changed.push(prop_name);
                        }
                        None => {
                            // Property added
                            changed.push(prop_name);
                        }
                        _ => {} // Property unchanged
                    }
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
        })
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
/// let view = RawPropertyBankView::new(content, created_at, modified_at)?;
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
#[derive(Debug, Clone, PartialEq, Archive, Serialize, Deserialize)]
pub struct RawPropertyBankView {
    /// Version history (ring buffer, max 5 versions, newest first).
    versions: VecDeque<RawFileVersion>,
}

impl RawPropertyBankView {
    /// Creates a new property bank view with initial version.
    ///
    /// # Errors
    /// Returns error if compression fails.
    #[inline]
    pub fn new(
        content: &str,
        property_hashes: BTreeMap<PropertyName, [u8; 32]>,
        created_at: Option<SystemTime>,
        modified_at: Option<SystemTime>,
    ) -> Result<Self, std::io::Error> {
        let mut versions = VecDeque::with_capacity(MAX_VERSIONS);
        let version = RawFileVersion::new(
            content,
            property_hashes,
            created_at,
            modified_at,
        )?;
        versions.push_front(version);

        Ok(Self {
            versions,
        })
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
    ///
    /// # Errors
    /// Returns error if compression fails.
    #[inline]
    pub fn add_version(
        &mut self,
        content: &str,
        property_hashes: BTreeMap<PropertyName, [u8; 32]>,
        created_at: Option<SystemTime>,
        modified_at: Option<SystemTime>,
    ) -> Result<(), std::io::Error> {
        let version = RawFileVersion::new(
            content,
            property_hashes,
            created_at,
            modified_at,
        )?;

        if self.versions.len() >= MAX_VERSIONS {
            self.versions.pop_back(); // Remove oldest
        }

        self.versions.push_front(version); // Add newest
        Ok(())
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
        self.current().is_some_and(|current| {
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
    pub fn filter_changed_properties(
        &self,
        metadata: &super::super::raw::RawSchemaMetadata,
    ) -> Vec<PropertyName> {
        self.current().map_or_else(Vec::new, |current| {
            let mut changed = Vec::new();

            // Check for added or modified properties
            for (name, new_hash) in &metadata.property_hashes {
                if let Ok(prop_name) = PropertyName::try_new(name.as_ref()) {
                    match current.property_hashes.get(&prop_name) {
                        Some(old_hash) if old_hash != new_hash => {
                            // Property modified
                            changed.push(prop_name);
                        }
                        None => {
                            // Property added
                            changed.push(prop_name);
                        }
                        _ => {} // Property unchanged
                    }
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
        })
    }
}

/// A single version of a raw file with compressed content and metadata.
///
/// Uses zstd compression (level 3) and Blake3 hashing for content verification.
/// Timestamps are optional (files might not have filesystem metadata).
///
/// # Examples
///
/// ```ignore
/// use lithos_core::schema::views::RawFileVersion;
///
/// let version = RawFileVersion::new(
///     "name: note\nproperties: []",
///     Some(created_at),
///     Some(modified_at),
/// )?;
/// assert!(version.is_content_match("name: note\nproperties: []"));
/// # Ok::<(), std::io::Error>(())
/// ```
#[derive(Debug, Clone, PartialEq, Archive, Serialize, Deserialize)]
pub struct RawFileVersion {
    /// Compressed file content (zstd level 3).
    compressed_content: Vec<u8>,
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
}

impl RawFileVersion {
    /// Creates a new file version from content and metadata.
    ///
    /// # Parameters
    /// - `content`: Raw file content
    /// - `property_hashes`: Per-property hashes for incremental resolution
    ///   (computed by caller)
    /// - `created_at`: File creation timestamp
    /// - `modified_at`: File modification timestamp
    ///
    /// # Errors
    /// Returns error if compression fails.
    #[inline]
    pub fn new(
        content: &str,
        property_hashes: BTreeMap<PropertyName, [u8; 32]>,
        created_at: Option<SystemTime>,
        modified_at: Option<SystemTime>,
    ) -> Result<Self, std::io::Error> {
        let compressed_content = Self::compress(content)?;
        let content_hash = *blake3::hash(content.as_bytes()).as_bytes();
        let recorded_at = SystemTime::now();

        Ok(Self {
            compressed_content,
            content_hash,
            property_hashes,
            created_at,
            modified_at,
            recorded_at,
        })
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

    /// Decompresses and returns file content.
    ///
    /// # Errors
    /// Returns error if decompression fails.
    #[inline]
    pub fn content(&self) -> Result<String, DecompressionError> {
        Self::decompress(&self.compressed_content)
    }

    /// Returns compressed size in bytes.
    #[inline]
    #[must_use]
    pub fn compressed_size(&self) -> usize {
        self.compressed_content.len()
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
    /// Returns `true` if the Blake3 hash of the provided content matches
    /// the stored hash. This is slower but handles timestamp edge cases.
    #[inline]
    #[must_use]
    pub fn is_content_match(&self, content: &str) -> bool {
        let hash = blake3::hash(content.as_bytes());
        hash.as_bytes() == &self.content_hash
    }

    /// Compresses string content using zstd.
    ///
    /// # Errors
    /// Returns error if compression fails.
    #[inline]
    fn compress(content: &str) -> Result<Vec<u8>, std::io::Error> {
        zstd::encode_all(content.as_bytes(), COMPRESSION_LEVEL)
    }

    /// Decompresses zstd data to string.
    ///
    /// # Errors
    /// Returns error if decompression fails or output is not UTF-8.
    #[inline]
    fn decompress(compressed: &[u8]) -> Result<String, DecompressionError> {
        let mut decompressed = Vec::new();
        zstd::Decoder::new(compressed)?.read_to_end(&mut decompressed)?;

        String::from_utf8(decompressed)
            .map_err(|e| DecompressionError::Utf8(e.utf8_error()))
    }
}

/// Decompression error.
#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub enum DecompressionError {
    /// I/O error during decompression.
    #[error("decompression I/O error: {0}")]
    Io(#[from] std::io::Error),

    /// Decompressed data is not valid UTF-8.
    #[error("decompressed data is not valid UTF-8: {0}")]
    Utf8(#[from] std::str::Utf8Error),
}
