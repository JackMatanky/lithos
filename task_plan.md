# Task Plan: Schema Discovery Refactoring - Consolidate Discovery Logic

## Goal
Refactor schema discovery to route all discovery logic through a unified `DiscoveryEngine` in `discovery.rs`, eliminating duplicate filesystem/database scanning capabilities from `builder.rs`, `property_bank_processor.rs`, and `schema_processor.rs`.

## Current Phase
Phase 3

## Phases

### Phase 1: Analysis & Discovery
- [x] Understand refactoring scope and objectives
- [x] Map current discovery responsibilities across all files
- [x] Identify duplicate capabilities and overlapping logic
- [x] Document dependencies and call chains
- [x] Identify what DiscoveryEngine already provides
- [x] Document gaps between current DiscoveryEngine and needed capabilities
- **Status:** complete

### Phase 2: Design Unified Discovery API
- [x] Design DiscoveryEngine API to support all use cases
- [x] Define what discovery data builder.rs needs
- [x] Define what discovery data property_bank_processor.rs needs
- [x] Define what discovery data schema_processor.rs needs
- [x] Plan how to eliminate `discover_files()` from builder.rs
- [x] Plan how to eliminate `discover()` from property_bank_processor.rs
- [x] Plan how to eliminate discovery logic from schema_processor.rs
- [x] Document the new unified flow
- **Status:** complete

### Phase 3: Refactor DiscoveryEngine
- [x] Rename `DiscoveryOutcome` → `DiscoveryResult`
- [x] Create new types: `SchemaDiscovery`, `PropertyBankDiscovery`, `SchemaCachedState`, `CachedState`
- [x] Decompose `run()` into 5 focused methods: scan_filesystem, separate_property_bank, query_cached_state, build_result, detect_deleted_schemas
- [x] Update `run()` method signature and implementation
- [x] Remove `FileDiscovery` wrapper type
- [x] Update tests for new structure
- [x] Run `mise run test:unit:schema` - ALL TESTS PASS
- **Status:** complete

### Phase 4: Refactor Builder to Use DiscoveryEngine
- [ ] Replace `discover_files()` with DiscoveryEngine call
- [ ] Replace `discover_graph()` with DiscoveryEngine data
- [ ] Update `load_all()` to use unified DiscoveryOutcome
- [ ] Remove duplicate DirScanner usage from builder.rs
- [ ] Update tests in builder.rs
- [ ] Verify all tests pass: `mise run test:unit:schema`
- **Status:** pending

### Phase 5: Refactor PropertyBankProcessor Discovery Phase
- [ ] Remove filesystem scanning from property_bank_processor.rs
- [ ] Update `discover()` to accept DiscoveredFile data
- [ ] Ensure processor only handles comparison/parsing/construction
- [ ] Update tests in property_bank_processor.rs
- [ ] Verify all tests pass: `mise run test:unit:schema`
- **Status:** pending

### Phase 6: Refactor SchemaProcessor Discovery Phase
- [ ] Remove discovery logic from schema_processor.rs
- [ ] Update discovery methods to accept DiscoveryOutcome data
- [ ] Ensure processor only handles comparison/parsing/graphing
- [ ] Update tests in schema_processor.rs
- [ ] Verify all tests pass: `mise run test:unit:schema`
- **Status:** pending

### Phase 7: Integration & Verification
- [ ] Run full test suite: `mise run test`
- [ ] Run quality checks: `mise run quality`
- [ ] Verify no regressions in functionality
- [ ] Check for any remaining duplicate code
- [ ] Run full verification: `mise run verify`
- [ ] Document architectural changes (ADR if needed)
- **Status:** pending

### Phase 8: Final Review & Cleanup
- [ ] Review all modified files for consistency
- [ ] Ensure naming conventions followed
- [ ] Verify no unwrap()/panic!() in production code
- [ ] Ensure all public APIs documented
- [ ] Final verification: `mise run verify`
- [ ] Mark refactoring complete
- **Status:** pending

## Key Questions
1. What discovery data does builder.rs currently gather? (filesystem scan, graph from DB, property bank detection)
2. What discovery data does property_bank_processor.rs currently gather? (property bank file info, cached view)
3. What discovery data does schema_processor.rs currently gather? (schema files with cached views, deleted schemas)
4. Does DiscoveryEngine already provide all of this? (YES - DiscoveryOutcome has files, graph, deleted_schemas)
5. Can DiscoveryEngine be the single entry point? (YES - needs API to extract specific discovery data)
6. Will this eliminate duplicate DirScanner usage? (YES - single scan in DiscoveryEngine)
7. Will this eliminate duplicate Repository queries? (YES - single batch read in DiscoveryEngine)
8. How to handle property bank vs schema separation? (DiscoveryOutcome.property_bank() already provides this)

## Decisions Made
| Decision | Rationale |
|----------|-----------|
| Use DiscoveryEngine as single source of truth | Already implements atomic discovery with batch operations; eliminates duplication |
| Keep DiscoveryEngine in discovery.rs | Clear separation of concerns; discovery is infrastructure, processors are pipeline stages |
| Route all discovery through Builder.load_all() | Builder orchestrates the pipeline; single call to DiscoveryEngine simplifies flow |
| Processors receive discovered data, don't discover | Typestate processors should focus on transformation stages, not I/O |
| Preserve zero-copy patterns from DiscoveryEngine | Uses `with_archived()` pattern for performance-critical paths |

## Errors Encountered
| Error | Attempt | Resolution |
|-------|---------|------------|
|       | 1       |            |

## Notes
- DiscoveryEngine already exists and provides unified discovery via `DiscoveryEngine::run()`
- DiscoveryOutcome contains: files (HashMap), graph (Option), deleted_schemas (Vec)
- DiscoveredFile contains: kind, view (Option), info (FileInfo)
- SchemaFileKind differentiates PropertyBank from Schema(SchemaId)
- Property bank separation already handled via DiscoveryOutcome::property_bank()
- Builder currently does manual DirScanner calls - these should be eliminated
- PropertyBankProcessor has Discovery stage - should receive data instead
- SchemaProcessor has Discovery stage - should receive data instead
- All three locations use similar DirScanner patterns - prime for consolidation
