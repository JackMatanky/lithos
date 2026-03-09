//! Storage types for schema system (read models and raw file versions).
//!
//! This module contains all storage-related types for the schema system,
//! organized into two main categories:
//!
//! ## Raw File Storage
//!
//! Versioned storage of source files (before parsing/resolution):
//! - [`RawFileVersion`] - Single version of a raw file with compression
//! - [`RawSchemaFile`] - Schema file with version history (up to 5 versions)
//! - [`RawPropertyBankFile`] - Property bank file with version history
//! - [`FileChange`] - Change detection for staleness checks
//!
//! ## Resolved Data Storage
//!
//! Storage representations of processed domain data (read model pattern):
//! - [`StoredSchema`] - Resolved schema read model
//! - [`StoredProperty`] - Property read model
//! - [`StoredMetadata`] - Schema/bank metadata for staleness tracking
//!
//! ## Read Model Architecture
//!
//! The "Stored*" types follow the **read model pattern**:
//! - **No behavior**: Only field accessors (getters)
//! - **No events**: Event emission happens in [`crate::schema::loader`]
//! - **No domain logic**: Pure data structures optimized for storage
//! - **Zero-copy reads**: Uses `rkyv` for fast deserialization-free access
//!
//! ## Storage Tables
//!
//! Schema storage:
//! - `schema_by_id` - Resolved schemas (rkyv-serialized `StoredSchema`)
//! - `schema_metadata` - Staleness metadata (`StoredMetadata`)
//! - `raw_schema_files` - Versioned raw files (`RawSchemaFile`)
//!
//! Property bank storage:
//! - `bank_metadata` - Version/timestamp tracking
//! - `bank_property_by_id` - ID-keyed property snapshots
//! - `bank_property_by_name` - Name-keyed property snapshots
//! - `raw_property_bank_file` - Versioned raw property bank file

// Clippy false positive: Archive macro generates internal types that trigger
// exhaustive_structs, but our public types are marked #[non_exhaustive].
// This cannot be fixed without changes to rkyv.
#![expect(
    clippy::exhaustive_structs,
    reason = "False positive from rkyv Archive macro - all public types use \
              #[non_exhaustive]"
)]

use std::{io::Read as _, time::SystemTime};

use rkyv::{
    Archive, Deserialize, Serialize,
    with::{AsUnixTime, Map},
};

use super::{
    bank::{BankVersion, PropertyBank},
    error::SchemaError,
    hash::Blake3Hash,
    id::SchemaId,
    property::{Multiplicity, Optionality, Property, PropertyId, PropertyName},
    property_spec::PropertySpec,
};

// ============================================================================
// Raw File Storage (Versioned Source Files)
// ============================================================================

// ─────────────────────────────────────────────────────────────────────────────
//  Compression Utilities
// ─────────────────────────────────────────────────────────────────────────────

/// Compression level (3 = balanced speed/ratio).
const COMPRESSION_LEVEL: i32 = 3;

/// Compress string content using zstd.
///
/// # Errors
/// Returns error if compression fails.
#[inline]
fn compress(content: &str) -> Result<Vec<u8>, std::io::Error> {
    zstd::encode_all(content.as_bytes(), COMPRESSION_LEVEL)
}

/// Decompress zstd data to string.
///
/// # Errors
/// Returns error if decompression fails or output is not UTF-8.
#[inline]
fn decompress(compressed: &[u8]) -> Result<String, DecompressionError> {
    let mut decompressed = Vec::new();
    zstd::Decoder::new(compressed)?.read_to_end(&mut decompressed)?;
    String::from_utf8(decompressed).map_err(DecompressionError::InvalidUtf8)
}

