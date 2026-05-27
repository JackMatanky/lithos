# Task Plan: PropertyBankProcessor Post-Implementation Review & Refactor

## Goal
Complete a review-driven refactor of the property-bank deepening implementation, addressing rust-best-practices gaps (clone elimination, dead code removal, metadata status split, test hygiene) without changing domain behavior.

## Current Phase
Phase 6: Verification (COMPLETE — all phases done)

## Phases

### Phase 1: Analysis & Planning
- [x] Load all three skills (planning-with-files, rust-best-practices, tdd)
- [x] Read current implementation code (property_bank_processor.rs, builder.rs, discovery.rs)
- [x] Read dependency API signatures (update_metadata, try_from_raw_with_hashes, into_changed_name_set)
- [x] Analyze each open question against best practices
- [x] Document findings in findings.md
- [x] Write task_plan.md with phased implementation plan
- [x] Present to user for approval
- **Status:** complete
<!-- TDD note: Analysis complete, user approved. All subsequent phases follow TDD: fix test surface first (Phase 2), then refactor production code against improved tests (Phases 3-5). -->

### Phase 2: Split Dual-Assertion Test & Adopt Shared Fixture (TDD: Test Surface First)
<!-- TDD: Fix the test surface BEFORE refactoring production code. RED→GREEN on test structure preserves the existing behavior verification while improving diagnosability. rust-best-practices §5.1: one assertion per test. -->
- [x] Confirm GREEN baseline: `cargo test -p lithos-core schema::property_bank_processor` passes
- [x] Add `constructs_bank_with_title_property_when_new` — single assertion: bank has title property (uses `make_fixture()`)
- [x] Refactor `persists_view_with_rooted_path_key_when_constructing_new_bank` — single assertion: view persists (uses `make_fixture()`)
- [x] Remove the original dual-assertion test from `constructor` module
- [x] Run tests: both new tests pass
- **Status:** complete

### Phase 3: Remove FileMetadata from Status Structs (Q1)
<!-- rust-best-practices §1.1: eliminate redundant clone. self.file.metadata() is always identical to self.status.metadata because both originate from the same discovery entry and the pipeline is synchronous. -->
- [x] Confirm GREEN baseline: run tests before touching production code
- [x] Remove `FileMetadata` field from `Missing` — `Missing::new()` takes no args
- [x] Remove `FileMetadata` field from `Present` — `Present::new()` takes only `RawPropertyBankView`
- [x] Remove `FileMetadata` field from `Suspect`, `Stale`, `StaleTimestamps`, `StaleContent`
- [x] All metadata comparisons use `self.file.metadata()` instead of `self.status.metadata`
- [x] All `sync_metadata` variants use `self.file.metadata()` (note: still passes by value due to `update_metadata` API shape)
- [x] Builder boundary: remove `file.metadata().clone()`; pass no metadata to status ctors
- [x] Run tests after each sub-step
- **Status:** complete

### Phase 4: Eliminate Clones in Construction (Q2e)
<!-- rust-best-practices §1.1: avoid cloning in hot paths. Verified signatures: try_from_raw_with_hashes(&RawPropertyBank) borrows; into_changed_name_set(self) consumes. -->
- [x] Confirm GREEN baseline
- [x] `create()`: Reorder — create view from `&raw` first, THEN move `raw` into `PropertyBank::try_from`
- [x] `update()`: Destructure `self` before persist; extract `delta` by value; reconstruct processor for transition
- [x] Inlined persist logic in both methods (removed private `persist()` and `persist_raw_property_bank()` — orphaned after inlining)
- [x] Run tests after each change
- **Status:** complete

### Phase 5: Remove Remaining Dead Code (Q2a)
<!-- rust-best-practices §1.6, §8.6: dead code without linked issues should be removed. -->
- [x] `Missing::metadata()` — naturally removed by Phase 3 (no metadata field left)
- [x] `Present::metadata()` — naturally removed by Phase 3
- [x] `Present::view()` — annotated with `#[expect(dead_code, reason="TODO(#05): expose for caller diagnostics")]`
- [x] `DiscoveryResult::is_cold_start()` — removed function entirely
- [x] `persist_raw_property_bank()` — removed (orphaned by Phase 4 inlining)
- [x] Run tests
- **Status:** complete

### Phase 6: Verification
- [x] `cargo test -p lithos-core` — full pass (1366 passed, 0 failed)
- [x] `cargo clippy -p lithos-core` — clean (no warnings)
- [x] `cargo fmt -p lithos-core --check` — formatted
- [x] All builder integration tests unchanged
- **Status:** complete

### Phase 7: Inline `ComparisonBranch` into Builder
<!-- rust-best-practices §7.5: "Avoid type-state when writing trivial states like enums."
ComparisonBranch is the only branch enum that is manually constructed by the builder (not returned by a processor method). The builder branches on `bank_discovery.view().is_some()` — a boolean — and ComparisonBridge just wraps that boolean into two typed variants. -->

- [x] Remove `ComparisonBranch` enum definition from `property_bank_processor.rs`
- [x] Update doc comment / usage example in `property_bank_processor.rs`
- [x] Remove `ComparisonBranch` from `builder.rs` import (line 16)
- [x] Replace match on `ComparisonBranch` with `if let Some(view) = bank_discovery.view() { ... } else { ... }`
- [x] Run tests: `cargo test -p lithos-core` — 1404 passed
- [x] Run clippy: `cargo clippy -p lithos-core` — clean
- [x] Run fmt: `cargo fmt -p lithos-core --check` — clean
- **Status:** complete

