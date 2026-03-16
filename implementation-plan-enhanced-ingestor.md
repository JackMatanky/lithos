# Implementation Plan: Enhanced Ingestor with Repository Embedding

## Overview

Transform the `Ingestor` from a pure file→raw translator into a smart component that:

1. Embeds the `Repository` for database access
2. Provides per-file staleness checking at decision points
3. Avoids unnecessary file I/O through staged checks
4. Returns hydrated `Raw*` types when fresh, or fresh raw data when stale
5. Handles persistence of Raw*View types (including compression)

## Motivation

### Current Pipeline (Before)

```
1. Load name_to_id from DB
2. Load ALL property_bank files → parse → hash → check staleness
3. Scan ALL schema files → parse → hash → check staleness
4. Partition by staleness
5. Resolve only stale schemas
```

**Problem**: Steps 2-3 read ALL file content before checking staleness, even when files haven't changed.

### Proposed Pipeline (After)

```
For each file:
  1. Get timestamps from filesystem (fast)
  2. Query DB view: is_timestamp_match(timestamps)?
     - YES → Reconstruct Raw from DB view (no content read!)
     - NO  → Read content → hash → is_content_match?
       - YES → Reconstruct Raw from DB view
       - NO  → Full parse needed
```

This avoids unnecessary file I/O at each decision point.

---

## Architecture

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                                Loader                                       │
│  ┌─────────────────────────────────────────────────────────────────────┐   │
│  │                     Ingestor<R: Repository>                         │   │
│  │  ┌─────────────┐  ┌──────────────────┐  ┌────────────────────┐    │   │
│  │  │   FsReader  │  │  Repository (R)   │  │  Staleness State   │    │   │
│  │  │  (file I/O) │  │   (embedded)     │  │  (path → view)     │    │   │
│  │  └─────────────┘  └──────────────────┘  └────────────────────┘    │   │
│  │                                                                       │   │
│  │  Main Methods (same names, enhanced behavior):                        │   │
│  │  - property_bank() → Option<IngestResult<RawPropertyBank>>            │   │
│  │  - schema(PathBuf) → IngestResult<RawSchema>                       │   │
│  │  - all_schemas() → Vec<IngestResult<RawSchema>>                    │   │
│  │                                                                       │   │
│  │  Persistence (handled internally):                                    │   │
│  │  - Saves Raw*View on Stale result                                   │   │
│  └─────────────────────────────────────────────────────────────────────┘   │
│                              ↓                                             │
│                    Resolution Pipeline                                     │
│                         (unchanged)                                       │
└─────────────────────────────────────────────────────────────────────────────┘
```

---

## Phase 1: Database Changes

### 1.1 Add Secondary Index Table

**File:** `lithos-core/src/schema/mod.rs`

In the `db_table` module, add after `RAW_SCHEMA_VIEWS`:

```rust
/// Maps file path to SchemaId for raw view lookup by path.
/// Key: file_path (e.g., "schemas/note.toml")
/// Value: SchemaId (as UUID string)
pub(crate) const RAW_SCHEMA_VIEW_BY_PATH: TableDefinition<&str, &str> =
    TableDefinition::new("raw_schema_view_by_path");
```

### 1.2 Add Repository Methods

**File:** `lithos-core/src/schema/storage.rs`

In the `Repository` trait, add:

```rust
/// Finds a raw schema view by file path.
/// Returns None if no view exists for the given path.
fn find_raw_schema_view_by_path(
    &self,
    file_path: &str,
) -> Result<Option<super::views::RawSchemaView>, Self::Error>;