/// Decompression errors.
#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub enum DecompressionError {
    /// I/O error during decompression.
    #[error("decompression I/O error: {0}")]
    Io(#[from] std::io::Error),

    /// Decompressed data is not valid UTF-8.
    #[error("decompressed data is not valid UTF-8: {0}")]
    InvalidUtf8(#[from] std::string::FromUtf8Error),
}

// ─────────────────────────────────────────────────────────────────────────────
//  Ring Buffer (Fixed-Size Version History)
// ─────────────────────────────────────────────────────────────────────────────

/// Fixed-size ring buffer for versioned file storage (compile-time size, zero
/// allocation).
///
/// This ring buffer uses `u8` indices for memory efficiency (5 versions max).
/// All arithmetic and indexing is safe by design (modulo wraparound prevents
/// out-of-bounds).
#[derive(Debug, Clone, PartialEq, Archive, Serialize, Deserialize)]
struct RingBuffer<T, const N: usize> {
    items: [Option<T>; N],
    head: u8, // Next write position
    len: u8,  // Current count (0..=N)
}

impl<T, const N: usize> RingBuffer<T, N> {
    /// Create empty ring buffer.
    #[inline]
    const fn new() -> Self {
        Self {
            items: [const { None }; N],
            head: 0,
            len: 0,
        }
    }

    /// Push item (evicts oldest if full).
    #[inline]
    #[expect(
        clippy::arithmetic_side_effects,
        reason = "Ring buffer arithmetic is modulo-bounded (0..N)"
    )]
    #[expect(
        clippy::indexing_slicing,
        reason = "All indices are modulo N (cannot exceed array bounds)"
    )]
    #[expect(
        clippy::as_conversions,
        reason = "u8 <-> usize conversions safe for N = 5"
    )]
    #[expect(
        clippy::cast_possible_truncation,
        reason = "N is constrained to 5 (ring buffer for file versions)"
    )]
    #[expect(
        clippy::integer_division_remainder_used,
        reason = "Modulo operation is fundamental to ring buffer wraparound"
    )]
    fn push(&mut self, item: T) {
        self.items[self.head as usize] = Some(item);
        self.head = (self.head + 1) % (N as u8);
        if self.len < N as u8 {
            self.len += 1;
        }
    }

    /// Get most recent item.
    #[inline]
    #[expect(
        clippy::arithmetic_side_effects,
        reason = "Ring buffer arithmetic is modulo-bounded (0..N)"
    )]
    #[expect(
        clippy::indexing_slicing,
        reason = "All indices are modulo N (cannot exceed array bounds)"
    )]
    #[expect(
        clippy::as_conversions,
        reason = "u8 <-> usize conversions safe for N = 5"
    )]
    #[expect(
        clippy::cast_possible_truncation,
        reason = "N is constrained to 5 (ring buffer for file versions)"
    )]
    #[expect(
        clippy::integer_division_remainder_used,
        reason = "Modulo operation is fundamental to ring buffer wraparound"
    )]
    fn current(&self) -> Option<&T> {
        if self.len == 0 {
            return None;
        }
        let idx = (self.head + (N as u8) - 1) % (N as u8);
        self.items[idx as usize].as_ref()
    }

    /// Get item at index (0 = oldest, len-1 = newest).
    #[inline]
    #[expect(
        clippy::arithmetic_side_effects,
        reason = "Ring buffer arithmetic is modulo-bounded (0..N)"
    )]
    #[expect(
        clippy::indexing_slicing,
        reason = "All indices are modulo N (cannot exceed array bounds)"
    )]
    #[expect(
        clippy::as_conversions,
        reason = "u8 <-> usize conversions safe for N = 5"
    )]
    #[expect(
        clippy::cast_possible_truncation,
        reason = "N is constrained to 5 (ring buffer for file versions)"
    )]
    #[expect(
        clippy::integer_division_remainder_used,
        reason = "Modulo operation is fundamental to ring buffer wraparound"
    )]
    fn get(&self, index: usize) -> Option<&T> {
        if index >= self.len as usize {
            return None;
        }
        let offset =
            (self.head + (N as u8) - self.len + index as u8) % (N as u8);
        self.items[offset as usize].as_ref()
    }

    /// Number of items.
    #[inline]
    #[expect(
        clippy::as_conversions,
        reason = "u8 -> usize is always safe and lossless"
    )]
    const fn len(&self) -> usize {
        self.len as usize
    }

    /// Iterate over items (oldest to newest).
    #[inline]
    fn iter(&self) -> impl Iterator<Item = &T> {
        (0..self.len()).filter_map(move |i| self.get(i))
    }
}

// ─────────────────────────────────────────────────────────────────────────────
//  Raw File Version Storage
// ─────────────────────────────────────────────────────────────────────────────

