//! Version types for schema and property bank views.
//!
//! These types store validated, typed data extracted from Raw* types,
//! avoiding the need to add rkyv serialization to the raw parsing layer.
//!
//! ## Hybrid Serialization Strategy
//!
//! These types use a hybrid approach to avoid adding rkyv derives to Raw*
//! types:
//! - Metadata fields (`extends`, `excludes`, `version`) are stored as validated
//!   types
//! - Complex property trees remain in the Raw* layer
//! - Version views store only validated metadata needed for staleness checks
//!
//! This keeps the parsing layer (Raw* types with serde) separate from the
//! storage layer (version types with rkyv), while maintaining queryability of
//! key metadata fields.

#![expect(
    clippy::same_name_method,
    reason = "Trait contracts intentionally mirror established version API"
)]
//!
//! This keeps the parsing layer (Raw* types with serde) separate from the
//! storage layer (version types with rkyv), while maintaining queryability of
//! key metadata fields.

use std::collections::{HashMap, HashSet};

use rkyv::{Archive, Deserialize, Serialize};

use super::metadata::{FileTimesMetadata, HashMetadata};
use crate::{
    schema::{
        aggregate::SchemaName,
        error::SchemaIngestionError,
        property::{PropertyMap, PropertyName},
        raw::RawSchema,
    },
    support::hash::Blake3Hash,
};

/// Read-only access contract shared by version payloads.
pub trait VersionRead {
    /// Checks whether the content hash matches this version.
    fn is_content_match(&self, hash: &Blake3Hash) -> bool;

    /// Checks whether filesystem timestamps match this version's metadata.
    fn is_timestamp_match(
        &self,
        created_at: Option<std::time::SystemTime>,
        modified_at: Option<std::time::SystemTime>,
    ) -> bool;

    /// Returns the format version string.
    fn version(&self) -> &str;
}

/// Mutable version contract for persisted raw views.
pub trait Version: VersionRead + Sized {
    /// Returns file timestamp metadata.
    fn file_times(&self) -> &FileTimesMetadata;

    /// Returns hash metadata.
    fn hashes(&self) -> &HashMetadata;

    /// Updates file timestamp metadata in-place.
    fn set_file_times(&mut self, file_times: FileTimesMetadata);

    /// Clones this version with replacement metadata.
    #[must_use]
    fn with_metadata(
        &self,
        file_times: FileTimesMetadata,
        hashes: HashMetadata,
    ) -> Self;
}

// ─────────────────────────────────────────────────────────────────────────────
//  SchemaVersion
// ─────────────────────────────────────────────────────────────────────────────

/// A single version of a schema file with validated, typed data.
///
/// Stores:
/// - File and hash metadata for staleness detection
/// - Schema format version, inheritance metadata (validated, queryable)
/// - Cached expanded properties for incremental resolution
///
/// ## Design Rationale
///
/// This hybrid approach keeps metadata fields (`extends`, `excludes`) as
/// validated types for direct querying while leaving the raw property tree in
/// the Raw* parsing layer. This avoids adding rkyv derives to the raw schema
/// parsing types while maintaining queryability of inheritance metadata.
#[derive(Debug, Clone, PartialEq, Archive, Serialize, Deserialize)]
pub struct SchemaVersion {
    /// File timestamp metadata.
    file_times: FileTimesMetadata,

    /// Hash metadata for staleness detection.
    hashes: HashMetadata,

    /// Schema format version as simple string (e.g., "1.0").
    ///
    /// Stored as `Box<str>` instead of `RawSchemaVersion` to avoid requiring
    /// rkyv derives on Raw* types.
    version: Box<str>,

    /// Parent schema name (from `extends` field).
    ///
    /// Validated and stored as typed field for efficient querying.
    extends: Option<SchemaName>,

    /// Property names to exclude from parent (from `excludes` field).
    ///
    /// Validated and stored as typed field for efficient querying.
    excludes: Vec<PropertyName>,