/// Gets the SchemaId for a file path, if any.
/// Returns None if no schema exists at that path.
fn find_schema_id_by_path(
    &self,
    file_path: &str,
) -> Result<Option<SchemaId>, Self::Error>;
```

**Implementation in RedbRepository:**

```rust
#[inline]
fn find_raw_schema_view_by_path(
    &self,
    file_path: &str,
) -> Result<Option<RawSchemaView>, Self::Error> {
    use crate::schema::db_table::RAW_SCHEMA_VIEW_BY_PATH;

    // First lookup SchemaId by path
    let id_str = self.db.get_owned::<Box<str>>(RAW_SCHEMA_VIEW_BY_PATH, file_path)?;
    match id_str {
        Some(id_str) => {
            // Parse SchemaId and lookup view
            let id = SchemaId::try_from(id_str.as_ref())?;
            self.get_raw_schema_view(id)
        }
        None => Ok(None),
    }
}

#[inline]
fn find_schema_id_by_path(
    &self,
    file_path: &str,
) -> Result<Option<SchemaId>, Self::Error> {
    use crate::schema::db_table::RAW_SCHEMA_VIEW_BY_PATH;

    let id_str = self.db.get_owned::<Box<str>>(RAW_SCHEMA_VIEW_BY_PATH, file_path)?;
    match id_str {
        Some(id_str) => {
            let id = SchemaId::try_from(id_str.as_ref())?;
            Ok(Some(id))
        }
        None => Ok(None),
    }
}
```

### 1.3 Update Save Logic to Maintain Index

**File:** `lithos-core/src/schema/storage.rs`

Update `save_raw_schema_view`:

```rust
#[inline]
fn save_raw_schema_view(
    &self,
    id: SchemaId,
    view: &RawSchemaView,
) -> Result<(), Self::Error> {
    use crate::schema::db_table::{RAW_SCHEMA_VIEWS, RAW_SCHEMA_VIEW_BY_PATH};

    let key = id.to_string();
    self.db.batch_write(|batch| {
        batch.put(RAW_SCHEMA_VIEWS, key.as_str(), view)?;
        batch.put(RAW_SCHEMA_VIEW_BY_PATH, view.file_path(), key.as_str())?;
        Ok(())
    })?;

    Ok(())
}
```

---

## Phase 2: Raw*View Changes

### 2.1 Existing Capabilities (No Changes Needed)

The following already exist in `views/raw.rs`:

1. **RawFileVersion.is_timestamp_match(created_at, modified_at)** - Takes raw SystemTime values directly (not RawSchemaMetadata)
2. **RawFileVersion.is_content_match(content: &str)** - Checks if content matches hash
3. **TryFrom<&RawPropertyBank> for RawPropertyBankView** - Already exists
4. **TryFrom<&RawSchema> for RawSchemaView** - Already exists

### 2.2 Add Compressed Content Storage

**File:** `lithos-core/src/schema/views/raw.rs`

Add to `RawFileVersion`:

```rust
#[derive(Debug, Clone, PartialEq, Archive, Serialize, Deserialize)]
pub struct RawFileVersion {
    content_hash: [u8; 32],
    property_hashes: BTreeMap<PropertyName, [u8; 32]>,
    created_at: Option<SystemTime>,
    modified_at: Option<SystemTime>,
    recorded_at: SystemTime,
    /// Compressed original file content (zstd) - enables exact reconstruction
    compressed_content: Option<Vec<u8>>,  // NEW
}
```

### 2.3 Add Reconstruction Methods to Raw*View Types

Add methods to reconstruct Raw* types from cached content:

```rust
impl RawSchemaView {
    /// Reconstruct RawSchema from cached content.
    pub fn to_raw(&self) -> Option<RawSchema> {
        let version = self.current()?;
        let content = version.decompress_content()?;
        // Parse based on file extension stored in file_path
        todo!("Implement with stored format")
    }
}

impl RawPropertyBankView {
    /// Reconstruct RawPropertyBank from cached content.
    pub fn to_raw(&self) -> Option<RawPropertyBank> {
        let version = self.current()?;
        let content = version.decompress_content()?;
        // Parse based on file extension
        todo!("Implement with stored format")
    }
}
```

---

## Phase 3: Ingestor Changes

### 3.1 Restructure Ingestor to Embed Repository

**File:** `lithos-core/src/schema/ingestor.rs`

Key changes:
- Add `repository: R` field
- Implement `Repository` bound on generic parameter
- Keep same method names: `property_bank()`, `schema()`, `all_schemas()`

```rust
use crate::{
    config::aggregate::Config,
    fs::FsReader,
    schema::{
        error::SchemaIngestionError,
        raw::{RawPropertyBank, RawSchema},
        storage::Repository,
    },
};