/// A single version of a raw file (content + metadata + hash).
#[derive(Debug, Clone, PartialEq, Archive, Serialize, Deserialize)]
pub struct RawFileVersion {
    /// Compressed file content (zstd level 3).
    compressed_content: Vec<u8>,
    /// Blake3 hash of uncompressed content.
    content_hash: Blake3Hash,
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
    /// Create a new file version from content and metadata.
    ///
    /// # Errors
    /// Returns error if compression fails.
    #[inline]
    pub fn new(
        content: &str,
        created_at: Option<SystemTime>,
        modified_at: Option<SystemTime>,
    ) -> Result<Self, std::io::Error> {
        let compressed_content = compress(content)?;
        let content_hash = Blake3Hash::compute(content.as_bytes());
        let recorded_at = SystemTime::now();

        Ok(Self {
            compressed_content,
            content_hash,
            created_at,
            modified_at,
            recorded_at,
        })
    }

    /// Get the Blake3 hash of the content.
    #[inline]
    #[must_use]
    pub fn content_hash(&self) -> &Blake3Hash {
        &self.content_hash
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

    /// Decompress and return file content.
    ///
    /// # Errors
    /// Returns error if decompression fails.
    #[inline]
    pub fn content(&self) -> Result<String, DecompressionError> {
        decompress(&self.compressed_content)
    }

    /// Get compressed size in bytes.
    #[inline]
    #[must_use]
    pub fn compressed_size(&self) -> usize {
        self.compressed_content.len()
    }
}

/// Raw schema file with version history (up to 5 versions).
#[derive(Debug, Clone, PartialEq, Archive, Serialize, Deserialize)]
pub struct RawSchemaFile {
    /// File path relative to vault root.
    file_path: String,
    /// Version history (ring buffer, max 5 versions).
    versions: RingBuffer<RawFileVersion, 5>,
}

impl RawSchemaFile {
    /// Create a new raw schema file with initial version.
    ///
    /// # Errors
    /// Returns error if compression fails.
    #[inline]
    pub fn new(
        file_path: String,
        content: &str,
        created_at: Option<SystemTime>,
        modified_at: Option<SystemTime>,
    ) -> Result<Self, std::io::Error> {
        let mut versions = RingBuffer::new();
        let version = RawFileVersion::new(content, created_at, modified_at)?;
        versions.push(version);

        Ok(Self {
            file_path,
            versions,
        })
    }

    /// Add a new version (evicts oldest if at capacity).
    ///
    /// # Errors
    /// Returns error if compression fails.
    #[inline]
    pub fn add_version(
        &mut self,
        content: &str,
        created_at: Option<SystemTime>,
        modified_at: Option<SystemTime>,
    ) -> Result<(), std::io::Error> {
        let version = RawFileVersion::new(content, created_at, modified_at)?;
        self.versions.push(version);
        Ok(())
    }

    /// Get the current (most recent) version.
    #[inline]
    #[must_use]
    pub fn current(&self) -> Option<&RawFileVersion> {
        self.versions.current()
    }

    /// Get file path.
    #[inline]
    #[must_use]
    pub fn file_path(&self) -> &str {
        &self.file_path
    }

    /// Get version history iterator (oldest to newest).
    #[inline]
    pub fn versions(&self) -> impl Iterator<Item = &RawFileVersion> {
        self.versions.iter()
    }

    /// Get version count.
    #[inline]
    #[must_use]
    pub fn version_count(&self) -> usize {
        self.versions.len()
    }
}

/// Raw property bank file (singleton, up to 5 versions).
#[derive(Debug, Clone, PartialEq, Archive, Serialize, Deserialize)]
pub struct RawPropertyBankFile {
    /// Version history (ring buffer, max 5 versions).
    versions: RingBuffer<RawFileVersion, 5>,
}

impl RawPropertyBankFile {
    /// Create a new property bank file with initial version.
    ///
    /// # Errors
    /// Returns error if compression fails.
    #[inline]
    pub fn new(
        content: &str,
        created_at: Option<SystemTime>,
        modified_at: Option<SystemTime>,
    ) -> Result<Self, std::io::Error> {
        let mut versions = RingBuffer::new();
        let version = RawFileVersion::new(content, created_at, modified_at)?;
        versions.push(version);

        Ok(Self {
            versions,
        })
    }

