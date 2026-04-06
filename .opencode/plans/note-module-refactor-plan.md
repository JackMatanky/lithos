# Note Module Refactor Plan (Comprehensive, Phased)

## Purpose

This plan provides a complete, restartable refactor guide for
`lithos-core/src/note/`. It is designed so work can pause and resume without
losing context. It captures problems, why they matter, decisions, phases,
tasks, subtasks, acceptance criteria, and verification steps.

## Scope

- In scope: note parsing, raw-to-domain conversion, link resolution, task
  promotion, validation, and performance within `lithos-core/src/note/`.
- Out of scope: new storage schemas, new `*View` types, cross-context imports,
  repository method additions, and non-note modules unless required for tests.

## Constraints and Non-Negotiables

- Maintain raw/domain separation (no reference metadata in `Note`).
- Preserve context isolation (note must not import other business contexts).
- Use unified repository traits as-is (no file I/O in repositories).
- Avoid new dependencies unless essential; prefer small local parsing logic.
- No string allocation anti-patterns (avoid `to_owned().into()` etc.).
- Preserve user data on invalid Task fields (lenient behavior).

## Locked Decisions

- Invalid Task field values are stored as `FieldValue::String(raw)`.
- Emit `tracing::warn!` for invalid Task fields (configurable to `trace` later).
- Reference links resolved at parse time only; no stored ref metadata.

## Problem Findings (Detailed)

### Finding A: Reference link resolution is not CommonMark-correct

- Problem:
  - Reference labels are not normalized (case fold, whitespace collapse,
    escape handling).
  - Later definitions overwrite earlier ones ("last wins").
  - Scanner is line-based and does not correctly implement the spec.
- Why it matters:
  - Targets resolve incorrectly, breaking compatibility with CommonMark and
    other markdown tools.
  - Order-dependent behavior causes unstable results in large notes.
- Impacted files:
  - `lithos-core/src/note/parser.rs`

### Finding B: Reference definitions are parsed in frontmatter and code

- Problem:
  - Reference definition scan is applied to all markdown text without
    excluding frontmatter or code blocks.
- Why it matters:
  - Definitions inside YAML/TOML or code blocks should be ignored. Including
    them changes link resolution and violates markdown rules.
- Impacted files:
  - `lithos-core/src/note/parser.rs`

### Finding C: Task field specs are not enforced during promotion

- Problem:
  - `Task::promote` copies inline fields without validating against
    `TaskConfigSpec.field_specs`.
- Why it matters:
  - Domain data becomes untrusted; downstream features cannot rely on Task
    fields being valid.
- Impacted files:
  - `lithos-core/src/note/task.rs`
  - `lithos-core/src/note/aggregate.rs`

### Finding D: Date parsing ignores configured DateSpec

- Problem:
  - Inline field parsing is heuristic; `TaskDateValue` accepts typed values
    even when they violate DateSpec.
- Why it matters:
  - Configured validation is bypassed; user expectations are violated.
- Impacted files:
  - `lithos-core/src/note/raw/inline_field.rs`
  - `lithos-core/src/note/task.rs`

### Finding E: External URL detection is incomplete

- Problem:
  - Detection only checks `http/https/ftp/mailto`, not general schemes.
  - Fragment splitting can corrupt external URLs.
- Why it matters:
  - Links like `obsidian://` or `file://` are misclassified and broken.
- Impacted files:
  - `lithos-core/src/note/link.rs`
  - `lithos-core/src/note/parser.rs`

### Finding F: Tag dedup is O(n^2)

- Problem:
  - Tags are deduped by scanning a Vec repeatedly.
- Why it matters:
  - Performance degrades as tag count grows (lists + tasks + frontmatter).
- Impacted files:
  - `lithos-core/src/note/aggregate.rs`

### Finding G: Unused or overexposed API surface

- Problem:
  - Public types like `Segments` are unused.
- Why it matters:
  - Increases API bloat and maintenance cost; confusing for users.
- Impacted files:
  - `lithos-core/src/note/tag.rs`

### Finding H: Reference map built unconditionally

- Problem:
  - Reference definition map is built even when no reference links exist.
- Why it matters:
  - Extra allocations and scanning in common cases.
- Impacted files:
  - `lithos-core/src/note/parser.rs`

## Intended End State (Acceptance Criteria)

- Reference link resolution is CommonMark-compatible.
- Reference definitions ignore frontmatter and code blocks.
- First definition wins; labels are normalized.
- External URL detection handles any valid URI scheme.
- Task promotion validates fields; invalid values preserved as strings with
  warn logs.
- Date parsing respects configured DateSpec.
- Tag dedup uses O(n) or O(n log n) approach.
- Unused public types removed or hidden.
- All tests pass; no clippy warnings; formatting clean.

## Baseline Behavior Captured (Current State)

- Reference definitions are case-sensitive.
- Duplicate reference definitions: the last definition wins.
- Reference definitions inside frontmatter are currently scanned and used.
- Reference definitions inside fenced code blocks are currently ignored.

## Detailed Phased Refactor Plan

### Phase 0: Baseline and Alignment

Purpose: make the refactor safe to pause and resume.

Tasks:
- Capture current behavior in baseline tests:
  - Reference definitions in frontmatter/code blocks currently affect links.
  - Current reference resolution behavior ("last wins").
- Record all note-related behavior to be preserved (except for fixes).
- Confirm log level for invalid Task fields (default to warn).

Deliverables:
- Baseline tests or notes in this plan documenting current behavior.

Checkpoint:
- All baseline tests pass on current code.

### Phase 1: Reference Link Correctness

