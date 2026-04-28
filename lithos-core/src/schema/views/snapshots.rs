//! Version snapshot payloads for schema and property bank views.
//!
//! ## Purpose
//!
//! This module defines the **version payload types** ([`SchemaVersion`],
//! [`PropertyBankVersion`]) that represent individual historical snapshots in
//! the ring buffer maintained by [`RawSchemaView`] and [`RawPropertyBankView`].
//!
//! **Critical architectural role**: These types extract and store **metadata
//! from Raw\* types** ([`RawSchema`], [`RawPropertyBank`]) which use
//! **serde-only** serialization and **cannot be directly persisted**. Version
//! types provide rkyv-serializable containers for this extracted metadata,
//! enabling staleness detection without re-parsing files.
//!
//! ## Version Payload Structure
//!
//! ### What Gets Stored in a Version?
//!
//! Each version snapshot contains:
//!
//! 1. **File Metadata** ([`FileStats`]):
//!    - Created timestamp (filesystem creation time)
//!    - Modified timestamp (filesystem last modified time)
//!    - File size in bytes
//!    - **Purpose**: Fast timestamp-based staleness checks without hashing
//!
//! 2. **Content Integrity** ([`HashRecord`]):
//!    - Content hash (Blake3 of entire file)
//!    - Per-property hashes (Blake3 per property definition)
//!    - **Purpose**: Accurate staleness detection and incremental resolution
//!
//! 3. **Business Metadata** (schema-specific):
//!    - **[`SchemaVersion`]**: `version`, `extends`, `excludes`,
//!      `bank_references`, optional `expanded_properties`
//!    - **[`PropertyBankVersion`]**: `version` (semantic version string)
//!    - **Purpose**: Enable queries without deserializing domain aggregates
//!
//! 4. **Recording Timestamp** ([`SystemTime`]):
//!    - When this version was ingested (wall clock time)
//!    - **Purpose**: Audit trail, debugging version rotation issues
//!
//! ### Why Not Store Full Domain Aggregates?
//!
//! Versions store **metadata only**, not full domain aggregates ([`Schema`],
//! [`PropertyBank`]). This separation provides:
//!
//! - **Stability**: Version structure changes less frequently than domain logic
//! - **Performance**: Smaller payloads (metadata-only) = faster queries
//! - **Queryability**: Extract only what's needed for staleness/inheritance
//!   checks
//! - **Storage efficiency**: No duplication of full property trees across
//!   versions
//!
//! ## Why Version Types Exist: The Raw\* Serialization Gap
//!
//! Version types solve a fundamental architectural constraint:
//!
//! **Raw\* types do NOT have rkyv derives** (serde-only by design):
//!
//! ```text
//! ❌ PROBLEM:
//!    Need to persist staleness metadata for RawSchema
//!    └─ But RawSchema only has serde derives (parsing-only type)
//!    └─ Cannot store in database (requires rkyv for zero-copy)
//!
//! ✅ SOLUTION:
//!    Extract metadata from RawSchema → SchemaVersion (has rkyv)
//!    └─ Store SchemaVersion in database
//!    └─ Enable staleness checks without re-parsing RawSchema
//! ```
//!
//! ## Hybrid Serialization Strategy
//!
//! Version types bridge the serialization gap between Raw\* and persistence:
//!
//! ```text
//! ┌──────────────────────────────────────────────────────────────┐
//! │  RAW LAYER (serde only) — NEVER PERSISTED                    │
//! │  RawSchema, RawPropertyBank                                  │
//! │  • Tolerant parsing (Option<T> fields)                       │
//! │  • Human-editable formats (YAML, JSON, TOML)                 │
//! │  • Syntax validation only                                    │
//! │  • Transient (exists only during ingestion)                  │
//! └───────────────────────┬──────────────────────────────────────┘
//!                         │
//!                         ├─── Extract metadata ────┐
//!                         │                         │
//!                         ▼                         ▼
//! ┌────────────────────────────────┐  ┌──────────────────────────┐
//! │  VIEW LAYER (rkyv only)        │  │  DOMAIN LAYER (rkyv)     │
//! │  ← THIS MODULE                 │  │  Schema, PropertyBank    │
//! │  SchemaVersion, etc.           │  │  • From RawSchema via    │
//! │  • Metadata from RawSchema     │  │    TryFrom (validation)  │
//! │  • File stats, hashes          │  │  • Business logic        │
//! │  • Inheritance metadata        │  │  • Invariants            │
//! │  • Zero-copy staleness checks  │  └──────────────────────────┘
//! └────────────────────────────────┘
//!          │                                     │
//!          └────────── Both persisted ───────────┘
//!                           │
//!                           ▼
//! ┌──────────────────────────────────────────────────────────────┐
//! │  DATABASE (rkyv-only storage)                                │
//! │  • RawSchemaView contains Vec<SchemaVersion> (metadata)      │
//! │  • Schema aggregate (domain logic)                           │
//! │  • Separate tables, separate concerns                        │
//! └──────────────────────────────────────────────────────────────┘
//! ```
//!
//! **Why this separation?**
//!
//! - **Raw Layer**: Optimized for **parsing** (serde only, no persistence)
//! - **View Layer**: Bridges serialization gap (extracts rkyv-serializable
//!   metadata from Raw\*)
//! - **Domain Layer**: Optimized for **business logic** (validated via
//!   `TryFrom<Raw*>`)
//!
//! Adding `rkyv` derives to `Raw*` types would:
//! - Violate separation of concerns (parsing vs. persistence)
//! - Create dual serialization maintenance burden
//! - Pollute parsing layer with storage artifacts
//!
//! ## `SchemaVersion`: Inheritance Metadata Extraction
//!
//! [`SchemaVersion`] extracts inheritance metadata during ingestion:
//!
//! - **`extends`**: Optional parent schema name for property inheritance
//! - **`excludes`**: Property names excluded from inheritance (validated)
//! - **`bank_references`**: Map of properties using `$ref` to property bank
//!   (enables incremental re-expansion)
//!
//! This metadata is stored in **every version** to enable:
//!
//! 1. **Inheritance graph construction**: Query `extends` without loading full
//!    schemas
//! 2. **Incremental resolution**: Identify which schemas need re-expansion when
//!    property bank changes
//! 3. **Zero-copy queries**: Access via `ArchivedSchemaVersion` without
//!    deserialization
//!
//! ## `PropertyBankVersion`: Simplified Structure
//!
//! [`PropertyBankVersion`] has a simpler structure (no inheritance metadata):
//!
//! - **`version`**: Semantic version string (e.g., "1.0")
//! - **`hashes`**: Content hash + per-property hashes
//! - **`file_stats`**: Timestamps and file size
//! - **`recorded_at`**: When ingested
//!
//! The property bank is a **singleton** (one per vault), so version tracking
//! focuses on **staleness detection** and **per-property change tracking**.
//!
//! ## Expanded Properties Caching (Future Optimization)
//!
//! [`SchemaVersion`] includes an **optional** `expanded_properties` field:
//!
//! ```rust,ignore
//! pub struct SchemaVersion {
//!     // ...
//!     /// Cached result of RefExpander (optional optimization).
//!     expanded_properties: Option<PropertyMap>,
//! }
//! ```
//!
//! **Purpose**: Cache the output of `RefExpander` (property bank reference
//! expansion) to avoid re-expanding on every load.
//!
//! **Current status**: Not yet implemented (placeholder for future
//! optimization).
//!
//! **When to populate**: After first expansion, store result here. On
//! subsequent loads, check if property bank hashes match—if so, use cached
//! expanded properties.
//!
//! ## Zero-Copy Access
//!
//! Version types are stored via `rkyv` serialization, enabling zero-copy
//! access:
//!
//! ```rust,ignore
//! // Hot path: zero-copy metadata query (no allocation)
//! let archived: &ArchivedSchemaVersion = view.current();
//! if let Some(parent) = archived.extends() {
//!     // Query inheritance without deserializing full schema
//! }
//! ```
//!
//! The archived types (`ArchivedSchemaVersion`, `ArchivedPropertyBankVersion`)
//! implement the same traits ([`VersionRead`]) as owned types, ensuring
//! consistent behavior.
//!
//! ## Types Defined
//!
//! - [`SchemaVersion`]: Version snapshot payload for schema files. Includes
//!   inheritance metadata (`extends`, `excludes`), bank references, and
//!   optional cached expanded properties.
//!
//! - [`PropertyBankVersion`]: Version snapshot payload for property bank files.
//!   Includes version string, hashes, and file stats (no inheritance metadata).
//!
//! [`RawSchemaView`]: super::raw::RawSchemaView
//! [`RawPropertyBankView`]: super::raw::RawPropertyBankView
//! [`Schema`]: crate::schema::aggregate::Schema
//! [`PropertyBank`]: crate::schema::bank::PropertyBank

