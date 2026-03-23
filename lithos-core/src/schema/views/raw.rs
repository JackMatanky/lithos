//! Raw schema and property bank views for persistence.
//!
//! These types track version history for schemas and property banks,
//! enabling staleness detection and incremental updates.

use std::{collections::VecDeque, path::Path};

use rkyv::{Archive, Deserialize, Serialize};

use super::{PropertyBankVersion, SchemaVersion};
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
/// use lithos_core::schema::views::{RawSchemaView, Filename, SchemaVersion};
///
/// let filename = Filename::new("note.toml".into());
/// let version = SchemaVersion::new(/* ... */)?;
/// let view = RawSchemaView::new(filename, version);
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
#[derive(Debug, Clone, PartialEq, Archive, Serialize, Deserialize)]
pub struct RawSchemaView {
    /// Filename with extension (e.g., "note.toml").
    filename: Filename,

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
    pub fn new(filename: Filename, version: SchemaVersion) -> Self {
        let mut versions = VecDeque::with_capacity(MAX_VERSIONS);
        versions.push_front(version);

        Self {
            filename,
            versions,
        }
    }

    /// Returns the filename.
    #[inline]
    #[must_use]
    pub fn file_path(&self) -> &Filename {
        &self.filename
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
        self.filename.basename()
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

        // Deserialize full RawSchema from stored JSON
        // raw_properties contains the complete RawSchema JSON (not just
        // properties)
        let raw_schema =
            serde_json::from_slice::<RawSchema>(version.raw_properties())
                .map_err(|e| {
                    crate::schema::error::SchemaIngestionError::Parse(
                        crate::schema::error::SchemaParseError::CachedView {
                            path: std::path::PathBuf::from(self.name()),
                            reason: format!(
                                "failed to reconstruct schema: {e}"
                            )
                            .into(),
                        },
                    )
                })?
                .with_name(self.name().into());

        Ok(Some(raw_schema))
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
        filename: &str,
        content: &str,
    ) -> Result<Self, crate::schema::error::SchemaIngestionError> {
        use super::{FileTimesMetadata, HashMetadata};

        // Compute content hash from raw file content (truncated to 128 bits)
        let content_hash = blake3::hash(content.as_bytes());

        // Compute per-property hashes
        let property_hashes =
            HashMetadata::compute_property_hashes(raw.properties());

        let file_times = FileTimesMetadata::new(
            raw.file_times().created_at,
            raw.file_times().modified_at,
        );
        let hashes =
            HashMetadata::new(*content_hash.as_bytes(), property_hashes);

        let version = SchemaVersion::new(file_times, hashes, raw)?;

        Ok(Self::new(Filename::new(filename.into()), version))
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

        // Deserialize full RawPropertyBank from stored JSON
        // raw_properties contains the complete RawPropertyBank JSON (not just
        // properties)
        let raw_bank: RawPropertyBank =
            serde_json::from_slice::<RawPropertyBank>(version.raw_properties())
                .map_err(|e| {
                    crate::schema::error::SchemaIngestionError::Parse(
                        crate::schema::error::SchemaParseError::CachedView {
                            path: std::path::PathBuf::from("property_bank"),
                            reason: format!(
                                "failed to reconstruct property bank: {e}"
                            )
                            .into(),
                        },
                    )
                })?;

        Ok(Some(raw_bank))
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
    pub fn try_from_raw_with_content(
        raw: &RawPropertyBank,
        content: &str,
    ) -> Result<Self, crate::schema::error::SchemaIngestionError> {
        use super::{FileTimesMetadata, HashMetadata};

        // Compute content hash from raw file content (truncated to 128 bits)
        let content_hash = blake3::hash(content.as_bytes());

        // Compute per-property hashes
        let property_hashes =
            HashMetadata::compute_property_hashes(raw.properties());

        let file_times = FileTimesMetadata::new(
            raw.file_times().created_at,
            raw.file_times().modified_at,
        );
        let hashes =
            HashMetadata::new(*content_hash.as_bytes(), property_hashes);

        let version = PropertyBankVersion::new(file_times, hashes, raw)?;

        Ok(Self::new(version))
    }
}

/// Filename for schema/property bank files with extension.
///
/// Stores only the filename (e.g., "note.toml"). The schema directory
/// is always determined by configuration and is assumed to be flat.
/// Provides methods to extract stem and extension without repeatedly
/// parsing the filename.
#[derive(
    Debug, Clone, PartialEq, Eq, Hash, Archive, Serialize, Deserialize,
)]
pub struct Filename(Box<str>);

impl Filename {
    /// Create a new filename.
    #[inline]
    #[must_use]
    pub fn new(filename: Box<str>) -> Self {
        Self(filename)
    }

