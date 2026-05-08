//! Typed version snapshots for zero-copy storage.
//!
//! Provides the [`SchemaVersion`] and [`PropertyBankVersion`] types which
//! capture validated snapshots of domain objects. These types are optimized
//! for fast staleness detection and efficient topological graph construction
//! without requiring full deserialization of the underlying raw data.

use std::{
    collections::{HashMap, HashSet},
    time::SystemTime,
};

use rkyv::{Archive, Deserialize, Serialize};

use crate::{
    fs::FileInfo,
    schema::{
        error::SchemaIngestionError,
        identifier::SchemaName,
        property::{PropertyMap, PropertyName},
        raw::{RawPropertyBank, RawSchema},
        views::{
            contracts::{Version, VersionRead},
            hashes::HashRecord,
        },
    },
    support::hash::Blake3Hash,
};

// ─────────────────────────────────────────────────────────────────────────────
//  SchemaVersion
// ─────────────────────────────────────────────────────────────────────────────

/// Represents a single version of a schema file with validated, typed metadata.
///
/// Stores:
/// - File and hash metadata for staleness detection.
/// - Inheritance metadata (`extends`, `excludes`) for graph construction.
/// - Bank property references for impact analysis.
/// - Cached expanded properties to optimize resolution.
///
/// ## Design Rationale
///
/// This structure acts as a "View" optimized for storage and frequent queries.
/// By storing inheritance and bank references as typed fields, we can perform
/// topological sorts and staleness detection with zero-copy reads, avoiding
/// the overhead of full JSON/YAML/TOML deserialization.
#[derive(Debug, Clone, PartialEq, Archive, Serialize, Deserialize)]
pub struct SchemaVersion {
    /// File statistics metadata for staleness detection.
    info: FileInfo,

    /// Hash metadata for staleness and incremental resolution.
    hashes: HashRecord,

    /// Schema format version as simple string (e.g., `"1.0"`).
    ///
    /// Stored as `Box<str>` instead of `RawSchemaVersion` to avoid requiring
    /// rkyv derives on Raw* types.
    version: Box<str>,

    /// Parent schema name from the `extends` field, if any.
    ///
    /// Validated and stored as typed field for efficient querying.
    extends: Option<SchemaName>,

    /// Property names excluded from the parent (from `excludes` field).
    ///
    /// Validated and stored as typed field for efficient querying.
    excludes: Vec<PropertyName>,

    /// Map of schema property name to property bank target name.
    ///
    /// Extracted from `$ref` entries during ingestion.
    bank_references: HashMap<PropertyName, PropertyName>,

    /// Cached expanded properties from `RefExpander`.
    ///
    /// Enables skipping expansion when [`PropertyBank`] is fresh.
    expanded_properties: Option<PropertyMap>,

    /// When this version was recorded in storage.
    #[rkyv(with = rkyv::with::AsUnixTime)]
    recorded_at: SystemTime,
}

impl SchemaVersion {
    /// Creates a new schema version from a parsed [`RawSchema`].
    ///
    /// Extracts inheritance metadata (`extends`, `excludes`) and bank
    /// references from the parsed schema.
    ///
    /// # Errors
    ///
    /// This constructor is currently infallible; the [`Result`] is retained for
    /// pipeline compatibility if future validation is added.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use lithos_core::schema::views::SchemaVersion;
    /// # use lithos_core::schema::raw::RawSchema;
    /// # use lithos_core::fs::FileInfo;
    /// # use lithos_core::schema::views::HashRecord;
    /// #
    /// # let raw: RawSchema = todo!();
    /// # let info: FileInfo = todo!();
    /// # let hashes: HashRecord = todo!();
    /// let version = SchemaVersion::new(info, hashes, &raw).unwrap();
    /// ```
    #[inline]
    pub fn new(
        info: FileInfo,
        hashes: HashRecord,
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
            info,
            hashes,
            version: raw.version().as_str().into(),
            extends,
            excludes,
            bank_references,
            expanded_properties: None,
            recorded_at: SystemTime::now(),
        })
    }

    /// Returns file statistics metadata for this version.
    #[inline]
    #[must_use]
    pub fn info(&self) -> &FileInfo {
        &self.info
    }

    /// Returns the parent schema name from `extends`, if any.
    #[inline]
    #[must_use]
    pub fn extends(&self) -> Option<&SchemaName> {
        self.extends.as_ref()
    }

    /// Returns excluded property names from the `excludes` field.
    #[inline]
    #[must_use]
    pub fn excludes(&self) -> &[PropertyName] {
        &self.excludes
    }

    /// Returns bank property references.
    ///
    /// Returns a map of schema property name to target bank property name.
    #[inline]
    #[must_use]
    pub fn bank_references(&self) -> &HashMap<PropertyName, PropertyName> {
        &self.bank_references
    }

    /// Returns schema properties affected by property bank changes.
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

    /// Returns cached expanded properties, if available.
    #[inline]
    #[must_use]
    pub fn expanded_properties(&self) -> Option<&PropertyMap> {
        self.expanded_properties.as_ref()
    }

    /// Caches expanded properties after [`RefExpander`] runs.
    #[inline]
    pub fn set_expanded_properties(&mut self, properties: PropertyMap) {
        self.expanded_properties = Some(properties);
    }
}