/// Supported schema file extensions.
const SCHEMA_EXTENSIONS: &[&str] = &["json", "toml", "yaml", "yml"];

/// Ingestor result that includes staleness information.
#[derive(Debug, Clone)]
pub enum IngestResult<T> {
    /// Data is fresh (reused from cache)
    Fresh(T),
    /// Data was stale, returned the newly loaded raw data
    Stale(T),
}

/// Ingestor for loading raw schema files with embedded Repository for caching.
///
/// This adapter is responsible for:
/// - Loading the property bank file (JSON, TOML, or YAML)
/// - Scanning the schemas directory for schema files
/// - Per-file staleness checking to avoid unnecessary I/O
/// - Providing both fresh and stale variants based on file state
/// - Persisting Raw*View types (including compression)
///
/// It does NOT:
/// - Perform validation beyond deserialization
/// - Resolve references or build inheritance trees
pub struct Ingestor<'config, R> {
    source: FsReader,
    config: &'config Config,
    repository: R,
}

impl<'config, R> Ingestor<'config, R>
where
    R: Repository,
    R::Error: Into<SchemaIngestionError>,
{
    /// Create a new ingestor with the given file source, config, and repository.
    #[inline]
    #[must_use]
    pub fn new(
        source: FsReader,
        config: &'config Config,
        repository: R,
    ) -> Self {
        Self {
            source,
            config,
            repository,
        }
    }

    /// Get embedded repository reference (for Loader to persist results).
    pub fn repository(&self) -> &R {
        &self.repository
    }
}
```

### 3.2 Main Ingestor Methods (Same Names, Enhanced)

```rust
impl<R> Ingestor<'_, R>
where
    R: Repository,
    R::Error: Into<SchemaIngestionError>,
{
    /// Get the property bank with staleness detection.
    ///
    /// Returns:
    /// - `Ok(Some(IngestResult::Fresh(RawPropertyBank)))` if timestamps match
    /// - `Ok(Some(IngestResult::Stale(RawPropertyBank)))` if stale, with newly loaded data
    /// - `Ok(None)` if property bank file doesn't exist
    ///
    /// On Stale result, persists the RawPropertyBankView to database.
    pub fn property_bank(&self) -> Result<Option<IngestResult<RawPropertyBank>>, SchemaIngestionError> {
        let path = self.config.paths().property_bank_path();

        if !self.source.exists(&path) {
            return Ok(None);
        }

        // Step 1: Get timestamps (fast - no content read)
        let created_at = self.source.created_at(&path);
        let modified_at = self.source.modified_at(&path);

        // Step 2: Check staleness via embedded repository
        let view = self.repository.get_raw_property_bank_view()
            .map_err(|e| SchemaIngestionError::Io {
                path: "database".into(),
                reason: e.to_string().into()
            })?;

        if let Some(view) = view {
            // Step 2a: Fast timestamp check using RawFileVersion's existing method
            if view.is_timestamp_match(created_at, modified_at) {
                // Fresh! Reconstruct from cached content
                let cached = view.to_raw()
                    .ok_or_else(|| SchemaIngestionError::Io {
                        path: "property bank".into(),
                        reason: "failed to reconstruct from cache".into()
                    })?;
                return Ok(Some(IngestResult::Fresh(cached)));
            }

            // Step 2b: Content hash check (need to read file)
            let raw_bytes = self.source.read_bytes(&path)?;
            let content_hash = blake3::hash(&raw_bytes);

            if view.is_content_match(raw_bytes) { // Use existing is_content_match
                // Content matches - timestamps were wrong (clock skew, etc)
                let cached = view.to_raw()
                    .ok_or_else(|| SchemaIngestionError::Io {
                        path: "property bank".into(),
                        reason: "failed to reconstruct from cache".into()
                    })?;
                return Ok(Some(IngestResult::Fresh(cached)));
            }
        }

        // Step 3: Stale - full load required
        let raw_bank = self.load_full_property_bank()?;

        // Persist view (including compression) - uses existing TryFrom
        let view = RawPropertyBankView::try_from(&raw_bank)
            .map_err(|e| SchemaIngestionError::Io {
                path: "property bank".into(),
                reason: e.to_string().into(),
            })?;

        self.repository.save_raw_property_bank_view(&view)
            .map_err(|e| SchemaIngestionError::Io {
                path: "database".into(),
                reason: e.to_string().into(),
            })?;

        Ok(Some(IngestResult::Stale(raw_bank)))
    }

    /// Load all schema files with staleness detection.
    ///
    /// Returns Vec of IngestResult, one for each discovered schema file.
    /// On Stale results, persists the RawSchemaView to database.
    pub fn all_schemas(&self) -> Result<Vec<IngestResult<RawSchema>>, SchemaIngestionError> {
        let paths = self.config.paths();
        let schemas_dir = paths.schema.schemas_dir().as_path();
        let property_bank_filename = paths.property_bank.as_str();
        let vault_root = paths.vault_root();

        let mut results = Vec::new();

        for ext in SCHEMA_EXTENSIONS {
            let pattern = format!("{}/**/*.{}", schemas_dir.display(), ext);
            let files = self.source.list_files(&pattern).map_err(|error| {
                SchemaIngestionError::FileSystem(error.to_string().into())
            })?;

            for path in files {
                if path.file_name().is_some_and(|name| name == property_bank_filename) {
                    continue;
                }

                // Compute relative path: strip vault_root from path
                let relative_path = path.strip_prefix(vault_root)
                    .map(|p| p.to_string_lossy().into_boxed_str())
                    .unwrap_or_else(|_| path.to_string_lossy().into_boxed_str());

                let result = self.schema_internal(&path, &relative_path)?;
                results.push(result);
            }
        }

        Ok(results)
    }

    /// Load a single schema file with staleness detection.
    pub fn schema(&self, path: &Path) -> Result<IngestResult<RawSchema>, SchemaIngestionError> {
        // Compute relative path from vault_root
        let vault_root = self.config.paths().vault_root();
        let relative_path = path.strip_prefix(vault_root)
            .map(|p| p.to_string_lossy().into_boxed_str())
            .unwrap_or_else(|_| path.to_string_lossy().into_boxed_str());

        self.schema_internal(path, &relative_path)
    }

    // --- Private helper methods ---

    /// Internal implementation for schema loading with staleness.
    fn schema_internal(
        &self,
        path: &Path,
        relative_path: &str,
    ) -> Result<IngestResult<RawSchema>, SchemaIngestionError> {
        // Step 1: Get timestamps (fast)
        let created_at = self.source.created_at(path);
        let modified_at = self.source.modified_at(path);

        // Step 2: Check staleness via embedded repository
        let view = self.repository.find_raw_schema_view_by_path(relative_path)
            .map_err(|e| SchemaIngestionError::Io {
                path: "database".into(),
                reason: e.to_string().into()
            })?;

        if let Some(view) = view {
            // Step 2a: Fast timestamp check using RawFileVersion's existing method
            if view.is_timestamp_match(created_at, modified_at) {
                // Fresh! Reconstruct from cached content
                let cached = view.to_raw()
                    .ok_or_else(|| SchemaIngestionError::Io {
                        path: relative_path.into(),
                        reason: "failed to reconstruct from cache".into()
                    })?;
                return Ok(IngestResult::Fresh(cached));
            }

            // Step 2b: Content hash check
            let raw_bytes = self.source.read_bytes(path)?;

            if view.is_content_match(&raw_bytes) { // Use RawFileVersion's is_content_match
                let cached = view.to_raw()
                    .ok_or_else(|| SchemaIngestionError::Io {
                        path: relative_path.into(),
                        reason: "failed to reconstruct from cache".into()
                    })?;
                return Ok(IngestResult::Fresh(cached));
            }
        }

        // Step 3: Stale - full parse required
        let raw_schema = self.load_full_schema(path)?;

        // Persist view (including compression) - uses existing TryFrom
        // Get or create SchemaId for this path
        let id = self.repository.find_schema_id_by_path(relative_path)
            .map_err(|e| SchemaIngestionError::Io {
                path: "database".into(),
                reason: e.to_string().into(),
            })?
            .unwrap_or_else(SchemaId::new);

        let view = RawSchemaView::try_from(&raw_schema)
            .map_err(|e| SchemaIngestionError::Io {
                path: relative_path.into(),
                reason: e.to_string().into(),
            })?;

        self.repository.save_raw_schema_view(id, &view)
            .map_err(|e| SchemaIngestionError::Io {
                path: "database".into(),
                reason: e.to_string().into(),
            })?;

        Ok(IngestResult::Stale(raw_schema))
    }

    /// Load full property bank (without staleness check).
    /// This is the current implementation moved to a private method.
    fn load_full_property_bank(&self) -> Result<RawPropertyBank, SchemaIngestionError> {
        let path = self.config.paths().property_bank_path();

        let created_at = self.source.created_at(&path);
        let modified_at = self.source.modified_at(&path);
        let raw_bytes = self.source.read_bytes(&path)?;
        let content_hash = blake3::hash(&raw_bytes);

        let mut bank: RawPropertyBank = self.source.parse_structured(&path)?
            .validated(&path.to_string_lossy())?;

        let property_hashes = bank.properties.iter()
            .filter_map(|(name, entry)| {
                serde_json::to_string(entry).ok().map(|json| {
                    let hash = blake3::hash(json.as_bytes());
                    (name.clone(), *hash.as_bytes())
                })
            })
            .collect();

        bank.metadata = RawSchemaMetadata {
            created_at,
            modified_at,
            content_hash: Some(*content_hash.as_bytes()),
            property_hashes,
        };

        Ok(bank)
    }

    /// Load full schema (without staleness check).
    /// This is the current implementation.
    fn load_full_schema(&self, path: &Path) -> Result<RawSchema, SchemaIngestionError> {
        let filename_stem = path.file_stem().and_then(|s| s.to_str()).ok_or_else(|| {
            SchemaIngestionError::FileSystem(
                format!("Invalid filename for schema: {}", path.display()).into()
            )
        })?;

        let created_at = self.source.created_at(path);
        let modified_at = self.source.modified_at(path);
        let raw_bytes = self.source.read_bytes(path)?;
        let content_hash = blake3::hash(&raw_bytes);

        let mut raw: RawSchema = self.source.parse_structured(path)?;
        raw.name = filename_stem.into();
        let mut raw = raw.validated(&path.to_string_lossy())?;

        raw.metadata = RawSchemaMetadata {
            created_at,
            modified_at,
            content_hash: Some(*content_hash.as_bytes()),
            property_hashes: RawSchemaMetadata::compute_property_hashes(&raw.properties),
        };

        Ok(raw)
    }
}
```

---

## Phase 4: Loader Changes

### 4.1 Simplify Loader to Use Enhanced Ingestor

**File:** `lithos-core/src/schema/loader.rs`

The Loader becomes much simpler since Ingestor now handles staleness and persistence:

```rust
pub struct Loader<'config, R> {
    ingestor: Ingestor<'config, R>,
}

