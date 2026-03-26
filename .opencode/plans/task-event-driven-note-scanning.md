# Task Plan: Event-Driven Note Scanning + Raw Cleanup

**Status**: Planning
**Created**: 2026-03-26
**Priority**: High

---

## Objective

Refactor note ingestion so scanning is driven by pulldown-cmark event ranges
instead of a global scan + filter pass. Remove `master_artifacts` from raw
storage, and add `is_checked: Option<bool>` to `RawListItem` while preserving
the exact task marker character whenever a checkbox is detected.

This plan is designed to be **self-contained** and **restartable** after
compaction or in a new conversation.

---

## Non-Negotiable Invariants

1. `is_checked` is authoritative for checkbox presence.
2. If `is_checked` is `Some(_)`, then a task marker **should** exist.
   - Do **not** error; treat missing marker as a scanning failure.
   - Emit a warning/trace and continue (see Logging section).
3. Task marker scanning is only relevant when pulldown reports a checkbox.
   - Ignore markers if `is_checked` is `None`.
4. Artifact order per block remains left-to-right.
5. Preserve zero-copy behavior; allocation should only occur at raw creation
   boundaries.

---

## Components to Remove / Cleanup (Explicit)

### Parser Global Scan Pipeline
- Remove `master_artifacts` creation in `MarkdownParser::parse`.
- Remove `filter_artifacts_by_range` function.
- Remove `is_scannable_position` function.
- Remove `ScannedBlock` struct.

### Raw Model Bloat
- Remove `master_artifacts` field from `RawNote`.
- Update `RawNote::new` signature and all callers.

---

## Affected Files (Primary)

- `lithos-core/src/note/parser.rs`
- `lithos-core/src/note/scanner.rs`
- `lithos-core/src/note/raw/aggregate.rs`
- `lithos-core/src/note/raw/list.rs`
- `lithos-core/src/note/aggregate.rs` (tests and RawNote destructure)

### Additional Likely Touchpoints
- `lithos-core/src/note/raw/mod.rs` (exports if signatures change)
- Any tests using `RawListItem::new` or `RawNote::new`
- Tests in `lithos-core/src/note/parser.rs`

---

## High-Level Architecture Change

### Before

```
scan_block(full markdown) -> master_artifacts
  ↓
parser collects scannable_ranges per block
  ↓
filter_artifacts_by_range(master_artifacts, ranges)
  ↓
raw outputs
```

### After

```
parser collects scannable_ranges per block
  ↓
scan_ranges(text, ranges) per block
  ↓
raw outputs
```

Benefits: single-pass, no global artifact cache, smaller memory footprint.

---

## Detailed Implementation Plan

### Phase 1: Scanner API for Range-Driven Scans

**Goal**: Add a scanning entrypoint that accepts event ranges and preserves
cursor state across disjoint ranges.

**Tasks**:

1. Add a new helper to `NoteScanner` in
   `lithos-core/src/note/scanner.rs`:
   - **Proposed signature**:
     ```rust
     pub fn scan_ranges<'source>(
         &self,
         text: &'source str,
         ranges: &[std::ops::Range<usize>],
         artifacts: &mut Vec<ScannedArtifact<'source>>,
     ) -> Result<(), NoteError>
     ```
   - **Subtasks**:
     - Create a `Cursor` once, reuse with `Cursor::reset`.
     - For each range:
       - Skip empty ranges.
       - Slice `text[range.clone()]` safely (`get`), and use range.start as
         `base_offset` via `SourceByteOffset::try_from`.
     - Keep cursor `prev_alnum` and `mode` between ranges.
     - Propagate `NoteError` for offset overflow.

2. (Optional) Add a small conversion helper in `parser.rs`:
   - Convert `ScannedArtifact` into raw structs to avoid duplicating match
     logic across finalize functions.

**Notes**:
- This does not alter existing `scan_block` or `scan_cursor` APIs.
- `scan_block` remains available for tests and standalone usage.

---

### Phase 2: Switch Parser to Event-Driven Scanning

**Goal**: Remove the global scan + filter pass.

**Tasks**:

1. In `MarkdownParser::parse` (`lithos-core/src/note/parser.rs`):
   - Remove `master_artifacts = scanner.scan_block(...)`.
   - Keep `NoteScanner` instance for range-driven calls.

2. Update block finalization paths to scan locally:
   - **Heading close** (`BlockKind::Heading`):
     - call `scanner.scan_ranges` with `block.scannable_ranges`.
     - attach tags/fields/refs to heading/sections.
   - **Paragraph finalization** (`finalize_paragraph`):
     - call `scanner.scan_ranges` with `block.scannable_ranges`.
     - attach tags/fields/refs to paragraph.
   - **List item finalization** (`finalize_list_item`):
     - call `scanner.scan_ranges` with `block.scannable_ranges`.
     - attach tags/fields/refs and task marker.

3. Remove old filter machinery:
   - Delete `filter_artifacts_by_range`, `is_scannable_position`, and
     `ScannedBlock`.
   - Remove `master_artifacts` parameters from:
     - `handle_end_tag`
     - `finalize_paragraph`
     - `finalize_list_item`