    /// Map of schema property name to property bank target name.
    ///
    /// Extracted from `$ref` entries during ingestion.
    bank_references: HashMap<PropertyName, PropertyName>,

    /// Cached expanded properties (from `RefExpander`).
    ///
    /// Enables skipping expansion when `PropertyBank` is fresh.
    expanded_properties: Option<PropertyMap>,
}

impl SchemaVersion {
    /// Create a new schema version from a `RawSchema`.
    ///
    /// # Errors
    /// This constructor is currently infallible; the `Result` is retained for
    /// pipeline compatibility if future validation is added.
    #[inline]
    pub fn new(
        file_times: FileTimesMetadata,
        hashes: HashMetadata,
        raw: &RawSchema,
    ) -> Result<Self, SchemaIngestionError> {
        // extends and excludes are already validated during deserialization
        // (custom Deserialize impls ensure type safety)
        let extends = raw.extends().cloned();
        let excludes = raw.excludes().to_vec();

        // Property names are already validated via RawPropertyMap
        // deserialization No need to validate them again here

        // Extract bank references from properties map
        let mut bank_references = HashMap::new();
        #[expect(
            clippy::iter_over_hash_type,
            reason = "Ordering is irrelevant for bank reference extraction"
        )]
        for (prop_name, ref_entry) in raw.properties().ref_entries() {
            bank_references
                .insert(prop_name, ref_entry.ref_path.target_name().clone());
        }

        Ok(Self {
            file_times,
            hashes,
            version: raw.version().as_str().into(),
            extends,
            excludes,
            bank_references,
            expanded_properties: None,
        })
    }

    /// Get file times metadata.
    #[inline]
    #[must_use]
    pub fn file_times(&self) -> &FileTimesMetadata {
        &self.file_times
    }

    /// Get hash metadata.
    #[inline]
    #[must_use]
    pub fn hashes(&self) -> &HashMetadata {
        &self.hashes
    }

    /// Get version string.
    #[inline]
    #[must_use]
    pub fn version(&self) -> &str {
        &self.version
    }

    /// Get parent schema name.
    #[inline]
    #[must_use]
    pub fn extends(&self) -> Option<&SchemaName> {
        self.extends.as_ref()
    }

    /// Get excluded property names.
    #[inline]
    #[must_use]
    pub fn excludes(&self) -> &[PropertyName] {
        &self.excludes
    }

    /// Get bank property references.
    ///
    /// Returns a map of schema property name to target bank property name.
    #[inline]
    #[must_use]
    pub fn bank_references(&self) -> &HashMap<PropertyName, PropertyName> {
        &self.bank_references
    }

    /// Compute schema properties affected by property bank changes.
    ///
    /// Returns property names that refer to any of the changed property names
    /// in the provided `bank_delta`.
    #[inline]
    #[must_use]
    pub fn changed_bank_references(
        &self,
        bank_delta: &HashSet<PropertyName>,
    ) -> Vec<PropertyName> {
        let mut changed = Vec::new();

        #[expect(
            clippy::iter_over_hash_type,
            reason = "Ordering is irrelevant for detecting affected references"
        )]
        for (prop_name, bank_name) in &self.bank_references {
            if bank_delta.contains(bank_name) {
                changed.push(prop_name.clone());
            }
        }

        changed
    }

    /// Get cached expanded properties if available.
    #[inline]
    #[must_use]
    pub fn expanded_properties(&self) -> Option<&PropertyMap> {
        self.expanded_properties.as_ref()
    }

    /// Set cached expanded properties.
    ///
    /// Called after `RefExpander` processes the schema.
    #[inline]
    pub fn set_expanded_properties(&mut self, properties: PropertyMap) {
        self.expanded_properties = Some(properties);
    }

    /// Clone the version with updated file times and hashes.
    ///
    /// Resets cached expanded properties to keep refresh behavior consistent
    /// with raw-based reconstruction.
    #[inline]
    #[must_use]
    pub fn with_metadata(
        &self,
        file_times: FileTimesMetadata,
        hashes: HashMetadata,
    ) -> Self {
        Self {
            file_times,
            hashes,
            version: self.version.clone(),
            extends: self.extends.clone(),
            excludes: self.excludes.clone(),
            bank_references: self.bank_references.clone(),
            expanded_properties: None,
        }
    }
}