impl<R> Loader<'_, R>
where
    R: Repository,
    R::Error: Into<SchemaRepositoryError>,
{
    pub fn new(
        repository: R,
        source: FsReader,
        config: &'config Config,
    ) -> Self {
        Self {
            ingestor: Ingestor::new(source, config, repository),
        }
    }

    pub fn load(&self) -> Result<Vec<Schema>, SchemaLoaderError> {
        // --- Step 1: Property bank with staleness detection ---
        // Ingestor handles persistence of RawPropertyBankView
        let bank_result = self.ingestor.property_bank()
            .map_err(|e| SchemaLoaderError::Ingestion(e))?;

        let (bank, changed_properties) = match bank_result {
            Some(IngestResult::Fresh(cached)) => {
                // No changes - use cached
                let bank: PropertyBank = cached.try_into()?;
                (bank, Vec::new())
            }
            Some(IngestResult::Stale(raw)) => {
                // Already persisted by Ingestor
                let bank: PropertyBank = raw.try_into()?;
                self.ingestor.repository().save_property_bank(&bank)
                    .map_err(|e| SchemaLoaderError::Repository(e.into()))?;

                // Track changes for incremental resolution
                // TODO: compute from view comparison
                let changed = Vec::new();
                (bank, changed)
            }
            None => return Err(SchemaLoaderError::Ingestion(
                SchemaIngestionError::FileSystem("Property bank not found".into())
            )),
        };

        // --- Step 2: Load schemas with staleness detection ---
        // Ingestor handles persistence of RawSchemaView
        let schema_results = self.ingestor.all_schemas()
            .map_err(|e| SchemaLoaderError::Ingestion(e))?;

        let mut fresh_ids = Vec::new();
        let mut stale_raws = Vec::new();

        for result in schema_results {
            match result {
                IngestResult::Fresh(_cached) => {
                    // Already persisted by Ingestor - load ID for known_parents
                    // Need path from somewhere - TODO: add path to IngestResult
                }
                IngestResult::Stale(raw) => {
                    stale_raws.push(raw);
                }
            }
        }

        // --- Step 3: Resolution pipeline (unchanged logic) ---
        // ... rest of resolution

        Ok(resolved)
    }
}
```

---

## Phase 5: Test Suite Design

### 5.1 Unit Tests for Ingestor

**File:** `lithos-core/src/schema/ingestor.rs` (add to existing tests)

```rust
#[cfg(test)]
mod staleness_tests {
    use super::*;
    use crate::schema::storage::FakeRepository;

