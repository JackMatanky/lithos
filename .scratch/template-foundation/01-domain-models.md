---
title: 01-domain-models
category: enhancement
label: ready-for-agent
status: open
branch:
merge_commit:
date_created: 2026-06-11
date_completed:
---

# Template Domain Models

Status: ready-for-agent

## Parent

`.scratch/template-foundation/PRD.md`

## What to build

Define all Template domain value objects and aggregates that the rest of the Template context depends on. This is the foundational data layer — no behavior, no persistence, no engine.

Introduce:

- `TemplateId` — newtype over `UuidV7`, following the `NoteId`/`SchemaId` pattern. Inner field is `pub(crate)`. Derives `Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord` plus `rkyv::Archive, rkyv::Serialize, rkyv::Deserialize` with `#[rkyv(derive(Debug, Hash, PartialEq, Eq))]` (matching `SchemaId`). Implements `Default` + `Display` (delegating to inner `UuidV7`). Provides `From<UuidV7>` + `TryFrom<Uuid>` conversions (matching `SchemaId`).
- `TemplateName` — derived from the vault-relative path stem of the template file, qualified by subdirectory structure within the configured template directory (e.g. `templates/daily/standup.md` → `TemplateName("daily/standup")`). The `/` separator is permitted. Validated at construction from a file path, not a separate field.
- `TemplateBody` — wraps renderable source text. Structural invariants only: non-empty, valid UTF-8. Jinja syntax validity is explicitly NOT the domain's responsibility.
- `RawTemplate` — thin raw-content DTO carrying the file path and raw string content.
- `RawTemplateView` — freshness/cache view carrying `PathKey`, `Blake3Hash`, file metadata (`FileMetadata` for mtime/size), and recorded time. Implements `HasContentHash` and `HasContentHashMut` from `support::content_hash` (both are `pub(crate)` — permissible since all types live in `lithos-core`). Follows the versioned-history pattern of `RawSchemaView` (ring buffer of versions), not a flat struct. Derives `rkyv::Archive, rkyv::Serialize, rkyv::Deserialize`.
- `Template` — primary renderable aggregate with `TemplateId`, `PathKey`, derived `TemplateName`, `TemplateBody`, and recorded ingestion time. Non-exhaustive (`#[non_exhaustive]`) for future frontmatter/query evolution. Derives `rkyv::Archive, rkyv::Serialize, rkyv::Deserialize`.

All domain types live in `lithos-core` under the `template` context. No MiniJinja types appear anywhere in this module.

**Code file placement:**
- `Template` → `aggregate.rs` (alongside any error types)
- `RawTemplate` → `raw.rs`
- `RawTemplateView` → `views.rs`
- If `aggregate.rs` gets too long, extract `TemplateId` + `TemplateName` into `identifier.rs`

**Visibility principle:** Start with minimal visibility — prefer private or `pub(crate)` over `pub` unless there's a concrete cross-context caller. Only add derives/trait impls with clear justification (avoid cargo-culting from `NoteId`/`SchemaId` without evaluating whether each derive is needed). For example, `Hash` on `TemplateId` is required by redb key semantics; `Copy` is justified because `UuidV7` is Copy and the newtype is small.

## Acceptance criteria

- [ ] `TemplateId` is a newtype over `UuidV7` with `new()`, `parse()`, and `as_uuid_v7()` methods, matching the `NoteId`/`SchemaId` pattern
- [ ] `TemplateId` inner field is `pub(crate)` (matching `NoteId`/`SchemaId`)
- [ ] `TemplateId` derives `rkyv::Archive + Serialize + Deserialize` with `#[rkyv(derive(Debug, Hash, PartialEq, Eq))]`
- [ ] `TemplateId` implements `Default` (delegates to `UuidV7::new()`) and `Display` (delegates to inner `UuidV7`)
- [ ] `TemplateId` implements `From<UuidV7>` and `TryFrom<Uuid>` (matching `SchemaId`)
- [ ] `TemplateName` uses a fallible constructor named `try_new()` per naming taxonomy (`try_` prefix for `Result`-returning constructors), taking `(&Path, &Path)` for template file path and template directory root
- [ ] `TemplateName` rejects paths that would produce an empty stem
- [ ] `TemplateBody` rejects empty strings at construction; valid UTF-8 is guaranteed by Rust's `String` type
- [ ] `TemplateBody` makes no claim about Jinja syntax validity
- [ ] `RawTemplate` is a thin DTO — no validation beyond what `PathKey` and `String` provide
- [ ] `RawTemplateView` holds `PathKey`, `Blake3Hash`, file metadata, and recorded time
- [ ] `RawTemplateView` implements `HasContentHash` and `HasContentHashMut`
- [ ] `RawTemplateView` derives `rkyv::Archive + Serialize + Deserialize`
- [ ] `Template` is `#[non_exhaustive]` with all fields private; provides accessor methods for each field (`id()`, `path()`, `name()`, `body()`, `recorded_at()`)
- [ ] `Template` derives `rkyv::Archive + Serialize + Deserialize`
- [ ] Unit tests cover: `TemplateId` construction, parse round-trip, `Default` and `Display` impls, `From<UuidV7>`/`TryFrom<Uuid>` conversions, rkyv round-trip; `TemplateName` derivation (flat and subdirectory cases); `TemplateBody` empty rejection; `RawTemplateView` hash trait impls, rkyv round-trip; `Template` accessor methods, rkyv round-trip