Purpose: fix reference link resolution to be spec-correct.

Tasks:
1) Implement label normalization (CommonMark):
   - ASCII lower-case
   - Collapse whitespace to single space
   - Handle backslash escapes
2) Ensure "first definition wins":
   - Only insert if label does not exist.
3) Exclude frontmatter and code blocks from scanning:
   - Skip frontmatter ranges from parser state.
   - Ignore fenced and indented code blocks from event stream.
4) Support optional titles and multi-line definitions where applicable.
5) Update reference resolution to use normalized label keys.

Subtasks:
- Define a helper for label normalization in `parser.rs` or a new module.
- Unit tests for normalization.
- Integration tests for reference definition precedence.

Deliverables:
- Updated reference definition scanning and resolution.
- New tests that validate CommonMark behavior.

Checkpoint:
- Reference link tests pass.
- No regressions in inline link parsing.

### Phase 2: External URL Detection

Purpose: avoid corrupting external URLs and support all schemes.

Tasks:
1) Replace scheme detection with RFC3986-style parsing:
   - `ALPHA *( ALPHA / DIGIT / "+" / "-" / "." ) ":"`
2) Treat any valid scheme as external.
3) Split anchors only for internal targets.

Subtasks:
- Add tests for `obsidian://`, `file://`, `s3://`.
- Add test for external URL with fragment.

Deliverables:
- Correct external/internal classification.

Checkpoint:
- All link-related tests pass.

### Phase 3: Task Field Validation (Lenient)

Purpose: validate Task fields while preserving raw data.

Tasks:
1) Enforce `TaskConfigSpec.field_specs` during `Task::promote`:
   - Validate fields when spec exists.
   - On invalid value, store `FieldValue::String(raw)`.
   - Emit `tracing::warn!` with field key and raw value.
2) Date parsing alignment:
   - Prefer DateSpec-based parsing when spec exists.
   - Fallback to heuristic only if explicitly allowed (config or default).
3) Ensure inline field parsing respects specs where available.

Subtasks:
- Add a validation helper for field specs in `task.rs`.
- Wire note path into log context if available.
- Tests for invalid field value fallback.

Deliverables:
- Task promotion validates and preserves invalid values.
- Logs visible for invalid fields.

Checkpoint:
- Task promotion tests updated and passing.

### Phase 4: Performance Improvements

Purpose: reduce overhead in common cases.

Tasks:
1) Build reference definition map lazily:
   - Create only after first reference link event.
2) Replace tag dedup O(n^2):
   - Use `HashSet` keyed by `full_path` or
   - Sort + dedup after collection.
3) Reduce intermediate allocations in raw-to-domain conversion:
   - Use direct iterator pipelines to build `Box<[T]>`.

Deliverables:
- Lower allocations and improved performance in parsing.

Checkpoint:
- Benchmarks or targeted perf tests (if available) show no regression.

### Phase 5: API Surface and Bloat Cleanup

Purpose: reduce unused and overexposed API.

Tasks:
1) Remove or privatize unused public types (example: `Segments`).
2) Narrow module-level `#[allow]` or `#[expect]` to smallest scope.
3) Audit `note/mod.rs` re-exports for unused items.

Deliverables:
- Reduced public surface and less bloat.

Checkpoint:
- No external API consumers broken (if any), or documented breaking changes.

### Phase 6: Verification and Hardening

Purpose: ensure correctness, quality gates, and documentation updates.

Tasks:
- Run quality gates:
  - `mise run fmt`
  - `mise run lint`
  - `mise run test`
  - `mise run adr:validate` if ADR changes were made
- Run `cargo test --doc` if public docs/examples changed.
- Update module docs describing reference link behavior and Task field
  validation.

Deliverables:
- Clean build and tests.
- Updated documentation.

Checkpoint:
- All tests pass and no clippy warnings.

## Restart Guide (When Resuming Work)

1) Identify last completed phase.
2) Re-read the "Deliverables" and "Checkpoint" items for that phase.
3) Re-run tests and quality gates relevant to the last phase.
4) Continue with the next phase tasks.

## Open Questions (Keep Updated)

- Do we want a configuration flag to downgrade invalid field logs to `trace`?
- Should heuristic date parsing be allowed if a DateSpec exists?
- Do we want to track invalid field metrics (counts) for analytics?

## File Map (Primary Touch Points)

- Parsing and link resolution:
  - `lithos-core/src/note/parser.rs`
- Link domain:
  - `lithos-core/src/note/link.rs`
- Task promotion and validation:
  - `lithos-core/src/note/task.rs`
  - `lithos-core/src/note/aggregate.rs`
- Tag domain:
  - `lithos-core/src/note/tag.rs`

## Testing Matrix

- Reference links:
  - normalization
  - first definition wins
  - ignore frontmatter
  - ignore code blocks
  - multi-line definitions
- External schemes:
  - `obsidian://`
  - `file://`
  - `s3://`
  - external with fragment
- Task fields:
  - invalid values stored as string
  - logs emitted
  - DateSpec enforcement
- Performance:
  - no regression in parsing time for note without reference links

## Risks and Mitigations

- Behavior change in reference resolution:
  - Mitigation: tests + documentation describing CommonMark compliance.
- Logging noise from invalid fields:
  - Mitigation: configuration for log level or rate limiting later.
- Hidden coupling between parsing and task promotion:
  - Mitigation: keep validation in `Task::promote` only; avoid spreading logic.

## Out of Scope

- Storage schema changes or new domain types for validation errors.
- New repository methods or cross-context imports.
- Adding new dependencies beyond minimal parsing (unless required).