/// Implements [`Version`] for [`SchemaVersion`].
impl Version for SchemaVersion {
    #[inline]
    fn file_info(&self) -> &FileInfo {
        self.info()
    }

    #[inline]
    fn recorded_at(&self) -> SystemTime {
        self.recorded_at
    }

    #[inline]
    fn hashes(&self) -> &HashRecord {
        &self.hashes
    }

    #[inline]
    fn set_file_info(&mut self, info: FileInfo) {
        self.info = info;
        self.recorded_at = SystemTime::now();
    }

    #[inline]
    fn with_metadata(&self, info: FileInfo, hashes: HashRecord) -> Self {
        Self {
            info,
            hashes,
            version: self.version.clone(),
            extends: self.extends.clone(),
            excludes: self.excludes.clone(),
            bank_references: self.bank_references.clone(),
            expanded_properties: None,
            recorded_at: SystemTime::now(),
        }
    }
}

/// Implements [`VersionRead`] for [`SchemaVersion`].
impl VersionRead for SchemaVersion {
    #[inline]
    fn file_info(&self) -> &FileInfo {
        &self.info
    }

    #[inline]
    fn is_timestamp_match(
        &self,
        created_at: Option<SystemTime>,
        modified_at: Option<SystemTime>,
    ) -> bool {
        self.info.is_timestamp_match(created_at, modified_at)
    }

    #[inline]
    fn is_content_match(&self, hash: &Blake3Hash) -> bool {
        self.hashes.is_content_match(hash)
    }

    #[inline]
    fn version(&self) -> &str {
        &self.version
    }
}

/// Implements [`VersionRead`] for [`ArchivedSchemaVersion`] (zero-copy).
#[expect(
    clippy::missing_trait_methods,
    reason = "file_info() default panics - archived types use \
              is_timestamp_match() directly"
)]
impl VersionRead for ArchivedSchemaVersion {
    #[inline]
    fn is_content_match(&self, hash: &Blake3Hash) -> bool {
        self.hashes.is_content_match(hash)
    }

    #[inline]
    fn is_timestamp_match(
        &self,
        created_at: Option<SystemTime>,
        modified_at: Option<SystemTime>,
    ) -> bool {
        self.info.is_timestamp_match(created_at, modified_at)
    }

    #[inline]
    fn version(&self) -> &str {
        self.version.as_ref()
    }
}

// ─────────────────────────────────────────────────────────────────────────────
//  PropertyBankVersion
// ─────────────────────────────────────────────────────────────────────────────

/// Represents a single version of the property bank file with validated, typed
/// data.
///
/// Stores:
/// - File and hash metadata for staleness detection.
/// - Property bank format version as simple string.
///
/// ## Design Rationale
///
/// Similar to [`SchemaVersion`], this uses a hybrid approach: metadata fields
/// are stored as validated types, while the complex property tree remains in
/// the Raw* parsing layer to avoid adding rkyv derives.
#[derive(Debug, Clone, PartialEq, Archive, Serialize, Deserialize)]
pub struct PropertyBankVersion {
    /// File statistics metadata for staleness detection.
    info: FileInfo,

    /// Hash metadata for staleness and incremental resolution.
    hashes: HashRecord,

    /// Property bank format version as simple string (e.g., `"1.0"`).
    version: Box<str>,

    /// When this version was recorded in storage.
    #[rkyv(with = rkyv::with::AsUnixTime)]
    recorded_at: SystemTime,
}

impl PropertyBankVersion {
    /// Creates a new property bank version from a parsed [`RawPropertyBank`].
    ///
    /// # Errors
    ///
    /// Currently infallible, but returns `Result` for future validation.
    #[inline]
    pub fn new(
        info: FileInfo,
        hashes: HashRecord,
        raw: &RawPropertyBank,
    ) -> Result<Self, SchemaIngestionError> {
        Ok(Self {
            info,
            hashes,
            version: raw.version().as_str().into(),
            recorded_at: SystemTime::now(),
        })
    }

    /// Returns file statistics metadata for this version.
    #[inline]
    #[must_use]
    pub fn info(&self) -> &FileInfo {
        &self.info
    }
}

/// Implements [`Version`] for [`PropertyBankVersion`].
impl Version for PropertyBankVersion {
    #[inline]
    fn file_info(&self) -> &FileInfo {
        &self.info
    }