    /// Test: Fresh property bank returns Fresh variant when timestamps match
    #[test]
    fn fresh_property_bank_returns_fresh() {
        // Setup: DB has property bank with matching timestamps
        // Action: Call property_bank()
        // Assert: Returns IngestResult::Fresh
    }

    /// Test: Stale property bank (timestamp mismatch) returns Stale
    #[test]
    fn stale_property_bank_by_timestamp() {
        // Setup: DB has view with old timestamps
        // Action: Call property_bank() with newer file
        // Assert: Returns IngestResult::Stale with newly loaded data
    }

    /// Test: Stale property bank (content hash mismatch) returns Stale
    #[test]
    fn stale_property_bank_by_hash() {
        // Setup: DB has view with same timestamps but different content
        // Action: Call property_bank()
        // Assert: Returns IngestResult::Stale
    }

    /// Test: Content matches but timestamps differ (clock skew) - returns Fresh
    #[test]
    fn timestamp_mismatch_uses_content_hash() {
        // Setup: File content matches, but filesystem timestamps differ
        // Action: Call property_bank()
        // Assert: Returns IngestResult::Fresh (content hash takes precedence)
    }

    /// Test: Fresh schema returns Fresh variant
    #[test]
    fn fresh_schema_returns_fresh() {
        // Similar to property bank tests
    }

