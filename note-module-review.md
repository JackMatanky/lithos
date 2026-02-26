# Note Module Review: Findings & Implementation Guidance

**Date**: 2026-02-26
**Scope**: `lithos-core/src/note/` (full), plus ingestion dependencies (context-only)
**Status**: Exhaustive read-only audit (no refactor performed)

---

## 1. Executive Summary

The note module is structurally solid (CQRS ports, clean domain types, rkyv
usage, well-formed adapters), but has critical correctness gaps in indexing
and markdown parsing. Link text inside headings/list items is lost, markdown
link display text/alt text is never captured, and external URL fragments are
split incorrectly. Note tags are never extracted by the reader, so tag indexes
are silently empty. Indexes beyond path/tag are declared and queried but never
written, effectively breaking multiple query APIs. The test suite contains
time-based flakiness and a large set of low-value getter tests that dilute
signal. Performance issues exist but are secondary to correctness.

This document lists **all** identified issues and provides prioritized
remediation guidance. It also includes an explicit separation of
out-of-scope modules reviewed for context only, and a pulldown-cmark 0.13.1
capability review.

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
- Ingestion is performed via `NoteReader` (adapter) with `FsReader`, consistent
  with current architecture (only `FsReader`/`FsWriter` exist).

### 3.2 Port-Based CQRS

- Query/Command are generic over storage ports as required.
- Zero-copy access patterns are present in query adapters.

### 3.3 Context Isolation

- Note context does not import schema/template contexts. No violations found.

### 3.4 Source Location Semantics (Offsets vs Lines)

- The module stores source byte offsets/ranges in domain types.
- Line numbers are not more stable for persistence; any line insertion or
  deletion above a span invalidates all downstream line numbers.
- Byte offsets are precise and align with pulldown-cmark output. If line/column
  display is needed, derive it from the source text at render time.

### 3.5 Task Parsing Config Flow (Current)

- `NoteReader` owns `Config` and passes `config.task()` into task parsing.
- `Task::should_promote` and `Task::from_checkbox` currently take `&TaskConfig`.
- This means task parsing in `note/task.rs` depends directly on config types,
  rather than being driven purely by a higher-level service.

### 3.6 Type-Driven Design Review

- Most domain entities use strong newtypes (`NoteId`, `NotePath`, `TaskId`) and
  `Box<str>` for owned strings, which aligns with the type-driven guidelines.
- Errors still carry untyped `String` payloads where `NoteId`/`NotePath` would
  be more expressive and safer.

---

## 4. Critical Findings (Correctness)

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
It also means markdown link display text and image alt text are never captured
as `Link::alias` (only wikilink aliases are stored).

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
should retain `MdLink` style. Additionally, `Tag::Image` is always routed
through `Target::Unresolved`, so external image URLs become unresolved targets
and image alt text is discarded.

**Files**:

- `lithos-core/src/note/adapter/reader.rs`
- `lithos-core/src/note/link.rs`

---

### N-CR-05 — Frontmatter Block Parsing Likely Loses Newlines

**Severity**: High

While inside metadata blocks, only `Event::Text`/`Event::Code` are appended to
the YAML buffer. `SoftBreak`/`HardBreak` are ignored, so if pulldown-cmark emits
line breaks for metadata blocks, multi-line YAML can be collapsed and
mis-parsed. (If metadata arrives as a single `Text` event with newlines, this
is harmless; verify with a targeted test.)

**Files**:

- `lithos-core/src/note/adapter/reader.rs`

---

### N-CR-06 — Wikilink Alias Text Overwrites Instead of Merging

`LinkState.alias` is overwritten on each `Event::Text` while a wikilink alias
is active. pulldown-cmark can emit multiple consecutive `Event::Text` segments,
so aliases can be truncated unless the text is appended or merged.

**Files**:

- `lithos-core/src/note/adapter/reader.rs`

---

### N-CR-07 — Plus-Delimited Metadata Blocks Ignored

Only `MetadataBlockKind::YamlStyle` is handled, so TOML frontmatter delimited
by `+++` is ignored even if the option is enabled.

**Files**:

- `lithos-core/src/note/adapter/reader.rs`

---

### N-CR-08 — Note Tags Are Never Extracted