impl VersionRead for SchemaVersion {
    #[inline]
    fn is_timestamp_match(
        &self,
        created_at: Option<std::time::SystemTime>,
        modified_at: Option<std::time::SystemTime>,
    ) -> bool {
        self.file_times().is_timestamp_match(created_at, modified_at)
    }

    #[inline]
    fn is_content_match(&self, hash: &Blake3Hash) -> bool {
        self.hashes().is_content_match(hash)
    }

    #[inline]
    fn version(&self) -> &str {
        self.version.as_ref()
    }
}

impl Version for SchemaVersion {
    #[inline]
    fn file_times(&self) -> &FileTimesMetadata {
        self.file_times()
    }

    #[inline]
    fn hashes(&self) -> &HashMetadata {
        self.hashes()
    }

    #[inline]
    fn set_file_times(&mut self, file_times: FileTimesMetadata) {
        self.file_times = file_times;
    }

    #[inline]
    fn with_metadata(
        &self,
        file_times: FileTimesMetadata,
        hashes: HashMetadata,
    ) -> Self {
        SchemaVersion::with_metadata(self, file_times, hashes)
    }
}

// ─────────────────────────────────────────────────────────────────────────────
//  PropertyBankVersion
// ─────────────────────────────────────────────────────────────────────────────

/// A single version of the property bank file with validated, typed data.
///
/// Stores:
/// - File and hash metadata for staleness detection
/// - Property bank format version as simple string
///
/// ## Design Rationale
///
/// Similar to `SchemaVersion`, this uses a hybrid approach: metadata fields are
/// stored as validated types, while the complex property tree remains in the
/// Raw* parsing layer to avoid adding rkyv derives.
#[derive(Debug, Clone, PartialEq, Archive, Serialize, Deserialize)]
pub struct PropertyBankVersion {
    /// File timestamp metadata.
    file_times: FileTimesMetadata,

    /// Hash metadata for staleness detection.
    hashes: HashMetadata,

    /// Property bank format version as simple string (e.g., "1.0").
    ///
    /// Stored as `Box<str>` instead of `RawSchemaVersion` to avoid requiring
    /// rkyv derives on Raw* types.
    version: Box<str>,
}

impl PropertyBankVersion {
    /// Create a new property bank version from a version string.
    ///
    /// # Errors
    /// This constructor is currently infallible; the `Result` is retained for
    /// pipeline compatibility if future validation is added.
    #[inline]
    pub fn new(
        file_times: FileTimesMetadata,
        hashes: HashMetadata,
        version: &str,
    ) -> Result<Self, SchemaIngestionError> {
        Ok(Self {
            file_times,
            hashes,
            version: version.into(),
        })
    }

    /// Get file times metadata.
    #[inline]
    #[must_use]
    pub fn file_times(&self) -> &FileTimesMetadata {
        &self.file_times
    }

    /// Set file times metadata.
    #[inline]
    pub fn set_file_times(&mut self, file_times: FileTimesMetadata) {
        self.file_times = file_times;
    }

    /// Get hash metadata.
    #[inline]
    #[must_use]
    pub fn hashes(&self) -> &HashMetadata {
        &self.hashes
    }

    /// Get version string.
    #[inline]
    #[must_use]
    pub fn version(&self) -> &str {
        &self.version
    }
}

impl VersionRead for PropertyBankVersion {
    #[inline]
    fn is_timestamp_match(
        &self,
        created_at: Option<std::time::SystemTime>,
        modified_at: Option<std::time::SystemTime>,
    ) -> bool {
        self.file_times().is_timestamp_match(created_at, modified_at)
    }

    #[inline]
    fn is_content_match(&self, hash: &Blake3Hash) -> bool {
        self.hashes().is_content_match(hash)
    }