    /// Test: Path-based lookup finds correct view
    #[test]
    fn path_based_lookup_finds_view() {
        // Setup: Save view with known path
        // Action: Query by path
        // Assert: Returns correct view
    }

    /// Test: Unknown path returns None (not error)
    #[test]
    fn unknown_path_returns_none() {
        // Action: Query non-existent path
        // Assert: Returns Ok(None)
    }

    /// Test: New schema is detected as stale (no view exists)
    #[test]
    fn new_schema_detected() {
        // Setup: No view exists for schema path
        // Action: Call schema()
        // Assert: Returns IngestResult::Stale
    }
}
```

### 5.2 Integration Tests

**File:** `lithos-core/tests/schema_staleness.rs` (new)

```rust
/// Integration tests for staleness detection and caching behavior.

use lithos_core::schema::loader::Loader;

/// Test: Second load with unchanged files uses cache (no re-resolution)
#[test]
fn unchanged_files_use_cache() {
    // First load
    let initial = loader.load()?;

    // Second load (no file changes)
    let cached = loader.load()?;

    // Should return empty - nothing was stale
    assert!(cached.is_empty());
}

/// Test: File modification triggers re-resolution
#[test]
fn file_modification_triggers_resolution() {
    // First load
    // Modify file
    // Second load
    // Assert: Schema is in returned set
}

