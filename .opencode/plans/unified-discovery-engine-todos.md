# Unified Discovery Engine - Implementation Todos

## Phase 1: Foundation (No Refactoring of Builder/Processors)

---

## Commit 1: Storage Layer Enhancement (BatchSchemaReader methods)

**Objective**: Add 3 new batch read methods to support unified discovery without touching existing builder/processor logic.

### Files to Modify/Create
- `lithos-core/src/schema/storage.rs`
- `lithos-core/src/schema/testing.rs`

### Tasks
- [ ] Add `get_raw_property_bank_view()` method signature to `BatchSchemaReader` trait in `storage.rs`
  - Accept `filename: &Filename` parameter
  - Return `Result<Option<RawPropertyBankView>, Self::Error>`
  - Add comprehensive doc comment with `# Errors` section
- [ ] Add `find_raw_schema_views_by_paths()` method signature to `BatchSchemaReader` trait
  - Accept `file_paths: &[RelativePath]` parameter
  - Return `Result<HashMap<RelativePath, RawSchemaView>, Self::Error>`
  - Document bulk query efficiency in doc comment
- [ ] Add `find_schema_ids_by_paths()` method signature to `BatchSchemaReader` trait
  - Accept `file_paths: &[RelativePath]` parameter
  - Return `Result<HashMap<RelativePath, SchemaId>, Self::Error>`
  - Document bulk query efficiency in doc comment
- [ ] Implement `get_raw_property_bank_view()` in `RedbBatchSchemaReader` (in `storage.rs`)
  - Use `RAW_PROPERTY_BANK_VIEW` table constant
  - Call `self.reader.get_owned::<RawPropertyBankView>(RAW_PROPERTY_BANK_VIEW, filename.as_str())`
  - Map error with `map_db_error` wrapper
- [ ] Implement `find_raw_schema_views_by_paths()` in `RedbBatchSchemaReader`
  - Create empty `HashMap` for results
  - Iterate over `file_paths`
  - For each path: lookup SchemaId via `SCHEMA_ID_BY_PATH` table
  - If ID found: lookup RawSchemaView via `RAW_SCHEMA_VIEWS` table using `get_owned_by_uuid()`
  - Insert found views into result map
  - Handle errors with `map_db_error`
- [ ] Implement `find_schema_ids_by_paths()` in `RedbBatchSchemaReader`
  - Create empty `HashMap` for results
  - Iterate over `file_paths` and lookup each ID via `SCHEMA_ID_BY_PATH` table
  - Insert found IDs into result map
  - Handle errors with `map_db_error`
- [ ] Implement all 3 methods in `InMemoryBatchSchemaReader` (in `testing.rs`)
  - Delegate to existing `InMemoryRepository` internal data structures
  - Follow existing lock acquisition patterns for thread safety
  - Use `.clone()` when returning views/IDs from internal storage
- [ ] Write unit test `batch_reader_fetches_property_bank_view()` in `testing.rs`
  - Create `InMemoryRepository`, save a property bank view, fetch via batch reader
  - Assert `Some(view)` returned
- [ ] Write unit test `batch_reader_fetches_schema_views_by_paths()` in `testing.rs`
  - Setup repo with 2 schemas at different paths
  - Fetch both via batch reader
  - Assert map contains 2 entries with correct views
- [ ] Write unit test `batch_reader_returns_empty_for_unknown_paths()` in `testing.rs`
  - Query batch reader for paths that don't exist
  - Assert empty HashMap returned (not an error)
- [ ] Write unit test `batch_reader_fetches_schema_ids_by_paths()` in `testing.rs`
  - Setup repo with schemas, fetch IDs via batch reader
  - Assert correct IDs returned

### Verification
- [ ] Run `mise run test:unit:schema` - all tests pass
- [ ] Run `mise run lint` - no clippy warnings
- [ ] Run `mise run fmt` - code formatted correctly
- [ ] Verify test coverage includes all 3 new methods for both implementations

### Acceptance Criteria
- [ ] All 3 method signatures added to `BatchSchemaReader` trait with comprehensive docs
- [ ] All 3 methods implemented correctly in `RedbBatchSchemaReader` with proper error handling
- [ ] All 3 methods implemented correctly in `InMemoryBatchSchemaReader` with thread safety
- [ ] At least 4 unit tests written and passing
- [ ] No clippy warnings introduced
- [ ] Existing `BatchSchemaReader` tests still pass (no regressions)

---

## Commit 2: Discovery Module Implementation (DiscoveryEngine core)

