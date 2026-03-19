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
//! - Complex property trees (`RawProperty`, `RawPropertyBankEntry`) are
//!   serialized via serde
//! - The serde-serialized bytes are stored in `Vec<u8>` fields which rkyv
//!   handles natively
//!
//! This keeps the parsing layer (Raw* types with serde) separate from the
//! storage layer (version types with rkyv), while maintaining queryability of
//! key metadata fields.

use std::{collections::HashMap, path::PathBuf};

use rkyv::{Archive, Deserialize, Serialize};

use super::metadata::{FileTimesMetadata, HashMetadata};
use crate::schema::{
    aggregate::SchemaName,
    error::SchemaIngestionError,
    property::{Property, PropertyName},
    raw::{RawPropertyBank, RawSchema},
};

// ─────────────────────────────────────────────────────────────────────────────
//  SchemaVersion
// ─────────────────────────────────────────────────────────────────────────────

/// A single version of a schema file with validated, typed data.
///
/// Stores:
/// - File and hash metadata for staleness detection
/// - Schema format version, inheritance metadata (validated, queryable)
/// - Properties as serde-serialized bytes (avoids rkyv on Raw* types)
/// - Cached expanded properties for incremental resolution
///
/// ## Design Rationale
///
/// This hybrid approach keeps metadata fields (`extends`, `excludes`) as
/// validated types for direct querying, while serializing the complex property
/// tree via serde. This avoids adding rkyv derives to the entire Raw* parsing
/// layer while maintaining queryability of inheritance metadata.
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

    /// Raw properties map (serde JSON format).
    ///
    /// Contains: `HashMap<Box<str>, RawProperty>`.
    ///
    /// Properties are serialized via serde (not rkyv) to avoid adding rkyv
    /// derives to the Raw* parsing layer. The serialization format (JSON)
    /// is independent of the original schema file format (TOML/JSON/YAML).
    raw_properties: Vec<u8>,

    /// Cached expanded properties (from `RefExpander`).
    ///
    /// Enables skipping expansion when `PropertyBank` is fresh.
    expanded_properties: Option<HashMap<PropertyName, Property>>,
}

impl SchemaVersion {
    /// Create a new schema version from a `RawSchema`.
    ///
    /// # Errors
    /// Returns error if property name validation fails or serialization fails.
    #[inline]
    pub fn new(
        file_times: FileTimesMetadata,
        hashes: HashMetadata,
        raw: &RawSchema,
    ) -> Result<Self, SchemaIngestionError> {
        // Validate and convert excludes
        let mut excludes = Vec::with_capacity(raw.excludes.len());
        for exclude in &raw.excludes {
            let prop_name = PropertyName::try_new_with_context(
                exclude.as_ref(),
                crate::schema::error::PropertyNameContext::Exclude,
            )
            .map_err(|e| SchemaIngestionError::Schema {
                path: PathBuf::from(raw.name.as_ref()),
                source: e,
            })?;
            excludes.push(prop_name);
        }

        // Validate extends if present
        let extends = raw
            .extends
            .as_ref()
            .map(|name| {
                SchemaName::try_new(name.as_ref()).map_err(|e| {
                    SchemaIngestionError::Schema {
                        path: PathBuf::from(raw.name.as_ref()),
                        source: e,
                    }
                })
            })
            .transpose()?;

        // Validate property names (without consuming the map)
        #[expect(
            clippy::iter_over_hash_type,
            reason = "Validation does not depend on iteration order"
        )]
        for prop_name in raw.properties.keys() {
            PropertyName::try_new(prop_name.as_ref()).map_err(|e| {
                SchemaIngestionError::Schema {
                    path: PathBuf::from(raw.name.as_ref()),
                    source: e,
                }
            })?;
        }

        // Serialize properties via serde (not rkyv)
        let raw_properties =
            serde_json::to_vec(&raw.properties).map_err(|e| {
                SchemaIngestionError::Parse(
                    crate::schema::error::SchemaParseError::Serialization {
                        path: PathBuf::from(raw.name.as_ref()),
                        reason: e.to_string().into(),
                    },
                )
            })?;

        Ok(Self {
            file_times,
            hashes,
            version: raw.version.as_str().into(),
            extends,
            excludes,
            raw_properties,
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

    /// Get raw properties as serialized bytes.
    #[inline]
    #[must_use]
    pub fn raw_properties(&self) -> &[u8] {
        &self.raw_properties
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
}

// ─────────────────────────────────────────────────────────────────────────────
//  PropertyBankVersion
// ─────────────────────────────────────────────────────────────────────────────

/// A single version of the property bank file with validated, typed data.
///
/// Stores:
/// - File and hash metadata for staleness detection
/// - Property bank format version as simple string
/// - Properties as serde-serialized bytes (avoids rkyv on Raw* types)
///
/// ## Design Rationale
///
/// Similar to `SchemaVersion`, this uses a hybrid approach: metadata fields are
/// stored as validated types, while the complex property tree is serialized via
/// serde to avoid adding rkyv derives to the Raw* parsing layer.
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

    /// Raw properties map (serde JSON format).
    ///
    /// Contains: `HashMap<Box<str>, RawPropertyBankEntry>`.
    ///
    /// Properties are serialized via serde (not rkyv) to avoid adding rkyv
    /// derives to the Raw* parsing layer.
    raw_properties: Vec<u8>,
}

impl PropertyBankVersion {
    /// Create a new property bank version from a `RawPropertyBank`.
    ///
    /// # Errors
    /// Returns error if property name validation fails or serialization fails.
    #[inline]
    pub fn new(
        file_times: FileTimesMetadata,
        hashes: HashMetadata,
        raw: &RawPropertyBank,
    ) -> Result<Self, SchemaIngestionError> {
        // Validate property names (without consuming the map)
        #[expect(
            clippy::iter_over_hash_type,
            reason = "Validation does not depend on iteration order"
        )]
        for prop_name in raw.properties.keys() {
            PropertyName::try_new_with_context(
                prop_name.as_ref(),
                crate::schema::error::PropertyNameContext::PropertyBank,
            )
            .map_err(|e| SchemaIngestionError::Schema {
                path: PathBuf::from("property_bank"),
                source: e,
            })?;
        }

        // Serialize properties via serde (not rkyv)
        let raw_properties =
            serde_json::to_vec(&raw.properties).map_err(|e| {
                SchemaIngestionError::Parse(
                    crate::schema::error::SchemaParseError::Serialization {
                        path: PathBuf::from("property_bank"),
                        reason: e.to_string().into(),
                    },
                )
            })?;

        Ok(Self {
            file_times,
            hashes,
            version: raw.version.as_str().into(),
            raw_properties,
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

    /// Get raw properties as serialized bytes.
    #[inline]
    #[must_use]
    pub fn raw_properties(&self) -> &[u8] {
        &self.raw_properties
    }
}

#[cfg(test)]
mod tests {
    use std::{collections::HashMap, time::SystemTime};

    use super::*;
    use crate::schema::raw::{
        RawSchemaMetadata, RawSchemaVersion, property::RawProperty,
    };

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
        let hashes = HashMetadata::new([1u8; 32], HashMap::default());

        let result = SchemaVersion::new(file_times, hashes, &raw);
        result.unwrap_err();
    }
}
