# Task Plan: PropertyBankProcessor Post-Implementation Review & Refactor

## Goal
Complete a review-driven refactor of the property-bank deepening implementation, addressing rust-best-practices gaps (clone elimination, dead code removal, metadata status split, test hygiene) without changing domain behavior.

## Current Phase
Phase 1: Analysis & Planning (DRAFT — awaiting user approval)

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
- [ ] Confirm GREEN baseline: `cargo test -p lithos-core schema::property_bank_processor` passes
- [ ] Add `constructs_bank_with_title_property_when_new` — single assertion: bank has title property (uses `make_fixture()`)
- [ ] Refactor `persists_view_with_rooted_path_key_when_constructing_new_bank` — single assertion: view persists (uses `make_fixture()`)
- [ ] Remove the original dual-assertion test from `constructor` module
- [ ] Run tests: both new tests pass
- **Status:** pending

### Phase 3: Remove FileMetadata from Status Structs (Q1)
<!-- rust-best-practices §1.1: eliminate redundant clone. self.file.metadata() is always identical to self.status.metadata because both originate from the same discovery entry and the pipeline is synchronous. -->
- [ ] Confirm GREEN baseline: run tests before touching production code
- [ ] Remove `FileMetadata` field from `Missing` — `Missing::new()` takes no args
- [ ] Remove `FileMetadata` field from `Present` — `Present::new()` takes only `RawPropertyBankView`
- [ ] Remove `FileMetadata` field from `Suspect`, `Stale`, `StaleTimestamps`, `StaleContent`
- [ ] All metadata comparisons use `self.file.metadata()` instead of `self.status.metadata`
- [ ] All `sync_metadata` variants use `self.file.metadata()` (note: still passes by value due to `update_metadata` API shape)
- [ ] Builder boundary: remove `file.metadata().clone()`; pass no metadata to status ctors
- [ ] Run tests after each sub-step
- **Status:** pending

### Phase 4: Eliminate Clones in Construction (Q2e)
<!-- rust-best-practices §1.1: avoid cloning in hot paths. Verified signatures: try_from_raw_with_hashes(&RawPropertyBank) borrows; into_changed_name_set(self) consumes. -->
- [ ] Confirm GREEN baseline
- [ ] `create()`: Reorder — create view from `&raw` first, THEN move `raw` into `PropertyBank::try_from`
- [ ] `update()`: Destructure `self` before persist; extract `delta` by value; reconstruct processor for transition
- [ ] Run tests after each change
- **Status:** pending

### Phase 5: Remove Remaining Dead Code (Q2a)
<!-- rust-best-practices §1.6, §8.6: dead code without linked issues should be removed. -->
- [ ] `Missing::metadata()` — naturally removed by Phase 3 (no metadata field left)
- [ ] `Present::metadata()` — naturally removed by Phase 3
- [ ] `Present::view()` — annotate with `#[expect(dead_code, reason="TODO(#XXX): expose for caller diagnostics")]` using tracking issue
- [ ] `DiscoveryResult::is_cold_start()` — remove function entirely
- [ ] Run tests
- **Status:** pending

### Phase 6: Verification
- [ ] `cargo test -p lithos-core` — full pass
- [ ] `cargo clippy --all-targets --all-features -- -D warnings` — clean
- [ ] `cargo fmt` — formatted
- [ ] Verify all builder integration tests unchanged
- **Status:** pending

## Key Questions
1. Should `DeadCode` accessors be deleted or kept-dead with linked issue references? (User preference)
2. Should `Present::view()` also be kept for potential future builder use, or removed as dead code?
3. What tracking issue/ADR number to use for deferred-accessor references?

## Decisions Made
| Decision | Rationale |
|----------|-----------|
| Remove FileMetadata from status | Redundant with `self.file.metadata()` — eliminates metadata clone at builder + sync_metadata |
| Reorder operations in `create()` | `try_from_raw_with_hashes` takes `&RawPropertyBank` so we can borrow before move |
| Destructure `self` in `update()` | Enables moving `delta` out before persist call, eliminating clone |
| Keep `FsFile` clone at builder boundary | `from_fs_file()` needs ownership — unavoidable with current API shape; could add `From<FsFile>` / `&FsFile` ctor as future opt |

## Errors Encountered
_N/A — planning phase only_

## Notes
- Do NOT implement anything until user approves this plan.
- All refactors must maintain existing behavior — rely on existing integration tests for safety net.
- Phase 6 verification is the gate: everything must pass before merging.
