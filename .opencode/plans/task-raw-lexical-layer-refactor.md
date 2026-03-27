# Task Plan: Raw Lexical Layer Refactor (Highly Detailed)

**Status**: Planning
**Created**: 2026-03-27
**Priority**: High
**Owner**: Note Context (parser/scanner/raw/domain)
**Scope**: Lithos note ingestion pipeline

---

## Executive Summary

Refactor the note ingestion pipeline so `Raw*` types represent **lexical
evidence** (syntax‑level parsed results) while all **config‑driven semantics**
and **validation** live in a later interpretation layer (resolver/domain).

Key principles:
- **Raw = lexical parsing + provenance** (ranges, source bytes, typed literals).
- **Domain = semantic meaning + validation** (config rules, promotion logic).
- **Scanner/Parser alignment**: eliminate duplicated token shapes where possible
  by aligning scanner output with Raw tokens.

This plan is intentionally comprehensive to support resuming after session
compaction or a new conversation.

---

## Why This Change (Conceptual Rationale)

Markdown extraction is not structured deserialization (unlike JSON/TOML/YAML).
It is **signal extraction** from free‑form text. Therefore:

1. **Raw must remain stable evidence** of what the file said.
2. **Lexical typing** (dates, numbers, booleans) is still parsing, but it is
   purely syntactic and should remain in Raw.
3. **Config‑driven meaning** (emoji → keyword, task promotion, field constraints)
   is semantic and should not live in Raw.

This refactor makes the boundary explicit and reduces accidental coupling.

---

## Definitions

**Lexical parsing**: converting text to typed literals without consulting
config or semantics (e.g., `"2024-03-20" → Date`).

**Semantic interpretation**: applying config rules (e.g., `"📅" → "due"`) and
validation (type constraints, promotion tags).

---

## Current Pain Points (Concrete Examples)

1. `RawInlineField::map_emoji_key` depends on `TaskConfigSpec`.
   - This is semantic interpretation living in Raw.

2. `RawList` and `RawFrontmatter` embed config specs.
   - Raw becomes non‑portable and config‑coupled.

3. Scanner emits `ScannedArtifact` which duplicates Raw shapes.
   - Leads to redundant conversions and more code paths.

---

## Target Architecture

```
File / IO
  ↓
Scanner (token extraction)
  ↓
Parser (structural assembly + lexical typing)
  ↓
Raw DTOs (lexical evidence + provenance)
  ↓
Resolver (semantic mapping + config)
  ↓
Domain (validation + normalization)
  ↓
Storage
```

**Note:** `RawFrontmatter` can include parsed YAML/TOML trees (lexical parsing)
because pulldown‑cmark only extracts the block, it does not parse it.

---

## Invariants (Must Hold)

1. Raw types **never** depend on config for meaning.
2. Raw types **may** perform lexical parsing.
3. Domain types **never** parse markdown text directly.
4. Scanner/Parser output is always representable by Raw types (no semantic loss).
5. All mapping of emoji → keyword and promotion logic occurs after Raw.

---

## Non‑Goals (Explicit)

- Removing Raw types entirely (Raw stays as DTO boundary).
- Migrating lexical parsing to Domain.
- Rewriting pulldown‑cmark usage beyond targeted changes.

---

## Phase Plan (Detailed)

### Phase 0 — Inventory & Baseline (No code changes)

**Purpose**: Document current state and all config‑dependent Raw behaviors.

**Tasks**:
1. Identify every Raw* type holding config or behavior:
   - `RawInlineField::map_emoji_key` (config behavior)
   - `RawList.task_spec` (config embedded)
   - `RawFrontmatter.spec` (config embedded)
2. Identify all scanner/parser types duplicating Raw shapes:
   - `ScannedArtifact::Tag` vs `RawTag`
   - `ScannedArtifact::BlockRef` vs `RawBlockRef`
   - `ScannedArtifact::InlineField` vs `RawInlineField`
