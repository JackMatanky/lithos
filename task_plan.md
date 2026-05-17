# Task Plan: Update SchemaConfigSpec to Store Absolute Paths

## Goal
Refactor `SchemaConfigSpec` to store `PathBuf` instead of `RelativePath`, with both the schema directory and property bank file paths resolved as absolute paths (joined to VaultRoot), eliminating the need to pass VaultRoot separately to builder.rs and discovery.rs.

## Current Phase
Phase 3 (TDD - Write Failing Tests)

## Phases

### Phase 1: Requirements & Impact Analysis
- [x] Understand current SchemaConfigSpec structure (stores RelativePath)
- [x] Identify all consumers (builder.rs:61, discovery.rs:197,244,652)
- [x] Run impact analysis on SchemaConfigSpec (RESULT: LOW risk, 0 direct dependents)
- [x] Understand to_schema_spec construction in aggregate.rs:226
- [x] Document findings in findings.md
- **Status:** complete

### Phase 2: Design & Planning
- [x] Design new SchemaConfigSpec API (DirPath/FilePath fields)
- [x] Plan migration path for existing tests
- [x] Identify all test files needing updates (4 files, ~8 tests)
- [x] Document design decisions in findings.md
- [x] Create isolated worktree for implementation
- [x] Verify clean baseline (1157 unit + 38 integration tests passing)
- [x] Create comprehensive TDD_PLAN.md with all test cases
- **Status:** complete

### Phase 3: TDD - Write Failing Tests
- [ ] Test 1.1: SchemaConfigSpec accepts DirPath and FilePath (paths.rs)
- [ ] Test 2.1: to_schema_spec creates absolute paths from vault root (aggregate.rs)
- [ ] Test 3.1: DiscoveryEngine::run works without vault_root parameter (discovery.rs)
- [ ] Test 4.1: Builder::load_all works with new signature (builder.rs)
- [ ] Test 5.1: Update accepts_read_repository_only (discovery.rs tests)
- [ ] Test 5.2: Update run_skips_schema_batch_lookups test (discovery.rs tests)
- [ ] All tests should fail initially (RED phase)
- **Status:** pending
- **See:** TDD_PLAN.md in worktree for detailed test code

### Phase 4: Implementation - Make Tests Pass
- [ ] Impl 1: SchemaConfigSpec struct (DirPath/FilePath fields, const accessors)
- [ ] Impl 2: Config::to_schema_spec() joins vault root with relative paths
- [ ] Impl 3: DiscoveryEngine::run() removes vault_root param, uses spec paths
- [ ] Impl 4: Builder::load_all() removes vault_root argument
- [ ] Impl 5: Update all discovery test call sites (3 tests)
- [ ] All tests should pass (GREEN phase)
- **Status:** pending
- **See:** TDD_PLAN.md in worktree for implementation code

### Phase 5: Refactor & Clean Up
- [ ] Remove vault_root parameter from DiscoveryEngine::run()
- [ ] Update all doc comments and examples
- [ ] Run `mise run verify` (fmt + lint + tests)
- [ ] Address any clippy warnings
- [ ] Verify no string allocation anti-patterns
- **Status:** pending

### Phase 6: Verification
- [ ] Run full test suite (`mise run test`)
- [ ] Run linter (`mise run lint`)
- [ ] Run formatter check (`mise run fmt`)
- [ ] Review git diff for unintended changes
- [ ] Document test results in progress.md
- **Status:** pending

## Key Questions
1. Should SchemaConfigSpec store `PathBuf` or `Path`?
   - **Answer:** Use `DirPath` and `FilePath` (type-safe, domain-appropriate)
2. Should we keep accessor methods or make fields public?
   - **Answer:** Keep accessors for encapsulation (directory(), property_bank())
3. Does this change affect the rkyv serialization?
   - **Answer:** No, SchemaConfigSpec is not serialized (no Archive derive)
4. Do we need to update the property_bank_path logic in Paths?
   - **Answer:** Yes, but that stays relative - SchemaConfigSpec gets absolute paths
5. How to construct DirPath/FilePath without filesystem validation?
   - **Answer:** Use `From<PathBuf>` trait (bypasses .is_file()/.is_dir() checks)

## Decisions Made
| Decision | Rationale |
|----------|-----------|
| Use DirPath for schema directory | Type-safe, domain-appropriate, wraps PathBuf |
| Use FilePath for property bank | Type-safe, domain-appropriate, wraps PathBuf |
| Use From<PathBuf> for construction | Bypasses filesystem validation (safe for joined paths) |
| Keep accessor methods | Maintains encapsulation, follows existing pattern |
| Resolve paths in to_schema_spec() | Single point of vault root joining, clear responsibility |
| Remove vault_root from DiscoveryEngine::run() | Impact: 3 call sites (LOW risk), paths now in spec |

## Errors Encountered
| Error | Attempt | Resolution |
|-------|---------|------------|
|       |         |            |

## Notes
- Impact analysis shows LOW risk (0 direct dependents on SchemaConfigSpec struct)
- Main consumers: builder.rs, discovery.rs, aggregate.rs
- TDD workflow: RED (failing tests) → GREEN (passing impl) → REFACTOR
- Must maintain type-driven design: validated constructors, private fields
