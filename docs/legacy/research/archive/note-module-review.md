# Note Module Review: Findings & Implementation Guidance

**Date**: 2026-02-26
**Scope**: `lithos-core/src/note/` (full), plus ingestion dependencies (context-only)
**Status**: Exhaustive read-only audit (no refactor performed)

---

## 1. Executive Summary

The note module is structurally solid (CQRS ports, clean domain types, rkyv
usage, well-formed adapters), but has critical correctness gaps in indexing
and markdown parsing. Several paths drop user-visible text or mis-handle
external URLs. Indexes beyond path/tag are declared and queried but never
written, effectively breaking multiple query APIs. The test suite contains
time-based flakiness and a large set of low-value getter tests that dilute
signal. Performance issues exist but are secondary to correctness.

This document lists **all** identified issues and provides prioritized
remediation guidance. It also includes an explicit separation of
out-of-scope modules reviewed for context only.

---

## 2. Scope and Boundaries

### 2.1 In-Scope (Reviewed for correctness, performance, tests, idioms)

- `lithos-core/src/note/**/*.rs`
- `lithos-core/tests/note_ingest.rs`
- `lithos-core/benches/note_parsing.rs` (note ingestion)
- `lithos-core/benches/db_storage.rs` (note usage only)

### 2.2 Context-Only (Reviewed for ingestion/validation context, **not** for refactoring)

- `lithos-core/src/fs/*` (reader, validator, types)
- `lithos-core/src/config/*` (task, frontmatter, aggregate)
- `lithos-core/src/application/*` (schema service only)

**Important**: Any findings in context-only modules are explicitly flagged as
out-of-scope for refactoring in this review.

---

## 3. Architecture & Layering Checks

### 3.1 Domain/Infrastructure Separation

- Domain types in `note/` do not import fs/infrastructure, which is good.
- Ingestion is performed via `NoteReader` (adapter) with `FileReader`, consistent
  with architecture constraints.

### 3.2 Port-Based CQRS

- Query/Command are generic over storage ports as required.
- Zero-copy access patterns are present in query adapters.

### 3.3 Context Isolation

- Note context does not import schema/template contexts. No violations found.

---

## 4. Critical Findings (Correctness)

### N-CR-01 — Indexes Declared and Queried but Never Written

**Severity**: Critical

The note module declares and queries multiple indexes but only writes two.
`PATH_TO_ID` and `TAGS_TO_NOTES` are maintained; all others are never written.
This makes query APIs for aliases, file class, folders, frontmatter, and task
indexes return empty results even when data is present in notes.

**Missing index writes**:

- `ALIAS_TO_ID`
- `FILE_CLASS_TO_ID`
- `FOLDER_TO_ID`
- `TASKS_BY_*`
- `FRONTMATTER_KV`

**Impact**:

- `Query::find_by_alias`, `find_by_file_class`, `find_by_folder`,
  `find_by_task_*`, `find_by_frontmatter` are effectively non-functional.

**Files**:

- `lithos-core/src/note/mod.rs`
- `lithos-core/src/note/adapter/command.rs`
- `lithos-core/src/note/adapter/query.rs`

---

### N-CR-02 — Link Text Dropped from Headings and List Items

**Severity**: Critical

When a link is active, `Event::Text` is consumed by link alias handling and not
appended to heading or list-item text buffers. This removes visible link text
from headings and list items, affecting task content and heading extraction.

**Files**:

- `lithos-core/src/note/adapter/reader.rs`

---

### N-CR-03 — External URL Fragment Handling is Wrong

**Severity**: Critical

`build_link` splits destination URLs on `#` indiscriminately. For external
links (`https://example.com#frag`), the fragment should remain in the URL. It
is currently split into a separate anchor, which is invalid for external links.

**Files**:

- `lithos-core/src/note/adapter/reader.rs`

---

### N-CR-04 — Markdown Images Misclassified as Wiki-Style Embeds

**Severity**: High

Markdown image links (`![alt](url)`) are stored via `Link::new_embed`, which
sets style `WikiLink` and implies Obsidian-style embedding. Markdown images
should retain `MdLink` style.

**Files**:

- `lithos-core/src/note/adapter/reader.rs`
- `lithos-core/src/note/link.rs`

---

### N-CR-05 — Frontmatter Block Parsing Likely Loses Newlines

**Severity**: High

While inside metadata blocks, only `Event::Text`/`Event::Code` are appended to
the YAML buffer. `SoftBreak`/`HardBreak` are ignored, so multi-line YAML can be
collapsed and mis-parsed.

**Files**:

- `lithos-core/src/note/adapter/reader.rs`

---

## 5. Major Findings (Edge Cases & Semantics)

### N-MJ-01 — NotePath Normalization Does Not Reject `.` Components

`Component::CurDir` is ignored, allowing paths like `./note.md`. If the vault
expects normalized paths, this is inconsistent with other validation rules.

**Files**:

- `lithos-core/src/note/aggregate.rs`

---

### N-MJ-02 — Windows Prefix Handling is Undefined in NotePath

`Component::Prefix` is ignored, so `C:foo.md` or `C:\foo.md` can slip through
in some contexts. `PathValidator` in fs handles Windows drive detection, but
`NotePath::try_new` does not.

**Files**:

- `lithos-core/src/note/aggregate.rs`

---

### N-MJ-03 — Task Tag Validation Rules Are Inconsistent

- `Tag::new` allows Unicode alphanumeric characters.
- Task tag regex matches ASCII only.
- Config task tags are ASCII-only.

This creates inconsistent behavior between tag parsing and task tagging.

**Files**:

- `lithos-core/src/note/tag.rs`
- `lithos-core/src/note/task.rs`
- `lithos-core/src/config/task.rs` (context-only)

---

### N-MJ-04 — Task::extract_tags Fails the Entire Task on a Single Invalid Tag

If any matched tag is invalid, the task fails to parse even when the rest of
the task is valid. This is overly strict for real-world notes.

**Files**:

- `lithos-core/src/note/task.rs`

---

### N-MJ-05 — Task::should_promote Uses Naive Substring Matching

`text.contains(tag)` can match in the middle of words or code, causing false
positives. Token-aware matching is safer and aligns with intent.

**Files**:

- `lithos-core/src/note/task.rs`

---

### N-MJ-06 — External Anchor Validation Should Reject or Ignore Anchors

`Link::validate_external_anchor` only rejects block refs. For external URLs,
anchors should remain part of the URL, not a separate anchor field.

**Files**:

- `lithos-core/src/note/link.rs`

---

## 6. pulldown-cmark Usage Review

### 6.1 Enabled Options

- `ENABLE_WIKILINKS`
- `ENABLE_TASKLISTS`
- `ENABLE_YAML_STYLE_METADATA_BLOCKS`

### 6.2 Missing Options (Potential Obsidian Gaps)

- `ENABLE_PLUSES_DELIMITED_METADATA_BLOCKS` (TOML frontmatter)
- `ENABLE_GFM` (callouts like `[!NOTE]`)

### 6.3 Event Handling Gaps

- Missing link text in headings/list items (N-CR-02).
- Metadata newline handling (N-CR-05).
- Markdown link alias not captured for `[]()` links.

---

## 7. Performance & Idiomatic Rust Issues

### N-PR-01 — String Allocation Anti-Patterns

Multiple occurrences of `"literal".to_owned()` violate repo guidelines.
These should be replaced by `"literal".into()`.

**Files**:

- `lithos-core/src/note/tag.rs` (example)

---

### N-PR-02 — UUID Stringification in Hot Paths

`Uuid::to_string()` is used repeatedly in adapter loops. This is avoidable
allocation. Not critical, but measurable at scale.

**Files**:

- `lithos-core/src/note/adapter/command.rs`
- `lithos-core/src/note/adapter/query.rs`

---

### N-PR-03 — FieldValue JSON Semantics Are Questionable

- `FieldValue::from_json` silently maps non-representable numbers to `0.0`.
- `FieldValue::to_json_string` is unused, does not escape control chars, and
  serializes maps in nondeterministic order.

**Files**:

- `lithos-core/src/note/value.rs`

---

## 8. Test Suite Audit

### 8.1 Flaky/Time-Dependent Tests

**N-TF-01**: `task_timestamp_provides_semantic_methods` uses fixed dates and
calls `is_future(None)`. This fails as time advances.

**File**:

- `lithos-core/src/note/task.rs`

---

### 8.2 Low-Value Tests

Many tests only validate getters or trivial constructors with no edge cases.
These inflate the suite without increasing confidence.

**Examples**:

- `lithos-core/src/note/structure.rs` (Heading/Section accessors)
- `lithos-core/src/note/frontmatter.rs` (accessor-only cases)
- `lithos-core/src/note/value.rs` (getter-only cases)

---

### 8.3 Missing Edge Case Tests

Recommended tests not currently present:

- Link text inside headings/list items.
- Markdown images vs wiki embeds.
- External URL fragments.
- Metadata blocks with line breaks.
- `NotePath` with `.` components and Windows prefixes.
- Task tag parsing with invalid tags and mixed Unicode.
- Task promotion boundary matching.