3. Confirm lexical parsing is already in Raw:
   - `RawFieldValue::from_str_with_spec`
   - `RawFrontmatter` YAML/TOML parsing

**Outputs**:
- A checklist of all refactor points.

---

### Phase 1 — Align Scanner with Raw (Reduce Duplication)

**Purpose**: Reduce duplication between `ScannedArtifact` and Raw types.

**Decision**: Raw remains lexical evidence. Scanner should emit Raw‑aligned
tokens where possible.

#### Option A (Recommended): Mixed Raw + Token

**Concept**:
- Scanner emits RawTag / RawBlockRef directly.
- Scanner emits a lightweight RawInlineFieldToken (key/value text + range).
- Parser applies lexical typing to create RawInlineField.

**Tasks**:
1. Add `RawInlineFieldToken` struct (new file or in `raw/inline_field.rs`).
2. Update scanner to output:
   - `RawTag` instead of `ScannedArtifact::Tag`.
   - `RawBlockRef` instead of `ScannedArtifact::BlockRef`.
   - `RawInlineFieldToken` instead of `ScannedArtifact::InlineField`.
3. Update parser conversion:
   - `RawInlineFieldToken` → `RawInlineField` via `RawFieldValue::from_str_with_spec(None)`.
4. Remove `ScannedArtifact::Tag/BlockRef/InlineField` or replace `ScannedArtifact`
   entirely with Raw‑aligned outputs.

**Benefits**:
- Scanner outputs Raw‑aligned tokens, reducing duplication.
- Parser remains the place where lexical typing is applied for inline fields.

#### Option B (Alternate): Scanner emits RawInlineField directly

**Concept**:
- Scanner calls lexical parser (`RawFieldValue::from_str_with_spec(None)`)
  and emits RawInlineField immediately.

**Benefits**:
- One less type (no token).
**Trade‑off**:
- Scanner becomes aware of RawFieldValue.

**Decision Needed**:
- Choose Option A (preferred for layered separation) or Option B (fewer types).

---

### Phase 2 — Remove Config/Behavior from Raw

**Purpose**: Ensure Raw contains only lexical evidence, not semantics.

**Tasks**:
1. Remove `RawInlineField::map_emoji_key` from `raw/inline_field.rs`.
2. Remove `task_spec` field from `RawList`.
3. Remove `spec` field from `RawFrontmatter`.

**Migration Targets**:
- Emoji mapping → resolver or domain conversion.
- Task spec → task promotion step.
- Frontmatter spec → frontmatter resolver/constructor.

**Checks**:
- No Raw type should reference config types after this phase.

---

### Phase 3 — Resolver Layer (Semantic Interpretation)

**Purpose**: Provide a dedicated place for config‑driven semantics.

**Tasks**:
1. Prefer `TryFrom<Raw*>` / `From<Raw*>` implementations for semantic mapping
   instead of a separate resolver module.
   - `impl TryFrom<(&RawInlineField, &TaskConfigSpec)> for InlineField`
   - `impl TryFrom<(&RawFrontmatter, &FrontmatterConfigSpec)> for Frontmatter`
   - `impl TryFrom<(&RawListItem, &TaskConfigSpec)> for Task` (if appropriate)
2. Update conversion call sites to use these `TryFrom`/`From` implementations.

**Detailed conversion rules (explicit and testable):**
- InlineField conversion:
  - Inputs: `&RawInlineField`, `&TaskConfigSpec`.
  - Emoji mapping:
    - If `raw.key` is a single emoji and exists in `task_spec.temporal_specs`,
      map to the corresponding keyword.
    - Unknown emoji policy must be defined (see Open Decisions).
  - Key normalization:
    - Preserve raw key string; do not alter casing here.
    - Let `InlineFieldKey::new` handle normalization (kebab/snake) later.
  - Value mapping:
    - Convert `RawFieldValue` to `FieldValue` without re‑parsing.
    - Reject/handle impossible variants only if domain constraints require it.