`NoteReader` does not extract tags from the note body or frontmatter into
`Note.tags`, yet the storage layer indexes `note.tags()` into `TAGS_TO_NOTES`.
This makes tag indexes and tag-based queries empty unless tags are injected by
some other pipeline (none found in this module).

**Files**:

- `lithos-core/src/note/adapter/reader.rs`
- `lithos-core/src/note/aggregate.rs`
- `lithos-core/src/note/adapter/command.rs`

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

---

### N-MJ-07 — Wiki Embeds Drop Anchors Entirely

`![[note#heading]]` and `![[note#^block]]` are valid Obsidian embed forms, but
`Link::new_embed` has no anchor parameter and `build_link` discards computed
anchors when `is_embed == true`. Anchors are silently lost.

**Files**:

- `lithos-core/src/note/adapter/reader.rs`
- `lithos-core/src/note/link.rs`

---

### N-MJ-08 — Non-HTTP Schemes Treated as Internal Links

`is_external_url` only recognizes `http://` and `https://`. Autolinks or
markdown links for `mailto:`, `tel:`, `obsidian://`, etc. are stored as
`Target::Unresolved` instead of `Target::External`.

**Files**:

- `lithos-core/src/note/adapter/reader.rs`

---

### N-MJ-09 — Custom Status Symbols Are Unrepresentable

The task config supports arbitrary printable ASCII status symbols, but parsing
only uses `Event::TaskListMarker(checked)` and maps it to `' '` or `'x'`. Any
custom status symbol in the source is lost because pulldown-cmark does not
expose the original bracket character.

**Files**:

- `lithos-core/src/note/adapter/reader.rs`
- `lithos-core/src/config/task.rs` (context-only)

---

### N-MJ-10 — Note Path Uniqueness Is Not Enforced

`CommandAdapter::create` does not check for an existing note with the same
path. `PATH_TO_ID` is a multimap, and `Query::find_by_path` returns only the
first ID. This makes duplicate paths possible and produces ambiguous reads.

**Files**:

- `lithos-core/src/note/adapter/command.rs`
- `lithos-core/src/note/adapter/query.rs`

---

### N-MJ-11 — SourceByteRange Has No Ordering Validation

`SourceByteRange::new` and `Section::new` accept any start/end combination.
This allows inverted or zero-length ranges to be stored without validation.
If range semantics matter (e.g., section highlighting), enforce `start <= end`.

**Files**:

- `lithos-core/src/note/types.rs`
- `lithos-core/src/note/structure.rs`

---

### N-MJ-12 — Task Tag Regex Overmatches and Is ASCII-Only

`Task::extract_tags` uses a regex that matches `#[a-zA-Z0-9_\-/]+` anywhere in
the raw task text. This can capture tags inside URLs/code or other contexts and
does not match the Unicode allowance in `Tag::new`.

**Files**:

- `lithos-core/src/note/task.rs`

---

### N-MJ-13 — Module Docs Overstate Supported Features

The note module docs claim “full support” for hierarchical tags and TOML
frontmatter, but the reader does not populate `Note.tags` and only parses YAML
metadata blocks. The docs should reflect current capabilities.

**Files**:

- `lithos-core/src/note/mod.rs`
- `lithos-core/src/note/frontmatter.rs`

---

## 6. pulldown-cmark Usage Review

### 6.1 Enabled Options

- `ENABLE_WIKILINKS`
- `ENABLE_TASKLISTS`
- `ENABLE_YAML_STYLE_METADATA_BLOCKS`

### 6.2 Missing Options (Potential Obsidian Gaps)

- `ENABLE_PLUSES_DELIMITED_METADATA_BLOCKS` (TOML frontmatter)
- `ENABLE_GFM` (callouts via `Tag::BlockQuote(BlockQuoteKind)`)

### 6.3 Event Handling Gaps

- Missing link text in headings/list items (N-CR-02).
- Markdown link display text / image alt text not captured (N-CR-02/N-CR-04).
- Metadata newline handling (N-CR-05).
- Autolink/email link types are available but not used for classification.
- Offsets are available via `Parser::into_offset_iter` but not used to stabilize
  merged text handling.

### 6.4 pulldown-cmark 0.13.1 Features Not Leveraged

- `Parser::into_offset_iter()` yields `(Event, Range<usize>)` to capture
  positions for all events, not just a few domains.