---

## 9. Superfluous or Underused Components

These items appear unused or used only in tests/benches. Removal is not
recommended without product direction, but they should be flagged.

### N-SU-01 — Section is Never Built by the Parser

`Note` supports sections, but `NoteReader` never constructs them. `Section`
appears only in tests/bench data.

**Files**:

- `lithos-core/src/note/structure.rs`
- `lithos-core/src/note/aggregate.rs`

---

### N-SU-02 — NoteEvents Are Not Emitted Outside Tests

`Note` accumulates events, but no production pipeline consumes them.

**Files**:

- `lithos-core/src/note/aggregate.rs`
- `lithos-core/src/note/events.rs`

---

## 10. Context-Only Findings (No Refactor)

These are **explicitly out of scope** for refactoring in this review.

### 10.1 fs Reader/Validator Mismatch with NotePath

`PathValidator::validate_vault_path` rejects dotfiles and Windows prefixes.
`NotePath::try_new` does not. This mismatch should be documented if `NotePath`
is intended to represent already-validated vault paths.

**Files**:

- `lithos-core/src/fs/validator.rs` (context-only)
- `lithos-core/src/note/aggregate.rs`

---

### 10.2 Task Config is ASCII-Only

Config task tags require ASCII-only; note tag parsing allows Unicode.

**Files**:

- `lithos-core/src/config/task.rs` (context-only)
- `lithos-core/src/note/tag.rs`

---

### 10.3 Frontmatter Config Is Minimal and Consistent

No issues found; behavior is consistent with note frontmatter handling.

**File**:

- `lithos-core/src/config/frontmatter.rs` (context-only)

---

### 10.4 Application Layer Has No Note Ingestion Service

Only schema service exists. Note ingestion is adapter-driven via `NoteReader`.

**File**:

- `lithos-core/src/application/mod.rs` (context-only)

---

## 11. Recommended Remediation (Priority)

This section is guidance only. No refactor performed.

### P0 (Immediate)

- Fix index writes for all declared indexes.
- Fix link text handling in headings/items.
- Fix external URL fragment handling.
- Fix markdown image style classification.
- Preserve frontmatter newlines in metadata blocks.

### P1 (Short Term)

- Align tag validation rules (Unicode vs ASCII) across Tag/Task/config.
- Make task promotion tag matching token-aware.
- Adjust task tag extraction to avoid full-task failure.
- Normalize/validate NotePath curdir and Windows prefix cases.

### P2 (Cleanup)

- Remove string allocation anti-patterns.
- Replace time-dependent tests with deterministic timestamps.
- Replace low-value tests with edge-case coverage.
- Review FieldValue JSON semantics and unused `to_json_string`.

---

## 12. Verification Checklist (Exhaustive Review)

- [x] All `lithos-core/src/note/**/*.rs` reviewed end-to-end
- [x] Integration tests (`lithos-core/tests/note_ingest.rs`) reviewed
- [x] Benchmarks (`note_parsing`, `db_storage`) reviewed for note usage
- [x] pulldown-cmark options and event model verified
- [x] Ingestion dependencies reviewed (fs, config, application) for context

---

## 13. Issue Index (Quick Reference)

| ID      | Severity | Title                                        |
| ------- | -------- | -------------------------------------------- |
| N-CR-01 | Critical | Indexes declared/queried but never written   |
| N-CR-02 | Critical | Link text dropped from headings/list items   |
| N-CR-03 | Critical | External URL fragments mis-handled           |
| N-CR-04 | High     | Markdown images treated as wiki embeds       |
| N-CR-05 | High     | Frontmatter parsing loses newlines           |
| N-MJ-01 | Major    | NotePath allows `.` components               |
| N-MJ-02 | Major    | NotePath ignores Windows prefixes            |
| N-MJ-03 | Major    | Tag validation rules inconsistent            |
| N-MJ-04 | Major    | Task::extract_tags fails task on invalid tag |
| N-MJ-05 | Major    | Task::should_promote uses substring match    |
| N-MJ-06 | Major    | External anchor validation inconsistent      |
| N-PR-01 | Minor    | String allocation anti-patterns              |
| N-PR-02 | Minor    | UUID stringification in hot paths            |
| N-PR-03 | Minor    | FieldValue JSON semantics questionable       |
| N-TF-01 | Major    | Time-dependent task timestamp test           |
| N-SU-01 | Minor    | Sections never constructed                   |
| N-SU-02 | Minor    | NoteEvents not used outside tests            |
