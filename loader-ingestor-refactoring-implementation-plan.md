# Loader/Ingestor Refactoring - Implementation Plan

**Date**: 2026-03-17
**Status**: Implementation Ready
**Related Document**: `loader-ingestor-architecture-review.md`

---

## Overview

This plan implements the architectural redesign of the schema loading pipeline to eliminate:
- Double-loop anti-pattern
- N+1 database queries
- Missing incremental resolution
- Inefficient caching strategies
- Metadata duplication

**Expected Performance Improvement**: 80% reduction in load time for fresh schemas

---

## Implementation Phases

### Phase 1: Create New Version Type Hierarchy
**Goal**: Replace `RawFileVersion` with four specialized types
**Duration**: 1-2 days
**Risk**: Low - purely additive, no breaking changes yet

#### Phase 1.1: Create Shared Metadata Types

**File**: `lithos-core/src/schema/views/metadata.rs` (new file)

```rust
//! Shared metadata types for schema and property bank versions.

use std::time::SystemTime;
use std::collections::BTreeMap;

use rkyv::{Archive, Deserialize, Serialize};
use rkyv::with::{AsUnixTime, Map};

use crate::schema::property::PropertyName;
use crate::schema::raw::RawProperty;

// ─────────────────────────────────────────────────────────────────────────────
//  FileVersionMetadata
// ─────────────────────────────────────────────────────────────────────────────

/// File timestamp metadata shared by schema and property bank versions.
///
/// Tracks when the file was created, modified, and when this version
/// was recorded in the database.
#[derive(Debug, Clone, PartialEq, Eq, Archive, Serialize, Deserialize)]
pub struct FileVersionMetadata {
    /// File creation timestamp from filesystem
    #[rkyv(with = Map<AsUnixTime>)]
    created_at: Option<SystemTime>,

    /// File modification timestamp from filesystem
    #[rkyv(with = Map<AsUnixTime>)]
    modified_at: Option<SystemTime>,

    /// When this version was recorded in the database
    #[rkyv(with = AsUnixTime)]
    recorded_at: SystemTime,
}

impl FileVersionMetadata {
    /// Create new file version metadata.
    #[inline]
    #[must_use]
    pub fn new(
        created_at: Option<SystemTime>,
        modified_at: Option<SystemTime>,
    ) -> Self {
        Self {
            created_at,
            modified_at,
            recorded_at: SystemTime::now(),
        }
    }

    /// Get file creation timestamp.
    #[inline]
    #[must_use]
    pub fn created_at(&self) -> Option<SystemTime> {
        self.created_at
    }

    /// Get file modification timestamp.
    #[inline]
    #[must_use]
    pub fn modified_at(&self) -> Option<SystemTime> {
        self.modified_at
    }

    /// Get database recording timestamp.
    #[inline]
    #[must_use]
    pub fn recorded_at(&self) -> SystemTime {
        self.recorded_at
    }

    /// Check if timestamps match (for staleness detection).
    #[inline]
    #[must_use]
    pub fn matches(
        &self,
        created_at: Option<SystemTime>,
        modified_at: Option<SystemTime>,
    ) -> bool {
        self.created_at == created_at && self.modified_at == modified_at
    }
}

// ─────────────────────────────────────────────────────────────────────────────
//  HashMetadata
// ─────────────────────────────────────────────────────────────────────────────

/// Content and property hash metadata shared by schema and property bank versions.
///
/// Used for staleness detection (content hash) and incremental resolution
/// (property hashes).
#[derive(Debug, Clone, PartialEq, Eq, Archive, Serialize, Deserialize)]
pub struct HashMetadata {
    /// Blake3 hash of file content for staleness detection
    content_hash: [u8; 32],

    /// Per-property Blake3 hashes for incremental updates/resolution
    property_hashes: BTreeMap<PropertyName, [u8; 32]>,
}

impl HashMetadata {
    /// Create new hash metadata.
    #[inline]
    #[must_use]
    pub fn new(
        content_hash: [u8; 32],
        property_hashes: BTreeMap<PropertyName, [u8; 32]>,
    ) -> Self {
        Self {
            content_hash,
            property_hashes,
        }
    }

    /// Get content hash.
    #[inline]
    #[must_use]
    pub fn content_hash(&self) -> &[u8; 32] {
        &self.content_hash
    }

    /// Get property hashes.
    #[inline]
    #[must_use]
    pub fn property_hashes(&self) -> &BTreeMap<PropertyName, [u8; 32]> {
        &self.property_hashes
    }

    /// Check if content hash matches (for staleness detection).
    #[inline]
    #[must_use]
    pub fn content_matches(&self, hash: &[u8; 32]) -> bool {
        self.content_hash == *hash
    }

    /// Compute property hashes from raw properties.
    ///
    /// This is the canonical hash computation used by both schemas
    /// and property banks.
    pub fn compute_property_hashes(
        properties: &std::collections::HashMap<Box<str>, RawProperty>,
    ) -> BTreeMap<PropertyName, [u8; 32]> {
        properties
            .iter()
            .filter_map(|(name, prop)| {
                let hash = Self::hash_property(prop);
                PropertyName::try_new(name.as_ref())
                    .ok()
                    .map(|pn| (pn, hash))
            })
            .collect()
    }

    /// Compute changed properties by comparing with new hashes.
    ///
    /// Returns property names that were:
    /// - Added (in new but not in current)
    /// - Removed (in current but not in new)
    /// - Modified (different hash)
    pub fn changed_properties(
        &self,
        new_hashes: &BTreeMap<PropertyName, [u8; 32]>,
    ) -> Vec<PropertyName> {
        let mut changed = Vec::new();

        // Find modified or added properties
        for (name, new_hash) in new_hashes {
            if self.property_hashes.get(name) != Some(new_hash) {
                changed.push(name.clone());
            }
        }

        // Find removed properties
        for name in self.property_hashes.keys() {
            if !new_hashes.contains_key(name) {
                changed.push(name.clone());
            }
        }

        changed
    }

    /// Hash a single property definition.
    fn hash_property(prop: &RawProperty) -> [u8; 32] {
        let mut hasher = blake3::Hasher::new();

        // Hash the property spec
        hasher.update(prop.spec.to_string().as_bytes());

        // Hash the multiplicity
        hasher.update(&[prop.multi as u8]);

        *hasher.finalize().as_bytes()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn file_metadata_matches_same_timestamps() {
        let now = SystemTime::now();
        let metadata = FileVersionMetadata::new(Some(now), Some(now));

        assert!(metadata.matches(Some(now), Some(now)));
    }

    #[test]
    fn file_metadata_no_match_different_timestamps() {
        let now = SystemTime::now();
        let later = now + std::time::Duration::from_secs(1);
        let metadata = FileVersionMetadata::new(Some(now), Some(now));

        assert!(!metadata.matches(Some(later), Some(now)));
    }

    #[test]
    fn hash_metadata_content_matches() {
        let hash = [1u8; 32];
        let metadata = HashMetadata::new(hash, BTreeMap::new());

        assert!(metadata.content_matches(&hash));
        assert!(!metadata.content_matches(&[2u8; 32]));
    }

    #[test]
    fn hash_metadata_changed_properties_detects_added() {
        let current = HashMetadata::new([0u8; 32], BTreeMap::new());
        let mut new_hashes = BTreeMap::new();
        let prop_name = PropertyName::try_new("title").unwrap();
        new_hashes.insert(prop_name.clone(), [1u8; 32]);

        let changed = current.changed_properties(&new_hashes);

        assert_eq!(changed.len(), 1);
        assert_eq!(changed[0], prop_name);
    }

    #[test]
    fn hash_metadata_changed_properties_detects_removed() {
        let mut current_hashes = BTreeMap::new();
        let prop_name = PropertyName::try_new("title").unwrap();
        current_hashes.insert(prop_name.clone(), [1u8; 32]);
        let current = HashMetadata::new([0u8; 32], current_hashes);

        let changed = current.changed_properties(&BTreeMap::new());

        assert_eq!(changed.len(), 1);
        assert_eq!(changed[0], prop_name);
    }

    #[test]
    fn hash_metadata_changed_properties_detects_modified() {
        let prop_name = PropertyName::try_new("title").unwrap();
        let mut current_hashes = BTreeMap::new();
        current_hashes.insert(prop_name.clone(), [1u8; 32]);
        let current = HashMetadata::new([0u8; 32], current_hashes);

        let mut new_hashes = BTreeMap::new();
        new_hashes.insert(prop_name.clone(), [2u8; 32]);

        let changed = current.changed_properties(&new_hashes);

        assert_eq!(changed.len(), 1);
        assert_eq!(changed[0], prop_name);
    }
}
```