- `utils::TextMergeWithOffset` merges adjacent `Event::Text` while keeping
  offsets aligned.
- `LinkType::{Autolink, Email}` provide scheme-aware handling without custom
  URL prefix checks.
- `Tag::Heading` exposes optional `id`, `classes`, `attrs` when heading
  attributes are enabled.
- `Tag::BlockQuote` optionally includes `BlockQuoteKind` when `ENABLE_GFM` is
  enabled (callout detection).
- `MetadataBlockKind` distinguishes YAML vs plus-delimited metadata blocks.

---

### N-CR-06 — Wikilink Alias Text Overwrites Instead of Merging

`LinkState.alias` is overwritten on each `Event::Text` while a wikilink alias
is active. pulldown-cmark can emit multiple consecutive `Event::Text` segments,
so aliases can be truncated unless the text is appended or merged.

**Files**:

- `lithos-core/src/note/adapter/reader.rs`

---

### N-CR-07 — Plus-Delimited Metadata Blocks Ignored

Only `MetadataBlockKind::YamlStyle` is handled, so TOML frontmatter delimited
by `+++` is ignored even if the option is enabled.

**Files**:

- `lithos-core/src/note/adapter/reader.rs`

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

### N-PR-04 — Offset Types Not Paired With Line/Column Utilities

Offsets are stored as bytes (correct), but there is no standard utility to
derive line/column for diagnostics or UI. This leads to ad-hoc calculations
and risks inconsistencies.

**Files**:

- `lithos-core/src/note/types.rs`

---

### N-PR-05 — Link Alias Uses Per-Event Allocation

Alias text is stored as `String` and overwritten/allocated per `Event::Text`.
This is both incorrect (see N-CR-06) and allocation-heavy compared to merging
or using `TextMergeWithOffset`.

**Files**:

- `lithos-core/src/note/adapter/reader.rs`

---

### N-PR-06 — Task Status Getter Clones

`Task::status()` returns a cloned `StatusName` on each call. This is minor but
avoidable if the API returns `&StatusName` instead.

**Files**:

- `lithos-core/src/note/task.rs`

---

### N-PR-07 — Error Variants Store `String` Instead of `Box<str>`

`NoteError` and related error variants store owned `String`. The project
guidelines prefer `Box<str>` for immutable owned strings to reduce allocation
overhead. This is low priority but pervasive.

**Files**:

- `lithos-core/src/note/error.rs`

---

### N-PR-08 — `FieldValue` JSON Conversion Semantics Are Leaky

`FieldValue::from_json` accepts `serde_json::Value` but is used as a generic
conversion for YAML/TOML as well. It silently maps non‑representable numbers to
`0.0` and `Null` to empty string, which can corrupt metadata. If JSON interop is
required, prefer explicit `serde_json` serialization/deserialization or a
strict conversion that errors on lossy cases.

**Files**:

- `lithos-core/src/note/value.rs`

---

### N-PR-09 — Task Parsing Depends Directly on `TaskConfig`

`Task::from_checkbox` and related helpers take `&TaskConfig`, binding task
parsing to a lower-level config type and bypassing `Config` (aggregate) at the
call site. This complicates enforcing “active vault config” semantics and
makes the domain entity config-aware.

**Files**:

- `lithos-core/src/note/task.rs`
- `lithos-core/src/note/adapter/reader.rs`

---

### N-PR-10 — Task Text Stored as `String`

`Task.text` is stored as `String` but is immutable after construction. Use
`Box<str>` to reduce overallocation and align with project string guidelines.

**Files**:

- `lithos-core/src/note/task.rs`

---

### N-PR-11 — Task Parsing Logic Lives Inside the Domain Entity

`Task` owns parsing logic (regexes, metadata parsing, temporal field parsing).
This makes `Task` a god‑object within the note domain. Consider moving parsing
into a dedicated adapter/service (e.g., `TaskParser`) and keeping `Task` as a
pure value type.

**Files**:

- `lithos-core/src/note/task.rs`

---

### N-PR-12 — Errors Use Untyped Strings for IDs/Paths

Several `NoteError` variants store IDs/paths as `String` instead of using
`NoteId`/`NotePath`, which reduces type safety and makes errors harder to
consume programmatically.