- Frontmatter conversion:
  - Inputs: `&RawFrontmatter`, `&FrontmatterConfigSpec`.
  - Use `RawFrontmatter` parsed value tree; do not re‑parse the text.
  - Apply spec keys to extract title/aliases/tags/file_class/dates.
  - Preserve unknown fields unless the current domain behavior discards them.
- Task conversion:
  - Inputs: `&RawListItem`, `&TaskConfigSpec`.
  - Require `raw.is_checked.is_some()` and `raw.task_marker.is_some()`.
  - Apply promotion tags logic exactly as current Task promotion.

**Detailed conversion rules (must be specified explicitly and tested):**
- InlineField conversion:
  - Accepts `(&RawInlineField, &TaskConfigSpec)`.
  - If `raw.key` is a single emoji, map using `task_spec.temporal_specs`.
  - If mapping fails, use a defined policy (see Open Decisions).
  - Preserve `RawFieldValue` as is; do not re‑parse the raw text.
  - Use `Box<str>` for keys; avoid new `String` allocations.
- Frontmatter conversion:
  - Accepts `(&RawFrontmatter, &FrontmatterConfigSpec)`.
  - Reads parsed YAML/TOML tree and applies key mappings.
  - Preserve unknown fields if current domain behavior does so.
  - Errors must carry key/path context consistent with existing errors.
- Task conversion:
  - Accepts `(&RawListItem, &TaskConfigSpec)` when promotion is needed.
  - Requires `raw.is_checked.is_some()` and `raw.task_marker.is_some()`.
  - Promotion tags logic unchanged; use spec from conversion call site.

**Why this matters**:
- Keeps domain constructors free of config logic.
- Centralizes semantics for easier testing.

---

### Phase 4 — RawFrontmatter as Parsed Evidence

**Purpose**: Formalize RawFrontmatter’s role in parsing YAML/TOML.

**Tasks**:
1. Keep parsed YAML/TOML trees in RawFrontmatter:
   - e.g., `serde_yaml::Value` or `toml::Value` or unified `serde_json::Value`.
2. Remove semantic interpretation from RawFrontmatter.
3. Update resolver/domain to interpret the parsed tree using config.

**Parsing behavior requirements:**
- Parsing errors must preserve line/column info (current NoteParseError rules).
- Sanitization for Obsidian links remains in Raw (YAML) before parsing.
- Parsing must not depend on config specs or semantic rules.

**Parsing behavior requirements:**
- Preserve existing parse error types and range information.
- Avoid re‑parsing once `RawFrontmatter` has a parsed tree.
- Keep YAML/TOML parsing in Raw to avoid domain IO/parsing creep.

**Outcome**:
- RawFrontmatter = parsed evidence.
- Domain Frontmatter = validated meaning.

---

### Phase 5 — Domain Conversion Updates

**Purpose**: Move semantic logic fully into domain/resolver layer.

**Tasks**:
1. Add or adjust `TryFrom`/`From` impls on domain types for Raw inputs with
   config context.
2. Ensure semantic mapping (emoji→keyword, constraints) lives in these impls.
3. Update task promotion to use config passed at conversion time (not stored
   in RawList).

**Implementation checklist:**
- Avoid new `String` allocations where `Box<str>` or borrows are possible.
- Keep `FrontmatterError` / `NoteError` flows unchanged where possible.
- Do not re‑parse textual values in domain conversion.

**Implementation checklist:**
- Avoid `unwrap()`; convert errors into existing domain error types.
- Use `Box<str>` and borrowed refs where possible (follow idioms).
- Do not introduce new parsing of strings in domain (lexical parsing stays Raw).

---

### Phase 6 — Tests and Regression Coverage

**Purpose**: Ensure behavior matches current functionality after refactor.

**Tasks**:
1. Add tests for lexical typing stays in Raw:
   - Inline field values (date, number, bool) typed in Raw.
