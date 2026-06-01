# Unified Discovery Engine Implementation Plan

**Date**: 2026-04-30
**Status**: Ready for Implementation
**Complexity**: High
**Estimated Effort**: 2-3 days

---

## Executive Summary

This plan consolidates the fragmented discovery logic across `Builder`, `PropertyBankProcessor`, and `SchemaProcessor` into a single **`DiscoveryEngine`** component that performs all I/O operations in one atomic batch transaction.

### Benefits

- **66% reduction in repository transactions**: From 3 separate transactions to 1 atomic batch
- **Simplified orchestration**: Builder becomes a linear coordinator
- **Better error handling**: Atomic operations prevent inconsistent state
- **Improved testability**: All discovery I/O isolated in one component
- **Preserved type safety**: Existing typestate patterns remain intact

### Key Innovation

Uses a unified `DiscoveredFile` structure keyed by `DiscoveryOutcome.files` path, with `SchemaFileKind` + `DiscoveredView` for type-safe branching. This eliminates duplication while preserving path-scoped discovery semantics.

---

## Table of Contents

1. [Component Design](#component-design)
2. [Implementation Plan: Two-Stage Approach](#implementation-plan-two-stage-approach)
3. [Testing Strategy](#testing-strategy)
4. [Error Handling](#error-handling)
5. [Performance Characteristics](#performance-characteristics)
6. [Rollback Plan](#rollback-plan)

---

## Component Design

### 1. Core Data Structures

#### 1.1 `DiscoveryOutcome` - Complete Discovery Result

**File**: `lithos-core/src/schema/discovery.rs`

```rust
use std::collections::HashMap;
use crate::{
    fs::RelativePath,
    schema::{
        identifier::SchemaId,
        inheritance::InheritanceGraph,
        views::RawView,
    },
};

/// Complete discovery outcome containing all data needed to initialize both processors.
///
/// This structure is the result of a SINGLE batch I/O operation that fetches:
/// - Property bank view and file stats (if property bank exists)
/// - Schema views, IDs, and file stats for all schema files
/// - Topological graph (if present)
/// - Deleted schema detection (files in DB but not in filesystem)
///
/// # Invariants
///
/// - `files` contains entries for ALL files in `FilesContext::files` PLUS property bank (if it exists)
/// - `deleted_schemas` contains only IDs not present in `files`
/// - File stats are fetched for all files in a single efficient batch
/// - Exactly one file has `is_property_bank == true` (or none if no property bank)
///
/// # Architecture
///
/// The unified `DiscoveredFile` design eliminates duplication between property bank
/// and schema discovery by using the `RawView` trait. Both `RawPropertyBankView`
/// and `RawSchemaView` implement this trait, allowing polymorphic handling.
#[derive(Debug)]
pub(crate) struct DiscoveryOutcome {
    /// Discovered files, keyed by path.
    /// Contains ALL files from `FilesContext` (schemas + property bank if it exists).
    pub(crate) files: HashMap<RelativePath, DiscoveredFile>,

    /// Topological graph from previous run (None if never run or cold-start).
    pub(crate) graph: Option<InheritanceGraph<()>>,

    /// Schema IDs that exist in DB but have no corresponding file.
    /// These schemas were deleted from the filesystem.
    pub(crate) deleted_schemas: Vec<SchemaId>,
}

impl DiscoveryOutcome {
    /// Returns true if this is a cold-start (no previous data in DB).
    #[inline]
    #[must_use]
    pub(crate) fn is_cold_start(&self) -> bool {
        self.files.values().all(|f| f.view.is_none()) && self.graph.is_none()
    }

    /// Returns true if this is an incremental update (has previous data).
    #[inline]
    #[must_use]
    pub(crate) fn is_incremental(&self) -> bool {
        !self.is_cold_start()
    }

    /// Returns true if any schemas exist on disk.
    #[inline]
    #[must_use]
    pub(crate) fn has_schemas(&self) -> bool {
        self.files.values().any(|f| !f.is_property_bank)
    }

    /// Returns the property bank file, if it exists.
    #[inline]
    #[must_use]
    pub(crate) fn property_bank(&self) -> Option<&DiscoveredFile> {
        self.files.values().find(|f| f.is_property_bank)
    }

    /// Returns an iterator over schema files (excludes property bank).
    #[inline]
    pub(crate) fn schema_files(&self) -> impl Iterator<Item = &DiscoveredFile> {
        self.files.values().filter(|f| !f.is_property_bank)
    }
}
```

#### 1.2 `DiscoveredFile` - Unified File Snapshot

```rust
use crate::{
    fs::{FileStats, Filename, RelativePath},
    schema::{
        identifier::SchemaId,
        views::{RawPropertyBankView, RawSchemaView, RawView},
    },
};

/// Unified discovery data for a single file (schema or property bank).
///
/// This structure eliminates duplication by using the `RawView` trait.
/// Both `RawPropertyBankView` and `RawSchemaView` implement `RawView`,
/// allowing polymorphic handling of cached metadata.
///
/// # Design Rationale
///
/// Previously, we had separate `PropertyBankDiscovery` and `SchemaDiscovery`
/// types with nearly identical fields. This created maintenance overhead and
/// required duplicate logic in discovery and processor initialization.
///
/// The unified design:
/// - Uses `is_property_bank` flag to distinguish file types
/// - Stores view as an enum to preserve type safety
/// - Enables shared logic for timestamp/content comparison
/// - Simplifies the discovery engine implementation
#[derive(Debug, Clone)]
pub(crate) struct DiscoveredFile {
    /// Filename (e.g., "note.toml", "property-bank.json").
    pub(crate) filename: Filename,

    /// Relative path to the file.
    pub(crate) path: RelativePath,

    /// Schema ID or property bank ID.
    /// - For schemas: from DB if view exists, generated otherwise
    /// - For property bank: stable ID (not used for indexing)
    pub(crate) id: SchemaId,

    /// Whether this file is the property bank.
    pub(crate) is_property_bank: bool,

    /// Cached view from DB (None if never loaded).
    /// Use `view_type()` to safely access the specific view variant.
    pub(crate) view: Option<DiscoveredView>,

    /// Current file stats from filesystem.
    pub(crate) file_stats: FileStats,
}

/// Type-safe wrapper for cached views.
///
/// This enum preserves type safety while allowing unified handling
/// in discovery logic. Processors can pattern match to get the
/// specific view type they need.
#[derive(Debug, Clone)]
pub(crate) enum DiscoveredView {
    /// Cached schema view.
    Schema(RawSchemaView),

    /// Cached property bank view.
    PropertyBank(RawPropertyBankView),
}

impl DiscoveredFile {
    /// Checks if timestamps match between cached view and current file.
    ///
    /// Returns `false` if no cached view exists.
    #[must_use]
    pub(crate) fn is_timestamp_match(&self) -> bool {
        self.view.as_ref().map_or(false, |view| match view {
            DiscoveredView::Schema(v) => v.is_timestamp_match(
                self.file_stats.created_at(),
                self.file_stats.modified_at(),
            ),
            DiscoveredView::PropertyBank(v) => v.is_timestamp_match(
                self.file_stats.created_at(),
                self.file_stats.modified_at(),
            ),
        })
    }

    /// Returns true if this file has never been seen (no cached view).
    #[inline]
    #[must_use]
    pub(crate) fn is_new(&self) -> bool {
        self.view.is_none()
    }

    /// Returns the schema view if this is a schema file.
    #[inline]
    #[must_use]
    pub(crate) fn as_schema_view(&self) -> Option<&RawSchemaView> {
        match &self.view {
            Some(DiscoveredView::Schema(v)) => Some(v),
            _ => None,
        }
    }

    /// Returns the property bank view if this is the property bank.
    #[inline]
    #[must_use]
    pub(crate) fn as_property_bank_view(&self) -> Option<&RawPropertyBankView> {
        match &self.view {
            Some(DiscoveredView::PropertyBank(v)) => Some(v),
            _ => None,
        }
    }
}
```

### 2. `DiscoveryEngine` Implementation

**File**: `lithos-core/src/schema/discovery.rs`

````rust
use std::collections::{HashMap, HashSet};
use crate::{
    fs::{FileStats, Filename, FileReader, RelativePath},
    schema::{
        builder::{FilesContext, PropertyBankContext},
        error::{SchemaIngestionError, SchemaLoaderError, SchemaRepositoryError},
        identifier::SchemaId,
        inheritance::InheritanceGraph,
        storage::{BatchSchemaReader, Repository},
        views::{RawPropertyBankView, RawSchemaView},
    },
};

/// Discovery engine that consolidates ALL I/O operations into a single batch.
///
/// This component replaces the scattered discovery logic across:
/// - `PropertyBankProcessor::discover()` (property bank view + file stats)
/// - `SchemaProcessor::<Discovery, NeverSeen>::discover()` (cold-start: file stats only)
/// - `SchemaProcessor::<Discovery, Review>::discover()` (incremental: views + IDs + file stats + graph)
///
/// # Architecture
///
/// The engine performs a SINGLE batch operation that:
/// 1. Fetches property bank view and file stats (if property bank exists)
/// 2. Fetches schema views, IDs, and file stats for ALL schema files
/// 3. Fetches topological graph (if present)
/// 4. Detects deleted schemas (IDs in DB but no file)
///
/// All repository reads occur within a **single transaction** via `with_batch_schema_reader`.
/// All file stats are fetched efficiently (sequentially or in parallel).
///
/// # Unified File Discovery
///
/// The engine uses `DiscoveredFile` for both schemas and property bank,
/// eliminating duplication. The `is_property_bank` flag distinguishes file types,
/// and the `DiscoveredView` enum preserves type safety.
///
/// # Usage
///
/// ```ignore
/// let outcome = DiscoveryEngine::run(&context, &repo, &source)?;
///
/// if outcome.is_cold_start() {
///     // Initialize processors for NeverSeen path
/// } else {
///     // Initialize processors for Review path
/// }
/// ```
pub(crate) struct DiscoveryEngine;

impl DiscoveryEngine {
    /// Run the discovery engine to gather all I/O data in one batch.
    ///
    /// # Arguments
    ///
    /// - `context`: File discovery results (which files exist, property bank presence)
    /// - `repo`: Repository for fetching cached views, IDs, and graph
    /// - `source`: Filesystem reader for fetching file stats
    ///
    /// # Returns
    ///
    /// A `DiscoveryOutcome` containing all data needed to initialize processors.
    ///
    /// # Errors
    ///
    /// Returns error if:
    /// - Repository batch read fails
    /// - File stats cannot be fetched for any file
    /// - Property bank path is invalid
    pub(crate) fn run<R>(
        context: &FilesContext,
        repo: &R,
        source: &FileReader,
    ) -> Result<DiscoveryOutcome, SchemaLoaderError>
    where
        R: Repository,
        R::Error: Into<SchemaRepositoryError>,
    {
        // Execute single batch transaction
        repo.with_batch_schema_reader(|batch_reader| {
            Self::run_batch(context, batch_reader, source)
        })
        .map_err(|e| SchemaLoaderError::Repository(e.into()))
    }

    /// Internal batch operation (runs within a single transaction).
    fn run_batch<E>(
        context: &FilesContext,
        batch_reader: &dyn BatchSchemaReader<Error = E>,
        source: &FileReader,
    ) -> Result<DiscoveryOutcome, SchemaLoaderError>
    where
        E: Into<SchemaRepositoryError>,
    {
        // 1. Fetch topological graph
        let graph = batch_reader
            .get_topological_graph()
            .map_err(Into::into)
            .map_err(SchemaLoaderError::Repository)?;

        // 2. Discover property bank (if exists)
        let property_bank_file = context
            .property_bank_context()
            .map(|ctx| Self::discover_property_bank(ctx, batch_reader, source))
            .transpose()?;

        // 3. Discover all schemas
        let (schema_files, filesystem_ids) =
            Self::discover_schemas(&context.files, batch_reader, source)?;

        // 4. Combine all files
        let mut files = schema_files;
        if let Some(pb_file) = property_bank_file {
            files.insert(pb_file.path.clone(), pb_file);
        }

        // 5. Detect deleted schemas
        let deleted_schemas =
            Self::detect_deleted_schemas(graph.as_ref(), &filesystem_ids);

        Ok(DiscoveryOutcome {
            files,
            graph,
            deleted_schemas,
        })
    }

    /// Discover property bank (view + file stats).
    fn discover_property_bank<E>(
        bank_context: &PropertyBankContext,
        batch_reader: &dyn BatchSchemaReader<Error = E>,
        source: &FileReader,
    ) -> Result<DiscoveredFile, SchemaLoaderError>
    where
        E: Into<SchemaRepositoryError>,
    {
        // Fetch view from DB
        let view = batch_reader
            .get_raw_property_bank_view(&bank_context.filename)
            .map_err(Into::into)
            .map_err(SchemaLoaderError::Repository)?
            .map(DiscoveredView::PropertyBank);

        // Fetch current file stats
        let file_stats = source
            .stats(bank_context.path.as_path())
            .map_err(SchemaIngestionError::from)
            .map_err(SchemaLoaderError::Ingestion)?;

        Ok(DiscoveredFile {
            filename: bank_context.filename.clone(),
            path: bank_context.path.clone(),
            id: SchemaId::new(), // Property bank uses a synthetic ID (not indexed)
            is_property_bank: true,
            view,
            file_stats,
        })
    }

    /// Discover all schemas (views + IDs + file stats).
    ///
    /// Returns:
    /// - Map of path → DiscoveredFile (for all files in context)
    /// - Set of all schema IDs found on filesystem (for deletion detection)
    fn discover_schemas<E>(
        files: &[RelativePath],
        batch_reader: &dyn BatchSchemaReader<Error = E>,
        source: &FileReader,
    ) -> Result<
        (HashMap<RelativePath, DiscoveredFile>, HashSet<SchemaId>),
        SchemaLoaderError,
    >
    where
        E: Into<SchemaRepositoryError>,
    {
        // Batch fetch views and IDs from DB
        let views_by_path = batch_reader
            .find_raw_schema_views_by_paths(files)
            .map_err(Into::into)
            .map_err(SchemaLoaderError::Repository)?;

        let ids_by_path = batch_reader
            .find_schema_ids_by_paths(files)
            .map_err(Into::into)
            .map_err(SchemaLoaderError::Repository)?;

        // Batch fetch file stats
        let stats_by_path = Self::fetch_file_stats_batch(files, source)?;

        // Combine into DiscoveredFile nodes
        let mut discovered_files = HashMap::new();
        let mut filesystem_ids = HashSet::new();

        for path in files {
            let view = views_by_path
                .get(path)
                .cloned()
                .map(DiscoveredView::Schema);
            let id = ids_by_path
                .get(path)
                .copied()
                .unwrap_or_else(SchemaId::new);
            let file_stats = stats_by_path.get(path).copied().ok_or_else(|| {
                SchemaLoaderError::Ingestion(SchemaIngestionError::File(
                    crate::schema::error::SchemaFileError::Io {
                        path: path.as_path().to_path_buf(),
                        source: std::io::Error::other("missing file stats"),
                    },
                ))
            })?;

            let filename = source
                .filename(path.as_path())
                .map_err(SchemaIngestionError::from)
                .map_err(SchemaLoaderError::Ingestion)?;

            filesystem_ids.insert(id);

            discovered_files.insert(
                path.clone(),
                DiscoveredFile {
                    filename,
                    path: path.clone(),
                    id,
                    is_property_bank: false,
                    view,
                    file_stats,
                },
            );
        }

        Ok((discovered_files, filesystem_ids))
    }

    /// Fetch file stats for all files.
    fn fetch_file_stats_batch(
        files: &[RelativePath],
        source: &FileReader,
    ) -> Result<HashMap<RelativePath, FileStats>, SchemaLoaderError> {
        let mut stats_map = HashMap::new();

        for path in files {
            let stats = source
                .stats(path.as_path())
                .map_err(SchemaIngestionError::from)
                .map_err(SchemaLoaderError::Ingestion)?;
            stats_map.insert(path.clone(), stats);
        }

        Ok(stats_map)
    }

    /// Detect schemas that were deleted from filesystem.
    ///
    /// Returns schema IDs that exist in graph but have no corresponding file.
    fn detect_deleted_schemas(
        graph: Option<&InheritanceGraph<()>>,
        filesystem_ids: &HashSet<SchemaId>,
    ) -> Vec<SchemaId> {
        let Some(graph) = graph else {
            return Vec::new();
        };

        graph
            .topo_order()
            .iter()
            .filter(|id| !filesystem_ids.contains(id))
            .copied()
            .collect()
    }
}
````

### 3. Enhanced `BatchSchemaReader` Trait

**File**: `lithos-core/src/schema/storage.rs`

Add the following methods to the `BatchSchemaReader` trait:

```rust
/// Batch reader adapter for schema tables.
pub trait BatchSchemaReader {
    /// Storage-specific error type for batch reads.
    type Error;

    // ═════════════════════════════════════════════════════════════════════
    // Existing Methods (unchanged)
    // ═════════════════════════════════════════════════════════════════════

    /// Gets the raw schema view for a given schema ID.
    fn get_raw_schema_view(
        &self,
        id: SchemaId,
    ) -> Result<Option<RawSchemaView>, Self::Error>;

    /// Gets the topological graph singleton.
    fn get_topological_graph(
        &self,
    ) -> Result<Option<InheritanceGraph<()>>, Self::Error>;

    // ═════════════════════════════════════════════════════════════════════
    // NEW Methods for DiscoveryEngine
    // ═════════════════════════════════════════════════════════════════════

    /// Gets the raw property bank view.
    ///
    /// Returns `None` if the property bank has never been loaded.
    ///
    /// # Errors
    ///
    /// Returns storage-specific error if the batch read fails.
    fn get_raw_property_bank_view(
        &self,
        filename: &Filename,
    ) -> Result<Option<RawPropertyBankView>, Self::Error>;

    /// Finds multiple raw schema views by file paths (bulk query).
    ///
    /// More efficient than N individual queries as it performs reads
    /// within the current batch transaction. Returns a map of
    /// path → view for paths that have cached views.
    ///
    /// # Errors
    ///
    /// Returns storage-specific error if the batch read fails.
    fn find_raw_schema_views_by_paths(
        &self,
        file_paths: &[RelativePath],
    ) -> Result<HashMap<RelativePath, RawSchemaView>, Self::Error>;

    /// Finds multiple schema IDs by file paths (bulk query).
    ///
    /// More efficient than N individual queries as it performs reads
    /// within the current batch transaction. Returns a map of
    /// path → `SchemaId` for paths that have schemas.
    ///
    /// # Errors
    ///
    /// Returns storage-specific error if the batch read fails.
    fn find_schema_ids_by_paths(
        &self,
        file_paths: &[RelativePath],
    ) -> Result<HashMap<RelativePath, SchemaId>, Self::Error>;
}
```

**Implementation for `RedbBatchSchemaReader`**:

```rust
impl BatchSchemaReader for RedbBatchSchemaReader<'_> {
    type Error = SchemaRepositoryError;

    // ... existing methods unchanged ...

    #[inline]
    fn get_raw_property_bank_view(
        &self,
        filename: &Filename,
    ) -> Result<Option<RawPropertyBankView>, Self::Error> {
        use crate::schema::db_table::RAW_PROPERTY_BANK_VIEW;

        self.reader
            .get_owned::<RawPropertyBankView>(
                RAW_PROPERTY_BANK_VIEW,
                filename.as_str(),
            )
            .map_err(map_db_error)
    }

    #[inline]
    fn find_raw_schema_views_by_paths(
        &self,
        file_paths: &[RelativePath],
    ) -> Result<HashMap<RelativePath, RawSchemaView>, Self::Error> {
        use crate::schema::db_table::{RAW_SCHEMA_VIEWS, SCHEMA_ID_BY_PATH};

        let mut result = HashMap::new();

        for path in file_paths {
            let path_key = path.as_path().to_string_lossy();

            // Step 1: Look up SchemaId by path
            let Some(id) = self
                .reader
                .get_owned::<SchemaId>(SCHEMA_ID_BY_PATH, path_key.as_ref())
                .map_err(map_db_error)?
            else {
                continue;
            };

            // Step 2: Look up RawSchemaView by ID
            if let Some(view) = self
                .reader
                .get_owned_by_uuid::<RawSchemaView>(
                    RAW_SCHEMA_VIEWS,
                    id.into_uuid(),
                )
                .map_err(map_db_error)?
            {
                result.insert(path.clone(), view);
            }
        }

        Ok(result)
    }

    #[inline]
    fn find_schema_ids_by_paths(
        &self,
        file_paths: &[RelativePath],
    ) -> Result<HashMap<RelativePath, SchemaId>, Self::Error> {
        use crate::schema::db_table::SCHEMA_ID_BY_PATH;

        let mut result = HashMap::new();

        for path in file_paths {
            let path_key = path.as_path().to_string_lossy();
            if let Some(id) = self
                .reader
                .get_owned::<SchemaId>(SCHEMA_ID_BY_PATH, path_key.as_ref())
                .map_err(map_db_error)?
            {
                result.insert(path.clone(), id);
            }
        }

        Ok(result)
    }
}
```

### 4. `FilesContext` Enhancement

**File**: `lithos-core/src/schema/builder.rs`

```rust
#[derive(Debug, Clone)]
pub(crate) struct FilesContext {
    pub(crate) files: Vec<RelativePath>,
    pub(crate) has_property_bank: bool,
    property_bank_context: Option<PropertyBankContext>,  // NEW
}

impl FilesContext {
    #[inline]
    fn new(files: Vec<RelativePath>) -> Self {
        if files.is_empty() {
            info!(
                "No schema files found; schema processing skipped. Add a \
                 schema file (json, yaml, or toml) to enable schema \
                 validation."
            );
        }
        Self {
            files,
            has_property_bank: false,
            property_bank_context: None,  // NEW
        }
    }

    #[inline]
    fn set_property_bank(&mut self, context: PropertyBankContext) {  // MODIFIED
        self.has_property_bank = true;
        self.property_bank_context = Some(context);
    }

    /// Returns the property bank context if it exists.
    #[inline]
    #[must_use]
    pub(crate) fn property_bank_context(&self) -> Option<&PropertyBankContext> {
        self.property_bank_context.as_ref()
    }
}
```

Update `discover_files()` to populate the context:

```rust
// In Builder::discover_files()
if file_name == bank_filename {
    if has_property_bank {
        return Err(/* duplicate error */);
    }
    has_property_bank = true;

    // NEW: Store full context instead of just setting flag
    let bank_context = PropertyBankContext {
        filename: bank_filename.clone(),
        path: property_bank_path.clone(),
    };
    context.set_property_bank(bank_context);  // MODIFIED
    continue;
}
```

---

## Implementation Plan: Two-Stage Approach

### **Phase 1: Foundation (No Refactoring of Builder/Processors)**

This phase implements all new components WITHOUT touching the existing `Builder`, `PropertyBankProcessor`, or `SchemaProcessor` logic. Everything is additive and testable in isolation.

#### Task 1.1: Storage Layer Enhancement

**Files to Modify**:
- `lithos-core/src/schema/storage.rs`
- `lithos-core/src/schema/testing.rs`

**Steps**:

1. **Add methods to `BatchSchemaReader` trait** (in `storage.rs`):
   ```rust
   fn get_raw_property_bank_view(&self, filename: &Filename)
       -> Result<Option<RawPropertyBankView>, Self::Error>;

   fn find_raw_schema_views_by_paths(&self, file_paths: &[RelativePath])
       -> Result<HashMap<RelativePath, RawSchemaView>, Self::Error>;

   fn find_schema_ids_by_paths(&self, file_paths: &[RelativePath])
       -> Result<HashMap<RelativePath, SchemaId>, Self::Error>;
   ```

2. **Implement methods in `RedbBatchSchemaReader`** (in `storage.rs`):
   - Use existing table definitions
   - Follow existing error handling with `map_db_error`
   - Use two-step lookup for views (path→id→view)

3. **Implement methods in `InMemoryBatchSchemaReader`** (in `testing.rs`):
   - Delegate to `InMemoryRepository` methods
   - Follow existing lock patterns
   - Ensure thread safety

**Verification**:
```rust
#[test]
fn batch_reader_fetches_property_bank_view() {
    let repo = InMemoryRepository::new();
    let filename = Filename::try_from("property-bank.json").unwrap();

    // Save a view
    let view = RawPropertyBankView::new(/* ... */);
    repo.save_raw_property_bank_view(&filename, &view).unwrap();

    // Fetch via batch reader
    repo.with_batch_schema_reader(|reader| {
        let fetched = reader.get_raw_property_bank_view(&filename).unwrap();
        assert!(fetched.is_some());
        Ok(())
    }).unwrap();
}

#[test]
fn batch_reader_fetches_schema_views_by_paths() {
    let repo = InMemoryRepository::new();
    let paths = vec![
        RelativePath::try_from("schemas/a.toml").unwrap(),
        RelativePath::try_from("schemas/b.toml").unwrap(),
    ];

    // Save views for both paths
    // ... setup code ...

    repo.with_batch_schema_reader(|reader| {
        let views = reader.find_raw_schema_views_by_paths(&paths).unwrap();
        assert_eq!(views.len(), 2);
        Ok(())
    }).unwrap();
}

#[test]
fn batch_reader_returns_empty_for_unknown_paths() {
    let repo = InMemoryRepository::new();
    let paths = vec![RelativePath::try_from("unknown.toml").unwrap()];

    repo.with_batch_schema_reader(|reader| {
        let views = reader.find_raw_schema_views_by_paths(&paths).unwrap();
        assert!(views.is_empty());
        Ok(())
    }).unwrap();
}
```

**Acceptance Criteria**:
- ✅ All three new methods added to `BatchSchemaReader` trait
- ✅ All three methods implemented for `RedbBatchSchemaReader`
- ✅ All three methods implemented for `InMemoryBatchSchemaReader`
- ✅ Unit tests pass for all implementations
- ✅ No clippy warnings

---

#### Task 1.2: Discovery Module Implementation

**Files to Create**:
- `lithos-core/src/schema/discovery.rs` (NEW)

**Files to Modify**:
- `lithos-core/src/schema/mod.rs` (add `pub(crate) mod discovery;`)

**Steps**:

1. **Create `discovery.rs` with complete implementation**:
   - Define `DiscoveryOutcome` struct
   - Define `DiscoveredFile` struct
   - Define `DiscoveredView` enum
   - Implement `DiscoveryEngine` with all methods

2. **Implement helper methods**:
   - `DiscoveryEngine::run()` - public entry point
   - `DiscoveryEngine::run_batch()` - internal transaction logic
   - `DiscoveryEngine::discover_property_bank()` - property bank snapshot
   - `DiscoveryEngine::discover_schemas()` - schema snapshots
   - `DiscoveryEngine::fetch_file_stats_batch()` - filesystem stats
   - `DiscoveryEngine::detect_deleted_schemas()` - deletion detection

3. **Add comprehensive doc comments**:
   - Module-level documentation
   - Struct-level documentation
   - Method-level documentation with examples

**Verification**:
```rust
#[test]
fn discovery_engine_cold_start() {
    let repo = InMemoryRepository::new();
    let temp = TempDir::new().unwrap();
    let source = FileReader::new(temp.path().to_path_buf());

    // Create test files
    create_test_files(&temp, &["schemas/a.toml", "schemas/b.toml"]);

    let context = create_test_context(&["schemas/a.toml", "schemas/b.toml"]);
    let outcome = DiscoveryEngine::run(&context, &repo, &source).unwrap();

    assert!(outcome.is_cold_start());
    assert_eq!(outcome.files.len(), 2);
    assert!(outcome.files.values().all(|f| f.is_new()));
    assert!(outcome.graph.is_none());
    assert!(outcome.deleted_schemas.is_empty());
}

#[test]
fn discovery_engine_incremental_with_property_bank() {
    let repo = setup_repo_with_data();
    let temp = TempDir::new().unwrap();
    let source = FileReader::new(temp.path().to_path_buf());

    create_test_files(&temp, &[
        "schemas/a.toml",
        "property-bank.json",
    ]);

    let mut context = create_test_context(&["schemas/a.toml"]);
    context.set_property_bank(PropertyBankContext {
        filename: Filename::try_from("property-bank.json").unwrap(),
        path: RelativePath::try_from("property-bank.json").unwrap(),
    });

    let outcome = DiscoveryEngine::run(&context, &repo, &source).unwrap();

    assert!(outcome.is_incremental());
    assert_eq!(outcome.files.len(), 2); // schema + property bank
    assert_eq!(outcome.property_bank().unwrap().is_property_bank, true);
    assert_eq!(outcome.schema_files().count(), 1);
}

#[test]
fn discovery_engine_detects_deleted_schemas() {
    let repo = setup_repo_with_persisted_schemas(&["a.toml", "b.toml", "c.toml"]);
    let temp = TempDir::new().unwrap();
    let source = FileReader::new(temp.path().to_path_buf());

    // Only create a.toml and b.toml (c.toml is missing)
    create_test_files(&temp, &["schemas/a.toml", "schemas/b.toml"]);

    let context = create_test_context(&["schemas/a.toml", "schemas/b.toml"]);
    let outcome = DiscoveryEngine::run(&context, &repo, &source).unwrap();

    assert_eq!(outcome.deleted_schemas.len(), 1);
}

#[test]
fn discovery_engine_uses_single_transaction() {
    let repo = MockRepository::new();
    let temp = TempDir::new().unwrap();
    let source = FileReader::new(temp.path().to_path_buf());

    create_test_files(&temp, &["schemas/a.toml"]);
    let context = create_test_context(&["schemas/a.toml"]);

    let _ = DiscoveryEngine::run(&context, &repo, &source);

    // Verify only ONE transaction was created
    assert_eq!(repo.transaction_count(), 1);
}
```

**Acceptance Criteria**:
- ✅ `discovery.rs` module created with all components
- ✅ `DiscoveryOutcome`, `DiscoveredFile`, `DiscoveredView` fully implemented
- ✅ `DiscoveryEngine` with all helper methods
- ✅ Comprehensive unit tests (cold start, incremental, property bank, deleted schemas)
- ✅ No clippy warnings
- ✅ All doc comments present

---

#### Task 1.3: FilesContext Enhancement

**Files to Modify**:
- `lithos-core/src/schema/builder.rs`

**Steps**:

1. **Add `property_bank_path` field to `FilesContext`**:
   - Store property bank as `Option<RelativePath>`
   - Keep discovery path-scoped; no `PropertyBankContext` dependency
   - Maintain backward compatibility with `has_property_bank` accessor

2. **Update `set_property_bank()` method**:
   - Accept `path: RelativePath`
   - Set `has_property_bank = true` and `property_bank_path = Some(path)`

3. **Add `property_bank_path()` accessor**:
   - Returns `Option<&RelativePath>`
   - Used by `DiscoveryEngine`

4. **Update `discover_files()` implementation**:
   - Capture property bank relative path when found
   - Pass path to `set_property_bank()`

**Verification**:
```rust
#[test]
fn files_context_stores_property_bank_context() {
    let mut context = FilesContext::new(vec![]);

    assert!(context.property_bank_context().is_none());
    assert!(!context.has_property_bank);

    let bank_context = PropertyBankContext {
        filename: Filename::try_from("property-bank.json").unwrap(),
        path: RelativePath::try_from("property-bank.json").unwrap(),
    };

    context.set_property_bank(bank_context.clone());

    assert!(context.has_property_bank);
    assert!(context.property_bank_context().is_some());
    assert_eq!(
        context.property_bank_context().unwrap().filename,
        bank_context.filename
    );
}

#[test]
fn discover_files_populates_property_bank_context() {
    let temp = TempDir::new().unwrap();
    create_schema_files(&temp, &["schema_a.toml", "property_bank.json"]);

    let repo = InMemoryRepository::new();
    let config = setup_test_config(&temp);
    let source = FileReader::new(temp.path().to_path_buf());
    let builder = Builder::new(repo, source, &config);

    let context = builder.discover_files().unwrap();

    assert!(context.has_property_bank);
    assert!(context.property_bank_context().is_some());

    let pb_ctx = context.property_bank_context().unwrap();
    assert_eq!(pb_ctx.filename.as_str(), "property_bank.json");
}
```

**Acceptance Criteria**:
- ✅ `FilesContext` enhanced with `property_bank_context` field
- ✅ `set_property_bank()` updated to store full context
- ✅ `property_bank_context()` accessor added
- ✅ `discover_files()` updated to populate context
- ✅ All existing tests still pass
- ✅ New tests for context storage pass

---

### **Phase 1 Summary**

At the end of Phase 1, we have:
- ✅ Enhanced `BatchSchemaReader` trait with 3 new methods
- ✅ Implementations for both `RedbBatchSchemaReader` and `InMemoryBatchSchemaReader`
- ✅ Complete `discovery.rs` module with all components
- ✅ Enhanced `FilesContext` with property bank context storage
- ✅ Comprehensive unit tests for all new components
- ✅ No changes to `Builder`, `PropertyBankProcessor`, or `SchemaProcessor`

**All tests pass. Ready for Phase 2.**

---

### **Phase 2: Integration (Refactoring Builder & Processors)**

This phase refactors `Builder`, `PropertyBankProcessor`, and `SchemaProcessor` to use the new `DiscoveryEngine`.

#### Task 2.1: Builder Refactoring

**Files to Modify**:
- `lithos-core/src/schema/builder.rs`

**Steps**:

1. **Simplify `load_all()` method**:

```rust
pub fn load_all(&mut self) -> Result<Vec<Arc<Schema>>, SchemaLoaderError> {
    use super::discovery::DiscoveryEngine;

    // Step 1: Discover files (unchanged)
    let files_context = self.discover_files()?;

    // Step 2: SINGLE unified discovery operation
    let discovery_outcome = DiscoveryEngine::run(
        &files_context,
        &self.repository,
        &self.source,
    )?;

    // Step 3: Load property bank using discovery data
    let property_bank = if let Some((pb_path, pb_file)) = discovery_outcome.property_bank() {
        Some(self.load_property_bank_from_discovery(pb_path, pb_file)?)
    } else {
        None
    };

    // Step 4: Early exit if no schemas
    if !discovery_outcome.has_schemas() {
        return Ok(Vec::new());
    }

    let property_bank = property_bank.unwrap_or_else(PropertyBank::new);

    // Step 5: Delete removed schemas
    for deleted_id in &discovery_outcome.deleted_schemas {
        self.repository
            .delete_schema(*deleted_id)
            .map_err(|e| SchemaLoaderError::Repository(e.into()))?;
    }

    // Step 6: Process schemas based on cold-start vs incremental
    if discovery_outcome.is_cold_start() {
        self.process_cold_start(&discovery_outcome, &property_bank)
    } else {
        self.process_incremental(&discovery_outcome, &property_bank)
    }
}
```

2. **Remove old `discover_graph()` method**:
   - Graph is now in `DiscoveryOutcome`
   - No longer needed

3. **Add new helper methods**:

```rust
/// Load property bank from discovery data.
fn load_property_bank_from_discovery(
    &mut self,
    discovered_path: &RelativePath,
    discovered_file: &DiscoveredFile,
) -> Result<PropertyBank, SchemaLoaderError> {
    use crate::schema::property_bank_processor::{
        PropertyBankProcessor, Discovery, Unknown,
    };

    // Convert DiscoveredFile to processor-specific types
    let pipeline = PropertyBankProcessor::<Discovery, Unknown>::new();
    let branch = pipeline.from_discovery(discovered_file)?;

    // Process through existing pipeline
    let (completed, delta) = match branch {
        ComparisonBranch::Missing(p) => {
            self.handle_missing(p, &discovered_file.filename,
                discovered_path.as_path())?
        }
        ComparisonBranch::Present(p) => {
            self.handle_present(p, &discovered_file.filename,
                discovered_path.as_path())?
        }
    };

    self.property_bank_delta = delta;
    Ok(completed)
}

/// Process cold-start (all schemas are new).
fn process_cold_start(
    &self,
    outcome: &DiscoveryOutcome,
    bank: &PropertyBank,
) -> Result<Vec<Arc<Schema>>, SchemaLoaderError> {
    use super::schema_processor::{
        SchemaProcessor, Discovery, NeverSeen,
    };

    let branch = SchemaProcessor::<Discovery, NeverSeen>::from_discovery(
        outcome.schema_files().collect(),
    )?;

    match branch {
        DiscoveryBranch::AllMissing(missing) => {
            let parsed_new = missing.parse(&self.source)?;
            let new_build = parsed_new.build_new_graph()?;
            new_build.construct_new_schemas(&self.repository, bank)
        }
        _ => unreachable!("cold-start always returns AllMissing"),
    }
}

/// Process incremental update (some schemas exist).
fn process_incremental(
    &self,
    outcome: &DiscoveryOutcome,
    bank: &PropertyBank,
) -> Result<Vec<Arc<Schema>>, SchemaLoaderError> {
    use super::schema_processor::{
        SchemaProcessor, Discovery, Review,
    };

    let graph = outcome.graph.as_ref().ok_or_else(|| {
        SchemaLoaderError::Ingestion(SchemaIngestionError::File(
            SchemaFileError::FileSystem {
                reason: "incremental update requires graph".into(),
            }
        ))
    })?;

    let branch = SchemaProcessor::<Discovery, Review>::from_discovery(
        outcome.schema_files().collect(),
        graph.clone(),
    )?;

    match branch {
        DiscoveryBranch::HasPresent(present) => {
            let compared = present.compare(
                &self.source,
                self.property_bank_delta.as_ref(),
            )?;
            let parsed = compared.parse(&self.source)?;
            let graphed = parsed.build_graph()?;
            let analyzed = graphed.analyze_properties(
                &self.source,
                bank,
                self.property_bank_delta.as_ref(),
            )?;
            let refreshed = analyzed.refresh_metadata(&self.repository)?;
            let constructed = refreshed.construct_schemas(&self.repository, bank)?;
            let schemas = constructed.complete(&self.repository)?.into_schemas();
            Ok(schemas)
        }
        _ => unreachable!("incremental always returns HasPresent"),
    }
}
```

**Verification**:
```rust
#[test]
fn builder_uses_discovery_engine_cold_start() {
    let temp = TempDir::new().unwrap();
    create_schema_files(&temp, &["schema_a.toml"]);

    let repo = InMemoryRepository::new();
    let config = setup_test_config(&temp);
    let source = FileReader::new(temp.path().to_path_buf());
    let mut builder = Builder::new(repo, source, &config);

    let schemas = builder.load_all().unwrap();

    assert_eq!(schemas.len(), 1);
}

#[test]
fn builder_uses_discovery_engine_incremental() {
    let temp = TempDir::new().unwrap();
    let repo = setup_repo_with_persisted_schemas(&["schema_a.toml"]);

    create_schema_files(&temp, &["schema_a.toml", "schema_b.toml"]);

    let config = setup_test_config(&temp);
    let source = FileReader::new(temp.path().to_path_buf());
    let mut builder = Builder::new(repo, source, &config);

    let schemas = builder.load_all().unwrap();

    assert_eq!(schemas.len(), 2);
}

#[test]
fn builder_deletes_schemas_removed_from_filesystem() {
    let temp = TempDir::new().unwrap();
    let repo = setup_repo_with_persisted_schemas(&["a.toml", "b.toml"]);

    // Only create a.toml (b.toml was deleted)
    create_schema_files(&temp, &["schema_a.toml"]);

    let config = setup_test_config(&temp);
    let source = FileReader::new(temp.path().to_path_buf());
    let mut builder = Builder::new(repo.clone(), source, &config);

    let _ = builder.load_all().unwrap();

    // Verify b.toml was deleted from repository
    assert_eq!(repo.schema_count(), 1);
}
```

**Acceptance Criteria**:
- ✅ `load_all()` simplified to use `DiscoveryEngine`
- ✅ Old `discover_graph()` method removed
- ✅ New helper methods added and tested
- ✅ All existing builder tests pass
- ✅ Integration tests for cold-start and incremental paths pass

---

#### Task 2.2: PropertyBankProcessor Integration

**Files to Modify**:
- `lithos-core/src/schema/property_bank_processor.rs`

**Steps**:

1. **Add new entry point for `DiscoveredFile`**:

```rust
impl PropertyBankProcessor<Discovery, Unknown> {
    /// Initialize from discovered file.
    ///
    /// This bypasses the I/O-heavy `discover()` method and directly
    /// enters the pipeline at the appropriate stage.
    ///
    /// # Errors
    ///
    /// Returns error if the discovered file is not a property bank.
    pub(crate) fn from_discovery(
        discovered: &DiscoveredFile,
    ) -> Result<ComparisonBranch, SchemaLoaderError> {
        if discovered.kind != SchemaFileKind::PropertyBank {
            return Err(SchemaLoaderError::Ingestion(
                SchemaIngestionError::File(SchemaFileError::FileSystem {
                    reason: "expected property bank, got schema file".into(),
                })
            ));
        }

        match &discovered.view {
            None => {
                // No cached view → Missing state
                Ok(ComparisonBranch::Missing(Self::transition(
                    Parsed,
                    Missing {
                        stats: discovered.file_stats,
                    },
                )))
            }
            Some(DiscoveredView::PropertyBank(view)) => {
                // Cached view exists → Present state
                Ok(ComparisonBranch::Present(Self::transition(
                    Comparison,
                    Present {
                        stats: discovered.file_stats,
                        view: view.clone(),
                    },
                )))
            }
            Some(DiscoveredView::Schema(_)) => {
                Err(SchemaLoaderError::Ingestion(
                    SchemaIngestionError::File(SchemaFileError::FileSystem {
                        reason: "discovered file kind/view mismatch: property-bank kind required property-bank view"
                            .into(),
                    })
                ))
            }
        }
    }
}
```

2. **Keep existing `discover()` method** (deprecated but maintained for now)

**Verification**:
```rust
#[test]
fn property_bank_processor_from_discovery_missing() {
    let discovered = DiscoveredFile {
        filename: Filename::try_from("property-bank.json").unwrap(),
        path: RelativePath::try_from("property-bank.json").unwrap(),
        id: SchemaId::new(),
        is_property_bank: true,
        view: None,
        file_stats: FileStats::new(None, None, 100),
    };

    let branch = PropertyBankProcessor::<Discovery, Unknown>::from_discovery(&discovered)
        .unwrap();

    assert!(matches!(branch, ComparisonBranch::Missing(_)));
}

#[test]
fn property_bank_processor_from_discovery_present() {
    let view = RawPropertyBankView::new(/* ... */);

    let discovered = DiscoveredFile {
        filename: Filename::try_from("property-bank.json").unwrap(),
        path: RelativePath::try_from("property-bank.json").unwrap(),
        id: SchemaId::new(),
        is_property_bank: true,
        view: Some(DiscoveredView::PropertyBank(view)),
        file_stats: FileStats::new(None, None, 100),
    };

    let branch = PropertyBankProcessor::<Discovery, Unknown>::from_discovery(&discovered)
        .unwrap();

    assert!(matches!(branch, ComparisonBranch::Present(_)));
}

#[test]
fn property_bank_processor_from_discovery_rejects_schema() {
    let discovered = DiscoveredFile {
        filename: Filename::try_from("schema.toml").unwrap(),
        path: RelativePath::try_from("schema.toml").unwrap(),
        id: SchemaId::new(),
        is_property_bank: false, // Schema, not property bank
        view: None,
        file_stats: FileStats::new(None, None, 100),
    };

    let result = PropertyBankProcessor::<Discovery, Unknown>::from_discovery(&discovered);
    assert!(result.is_err());
}
```

**Acceptance Criteria**:
- ✅ `from_discovery()` method added
- ✅ Handles Missing and Present states correctly
- ✅ Validates `SchemaFileKind` before branching
- ✅ Rejects kind/view mismatches with explicit errors
- ✅ Unit tests pass
- ✅ Integration with Builder works

---

#### Task 2.3: SchemaProcessor Integration

**Files to Modify**:
- `lithos-core/src/schema/schema_processor.rs`

**Steps**:

1. **Add entry points for discovery outcome**:

```rust
impl SchemaProcessor<Discovery, NeverSeen> {
    /// Initialize from discovered files (cold-start path).
    ///
    /// Creates a batch of new schemas from the discovery outcome.
    pub(crate) fn from_discovery(
        discovered_files: Vec<(&RelativePath, &DiscoveredFile)>,
    ) -> Result<DiscoveryBranch, SchemaLoaderError> {
        let mut new_schemas = NewBatch::new();

        for (path, file) in discovered_files {
            if file.kind == SchemaFileKind::PropertyBank {
                continue; // Skip property bank
            }

            new_schemas.insert(
                file.id,
                InitialScan {
                    path: path.clone(),
                    stats: file.file_stats,
                },
            );
        }

        Ok(DiscoveryBranch::AllMissing(Self::transition(
            FileParsed,
            AllMissing { new_schemas },
        )))
    }
}

impl SchemaProcessor<Discovery, Review> {
    /// Initialize from discovered files (incremental path).
    ///
    /// Builds the processing graph with Present payloads for existing schemas.
    pub(crate) fn from_discovery(
        discovered_files: Vec<(&RelativePath, &DiscoveredFile)>,
        graph: InheritanceGraph<()>,
    ) -> Result<DiscoveryBranch, SchemaLoaderError> {
        let mut builder = SchemaGraphBuilder::new();
        let mut new_schemas = NewBatch::new();

        for (path, file) in discovered_files {
            if file.kind == SchemaFileKind::PropertyBank {
                continue; // Skip property bank
            }

            if let Some(DiscoveredView::Schema(view)) = &file.view {
                // Existing schema with cached view
                let payload = PipelinePayload::Present(PresentPayload::Found(
                    FoundPayload {
                        path: path.clone(),
                        stats: file.file_stats,
                        view: view.clone(),
                    },
                ));

                builder.add_node(
                    file.id,
                    ProcessorNode::new(
                        NodeStatus::Fresh,
                        ExtendsChangeKind::Unchanged,
                        payload,
                    ),
                );
            } else {
                // New schema (no cached view)
                new_schemas.insert(
                    file.id,
                    InitialScan {
                        path: path.clone(),
                        stats: file.file_stats,
                    },
                );
            }
        }

        // Add edges from old graph
        for (child_id, &()) in graph.iter() {
            for &parent_id in graph.parents_of(child_id) {
                builder.add_parent(child_id, parent_id);
            }
        }

        let present_graph = builder.build();

        Ok(DiscoveryBranch::HasPresent(Self::transition(
            Comparison,
            Present {
                graph: present_graph,
                new_schemas,
                deleted_ids: Vec::new(), // Already handled by Builder
            },
        )))
    }
}
```

2. **Keep existing `discover()` methods** (deprecated but maintained for now)

**Verification**:
```rust
#[test]
fn schema_processor_from_discovery_cold_start() {
    let files = vec![
        DiscoveredFile {
            filename: Filename::try_from("a.toml").unwrap(),
            path: RelativePath::try_from("schemas/a.toml").unwrap(),
            id: SchemaId::new(),
            is_property_bank: false,
            view: None,
            file_stats: FileStats::new(None, None, 100),
        },
        DiscoveredFile {
            filename: Filename::try_from("b.toml").unwrap(),
            path: RelativePath::try_from("schemas/b.toml").unwrap(),
            id: SchemaId::new(),
            is_property_bank: false,
            view: None,
            file_stats: FileStats::new(None, None, 100),
        },
    ];

    let branch = SchemaProcessor::<Discovery, NeverSeen>::from_discovery(files)
        .unwrap();

    match branch {
        DiscoveryBranch::AllMissing(processor) => {
            assert_eq!(processor.status.new_schemas.len(), 2);
        }
        _ => panic!("expected AllMissing"),
    }
}

#[test]
fn schema_processor_from_discovery_incremental() {
    let view = RawSchemaView::new(/* ... */);

    let files = vec![
        DiscoveredFile {
            filename: Filename::try_from("a.toml").unwrap(),
            path: RelativePath::try_from("schemas/a.toml").unwrap(),
            id: SchemaId::new(),
            is_property_bank: false,
            view: Some(DiscoveredView::Schema(view)),
            file_stats: FileStats::new(None, None, 100),
        },
    ];

    let graph = InheritanceGraph::default();
    let branch = SchemaProcessor::<Discovery, Review>::from_discovery(files, graph)
        .unwrap();

    match branch {
        DiscoveryBranch::HasPresent(processor) => {
            assert_eq!(processor.status.graph.node_count(), 1);
        }
        _ => panic!("expected HasPresent"),
    }
}

#[test]
fn schema_processor_from_discovery_skips_property_bank() {
    let files = vec![
        DiscoveredFile {
            filename: Filename::try_from("property-bank.json").unwrap(),
            path: RelativePath::try_from("property-bank.json").unwrap(),
            id: SchemaId::new(),
            is_property_bank: true, // Property bank
            view: None,
            file_stats: FileStats::new(None, None, 100),
        },
        DiscoveredFile {
            filename: Filename::try_from("a.toml").unwrap(),
            path: RelativePath::try_from("schemas/a.toml").unwrap(),
            id: SchemaId::new(),
            is_property_bank: false, // Schema
            view: None,
            file_stats: FileStats::new(None, None, 100),
        },
    ];

    let branch = SchemaProcessor::<Discovery, NeverSeen>::from_discovery(files)
        .unwrap();

    match branch {
        DiscoveryBranch::AllMissing(processor) => {
            // Only schema file, property bank skipped
            assert_eq!(processor.status.new_schemas.len(), 1);
        }
        _ => panic!("expected AllMissing"),
    }
}
```

**Acceptance Criteria**:
- ✅ `from_discovery()` methods added for both NeverSeen and Review
- ✅ Correctly handles cold-start vs incremental paths
- ✅ Skips property bank files
- ✅ Uses `(path, file)` pairs from `DiscoveryOutcome.files` (path-keyed input)
- ✅ Rejects kind/view mismatch payloads
- ✅ Unit tests pass
- ✅ Integration with Builder works

---

#### Task 2.4: Cleanup and Documentation

**Files to Modify**:
- `lithos-core/src/schema/property_bank_processor.rs`
- `lithos-core/src/schema/schema_processor.rs`
- `lithos-core/src/schema/builder.rs`

**Steps**:

1. **Mark old discovery methods as deprecated**:
   ```rust
   #[deprecated(
       since = "0.2.0",
       note = "Use `from_discovery()` instead. Old method will be removed in 0.3.0"
   )]
   pub(crate) fn discover(/* ... */) -> Result</* ... */> {
       // ... existing implementation ...
   }
   ```

2. **Update documentation** to reference new patterns:
   - `DiscoveredFile` is metadata-only (no embedded path)
   - Path is sourced from `DiscoveryOutcome.files` keys
   - Kind branching uses `SchemaFileKind`, not `is_property_bank`
   - Discovery remains path-scoped batch lookup (no SchemaIndex-primary switch)

3. **Consider removal in future version** (Phase 3, not part of this plan)

**Acceptance Criteria**:
- ✅ Old methods marked as deprecated
- ✅ Documentation updated
- ✅ No breaking changes (old methods still work)

---

### **Phase 2 Summary**

At the end of Phase 2, we have:
- ✅ `Builder` refactored to use `DiscoveryEngine`
- ✅ `PropertyBankProcessor` with `from_discovery()` entry point
- ✅ `SchemaProcessor` with `from_discovery()` entry points
- ✅ All existing tests pass
- ✅ Integration tests for full pipeline pass
- ✅ 66% reduction in repository transactions (3 → 1)

**System fully integrated. Ready for deployment.**

---

## Testing Strategy

### Unit Tests (Phase 1)

**Coverage Requirements**:
- ✅ `BatchSchemaReader` methods: 100%
- ✅ `DiscoveryEngine` methods: 100%
- ✅ `FilesContext` enhancements: 100%

**Key Test Cases**:
1. Cold start (no data in DB)
2. Incremental (data exists in DB)
3. Property bank present/missing
4. Deleted schema detection
5. Empty file list
6. Large file list (1000+ schemas)
7. Error propagation (bad paths, repository errors)
8. Single transaction verification

### Integration Tests (Phase 2)

**Coverage Requirements**:
- ✅ Full pipeline (discovery → processing → storage): 100%
- ✅ Builder orchestration: 100%

**Key Test Cases**:
1. Cold start with property bank
2. Incremental with deleted schemas
3. Incremental with new schemas
4. Property bank staleness detection
5. Schema staleness detection
6. Mixed fresh/stale schemas

### Performance Tests

**Benchmarks**:
- ✅ Discovery engine batch fetch (10, 100, 1000 schemas)
- ✅ File stats collection (sequential vs parallel)
- ✅ Memory usage for large `DiscoveryOutcome`

**Regression Tests**:
- ✅ Ensure transaction count reduced from 3 to 1
- ✅ Ensure no performance degradation in file stats collection

---

## Error Handling

### Error Propagation Strategy

All errors follow existing patterns:

- **Repository errors**: Wrapped in `SchemaLoaderError::Repository`
- **Filesystem errors**: Wrapped in `SchemaLoaderError::Ingestion`
- **Fail-fast**: No partial results returned
- **Atomic transactions**: All repository reads in one transaction

### Error Types

```rust
pub enum SchemaLoaderError {
    /// Repository operation failed.
    Repository(SchemaRepositoryError),

    /// File ingestion failed.
    Ingestion(SchemaIngestionError),

    // ... other variants ...
}
```

### Partial Failure Handling

The `DiscoveryEngine` uses **fail-fast semantics**:

- If ANY file stat cannot be fetched → return error immediately
- If repository batch read fails → return error immediately
- No partial results are returned

This ensures consistent state and prevents silent data loss.

---

## Performance Characteristics

### Before vs After

| Metric                   | Before                            | After                  | Improvement |
| ------------------------ | --------------------------------- | ---------------------- | ----------- |
| Repository transactions  | 3 (PropertyBank + Schema + Graph) | 1 (single batch)       | **66% ↓**   |
| File stat syscalls       | O(N) (sequential)                 | O(N) (can parallelize) | Same        |
| Graph queries            | 1 separate query                  | Batched with views     | **Faster**  |
| Property bank view query | 1 separate query                  | Batched with schemas   | **Faster**  |
| Memory usage             | Minimal (streaming)               | ~1 MB for 1000 schemas | Acceptable  |

### Memory Analysis

The `DiscoveryOutcome` holds all discovery data in memory:

- Property bank file: ~1 KB
- Schema files: ~1 KB per schema × N schemas
- File stats: ~48 bytes per file × N files
- Graph: ~small (only metadata)

**For 1000 schemas**: ~1 MB total (acceptable).

### Optimization Opportunities

1. **Parallel file stats** (future enhancement):
   ```rust
   use rayon::prelude::*;

   let stats_map: HashMap<_, _> = files
       .par_iter()
       .map(|path| {
           let stats = source.stats(path.as_path())?;
           Ok((path.clone(), stats))
       })
       .collect::<Result<_, _>>()?;
   ```

2. **Streaming discovery** (future enhancement):
   - Return iterator instead of complete `HashMap`
   - Reduces memory for very large vaults (10,000+ schemas)

---

## Rollback Plan

If issues arise during Phase 2:

### Rollback Procedure

1. **Revert Builder changes**:
   - Restore old `load_all()` implementation
   - Restore `discover_graph()` method

2. **Keep Phase 1 components**:
   - `DiscoveryEngine` remains (unused but harmless)
   - `BatchSchemaReader` enhancements remain (backward compatible)

3. **Feature flag approach** (alternative):
   ```rust
   #[cfg(feature = "unified-discovery")]
   pub fn load_all(&mut self) -> Result</* ... */> {
       // New implementation
   }

   #[cfg(not(feature = "unified-discovery"))]
   pub fn load_all(&mut self) -> Result</* ... */> {
       // Old implementation
   }
   ```

### Rollback Testing

- ✅ Ensure all tests pass with old implementation
- ✅ Verify no data corruption
- ✅ Confirm performance characteristics unchanged

---

## Dependencies

### External Dependencies

**No new external dependencies required.**

Existing dependencies used:
- `std::collections::HashMap` (stdlib)
- `std::collections::HashSet` (stdlib)
- `tracing` (logging)
- `rkyv` (serialization)
- `redb` (database)

### Internal Dependencies

- `crate::fs::{FileStats, Filename, FileReader, RelativePath}`
- `crate::schema::{identifier::SchemaId, inheritance::InheritanceGraph}`
- `crate::schema::views::{RawPropertyBankView, RawSchemaView, RawView}`
- `crate::schema::storage::{BatchSchemaReader, Repository}`

---

## Documentation Updates

### 1. Module Documentation

**File**: `lithos-core/src/schema/discovery.rs`

Add comprehensive module-level documentation:

```rust
//! Unified discovery engine for schema and property bank ingestion.
//!
//! This module consolidates ALL I/O operations from the fragmented discovery
//! logic into a single `DiscoveryEngine` component that performs batch fetches
//! in one atomic repository transaction.
//!
//! # Architecture
//!
//! The discovery process has two phases:
//!
//! 1. **File Discovery** (`Builder::discover_files()`):
//!    - Scans filesystem for schema files and property bank
//!    - Returns `FilesContext` with file paths
//!
//! 2. **Metadata Discovery** (`DiscoveryEngine::run()`):
//!    - Fetches cached views, IDs, and graph from repository
//!    - Fetches file stats from filesystem
//!    - Detects deleted schemas
//!    - Returns `DiscoveryOutcome` with all data
//!
//! # Performance
//!
//! The unified design reduces repository transactions from 3 to 1:
//! - Before: Property bank query + Schema views query + Graph query
//! - After: Single batch read with all queries
//!
//! # Usage
//!
//! ```ignore
//! let files_context = builder.discover_files()?;
//! let outcome = DiscoveryEngine::run(&files_context, &repo, &source)?;
//!
//! if outcome.is_cold_start() {
//!     // Process all schemas as new
//! } else {
//!     // Process incremental updates
//! }
//! ```
```

### 2. Architecture Guide Update

**File**: `docs/architecture/schema-ingestion.md` (create if doesn't exist)

Document the new discovery architecture with diagrams.

### 3. ADR Documentation

**File**: `docs/adr/0XX-unified-discovery-engine.md` (create)

Create ADR documenting:
- Context: Fragmented discovery logic
- Decision: Consolidate into single engine
- Consequences: Performance improvement, simplified orchestration
- Alternatives considered: Status quo, SchemaIndex-first discovery (rejected), parallel execution
- Rationale: SchemaIndex is not the primary discovery source because path-scoped existence/deletion checks require filesystem + path-keyed lookup as source of truth

### 4. Inline Documentation

Add examples to key types:

```rust
/// # Examples
///
/// ```
/// use lithos_core::schema::discovery::DiscoveryEngine;
///
/// let outcome = DiscoveryEngine::run(&context, &repo, &source)?;
/// println!("Discovered {} schemas", outcome.schema_files().count());
/// ```
```

---

## Success Criteria

### Phase 1 Success Criteria

- [✅] All `BatchSchemaReader` methods implemented for both backends
- [✅] Complete `discovery.rs` module with all components
- [✅] `FilesContext` enhanced with property bank context
- [✅] 100% unit test coverage for new components
- [✅] No clippy warnings
- [✅] All existing tests still pass

### Phase 2 Success Criteria

- [✅] `Builder` refactored to use `DiscoveryEngine`
- [✅] Processors have `from_discovery()` entry points
- [✅] All existing tests still pass
- [✅] Integration tests for full pipeline pass
- [✅] Performance benchmarks show improvement (3 → 1 transactions)
- [✅] No regressions in error handling or staleness detection

### Overall Success Criteria

- [✅] 66% reduction in repository transactions
- [✅] Simplified `Builder` orchestration (linear flow)
- [✅] No breaking changes to public API
- [✅] Comprehensive documentation
- [✅] All tests pass (unit + integration + performance)

---

## Timeline

### Phase 1: Foundation (1-1.5 days)

- **Task 1.1**: Storage layer enhancement (4 hours)
- **Task 1.2**: Discovery module implementation (6 hours)
- **Task 1.3**: FilesContext enhancement (2 hours)

### Phase 2: Integration (1-1.5 days)

- **Task 2.1**: Builder refactoring (4 hours)
- **Task 2.2**: PropertyBankProcessor integration (2 hours)
- **Task 2.3**: SchemaProcessor integration (4 hours)
- **Task 2.4**: Cleanup (2 hours)

### Total Estimated Effort: 2-3 days

---

## Risks and Mitigations

### Risk 1: Performance Regression

**Risk**: Batching all data in memory could cause OOM for large vaults.

**Mitigation**:
- Benchmark with 1000+ schemas
- Add streaming mode if needed (future enhancement)
- Document memory usage limits

### Risk 2: Breaking Changes

**Risk**: Refactoring could introduce subtle bugs or breaking changes.

**Mitigation**:
- Keep old methods during transition (deprecated but functional)
- Comprehensive integration tests
- Feature flag for rollback

### Risk 3: Test Coverage Gaps

**Risk**: New code might not be fully covered by tests.

**Mitigation**:
- Mandate 100% unit test coverage for Phase 1
- Add integration tests for full pipeline in Phase 2
- Use `tarpaulin` to verify coverage

---

## Appendix A: File Checklist

### Files to Create

- [✅] `lithos-core/src/schema/discovery.rs`

### Files to Modify

- [✅] `lithos-core/src/schema/mod.rs`
- [✅] `lithos-core/src/schema/storage.rs`
- [✅] `lithos-core/src/schema/testing.rs`
- [✅] `lithos-core/src/schema/builder.rs`
- [✅] `lithos-core/src/schema/property_bank_processor.rs`
- [✅] `lithos-core/src/schema/schema_processor.rs`

### Files to Review

- [✅] `lithos-core/src/schema/views.rs` (ensure `RawView` trait is sufficient)
- [✅] `lithos-core/src/schema/error.rs` (ensure error types cover new cases)

---

## Appendix B: Implementation Checklist

### Phase 1 Tasks

- [ ] Task 1.1: Storage Layer Enhancement
  - [ ] Add `get_raw_property_bank_view()` to `BatchSchemaReader`
  - [ ] Add `find_raw_schema_views_by_paths()` to `BatchSchemaReader`
  - [ ] Add `find_schema_ids_by_paths()` to `BatchSchemaReader`
  - [ ] Implement all methods in `RedbBatchSchemaReader`
  - [ ] Implement all methods in `InMemoryBatchSchemaReader`
  - [ ] Write unit tests for all implementations
  - [ ] Run `mise run test:unit:schema`
  - [ ] Run `mise run lint`

- [ ] Task 1.2: Discovery Module Implementation
  - [ ] Create `discovery.rs` with module docs
  - [ ] Define `DiscoveryOutcome` struct
  - [ ] Define `DiscoveredFile` struct
  - [ ] Define `DiscoveredView` enum
  - [ ] Implement `DiscoveryEngine::run()`
  - [ ] Implement `DiscoveryEngine::run_batch()`
  - [ ] Implement `DiscoveryEngine::discover_property_bank()`
  - [ ] Implement `DiscoveryEngine::discover_schemas()`
  - [ ] Implement `DiscoveryEngine::fetch_file_stats_batch()`
  - [ ] Implement `DiscoveryEngine::detect_deleted_schemas()`
  - [ ] Add comprehensive unit tests
  - [ ] Run `mise run test:unit:schema`
  - [ ] Run `mise run lint`

- [ ] Task 1.3: FilesContext Enhancement
  - [ ] Add `property_bank_context` field to `FilesContext`
  - [ ] Update `set_property_bank()` method
  - [ ] Add `property_bank_context()` accessor
  - [ ] Update `discover_files()` implementation
  - [ ] Write unit tests
  - [ ] Run `mise run test:unit:schema`
  - [ ] Run `mise run lint`

### Phase 2 Tasks

- [ ] Task 2.1: Builder Refactoring
  - [ ] Simplify `load_all()` method
  - [ ] Remove `discover_graph()` method
  - [ ] Add `load_property_bank_from_discovery()` helper
  - [ ] Add `process_cold_start()` helper
  - [ ] Add `process_incremental()` helper
  - [ ] Write integration tests
  - [ ] Run `mise run test:integration`
  - [ ] Run `mise run lint`

- [ ] Task 2.2: PropertyBankProcessor Integration
  - [ ] Add `from_discovery()` entry point
  - [ ] Write unit tests
  - [ ] Run `mise run test:unit:schema`
  - [ ] Run `mise run lint`

- [ ] Task 2.3: SchemaProcessor Integration
  - [ ] Add `from_discovery()` to NeverSeen
  - [ ] Add `from_discovery()` to Review
  - [ ] Write unit tests
  - [ ] Run `mise run test:unit:schema`
  - [ ] Run `mise run lint`

- [ ] Task 2.4: Cleanup
  - [ ] Mark old methods as deprecated
  - [ ] Update documentation
  - [ ] Run `mise run verify`

### Final Verification

- [ ] Run full test suite: `mise run test`
- [ ] Run linter: `mise run lint`
- [ ] Run formatter: `mise run fmt`
- [ ] Run full verification: `mise run verify`
- [ ] Verify no clippy warnings
- [ ] Verify no test failures
- [ ] Review all doc comments
- [ ] Update CHANGELOG.md

---

## End of Implementation Plan