**Tasks**:
1. Create `lithos-core/src/schema/views/metadata.rs`
2. Add module declaration in `lithos-core/src/schema/views/mod.rs`
3. Write unit tests for both metadata types
4. Run `cargo test --lib -p lithos-core views::metadata`
5. Commit: `feat(schema): add shared metadata types for version tracking`

---

#### Phase 1.2: Create SchemaVersion Type

**File**: `lithos-core/src/schema/views/version.rs` (new file)

```rust
//! Version types for schema and property bank views.

use std::collections::{HashMap, VecDeque};

use rkyv::{Archive, Deserialize, Serialize};

use crate::schema::{
    aggregate::SchemaName,
    error::SchemaIngestionError,
    property::{Property, PropertyName},
    raw::{RawPropertyBank, RawSchema},
};

use super::metadata::{FileVersionMetadata, HashMetadata};

// ─────────────────────────────────────────────────────────────────────────────
//  SchemaVersion
// ─────────────────────────────────────────────────────────────────────────────

/// A single version of a schema file with cached data.
///
/// Stores:
/// - File and hash metadata for staleness detection
/// - Serialized RawSchema for fast reconstruction
/// - Cached expanded properties for incremental resolution
#[derive(Debug, Clone, PartialEq, Archive, Serialize, Deserialize)]
pub struct SchemaVersion {
    /// File timestamp metadata
    file_metadata: FileVersionMetadata,

    /// Hash metadata for staleness detection
    hash_metadata: HashMetadata,

    /// Serialized RawSchema (rkyv format, optionally compressed)
    archived_schema: Vec<u8>,

    /// Cached expanded properties (from RefExpander)
    /// Enables skipping expansion when PropertyBank is fresh
    expanded_properties: Option<HashMap<PropertyName, Property>>,
}

impl SchemaVersion {
    /// Create a new schema version.
    ///
    /// # Parameters
    /// - `file_metadata`: File timestamp metadata
    /// - `hash_metadata`: Content and property hashes
    /// - `raw`: The RawSchema to serialize and cache
    ///
    /// # Errors
    /// Returns error if serialization fails.
    pub fn new(
        file_metadata: FileVersionMetadata,
        hash_metadata: HashMetadata,
        raw: &RawSchema,
    ) -> Result<Self, SchemaIngestionError> {
        let archived_schema = rkyv::to_bytes::<_, 256>(raw)
            .map_err(|e| SchemaIngestionError::Io {
                path: format!("schema {}", raw.name).into(),
                reason: format!("rkyv serialization failed: {e}").into(),
            })?
            .to_vec();

        Ok(Self {
            file_metadata,
            hash_metadata,
            archived_schema,
            expanded_properties: None,
        })
    }

    /// Get file metadata.
    #[inline]
    #[must_use]
    pub fn file_metadata(&self) -> &FileVersionMetadata {
        &self.file_metadata
    }

    /// Get hash metadata.
    #[inline]
    #[must_use]
    pub fn hash_metadata(&self) -> &HashMetadata {
        &self.hash_metadata
    }

    /// Get cached expanded properties if available.
    #[inline]
    #[must_use]
    pub fn expanded_properties(&self) -> Option<&HashMap<PropertyName, Property>> {
        self.expanded_properties.as_ref()
    }

    /// Set cached expanded properties.
    ///
    /// Called after RefExpander processes the schema.
    #[inline]
    pub fn set_expanded_properties(
        &mut self,
        properties: HashMap<PropertyName, Property>,
    ) {
        self.expanded_properties = Some(properties);
    }

    /// Deserialize cached RawSchema.
    ///
    /// # Errors
    /// Returns error if deserialization fails.
    pub fn to_raw(&self) -> Result<RawSchema, SchemaIngestionError> {
        rkyv::from_bytes::<RawSchema>(&self.archived_schema).map_err(|e| {
            SchemaIngestionError::Io {
                path: "cached schema".into(),
                reason: format!("rkyv deserialization failed: {e}").into(),
            }
        })
    }

    /// Check if this version is fresh (matches file metadata).
    #[inline]
    #[must_use]
    pub fn is_fresh(
        &self,
        created_at: Option<std::time::SystemTime>,
        modified_at: Option<std::time::SystemTime>,
    ) -> bool {
        self.file_metadata.matches(created_at, modified_at)
    }

    /// Check if content matches (for hash-based staleness detection).
    #[inline]
    #[must_use]
    pub fn content_matches(&self, hash: &[u8; 32]) -> bool {
        self.hash_metadata.content_matches(hash)
    }
}

// ─────────────────────────────────────────────────────────────────────────────
//  PropertyBankVersion
// ─────────────────────────────────────────────────────────────────────────────

/// A single version of the property bank file with cached data.
///
/// Stores:
/// - File and hash metadata for staleness detection
/// - Serialized RawPropertyBank for fast reconstruction
#[derive(Debug, Clone, PartialEq, Archive, Serialize, Deserialize)]
pub struct PropertyBankVersion {
    /// File timestamp metadata
    file_metadata: FileVersionMetadata,

    /// Hash metadata for staleness detection
    hash_metadata: HashMetadata,

    /// Serialized RawPropertyBank (rkyv format, optionally compressed)
    archived_property_bank: Vec<u8>,
}

impl PropertyBankVersion {
    /// Create a new property bank version.
    ///
    /// # Errors
    /// Returns error if serialization fails.
    pub fn new(
        file_metadata: FileVersionMetadata,
        hash_metadata: HashMetadata,
        raw: &RawPropertyBank,
    ) -> Result<Self, SchemaIngestionError> {
        let archived_property_bank = rkyv::to_bytes::<_, 256>(raw)
            .map_err(|e| SchemaIngestionError::Io {
                path: "property bank".into(),
                reason: format!("rkyv serialization failed: {e}").into(),
            })?
            .to_vec();

        Ok(Self {
            file_metadata,
            hash_metadata,
            archived_property_bank,
        })
    }

    /// Get file metadata.
    #[inline]
    #[must_use]
    pub fn file_metadata(&self) -> &FileVersionMetadata {
        &self.file_metadata
    }

    /// Get hash metadata.
    #[inline]
    #[must_use]
    pub fn hash_metadata(&self) -> &HashMetadata {
        &self.hash_metadata
    }

    /// Deserialize cached RawPropertyBank.
    ///
    /// # Errors
    /// Returns error if deserialization fails.
    pub fn to_raw(&self) -> Result<RawPropertyBank, SchemaIngestionError> {
        rkyv::from_bytes::<RawPropertyBank>(&self.archived_property_bank)
            .map_err(|e| SchemaIngestionError::Io {
                path: "cached property bank".into(),
                reason: format!("rkyv deserialization failed: {e}").into(),
            })
    }

    /// Check if this version is fresh (matches file metadata).
    #[inline]
    #[must_use]
    pub fn is_fresh(
        &self,
        created_at: Option<std::time::SystemTime>,
        modified_at: Option<std::time::SystemTime>,
    ) -> bool {
        self.file_metadata.matches(created_at, modified_at)
    }

    /// Check if content matches (for hash-based staleness detection).
    #[inline]
    #[must_use]
    pub fn content_matches(&self, hash: &[u8; 32]) -> bool {
        self.hash_metadata.content_matches(hash)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::SystemTime;

    fn create_test_raw_schema() -> RawSchema {
        RawSchema {
            name: "test".into(),
            extends: None,
            excludes: Vec::new(),
            properties: HashMap::new(),
        }
    }

    fn create_test_raw_property_bank() -> RawPropertyBank {
        RawPropertyBank {
            properties: HashMap::new(),
        }
    }

    #[test]
    fn schema_version_round_trip() {
        let raw = create_test_raw_schema();
        let file_meta = FileVersionMetadata::new(Some(SystemTime::now()), Some(SystemTime::now()));
        let hash_meta = HashMetadata::new([1u8; 32], Default::default());

        let version = SchemaVersion::new(file_meta, hash_meta, &raw).unwrap();
        let deserialized = version.to_raw().unwrap();

        assert_eq!(deserialized.name, raw.name);
    }

    #[test]
    fn property_bank_version_round_trip() {
        let raw = create_test_raw_property_bank();
        let file_meta = FileVersionMetadata::new(Some(SystemTime::now()), Some(SystemTime::now()));
        let hash_meta = HashMetadata::new([1u8; 32], Default::default());

        let version = PropertyBankVersion::new(file_meta, hash_meta, &raw).unwrap();
        let deserialized = version.to_raw().unwrap();

        assert_eq!(deserialized.properties.len(), 0);
    }

    #[test]
    fn schema_version_expanded_properties() {
        let raw = create_test_raw_schema();
        let file_meta = FileVersionMetadata::new(Some(SystemTime::now()), Some(SystemTime::now()));
        let hash_meta = HashMetadata::new([1u8; 32], Default::default());

        let mut version = SchemaVersion::new(file_meta, hash_meta, &raw).unwrap();

        assert!(version.expanded_properties().is_none());

        let expanded = HashMap::new();
        version.set_expanded_properties(expanded);

        assert!(version.expanded_properties().is_some());
    }
}
```