    /// Get the filename as a string.
    #[inline]
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }

    /// Get the basename (filename without extension).
    ///
    /// Uses Obsidian terminology where "basename" means filename without
    /// extension.
    ///
    /// # Examples
    /// ```ignore
    /// use lithos_core::schema::views::Filename;
    ///
    /// let filename = Filename::new("note.toml".into());
    /// assert_eq!(filename.basename(), "note");
    /// ```
    #[inline]
    #[must_use]
    pub fn basename(&self) -> &str {
        Path::new(self.as_str())
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("")
    }

    /// Get the file extension.
    ///
    /// # Examples
    /// ```ignore
    /// use lithos_core::schema::views::Filename;
    ///
    /// let filename = Filename::new("note.toml".into());
    /// assert_eq!(filename.extension(), Some("toml"));
    /// ```
    #[inline]
    #[must_use]
    pub fn extension(&self) -> Option<&str> {
        Path::new(self.as_str()).extension().and_then(|s| s.to_str())
    }

    /// Get the underlying filename as a `Path`.
    #[inline]
    #[must_use]
    pub fn as_path(&self) -> &Path {
        Path::new(self.as_str())
    }
}

impl From<Box<str>> for Filename {
    #[inline]
    fn from(filename: Box<str>) -> Self {
        Self::new(filename)
    }
}

impl From<String> for Filename {
    #[inline]
    fn from(filename: String) -> Self {
        Self::new(filename.into_boxed_str())
    }
}

impl AsRef<str> for Filename {
    #[inline]
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}

impl AsRef<Path> for Filename {
    #[inline]
    fn as_ref(&self) -> &Path {
        self.as_path()
    }
}

#[cfg(test)]
mod tests {
    mod filename {
        use super::super::Filename;

        #[test]
        fn basename_extracts_filename_without_extension() {
            let filename = Filename::new("note.toml".into());
            assert_eq!(filename.basename(), "note");
        }

        #[test]
        fn basename_handles_hyphens() {
            let filename = Filename::new("base-note.toml".into());
            assert_eq!(filename.basename(), "base-note");
        }

        #[test]
        fn extension_returns_file_extension() {
            let filename = Filename::new("note.toml".into());
            assert_eq!(filename.extension(), Some("toml"));
        }

        #[test]
        fn extension_returns_none_for_no_extension() {
            let filename = Filename::new("note".into());
            assert_eq!(filename.extension(), None);
        }

        #[test]
        fn as_str_returns_full_filename() {
            let filename = Filename::new("note.toml".into());
            assert_eq!(filename.as_str(), "note.toml");
        }
    }

    mod schema {
        use std::collections::HashMap;

        use super::super::{Filename, RawSchemaView, SchemaVersion};
        use crate::schema::{
            raw::RawSchema,
            views::{FileTimesMetadata, HashMetadata},
        };

        #[test]
        fn to_raw_reconstructs_schema() {
            // Create a test RawSchema via deserialization
            let json = r#"{
                "$version": "1.0",
                "properties": {}
            }"#;
            let raw = serde_json::from_str::<RawSchema>(json)
                .expect("valid schema should deserialize")
                .with_name("test".into());

            let file_times = FileTimesMetadata::new(None, None);
            let hashes = HashMetadata::new([0; 32], HashMap::new());
            let version = SchemaVersion::new(file_times, hashes, &raw).unwrap();

            let filename = Filename::new("test.toml".into());
            let view = RawSchemaView::new(filename, version);

            let reconstructed = view
                .to_raw()
                .expect("should succeed")
                .expect("should have value");
            assert_eq!(reconstructed.name(), "test");
            assert_eq!(reconstructed.properties().len(), 0);
        }
    }

    mod property_bank {
        use std::collections::HashMap;

        use super::super::{PropertyBankVersion, RawPropertyBankView};
        use crate::schema::{
            raw::RawPropertyBank,
            views::{FileTimesMetadata, HashMetadata},
        };

        #[test]
        fn to_raw_reconstructs_property_bank() {
            // Create a test RawPropertyBank via deserialization
            let json = r#"{
                "$version": "1.0",
                "properties": {}
            }"#;
            let raw: RawPropertyBank =
                serde_json::from_str::<RawPropertyBank>(json)
                    .expect("valid property bank should deserialize");

            let file_times = FileTimesMetadata::new(None, None);
            let hashes = HashMetadata::new([0; 32], HashMap::new());
            let version =
                PropertyBankVersion::new(file_times, hashes, &raw).unwrap();

            let view = RawPropertyBankView::new(version);

            let reconstructed = view
                .to_raw()
                .expect("should succeed")
                .expect("should have value");
            assert_eq!(reconstructed.properties().len(), 0);
        }
    }
}