**Objective**: Create the complete `discovery.rs` module with all data structures and engine logic in isolation.

### Files to Modify/Create
- `lithos-core/src/schema/discovery.rs` (NEW - create entire file)
- `lithos-core/src/schema/mod.rs` (add module declaration)

### Tasks
- [ ] Create `lithos-core/src/schema/discovery.rs` with module-level documentation
  - Document the unified discovery architecture
  - Explain the 66% transaction reduction benefit
  - Include usage example in doc comment
- [ ] Define `DiscoveredView` enum with proper derives
  - `Schema(RawSchemaView)` variant
  - `PropertyBank(RawPropertyBankView)` variant
  - Derive `Debug, Clone`
- [ ] Define `DiscoveredFile` struct with all fields
  - `filename: Filename`
  - `path: RelativePath`
  - `id: SchemaId`
  - `is_property_bank: bool`
  - `view: Option<DiscoveredView>`
  - `file_stats: FileStats`
  - Derive `Debug, Clone`
  - Add comprehensive struct-level doc comment explaining unified design
- [ ] Implement `DiscoveredFile::is_timestamp_match()` method
  - Return `false` if `view.is_none()`
  - Pattern match on `DiscoveredView` enum to call appropriate `is_timestamp_match()` for Schema or PropertyBank
  - Mark with `#[must_use]` attribute
- [ ] Implement `DiscoveredFile::is_new()` method
  - Return `self.view.is_none()`
  - Mark with `#[must_use]` and `#[inline]`
- [ ] Implement `DiscoveredFile::as_schema_view()` method
  - Return `Option<&RawSchemaView>` via pattern matching
  - Mark with `#[must_use]` and `#[inline]`
- [ ] Implement `DiscoveredFile::as_property_bank_view()` method
  - Return `Option<&RawPropertyBankView>` via pattern matching
  - Mark with `#[must_use]` and `#[inline]`
- [ ] Define `DiscoveryOutcome` struct with all fields
  - `files: HashMap<RelativePath, DiscoveredFile>`
  - `graph: Option<InheritanceGraph<()>>`
  - `deleted_schemas: Vec<SchemaId>`
  - Derive `Debug`
  - Add comprehensive struct-level doc comment with invariants section
- [ ] Implement `DiscoveryOutcome::is_cold_start()` method
  - Check if all files have `view.is_none()` AND `graph.is_none()`
  - Mark with `#[must_use]` and `#[inline]`
- [ ] Implement `DiscoveryOutcome::is_incremental()` method
  - Return `!self.is_cold_start()`
  - Mark with `#[must_use]` and `#[inline]`
- [ ] Implement `DiscoveryOutcome::has_schemas()` method
  - Return true if any file has `is_property_bank == false`
  - Mark with `#[must_use]` and `#[inline]`
- [ ] Implement `DiscoveryOutcome::property_bank()` method
  - Find and return first file where `is_property_bank == true`
  - Mark with `#[must_use]` and `#[inline]`
- [ ] Implement `DiscoveryOutcome::schema_files()` method
  - Return iterator filtering out property bank files
  - Mark with `#[inline]`
- [ ] Define `DiscoveryEngine` as empty struct (zero-sized type)
- [ ] Implement `DiscoveryEngine::run()` public entry point
  - Accept `context: &FilesContext`, `repo: &R`, `source: &FileReader`
  - Generic over `R: Repository` with error bound
  - Call `repo.with_batch_schema_reader()` wrapping `run_batch()`
  - Map repository errors to `SchemaLoaderError::Repository`
  - Add comprehensive doc comment with # Arguments, # Returns, # Errors sections
- [ ] Implement `DiscoveryEngine::run_batch()` internal method
  - Accept `context`, `batch_reader: &dyn BatchSchemaReader`, `source`
  - Step 1: Fetch topological graph via `batch_reader.get_topological_graph()`
  - Step 2: Call `discover_property_bank()` if property bank exists in context
  - Step 3: Call `discover_schemas()` for all schema files
  - Step 4: Combine results into single `files` HashMap
  - Step 5: Call `detect_deleted_schemas()` to find removed files
  - Return `DiscoveryOutcome`