#![expect(
    clippy::same_name_method,
    reason = "Trait contracts intentionally mirror established version API"
)]

use std::{
    collections::{HashMap, HashSet},
    time::SystemTime,
};

use rkyv::{Archive, Deserialize, Serialize};

use super::{
    HashRecord,
    contracts::{Version, VersionRead},
};
use crate::{
    fs::FileStats,
    schema::{
        aggregate::SchemaName,
        error::SchemaIngestionError,
        property::{PropertyMap, PropertyName},
        raw::RawSchema,
    },
    support::hash::Blake3Hash,
};

// ─────────────────────────────────────────────────────────────────────────────
//  SchemaVersion
// ─────────────────────────────────────────────────────────────────────────────

/// Represents a single version of a schema file with validated, typed data.
///
/// Stores:
/// - File and hash metadata for staleness detection.
/// - Schema format version, inheritance metadata (validated, queryable).
/// - Cached expanded properties for incremental resolution.
///
/// ## Design Rationale
///
/// This hybrid approach keeps metadata fields (`extends`, `excludes`) as
/// validated types for direct querying while leaving the raw property tree in
/// the Raw* parsing layer. This avoids adding rkyv derives to the raw schema
/// parsing types while maintaining queryability of inheritance metadata.
#[derive(Debug, Clone, PartialEq, Archive, Serialize, Deserialize)]
pub struct SchemaVersion {
    /// File statistics metadata for staleness detection.
    file_stats: FileStats,

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
    /// # use lithos_core::fs::FileStats;
    /// # use lithos_core::schema::views::HashRecord;
    /// #
    /// # let raw: RawSchema = todo!();
    /// # let stats: FileStats = todo!();
    /// # let hashes: HashRecord = todo!();
    /// let version = SchemaVersion::new(stats, hashes, &raw).unwrap();
    /// ```
    #[inline]
    pub fn new(
        file_stats: FileStats,
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
            file_stats,
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
    pub fn file_stats(&self) -> &FileStats {
        &self.file_stats
    }

    /// Returns when this version was recorded in storage.
    #[inline]
    #[must_use]
    pub fn recorded_at(&self) -> SystemTime {
        self.recorded_at
    }

    /// Returns hash metadata for staleness detection.
    #[inline]
    #[must_use]
    pub fn hashes(&self) -> &HashRecord {
        &self.hashes
    }

    /// Returns the schema format version string (e.g., `"1.0"`).
    #[inline]
    #[must_use]
    pub fn version(&self) -> &str {
        &self.version
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

    /// Clones this version with updated file stats and hashes.
    ///
    /// Resets cached expanded properties to keep refresh behavior consistent
    /// with raw-based reconstruction.
    #[inline]
    #[must_use]
    pub fn with_metadata(
        &self,
        file_stats: FileStats,
        hashes: HashRecord,
    ) -> Self {
        Self {
            file_stats,
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

/// Implements [`Version`] for [`SchemaVersion`].
impl Version for SchemaVersion {
    #[inline]
    fn file_stats(&self) -> &FileStats {
        self.file_stats()
    }

    #[inline]
    fn recorded_at(&self) -> SystemTime {
        self.recorded_at()
    }

    #[inline]
    fn hashes(&self) -> &HashRecord {
        self.hashes()
    }

    #[inline]
    fn set_file_stats(&mut self, file_stats: FileStats) {
        self.file_stats = file_stats;
        self.recorded_at = SystemTime::now();
    }

    #[inline]
    fn with_metadata(&self, file_stats: FileStats, hashes: HashRecord) -> Self {
        SchemaVersion::with_metadata(self, file_stats, hashes)
    }
}

/// Implements [`VersionRead`] for [`SchemaVersion`].
impl VersionRead for SchemaVersion {
    #[inline]
    fn is_timestamp_match(
        &self,
        created_at: Option<SystemTime>,
        modified_at: Option<SystemTime>,
    ) -> bool {
        self.file_stats().is_timestamp_match(created_at, modified_at)
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

/// Implements [`VersionRead`] for [`ArchivedSchemaVersion`] (zero-copy).
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
        self.file_stats.is_timestamp_match(created_at, modified_at)
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
    file_stats: FileStats,

    /// Hash metadata for staleness and incremental resolution.
    hashes: HashRecord,

    /// Property bank format version as simple string (e.g., `"1.0"`).
    ///
    /// Stored as `Box<str>` instead of `RawSchemaVersion` to avoid requiring
    /// rkyv derives on Raw* types.
    version: Box<str>,

    /// When this version was recorded in storage.
    #[rkyv(with = rkyv::with::AsUnixTime)]
    recorded_at: SystemTime,
}

impl PropertyBankVersion {
    /// Creates a new property bank version from a version string.
    ///
    /// # Errors
    ///
    /// This constructor is currently infallible; the [`Result`] is retained for
    /// pipeline compatibility if future validation is added.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use lithos_core::schema::views::PropertyBankVersion;
    /// # use lithos_core::fs::FileStats;
    /// # use lithos_core::schema::views::HashRecord;
    /// #
    /// # let stats: FileStats = todo!();
    /// # let hashes: HashRecord = todo!();
    /// let version = PropertyBankVersion::new(stats, hashes, "1.0").unwrap();
    /// ```
    #[inline]
    pub fn new(
        file_stats: FileStats,
        hashes: HashRecord,
        version: &str,
    ) -> Result<Self, SchemaIngestionError> {
        Ok(Self {
            file_stats,
            hashes,
            version: version.into(),
            recorded_at: SystemTime::now(),
        })
    }

    /// Returns file statistics metadata for this version.
    #[inline]
    #[must_use]
    pub fn file_stats(&self) -> &FileStats {
        &self.file_stats
    }

    /// Returns when this version was recorded in storage.
    #[inline]
    #[must_use]
    pub fn recorded_at(&self) -> SystemTime {
        self.recorded_at
    }

    /// Updates file statistics metadata in-place.
    #[inline]
    pub fn set_file_stats(&mut self, file_stats: FileStats) {
        self.file_stats = file_stats;
        self.recorded_at = SystemTime::now();
    }

    /// Returns hash metadata for staleness detection.
    #[inline]
    #[must_use]
    pub fn hashes(&self) -> &HashRecord {
        &self.hashes
    }

    /// Returns the property bank format version string (e.g., `"1.0"`).
    #[inline]
    #[must_use]
    pub fn version(&self) -> &str {
        &self.version
    }
}

/// Implements [`Version`] for [`PropertyBankVersion`].
impl Version for PropertyBankVersion {
    #[inline]
    fn file_stats(&self) -> &FileStats {
        self.file_stats()
    }

    #[inline]
    fn recorded_at(&self) -> SystemTime {
        self.recorded_at()
    }

    #[inline]
    fn hashes(&self) -> &HashRecord {
        self.hashes()
    }

    #[inline]
    fn set_file_stats(&mut self, file_stats: FileStats) {
        self.file_stats = file_stats;
        self.recorded_at = SystemTime::now();
    }

    #[inline]
    fn with_metadata(&self, file_stats: FileStats, hashes: HashRecord) -> Self {
        Self {
            file_stats,
            hashes,
            version: self.version.clone(),
            recorded_at: SystemTime::now(),
        }
    }
}

/// Implements [`VersionRead`] for [`PropertyBankVersion`].
impl VersionRead for PropertyBankVersion {
    #[inline]
    fn is_timestamp_match(
        &self,
        created_at: Option<SystemTime>,
        modified_at: Option<SystemTime>,
    ) -> bool {
        self.file_stats().is_timestamp_match(created_at, modified_at)
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

/// Implements [`VersionRead`] for [`ArchivedPropertyBankVersion`] (zero-copy).
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
        self.file_stats.is_timestamp_match(created_at, modified_at)
    }

    #[inline]
    fn version(&self) -> &str {
        self.version.as_ref()
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
        let file_times =
            FileStats::new(Some(SystemTime::now()), Some(SystemTime::now()), 0);
        let hashes =
            HashRecord::new(Blake3Hash::new([1u8; 32]), HashMap::default());

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

        let file_times = FileStats::new(None, None, 0);
        let hashes = HashRecord::new(Blake3Hash::new([0; 32]), HashMap::new());
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
            FileStats::new(None, None, 0),
            HashRecord::new(Blake3Hash::new([0; 32]), HashMap::new()),
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
    fn schema_version_set_file_stats_updates_metadata() {
        let raw = create_test_raw_schema();
        let initial = FileStats::new(None, None, 0);
        let hashes = HashRecord::new(Blake3Hash::new([0; 32]), HashMap::new());
        let mut version = SchemaVersion::new(initial, hashes, &raw).unwrap();

        let updated = FileStats::new(Some(SystemTime::now()), None, 0);
        Version::set_file_stats(&mut version, updated);

        assert_eq!(version.file_stats(), &updated);
    }

    #[test]
    fn property_bank_version_with_metadata_replaces_hash_and_timestamps() {
        let original = PropertyBankVersion::new(
            FileStats::new(None, None, 0),
            HashRecord::new(Blake3Hash::new([1; 32]), HashMap::new()),
            "1.0",
        )
        .unwrap();

        let replacement = Version::with_metadata(
            &original,
            FileStats::new(Some(SystemTime::now()), None, 0),
            HashRecord::new(Blake3Hash::new([2; 32]), HashMap::new()),
        );

        assert_eq!(replacement.hashes().content(), &Blake3Hash::new([2; 32]));
        assert_eq!(replacement.version(), "1.0");
    }

    #[test]
    fn archived_schema_version_supports_zero_copy_version_read() {
        let raw = create_test_raw_schema();
        let version = SchemaVersion::new(
            FileStats::new(None, None, 0),
            HashRecord::new(Blake3Hash::new([7; 32]), HashMap::new()),
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
            FileStats::new(None, None, 0),
            HashRecord::new(Blake3Hash::new([3; 32]), HashMap::new()),
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