    /// Add a new version (evicts oldest if at capacity).
    ///
    /// # Errors
    /// Returns error if compression fails.
    #[inline]
    pub fn add_version(
        &mut self,
        content: &str,
        created_at: Option<SystemTime>,
        modified_at: Option<SystemTime>,
    ) -> Result<(), std::io::Error> {
        let version = RawFileVersion::new(content, created_at, modified_at)?;
        self.versions.push(version);
        Ok(())
    }

    /// Get the current (most recent) version.
    #[inline]
    #[must_use]
    pub fn current(&self) -> Option<&RawFileVersion> {
        self.versions.current()
    }

    /// Get version history iterator (oldest to newest).
    #[inline]
    pub fn versions(&self) -> impl Iterator<Item = &RawFileVersion> {
        self.versions.iter()
    }

    /// Get version count.
    #[inline]
    #[must_use]
    pub fn version_count(&self) -> usize {
        self.versions.len()
    }
}

// ─────────────────────────────────────────────────────────────────────────────
//  File Change Detection
// ─────────────────────────────────────────────────────────────────────────────

/// Type of change detected between two file versions.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum FileChange {
    /// Content hash unchanged (file not modified).
    Unchanged,
    /// Content hash changed (file was modified).
    Modified,
    /// Same hash but different path (file was renamed).
    Renamed,
}

/// Compare two file versions to detect changes.
///
/// This helper enables accurate change detection:
/// - **Unchanged**: Hash matches, path matches → no action needed
/// - **Modified**: Hash differs → re-parse and re-resolve
/// - **Renamed**: Hash matches but path differs → update path, no re-parse
///
/// # Examples
/// ```
/// # use lithos_core::schema::storage::{RawFileVersion, FileChange, diff_raw_files};
/// let v1 = RawFileVersion::new("content", None, None).unwrap();
/// let v2 = RawFileVersion::new("content", None, None).unwrap();
/// assert_eq!(diff_raw_files(&v1, &v2, "same.json", "same.json"), FileChange::Unchanged);
///
/// let v3 = RawFileVersion::new("changed", None, None).unwrap();
/// assert_eq!(diff_raw_files(&v1, &v3, "file.json", "file.json"), FileChange::Modified);
///
/// assert_eq!(diff_raw_files(&v1, &v2, "old.json", "new.json"), FileChange::Renamed);
/// ```
#[inline]
#[must_use]
pub fn diff_raw_files(
    cached: &RawFileVersion,
    current: &RawFileVersion,
    cached_path: &str,
    current_path: &str,
) -> FileChange {
    if cached.content_hash() != current.content_hash() {
        FileChange::Modified
    } else if cached_path != current_path {
        FileChange::Renamed
    } else {
        FileChange::Unchanged
    }
}

// ============================================================================
// Resolved Data Storage (Read Model Pattern)
// ============================================================================

/// Storage representation of a resolved schema (read model).
///
/// ## Read Model Pattern
///
/// This type is a **read model** - it has no behavior, no events, and no
/// domain logic. It exists purely to store and retrieve resolved schema data.
///
/// - **No Methods**: Only field accessors (getters)
/// - **No State Transitions**: Immutable after resolution
/// - **No Events**: Event emission happens in [`crate::schema::loader`]
///
/// ## Storage
///
/// Persisted to the `schema_by_id` table using `rkyv` serialization.
/// Contains all fields required for staleness checking and inheritance
/// tree reconstruction.
///
/// This is now the primary schema type used throughout the system.
/// Files are the source of truth; schemas are loaded, resolved, and stored
/// as `StoredSchema` values.
#[derive(Debug, Clone, PartialEq, Archive, Serialize, Deserialize)]
#[non_exhaustive]
pub struct StoredSchema {
    /// Schema identity.
    pub id: SchemaId,
    /// Schema name (flattened from `SchemaName` newtype).
    pub name: Box<str>,
    /// Parent schema ID, for `SchemaTree` reconstruction.
    pub parent_id: Option<SchemaId>,
    /// Resolved properties (flattened).
    pub properties: Vec<StoredProperty>,
}

impl StoredSchema {
    /// Create a new `StoredSchema` for testing purposes.
    #[inline]
    #[must_use]
    pub fn new(
        id: SchemaId,
        name: Box<str>,
        parent_id: Option<SchemaId>,
        properties: Vec<StoredProperty>,
    ) -> Self {
        Self {
            id,
            name,
            parent_id,
            properties,
        }
    }
}