/// Test: Property bank change cascades to dependent schemas
#[test]
fn property_bank_change_cascades() {
    // First load with schema referencing property bank
    // Modify property bank
    // Second load
    // Assert: Schema is re-resolved
}
```

### 5.3 Repository Tests for New Methods

**File:** `lithos-core/src/schema/storage.rs` (add tests)

```rust
#[cfg(test)]
mod repository_tests {
    use super::*;

    /// Test: find_raw_schema_view_by_path returns correct view
    #[test]
    fn find_raw_schema_view_by_path() {
        // Use FakeRepository for controlled testing
    }

    /// Test: find_schema_id_by_path returns correct id
    #[test]
    fn find_schema_id_by_path() {
        // Use FakeRepository
    }

    /// Test: Index is updated on save_raw_schema_view
    #[test]
    fn index_updated_on_save() {
        // Save view, then query by path, verify it's found
    }

    /// Test: Index lookup returns None for unknown path
    #[test]
    fn index_returns_none_for_unknown() {
        // Query non-existent path
        // Assert: Ok(None)
    }
}
```

---

## Summary of File Changes

| File                              | Changes                                                                        |
| --------------------------------- | ------------------------------------------------------------------------------ |
| `schema/mod.rs`                   | Add `RAW_SCHEMA_VIEW_BY_PATH` table constant                                   |
| `schema/storage.rs`               | Add new Repository methods, implement in RedbRepository, update save method   |
| `schema/views/raw.rs`             | Add compressed_content field, to_raw() methods                                 |
| `schema/ingestor.rs`              | Restructure to embed Repository, add staleness detection to main methods      |
| `schema/loader.rs`                | Simplify to use enhanced Ingestor                                             |
| New: `tests/schema_staleness.rs`  | Integration tests                                                              |

---

## Open Questions

### Q1: Compression Format
**What compression format should we use for compressed_content?**

Options: zstd, lz4, zlib
- **zstd**: Good balance of speed and compression, growing popularity
- **lz4**: Very fast, lower compression ratio
- **zlib**: Universal support, but slower

**Recommendation**: zstd - modern, fast, good compression.

---

### Q2: IngestResult Path Inclusion
**Should IngestResult include the file path for Loader to use?**

Currently, for fresh schemas, we need the path to look up the SchemaId for known_parents.

Options:
- **A**: Include path in IngestResult::Fresh variant
- **B**: Add separate method to get fresh paths
- **C**: Have Loader do the path lookup separately

**Recommendation**: Option A - include path in Fresh variant.

---

### Q3: Error Handling
**How should stale-check failures be handled?**

- **Option A (Fail hard)**: DB unavailable = fail entire load.
- **Option B (Fall back)**: DB unavailable = fall back to full parse.
- **Option C (Configurable)**: Allow configuration.

**Current recommendation**: Option A - fail hard, since if we can't check staleness, we shouldn't assume freshness.

---

## Changelog

| Date       | Description                                                    |
| ---------- | -------------------------------------------------------------- |
| 2026-03-16 | Initial plan created                                          |
| 2026-03-16 | Updated based on review: removed unnecessary helpers, use existing TryFrom and is_timestamp_match methods |