**Tasks**:
1. Create `lithos-core/src/schema/views/version.rs`
2. Add module declaration and re-exports in `lithos-core/src/schema/views/mod.rs`
3. Write unit tests for both version types
4. Test serialization/deserialization round-trips
5. Run `cargo test --lib -p lithos-core views::version`
6. Commit: `feat(schema): add SchemaVersion and PropertyBankVersion types`

---

#### Phase 1.3: Update RawSchemaView and RawPropertyBankView

**Goal**: Migrate from `VecDeque<RawFileVersion>` to new version types

**File**: `lithos-core/src/schema/views/raw.rs`

**Changes**:

```rust
// OLD:
pub struct RawSchemaView {
    file_path: Box<str>,
    extends: Option<SchemaName>,
    excludes: Vec<PropertyName>,
    versions: VecDeque<RawFileVersion>,  // ← OLD
}

// NEW:
pub struct RawSchemaView {
    file_path: Box<str>,
    extends: Option<SchemaName>,
    excludes: Vec<PropertyName>,
    versions: VecDeque<SchemaVersion>,  // ← NEW
}

// Remove all delegating methods and use version methods directly
impl RawSchemaView {
    // Remove: is_fresh(), is_timestamp_match(), is_content_match(), etc.
    // Keep: current(), versions(), add_version(), to_raw()

    /// Get current version.
    pub fn current(&self) -> Option<&SchemaVersion> {
        self.versions.front()
    }

    /// Check if current version is fresh.
    pub fn is_fresh(
        &self,
        created_at: Option<SystemTime>,
        modified_at: Option<SystemTime>,
    ) -> bool {
        self.current()
            .is_some_and(|v| v.is_fresh(created_at, modified_at))
    }

    /// Get changed properties.
    pub fn changed_properties(
        &self,
        new_hashes: &BTreeMap<PropertyName, [u8; 32]>,
    ) -> Vec<PropertyName> {
        self.current()
            .map(|v| v.hash_metadata().changed_properties(new_hashes))
            .unwrap_or_default()
    }

    /// Deserialize current RawSchema.
    pub fn to_raw(&self) -> Option<RawSchema> {
        self.current()?.to_raw().ok()
    }

    /// Add a new version.
    pub fn add_version(&mut self, version: SchemaVersion) {
        if self.versions.len() >= MAX_VERSIONS {
            self.versions.pop_back();
        }
        self.versions.push_front(version);
    }
}
```

**Migration Strategy**:
1. Add new field alongside old field temporarily
2. Update constructors to populate both
3. Update all readers to use new field
4. Update all writers to use new field
5. Remove old field

**Tasks**:
1. Add `SchemaVersion` field to `RawSchemaView` (alongside existing)
2. Add `PropertyBankVersion` field to `RawPropertyBankView` (alongside existing)
3. Update `new()` constructors to create new version types
4. Update all access methods to use new types
5. Update tests to use new types
6. Remove old `versions: VecDeque<RawFileVersion>` field
7. Run `cargo test --lib -p lithos-core`
8. Commit: `refactor(schema): migrate views to new version types`

---

#### Phase 1.4: Remove RawSchemaMetadata

**Goal**: Remove `RawSchemaMetadata` from `RawSchema` and `RawPropertyBank`

**Files**:
- `lithos-core/src/schema/raw/mod.rs`
- `lithos-core/src/schema/raw/property.rs`

**Changes**:

```rust
// OLD RawSchema:
pub struct RawSchema {
    pub name: Box<str>,
    pub extends: Option<Box<str>>,
    pub excludes: Vec<Box<str>>,
    pub properties: HashMap<Box<str>, RawProperty>,
    pub metadata: RawSchemaMetadata,  // ← REMOVE
}

// NEW RawSchema:
pub struct RawSchema {
    pub name: Box<str>,
    pub extends: Option<Box<str>>,
    pub excludes: Vec<Box<str>>,
    pub properties: HashMap<Box<str>, RawProperty>,
    // NO metadata field!
}

// Delete RawSchemaMetadata struct entirely
```

**Impact Analysis**:
- Ingestor: Compute hashes on-the-fly, pass to `HashMetadata::new()`
- Views: No longer extract metadata from RawSchema
- Tests: Update to not expect metadata field

**Tasks**:
1. Remove `metadata` field from `RawSchema`
2. Remove `metadata` field from `RawPropertyBank`
3. Delete `RawSchemaMetadata` struct definition
4. Update ingestor to compute hashes directly
5. Update all tests that access `.metadata`
6. Run `cargo test --lib -p lithos-core`
7. Commit: `refactor(schema): remove RawSchemaMetadata - hashes now in versions`

---

### Phase 2: PropertyBank Incremental Updates
**Goal**: Return `PropertyBank` directly with incremental update support
**Duration**: 2-3 days
**Risk**: Medium - changes public API contracts

#### Phase 2.1: Create PropertyBankResult Type

**File**: `lithos-core/src/schema/bank.rs`