**Files**:

- `lithos-core/src/note/error.rs`

---

### N-PR-13 — SourceByteRange Fields Are Public

`SourceByteRange` exposes `start`/`end` publicly, allowing construction of
invalid ranges without validation. This bypasses type-driven invariants.

**Files**:

- `lithos-core/src/note/types.rs`

---

### N-PR-14 — Frontmatter Aliases Always Allocate

`Frontmatter::aliases` returns a `Vec<Box<str>>` on every call, even when
callers only need borrowed data. Consider providing a borrowed accessor or
iterator to avoid repeated allocations.

**Files**:

- `lithos-core/src/note/frontmatter.rs`

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
- `lithos-core/src/note/tag.rs` (getter-only + trivial parsing)
- `lithos-core/src/note/link.rs` (accessor-only tests)
- `lithos-core/src/note/list.rs` (setter/getter only)

---

### 8.3 Missing Edge Case Tests

Recommended tests not currently present:

- Link text inside headings/list items.
- Markdown link display text / image alt text captured.
- Markdown images vs wiki embeds (style + target type).
- Wiki embeds with anchors (`![[note#heading]]`).
- External URL fragments and non-HTTP schemes.
- Metadata blocks with line breaks.
- `NotePath` with `.` components and Windows prefixes.
- Task tag parsing with invalid tags and mixed Unicode.
- Task promotion boundary matching.
- Custom status symbols (beyond `x`/space).
- Duplicate path prevention and query behavior.

---

## 9. Superfluous or Underused Components

These items appear unused or used only in tests/benches. Removal is not
recommended without product direction, and some are explicitly *not* removal
candidates due to planned usage (see notes).

### N-SU-01 — Section is Never Built by the Parser

`Note` supports sections, but `NoteReader` never constructs them. `Section`
appears only in tests/bench data. **Do not remove**: this is intended to
represent Obsidian `CachedMetadata` blocks and will be populated later.

**Files**:

- `lithos-core/src/note/structure.rs`
- `lithos-core/src/note/aggregate.rs`

---

### N-SU-02 — NoteEvents Are Not Emitted Outside Tests

`Note` accumulates events, but no production pipeline consumes them. **Do not
remove**: this is intended for event-driven design adoption.

**Files**:

- `lithos-core/src/note/aggregate.rs`
- `lithos-core/src/note/events.rs`

---

### N-SU-03 — No Additional Removal Candidates Found

Beyond the items above (and the explicitly unused `FieldValue::to_json_string`
utility), no other components in `note/` appear removable or superseded by
existing components without a product decision.

---

### N-SU-04 — `FieldValue::to_json_string` Is Unused and Fragile

`to_json_string` is unused and produces nondeterministic key ordering with
partial escaping. Either remove it or replace it with `serde_json` for stable
indexing output.

**Files**:

- `lithos-core/src/note/value.rs`

---

### N-SU-05 — TaskMetadata Convenience Accessors May Be Premature

`TaskMetadata::priority`, `project`, and `area` hardcode field names despite
the task metadata system being schema‑driven. If metadata keys are entirely
configurable, these methods may be redundant or belong in a higher‑level
service.

**Files**:

- `lithos-core/src/note/task.rs`

---

### N-SU-07 — Frontmatter::new Is Infallible

`Frontmatter::new` returns `Result` but currently cannot fail. Consider
returning `Self` directly or adding real validation to justify the fallible
API.

**Files**:

- `lithos-core/src/note/frontmatter.rs`

---

### N-SU-06 — NoteReader ParseState Is a God Object

`ParseState` owns list, task, heading, link, and frontmatter parsing and all
associated state. This concentrates multiple concerns into one type, making it
hard to reason about and test. Consider splitting into dedicated sub-parsers
(`ListParser`, `LinkParser`, `FrontmatterParser`) with shared event input.

**Files**:

- `lithos-core/src/note/adapter/reader.rs`

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
- Fix link text handling in headings/items and capture markdown link text/alt.
- Fix external URL fragment handling.
- Fix markdown image modeling (style + target + alt text handling).
- Preserve frontmatter newlines in metadata blocks (or assert pulldown emits).
- Use `LinkType::Autolink`/`Email` to classify external targets correctly.
- Merge link alias text across consecutive `Event::Text` events.
- Handle plus-delimited metadata blocks if TOML frontmatter is desired.
- Extract note tags into `Note.tags` (body and/or frontmatter) or remove tag
  indexes until population exists.

