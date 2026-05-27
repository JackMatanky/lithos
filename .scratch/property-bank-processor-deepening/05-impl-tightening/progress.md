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
- **Status:** pending
- TDD approach: Fix test surface FIRST (split test into one-assertion-per-test), then refactor production code against improved tests.
- Pre-condition: `cargo test -p lithos-core` passes GREEN.

### Phase 3: Remove FileMetadata from Status Structs
- **Status:** pending

### Phase 4: Eliminate Clones in Construction
- **Status:** pending

### Phase 5: Remove Remaining Dead Code
- **Status:** pending

### Phase 6: Verification
- **Status:** pending

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

## 5-Question Reboot Check
| Question | Answer |
|----------|--------|
| Where am I? | Phase 1: Analysis & Planning |
| Where am I going? | Phases 2-6 (awaiting user approval) |
| What's the goal? | Complete review-driven refactor of property-bank deepening implementation |
| What have I learned? | See findings.md |
| What have I done? | See above |