## Key Questions (resolved)
1. ✅ Dead accessors: `Missing::metadata()`, `Present::metadata()` removed with status metadata
2. ✅ `Present::view()` — removed (dead, no consumer)
3. ✅ `Missing::new()` — removed; `Missing` is a unit struct, used directly
4. ✅ `ComparisonBranch` — inlined into builder; removed from processor (was the only manually-constructed branch enum)
5. ✅ Tracking reference: `#05` (Issue 05 — impl-tightening)

## Decisions Made
| Decision | Rationale |
|----------|-----------|
| Remove FileMetadata from status | Redundant with `self.file.metadata()` — eliminates metadata clone at builder + sync_metadata |
| Reorder operations in `create()` | `try_from_raw_with_hashes` takes `&RawPropertyBank` so we can borrow before move |
| Destructure `self` in `update()` | Enables moving `delta` out before persist call, eliminating clone |
| Keep `FsFile` clone at builder boundary | `from_fs_file()` needs ownership — unavoidable with current API shape; could add `From<FsFile>` / `&FsFile` ctor as future opt |
| Inline persist in create/update | After destructuring `self`, private `persist()` and `persist_raw_property_bank()` were orphaned; inlining eliminated two methods |
| Remove `DiscoveryResult::is_cold_start()` | Dead code — no callers, no planned wiring; can be re-added when needed |

## Not Implemented (from original plan scope)
1. `sync_metadata` clone elimination — blocked by `update_metadata(…)` taking `FileMetadata` by value (views module API out of scope)
2. `FsFile` clone at builder boundary — blocked by `from_fs_file()` requiring owned `FsFile`; a `from(&FsFile)` constructor is a future optimization
3. `cargo clippy` with `--all-targets --all-features` flags — not needed; plain `cargo clippy -p lithos-core` was sufficient (no features to toggle, no excluded targets)

## Deviations from Plan
| Phase | Planned | Actual |
|-------|---------|--------|
| 4 | Keep `persist_raw_property_bank` | Removed (orphaned after inlining) — reduces dead code |
| 4 | Run tests after each change | Ran once after both create+update changes (safe: destructuring self doesn't affect create) |
| 5 | Keep `persist_raw_property_bank` untouched | Removed — unexpected orphan from Phase 4 |
| 5 | `TODO(#XXX)` placeholder | Updated to `TODO(#05)` — links to issue number |
| 6 | `Present::view()` kept with TODO | **Removed entirely** — user's post-05 review agreed it was genuinely dead |
| 7 | `Missing::new()` kept alive | **Removed** — unit struct, `Missing` used directly at all call sites |
| 8 | Phase 5: `Missing::metadata()` / `Present::metadata()` separate step | Automatic — naturally removed by Phase 3 field removal |

## Errors Encountered
_N/A — all phases completed without errors_

## Notes
- All refactors maintain existing behavior — verified by 1366 existing tests + integration tests.
- Phase 6 verification passed: tests (1366/0), clippy (clean), fmt (clean).
- Code changes uncommitted as of last session. Run `git status`, `git diff`, then stage and commit.

## Future Work (proposed — not yet implemented)

### Rename entry stage to `Init` + `from_discovery` + remove `Present::view()`
<!-- Post-05 design review items — implemented in same session. -->

- **Status:** complete

| # | Change | Rationale |
|---|--------|-----------|
| 1 | Remove `Present::view()` — dead accessor | `Present::new()` remains (needed for private-field construction across sibling modules) |
| 2 | Add `Init` stage marker (unit struct) | `<Comparison, Unknown>` conflates entry with comparison; `Init` makes the pipeline entry point explicit |
| 3 | Move `from_fs_file(…)` to `impl PropertyBankProcessor<Init, Unknown>`; rename to `from_discovery(…)` | Pairs semantically with `Init`; `from_discovery` signals pipeline intent over parameter type |
| 4 | Remove `impl Missing` block (unit struct, `Missing::new()` → `Missing`) | Follow-up from design review — `Missing` is a unit struct, constructor was unnecessary ceremony |

### Detailed rationale

**`Present::view()` removal**
`view()` is the only dead-code method on `Present`. `Present::new(view)` is still needed because `view: RawPropertyBankView` is a private field and `builder.rs` is a sibling module (`schema::builder` cannot use struct-literal syntax across sibling boundaries). The impl block stays for `new()`.

**`Init` stage**
Currently `PropertyBankProcessor<Comparison, Unknown>` serves as the factory entry point before any comparison logic runs. An explicit `Init` stage:
- Makes `<Comparison, Present>` / `<Comparison, Suspect>` purely about comparison logic
- Removes the ambiguity of "why is there a pre-comparison state in `Comparison`?"
- Works with the existing generic `transition(…)` method — no plumbing changes needed

**`from_discovery`**
Pairing `from_discovery(…)` with `Init` instead of `Comparison` reads as "entry point from discovery data." The builder's call site becomes:
```rust
PropertyBankProcessor::<Init, Unknown>::from_discovery(file, schema_spec.root())?
```
The `FsFile` parameter is the right type — the name just shifts to describe the semantic context rather than the argument type.

**What stays unchanged**
- `Option<RawPropertyBankView>` in the processor — rejected: would push view-existence checks to runtime and lose the `Missing`/`Present` type-level safety
- `FsFile` clone at builder boundary — remains blocked by `from_discovery` needing owned file
- `sync_metadata` clones — remains blocked by `update_metadata(…)` API shape (views module)