- [ ] Implement `DiscoveryEngine::discover_property_bank()` helper
  - Fetch view via `batch_reader.get_raw_property_bank_view()`
  - Wrap view in `DiscoveredView::PropertyBank` if present
  - Fetch file stats via `source.stats()`
  - Generate synthetic `SchemaId` (property bank doesn't use it for indexing)
  - Return `DiscoveredFile` with `is_property_bank = true`
- [ ] Implement `DiscoveryEngine::discover_schemas()` helper
  - Batch fetch views via `batch_reader.find_raw_schema_views_by_paths()`
  - Batch fetch IDs via `batch_reader.find_schema_ids_by_paths()`
  - Call `fetch_file_stats_batch()` for all files
  - Build `HashMap<RelativePath, DiscoveredFile>` combining all data
  - Extract filenames via `source.filename()`
  - Generate new `SchemaId` for files without cached ID
  - Collect all IDs into `HashSet` for deletion detection
  - Return tuple `(HashMap, HashSet)`
- [ ] Implement `DiscoveryEngine::fetch_file_stats_batch()` helper
  - Iterate over paths and call `source.stats()` for each
  - Build `HashMap<RelativePath, FileStats>`
  - Propagate IO errors as `SchemaLoaderError::Ingestion`
  - NOTE: Sequential for now, can be parallelized later
- [ ] Implement `DiscoveryEngine::detect_deleted_schemas()` helper
  - Return empty Vec if `graph.is_none()`
  - Iterate over graph topology
  - Filter IDs that are NOT in `filesystem_ids` set
  - Return `Vec<SchemaId>` of deleted schemas
- [ ] Add module declaration to `lithos-core/src/schema/mod.rs`: `pub(crate) mod discovery;`
- [ ] Write unit test `discovery_engine_cold_start()`
  - Setup: Empty repo, temp dir with 2 schema files, no property bank
  - Run discovery engine
  - Assert `is_cold_start() == true`, `files.len() == 2`, all files `is_new() == true`, `graph.is_none()`, `deleted_schemas.is_empty()`
- [ ] Write unit test `discovery_engine_incremental_with_property_bank()`
  - Setup: Repo with persisted data, temp dir with 1 schema + property bank
  - Run discovery engine
  - Assert `is_incremental() == true`, `files.len() == 2`, property bank found via `property_bank()`, `schema_files().count() == 1`
- [ ] Write unit test `discovery_engine_detects_deleted_schemas()`
  - Setup: Repo with 3 persisted schemas, filesystem with only 2 files
  - Run discovery engine
  - Assert `deleted_schemas.len() == 1` with correct ID
- [ ] Write unit test `discovery_engine_uses_single_transaction()`
  - Use mock repository that counts transactions
  - Run discovery engine
  - Assert transaction count == 1
- [ ] Write unit test `discovery_engine_handles_empty_file_list()`
  - Empty `FilesContext`
  - Run discovery, assert no errors and empty outcome

### Verification
- [ ] Run `mise run test:unit:schema` - all new tests pass
- [ ] Run `mise run lint` - no clippy warnings
- [ ] Run `mise run fmt` - code formatted
- [ ] Run `cargo test --doc` - doc examples compile (if any added)
- [ ] Verify all public methods have doc comments with proper sections

### Acceptance Criteria
- [ ] `discovery.rs` module created with comprehensive module-level docs
- [ ] All 3 data structures (`DiscoveryOutcome`, `DiscoveredFile`, `DiscoveredView`) fully implemented
- [ ] `DiscoveryEngine` with all 6 methods implemented correctly
- [ ] At least 5 comprehensive unit tests covering cold-start, incremental, property bank, deletions, and single transaction
- [ ] No clippy warnings
- [ ] All helper methods have proper error handling and propagation
- [ ] Code follows Rust idioms (proper use of `#[must_use]`, `#[inline]`, borrowing patterns)

---

## Commit 3: FilesContext Enhancement (Property Bank Context Storage)

**Objective**: Enhance `FilesContext` to store full property bank context instead of just a boolean flag.

### Files to Modify/Create
- `lithos-core/src/schema/builder.rs`

### Tasks
- [ ] Add `property_bank_context: Option<PropertyBankContext>` field to `FilesContext` struct
  - Place after `has_property_bank: bool` field
  - Keep `has_property_bank` for backward compatibility
- [ ] Initialize `property_bank_context: None` in `FilesContext::new()`
- [ ] Modify `FilesContext::set_property_bank()` method signature
  - Change parameter from nothing to `context: PropertyBankContext`
  - Set `self.has_property_bank = true`
  - Set `self.property_bank_context = Some(context)`
- [ ] Add new `FilesContext::property_bank_context()` accessor method
  - Return `Option<&PropertyBankContext>`
  - Mark with `#[inline]` and `#[must_use]`
  - Add doc comment explaining usage by `DiscoveryEngine`
- [ ] Update `Builder::discover_files()` method where property bank is detected
  - Find the section: `if file_name == bank_filename { ... }`
  - After duplicate check, construct `PropertyBankContext`:
    ```rust
    let bank_context = PropertyBankContext {
        filename: bank_filename.clone(),
        path: property_bank_path.clone(),
    };
    ```
  - Call `context.set_property_bank(bank_context)` instead of `context.set_property_bank()`
- [ ] Write unit test `files_context_stores_property_bank_context()`
  - Create empty `FilesContext`
  - Assert `property_bank_context().is_none()` and `!has_property_bank`
  - Create `PropertyBankContext` with test data
  - Call `set_property_bank(context)`
  - Assert `has_property_bank == true` and `property_bank_context().is_some()`
  - Verify stored context has correct filename and path
- [ ] Write unit test `discover_files_populates_property_bank_context()`
  - Create temp dir with schema file + property bank file
  - Run `builder.discover_files()`
  - Assert returned context has property bank
  - Assert `property_bank_context()` returns correct filename and path
- [ ] Write unit test `files_context_without_property_bank_returns_none()`
  - Create temp dir with only schema files
  - Run `builder.discover_files()`
  - Assert `property_bank_context().is_none()`

### Verification
- [ ] Run `mise run test:unit:schema` - all tests pass including existing ones
- [ ] Run `mise run lint` - no clippy warnings
- [ ] Run `mise run fmt` - code formatted
- [ ] Verify no breaking changes to existing code using `FilesContext`
- [ ] Check that `has_property_bank` still works for backward compatibility

### Acceptance Criteria
- [ ] `property_bank_context` field added to `FilesContext` struct
- [ ] `set_property_bank()` updated to accept and store full context
- [ ] `property_bank_context()` accessor added with proper attributes
- [ ] `discover_files()` updated to populate the context when property bank found
- [ ] At least 3 unit tests written and passing
- [ ] All existing builder tests still pass (no regressions)
- [ ] Backward compatibility maintained for `has_property_bank` accessor

---

## Phase 2: Integration (Refactoring Builder & Processors)

---

## Commit 4: Builder Refactoring (Use DiscoveryEngine)

**Objective**: Refactor `Builder::load_all()` to use the new `DiscoveryEngine`, eliminating scattered discovery logic.

### Files to Modify/Create
- `lithos-core/src/schema/builder.rs`

### Tasks
- [ ] Add import: `use super::discovery::{DiscoveryEngine, DiscoveredFile};`
- [ ] Rewrite `Builder::load_all()` to use unified discovery flow:
  - Step 1: Call `self.discover_files()` (unchanged)
  - Step 2: Call `DiscoveryEngine::run(&files_context, &self.repository, &self.source)?`
  - Step 3: Load property bank if present using `load_property_bank_from_discovery()`
  - Step 4: Early return if `!discovery_outcome.has_schemas()`
  - Step 5: Delete removed schemas by iterating `discovery_outcome.deleted_schemas`
  - Step 6: Branch on `discovery_outcome.is_cold_start()` vs incremental
  - Call `process_cold_start()` or `process_incremental()` accordingly
- [ ] Remove old `Builder::discover_graph()` method (no longer needed)
- [ ] Add new private helper `Builder::load_property_bank_from_discovery()`
  - Accept `(path, discovered_file): (&RelativePath, &DiscoveredFile)`
  - Use `PropertyBankProcessor::<Discovery, Unknown>::new()`
  - Call `from_discovery(discovered_file)?` to get branch
  - Pattern match on `ComparisonBranch::{Missing, Present}`
  - Delegate to existing `handle_missing()` and `handle_present()` methods
  - Store delta in `self.property_bank_delta`
  - Return `PropertyBank`
- [ ] Add new private helper `Builder::process_cold_start()`
  - Accept `outcome: &DiscoveryOutcome`, `bank: &PropertyBank`
  - Collect schema entries via `outcome.schema_files().collect()`
  - Pass `(path, file)` pairs into `SchemaProcessor::<Discovery, NeverSeen>::from_discovery()`
  - Call `SchemaProcessor::<Discovery, NeverSeen>::from_discovery()`
  - Unwrap `DiscoveryBranch::AllMissing` (or use `expect` with message)
  - Follow existing cold-start pipeline: `parse()` → `build_new_graph()` → `construct_new_schemas()`
  - Return `Vec<Arc<Schema>>`
- [ ] Add new private helper `Builder::process_incremental()`
  - Accept `outcome: &DiscoveryOutcome`, `bank: &PropertyBank`
  - Extract graph from outcome or return error if missing
  - Collect schema entries via `outcome.schema_files().collect()`
  - Pass `(path, file)` pairs into `SchemaProcessor::<Discovery, Review>::from_discovery()`
  - Call `SchemaProcessor::<Discovery, Review>::from_discovery()`
  - Unwrap `DiscoveryBranch::HasPresent` (or use `expect` with message)
  - Follow existing incremental pipeline: `compare()` → `parse()` → `build_graph()` → `analyze_properties()` → `refresh_metadata()` → `construct_schemas()` → `complete()`
  - Return `Vec<Arc<Schema>>`
- [ ] Write integration test `builder_uses_discovery_engine_cold_start()`
  - Setup: Empty repo, temp dir with 1 schema file
  - Create builder and call `load_all()`
  - Assert 1 schema returned
  - Verify schema loaded correctly
- [ ] Write integration test `builder_uses_discovery_engine_incremental()`
  - Setup: Repo with 1 persisted schema, filesystem with 2 schemas (1 existing + 1 new)
  - Call `load_all()`
  - Assert 2 schemas returned
- [ ] Write integration test `builder_deletes_schemas_removed_from_filesystem()`
  - Setup: Repo with 2 persisted schemas
  - Filesystem has only 1 schema (1 deleted)
  - Call `load_all()`
  - Verify repo now contains only 1 schema (deletion occurred)
- [ ] Write integration test `builder_processes_property_bank_via_discovery()`
  - Setup: Filesystem with schema + property bank
  - Call `load_all()`
  - Verify property bank was loaded and delta recorded
- [ ] Update existing builder tests if needed to ensure compatibility
- [ ] Add integration test `builder_uses_path_key_for_processor_input()`
  - Assert builder passes `DiscoveryOutcome.files` path keys (not internal file path fields)

### Verification
- [ ] Run `mise run test:unit:schema` - all unit tests pass
- [ ] Run `mise run test:integration` - all integration tests pass
- [ ] Run `mise run lint` - no clippy warnings
- [ ] Run `mise run fmt` - code formatted
- [ ] Manually trace through code to verify only 1 repository transaction occurs
- [ ] Verify no regressions in error handling or staleness detection

### Acceptance Criteria
- [ ] `load_all()` simplified to ~30 lines using `DiscoveryEngine`
- [ ] Old `discover_graph()` method removed
- [ ] 3 new helper methods added with proper doc comments
- [ ] At least 4 integration tests written and passing
- [ ] All existing builder tests still pass (no regressions)
- [ ] Code follows linear orchestration pattern (no complex branching)
- [ ] Error handling preserved for all failure cases

---

## Commit 5: PropertyBankProcessor and SchemaProcessor Integration

**Objective**: Add `from_discovery()` entry points to both processors to accept `DiscoveredFile` data directly.

### Files to Modify/Create
- `lithos-core/src/schema/property_bank_processor.rs`
- `lithos-core/src/schema/schema_processor.rs`

### Tasks (PropertyBankProcessor)
- [ ] Add import: `use super::discovery::{DiscoveredFile, DiscoveredView};`
- [ ] Add `PropertyBankProcessor::<Discovery, Unknown>::from_discovery()` method
  - Accept `discovered: &DiscoveredFile`
  - Return `Result<ComparisonBranch, SchemaLoaderError>`
  - Validate `discovered.kind == SchemaFileKind::PropertyBank`, return error if false
  - Match on `discovered.view`:
    - `None` → return `ComparisonBranch::Missing` with `Parsed` stage and `Missing { stats }` status
    - `Some(DiscoveredView::PropertyBank(view))` → return `ComparisonBranch::Present` with `Comparison` stage and `Present { stats, view }` status
  - `Some(DiscoveredView::Schema(_))` → return error (kind/view mismatch)
  - Add explicit guard for impossible combinations (PropertyBank kind + Schema view)
  - Add comprehensive doc comment explaining bypass of I/O-heavy `discover()` method
- [ ] Keep existing `discover()` method (mark as deprecated in future commit)
- [ ] Write unit test `property_bank_processor_from_discovery_missing()`
  - Create `DiscoveredFile` with `view: None`, `kind: SchemaFileKind::PropertyBank`
  - Call `from_discovery()`
  - Assert returns `ComparisonBranch::Missing`
- [ ] Write unit test `property_bank_processor_from_discovery_present()`
  - Create `DiscoveredFile` with property bank view, `kind: SchemaFileKind::PropertyBank`
  - Call `from_discovery()`
  - Assert returns `ComparisonBranch::Present` with correct view
- [ ] Write unit test `property_bank_processor_from_discovery_rejects_schema()`
  - Create `DiscoveredFile` with `kind: SchemaFileKind::Schema`
  - Call `from_discovery()`
  - Assert returns error
- [ ] Write unit test `property_bank_processor_from_discovery_rejects_kind_view_mismatch()`
  - Create `DiscoveredFile` with `kind: SchemaFileKind::PropertyBank` and `DiscoveredView::Schema`
  - Assert error variant clearly indicates mismatch

### Tasks (SchemaProcessor - NeverSeen)
- [ ] Add import: `use super::discovery::{DiscoveredFile, DiscoveredView};`
- [ ] Add `SchemaProcessor::<Discovery, NeverSeen>::from_discovery()` method
  - Accept `discovered_files: Vec<(&RelativePath, &DiscoveredFile)>`
  - Return `Result<DiscoveryBranch, SchemaLoaderError>`
  - Create empty `NewBatch`
  - Iterate over `discovered_files`, skip `SchemaFileKind::PropertyBank`
  - For each schema file: insert into `NewBatch` with `InitialScan { path: path.clone(), stats }`
  - Return `DiscoveryBranch::AllMissing` with `FileParsed` stage
  - Add comprehensive doc comment explaining cold-start path
- [ ] Write unit test `schema_processor_from_discovery_cold_start()`
  - Create 2 `DiscoveredFile` instances with `view: None`, `kind: SchemaFileKind::Schema`
  - Call `from_discovery()`
  - Assert returns `AllMissing` with 2 entries in `new_schemas` batch

### Tasks (SchemaProcessor - Review)
- [ ] Add `SchemaProcessor::<Discovery, Review>::from_discovery()` method
  - Accept `discovered_files: Vec<(&RelativePath, &DiscoveredFile)>`, `graph: InheritanceGraph<()>`
  - Return `Result<DiscoveryBranch, SchemaLoaderError>`
  - Create `SchemaGraphBuilder` and `NewBatch`
  - Iterate over `discovered_files`, skip `SchemaFileKind::PropertyBank`
  - For files WITH `DiscoveredView::Schema(view)`:
    - Create `PipelinePayload::Present(PresentPayload::Found(FoundPayload { path: path.clone(), stats, view }))`
    - Add node to graph builder with `Fresh` status, `Unchanged` extends kind
  - For files WITHOUT view (new schemas):
    - Insert into `NewBatch` with `InitialScan { path: path.clone(), stats }`
  - Return error for kind/view mismatch (`SchemaFileKind::Schema` + `DiscoveredView::PropertyBank`)
  - Copy edges from old graph to builder
  - Build graph and return `DiscoveryBranch::HasPresent`
  - Add comprehensive doc comment explaining incremental path
- [ ] Write unit test `schema_processor_from_discovery_incremental()`
  - Create 1 `DiscoveredFile` with schema view (existing)
  - Create empty `InheritanceGraph`
  - Call `from_discovery()`
  - Assert returns `HasPresent` with 1 node in graph
- [ ] Write unit test `schema_processor_from_discovery_mixed_new_and_existing()`
  - Create 1 file with view (existing) + 1 file without view (new)
  - Call `from_discovery()`
  - Assert graph has 1 node and `new_schemas` has 1 entry
- [ ] Write unit test `schema_processor_from_discovery_skips_property_bank()`
  - Create 1 property bank file + 1 schema file
  - Call `from_discovery()`
  - Assert only schema file processed (property bank ignored)
- [ ] Write unit test `schema_processor_from_discovery_rejects_kind_view_mismatch()`
  - Create schema-kind file with property-bank view
  - Assert error indicates invalid discovery payload

### Verification
- [ ] Run `mise run test:unit:schema` - all tests pass
- [ ] Run `mise run test:integration` - integration tests pass
- [ ] Run `mise run lint` - no clippy warnings
- [ ] Run `mise run fmt` - code formatted
- [ ] Verify processors integrate correctly with builder (from Commit 4)

### Acceptance Criteria
- [ ] `PropertyBankProcessor` has `from_discovery()` method with proper error handling
- [ ] `SchemaProcessor<NeverSeen>` has `from_discovery()` for cold-start path
- [ ] `SchemaProcessor<Review>` has `from_discovery()` for incremental path
- [ ] At least 8 unit tests written and passing (4 for property bank, 4+ for schema processor)
- [ ] All existing processor tests still pass
- [ ] Property bank processor correctly validates `SchemaFileKind`
- [ ] Schema processor correctly skips property bank files
- [ ] Incremental processor correctly handles mixed new/existing schemas
- [ ] Both processors reject kind/view mismatch payloads with explicit errors

---

## Commit 6: Documentation, Cleanup, and Final Verification

**Objective**: Mark old methods as deprecated, add comprehensive documentation, and run full verification suite.

### Files to Modify/Create
- `lithos-core/src/schema/property_bank_processor.rs`
- `lithos-core/src/schema/schema_processor.rs`
- `lithos-core/src/schema/discovery.rs`
- `docs/adr/0XX-unified-discovery-engine.md` (NEW - create ADR)
- `CHANGELOG.md` (update)

### Tasks (Deprecation)
- [ ] Add `#[deprecated]` attribute to `PropertyBankProcessor::discover()` method
  - `#[deprecated(since = "0.2.0", note = "Use `from_discovery()` instead. Will be removed in 0.3.0")]`
- [ ] Add `#[deprecated]` attribute to `SchemaProcessor::<Discovery, NeverSeen>::discover()` method
  - Same deprecation message
- [ ] Add `#[deprecated]` attribute to `SchemaProcessor::<Discovery, Review>::discover()` method
  - Same deprecation message

### Tasks (Documentation)
- [ ] Review and enhance module-level documentation in `discovery.rs`
  - Ensure usage examples are clear and compile
  - Document the 66% transaction reduction benefit
  - Explain the unified file discovery design
- [ ] Review all doc comments in `discovery.rs` for completeness
  - Check all public types have proper doc comments
  - Verify `# Errors` sections present on fallible methods
  - Verify `# Panics` sections where appropriate
  - Add `# Examples` to key methods
- [ ] Enhance doc comment on `DiscoveredFile::is_timestamp_match()`
  - Explain when timestamps are compared (for staleness detection)
- [ ] Enhance doc comment on `DiscoveryEngine::run()`
  - Add detailed example showing usage in Builder context
  - Document single-transaction guarantee
- [ ] Create ADR: `docs/adr/0XX-unified-discovery-engine.md`
  - Title: "Unified Discovery Engine for Schema Ingestion"
  - Status: Accepted
  - Context: Fragmented discovery logic across 3 components, 3 separate transactions
  - Decision: Consolidate into single `DiscoveryEngine` with batch operations
  - Consequences: 66% reduction in transactions, simplified orchestration, better testability
  - Alternatives: Status quo (rejected - too fragmented), SchemaIndex-first discovery (rejected - not path-scoped and weaker for file-presence/deletion checks), parallel execution (rejected - complexity)
  - Clarify decision: discovery remains path-scoped batch lookup; `SchemaIndex` is supporting metadata, not primary source of truth
  - Follow existing ADR template format
- [ ] Update `CHANGELOG.md` with new entry
  - Section: `## [Unreleased]` or appropriate version
  - Add: "### Changed"
  - Entry: "Refactored schema discovery to use unified `DiscoveryEngine`, reducing repository transactions by 66% (from 3 to 1)"
  - Entry: "Added `from_discovery()` entry points to `PropertyBankProcessor` and `SchemaProcessor`"
  - Add: "### Deprecated"
  - Entry: "Deprecated `discover()` methods in processors in favor of `from_discovery()` (will be removed in 0.3.0)"

### Tasks (Final Verification)
- [ ] Run `mise run test` - ALL tests pass (unit + integration + e2e)
- [ ] Run `mise run lint` - zero clippy warnings
- [ ] Run `mise run fmt` - all code formatted
- [ ] Run `mise run verify` - full quality gate passes (fmt + lint + tests + adr:validate)
- [ ] Run `cargo test --doc` - all doc examples compile and pass
- [ ] Review code coverage report (if available via `mise run test:coverage`)
  - Ensure new code has high coverage (aim for >90%)
- [ ] Manual code review checklist:
  - [ ] No `unwrap()` or `panic!()` in production code
  - [ ] All errors properly propagated with context
  - [ ] No unnecessary `.clone()` or allocations in hot paths
  - [ ] No `.to_owned().into()` anti-patterns (use `.into()` directly)
  - [ ] Proper use of `#[inline]`, `#[must_use]` attributes
  - [ ] All public APIs have comprehensive doc comments
  - [ ] Naming follows taxonomy (no `get_` prefix on simple getters)
- [ ] Verify transaction count reduction (create test if needed):
  - [ ] Write test that counts repository transactions during `load_all()`
  - [ ] Assert count == 1 (down from 3 in old implementation)

### Tasks (Cleanup)
- [ ] Remove any debug prints, commented code, or TODOs introduced during implementation
- [ ] Ensure consistent formatting across all modified files
- [ ] Check for any unused imports or dead code (clippy should catch this)
- [ ] Verify no new dependencies were added (should use only existing crates)
- [ ] Review docs for stale `PropertyBankContext` references in discovery flow
- [ ] Review docs for stale `DiscoveredFile.path` and `is_property_bank` references

### Verification
- [ ] Run `mise run verify` - 100% green
- [ ] Run `mise run test:coverage` - verify high coverage on new code
- [ ] Run `cargo doc --open` - review generated documentation visually
- [ ] Manually test cold-start scenario with real schema files
- [ ] Manually test incremental scenario (modify schema, reload)
- [ ] Manually test deletion scenario (remove schema file, reload)

### Acceptance Criteria
- [ ] All old `discover()` methods marked as deprecated with clear migration path
- [ ] All public types in `discovery.rs` have comprehensive documentation
- [ ] ADR created documenting the architectural decision
- [ ] `CHANGELOG.md` updated with all changes
- [ ] `mise run verify` passes with zero warnings/errors
- [ ] Doc tests compile and pass
- [ ] No TODOs, debug prints, or commented code remaining
- [ ] Manual testing confirms cold-start, incremental, and deletion scenarios work
- [ ] Transaction count reduction verified (3 → 1)
- [ ] Code follows all Rust idioms and project conventions from AGENTS.md

---

## Final Definition of Done (All 6 Commits)

Before marking the implementation complete, verify:

- [ ] All 6 commits created with clear, atomic messages
- [ ] All unit tests pass (`mise run test:unit`)
- [ ] All integration tests pass (`mise run test:integration`)
- [ ] Full verification suite passes (`mise run verify`)
- [ ] No clippy warnings introduced
- [ ] Code formatted correctly (`mise run fmt`)
- [ ] All public APIs documented with examples
- [ ] ADR created and follows template
- [ ] CHANGELOG.md updated
- [ ] No `unwrap()`/`panic!()` in production code
- [ ] Context boundaries respected (no cross-imports between business contexts)
- [ ] Type-driven design maintained (private fields, validated constructors)
- [ ] No string allocation anti-patterns (`.to_owned().into()`, etc.)
- [ ] Transaction count reduced from 3 to 1 (verified by test)
- [ ] All existing builder/processor tests still pass (no regressions)
- [ ] Performance characteristics maintained or improved
- [ ] Error handling complete for all failure paths
- [ ] Memory usage acceptable for large schemas (tested with 100+ schemas)

---

## Dependencies Between Commits

**Must complete in order**:
1. Commit 1 → Commit 2 (DiscoveryEngine needs BatchSchemaReader methods)
2. Commit 2 → Commit 3 (DiscoveryEngine needs optional property-bank path in `FilesContext`)
3. Commits 1-3 → Commit 4 (Builder needs all foundation components)
4. Commit 4 → Commit 5 (Processors integrate with refactored Builder)
5. Commits 1-5 → Commit 6 (Documentation/cleanup after all implementation)

**Can work in parallel** (within phases):
- Phase 1: All 3 commits are independent after Commit 1 completes (Commits 2 and 3 can be done concurrently)
- Phase 2: Commit 5 can start as soon as Commit 4's public API is clear

---

## Rollback Plan (If Issues Arise)

If critical issues discovered after any commit:

**After Commit 1-3 (Phase 1)**:
- Safe to rollback - no existing code uses new components
- Simply revert commits, no side effects

**After Commit 4 (Builder refactored)**:
- Revert Commit 4 only
- Builder goes back to old `discover_graph()` pattern
- Phase 1 components remain (unused but harmless)

**After Commit 5 (Processors integrated)**:
- Revert Commits 4-5
- Old processor `discover()` methods still work
- Deprecation warnings harmless

**After Commit 6 (Complete)**:
- Feature flag approach: Use cargo features to toggle old/new implementation
- Keep both code paths until confident in new implementation