    #[inline]
    fn recorded_at(&self) -> SystemTime {
        self.recorded_at
    }

    #[inline]
    fn hashes(&self) -> &HashRecord {
        &self.hashes
    }

    #[inline]
    fn set_file_info(&mut self, info: FileInfo) {
        self.info = info;
        self.recorded_at = SystemTime::now();
    }

    #[inline]
    fn with_metadata(&self, info: FileInfo, hashes: HashRecord) -> Self {
        Self {
            info,
            hashes,
            version: self.version.clone(),
            recorded_at: SystemTime::now(),
        }
    }
}

/// Implements [`VersionRead`] for [`PropertyBankVersion`].
impl VersionRead for PropertyBankVersion {
    #[inline]
    fn file_info(&self) -> &FileInfo {
        &self.info
    }

    #[inline]
    fn is_timestamp_match(
        &self,
        created_at: Option<SystemTime>,
        modified_at: Option<SystemTime>,
    ) -> bool {
        self.info.is_timestamp_match(created_at, modified_at)
    }

    #[inline]
    fn is_content_match(&self, hash: &Blake3Hash) -> bool {
        self.hashes.is_content_match(hash)
    }

    #[inline]
    fn version(&self) -> &str {
        &self.version
    }
}

/// Implements [`VersionRead`] for [`ArchivedPropertyBankVersion`] (zero-copy).
#[expect(
    clippy::missing_trait_methods,
    reason = "file_info() default panics - archived types use \
              is_timestamp_match() directly"
)]
impl VersionRead for ArchivedPropertyBankVersion {
    #[inline]
    fn is_content_match(&self, hash: &Blake3Hash) -> bool {
        self.hashes.is_content_match(hash)
    }

    #[inline]
    fn is_timestamp_match(
        &self,
        created_at: Option<SystemTime>,
        modified_at: Option<SystemTime>,
    ) -> bool {
        self.info.is_timestamp_match(created_at, modified_at)
    }

    #[inline]
    fn version(&self) -> &str {
        self.version.as_ref()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    mod schema_version {
        use super::*;

        #[test]
        fn new_extracts_metadata_correctly() {
            let ref_path: crate::schema::raw::property::RawPropertyRefPath =
                serde_json::from_str(r##""#property_bank/target""##).unwrap();

            let raw = RawSchema {
                version: crate::schema::raw::RawSchemaVersion::default(),
                name: "Test".into(),
                extends: None,
                excludes: vec![],
                properties: {
                    let mut map = std::collections::HashMap::new();
                    map.insert(
                        "prop1".try_into().unwrap(),
                        crate::schema::raw::property::RawProperty::Ref(
                            crate::schema::raw::property::RawPropertyRef {
                                ref_path,
                                required: None,
                                multi: None,
                                options: None,
                                pattern: None,
                                min: None,
                                max: None,
                                step: None,
                                format: None,
                                directory: None,
                                file_class: None,
                            },
                        ),
                    );
                    crate::schema::raw::property::RawPropertyMap::from_map(map)
                },
                info: crate::fs::FileInfo::new(None, None, 0),
            };

            let info = crate::fs::FileInfo::new(None, None, 100);
            let hashes = crate::schema::views::hashes::HashRecord::new(
                crate::support::hash::Blake3Hash::new([0; 32]),
                crate::schema::views::RawPropertyHashIndex::default(),
            );

            let version =
                SchemaVersion::new(info, hashes.clone(), &raw).unwrap();

            assert_eq!(version.version(), "1.0");
            assert_eq!(version.info().size(), 100);
            assert_eq!(version.hashes().content(), hashes.content());

            let prop_name: crate::schema::property::PropertyName =
                "prop1".try_into().unwrap();
            let target_name: crate::schema::property::PropertyName =
                "target".try_into().unwrap();
            assert_eq!(
                version.bank_references().get(&prop_name),
                Some(&target_name)
            );
        }
    }

    mod property_bank_version {
        use super::*;

        #[test]
        fn new_extracts_metadata_correctly() {
            let raw = RawPropertyBank {
                version: crate::schema::raw::RawSchemaVersion::default(),
                properties: crate::schema::raw::property::RawPropertyMap::new(),
                info: crate::fs::FileInfo::new(None, None, 0),
            };

            let info = crate::fs::FileInfo::new(None, None, 100);
            let hashes = crate::schema::views::hashes::HashRecord::new(
                crate::support::hash::Blake3Hash::new([0; 32]),
                crate::schema::views::RawPropertyHashIndex::default(),
            );

            let version =
                PropertyBankVersion::new(info, hashes.clone(), &raw).unwrap();

            assert_eq!(version.version(), "1.0");
            assert_eq!(version.info().size(), 100);
            assert_eq!(version.hashes().content(), hashes.content());
        }
    }
}
