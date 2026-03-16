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

## Implementation Phases

This implementation is divided into 5 clearly defined phases, each with specific deliverables and validation steps.

---

## Phase 1: Database Schema Changes (Foundation)

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

### Phase 1 Deliverables

- [ ] New table constant `RAW_SCHEMA_VIEW_BY_PATH` added to `schema/mod.rs`
- [ ] New Repository trait methods added and documented
- [ ] RedbRepository implementation for new methods
- [ ] Updated `save_raw_schema_view` maintains both tables
- [ ] Unit tests for path-based lookup (new methods)
- [ ] All existing tests pass

### Phase 1 Validation

Run: `mise run test:unit:schema`

Expected: All tests pass, including new tests for `find_raw_schema_view_by_path` and `find_schema_id_by_path`.

---

## Phase 2: Raw*View Enhancements (Content Storage & Reconstruction)

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

### Phase 2 Deliverables

- [ ] `compressed_content` field added to `RawFileVersion`
- [ ] Update `RawFileVersion::new()` to accept and store compressed content
- [ ] Add `decompress_content()` method to `RawFileVersion`
- [ ] Add `to_raw()` methods to `RawSchemaView` and `RawPropertyBankView`
- [ ] Update `TryFrom` implementations to compress and store content
- [ ] Unit tests for compression/decompression round-trip
- [ ] Unit tests for `to_raw()` reconstruction
- [ ] All existing tests pass

### Phase 2 Validation

Run: `mise run test:unit:schema`

Expected: All tests pass, including new tests for content compression/decompression and Raw* reconstruction.

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

**Implementation details for the three main methods are left to implementation phase.**

The core behavior for each method:

#### `property_bank() -> Option<IngestResult<RawPropertyBank>>`

1. Check if file exists, return `None` if not
2. Try to load `RawPropertyBankView` from repository
3. If view exists:
   - Check timestamp match → if fresh, reconstruct and return `Fresh`
   - Read file content and check hash → if fresh, reconstruct and return `Fresh`
4. Parse file (stale or new)
5. Create and persist `RawPropertyBankView` using `TryFrom`
6. Return `Stale`

#### `schema(path) -> IngestResult<RawSchema>`

1. Derive relative path (path is already relative to vault root)
2. Try to load `RawSchemaView` from repository by path
3. If view exists:
   - Check timestamp match → if fresh, reconstruct and return `Fresh`
   - Read file content and check hash → if fresh, reconstruct and return `Fresh`
4. Parse file (stale or new)
5. Get or create SchemaId for this path
6. Create and persist `RawSchemaView` using `TryFrom`
7. Return `Stale`

#### `all_schemas() -> Vec<IngestResult<RawSchema>>`

1. Scan schemas directory for all schema files (exclude property bank)
2. For each file, call the equivalent logic of `schema(path)`
3. Return Vec of results

### Phase 3 Deliverables

- [ ] `IngestResult<T>` enum added to `ingestor.rs`
- [ ] Ingestor struct updated to embed `repository: R` field
- [ ] Constructor updated: `new(source, config, repository)`
- [ ] `property_bank()` method updated to return `Option<IngestResult<RawPropertyBank>>`
- [ ] `schema(path)` method updated to return `IngestResult<RawSchema>`
- [ ] `all_schemas()` method updated to return `Vec<IngestResult<RawSchema>>`
- [ ] All three methods implement staleness checking and view persistence
- [ ] Unit tests for each method covering Fresh and Stale cases
- [ ] All existing tests updated to pass repository to Ingestor
- [ ] Integration tests pass

### Phase 3 Validation

Run: `mise run test:unit:schema && mise run test:integration`

Expected: All tests pass with new Ingestor behavior. Fresh files return cached data without re-parsing.

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

### Phase 4 Deliverables

- [ ] Loader struct updated to embed `Ingestor<R>` instead of separate FsReader
- [ ] Loader constructor updated to create Ingestor with repository
- [ ] `load()` method updated to use `IngestResult` from Ingestor
- [ ] Logic updated to handle Fresh vs Stale results
- [ ] TODO resolved: Add path to IngestResult or alternative solution
- [ ] All existing loader tests updated and passing
- [ ] Integration tests for full pipeline

### Phase 4 Validation

Run: `mise run test`

Expected: All tests pass, including integration tests showing end-to-end staleness detection working.

---

## Phase 5: Comprehensive Test Suite

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

### Phase 5 Deliverables

- [ ] Unit tests for all three Ingestor methods (property_bank, schema, all_schemas)
- [ ] Unit tests for Repository path-based lookup methods
- [ ] Unit tests covering Fresh and Stale scenarios
- [ ] Unit tests for timestamp vs content hash priority
- [ ] Integration tests for end-to-end staleness detection
- [ ] Integration tests for property bank changes cascading to schemas
- [ ] Test coverage for new file detection (no view case)
- [ ] All tests documented with clear setup/action/assertion

### Phase 5 Validation

Run: `mise run verify`

Expected: All tests pass with 100% coverage of new staleness detection logic.

---

## Implementation Order

Execute phases in strict sequence:

1. **Phase 1** (Database) → Validates schema changes and new methods work
2. **Phase 2** (Views) → Enables content storage and reconstruction
3. **Phase 3** (Ingestor) → Implements staleness detection using Phases 1-2
4. **Phase 4** (Loader) → Integrates enhanced Ingestor into pipeline
5. **Phase 5** (Tests) → Comprehensive validation of entire feature

Each phase must be complete and validated before starting the next phase.

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
| 2026-03-16 | Fixed relative path logic: schemas_dir is already relative to vault root; added helper methods for timestamp/content staleness checks; no error when view is None (new file case) |
| 2026-03-16 | Simplified to focus on three main methods only (property_bank, schema, all_schemas); removed helper method details to avoid over-planning |
| 2026-03-16 | Added clear phase structure with deliverables and validation steps for each phase |