/// Adapter storage representation of a property bank snapshot.
#[derive(Debug, Clone, PartialEq, Archive, Serialize, Deserialize)]
pub(crate) struct StoredPropertyBank {
    /// Bank version at time of persistence.
    pub bank_version: BankVersion,
    /// Wall-clock timestamp when this record was written.
    #[rkyv(with = AsUnixTime)]
    pub recorded_at: SystemTime,
    /// Flattened properties in the bank.
    pub properties: Vec<StoredProperty>,
}

/// Adapter storage representation of property bank metadata.
///
/// # Timestamps
///
/// Uses `SystemTime` with rkyv's `AsUnixTime` wrapper for safe serialization.
/// This stores timestamps as Unix epoch seconds internally while preserving
/// `SystemTime`'s type safety.
#[derive(Debug, Clone, PartialEq, Archive, Serialize, Deserialize)]
#[non_exhaustive]
pub struct StoredMetadata {
    /// Bank version at time of persistence.
    pub bank_version: BankVersion,
    /// Blake3 hash of source file content (for accurate staleness detection).
    pub source_file_hash: Blake3Hash,
    /// Filesystem birthtime (from `Metadata::created()`), if available.
    #[rkyv(with = Map<AsUnixTime>)]
    pub created_at: Option<SystemTime>,
    /// Filesystem mtime (from `Metadata::modified()`), if available.
    #[rkyv(with = Map<AsUnixTime>)]
    pub modified_at: Option<SystemTime>,
    /// Wall-clock timestamp when this record was written.
    #[rkyv(with = AsUnixTime)]
    pub recorded_at: SystemTime,
}

impl StoredMetadata {
    /// Build metadata for storage.
    #[inline]
    pub(crate) fn new(
        bank_version: BankVersion,
        source_file_hash: Blake3Hash,
        created_at: Option<SystemTime>,
        modified_at: Option<SystemTime>,
    ) -> Self {
        let recorded_at = SystemTime::now();
        Self {
            bank_version,
            source_file_hash,
            created_at,
            modified_at,
            recorded_at,
        }
    }
}

/// Child schema metadata stored in the `schema_children` multimap.
///
/// **Storage pattern:**
/// - Table: `schema_children` (multimap)
/// - Key: Parent `SchemaId` (as UUID string)
/// - Values: Multiple `StoredChildSchema` entries (one per child)
///
/// Each parent can have many children. This structure stores each child's
/// inheritance metadata including which properties it excludes from the parent.
///
/// **Cascade staleness:** When a parent schema changes, query this multimap
/// to find all children that must be re-resolved.
#[derive(Debug, Clone, PartialEq, Archive, Serialize, Deserialize)]
pub(crate) struct StoredChildSchema {
    /// Child schema ID.
    pub child_id: SchemaId,
    /// Property names this child excludes from parent's properties.
    pub excludes: Vec<Box<str>>,
    /// Timestamp when this inheritance relationship was last resolved.
    #[rkyv(with = AsUnixTime)]
    pub resolved_at: SystemTime,
}

impl StoredChildSchema {
    /// Serialize to bytes for multimap storage.
    ///
    /// # Errors
    /// Returns serialization error if rkyv encoding fails.
    pub(crate) fn to_bytes(&self) -> Result<Vec<u8>, crate::db::DbError> {
        rkyv::to_bytes(self).map(|bytes| bytes.to_vec()).map_err(
            |e: rkyv::rancor::Error| {
                crate::db::DbError::Serialization(e.to_string())
            },
        )
    }
}

