//! Version types for schema and property bank views.
//!
//! These types store validated, typed data extracted from Raw* types,
//! avoiding the need to add rkyv serialization to the raw parsing layer.

use std::collections::HashMap;

use rkyv::{Archive, Deserialize, Serialize};

use super::metadata::{FileTimesMetadata, HashMetadata};
use crate::schema::{
    aggregate::SchemaName,
    error::SchemaIngestionError,
    property::{Property, PropertyName},
    raw::{
        RawPropertyBank, RawSchema, RawSchemaMetadata, RawSchemaVersion,
        property::{RawProperty, RawPropertyBankEntry},
    },
};

// ─────────────────────────────────────────────────────────────────────────────
//  SchemaVersion
// ─────────────────────────────────────────────────────────────────────────────

/// A single version of a schema file with validated, typed data.
///
/// Stores:
/// - File and hash metadata for staleness detection
/// - Schema format version, inheritance, and properties (validated)
/// - Cached expanded properties for incremental resolution
///
/// This type stores data in a form optimized for persistence and querying,
/// using validated types (`PropertyName`) instead of raw strings.
#[derive(Debug, Clone, PartialEq, Archive, Serialize, Deserialize)]
pub struct SchemaVersion {
    /// File timestamp metadata.
    file_times: FileTimesMetadata,

    /// Hash metadata for staleness detection.
    hashes: HashMetadata,

    /// Schema format version (e.g., "1.0").
    version: RawSchemaVersion,

    /// Parent schema name (from `extends` field).
    extends: Option<SchemaName>,

    /// Property names to exclude from parent (from `excludes` field).
    excludes: Vec<PropertyName>,

    /// Map of validated property name to property definition.
    ///
    /// Keys are `PropertyName` (validated) instead of `Box<str>` for type
    /// safety.
    properties: HashMap<PropertyName, RawProperty>,

    /// Cached expanded properties (from `RefExpander`).
    ///
    /// Enables skipping expansion when `PropertyBank` is fresh.
    expanded_properties: Option<HashMap<PropertyName, Property>>,
}