```rust
/// Result of property bank ingestion with staleness information.
#[derive(Debug)]
pub enum PropertyBankResult {
    /// Property bank file is new (first time seeing it)
    New(PropertyBank),

    /// Property bank file unchanged - loaded from database
    Fresh(PropertyBank),

    /// Property bank file changed - updated incrementally
    Stale {
        /// The updated property bank
        bank: PropertyBank,
        /// Properties that changed (for incremental resolution)
        changed: Vec<PropertyName>,
    },
}

impl PropertyBankResult {
    /// Get the property bank regardless of variant.
    #[must_use]
    pub fn bank(&self) -> &PropertyBank {
        match self {
            Self::New(bank) | Self::Fresh(bank) => bank,
            Self::Stale { bank, .. } => bank,
        }
    }

    /// Get the property bank, consuming self.
    #[must_use]
    pub fn into_bank(self) -> PropertyBank {
        match self {
            Self::New(bank) | Self::Fresh(bank) => bank,
            Self::Stale { bank, .. } => bank,
        }
    }

    /// Get changed properties if stale, otherwise empty vec.
    #[must_use]
    pub fn changed_properties(&self) -> &[PropertyName] {
        match self {
            Self::Stale { changed, .. } => changed,
            _ => &[],
        }
    }

    /// Check if the bank is fresh.
    #[must_use]
    pub fn is_fresh(&self) -> bool {
        matches!(self, Self::Fresh(_))
    }

    /// Check if the bank is new.
    #[must_use]
    pub fn is_new(&self) -> bool {
        matches!(self, Self::New(_))
    }

    /// Check if the bank is stale.
    #[must_use]
    pub fn is_stale(&self) -> bool {
        matches!(self, Self::Stale { .. })
    }
}
```

**Tasks**:
1. Add `PropertyBankResult` enum to `bank.rs`
2. Add convenience methods
3. Write unit tests
4. Commit: `feat(schema): add PropertyBankResult for incremental updates`

---

#### Phase 2.2: Add Incremental Update Method to PropertyBank

**File**: `lithos-core/src/schema/bank.rs`

```rust
impl PropertyBank {
    /// Update properties incrementally from raw property bank.
    ///
    /// Only processes the properties specified in `changed_names`.
    /// More efficient than rebuilding entire bank from scratch.
    ///
    /// # Errors
    /// Returns error if property validation fails.
    pub fn update_from_raw(
        &mut self,
        raw: &RawPropertyBank,
        changed_names: &[PropertyName],
    ) -> Result<(), PropertyBankError> {
        // Build update list from changed properties
        let updates: Vec<_> = changed_names
            .iter()
            .filter_map(|name| {
                raw.properties
                    .get(name.as_ref())
                    .map(|raw_prop| (name.clone(), raw_prop.clone()))
            })
            .collect();

        // Use existing update_properties method
        self.update_properties(&updates)
    }

    /// Create PropertyBank from RawPropertyBank (full conversion).
    ///
    /// Use this for new property banks. For updates, use `update_from_raw`.
    pub fn from_raw(raw: RawPropertyBank) -> Result<Self, PropertyBankError> {
        Self::try_from(raw)
    }
}
```

**Tasks**:
1. Add `update_from_raw()` method
2. Add `from_raw()` convenience method
3. Write unit tests for incremental updates
4. Test that unchanged properties aren't affected
5. Commit: `feat(schema): add PropertyBank::update_from_raw for incremental updates`

---

#### Phase 2.3: Update Ingestor to Return PropertyBankResult

**File**: `lithos-core/src/schema/ingestor.rs`

```rust
impl Ingestor {
    /// Ingest property bank with staleness detection.
    ///
    /// Returns PropertyBankResult indicating if the bank is:
    /// - New: First time seeing property bank
    /// - Fresh: File unchanged, loaded from DB
    /// - Stale: File changed, updated incrementally
    pub fn property_bank(&self) -> Result<PropertyBankResult, SchemaIngestionError> {
        let path = self.config.paths().property_bank_path();

        // Get file metadata
        let created_at = self.source.created_at(&path);
        let modified_at = self.source.modified_at(&path);

        // Try to load cached version
        let cached_version = self.repository
            .get_raw_property_bank_view()?
            .and_then(|view| view.current().cloned());

        // Case 1: No cached version - this is a NEW property bank
        let Some(cached) = cached_version else {
            return self.ingest_new_property_bank(&path, created_at, modified_at);
        };

        // Case 2: Check if FRESH (timestamps match)
        if cached.is_fresh(created_at, modified_at) {
            let bank = self.repository.get_property_bank()?
                .ok_or_else(|| SchemaIngestionError::Io {
                    path: path.to_string_lossy().into(),
                    reason: "PropertyBank missing from DB but view exists".into(),
                })?;
            return Ok(PropertyBankResult::Fresh(bank));
        }

        // Case 3: STALE - file changed, compute incremental update
        self.ingest_stale_property_bank(&path, created_at, modified_at, &cached)
    }

    fn ingest_new_property_bank(
        &self,
        path: &Path,
        created_at: Option<SystemTime>,
        modified_at: Option<SystemTime>,
    ) -> Result<PropertyBankResult, SchemaIngestionError> {
        let content = self.source.read_to_string(path)?;
        let content_hash = blake3::hash(content.as_bytes());

        let raw: RawPropertyBank = FsReader::parse_structured_from_str(path, &content)?;
        let raw = raw.validated(&path.to_string_lossy())?;

        // Compute hashes
        let property_hashes = HashMetadata::compute_property_hashes(&raw.properties);

        // Create version
        let file_meta = FileVersionMetadata::new(created_at, modified_at);
        let hash_meta = HashMetadata::new(*content_hash.as_bytes(), property_hashes);
        let version = PropertyBankVersion::new(file_meta, hash_meta, &raw)?;

        // Save view
        let mut view = RawPropertyBankView::new();
        view.add_version(version);
        self.repository.save_raw_property_bank_view(&view)?;

        // Create PropertyBank
        let bank = PropertyBank::from_raw(raw)?;
        self.repository.save_property_bank(&bank)?;

        Ok(PropertyBankResult::New(bank))
    }

    fn ingest_stale_property_bank(
        &self,
        path: &Path,
        created_at: Option<SystemTime>,
        modified_at: Option<SystemTime>,
        cached: &PropertyBankVersion,
    ) -> Result<PropertyBankResult, SchemaIngestionError> {
        let content = self.source.read_to_string(path)?;
        let content_hash = blake3::hash(content.as_bytes());

        // Parse new version
        let raw: RawPropertyBank = FsReader::parse_structured_from_str(path, &content)?;
        let raw = raw.validated(&path.to_string_lossy())?;

        // Compute new hashes
        let new_property_hashes = HashMetadata::compute_property_hashes(&raw.properties);

        // Find changed properties
        let changed = cached.hash_metadata().changed_properties(&new_property_hashes);

        // Create new version
        let file_meta = FileVersionMetadata::new(created_at, modified_at);
        let hash_meta = HashMetadata::new(*content_hash.as_bytes(), new_property_hashes);
        let version = PropertyBankVersion::new(file_meta, hash_meta, &raw)?;

        // Update view
        let mut view = self.repository.get_raw_property_bank_view()?.unwrap();
        view.add_version(version);
        self.repository.save_raw_property_bank_view(&view)?;

        // Update PropertyBank incrementally
        let mut bank = self.repository.get_property_bank()?.unwrap_or_default();
        bank.update_from_raw(&raw, &changed)?;
        self.repository.save_property_bank(&bank)?;

        Ok(PropertyBankResult::Stale { bank, changed })
    }
}
```

**Tasks**:
1. Refactor `property_bank()` to return `PropertyBankResult`
2. Extract helper methods for New/Fresh/Stale cases
3. Update all callers in loader
4. Update tests
5. Run `cargo test --lib -p lithos-core`
6. Commit: `refactor(schema): ingestor returns PropertyBankResult`

---

### Phase 3: Structured Ingestor Results
**Goal**: Eliminate double-loop and N+1 queries
**Duration**: 3-4 days
**Risk**: High - significant architectural change

#### Phase 3.1: Create IngestorResults Types

**File**: `lithos-core/src/schema/ingestor.rs`