2. Add tests for semantic mapping in resolver:
   - Emoji → keyword mapping based on TaskConfigSpec.
3. Ensure RawFrontmatter parsing still works (YAML/TOML).
4. Update scanner/parser tests for new token types or Raw‑aligned output.

---

## File‑by‑File Impact Matrix

### Scanner
- `lithos-core/src/note/scanner.rs`
  - Replace `ScannedArtifact` or refactor to use Raw tokens.
  - Introduce `RawInlineFieldToken` if Option A chosen.

### Parser
- `lithos-core/src/note/parser.rs`
  - Consume Raw tokens from scanner.
  - Convert inline field tokens to `RawInlineField` via lexical typing.

### Raw Types
- `lithos-core/src/note/raw/inline_field.rs`
  - Remove `map_emoji_key`.
  - Add token struct if needed.
- `lithos-core/src/note/raw/list.rs`
  - Remove `task_spec`.
- `lithos-core/src/note/raw/frontmatter.rs`
  - Remove config spec field.
  - Ensure parsed tree is stored.

### Domain/Resolver
- `lithos-core/src/note/inline_fields.rs`
- `lithos-core/src/note/frontmatter.rs`
- `lithos-core/src/note/task.rs` (task promotion uses spec)
- New `lithos-core/src/note/resolver.rs` (if chosen)

---

## Migration Checklist (Detailed)

1. Decide Option A vs Option B for inline fields.
2. Add/modify scanner output types.
3. Update parser conversion logic.
4. Remove config fields from Raw types.
5. Add resolver layer or update domain conversion paths.
6. Update all constructor signatures and call sites.
7. Update tests.
8. Run full validation:
   - `mise run test:unit:note`
   - `mise run test:unit:core`

**Call‑site checklist (non‑exhaustive):**
- `note/aggregate.rs` conversions and helper functions
- `note/task.rs` promotion helpers and tests
- `note/storage.rs` fixtures and tests
- any `RawListItem::new` / `RawNote::new` constructors

**Call‑site checklist (non‑exhaustive):**
- `note/aggregate.rs` conversions and helper functions
- `note/task.rs` promotion helpers and tests
- `note/storage.rs` fixtures and tests
- any `RawListItem::new` / `RawNote::new` constructors

---

## Open Decisions (Explicit)

1. **Inline Field Output**:
   - A) Token + lexical typing in parser (preferred).
   - B) Scanner emits RawInlineField directly.

2. **Parsed Frontmatter Value Type**:
   - `serde_yaml::Value` / `toml::Value` stored as is,
     or convert both to `serde_json::Value`.

3. **Resolver Location**:
   - Use `TryFrom<Raw*>` / `From<Raw*>` impls (preferred).

4. **Unknown emoji policy**:
   - Preserve raw key, drop field, or tag as invalid.
5. **Unknown frontmatter fields**:
   - Preserve in domain or drop during conversion.

4. **Unknown emoji policy**:
   - Preserve raw key, drop field, or tag as invalid.
5. **Unknown frontmatter fields**:
   - Preserve in domain (current behavior?) or drop during conversion.

---

## Success Criteria

- Raw types contain **no config references**.
- Raw types still perform lexical parsing.
- Semantic interpretation is centralized outside Raw.
- Scanner and parser no longer duplicate Raw shapes unnecessarily.
- All tests pass and behavior unchanged for users.

---

## Rollback Strategy

If semantic changes cause regressions:
- Keep Raw lexical parsing intact.
- Temporarily reintroduce config mapping in resolver only (not Raw).
- Avoid restoring config fields in Raw unless absolutely necessary.

---

## Notes

- This plan explicitly keeps RawFrontmatter parsing in Raw because
  pulldown‑cmark does not parse YAML/TOML; only extraction is possible there.
- The “Raw = lexical evidence” model is aligned with your requirement that
  Raw types are the only DTOs between parsing and domain.
