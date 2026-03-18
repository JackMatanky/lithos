# Unit Test Critical Review - Ingestor & Loader (REVISED)
## Research-Backed Analysis Based on Rust Best Practices

**Date**: 2026-03-18 (Revised)
**Scope**: `lithos-core/src/schema/ingestor.rs` and `lithos-core/src/schema/loader.rs`
**Purpose**: Comprehensive review aligned with canonical Rust testing best practices

---

## Research Summary

### Sources Consulted

1. **The Rust Book (Chapter 11)**: Official Rust testing guidelines
2. **Project docs**: `docs/refs/rust/quality-tooling.md` and `TDD_QUICK_REFERENCE.md`
3. **matklad's "How to Test"**: Industry best practices from rust-analyzer author
4. **matklad's "Unit and Integration Tests"**: Purity vs Extent framework
5. **Existing project patterns**: `lithos-core/tests/common/mod.rs` test infrastructure

### Key Insights from Research

#### 1. Unit vs Integration Tests (Official Rust Definition)

**From The Rust Book**:
- **Unit tests**: Colocated with code in `#[cfg(test)]` modules, can test private functions
- **Integration tests**: In `tests/` directory, only use public API, each file is a separate crate

**From matklad's "Purity vs Extent" framework**:
- **Purity**: How much IO/concurrency is involved (pure functions → threads → filesystem → network → processes)
- **Extent**: How much code is exercised (single function → module → multiple modules → full pipeline)
- **Key insight**: Unit vs integration is a false dichotomy - optimize for purity, let extent be natural

#### 2. Test Doubles and Repository Testing

**From project research** (`lithos-core/tests/common/mod.rs`):
- Project already uses `TestDb` with RAII cleanup pattern
- Project mentions but hasn't implemented `InMemoryStorage` and `FakeStorage`
- Integration tests use real `RedbRepository` with temporary databases

**From "How to Test"**:
- Avoid mocking/stubbing your own code (reduces test fidelity)
- Mock only external systems (network, processes) not your own layers
- **Repository pattern**: Use in-memory implementation for speed, not for "unit purity"