### P1 (Short Term)

- Align tag validation rules (Unicode vs ASCII) across Tag/Task/config.
- Make task promotion tag matching token-aware.
- Adjust task tag extraction to avoid full-task failure.
- Normalize/validate NotePath curdir and Windows prefix cases.
- Decide how to represent wiki-embed anchors and non-HTTP schemes.
- Reconcile custom task status symbols with parser limitations.
- Enforce path uniqueness or make query behavior deterministic for duplicates.
- Consider enabling heading attributes + GFM callouts and mapping to domain
  fields where relevant.
- Add a line/column derivation utility built on byte offsets.
- Validate `SourceByteRange` ordering if range semantics are used downstream.
- Consider routing task parsing exclusively through `Config` (aggregate)
  instead of accepting `TaskConfig` directly, to ensure active vault config is
  always used.
- Tighten task tag parsing to be token‑aware and consistent with `Tag` rules.
- Consider extracting `Task` parsing into a dedicated parser to keep the domain
  entity lean.
- Consider splitting `NoteReader::ParseState` into sub-parsers to reduce
  god‑object complexity.
- Replace stringly‑typed error payloads with `NoteId`/`NotePath` where possible.
- Align module docs with current parsing/ingestion capabilities.

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
| N-CR-06 | Critical | Wikilink alias text overwritten              |
| N-CR-07 | Critical | Plus-delimited metadata blocks ignored       |
| N-CR-08 | Critical | Note tags never extracted                    |
| N-MJ-01 | Major    | NotePath allows `.` components               |
| N-MJ-02 | Major    | NotePath ignores Windows prefixes            |
| N-MJ-03 | Major    | Tag validation rules inconsistent            |
| N-MJ-04 | Major    | Task::extract_tags fails task on invalid tag |
| N-MJ-05 | Major    | Task::should_promote uses substring match    |
| N-MJ-06 | Major    | External anchor validation inconsistent      |
| N-MJ-07 | Major    | Wiki embeds drop anchors                     |
| N-MJ-08 | Major    | Non-HTTP schemes treated as internal         |
| N-MJ-09 | Major    | Custom status symbols unrepresentable        |
| N-MJ-10 | Major    | Note path uniqueness not enforced            |
| N-MJ-11 | Major    | SourceByteRange ordering not validated       |
| N-MJ-12 | Major    | Task tag regex overmatches/ASCII‑only        |
| N-MJ-13 | Major    | Module docs overstate capabilities           |
| N-PR-01 | Minor    | String allocation anti-patterns              |
| N-PR-02 | Minor    | UUID stringification in hot paths            |
| N-PR-03 | Minor    | FieldValue JSON semantics questionable       |
| N-PR-04 | Minor    | No line/column utility for byte offsets      |
| N-PR-05 | Minor    | Link alias per-event allocation              |
| N-PR-06 | Minor    | Task status getter clones                    |
| N-PR-07 | Minor    | Error variants store String                  |
| N-PR-08 | Minor    | FieldValue JSON conversion semantics leaky   |
| N-PR-09 | Minor    | Task parsing depends on TaskConfig           |
| N-PR-10 | Minor    | Task text stored as String                   |
| N-PR-11 | Minor    | Task parsing inside domain entity            |
| N-PR-12 | Minor    | Errors use untyped strings for IDs/paths     |
| N-PR-13 | Minor    | SourceByteRange fields are public            |
| N-PR-14 | Minor    | Frontmatter aliases always allocate          |
| N-TF-01 | Major    | Time-dependent task timestamp test           |
| N-SU-01 | Minor    | Sections never constructed                   |
| N-SU-02 | Minor    | NoteEvents not used outside tests            |
| N-SU-03 | Minor    | No additional removal candidates             |
| N-SU-04 | Minor    | FieldValue::to_json_string unused            |
| N-SU-05 | Minor    | TaskMetadata convenience accessors           |
| N-SU-06 | Minor    | NoteReader ParseState god object             |
| N-SU-07 | Minor    | Frontmatter::new infallible                  |
