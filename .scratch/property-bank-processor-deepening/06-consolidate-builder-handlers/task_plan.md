# Task Plan: Consolidate Builder Handler Chain via `PropertyBankProcessor::run()`

## Goal

Add `run()` to `PropertyBankProcessor<Init, Unknown>` that encapsulates the full property bank pipeline, then remove the builder's 7 handler methods, collapsing the builder import from 15 types to 1.

## Current Phase

Phase 0 — Design Review (awaiting user confirmation)

## Phases

### Phase 0: Design Review & Confirmation

- [ ] Present full analysis with blast radius
- [ ] Present design options considered (A, B, C)
- [ ] Confirm choice of Option B (single `run()` on Init)
- [ ] Confirm TDD implementation plan
- **Status:** in_progress

### Phase 1: RED — Write missing-path test for `run(None, ...)`

- [ ] Add test to `property_bank_processor.rs` `mod tests`: creates processor via `from_discovery`, calls `run(None, source, repo)`, asserts bank is returned with no delta
- [ ] The test must fail to compile or fail at runtime (RED: `run()` doesn't exist yet)
- **Status:** pending

### Phase 2: GREEN — Implement `run(None, ...)` on `PropertyBankProcessor<Init, Unknown>`

- [ ] Add `fn run(self, view: Option<&RawPropertyBankView>, source: &FileReader, repository: &R) -> Result<...>` on `impl PropertyBankProcessor<Init, Unknown>`
- [ ] Implement the `None` branch: `self.transition(Parsed, Missing).parse(source)?.create(repository)?.into_bank()`
- [ ] Cover only enough to pass the missing-path test
- [ ] Verify test passes
- **Status:** pending

### Phase 3: RED — Write present-path test for `run(Some(view), ...)`

- [ ] Test: pre-seed repository with a view, timestamps match → `run(Some(view), ...)` returns bank without delta
- [ ] Test: timestamps mismatch, content matches → bank returned, no delta
- [ ] Test: content mismatch, analysis empty → bank returned, no delta
- [ ] Test: content mismatch, analysis delta → bank + delta returned
- [ ] Test: content mismatch, analysis corrupt → bank returned (treated as new)
- Each test is a separate RED→GREEN mini-cycle; write one, pass one, write next
- **Status:** pending

### Phase 4: GREEN — Implement `run(Some(view), ...)` path

- [ ] Add `run_present(self, source, repo)` private helper on `PropertyBankProcessor<Comparison, Present>`
- [ ] The helper inlines the chain: `check_timestamps → check_content → parse → analyze → create/update/sync/fetch`
- [ ] Add `run_missing(self, source, repo)` private helper on `PropertyBankProcessor<Parsed, Missing>`
- [ ] Wire both into `run()` via `match view { Some(v) → run_present, None → run_missing }`
- [ ] Verify each test passes as we build the branches
- **Status:** pending

### Phase 5: REFACTOR — Replace builder handler chain

- [ ] Remove `load_property_bank` handler methods from `builder.rs`:
  - `handle_missing`
  - `handle_present`
  - `handle_content_mismatch`
  - `handle_analysis_branch`
  - `fetch_fresh`
  - `sync_and_fetch_timestamps`
  - `sync_and_fetch_content`
- [ ] Remove `PropertyBankCompletion` type alias if dead code
- [ ] Replace `load_property_bank` body with `processor.run(bank_discovery.view(), &self.source, &self.repository)?`
- [ ] Prune builder imports: 15 items → 1 (`PropertyBankProcessor`)
- [ ] Run tests (expect: still 1404 passing)
- [ ] Run clippy + fmt
- **Status:** pending

### Phase 6: Cleanup & Verification

- [ ] Remove any dead `use` imports in `builder.rs` (e.g., types no longer needed)
- [ ] Verify no test in `property_bank_processor.rs` uses struct-literal construction (should use `transition()` or `run()`)
- [ ] Full regression: `cargo nextest run -p lithos-core`
- [ ] Clippy: `cargo clippy -p lithos-core -- -D warnings`
- [ ] Format: `cargo fmt -p lithos-core --check`
- **Status:** pending

### Phase 7: Commit

- [ ] Stage changed files
- [ ] Conventional commit message per project standard
- [ ] Push (if user requests)
- **Status:** pending

## Key Questions

1. Should `run()` take `Option<&RawPropertyBankView>` (borrow) or `Option<RawPropertyBankView>` (owned)? ← **Answer:** borrow (`&`), because the processor only needs a reference to check timestamps — the clone to create `Present::new()` is internal. The builder retains ownership for potential reuse.
2. Should `run()` return `(PropertyBank, Option<HashSet<PropertyName>>)` or a dedicated struct? ← **Answer:** tuple, matching the existing `PropertyBankCompletion` type alias. A dedicated type is over-engineering for a single return from one method.
3. Should `run_present()` and `run_missing()` be private methods or inlined directly in `run()`? ← **Answer:** private methods on their respective `(Stage, Status)` impl blocks. This keeps the typestate discipline — each method only exists where its calls are valid.

## Decisions Made

| Decision | Rationale |
|----------|-----------|
| Option B: `run()` on `PropertyBankProcessor<Init, Unknown>` | Single entry point collapses builder import from 15 types to 1; branching moves inside the module where it's closer to the types that enforce post-branch invariants |
| `run()` returns `Result<(PropertyBank, Option<HashSet<PropertyName>>), SchemaLoaderError>` | Matches existing `PropertyBankCompletion` type alias; no new type needed for one return site |
| `run_present()` and `run_missing()` as private helpers | Preserves typestate discipline — each helper lives on the `(Stage, Status)` impl where its transitions are valid |
| `run()` takes `source: &FileReader` and `repository: &R` (borrows) | The processor doesn't own these resources; they're injected per-call |
| TDD tracer bullets through pipeline paths | Each path (Missing, Present→Match, Present→Mismatch→ContentMatch, etc.) is a vertical slice; one RED→GREEN cycle per path |
| Do NOT delete existing builder integration tests | `builder_load_all_orchestrates_discovery` tests the full `Builder`, not just handler methods — it survives unchanged |

## Errors Encountered

| Error | Attempt | Resolution |
|-------|---------|------------|
| — | — | — |