    #[inline]
    fn version(&self) -> &str {
        self.version.as_ref()
    }
}

impl VersionRead for ArchivedSchemaVersion {
    #[inline]
    fn is_content_match(&self, hash: &Blake3Hash) -> bool {
        self.hashes.is_content_match(hash)
    }

    #[inline]
    fn is_timestamp_match(
        &self,
        created_at: Option<std::time::SystemTime>,
        modified_at: Option<std::time::SystemTime>,
    ) -> bool {
        self.file_times.is_timestamp_match(created_at, modified_at)
    }

    #[inline]
    fn version(&self) -> &str {
        self.version.as_ref()
    }
}

impl VersionRead for ArchivedPropertyBankVersion {
    #[inline]
    fn is_content_match(&self, hash: &Blake3Hash) -> bool {
        self.hashes.is_content_match(hash)
    }

    #[inline]
    fn is_timestamp_match(
        &self,
        created_at: Option<std::time::SystemTime>,
        modified_at: Option<std::time::SystemTime>,
    ) -> bool {
        self.file_times.is_timestamp_match(created_at, modified_at)
    }

    #[inline]
    fn version(&self) -> &str {
        self.version.as_ref()
    }
}

impl Version for PropertyBankVersion {
    #[inline]
    fn file_times(&self) -> &FileTimesMetadata {
        self.file_times()
    }

    #[inline]
    fn hashes(&self) -> &HashMetadata {
        self.hashes()
    }

    #[inline]
    fn set_file_times(&mut self, file_times: FileTimesMetadata) {
        self.file_times = file_times;
    }

    #[inline]
    fn with_metadata(
        &self,
        file_times: FileTimesMetadata,
        hashes: HashMetadata,
    ) -> Self {
        Self {
            file_times,
            hashes,
            version: self.version.clone(),
        }
    }
}

#[cfg(test)]
mod tests {
    use std::{collections::HashMap, time::SystemTime};

    use super::*;
    use crate::schema::property::PropertyMap;

    fn create_test_raw_schema() -> RawSchema {
        let json = serde_json::json!({
            "$version": "1.0",
            "properties": {}
        });
        serde_json::from_value::<RawSchema>(json)
            .expect("valid test fixture")
            .with_name("test".into())
    }

    #[test]
    fn schema_version_expanded_properties() {
        let raw = create_test_raw_schema();
        let file_times = FileTimesMetadata::new(
            Some(SystemTime::now()),
            Some(SystemTime::now()),
        );
        let hashes =
            HashMetadata::new(Blake3Hash::new([1u8; 32]), HashMap::default());

        let mut version = SchemaVersion::new(file_times, hashes, &raw).unwrap();

        assert!(version.expanded_properties().is_none());

        let expanded = PropertyMap::new();
        version.set_expanded_properties(expanded);

        assert!(version.expanded_properties().is_some());
    }

    #[test]
    fn schema_version_bank_references() {
        let json = serde_json::json!({
            "$version": "1.0",
            "properties": {
                "inline": { "type": "bool" },
                "referenced": { "$ref": "#property_bank/status" }
            }
        });
        let raw: RawSchema = serde_json::from_value(json).unwrap();

        let file_times = FileTimesMetadata::new(None, None);
        let hashes =
            HashMetadata::new(Blake3Hash::new([0; 32]), HashMap::new());
        let version = SchemaVersion::new(file_times, hashes, &raw).unwrap();

        let refs = version.bank_references();
        assert_eq!(refs.len(), 1);
        let status_name = PropertyName::try_new("status").unwrap();
        let ref_name = PropertyName::try_new("referenced").unwrap();
        assert_eq!(refs.get(&ref_name), Some(&status_name));
    }