```rust
use std::path::PathBuf;
use std::collections::HashMap;

/// Results of ingesting all schemas and property bank.
#[derive(Debug)]
pub struct IngestorResults {
    /// Property bank result with staleness information
    pub property_bank: PropertyBankResult,

    /// Schema ingestion results by file path
    pub schemas: HashMap<PathBuf, SchemaIngestResult>,
}

/// Result of ingesting a single schema file.
#[derive(Debug)]
pub enum SchemaIngestResult {
    /// Schema file unchanged - can use cached data
    Fresh {
        /// Schema ID (from database)
        id: SchemaId,

        /// Cached expanded properties (if available)
        /// Enables skipping RefExpander when PropertyBank is fresh
        expanded: Option<HashMap<PropertyName, Property>>,
    },

    /// Schema file changed or new - needs processing
    Stale {
        /// Schema ID (from database or newly generated)
        id: SchemaId,

        /// Parsed raw schema
        raw: RawSchema,

        /// Cached expanded properties (if available and still valid)
        expanded: Option<HashMap<PropertyName, Property>>,
    },
}

impl SchemaIngestResult {
    /// Get the schema ID.
    #[must_use]
    pub fn id(&self) -> SchemaId {
        match self {
            Self::Fresh { id, .. } | Self::Stale { id, .. } => *id,
        }
    }

    /// Check if this result is fresh.
    #[must_use]
    pub fn is_fresh(&self) -> bool {
        matches!(self, Self::Fresh { .. })
    }

    /// Check if this result is stale.
    #[must_use]
    pub fn is_stale(&self) -> bool {
        matches!(self, Self::Stale { .. })
    }

    /// Get expanded properties if available.
    #[must_use]
    pub fn expanded(&self) -> Option<&HashMap<PropertyName, Property>> {
        match self {
            Self::Fresh { expanded, .. } | Self::Stale { expanded, .. } => {
                expanded.as_ref()
            }
        }
    }
}
```

**Tasks**:
1. Add `IngestorResults` struct
2. Add `SchemaIngestResult` enum
3. Add convenience methods
4. Write unit tests
5. Commit: `feat(schema): add IngestorResults for structured ingestion`

---

#### Phase 3.2: Add Bulk Query Methods to Repository Trait

**File**: `lithos-core/src/schema/storage.rs`

```rust
pub trait Repository {
    // ... existing methods ...

    /// Find multiple raw schema views by paths (bulk query).
    ///
    /// More efficient than N individual queries.
    fn find_raw_schema_views_by_paths(
        &self,
        paths: &[PathBuf],
    ) -> Result<HashMap<PathBuf, RawSchemaView>, Self::Error>;

    /// Find multiple schema IDs by paths (bulk query).
    ///
    /// Returns map of path → SchemaId for schemas that exist.
    fn find_schema_ids_by_paths(
        &self,
        paths: &[PathBuf],
    ) -> Result<HashMap<PathBuf, SchemaId>, Self::Error>;
}
```

**Implementation in RedbRepository**:

```rust
impl Repository for RedbRepository {
    fn find_raw_schema_views_by_paths(
        &self,
        paths: &[PathBuf],
    ) -> Result<HashMap<PathBuf, RawSchemaView>, Self::Error> {
        let db = self.db.read().map_err(|_| /* ... */)?;
        let txn = db.begin_read()?;
        let table = txn.open_table(SCHEMA_VIEWS_TABLE)?;

        let mut results = HashMap::new();
        for path in paths {
            let key = path.to_string_lossy();
            if let Some(guard) = table.get(key.as_ref())? {
                let view: RawSchemaView = rkyv::from_bytes(guard.value())?;
                results.insert(path.clone(), view);
            }
        }

        Ok(results)
    }

    fn find_schema_ids_by_paths(
        &self,
        paths: &[PathBuf],
    ) -> Result<HashMap<PathBuf, SchemaId>, Self::Error> {
        let db = self.db.read().map_err(|_| /* ... */)?;
        let txn = db.begin_read()?;
        let table = txn.open_table(SCHEMA_PATH_TO_ID_TABLE)?;

        let mut results = HashMap::new();
        for path in paths {
            let key = path.to_string_lossy();
            if let Some(id_bytes) = table.get(key.as_ref())? {
                let id = SchemaId::from_bytes(id_bytes.value());
                results.insert(path.clone(), id);
            }
        }

        Ok(results)
    }
}
```

**Tasks**:
1. Add bulk query methods to `Repository` trait
2. Implement in `RedbRepository`
3. Implement in `InMemoryRepository` (for tests)
4. Implement in `FakeRepository` (for tests)
5. Write tests for bulk queries
6. Commit: `feat(schema): add bulk query methods to Repository trait`

---

#### Phase 3.3: Refactor Ingestor to Use Bulk Queries

**File**: `lithos-core/src/schema/ingestor.rs`

```rust
impl Ingestor {
    /// Ingest all schemas and property bank.
    ///
    /// Returns structured results with pre-partitioned Fresh/Stale schemas.
    /// Eliminates double-loop by partitioning during ingestion.
    pub fn ingest_all(&self) -> Result<IngestorResults, SchemaIngestionError> {
        // Step 1: Ingest property bank
        let property_bank = self.property_bank()?;

        // Step 2: List all schema files
        let paths = self.list_all_schema_files()?;

        // Step 3: Bulk queries (NO N+1!)
        let views = self.repository.find_raw_schema_views_by_paths(&paths)?;
        let ids = self.repository.find_schema_ids_by_paths(&paths)?;

        // Step 4: Process each schema (single loop!)
        let mut schemas = HashMap::new();

        for path in paths {
            let view = views.get(&path);
            let id = ids.get(&path).copied().unwrap_or_else(SchemaId::new);

            let result = self.process_schema(&path, id, view)?;
            schemas.insert(path, result);
        }

        Ok(IngestorResults {
            property_bank,
            schemas,
        })
    }

    fn list_all_schema_files(&self) -> Result<Vec<PathBuf>, SchemaIngestionError> {
        let paths = self.config.paths();
        let schemas_dir = paths.schema.schemas_dir().as_path();
        let property_bank_filename = paths.property_bank.as_str();

        let mut all_paths = Vec::new();

        for ext in SCHEMA_EXTENSIONS {
            let pattern = format!("{}/**/*.{}", schemas_dir.display(), ext);
            let files = self.source.list_files(&pattern)?;

            for path in files {
                // Exclude property bank file
                if path.file_name().is_some_and(|n| n == property_bank_filename) {
                    continue;
                }
                all_paths.push(path);
            }
        }

        Ok(all_paths)
    }

    fn process_schema(
        &self,
        path: &Path,
        id: SchemaId,
        view: Option<&RawSchemaView>,
    ) -> Result<SchemaIngestResult, SchemaIngestionError> {
        let created_at = self.source.created_at(path);
        let modified_at = self.source.modified_at(path);

        // Check if fresh
        if let Some(view) = view
            && view.is_fresh(created_at, modified_at)
        {
            let expanded = view.current()
                .and_then(|v| v.expanded_properties().cloned());

            return Ok(SchemaIngestResult::Fresh { id, expanded });
        }

        // Stale or new - read and parse file
        let raw = self.parse_schema_file(path, created_at, modified_at)?;

        // Create and save new version
        let version = self.create_schema_version(&raw, created_at, modified_at)?;
        self.save_schema_view(path, version)?;

        // Get expanded properties from view if still valid
        let expanded = view
            .and_then(|v| v.current())
            .and_then(|v| v.expanded_properties().cloned());

        Ok(SchemaIngestResult::Stale {
            id,
            raw,
            expanded,
        })
    }
}
```

**Tasks**:
1. Add `ingest_all()` method to `Ingestor`
2. Add `list_all_schema_files()` helper
3. Add `process_schema()` helper
4. Keep old `all_schemas()` method for backward compatibility (deprecated)
5. Update tests
6. Run `cargo test --lib -p lithos-core`
7. Commit: `refactor(schema): ingestor uses bulk queries and returns structured results`

---

### Phase 4: Update Loader to Use IngestorResults
**Goal**: Simplify loader using structured results
**Duration**: 2-3 days
**Risk**: Medium - changes loader orchestration

#### Phase 4.1: Refactor Loader.load() Method