/// Parent schema reference, stored in `schema_parent` table.
///
/// **Storage pattern:**
/// - Table: `schema_parent` (regular table, not multimap)
/// - Key: Child `SchemaId` (as UUID string)
/// - Value: `StoredParentSchema`
///
/// This table tracks ALL schemas (both roots and children):
/// - Root schemas: `parent_id = None`
/// - Child schemas: `parent_id = Some(parent_id)`
///
/// **Update optimization:** When updating a child's parent, this table
/// provides O(1) lookup of the old parent plus the old excludes/timestamp
/// needed to reconstruct the exact bytes for removing the old entry from
/// the `schema_children` multimap.
///
/// **Data redundancy:** `excludes` and `resolved_at` are stored in both
/// `schema_parent` and `schema_children`. This trades ~10KB of storage
/// (for typical 100-schema vaults) for simpler, faster update logic.
#[derive(Debug, Clone, PartialEq, Archive, Serialize, Deserialize)]
pub(crate) struct StoredParentSchema {
    /// Parent schema ID, or None for root schemas.
    pub parent_id: Option<SchemaId>,
    /// Property names excluded from parent (cached for multimap removal).
    pub excludes: Vec<Box<str>>,
    /// Timestamp when relationship was resolved (cached for multimap removal).
    #[rkyv(with = AsUnixTime)]
    pub resolved_at: SystemTime,
}

/// Adapter storage representation of a single bank property snapshot.
#[derive(Debug, Clone, PartialEq, Archive, Serialize, Deserialize)]
pub(crate) struct StoredBankProperty {
    /// Bank version at time of persistence.
    pub bank_version: BankVersion,
    /// Wall-clock timestamp when this record was written.
    #[rkyv(with = AsUnixTime)]
    pub recorded_at: SystemTime,
    /// Flattened property payload.
    pub property: StoredProperty,
}

impl StoredBankProperty {
    /// Format a bank property key for the given version.
    #[inline]
    pub(crate) fn key(version: BankVersion, suffix: &str) -> String {
        format!("{}:{suffix}", version.as_u64())
    }

    /// Format a bank property key prefix for the given version.
    #[inline]
    pub(crate) fn prefix(version: BankVersion) -> String {
        format!("{}:", version.as_u64())
    }
}

impl TryFrom<StoredPropertyBank> for PropertyBank {
    type Error = SchemaError;

    #[inline]
    fn try_from(stored: StoredPropertyBank) -> Result<Self, Self::Error> {
        let properties: Result<Vec<_>, _> = stored
            .properties
            .into_iter()
            .map(|sp| {
                let prop_name = PropertyName::try_new(&sp.name)?;
                let optionality = Optionality::from(sp.required);
                let multiplicity = Multiplicity::from(sp.multi);
                Ok(Property::new(
                    sp.id,
                    prop_name,
                    optionality,
                    multiplicity,
                    sp.spec,
                ))
            })
            .collect();
        PropertyBank::try_reconstruct(properties?, stored.bank_version)
    }
}

/// Flat storage representation of a single property.
#[derive(Debug, Clone, PartialEq, Archive, Serialize, Deserialize)]
#[non_exhaustive]
pub struct StoredProperty {
    /// Property identity.
    pub id: PropertyId,
    /// Property name (flattened from `PropertyName` newtype).
    pub name: Box<str>,
    /// Whether the property is required (flattened from `Optionality`).
    pub required: bool,
    /// Whether the property accepts multiple values (flattened from
    /// `Multiplicity`).
    pub multi: bool,
    /// Type-specific validation constraints.
    pub spec: PropertySpec,
}