## Blocked by

None — can start immediately.

---

> *This was generated by AI during triage.*

## Triage Notes

**Codebase context gathered from existing patterns:**

| Pattern | Source | Key details |
|---------|--------|-------------|
| `NoteId` | `lithos-core/src/note/aggregate.rs:58` | `UuidV7` newtype, `pub(crate)` field, `#[rkyv(derive(Debug))]`, impl `Default` + `Display`, `From<NoteId> for Uuid` + `TryFrom<Uuid>` |
| `SchemaId` | `lithos-core/src/schema/identifier.rs:55` | Same as `NoteId` but `#[rkyv(derive(Debug, Hash, PartialEq, Eq))]` and `From<UuidV7>` — **no `parse()` method** |
| `RawSchemaView` | `lithos-core/src/schema/views/raw.rs:45` | Versioned ring buffer: `path: PathKey` + `versions: Vec<SchemaVersion>`, impl `RawView`/`RawViewRead`, `MAX_VERSIONS = 5` |
| `HasContentHash` | `lithos-core/src/support/content_hash.rs:25` | `pub(crate)` trait — implementors must be in `lithos-core` |
| `HasContentHashMut` | `lithos-core/src/support/content_hash.rs:37` | `pub(crate)` trait — mutable extension of `HasContentHash` |
| `Blake3Hash` | `lithos-core/src/support/content_hash.rs:42` | `pub(crate)` newtype over `[u8; 32]`, Copy semantics |

**Key findings and gaps filled:**
1. `HasContentHash`/`HasContentHashMut` are `pub(crate)` — fine since template context lives inside `lithos-core`
2. `SchemaId` does NOT have `parse()` — only `NoteId` does. The issue's "matching the NoteId/SchemaId pattern" description is self-contradictory on this point; `parse()` is retained since `TemplateId` needs round-trip string parsing for storage
3. `TemplateName` constructor should be `try_new()` per naming taxonomy (`docs/naming-taxonomy.md`: `try_` prefix for `Result`-returning constructors; `new()` reserved for infallible)
4. Both `NoteId` and `SchemaId` implement `Default` + `Display` — added to `TemplateId` requirements
5. `SchemaId` has `From<UuidV7>` and `TryFrom<Uuid>` — added to `TemplateId` requirements
6. `#[rkyv(derive(Debug, Hash, PartialEq, Eq))]` matches `SchemaId`'s richer rkyv derives (NoteId only has `Debug`)
7. Template module is commented out in `lib.rs:28` (`// pub mod template;  // TODO: rebuild from scratch`) — will need to uncomment
8. Workspace `Cargo.toml` enables `missing_docs = "deny"` — doc comments on all public items are mandatory, not optional
9. `unsafe_code = "forbid"`, `unwrap_used`/`expect_used` denied — no `unwrap()`/`expect()` in production code

**Risk assessment: LOW** — this is a greenfield module with clear existing patterns to follow. The main risk is missing rkyv or naming taxonomy details, which this triage has addressed.

## Agent Brief

**Category:** enhancement
**Summary:** Define all Template domain value objects and aggregates in `lithos-core` under the `template` context

**Current behavior:**
No `template` context exists in `lithos-core`. There are no `TemplateId`, `TemplateName`, `TemplateBody`, `RawTemplate`, `RawTemplateView`, or `Template` types. The rest of the template-foundation slice (issues 02–09) cannot proceed until these types are defined.

**Desired behavior:**
A `template` context module exists in `lithos-core/src/template/` exporting the following domain types, all free of MiniJinja imports:

- `TemplateId` — newtype over `UuidV7` with `new()`, `parse()`, and `as_uuid_v7()` methods, matching the `NoteId`/`SchemaId` pattern. Inner field `pub(crate)`. Derives `Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, rkyv::Archive, rkyv::Serialize, rkyv::Deserialize` with `#[rkyv(derive(Debug, Hash, PartialEq, Eq))]`. Implements `Default`, `Display`, `From<UuidV7>`, `TryFrom<Uuid>`.
- `TemplateName` — constructed from a file path relative to the configured template directory; produces subdirectory-qualified stems using `/` as separator (e.g. a file at `templates/daily/standup.md` with template dir `templates/` yields `TemplateName("daily/standup")`). Rejects paths with an empty stem.
- `TemplateBody` — wraps renderable source text with structural invariants only: non-empty string. Does not validate Jinja syntax — that is the engine's responsibility.
- `RawTemplate` — thin DTO carrying a `PathKey` and raw `String` content. No additional validation.
- `RawTemplateView` — freshness/cache view carrying `PathKey`, `Blake3Hash`, file metadata (mtime, size), and recorded time. Follows the versioned-history pattern of `RawSchemaView` (not a flat struct). Implements `HasContentHash` and `HasContentHashMut` from `support::content_hash`. Derives `rkyv::Archive, rkyv::Serialize, rkyv::Deserialize`.
- `Template` — primary renderable aggregate with `TemplateId`, `PathKey`, `TemplateName`, `TemplateBody`, and recorded ingestion time. Marked `#[non_exhaustive]`. Derives `rkyv::Archive, rkyv::Serialize, rkyv::Deserialize`.

**Key interfaces:**
- `TemplateId` — mirror the `NoteId`/`SchemaId` pattern; use `UuidV7` as the inner type with `pub(crate)` visibility; `new()` generates a fresh V7 UUID, `parse()` parses a string, `as_uuid_v7()` returns `&UuidV7`; implement `Default` (delegates to `new()`), `Display` (delegates to inner), `From<UuidV7>`, `TryFrom<Uuid>`
- `TemplateName` — fallible constructor `try_new()` (per naming taxonomy: `try_` prefix for `Result`-returning) takes a `&Path` (path of the template file) and a `&Path` (template directory root); strips the root prefix then takes the path stem; `/` is permitted as separator in the resulting name string
- `TemplateBody` — constructor rejects empty strings; provides an accessor returning `&str`
- `RawTemplateView` — must implement `HasContentHash` (returns `&Blake3Hash`) and `HasContentHashMut` (returns `&mut Blake3Hash`) from the existing `support::content_hash` module
- `Template` — all fields private; provide accessor methods; must be `#[non_exhaustive]`

**Acceptance criteria:**
- [ ] `TemplateId::new()` generates a fresh identifier; `TemplateId::parse()` round-trips through `as_uuid_v7()`
- [ ] `TemplateId` inner field is `pub(crate)`; derives `Default` + `Display` (delegating to inner `UuidV7`)
- [ ] `TemplateId` has `From<UuidV7>` and `TryFrom<Uuid>` conversions (matching `SchemaId`)
- [ ] `TemplateId` derives `rkyv::Archive + rkyv::Serialize + rkyv::Deserialize` with `#[rkyv(derive(Debug, Hash, PartialEq, Eq))]`
- [ ] `TemplateName` derives the correct stem for flat files (e.g. `standup` from `templates/standup.md`)
- [ ] `TemplateName` derives the correct subdirectory-qualified stem (e.g. `daily/standup` from `templates/daily/standup.md`)
- [ ] `TemplateName` returns an error or `None` when the path produces an empty stem
- [ ] `TemplateBody::new("")` returns an error; non-empty strings succeed
- [ ] `TemplateBody` makes no claim about Jinja syntax — no validation against a Jinja parser
- [ ] `RawTemplate` is constructible from a `PathKey` and `String` without additional validation
- [ ] `RawTemplateView` holds `PathKey`, `Blake3Hash`, file metadata, and recorded time
- [ ] `RawTemplateView` correctly implements `HasContentHash` and `HasContentHashMut`
- [ ] `RawTemplateView` derives `rkyv::Archive + rkyv::Serialize + rkyv::Deserialize`; round-trip serialization test passes
- [ ] `Template` is `#[non_exhaustive]` and derives `rkyv::Archive + rkyv::Serialize + rkyv::Deserialize`; round-trip serialization test passes
- [ ] `Template` all fields private with accessor methods: `id()`, `path()`, `name()`, `body()`, `recorded_at()`
- [ ] No `minijinja` import appears anywhere in the `template` context module
- [ ] All types have doc comments on public items
- [ ] `mise run test` passes

**Out of scope:**
- Repository traits, processor, service, engine, CLI (issues 02–09)
- redb storage adapter implementation
- MiniJinja dependency or any Jinja syntax validation
- Frontmatter parsing or query semantics
- `TemplateExtension`, `ExtensionRegistry`, or any extension types