**File**: `lithos-core/src/schema/loader.rs`

```rust
impl Loader {
    /// Load all schemas with incremental resolution.
    ///
    /// Uses structured ingestor results to eliminate double-loop
    /// and enable incremental resolution when PropertyBank is fresh.
    pub fn load(&self) -> Result<Vec<Schema>, SchemaLoaderError> {
        // Single ingestion call (no double loop!)
        let results = self.ingestor.ingest_all()?;

        // Extract property bank
        let bank = results.property_bank.into_bank();

        // Partition schemas based on PropertyBank staleness
        let (schemas_to_resolve, fresh_schemas) =
            self.partition_for_resolution(&results, &bank)?;

        // Resolve only what's needed
        let mut all_schemas = fresh_schemas;
        if !schemas_to_resolve.is_empty() {
            let resolved = self.resolve_schemas(schemas_to_resolve, &bank)?;
            all_schemas.extend(resolved);
        }

        Ok(all_schemas)
    }

    fn partition_for_resolution(
        &self,
        results: &IngestorResults,
        bank: &PropertyBank,
    ) -> Result<(Vec<(SchemaId, RawSchema)>, Vec<Schema>), SchemaLoaderError> {
        let mut to_resolve = Vec::new();
        let mut fresh = Vec::new();

        // If PropertyBank is fresh, we can use cached expanded schemas
        let bank_is_fresh = results.property_bank.is_fresh();

        for (path, result) in &results.schemas {
            match result {
                SchemaIngestResult::Fresh { id, expanded } if bank_is_fresh => {
                    // PropertyBank fresh + Schema fresh + has expanded = skip everything!
                    if let Some(expanded_props) = expanded {
                        // Load from DB (already fully resolved)
                        if let Some(schema) = self.load_schema_from_db(*id)? {
                            fresh.push(schema);
                            continue;
                        }
                    }
                    // Fall through to re-resolve if DB doesn't have it
                }

                SchemaIngestResult::Stale { id, raw, .. } => {
                    to_resolve.push((*id, raw.clone()));
                }

                _ => {
                    // Fresh schema but PropertyBank changed - need to re-resolve
                    // Load raw from view
                    // ...
                }
            }
        }

        Ok((to_resolve, fresh))
    }

    fn resolve_schemas(
        &self,
        schemas: Vec<(SchemaId, RawSchema)>,
        bank: &PropertyBank,
    ) -> Result<Vec<Schema>, SchemaLoaderError> {
        if schemas.is_empty() {
            return Ok(Vec::new());
        }

        // Full resolution pipeline
        let expanded = RefExpander::new(bank).expand_all(schemas)?;

        // TODO: Store expanded schemas here for next load

        let tree = Extender::build(expanded, &HashMap::new())?;
        let resolved = Resolver::resolve(&tree, &HashMap::new())?;

        Ok(resolved)
    }

    fn load_schema_from_db(
        &self,
        id: SchemaId,
    ) -> Result<Option<Schema>, SchemaLoaderError> {
        self.ingestor
            .repository()
            .find_schema_by_id(&id)
            .map_err(|e| SchemaLoaderError::Repository(e.into()))
    }
}
```

**Tasks**:
1. Refactor `load()` to use `ingest_all()`
2. Add `partition_for_resolution()` method
3. Add `resolve_schemas()` method
4. Add `load_schema_from_db()` helper
5. Remove old `load_property_bank()` method (now in ingestor)
6. Remove old double-loop partitioning logic
7. Update tests
8. Run `cargo test --lib -p lithos-core`
9. Commit: `refactor(schema): loader uses structured IngestorResults`

---

### Phase 5: Store Expanded Properties
**Goal**: Cache expanded schemas to enable incremental resolution
**Duration**: 2-3 days
**Risk**: Medium - adds new storage requirement

#### Phase 5.1: Update SchemaVersion After Expansion

**File**: `lithos-core/src/schema/loader.rs`

```rust
impl Loader {
    fn resolve_schemas(
        &self,
        schemas: Vec<(SchemaId, RawSchema)>,
        bank: &PropertyBank,
    ) -> Result<Vec<Schema>, SchemaLoaderError> {
        if schemas.is_empty() {
            return Ok(Vec::new());
        }

        // Expansion
        let expanded = RefExpander::new(bank).expand_all(schemas.clone())?;

        // STORE expanded properties in views
        self.store_expanded_properties(&schemas, &expanded)?;

        // Continue with resolution
        let tree = Extender::build(expanded, &HashMap::new())?;
        let resolved = Resolver::resolve(&tree, &HashMap::new())?;

        Ok(resolved)
    }

    fn store_expanded_properties(
        &self,
        schemas: &[(SchemaId, RawSchema)],
        expanded: &[RefExpandedSchema],
    ) -> Result<(), SchemaLoaderError> {
        // Create map of schema name → expanded properties
        let expanded_map: HashMap<_, _> = expanded
            .iter()
            .map(|exp| (exp.name.clone(), exp.properties.clone()))
            .collect();

        // Update each schema's view with expanded properties
        for (id, raw) in schemas {
            let Some(exp_props) = expanded_map.get(&SchemaName::try_new(&raw.name)?) else {
                continue;
            };

            // Load view, update current version, save
            let path = self.schema_id_to_path(*id)?;
            if let Some(mut view) = self.load_schema_view(&path)? {
                if let Some(current) = view.current_mut() {
                    current.set_expanded_properties(exp_props.clone());
                }
                self.save_schema_view(&path, view)?;
            }
        }

        Ok(())
    }
}
```

**Tasks**:
1. Add `store_expanded_properties()` method
2. Add `current_mut()` method to `RawSchemaView` (mutable access)
3. Call after expansion in `resolve_schemas()`
4. Update tests to verify expanded properties are stored
5. Commit: `feat(schema): cache expanded properties in schema views`

---

#### Phase 5.2: Use Cached Expanded Properties

**File**: `lithos-core/src/schema/loader.rs`

```rust
impl Loader {
    fn partition_for_resolution(
        &self,
        results: &IngestorResults,
        bank: &PropertyBank,
    ) -> Result<(Vec<(SchemaId, RawSchema)>, Vec<Schema>), SchemaLoaderError> {
        let mut to_expand = Vec::new();
        let mut with_expansion = Vec::new();
        let mut fresh = Vec::new();

        let bank_is_fresh = results.property_bank.is_fresh();

        for (path, result) in &results.schemas {
            match result {
                SchemaIngestResult::Fresh { id, expanded } if bank_is_fresh => {
                    // Best case: Schema fresh + PropertyBank fresh + has expanded
                    if expanded.is_some() {
                        // Load fully resolved schema from DB
                        if let Some(schema) = self.load_schema_from_db(*id)? {
                            fresh.push(schema);
                            continue;
                        }
                    }
                    // No expanded or not in DB - need to expand + resolve
                    let raw = self.load_raw_from_view(path)?;
                    to_expand.push((*id, raw));
                }

                SchemaIngestResult::Fresh { id, expanded } => {
                    // Schema fresh but PropertyBank changed
                    // Can we use cached expansion? Only if properties didn't change
                    let raw = self.load_raw_from_view(path)?;

                    if let Some(exp_props) = expanded
                        && !self.uses_changed_properties(&raw, &results.property_bank)?
                    {
                        // Cached expansion still valid! Skip to resolution
                        with_expansion.push((*id, raw, exp_props.clone()));
                    } else {
                        // Need to re-expand
                        to_expand.push((*id, raw));
                    }
                }

                SchemaIngestResult::Stale { id, raw, expanded } if bank_is_fresh => {
                    // Schema changed but PropertyBank fresh
                    // Can't use cached expansion (schema changed)
                    to_expand.push((*id, raw.clone()));
                }

                SchemaIngestResult::Stale { id, raw, .. } => {
                    // Both changed - need full re-expansion
                    to_expand.push((*id, raw.clone()));
                }
            }
        }

        // Resolve schemas with cached expansion (skip RefExpander!)
        if !with_expansion.is_empty() {
            let resolved = self.resolve_with_expansion(with_expansion)?;
            fresh.extend(resolved);
        }

        Ok((to_expand, fresh))
    }

    fn resolve_with_expansion(
        &self,
        schemas: Vec<(SchemaId, RawSchema, HashMap<PropertyName, Property>)>,
    ) -> Result<Vec<Schema>, SchemaLoaderError> {
        // Convert to RefExpandedSchema (skip RefExpander!)
        let expanded: Vec<_> = schemas
            .into_iter()
            .map(|(id, raw, props)| RefExpandedSchema {
                id,
                name: SchemaName::try_new(&raw.name).unwrap(),
                file_path: format!("schemas/{}.toml", raw.name).into(),
                extends: raw.extends.and_then(|e| SchemaName::try_new(&e).ok()),
                excludes: raw.excludes.iter()
                    .filter_map(|e| PropertyName::try_new(e).ok())
                    .collect(),
                properties: props,
            })
            .collect();

        // Go directly to Extender (skipped RefExpander!)
        let tree = Extender::build(expanded, &HashMap::new())?;
        let resolved = Resolver::resolve(&tree, &HashMap::new())?;

        Ok(resolved)
    }

    fn uses_changed_properties(
        &self,
        raw: &RawSchema,
        bank_result: &PropertyBankResult,
    ) -> Result<bool, SchemaLoaderError> {
        let changed = bank_result.changed_properties();
        if changed.is_empty() {
            return Ok(false);
        }

        // Check if schema uses any changed properties
        for prop in raw.properties.values() {
            // Check if property references any changed properties
            // (This requires parsing property spec for refs)
            // For now, conservatively assume it might use them
            // TODO: Implement proper ref dependency checking
        }

        Ok(true) // Conservative: assume it uses changed properties
    }
}
```