**Conclusion**: `InMemoryRepository` or `FakeRepository` IS a valid Rust pattern, but:
- **Purpose**: Achieve test purity (eliminate filesystem/process IO), NOT to isolate code
- **Naming**: Project uses `InMemory*` for speed and `Fake*` for controlled behavior
- **Usage**: Unit tests (#[cfg(test)]) use in-memory, integration tests (tests/) use real redb

#### 3. Data-Driven Testing Pattern

**From "How to Test"**:
- Use `check()` helper function to centralize test assertions
- Makes tests resilient to API changes (single point of update)
- Enables data-driven tests (input + expected output)
- **Neural network test**: Can you replace implementation with ML and keep tests?

**From project**: Already using this pattern in integration tests (`tests/schema_resolution.rs`)

#### 4. Test Organization

**From The Rust Book**:
- Unit tests in `#[cfg(test)] mod tests` at bottom of each file
- Use submodules to organize related tests
- Common helpers in `tests/common/mod.rs` (not `tests/common.rs`!)

**From project docs** (`quality-tooling.md`):
- Tests should read like documentation
- One behavior per test
- Name tests so output reads like a sentence
- Use modules: `mod parse { #[test] fn rejects_empty_input() {} }`

---

## Revised Findings

### What Needs to Change

#### 1. Test Double Strategy - CLARIFIED

**Previous assessment**: "Need FakeRepository"
**Revised assessment**: "Need InMemoryRepository for unit tests"

**Reasoning**:
- The 4 ignored loader tests fail because they do filesystem IO (redb persistence)
- Per matklad: "Purity > Extent" - we want pure tests, not necessarily "small" tests
- An in-memory repository achieves purity WITHOUT limiting extent
- This aligns with project's existing `TestDb` pattern

**Implementation**:
```rust
// NOT for isolation, but for purity (no filesystem IO)
pub struct InMemoryRepository {
    schemas: Arc<RwLock<HashMap<SchemaId, Schema>>>,
    views: Arc<RwLock<HashMap<SchemaId, RawSchemaView>>>,
    property_bank: Arc<RwLock<Option<PropertyBank>>>,
    bank_view: Arc<RwLock<Option<RawPropertyBankView>>>,
}
```

**Where to use**:
- **Unit tests** (`#[cfg(test)]` in `ingestor.rs`, `loader.rs`): Use `InMemoryRepository`
- **Integration tests** (`lithos-core/tests/`): Use `RedbRepository` with real database

#### 2. Unit vs Integration Classification - CORRECTED

**Previous assessment**: Based on "what's mocked"
**Revised assessment**: Based on **location** and **purity**

**Unit Tests** (in `src/**/*.rs` with `#[cfg(test)]`):
- **Purity**: Pure (no IO) or near-pure (in-memory only)
- **Extent**: Can be large! Testing full pipeline is GOOD if pure
- **Can test**: Private functions, internal behavior, full orchestration
- **Use**: `InMemoryRepository`, no filesystem, no real database

**Integration Tests** (in `tests/*.rs`):
- **Purity**: Impure (real filesystem, real database, maybe network)
- **Extent**: Naturally large (exercises public API)
- **Can test**: Only public API, end-to-end scenarios, cross-module integration
- **Use**: `RedbRepository`, real temp databases, real filesystem

#### 3. The `ingest_all()` Test Gap - CRITICAL

**Why it's missing**: Tests use deprecated `all_schemas()` with `#[expect(deprecated)]`
**Why this violates best practices**:
- Per "How to Test": Test features, not code - but we're testing old API!
- Per "Neural Network Test": If we remove `all_schemas()`, these tests become useless
- Per TDD: Tests should document current behavior, not legacy APIs

**What `ingest_all()` represents**:
- Primary API from Phase 3 refactoring
- Returns `IngestorResults` with structured staleness information
- Enables optimizations (Phase 5.2 cached expansion)

**Correct approach**:
1. Write NEW unit tests for `ingest_all()` using `InMemoryRepository`
2. Keep ONE smoke test for deprecated `all_schemas()`
3. Migration guide in comments, not test coverage

#### 4. Flaky Test Pattern - FIXED

**Previous assessment**: "Tests use sleep() - flaky"
**Revised understanding**: Tests are flaky AND slow because of impurity

**Root cause analysis**:
```rust
// IMPURE: Filesystem IO + timing dependency
std::thread::sleep(Duration::from_millis(10));
write_file(...);  // Real filesystem write
let results = ingestor.ingest_all()?;  // Reads from filesystem
```

**Matklad's ladder of impurity**:
1. ✅ Pure computation (fast, never flaky)
2. ✅ Multi-threaded parallel (still fast, rarely flaky)
3. ❌ Threads + time-based sync (slower, flaky)
4. ❌ Filesystem IO (much slower, environment-dependent)
5. ❌ Process/network (slowest, very flaky)

**Current tests are at level 3-4** - worst spot on the ladder!

**Solution**: Move to level 1 with `InMemoryRepository`:
```rust
// PURE: No IO, no timing dependency
let mut repo = InMemoryRepository::new();
repo.set_bank_view(/* explicit staleness control */);
let results = ingestor.ingest_all()?;  // Reads from memory
```

---

## Test Classification Matrix

### Current Ingestor Tests (lines 1077-1877)

| Test Group | Location | Purity | Extent | Correct? | Action |
|------------|----------|--------|--------|----------|---------|
| Property bank parsing | `#[cfg(test)]` | Impure (filesystem) | Medium | ❌ No | Move to InMemory |
| Staleness detection | `#[cfg(test)]` | Impure (filesystem+sleep) | Medium | ❌ No | Move to InMemory |
| Result type accessors | `#[cfg(test)]` | Pure | Small | ✅ Yes | Keep as-is |
| `all_schemas()` tests | `#[cfg(test)]` | Impure | Large | ❌ No | Deprecated - remove |
| **MISSING: `ingest_all()`** | - | - | - | ❌ No | **ADD unit tests** |

### Current Loader Tests (lines 477-935)

| Test | Location | Purity | Extent | Correct? | Action |
|------|----------|--------|--------|----------|---------|
| TEST-001: New schema | `#[cfg(test)]` | Impure (redb) | Large | ⚠️ Partial | Move to InMemory |
| TEST-002: File change | `#[cfg(test)]` | Impure (redb+sleep) | Large | ❌ No | Move to InMemory |
| TEST-003: Bank change | `#[cfg(test)]` | Impure (redb+sleep) | Large | ❌ No | Move to InMemory |
| TEST-004: Hash unchanged | `#[cfg(test)]` | Impure (redb+sleep) | Large | ❌ No | Move to InMemory |
| TEST-005: Mixed scenario | `#[cfg(test)]` | Impure (redb+sleep) | Large | ❌ No | Move to InMemory |
| **MISSING: Cached expansion** | - | - | - | ❌ No | **ADD unit tests** |

### Recommended Test Structure

#### Unit Tests (`src/schema/ingestor.rs`)

```rust
#[cfg(test)]
mod tests {
    use super::*;

    mod helpers {
        // Shared utilities
        fn in_memory_repo() -> InMemoryRepository { ... }
        fn write_to_memory(...) { ... }
    }

    mod property_bank {
        // Pure tests using InMemoryRepository
        #[test] fn parses_json() { ... }
        #[test] fn parses_yaml() { ... }
        #[test] fn detects_new_bank() { ... }
        #[test] fn detects_fresh_bank() { ... }
        #[test] fn detects_stale_bank() { ... }
    }

    mod schemas {
        #[test] fn parses_schema_files() { ... }
        #[test] fn detects_new_schema() { ... }
        #[test] fn detects_fresh_schema() { ... }
        #[test] fn detects_stale_schema() { ... }
    }

    mod ingest_all {  // NEW!
        #[test] fn full_pipeline_new_files() { ... }
        #[test] fn empty_directory() { ... }
        #[test] fn mixed_fresh_stale_new() { ... }
        #[test] fn bank_only_no_schemas() { ... }
    }

    mod result_types {
        // Existing tests - keep as-is
    }
}
```

#### Unit Tests (`src/schema/loader.rs`)

```rust
#[cfg(test)]
mod tests {
    use super::*;

    mod helpers {
        fn in_memory_repo() -> InMemoryRepository { ... }
        fn setup_bank(...) { ... }
        fn setup_schema(...) { ... }
    }

    mod pipeline {
        #[test] fn new_schema_full_resolution() { ... }
        #[test] fn stale_schema_full_resolution() { ... }
        #[test] fn fresh_schema_fresh_bank_no_work() { ... }
        #[test] fn all_fresh_returns_empty() { ... }
    }

    mod cached_expansion {  // NEW! (Phase 5.2)
        #[test] fn fresh_schema_stale_bank_with_cache_skips_refexpander() { ... }
        #[test] fn fresh_schema_stale_bank_no_cache_runs_refexpander() { ... }
        #[test] fn cached_properties_stored_after_expansion() { ... }
    }

    mod incremental_resolution {  // NEW!
        #[test] fn stale_bank_affects_all_schemas() { ... }
        #[test] fn changed_properties_tracked() { ... }
    }
}
```

#### Integration Tests (`tests/schema_loader_e2e.rs`) - NEW FILE

```rust
//! End-to-end integration tests using real filesystem and redb.
//!
//! These tests are IMPURE by design - they verify that the system
//! works with real IO, real database persistence, and real filesystem
//! staleness detection.

mod common;

use common::*;

// Test real filesystem staleness detection
#[test]
fn real_filesystem_timestamp_detection() -> TestResult {
    let vault_dir = TempDir::new()?;
    let test_db = TestDb::new()?;

    // First load
    write_file(vault_dir.path(), "schemas/task.json", "...");
    let repo = setup_repository(test_db.db());
    let loader = Loader::new(repo, FsReader::new(vault_dir.path()), &config);
    let initial = loader.load()?;

    // Modify file (REAL filesystem timing)
    std::thread::sleep(Duration::from_millis(10));
    write_file(vault_dir.path(), "schemas/task.json", "... modified ...");

    // Reopen database (REAL persistence round-trip)
    let db = test_db.reopen()?;
    let repo = setup_repository(&db);
    let loader = Loader::new(repo, FsReader::new(vault_dir.path()), &config);
    let updated = loader.load()?;

    assert_ne!(initial.len(), updated.len(), "Should detect file change");
    Ok(())
}

// Test real database persistence and round-tripping
#[test]
fn real_database_persistence() -> TestResult {
    // Similar pattern - test that schemas persist correctly
    // through real redb read/write cycles
}

// Test content hash fallback when timestamps unreliable
#[test]
fn content_hash_fallback() -> TestResult {
    // Modify file with same timestamp but different content
    // (simulates touch command or filesystems with low precision)
}
```

---

## Behavioral Coverage Matrix (Revised)

### Ingestor Behaviors - PURE Unit Tests

| Behavior | Test Type | Purity | Should Be | Gap |
|----------|-----------|--------|-----------|-----|
| **Property Bank Loading** |
| Parse JSON format | Unit | Pure (in-memory) | ✅ | Rewrite with InMemory |
| Parse YAML format | Unit | Pure (in-memory) | ✅ | Rewrite with InMemory |
| Parse TOML format | Unit | Pure (in-memory) | ✅ | Rewrite with InMemory |
| Invalid format error | Unit | Pure | ✅ | Keep |
| **Property Bank Staleness** |
| New bank (no cache) | Unit | Pure (explicit state) | ✅ | Rewrite with InMemory |
| Fresh bank (timestamp match) | Unit | Pure (explicit state) | ✅ | Rewrite with InMemory |
| Stale bank (hash changed) | Unit | Pure (explicit state) | ✅ | Rewrite with InMemory |
| Incremental update tracks changes | Unit | Pure | ❌ | **ADD** |
| **Schema File Scanning** |
| JSON/YAML/TOML schemas | Unit | Pure (in-memory FS) | ❌ | **ADD** |
| Empty directory | Unit | Pure | ❌ | **ADD** |
| **ingest_all() API** |
| Full pipeline | Unit | Pure | ❌ | **CRITICAL - ADD** |
| Empty results | Unit | Pure | ❌ | **ADD** |
| Mixed staleness | Unit | Pure | ❌ | **ADD** |

### Ingestor Behaviors - IMPURE Integration Tests

| Behavior | Test Type | Purity | Should Be | Gap |
|----------|-----------|--------|-----------|-----|
| Real filesystem staleness | Integration | Impure (real FS) | ✅ | Move from unit tests |
| Real database persistence | Integration | Impure (real redb) | ✅ | Move from unit tests |
| Content hash fallback | Integration | Impure (real FS) | ❌ | **ADD** |

### Loader Behaviors - PURE Unit Tests

| Behavior | Test Type | Purity | Should Be | Gap |
|----------|-----------|--------|-----------|-----|
| **Pipeline Orchestration** |
| New schema full pipeline | Unit | Pure | ⚠️ | Rewrite with InMemory |
| Stale schema full pipeline | Unit | Pure | ⚠️ | Rewrite with InMemory |
| Fresh schema + fresh bank = no work | Unit | Pure | ❌ | **ADD** |
| **Phase 5.2 Cached Expansion** |
| Skip RefExpander with cache | Unit | Pure | ❌ | **CRITICAL - ADD** |
| Fallback to RefExpander | Unit | Pure | ❌ | **CRITICAL - ADD** |
| Store expanded properties | Unit | Pure | ❌ | **ADD** |
| **Incremental Resolution** |
| Stale bank affects schemas | Unit | Pure | ❌ | **ADD** |
| Changed properties tracked | Unit | Pure | ❌ | **ADD** |
| **Inheritance** |
| Simple parent-child | Unit | Pure | ❌ | **ADD** |
| Deep chains | Unit | Pure | ❌ | **ADD** |
| Missing parent error | Unit | Pure | ❌ | **ADD** |

### Loader Behaviors - IMPURE Integration Tests

| Behavior | Test Type | Purity | Should Be | Gap |
|----------|-----------|--------|-----------|-----|
| Multi-load caching verification | Integration | Impure | ❌ | **ADD** |
| Real FS + redb round-trip | Integration | Impure | ⚠️ | Move from unit tests |
| Performance benchmarks | Integration | Impure | ❌ | Phase 6.3 |

---

## Corrected Recommendations

### Phase 1: Infrastructure (2-3 hours) - REVISED

**Goal**: Implement `InMemoryRepository` for test purity

**Tasks**:

1. **Implement `InMemoryRepository`** (NOT "FakeRepository")
   - Location: `lithos-core/src/schema/storage/memory.rs` (new file)
   - Purpose: Achieve test purity (no filesystem IO), not isolation
   - Features:
     - Full `Repository` trait implementation
     - Explicit staleness control methods
     - Thread-safe (Arc + RwLock) for potential parallel tests
   - Example API:
     ```rust
     impl InMemoryRepository {
         pub fn new() -> Self { ... }

         // Explicit staleness control (not filesystem-based)
         pub fn set_bank_staleness(&self, state: StalenessState) { ... }
         pub fn set_schema_staleness(&self, id: SchemaId, state: StalenessState) { ... }

         // Normal Repository methods
         fn find_schema_by_id(&self, id: SchemaId) -> Result<Option<Schema>> { ... }
         // ... etc
     }
     ```

2. **Extract test helpers** to submodules
   - `mod helpers` in both `ingestor.rs` and `loader.rs`
   - Shared utilities: `in_memory_repo()`, `setup_bank()`, `setup_schema()`
   - Data-driven `check()` functions

3. **Update existing test patterns**
   - Convert 1-2 existing tests to use `InMemoryRepository`
   - Verify pattern works before mass migration

**Success Criteria**:
- `InMemoryRepository` passes basic repository operations
- Can create bank + schemas without filesystem
- Explicit staleness control works (no sleep() needed)
- Pattern proven with 2 converted tests

### Phase 2: Critical Coverage (3-4 hours) - REVISED

**Goal**: Test primary APIs with pure unit tests

**Tasks**:

1. **Add `ingest_all()` unit tests** (CRITICAL)
   - `test_ingest_all_new_files` - empty repo → new files
   - `test_ingest_all_empty_directory` - edge case
   - `test_ingest_all_mixed_staleness` - complex scenario
   - `test_ingest_all_fresh_returns_cached` - optimization path
   - All tests: Pure (InMemoryRepository), Large extent (full pipeline)

2. **Add Phase 5.2 cached expansion tests** (CRITICAL)
   - `test_cached_expansion_skips_refexpander` - optimization verified
   - `test_cached_expansion_fallback` - fallback path
   - `test_store_expanded_properties` - cache storage
   - `test_resolve_with_cached_expansion` - cache usage
   - All tests: Pure, Large extent

3. **Convert ignored loader tests**
   - Change from `RedbRepository` + `TestDb` to `InMemoryRepository`
   - Remove all `std::thread::sleep` calls
   - Replace filesystem timing with explicit staleness control
   - Remove `#[ignore]` attributes
   - Verify all 5 tests pass

**Success Criteria**:
- `ingest_all()` has 4+ pure unit tests
- Phase 5.2 has 4+ pure unit tests
- All loader tests passing (0 ignored)
- No sleep() calls in unit tests
- All unit tests run in <100ms total

### Phase 3: Comprehensive Unit Coverage (3-4 hours)

**Goal**: Fill remaining gaps with pure unit tests

**Tasks**:

1. **Add missing behavior tests** (following matklad's data-driven pattern)
   - Incremental property bank updates
   - Schema file extension handling (JSON, YAML, TOML)
   - Inheritance chains (simple, deep, circular error)
   - Error propagation paths

2. **Add edge case tests**
   - Empty directory/no files
   - Bank only, no schemas
   - Schemas only, no bank
   - Invalid data/parse errors

3. **Organize tests into submodules**
   - Group related tests
   - Clear naming: `mod property_bank { mod parsing { ... } }`
   - Each submodule focuses on one behavior area

**Success Criteria**:
- All public methods have unit tests
- All error paths tested
- Clear test organization
- Unit test suite runs in <200ms

### Phase 4: Integration Tests (2-3 hours)

**Goal**: Write impure integration tests for real-world verification

**Tasks**:

1. **Create `tests/schema_loader_e2e.rs`**
   - Real filesystem staleness detection
   - Real redb persistence round-tripping
   - Content hash fallback (touch file without content change)
   - Multi-load caching verification

2. **Create `tests/schema_stress.rs`** (optional, can defer)
   - Large files (>1MB schemas)
   - Many files (>100 schemas)
   - Deep inheritance (>10 levels)

3. **Migrate flaky tests**
   - Move filesystem timing tests from unit → integration
   - Accept that integration tests may be slower
   - Document expected runtime

**Success Criteria**:
- 5-10 integration tests
- All tests pass reliably
- Real redb round-tripping verified
- Integration suite runs in <30 seconds

---

## Key Principles (From Research)

### 1. The Neural Network Test

"Can you replace your implementation with a trained model and keep the tests?"

**Good**: Testing that `load()` returns schemas with correct properties
**Bad**: Testing that `RefExpander` was called with specific arguments

### 2. Purity Over Extent

"Optimize ruthlessly for purity, let extent be natural"

**Good**: Testing full Loader pipeline with InMemoryRepository (pure, large extent)
**Bad**: Mocking internal layers to keep test "small" (pure but artificially limited)

### 3. Data-Driven Tests

"Write one `check()` function, many test cases"

**Good**:
```rust
#[track_caller]
fn check_staleness(bank_state: StalenessState, schema_state: StalenessState, expected: ExpectedBehavior) {
    let repo = InMemoryRepository::new();
    repo.set_bank_staleness(bank_state);
    // ... test logic ...
}

#[test] fn new_bank_new_schema() { check_staleness(New, New, FullPipeline); }
#[test] fn fresh_bank_fresh_schema() { check_staleness(Fresh, Fresh, NoWork); }
#[test] fn stale_bank_fresh_schema() { check_staleness(Stale, Fresh, Incremental); }
```

**Bad**: Every test duplicates setup and assertion logic

### 4. Test Organization

From The Rust Book: Unit tests go in `#[cfg(test)] mod tests`, integration tests in `tests/` directory

From matklad: Use submodules liberally to organize related tests

---

## Anti-Patterns to Avoid

### ❌ Don't Mock Your Own Code

```rust
// BAD: Mocking Ingestor to test Loader
let mut mock_ingestor = MockIngestor::new();
mock_ingestor.expect_ingest_all().returning(|| Ok(fake_results));
```

**Why bad**: Reduces test fidelity, makes refactoring harder

**Do instead**: Use InMemoryRepository to achieve purity while testing full pipeline

### ❌ Don't Use Conditional Compilation for Test Organization

```rust
// BAD: Using feature flags to separate test types
#[cfg(feature = "slow-tests")]
#[test]
fn slow_integration_test() { ... }
```

**Why bad**: Makes running tests harder, complicates build

**Do instead**: Check environment variable or use `#[ignore]` + explicit test name filter

### ❌ Don't Conflate Unit Tests with Small Tests

```rust
// BAD THINKING: "This tests too much code, must be integration test"
// GOOD THINKING: "This is pure (no IO), belongs in unit tests regardless of extent"
```

**Why bad**: Leads to artificial mocking and brittle tests

**Do instead**: Classify by purity, not extent

---

## Migration Strategy

### Step 1: Prove the Pattern (1 day)

1. Implement `InMemoryRepository`
2. Convert 2 simple tests (one ingestor, one loader)
3. Verify:
   - Tests are faster
   - No filesystem dependencies
   - Full pipeline still works
4. Get team feedback on pattern

### Step 2: Critical Paths (1 day)

1. Add `ingest_all()` tests
2. Add Phase 5.2 cached expansion tests
3. Convert ignored loader tests
4. Verify: All unit tests passing, <200ms total

### Step 3: Comprehensive Unit Coverage (1-2 days)

1. Fill remaining gaps
2. Organize into submodules
3. Add edge cases
4. Verify: Good coverage, well organized

### Step 4: Integration Tests (1 day)

1. Create `tests/schema_loader_e2e.rs`
2. Write real-world scenarios
3. Accept slower runtime
4. Document integration test purpose

**Total: 4-5 days of focused work**

---

## Conclusion

### What We Learned from Research

1. **Unit vs Integration is about location and purity, not size**
   - Unit: `#[cfg(test)]`, pure (no IO)
   - Integration: `tests/`, impure (real IO)

2. **InMemoryRepository is NOT a mock, it's a purity tool**
   - Purpose: Eliminate filesystem IO, not isolate code
   - Extent: Still test full pipeline
   - Speed: Moves from impure ladder (slow) to pure (fast)

3. **Test features, not code**
   - Neural network test: Tests should survive implementation replacement
   - Data-driven: `check()` function + many cases
   - Large extent is OK if pure

4. **Current tests violate best practices**
   - Using deprecated APIs
   - Impure unit tests (filesystem + sleep)
   - Missing primary API (`ingest_all()`)
   - 80% of loader tests disabled

### Next Steps

**Recommended**: Phase 1 (prove pattern) → Phase 2 (critical coverage) → Phase 3 (comprehensive) → Phase 4 (integration)

**Alternative (if time-constrained)**: Phase 1 + Phase 2 only, defer comprehensive coverage

**Do NOT**: Skip Phase 1 - without InMemoryRepository, we cannot fix the purity issues

---

## Appendix: Code Examples

### A1. InMemoryRepository Implementation

```rust
// lithos-core/src/schema/storage/memory.rs

use std::collections::HashMap;
use std::sync::{Arc, RwLock};

/// In-memory implementation of Repository for pure unit tests.
///
/// This implementation achieves test purity by eliminating filesystem IO,
/// NOT by isolating code under test. Tests using InMemoryRepository can
/// still exercise the full Loader pipeline.
///
/// # Purpose
///
/// - **Purity**: No filesystem, no process IO, just memory operations
/// - **Speed**: Orders of magnitude faster than redb (pure computation)
/// - **Determinism**: No filesystem timing dependencies, explicit state control
/// - **Extent**: Does NOT limit what code is tested (full pipeline OK)
///
/// # When to Use
///
/// - Unit tests (`#[cfg(test)]` modules) should use InMemoryRepository
/// - Integration tests (`tests/` directory) should use RedbRepository
///
/// # Example
///
/// ```
/// # use lithos_core::schema::storage::InMemoryRepository;
/// let repo = InMemoryRepository::new();
/// repo.set_bank_staleness(StalenessState::Fresh);
/// // Now tests can verify behavior without filesystem timing
/// ```
pub struct InMemoryRepository {
    schemas: Arc<RwLock<HashMap<SchemaId, Schema>>>,
    schema_views: Arc<RwLock<HashMap<SchemaId, RawSchemaView>>>,
    property_bank: Arc<RwLock<Option<PropertyBank>>>,
    bank_view: Arc<RwLock<Option<RawPropertyBankView>>>,
}

impl InMemoryRepository {
    pub fn new() -> Self {
        Self {
            schemas: Arc::new(RwLock::new(HashMap::new())),
            schema_views: Arc::new(RwLock::new(HashMap::new())),
            property_bank: Arc::new(RwLock::new(None)),
            bank_view: Arc::new(RwLock::new(None)),
        }
    }

    /// Set property bank staleness state explicitly.
    ///
    /// Replaces filesystem-based staleness detection with explicit control.
    /// This eliminates the need for `std::thread::sleep` in tests.
    pub fn set_bank_staleness(&self, state: StalenessState) {
        let mut view = self.bank_view.write().unwrap();
        *view = Some(/* construct view with specified state */);
    }

    /// Set schema staleness state explicitly.
    pub fn set_schema_staleness(&self, id: SchemaId, state: StalenessState) {
        let mut views = self.schema_views.write().unwrap();
        views.insert(id, /* construct view with specified state */);
    }
}

impl Repository for InMemoryRepository {
    type Error = InMemoryError;

    fn find_schema_by_id(&self, id: SchemaId) -> Result<Option<Schema>, Self::Error> {
        let schemas = self.schemas.read().unwrap();
        Ok(schemas.get(&id).cloned())
    }

    // ... implement all other Repository methods using HashMap operations
}

/// Staleness state for explicit test control.
pub enum StalenessState {
    New,
    Fresh { timestamp: SystemTime, hash: u64 },
    Stale { old_hash: u64, new_hash: u64 },
}
```

### A2. Data-Driven Test Example

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[track_caller]
    fn check_staleness_detection(
        bank_state: StalenessState,
        schema_state: StalenessState,
        expected_outcome: ExpectedOutcome,
    ) {
        // GIVEN: Repository with explicit staleness state
        let repo = InMemoryRepository::new();
        repo.set_bank_staleness(bank_state);
        repo.set_schema_staleness(SchemaId::new(), schema_state);

        // WHEN: Running ingestor
        let ingestor = Ingestor::new(FsReader::new_in_memory(), &config, repo);
        let results = ingestor.ingest_all().unwrap();

        // THEN: Outcome matches expectation
        match expected_outcome {
            ExpectedOutcome::AllFresh => {
                assert!(results.property_bank.is_fresh());
                assert!(results.schemas.values().all(|s| s.is_fresh()));
            }
            ExpectedOutcome::BankStale => {
                assert!(results.property_bank.is_stale());
            }
            ExpectedOutcome::SchemaStale => {
                assert!(results.schemas.values().any(|s| s.is_stale()));
            }
        }
    }

    #[test]
    fn new_bank_new_schema() {
        check_staleness_detection(
            StalenessState::New,
            StalenessState::New,
            ExpectedOutcome::AllNew,
        );
    }

    #[test]
    fn fresh_bank_fresh_schema() {
        check_staleness_detection(
            StalenessState::Fresh { timestamp: now(), hash: 123 },
            StalenessState::Fresh { timestamp: now(), hash: 456 },
            ExpectedOutcome::AllFresh,
        );
    }

    #[test]
    fn stale_bank_affects_schemas() {
        check_staleness_detection(
            StalenessState::Stale { old_hash: 123, new_hash: 789 },
            StalenessState::Fresh { timestamp: now(), hash: 456 },
            ExpectedOutcome::IncrementalResolution,
        );
    }
}
```

### A3. Integration Test Example

```rust
// lithos-core/tests/schema_loader_e2e.rs

//! End-to-end integration tests with REAL filesystem and database.
//!
//! These tests are INTENTIONALLY IMPURE - they verify that the system
//! works correctly with real IO, real persistence, and real timing.

mod common;

use common::*;
use std::thread;
use std::time::Duration;

/// Test that real filesystem timestamp detection works.
///
/// This test is IMPURE by design - it uses real filesystem writes
/// and sleep() to verify timestamp-based staleness detection.
#[test]
fn real_filesystem_timestamp_staleness() -> TestResult {
    // GIVEN: Real filesystem and database
    let vault_dir = TempDir::new()?;
    let test_db = TestDb::new()?;

    write_file(vault_dir.path(), "schemas/task.json", r#"{"$version": "1.0", "properties": {}}"#)?;

    let config = test_config(vault_dir.path())?;
    let repo = setup_repository(test_db.db());
    let source = FsReader::new(vault_dir.path());
    let loader = Loader::new(repo, source, &config);

    // WHEN: First load
    let initial = loader.load()?;
    assert_eq!(initial.len(), 1);

    // AND: File modified with REAL filesystem timing
    thread::sleep(Duration::from_millis(10));
    write_file(vault_dir.path(), "schemas/task.json", r#"{"$version": "1.0", "properties": {"new": {"type": "string"}}}"#)?;

    // AND: Database reopened (REAL persistence round-trip)
    let db = test_db.reopen()?;
    let repo = setup_repository(&db);
    let loader = Loader::new(repo, FsReader::new(vault_dir.path()), &config);

    // THEN: File change detected via timestamp
    let updated = loader.load()?;
    assert_eq!(updated.len(), 1);
    assert_eq!(updated[0].properties().len(), 1, "Should have new property");

    Ok(())
}
```