**Attention**:
- `block.scannable_ranges` are derived from `Event::Text` and `Event::Code`.
- Soft/Hard breaks do **not** add ranges; verify that tags/inline fields do
  not depend on scanning across break boundaries.

---

### Phase 3: Remove `master_artifacts` from RawNote

**Goal**: Eliminate raw storage bloat.

**Tasks**:

1. Update `RawNote` in `lithos-core/src/note/raw/aggregate.rs`:
   - Remove `master_artifacts` field.
   - Update `RawNote::new` signature.

2. Update call sites:
   - `MarkdownParser::parse` in `parser.rs`.
   - Any tests or helper constructors.

3. Update `RawNote` destructuring in `lithos-core/src/note/aggregate.rs`:
   - Should still compile, but verify.

---

### Phase 4: Add `is_checked` and Preserve Marker

**Goal**: Provide checkbox indicator and marker char together.

**Tasks**:

1. Update `RawListItem` in `lithos-core/src/note/raw/list.rs`:
   - Add `pub is_checked: Option<bool>`.
   - Update `RawListItem::new` signature.
   - Update `into_owned`.

2. Update list item finalization in `parser.rs`:
   - Use pulldown `Event::TaskListMarker(bool)` to set `is_checked`.
   - Only use scanner task marker if `is_checked` is `Some(_)`.
   - If marker missing but `is_checked` is `Some(_)`, log warning and continue.

3. Update all `RawListItem::new` call sites:
   - `parser.rs` (list item creation)
   - `note/aggregate.rs` tests (`list_item_from_text`)
   - Any other tests/helpers

**Marker Policy**:
- Do **not** infer marker char from `is_checked`.
- Treat missing marker as a scanning failure; still preserve `is_checked`.

---

### Phase 5: Logging / Tracing for Missing Marker

**Goal**: Make missing marker observable without hard error.

**Tasks**:

1. Identify existing logging facade in the codebase:
   - Search for `tracing::` or `log::` usage.
   - Use existing facade (preferred). If none, add minimal `tracing::warn!`
     only if already a dependency.

2. Log when `is_checked.is_some()` and marker missing:
   - Include:
     - `block_range` or list item range
     - `is_checked` value
     - note path if available

---

### Phase 6: Test Updates

**Goal**: Keep tests aligned with new flow.

**Tasks**:

1. Update `lithos-core/src/note/parser.rs` tests:
   - Add or adjust tests to assert:
     - `is_checked` is `Some(true)` for `- [x]`.
     - `is_checked` is `Some(false)` for `- [ ]`.
     - `task_marker` is present when `is_checked` is `Some(_)`.

2. Update `lithos-core/src/note/aggregate.rs` tests:
   - `list_item_from_text` helper must populate `is_checked`.
   - If it synthesizes `task_marker`, set `is_checked` accordingly.

3. Verify that tests using `scan_block` still pass.
   - No change required; ensure updated data model doesn’t break helpers.

---

### Phase 7: Verification

**Tasks**:

1. Run:
   - `mise run test:unit:note`
   - `mise run test:unit:core`
2. If failures in domain conversion:
   - Check `ListItem::try_from` uses `task_marker` only (still OK).
   - Confirm no new `unwrap` usage.

---

## Risk Register and Mitigations

### Risk 1: Missing markers in some edge formatting
- **Mitigation**: Log warning, do not error. Keep `is_checked`.

### Risk 2: Range-driven scan misses artifacts due to segmentation
- **Mitigation**:
  - Preserve cursor state across ranges.
  - Keep scanning order identical to range order.
  - Add tests for tags/fields near range boundaries.

### Risk 3: Task marker artifacts detected outside checkbox scope
- **Mitigation**:
  - Ignore `TaskMarker` artifacts unless `is_checked.is_some()`.

---

## Success Criteria

- `parser.rs` no longer performs a global scan or filtering pass.
- `RawNote` no longer stores `master_artifacts`.
- Each block scans only its event-derived ranges.
- `RawListItem` includes `is_checked` and retains the exact marker when present.
- Missing marker results in a warning log, not a hard error.
- Tests updated and passing.

---

## Rollback Strategy

- If new scanning flow breaks parsing, revert to global scan temporarily and
  keep range-driven helper as a feature flag. (Only if needed.)
- If `is_checked` causes churn, keep it but gate usage behind optional logic
  in the domain layer.

---

## Quick Reference (Change Checklist)

**Remove**:
- `master_artifacts` (RawNote + parser)
- `filter_artifacts_by_range`
- `is_scannable_position`
- `ScannedBlock`

**Add**:
- `NoteScanner::scan_ranges`
- `RawListItem.is_checked`

**Modify**:
- `MarkdownParser::parse`
- `finalize_paragraph`, `finalize_list_item`, heading close path
- `RawNote::new`
- `RawListItem::new` and `into_owned`
- Tests in parser and aggregate
