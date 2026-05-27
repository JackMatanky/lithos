# Progress Log — PropertyBankProcessor Review

## Session: 2026-05-27

### Phase 1: Analysis & Planning
- **Status:** complete
- **Started:** 2026-05-27 ~03:40
- Actions taken:
  - Loaded rust-best-practices, tdd, and planning-with-files skills
  - Read full current implementation: `property_bank_processor.rs` (1137 lines), `builder.rs`, `discovery.rs`
  - Read dependency API signatures: `update_metadata` (takes `FileMetadata` by value), `try_from_raw_with_hashes` (takes `&RawPropertyBank` by ref), `into_changed_name_set` (takes `self` by value)
  - Analyzed all open questions against rust-best-practices guidelines
  - Verified `try_from_raw_with_hashes` borrow pattern enables clone elimination in `create()`
  - Identified `sync_metadata` clones as forced by API shape (cannot fix without views module change)
  - Created task_plan.md, findings.md, progress.md
  - User approved plan
  - Audited plan against TDD: reordered phases so test split (Phase 2) comes before production refactoring
  - Moved artifacts to `.scratch/property-bank-processor-deepening/05-impl-tightening/`
- Files created/modified:
  - `.scratch/property-bank-processor-deepening/05-impl-tightening/task_plan.md` (updated with TDD ordering)
  - `.scratch/property-bank-processor-deepening/05-impl-tightening/findings.md` (created)
  - `.scratch/property-bank-processor-deepening/05-impl-tightening/progress.md` (updated)

### Phase 2: Split Dual-Assertion Test & Adopt Shared Fixture
- **Status:** complete
- TDD approach: Fix test surface FIRST (split test into one-assertion-per-test), then refactor production code against improved tests.
- Pre-condition: `cargo test -p lithos-core` passes GREEN.

### Phase 3: Remove FileMetadata from Status Structs
- **Status:** complete
- Changes:
  - Removed `FileMetadata` field from `Missing`, `Present`, `Suspect`, `Stale`, `StaleTimestamps`, `StaleContent`
  - `Present::new()` now accepts only `view: RawPropertyBankView`
  - `Missing::new()` now takes no args
  - All metadata comparisons use `self.file.metadata()` from processor root
  - Builder boundary: removed `file.metadata().clone()`
  - Removed `Missing::metadata()`, `Present::metadata()` accessors
  - Fixed 3 compiler warnings (unused import, unused variable, unfulfilled expect)
- Test result: 1366 passed, 0 failed

### Phase 4: Eliminate Clones in Construction
- **Status:** complete
- Changes:
  - `create()`: Destructured `self` via `into_parts()`, created view from `&status.raw` (borrow), then moved `raw` into `PropertyBank::try_from` — eliminates `raw.clone()`
  - `update()`: Destructured `self`, extracted `(raw, delta, content_hash)` by value, applied delta via borrow, consumed `delta.into_changed_name_set()` — eliminates `delta.clone()`
  - Private `persist()` method and `persist_raw_property_bank()` removed (orphaned after inlining persist logic into create/update)
- Test result: 1366 passed, 0 failed

### Phase 5: Remove Remaining Dead Code
- **Status:** complete
- Changes:
  - `Missing::metadata()` — naturally removed (no metadata field in `Missing`)
  - `Present::metadata()` — naturally removed (no metadata field in `Present`)
  - `Present::view()` — annotated with `#[expect(dead_code, reason="TODO(#05): expose for caller diagnostics")]` (kept, linked to issue)
  - `DiscoveryResult::is_cold_start()` — removed entirely
  - `persist_raw_property_bank()` — removed (orphaned by Phase 4 inlining)
- Test result: 1366 passed, 0 failed

### Phase 6: Verification
- **Status:** complete
- Results:
  - `cargo test -p lithos-core`: 1366 passed, 0 failed
  - `cargo clippy -p lithos-core`: no warnings
  - `cargo fmt -p lithos-core --check`: formatted
- All builder integration tests unchanged

## Analysis Summary per Question

### Q1: Builder metadata clone + redundant FileMetadata in status

**Problem**: `builder.rs:144` clones `file.metadata()`, then passes to `Present::new()`. Every status struct then carries its own `FileMetadata`. But `self.file.metadata()` on `PropertyBankProcessor` already provides the same data.

**Fix**: Remove `FileMetadata` from all status structs. All metadata comparisons use `self.file.metadata()`. `Present::new()` becomes `Present::new(view: RawPropertyBankView)`. `Missing` carries no metadata.