**Tasks**:
1. Update `partition_for_resolution()` to detect cached expansion
2. Add `resolve_with_expansion()` method (skips RefExpander)
3. Add `uses_changed_properties()` helper (conservative implementation)
4. Update tests to verify expansion is skipped
5. Add performance benchmarks
6. Commit: `feat(schema): use cached expanded properties to skip RefExpander`

---

### Phase 6: Testing and Validation
**Goal**: Comprehensive testing of new architecture
**Duration**: 2-3 days
**Risk**: Low - all about verification

#### Phase 6.1: Unit Tests

**Test Coverage Requirements**:

1. **Metadata Types** (`metadata.rs`):
   - ✅ FileVersionMetadata timestamp matching
   - ✅ HashMetadata content/property hash comparison
   - ✅ Changed property detection (added/removed/modified)
   - ✅ Property hash computation

2. **Version Types** (`version.rs`):
   - ✅ SchemaVersion serialization round-trip
   - ✅ PropertyBankVersion serialization round-trip
   - ✅ Expanded properties storage/retrieval
   - ✅ Staleness detection methods

3. **PropertyBank** (`bank.rs`):
   - ✅ PropertyBankResult variant accessors
   - ✅ Incremental update via `update_from_raw()`
   - ✅ Full conversion via `from_raw()`

4. **Ingestor** (`ingestor.rs`):
   - ✅ PropertyBank: New/Fresh/Stale detection
   - ✅ Schema: Fresh/Stale detection
   - ✅ Bulk queries return correct results
   - ✅ Single-loop partitioning

5. **Loader** (`loader.rs`):
   - ✅ Incremental resolution when PropertyBank fresh
   - ✅ Skip RefExpander when expansion cached
   - ✅ Full resolution when needed

**Task**: Write and run all unit tests
```bash
cargo test --lib -p lithos-core schema::views::metadata
cargo test --lib -p lithos-core schema::views::version
cargo test --lib -p lithos-core schema::bank
cargo test --lib -p lithos-core schema::ingestor
cargo test --lib -p lithos-core schema::loader
```

---

#### Phase 6.2: Integration Tests

**File**: `lithos-core/tests/schema_loading_integration_test.rs`

```rust
#[test]
fn fresh_property_bank_skips_expansion() {
    // Setup: Load schemas once
    let loader = setup_loader();
    let schemas1 = loader.load().unwrap();

    // Don't modify any files

    // Second load: PropertyBank fresh + Schemas fresh
    let schemas2 = loader.load().unwrap();

    // Verify: RefExpander was NOT called (check via tracing or metrics)
    // Verify: Schemas match
    assert_eq!(schemas1.len(), schemas2.len());
}

#[test]
fn property_bank_change_triggers_reexpansion() {
    let loader = setup_loader();
    let schemas1 = loader.load().unwrap();

    // Modify property bank
    modify_property_bank();

    // Second load: PropertyBank stale
    let schemas2 = loader.load().unwrap();

    // Verify: Only affected schemas re-expanded
    // Verify: Fresh schemas with no property refs skip expansion
}

#[test]
fn schema_change_triggers_reexpansion() {
    let loader = setup_loader();
    let schemas1 = loader.load().unwrap();

    // Modify one schema file
    modify_schema("task.toml");

    // Second load: One schema stale, others fresh
    let schemas2 = loader.load().unwrap();

    // Verify: Only modified schema re-expanded
    // Verify: Other schemas use cached expansion
}

#[test]
fn new_schema_added() {
    let loader = setup_loader();
    let schemas1 = loader.load().unwrap();

    // Add new schema file
    write_schema("event.toml", "...");

    // Second load: New schema detected
    let schemas2 = loader.load().unwrap();

    assert_eq!(schemas2.len(), schemas1.len() + 1);
}

#[test]
fn bulk_queries_avoid_n_plus_one() {
    let loader = setup_loader_with_metrics();

    // Load 100 schemas
    let schemas = loader.load().unwrap();

    // Verify: Only 2 DB queries total (views + IDs)
    // NOT 200 queries (2 per schema)
    let query_count = get_db_query_count();
    assert!(query_count < 10, "Expected <10 queries, got {}", query_count);
}
```

**Tasks**:
1. Write integration tests for each scenario
2. Add test fixtures for various schema configurations
3. Add metrics/tracing to verify optimization paths
4. Run integration tests
5. Commit: `test(schema): add integration tests for incremental loading`

---

#### Phase 6.3: Performance Benchmarks

**File**: `lithos-core/benches/schema_loading.rs`

```rust
use criterion::{black_box, criterion_group, criterion_main, Criterion};

fn bench_fresh_load_100_schemas(c: &mut Criterion) {
    let loader = setup_loader_with_100_schemas();

    // First load (populates cache)
    loader.load().unwrap();

    // Benchmark second load (all fresh)
    c.bench_function("load_100_fresh_schemas", |b| {
        b.iter(|| {
            loader.load().unwrap()
        })
    });
}

fn bench_fresh_load_with_expansion(c: &mut Criterion) {
    let loader = setup_loader_with_100_schemas();
    loader.load().unwrap();

    // Clear expanded properties cache
    clear_expanded_cache();

    c.bench_function("load_100_fresh_no_expansion_cache", |b| {
        b.iter(|| {
            loader.load().unwrap()
        })
    });
}

fn bench_stale_property_bank(c: &mut Criterion) {
    let loader = setup_loader_with_100_schemas();
    loader.load().unwrap();

    // Modify property bank
    modify_property_bank();

    c.bench_function("load_100_after_property_bank_change", |b| {
        b.iter(|| {
            loader.load().unwrap();
            // Reset for next iteration
            reset_property_bank();
        })
    });
}

criterion_group!(
    benches,
    bench_fresh_load_100_schemas,
    bench_fresh_load_with_expansion,
    bench_stale_property_bank
);
criterion_main!(benches);
```