impl StoredProperty {
    /// Create a new `StoredProperty`.
    #[inline]
    #[must_use]
    pub fn new(
        id: PropertyId,
        name: Box<str>,
        required: bool,
        multi: bool,
        spec: PropertySpec,
    ) -> Self {
        Self {
            id,
            name,
            required,
            multi,
            spec,
        }
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    mod diff_raw_files_tests {
        use super::*;

        #[test]
        fn unchanged_when_same_hash_and_path() {
            let v1 = RawFileVersion::new("content", None, None).unwrap();
            let v2 = RawFileVersion::new("content", None, None).unwrap();

            let change = diff_raw_files(&v1, &v2, "file.json", "file.json");
            assert_eq!(change, FileChange::Unchanged);
        }

        #[test]
        fn modified_when_hash_differs() {
            let v1 = RawFileVersion::new("old content", None, None).unwrap();
            let v2 = RawFileVersion::new("new content", None, None).unwrap();

            let change = diff_raw_files(&v1, &v2, "file.json", "file.json");
            assert_eq!(change, FileChange::Modified);
        }

        #[test]
        fn renamed_when_same_hash_different_path() {
            let v1 = RawFileVersion::new("content", None, None).unwrap();
            let v2 = RawFileVersion::new("content", None, None).unwrap();

            let change = diff_raw_files(&v1, &v2, "old.json", "new.json");
            assert_eq!(change, FileChange::Renamed);
        }

        #[test]
        fn modified_trumps_renamed() {
            let v1 = RawFileVersion::new("old", None, None).unwrap();
            let v2 = RawFileVersion::new("new", None, None).unwrap();

            // Different hash AND different path -> still Modified
            let change = diff_raw_files(&v1, &v2, "old.json", "new.json");
            assert_eq!(change, FileChange::Modified);
        }
    }

    #[test]
    fn raw_file_version_roundtrip() {
        let content = "schema: test\nproperties: []";
        let version = RawFileVersion::new(content, None, None)
            .expect("failed to create version");

        let decompressed = version.content().expect("failed to decompress");
        assert_eq!(content, decompressed);
    }

    #[test]
    fn raw_file_version_hash() {
        let content = "test content";
        let version1 = RawFileVersion::new(content, None, None)
            .expect("failed to create version");
        let version2 = RawFileVersion::new(content, None, None)
            .expect("failed to create version");

        // Same content produces same hash
        assert_eq!(version1.content_hash(), version2.content_hash());
    }

    #[test]
    fn raw_file_version_compression() {
        let content = "a".repeat(1000);
        let version = RawFileVersion::new(&content, None, None)
            .expect("failed to create version");

        // Compression should reduce size significantly
        assert!(
            version.compressed_size() < content.len(),
            "Compressed size should be smaller than original"
        );
    }

    #[test]
    fn raw_file_version_with_timestamps() {
        let now = SystemTime::now();
        let version = RawFileVersion::new("test", Some(now), Some(now))
            .expect("failed to create version");

        assert_eq!(version.created_at(), Some(now));
        assert_eq!(version.modified_at(), Some(now));
        assert!(version.recorded_at() >= now);
    }

    #[test]
    fn raw_schema_file_creation() {
        let file = RawSchemaFile::new(
            "schemas/test.toml".into(),
            "schema: test",
            None,
            None,
        )
        .expect("failed to create file");

        assert_eq!(file.file_path(), "schemas/test.toml");
        assert_eq!(file.version_count(), 1);
        assert!(file.current().is_some());
    }

    #[test]
    fn raw_schema_file_add_version() {
        let mut file = RawSchemaFile::new(
            "schemas/test.toml".into(),
            "version 1",
            None,
            None,
        )
        .expect("failed to create file");

        file.add_version("version 2", None, None)
            .expect("failed to add version");

        assert_eq!(file.version_count(), 2);
        let current = file.current().expect("no current version");
        let content = current.content().expect("failed to decompress");
        assert_eq!(content, "version 2");
    }

    #[test]
    fn raw_schema_file_version_limit() {
        let mut file = RawSchemaFile::new("test.toml".into(), "v1", None, None)
            .expect("failed to create file");

        // Add 5 more versions (total 6, should evict oldest)
        for i in 2i32..=6i32 {
            file.add_version(&format!("v{i}"), None, None)
                .expect("failed to add version");
        }

        // Should have exactly 5 versions (v2-v6, v1 evicted)
        assert_eq!(file.version_count(), 5);

        // Verify oldest is v2, newest is v6
        let versions: Vec<_> = file.versions().collect();
        assert_eq!(
            versions.first().and_then(|v| v.content().ok()).as_deref(),
            Some("v2")
        );
        assert_eq!(
            versions.last().and_then(|v| v.content().ok()).as_deref(),
            Some("v6")
        );
    }

    #[test]
    fn raw_property_bank_file_creation() {
        let file = RawPropertyBankFile::new("properties: []", None, None)
            .expect("failed to create file");

        assert_eq!(file.version_count(), 1);
        assert!(file.current().is_some());
    }

    #[test]
    fn raw_property_bank_file_add_version() {
        let mut file = RawPropertyBankFile::new("version 1", None, None)
            .expect("failed to create file");

        file.add_version("version 2", None, None)
            .expect("failed to add version");

        assert_eq!(file.version_count(), 2);
        let current = file.current().expect("no current version");
        let content = current.content().expect("failed to decompress");
        assert_eq!(content, "version 2");
    }
}
