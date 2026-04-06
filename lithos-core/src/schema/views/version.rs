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

use std::collections::HashMap;

use rkyv::{Archive, Deserialize, Serialize};

use super::metadata::{FileTimesMetadata, HashMetadata};
use crate::schema::{
    aggregate::SchemaName,
    error::SchemaIngestionError,
    property::{PropertyMap, PropertyName},
    raw::RawSchema,
};

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
        let hashes = HashMetadata::new([1u8; 32], HashMap::default());

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
                "referenced": { "$ref": "property_bank#/status" }
            }
        });
        let raw: RawSchema = serde_json::from_value(json).unwrap();

        let file_times = FileTimesMetadata::new(None, None);
        let hashes = HashMetadata::new([0; 32], HashMap::new());
        let version = SchemaVersion::new(file_times, hashes, &raw).unwrap();

        let refs = version.bank_references();
        assert_eq!(refs.len(), 1);
        let status_name = PropertyName::try_new("status").unwrap();
        let ref_name = PropertyName::try_new("referenced").unwrap();
        assert_eq!(refs.get(&ref_name), Some(&status_name));
    }

    // Note: Property name validation now happens during RawPropertyMap
    // deserialization Invalid property names cannot be constructed, making
    // this test obsolete
}