**Tasks**:
1. Create benchmark suite
2. Run baseline benchmarks on old implementation
3. Run benchmarks on new implementation
4. Verify 80% improvement for fresh loads
5. Document results in commit message
6. Commit: `perf(schema): benchmark incremental loading improvements`

---

### Phase 7: Cleanup and Documentation
**Goal**: Remove deprecated code and update docs
**Duration**: 1-2 days
**Risk**: Low

#### Phase 7.1: Remove Deprecated Code

**Files to update**:
- Remove old `RawFileVersion` type
- Remove old `all_schemas()` method (replaced by `ingest_all()`)
- Remove old loader partitioning logic
- Remove `IngestResult<T>` if no longer used

**Tasks**:
1. Search for `RawFileVersion` usage
2. Remove type definition
3. Update changelog
4. Commit: `refactor(schema): remove deprecated RawFileVersion type`

---

#### Phase 7.2: Update Documentation

**Files to update**:
- `lithos-core/src/schema/loader.rs` - Update module docs
- `lithos-core/src/schema/ingestor.rs` - Update module docs
- `lithos-core/src/schema/views/mod.rs` - Update module docs
- `loader-ingestor-architecture-review.md` - Add "Implemented" status

**Documentation Updates**:

```rust
//! Schema loader — orchestrates the full schema ingestion pipeline.
//!
//! ## Orchestration Pattern (Updated)
//!
//! The loader coordinates the file → raw → resolved → database pipeline
//! with **incremental resolution** to minimize redundant processing.
//!
//! ## Pipeline Flow (Optimized)
//!
//! 1. **Single ingestion call**: `ingestor.ingest_all()`
//!    - Returns `IngestorResults` with pre-partitioned Fresh/Stale
//!    - Uses bulk DB queries (no N+1 pattern)
//!    - PropertyBank returned as domain type (not raw)
//!
//! 2. **Incremental resolution**:
//!    - PropertyBank fresh + Schema fresh + has cached expansion → Load from DB
//!    - PropertyBank fresh + Schema fresh + no expansion → Expand + Resolve
//!    - PropertyBank stale + Schema fresh → Check if properties used, maybe skip expansion
//!    - Either stale → Full re-expansion
//!
//! 3. **Performance optimizations**:
//!    - Cached expanded properties eliminate RefExpander calls
//!    - Serialized RawSchema (rkyv) faster than parsing
//!    - Bulk queries eliminate N+1 database pattern
//!    - Single-loop processing (no double iteration)
//!
//! **Expected improvement**: 80% reduction in load time for fresh schemas.
```

**Tasks**:
1. Update all module documentation
2. Add examples of new API usage
3. Update architecture review doc with "Implemented" sections
4. Commit: `docs(schema): update for incremental loading architecture`

---

## Migration Path

### For Existing Code

**Breaking Changes**:
1. `Ingestor::property_bank()` return type changed:
   - Old: `Result<IngestResult<RawPropertyBank>, Error>`
   - New: `Result<PropertyBankResult, Error>`

2. New recommended method:
   - Old: `Ingestor::all_schemas()`
   - New: `Ingestor::ingest_all()`

**Migration Example**:

```rust
// OLD CODE:
let bank_result = ingestor.property_bank()?;
let bank = match bank_result {
    IngestResult::Fresh(raw) | IngestResult::Stale(raw) => {
        PropertyBank::try_from(raw)?
    }
};

let schema_results = ingestor.all_schemas()?;
for result in schema_results {
    match result {
        IngestResult::Fresh(raw) => { /* ... */ }
        IngestResult::Stale(raw) => { /* ... */ }
    }
}

// NEW CODE:
let results = ingestor.ingest_all()?;
let bank = results.property_bank.into_bank();

for (path, result) in results.schemas {
    match result {
        SchemaIngestResult::Fresh { id, expanded } => { /* ... */ }
        SchemaIngestResult::Stale { id, raw, expanded } => { /* ... */ }
    }
}
```

---

## Rollback Plan

Each phase is independently committable. If issues arise:

1. **Phase 1**: Can be rolled back without affecting existing code (purely additive)
2. **Phase 2**: Affects PropertyBank loading - revert to `IngestResult<RawPropertyBank>`
3. **Phase 3**: Affects all schema loading - most complex rollback
4. **Phase 4-5**: Loader changes - can revert to old orchestration
5. **Phase 6-7**: Tests and docs - safe to rollback

**Rollback Strategy**:
- Each phase has clear commit message
- Use `git revert <commit>` to undo specific phase
- Keep deprecated methods until Phase 7 for easier rollback

---

## Success Criteria

### Functional Requirements
- ✅ All existing tests pass
- ✅ New tests for incremental resolution pass
- ✅ PropertyBank incremental updates work correctly
- ✅ Cached expanded properties enable skipping RefExpander
- ✅ Bulk queries eliminate N+1 pattern

### Performance Requirements
- ✅ Fresh schema load: 80% faster than baseline
- ✅ PropertyBank stale load: No regression
- ✅ Database queries: <10 total (not 200+ for 100 schemas)
- ✅ Memory usage: No significant increase

### Code Quality Requirements
- ✅ No clippy warnings
- ✅ All public APIs documented
- ✅ Integration tests cover key scenarios
- ✅ Benchmarks demonstrate improvement

---

## Timeline Estimate

| Phase | Duration | Dependencies | Risk |
|-------|----------|--------------|------|
| Phase 1: New version types | 2 days | None | Low |
| Phase 2: PropertyBank incremental | 3 days | Phase 1 | Medium |
| Phase 3: Structured results | 4 days | Phase 1, 2 | High |
| Phase 4: Update loader | 3 days | Phase 3 | Medium |
| Phase 5: Store expanded | 3 days | Phase 4 | Medium |
| Phase 6: Testing | 3 days | Phase 5 | Low |
| Phase 7: Cleanup | 2 days | Phase 6 | Low |
| **Total** | **20 days** (~4 weeks) | | |

**Notes**:
- Assumes full-time work
- Includes buffer for unexpected issues
- Each phase can be split across multiple days
- Phases 1-2 can be done in parallel by different developers

---

## Open Questions

### Q1: Compression Strategy

**Question**: Should we compress serialized structs?

**Options**:
- A: `rkyv(RawSchema)` - Fastest, larger size
- B: `zstd(rkyv(RawSchema))` - Slower, smaller size

**Decision Needed By**: Phase 1.2

**Recommendation**: Start with uncompressed (Option A), add compression in Phase 6 if benchmarks show disk space is an issue.

---

### Q2: Expanded Properties Invalidation

**Question**: How do we know if cached expanded properties are still valid when PropertyBank changes?

**Options**:
- A: Conservative - always re-expand when PropertyBank changes
- B: Smart - track which properties each schema uses, only re-expand if used properties changed
- C: Hybrid - Conservative for now, Smart in future optimization

**Decision Needed By**: Phase 5.2

**Recommendation**: Option C - Start conservative, add smart tracking later if profiling shows it's worth the complexity.

---

### Q3: Backward Compatibility

**Question**: Do we need to support reading old `RawFileVersion` data?

**Options**:
- A: Yes - Add migration code to convert old format
- B: No - Require cache rebuild (simpler)

**Decision Needed By**: Phase 1.3

**Recommendation**: Option B - Cache rebuild is acceptable (one-time cost, simpler code).

---

## Next Steps

1. ✅ Review this implementation plan
2. ✅ Approve architectural decisions
3. ✅ Decide on open questions (compression, invalidation, compatibility)
4. 📋 Create GitHub issues for each phase
5. 📋 Begin Phase 1.1: Create metadata types

---

## Related Documents

- `loader-ingestor-architecture-review.md` - Architectural analysis
- `lithos-core/src/schema/loader.rs` - Current loader implementation
- `lithos-core/src/schema/ingestor.rs` - Current ingestor implementation
- `lithos-core/src/schema/views/raw.rs` - Current view types