    #[test]
    fn schema_version_detects_changed_bank_references() {
        let json = serde_json::json!({
            "$version": "1.0",
            "properties": {
                "title": { "$ref": "#property_bank/title_bank" },
                "tags": { "$ref": "#property_bank/tags_bank" },
                "inline": { "type": "bool" }
            }
        });
        let raw: RawSchema = serde_json::from_value(json).unwrap();
        let version = SchemaVersion::new(
            FileTimesMetadata::new(None, None),
            HashMetadata::new(Blake3Hash::new([0; 32]), HashMap::new()),
            &raw,
        )
        .unwrap();

        let mut delta = HashSet::new();
        delta.insert(PropertyName::try_new("title_bank").unwrap());

        let changed_single = version.changed_bank_references(&delta);
        assert_eq!(changed_single.len(), 1);
        assert_eq!(
            changed_single.first().map(PropertyName::as_str),
            Some("title")
        );

        // Multiple changes
        delta.insert(PropertyName::try_new("tags_bank").unwrap());
        let changed_multiple = version.changed_bank_references(&delta);
        assert_eq!(changed_multiple.len(), 2);
        let names: HashSet<_> =
            changed_multiple.iter().map(PropertyName::as_str).collect();
        assert!(names.contains("title"));
        assert!(names.contains("tags"));
    }

    #[test]
    fn schema_version_set_file_times_updates_metadata() {
        let raw = create_test_raw_schema();
        let initial = FileTimesMetadata::new(None, None);
        let hashes =
            HashMetadata::new(Blake3Hash::new([0; 32]), HashMap::new());
        let mut version = SchemaVersion::new(initial, hashes, &raw).unwrap();

        let updated = FileTimesMetadata::new(Some(SystemTime::now()), None);
        Version::set_file_times(&mut version, updated.clone());

        assert_eq!(version.file_times(), &updated);
    }

    #[test]
    fn property_bank_version_with_metadata_replaces_hash_and_timestamps() {
        let original = PropertyBankVersion::new(
            FileTimesMetadata::new(None, None),
            HashMetadata::new(Blake3Hash::new([1; 32]), HashMap::new()),
            "1.0",
        )
        .unwrap();

        let replacement = Version::with_metadata(
            &original,
            FileTimesMetadata::new(Some(SystemTime::now()), None),
            HashMetadata::new(Blake3Hash::new([2; 32]), HashMap::new()),
        );

        assert_eq!(replacement.hashes().content(), &Blake3Hash::new([2; 32]));
        assert_eq!(replacement.version(), "1.0");
    }

    #[test]
    fn archived_schema_version_supports_zero_copy_version_read() {
        let raw = create_test_raw_schema();
        let version = SchemaVersion::new(
            FileTimesMetadata::new(None, None),
            HashMetadata::new(Blake3Hash::new([7; 32]), HashMap::new()),
            &raw,
        )
        .expect("schema version should build");

        let bytes = rkyv::to_bytes::<rkyv::rancor::Error>(&version)
            .expect("serialize schema version");
        let archived = rkyv::access::<
            rkyv::Archived<SchemaVersion>,
            rkyv::rancor::Error,
        >(&bytes)
        .expect("access archived schema version");

        assert!(archived.is_content_match(&Blake3Hash::new([7; 32])));
        assert_eq!(archived.version(), "1.0");
    }

    #[test]
    fn archived_property_bank_version_supports_zero_copy_version_read() {
        let version = PropertyBankVersion::new(
            FileTimesMetadata::new(None, None),
            HashMetadata::new(Blake3Hash::new([3; 32]), HashMap::new()),
            "1.0",
        )
        .expect("property bank version should build");

        let bytes = rkyv::to_bytes::<rkyv::rancor::Error>(&version)
            .expect("serialize property bank version");
        let archived = rkyv::access::<
            rkyv::Archived<PropertyBankVersion>,
            rkyv::rancor::Error,
        >(&bytes)
        .expect("access archived property bank version");

        assert!(archived.is_content_match(&Blake3Hash::new([3; 32])));
        assert_eq!(archived.version(), "1.0");
    }

    // Note: Property name validation now happens during RawPropertyMap
    // deserialization Invalid property names cannot be constructed, making
    // this test obsolete
}