impl SchemaVersion {
    /// Create a new schema version from a `RawSchema`.
    ///
    /// # Errors
    /// Returns error if property name validation fails.
    #[inline]
    #[expect(
        clippy::iter_over_hash_type,
        reason = "Consuming iteration - order doesn't matter for validation"
    )]
    pub fn new(
        file_times: FileTimesMetadata,
        hashes: HashMetadata,
        raw: RawSchema,
    ) -> Result<Self, SchemaIngestionError> {
        // Validate and convert property keys from Box<str> to PropertyName
        let mut properties = HashMap::with_capacity(raw.properties.len());
        for (name, prop) in raw.properties {
            let prop_name = PropertyName::try_new(&name).map_err(|e| {
                SchemaIngestionError::Io {
                    path: raw.name.clone(),
                    reason: format!("invalid property name '{name}': {e}")
                        .into(),
                }
            })?;
            properties.insert(prop_name, prop);
        }

        // Validate and convert excludes
        let mut excludes = Vec::with_capacity(raw.excludes.len());
        for exclude in raw.excludes {
            let prop_name = PropertyName::try_new(&exclude).map_err(|e| {
                SchemaIngestionError::Io {
                    path: raw.name.clone(),
                    reason: format!("invalid exclude name '{exclude}': {e}")
                        .into(),
                }
            })?;
            excludes.push(prop_name);
        }

        // Validate extends if present
        let extends = raw
            .extends
            .map(|name| {
                SchemaName::try_new(&name).map_err(|e| {
                    SchemaIngestionError::Io {
                        path: raw.name.clone(),
                        reason: format!("invalid extends '{name}': {e}").into(),
                    }
                })
            })
            .transpose()?;

        Ok(Self {
            file_times,
            hashes,
            version: raw.version,
            extends,
            excludes,
            properties,
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

    /// Get properties.
    #[inline]
    #[must_use]
    pub fn properties(&self) -> &HashMap<PropertyName, RawProperty> {
        &self.properties
    }

    /// Get cached expanded properties if available.
    #[inline]
    #[must_use]
    pub fn expanded_properties(
        &self,
    ) -> Option<&HashMap<PropertyName, Property>> {
        self.expanded_properties.as_ref()
    }

    /// Set cached expanded properties.
    ///
    /// Called after `RefExpander` processes the schema.
    #[inline]
    pub fn set_expanded_properties(
        &mut self,
        properties: HashMap<PropertyName, Property>,
    ) {
        self.expanded_properties = Some(properties);
    }

    /// Reconstruct a `RawSchema` from this version.
    ///
    /// Used when we need to pass data to components expecting `RawSchema`.
    ///
    /// # Parameters
    /// - `name`: Schema name (typically derived from file basename)
    #[inline]
    #[must_use]
    pub fn to_raw(&self, name: Box<str>) -> RawSchema {
        // Convert PropertyName keys back to Box<str>
        let properties = self
            .properties
            .iter()
            .map(|(prop_name, prop)| (prop_name.as_str().into(), prop.clone()))
            .collect();

        let excludes = self
            .excludes
            .iter()
            .map(|prop_name| prop_name.as_str().into())
            .collect();

        let extends = self
            .extends
            .as_ref()
            .map(|schema_name| schema_name.as_str().into());

        RawSchema {
            version: self.version.clone(),
            name,
            extends,
            excludes,
            properties,
            metadata: RawSchemaMetadata::default(), /* Metadata not stored in
                                                     * version */
        }
    }

    /// Check if this version is fresh (matches file times).
    #[inline]
    #[must_use]
    pub fn is_fresh(
        &self,
        created_at: Option<std::time::SystemTime>,
        modified_at: Option<std::time::SystemTime>,
    ) -> bool {
        self.file_times.matches(created_at, modified_at)
    }

    /// Check if content matches (for hash-based staleness detection).
    #[inline]
    #[must_use]
    pub fn content_matches(&self, hash: &[u8; 32]) -> bool {
        self.hashes.content_matches(hash)
    }
}

// ─────────────────────────────────────────────────────────────────────────────
//  PropertyBankVersion
// ─────────────────────────────────────────────────────────────────────────────

/// A single version of the property bank file with validated, typed data.
///
/// Stores:
/// - File and hash metadata for staleness detection
/// - Property bank format version and properties (validated)
///
/// Keys are validated `PropertyName` instead of `Box<str>`.
#[derive(Debug, Clone, PartialEq, Archive, Serialize, Deserialize)]
pub struct PropertyBankVersion {
    /// File timestamp metadata.
    file_times: FileTimesMetadata,

    /// Hash metadata for staleness detection.
    hashes: HashMetadata,

    /// Property bank format version (e.g., "1.0").
    version: RawSchemaVersion,

    /// Map of validated property name to property bank entry.
    properties: HashMap<PropertyName, RawPropertyBankEntry>,
}

impl PropertyBankVersion {
    /// Create a new property bank version from a `RawPropertyBank`.
    ///
    /// # Errors
    /// Returns error if property name validation fails.
    #[inline]
    #[expect(
        clippy::iter_over_hash_type,
        reason = "Consuming iteration - order doesn't matter for validation"
    )]
    pub fn new(
        file_times: FileTimesMetadata,
        hashes: HashMetadata,
        raw: RawPropertyBank,
    ) -> Result<Self, SchemaIngestionError> {
        // Validate and convert property keys from Box<str> to PropertyName
        let mut properties = HashMap::with_capacity(raw.properties.len());
        for (name, entry) in raw.properties {
            let prop_name = PropertyName::try_new(&name).map_err(|e| {
                SchemaIngestionError::Io {
                    path: "property_bank".into(),
                    reason: format!("invalid property name '{name}': {e}")
                        .into(),
                }
            })?;
            properties.insert(prop_name, entry);
        }

        Ok(Self {
            file_times,
            hashes,
            version: raw.version,
            properties,
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

    /// Get properties.
    #[inline]
    #[must_use]
    pub fn properties(&self) -> &HashMap<PropertyName, RawPropertyBankEntry> {
        &self.properties
    }

    /// Reconstruct a `RawPropertyBank` from this version.
    ///
    /// Used when we need to pass data to components expecting
    /// `RawPropertyBank`.
    #[inline]
    #[must_use]
    pub fn to_raw(&self) -> RawPropertyBank {
        // Convert PropertyName keys back to Box<str>
        let properties = self
            .properties
            .iter()
            .map(|(name, entry)| (name.as_str().into(), entry.clone()))
            .collect();

        RawPropertyBank {
            version: self.version.clone(),
            properties,
            metadata: RawSchemaMetadata::default(), /* Metadata not stored in
                                                     * version */
        }
    }

    /// Check if this version is fresh (matches file times).
    #[inline]
    #[must_use]
    pub fn is_fresh(
        &self,
        created_at: Option<std::time::SystemTime>,
        modified_at: Option<std::time::SystemTime>,
    ) -> bool {
        self.file_times.matches(created_at, modified_at)
    }

    /// Check if content matches (for hash-based staleness detection).
    #[inline]
    #[must_use]
    pub fn content_matches(&self, hash: &[u8; 32]) -> bool {
        self.hashes.content_matches(hash)
    }
}

#[cfg(test)]
mod tests {
    use std::{
        collections::{BTreeMap, HashMap},
        time::SystemTime,
    };

    use super::*;

    fn create_test_raw_schema() -> RawSchema {
        RawSchema {
            version: RawSchemaVersion::default(),
            name: "test".into(),
            extends: None,
            excludes: Vec::new(),
            properties: HashMap::new(),
            metadata: RawSchemaMetadata::default(),
        }
    }

    fn create_test_raw_property_bank() -> RawPropertyBank {
        RawPropertyBank {
            version: RawSchemaVersion::default(),
            properties: HashMap::new(),
            metadata: RawSchemaMetadata::default(),
        }
    }

    #[test]
    fn schema_version_round_trip() {
        let raw = create_test_raw_schema();
        let file_times = FileTimesMetadata::new(
            Some(SystemTime::now()),
            Some(SystemTime::now()),
        );
        let hashes = HashMetadata::new([1u8; 32], BTreeMap::default());

        let version =
            SchemaVersion::new(file_times, hashes, raw.clone()).unwrap();
        let reconstructed = version.to_raw(raw.name.clone());

        assert_eq!(reconstructed.name, raw.name);
        assert_eq!(reconstructed.extends, raw.extends);
        assert_eq!(reconstructed.excludes, raw.excludes);
        assert_eq!(reconstructed.properties.len(), 0);
    }

    #[test]
    fn property_bank_version_round_trip() {
        let raw = create_test_raw_property_bank();
        let file_times = FileTimesMetadata::new(
            Some(SystemTime::now()),
            Some(SystemTime::now()),
        );
        let hashes = HashMetadata::new([1u8; 32], BTreeMap::default());

        let version =
            PropertyBankVersion::new(file_times, hashes, raw.clone()).unwrap();
        let reconstructed = version.to_raw();

        assert_eq!(reconstructed.properties.len(), 0);
    }

    #[test]
    fn schema_version_expanded_properties() {
        let raw = create_test_raw_schema();
        let file_times = FileTimesMetadata::new(
            Some(SystemTime::now()),
            Some(SystemTime::now()),
        );
        let hashes = HashMetadata::new([1u8; 32], BTreeMap::default());

        let mut version = SchemaVersion::new(file_times, hashes, raw).unwrap();

        assert!(version.expanded_properties().is_none());

        let expanded = HashMap::new();
        version.set_expanded_properties(expanded);

        assert!(version.expanded_properties().is_some());
    }

    #[test]
    fn schema_version_validates_property_names() {
        let mut raw = create_test_raw_schema();
        raw.properties.insert(
            "Invalid Name!".into(), // Invalid property name
            RawProperty::Inline(crate::schema::raw::property::RawPropertyInline {
                required: false,
                multi: false,
                spec: crate::schema::raw::property_spec::RawPropertySpec::Bool(
                    crate::schema::raw::property_spec::RawBoolSpec,
                ),
            }),
        );

        let file_times = FileTimesMetadata::new(
            Some(SystemTime::now()),
            Some(SystemTime::now()),
        );
        let hashes = HashMetadata::new([1u8; 32], BTreeMap::default());

        let result = SchemaVersion::new(file_times, hashes, raw);
        result.unwrap_err();
    }
}