**Impact on builder**:
```rust
// Before (builder.rs:143-172)
let file = bank_discovery.entry().clone();
let file_info = file.metadata().clone();
let processor = PropertyBankProcessor::<Comparison, Unknown>::from_fs_file(file, schema_spec.root())?;
let branch = match bank_discovery.view() {
    Some(view) => ComparisonBranch::Present(processor.transition(Comparison, Present::new(file_info, view.clone()))),
    None => ComparisonBranch::Missing(processor.transition(Parsed, Missing::new(file_info))),
};

// After
let file = bank_discovery.entry().clone();  // still needed — from_fs_file needs ownership
let processor = PropertyBankProcessor::<Comparison, Unknown>::from_fs_file(file, schema_spec.root())?;
let branch = match bank_discovery.view() {
    Some(view) => ComparisonBranch::Present(processor.transition(Comparison, Present::new(view.clone()))),
    None => ComparisonBranch::Missing(processor.transition(Parsed, Missing)),
};
```

### Q2a: Dead code resolution

| Method | Status | Action |
|--------|--------|--------|
| `Missing::metadata()` | Dead | **Remove** — `FileMetadata` removed from status entirely |
| `Present::metadata()` | Dead | **Remove** — metadata moved to processor root |
| `Present::view()` | Dead | **Keep with annotated issue** — may be useful for builder diagnostics; annotate as `#[expect(dead_code, reason="TODO(#XXX): expose for caller diagnostics")]` |
| `DiscoveryResult::is_cold_start()` | Dead | **Remove** — no callers, no planned wiring |

### Q2b: Test split

Split `persists_view_with_rooted_path_key_when_constructing_new_bank` into:
1. `constructs_bank_with_title_property_when_new` — asserts `bank.has(&"title")`
2. `persists_view_with_rooted_path_key_when_constructing_new_bank` — asserts `view.is_some()`

### Q2c: Shared fixture

Both new tests use `make_fixture()`, which was already created during Q2 implementation but not used by the first test.

**Before**: Test 1 has 48 lines of manual setup (identical to fixture).
**After**: Tests 1 and 2 call `let fixture = make_fixture();` (2 lines each).

### Q2e: Clone elimination feasibility

| Clone location | Eliminable? | Strategy |
|---------------|-------------|----------|
| `builder.rs:143` `FsFile::clone()` | No — `from_fs_file` needs ownership | Accept as necessary; could add `from(&FsFile)` ctor later |
| `builder.rs:144` `FileMetadata::clone()` | **Yes** (via Q1 fix) | Remove FileMetadata from status — no longer needed |
| `pbr.rs:650,674` `FileMetadata::clone()` | Partial | Only if `update_metadata` changes; accept if not |
| `pbr.rs:733` `RawPropertyBank::clone()` | **Yes** | Reorder: create view from `&raw`, then move `raw` to `try_from` |
| `pbr.rs:781` `PropertyDelta::clone()` | **Yes** | Destructure `self` before persist, extract `delta` by value |

## Post-05 Design Review (initial)
- **Date:** 2026-05-27
- **Status:** superseded (implemented below)

## Post-05 Implementation
- **Date:** 2026-05-27 (later same session)
- **Changes implemented:**
  1. **Removed `Present::view()`** — dead accessor removed; `Present::new()` kept for private-field construction
  2. **Added `Init` stage** — new unit struct before `Comparison`; `<Init, Unknown>` is now the explicit pipeline entry point
  3. **Renamed `from_fs_file` → `from_discovery`** on `<Init, Unknown>`; updated all call sites (builder + 2 tests + doc comment)
  4. **Removed `impl Missing` block** — `Missing` is a unit struct; replaced `Missing::new()` with `Missing` at all 3 call sites
- Test result: 1366 passed, 0 failed, clippy clean, fmt clean

## Deepening Analysis (GitNexus + rust-best-practices)
- **Date:** 2026-05-27
- **Findings:**
  1. `ComparisonBranch` is the only manually-constructed branch enum — confirmed anomalous
  2. Builder orchestration tree maps 1:1 to processor branch enums (visible re-implementation)
  3. Call graph: `load_all` → `load_property_bank` → 4 direct handler methods → 3 inline wrappers
  4. Verified: no test outside `schema` module depends on processor internals
- **Next steps planned in task_plan.md:**
  - Phase 7: Inline `ComparisonBranch` into builder
  - Candidate A: `Processor::run()` deepening opportunity (deferred to separate scratch if schema processor also involved)

## 5-Question Reboot Check
| Question | Answer |
|----------|--------|
| Where am I? | Phase 6: Verification (COMPLETE — all phases done) |
| Where am I going? | N/A — session complete. Next step: stage & commit. |
| What's the goal? | Complete review-driven refactor of property-bank deepening implementation |
| What have I learned? | See findings.md |
| What have I done? | Phases 1-6 complete. Test split (Phase 2), metadata removal from statuses (Phase 3), clone elimination (Phase 4), dead code removal (Phase 5), verification (Phase 6). All 1366 tests pass, clippy clean, fmt clean. |
