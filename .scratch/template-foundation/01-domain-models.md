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
- `RawTemplate` — thin raw-content newtype wrapping `String`. No path, no metadata — the ingestion context (processor) carries those separately.
- `RawTemplateView` — flat freshness/cache struct carrying `PathKey`, `Blake3Hash`, file metadata (`FileMetadata` for mtime/size), and recorded time. Implements `HasContentHash` and `HasContentHashMut` from `support::content_hash` (both are `pub(crate)` — permissible since all types live in `lithos-core`). This is a flat struct, NOT a versioned ring buffer (the schema pattern uses a ring buffer because schemas have inheritance metadata; templates do not). Derives `rkyv::Archive, rkyv::Serialize, rkyv::Deserialize`.
- `TemplateNameError`, `TemplateBodyError`, `TemplateError` — domain error types in `error.rs`. `TemplateNameError` is a single-variant enum (`Derivation`) covering stem derivation failures. `TemplateBodyError` covers empty content rejection. `TemplateError` embeds both via `#[from]`.
- `Template` — primary renderable aggregate with `TemplateId`, `PathKey`, derived `TemplateName`, `TemplateBody`, and recorded ingestion time. Non-exhaustive (`#[non_exhaustive]`) for future frontmatter/query evolution. Derives `rkyv::Archive, rkyv::Serialize, rkyv::Deserialize`.

All domain types live in `lithos-core` under the `template` context. No MiniJinja types appear anywhere in this module.

**Code file placement:**
- `mod.rs` — submodule declarations (`pub(crate) mod`) and public re-exports
- `aggregate.rs` — `TemplateId`, `TemplateName`, `TemplateBody`, `Template`
- `raw.rs` — `RawTemplate(String)`
- `views.rs` — `RawTemplateView` (flat struct)
- `error.rs` — `TemplateNameError`, `TemplateBodyError`, `TemplateError`
- If `aggregate.rs` gets too long, extract `TemplateId` + `TemplateName` into `identifier.rs`

**Visibility principle:** Start with minimal visibility — prefer `pub(crate)` over `pub` unless there's a concrete cross-context caller. Submodules declared `pub(crate)` in `mod.rs`; only specific types re-exported as `pub`. Only add derives/trait impls with clear justification (avoid cargo-culting from `NoteId`/`SchemaId` without evaluating whether each derive is needed). For example, `Hash` on `TemplateId` is required by redb key semantics; `Copy` is justified because `UuidV7` is Copy and the newtype is small.

**Newtype trait contracts:**
| Type | Inner | `AsRef<Inner>` | Notes |
|------|-------|----------------|-------|
| `TemplateId` | `UuidV7` | `AsRef<UuidV7>` | `Display`+`Default`+`From<UuidV7>`+`TryFrom<Uuid>`. Field `pub(crate)`. Std+rkyv derives matching `SchemaId`. |
| `TemplateName` | `String` | `AsRef<str>` | `Display`. rkyv derives. No `Deref`/`Borrow` in first pass. |
| `TemplateBody` | `String` | `AsRef<str>` | No `Deref`. |
| `RawTemplate` | `String` | `AsRef<str>` | `.into_inner(self) -> String`. No `Deref`. |

## Acceptance criteria

- [ ] `TemplateId` is a newtype over `UuidV7` with `new()`, `parse()`, and `as_uuid_v7()` methods, matching the `NoteId`/`SchemaId` pattern
- [ ] `TemplateId` inner field is `pub(crate)` (matching `NoteId`/`SchemaId`)
- [ ] `TemplateId` derives `rkyv::Archive + Serialize + Deserialize` with `#[rkyv(derive(Debug, Hash, PartialEq, Eq))]`
- [ ] `TemplateId` implements `Default` (delegates to `UuidV7::new()`) and `Display` (delegates to inner `UuidV7`)
- [ ] `TemplateId` implements `From<UuidV7>` and `TryFrom<Uuid>` (matching `SchemaId`)
- [ ] `TemplateName` uses a fallible constructor named `try_new()` per naming taxonomy (`try_` prefix for `Result`-returning constructors), taking `(&Path, &Path)` for template file path and template directory root
- [ ] `TemplateName` rejects paths that would produce an empty stem → returns `TemplateNameError::Derivation`
- [ ] `TemplateBody` rejects empty strings at construction → returns `TemplateBodyError::Empty`; valid UTF-8 is guaranteed by Rust's `String` type
- [ ] `TemplateBody` makes no claim about Jinja syntax validity
- [ ] `RawTemplate` is a thin newtype around `String` — no validation; provides `AsRef<str>` and `into_inner()`
- [ ] `RawTemplateView` is a flat struct (NOT versioned ring buffer) holding `PathKey`, `Blake3Hash`, file metadata, and recorded time
- [ ] `RawTemplateView` implements `HasContentHash` and `HasContentHashMut`
- [ ] `RawTemplateView` derives `rkyv::Archive + Serialize + Deserialize`
- [ ] `Template` constructor `new(id, path, name, body)` sets `recorded_at` internally to `SystemTime::now()`; all fields private with accessor methods (`id()`, `path()`, `name()`, `body()`, `recorded_at()`)
- [ ] `Template` derives `rkyv::Archive + Serialize + Deserialize` with `#[rkyv(with = AsUnixTime)]` on `recorded_at`
- [ ] `TemplateNameError` is a single-variant enum (`Derivation`); `TemplateBodyError` is a single-variant enum (`Empty`); `TemplateError` embeds both via `#[from]`
- [ ] Newtypes implement `AsRef<InnerType>`: `TemplateId: AsRef<UuidV7>`, `TemplateName: AsRef<str>`, `TemplateBody: AsRef<str>`, `RawTemplate: AsRef<str>` — no `Deref`/`Borrow` in first pass
- [ ] No `minijinja` import appears anywhere in the `template` context module
- [ ] All types have doc comments on public items (enforced by `missing_docs = "deny"`)
- [ ] Unit tests cover per the TDD plan: `TemplateId` construction, parse round-trip, `Default` and `Display` impls, `From<UuidV7>`/`TryFrom<Uuid>` conversions, rkyv round-trip; `TemplateName` derivation (flat and subdirectory cases) + empty stem rejection; `TemplateBody` empty rejection; `RawTemplateView` hash trait impls, rkyv round-trip; `Template` constructor, accessor methods, rkyv round-trip; error Display formatting; no-minijinja enforcement

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

**Review decisions (2026-06-11):**
- `RawTemplateView` is a FLAT struct (not versioned ring buffer). The component model was correct — templates don't have inheritance metadata that would justify the ring buffer pattern. `RawSchemaView`'s versioning exists for incremental resolution of property bank references; templates have no equivalent.
- `RawTemplate` is a newtype around `String` only — no `PathKey`. Per decisions.md and component model, the ingestion context (processor) carries the path separately, matching how `RawSchema` works.
- `TemplateNameError` is a single-variant enum (`Derivation`) because the only domain validation `TemplateName::try_new` performs is stem derivation. Non-`.md` filtering and root-scope checking belong to the Template Processor (Issue 04), not the domain type.
- `TemplateBody::try_new` is the constructor name (not `new`), matching naming taxonomy for fallible constructors. All newtypes (except `RawTemplate`) use `try_new`.
- `Template::new` takes `(id, path, name, body)` without `recorded_at` — the timestamp is set internally to `SystemTime::now()` at construction. This keeps the constructor signature simple and avoids callers passing inconsistent timestamps.
- Newtypes implement `AsRef<InnerType>` for zero-cost access. `Deref` and `Borrow` are deferred — they introduce semantic coupling (method forwarding, map-key semantics) that should only be added when a concrete use case demands it.
- `TemplateNameError`, `TemplateBodyError`, `TemplateError` live in `error.rs`. `TemplateError` embeds both child errors via `#[from]`, following the composition pattern of `FsError`.
- Submodule visibility: `pub(crate)` in `mod.rs` by default. Public re-exports only for types with concrete cross-context callers.

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
- `RawTemplate` — thin raw-content newtype wrapping `String`. No `PathKey`, no metadata — the ingestion context (processor) carries those separately.
- `RawTemplateView` — flat freshness/cache struct carrying `PathKey`, `Blake3Hash`, file metadata, and recorded time. This is a flat struct, NOT a versioned ring buffer. Implements `HasContentHash` and `HasContentHashMut` from `support::content_hash`. Derives `rkyv::Archive, rkyv::Serialize, rkyv::Deserialize`.
- `TemplateNameError`, `TemplateBodyError`, `TemplateError` — domain error types in `error.rs`. `TemplateNameError::Derivation` covers stem derivation failures. `TemplateError` embeds both child errors via `#[from]`.
- `Template` — primary renderable aggregate with `TemplateId`, `PathKey`, `TemplateName`, `TemplateBody`, and recorded ingestion time. `recorded_at` is set internally at construction, not passed as a parameter. Marked `#[non_exhaustive]`. Derives `rkyv::Archive, rkyv::Serialize, rkyv::Deserialize` with `#[rkyv(with = AsUnixTime)]` on `recorded_at`.

**Key interfaces:**
- `TemplateId` — mirror the `NoteId`/`SchemaId` pattern; use `UuidV7` as the inner type with `pub(crate)` visibility; `new()` generates a fresh V7 UUID, `parse()` parses a string, `as_uuid_v7()` returns `&UuidV7`; implement `Default` (delegates to `new()`), `Display` (delegates to inner), `From<UuidV7>`, `TryFrom<Uuid>`
- `TemplateName` — fallible constructor `try_new()` (per naming taxonomy: `try_` prefix for `Result`-returning) takes a `&Path` (path of the template file) and a `&Path` (template directory root); strips the root prefix then takes the path stem; `/` is permitted as separator in the resulting name string
- `TemplateBody` — constructor `try_new()` rejects empty strings → `TemplateBodyError::Empty`; provides `as_str() -> &str` and `AsRef<str>`
- `RawTemplate` — newtype `pub struct RawTemplate(String)` with `new(content: String) -> Self`, `AsRef<str>`, and `into_inner(self) -> String`. No PathKey, no validation beyond what String provides.
- `RawTemplateView` — flat struct with constructor `new(path, content_hash, metadata, recorded_at)`; accessors `path()`, `content_hash()`, `metadata()`, `recorded_at()`; implements `HasContentHash` and `HasContentHashMut` from the existing `support::content_hash` module
- `Template` — all fields private; constructor `new(id, path, name, body)` sets `recorded_at` internally; provide accessor methods; must be `#[non_exhaustive]`

**Acceptance criteria:**
- [ ] `TemplateId::new()` generates a fresh identifier; `TemplateId::parse()` round-trips through `as_uuid_v7()`
- [ ] `TemplateId` inner field is `pub(crate)`; derives `Default` + `Display` (delegating to inner `UuidV7`)
- [ ] `TemplateId` has `From<UuidV7>` and `TryFrom<Uuid>` conversions (matching `SchemaId`)
- [ ] `TemplateId` derives `rkyv::Archive + rkyv::Serialize + rkyv::Deserialize` with `#[rkyv(derive(Debug, Hash, PartialEq, Eq))]`
- [ ] `TemplateName` derives the correct stem for flat `.md` files (e.g. `standup` from `templates/standup.md`)
- [ ] `TemplateName` derives the correct subdirectory-qualified stem (e.g. `daily/standup` from `templates/daily/standup.md`)
- [ ] `TemplateName` returns `TemplateNameError::Derivation` when the path produces an empty stem
- [ ] `TemplateBody::try_new("")` returns `TemplateBodyError::Empty`; non-empty strings succeed
- [ ] `TemplateBody` makes no claim about Jinja syntax — no validation against a Jinja parser
- [ ] `RawTemplate` is constructible from `String` without additional validation; provides `AsRef<str>` and `into_inner()`
- [ ] `RawTemplateView` is a flat struct (NOT versioned ring buffer) holding `PathKey`, `Blake3Hash`, file metadata, and recorded time
- [ ] `RawTemplateView` correctly implements `HasContentHash` and `HasContentHashMut`
- [ ] `RawTemplateView` derives `rkyv::Archive + rkyv::Serialize + rkyv::Deserialize`; round-trip serialization test passes
- [ ] `Template` constructor `new(id, path, name, body)` sets `recorded_at` internally; `#[non_exhaustive]` and derives `rkyv::Archive + rkyv::Serialize + rkyv::Deserialize` with `#[rkyv(with = AsUnixTime)]` on `recorded_at`; round-trip serialization test passes
- [ ] `Template` all fields private with accessor methods: `id()`, `path()`, `name()`, `body()`, `recorded_at()`
- [ ] `TemplateNameError` is a single-variant enum (`Derivation`); `TemplateBodyError` is a single-variant enum (`Empty`); `TemplateError` embeds both via `#[from]`
- [ ] Newtypes implement `AsRef<InnerType>`: `TemplateId: AsRef<UuidV7>`, `TemplateName: AsRef<str>`, `TemplateBody: AsRef<str>`, `RawTemplate: AsRef<str>` — no `Deref`/`Borrow` in first pass
- [ ] No `minijinja` import appears anywhere in the `template` context module
- [ ] All types have doc comments on public items (enforced by `missing_docs = "deny"`)
- [ ] `mise run test` passes

**Out of scope:**
- Repository traits, processor, service, engine, CLI (issues 02–09)
- redb storage adapter implementation
- MiniJinja dependency or any Jinja syntax validation
- Frontmatter parsing or query semantics
- `TemplateExtension`, `ExtensionRegistry`, or any extension types

---

## TDD Implementation Plan

### File Layout

```
lithos-core/src/template/
├── mod.rs         → pub(crate) mod aggregate; pub(crate) mod error; pub(crate) mod raw; pub(crate) mod views;
│                    pub use aggregate::{Template, TemplateBody, TemplateId, TemplateName};
│                    pub use error::{TemplateBodyError, TemplateError, TemplateNameError};
│                    pub use raw::RawTemplate;
│                    pub use views::RawTemplateView;
├── aggregate.rs   → TemplateId, TemplateName, TemplateBody, Template
├── raw.rs         → RawTemplate(String)
├── views.rs       → RawTemplateView (flat struct)
└── error.rs       → TemplateNameError, TemplateBodyError, TemplateError
```

### Codebase Changes Required

| File                                    | Action                                                |
| --------------------------------------- | ----------------------------------------------------- |
| `lithos-core/src/lib.rs:28`               | `// pub mod template;` → `pub mod template;`              |
| `lithos-core/src/template/mod.rs`         | Create — submodule declarations + public re-exports       |
| `lithos-core/src/template/aggregate.rs`   | Create — `TemplateId`, `TemplateName`, `TemplateBody`, `Template` |
| `lithos-core/src/template/raw.rs`         | Create — `RawTemplate(String)`                              |
| `lithos-core/src/template/views.rs`       | Create — `RawTemplateView` (flat struct)                   |
| `lithos-core/src/template/error.rs`       | Create — `TemplateNameError`, `TemplateBodyError`, `TemplateError` |

### Tracer Bullet Order (Vertical Slices)

| #   | File         | Type            | Test                                                                                | Opens                                       |
| --- | ------------ | --------------- | ----------------------------------------------------------------------------------- | ------------------------------------------- |
| 1   | `aggregate.rs` | `TemplateId`      | `new()` + `as_uuid_v7()`                                                                | Module wiring, newtype pattern, derives     |
| 2   | `aggregate.rs` | `TemplateId`      | `parse()` + `Display` + `Default`                                                         | Std trait impls                             |
| 3   | `aggregate.rs` | `TemplateId`      | `From<UuidV7>` + `TryFrom<Uuid>` non-v7 reject + rkyv round-trip                        | Conversions, serialization                  |
| 4   | `aggregate.rs` | `TemplateName`    | `try_new(file, root)` — flat `.md` at root                                              | Core derivation logic                       |
| 5   | `aggregate.rs` | `TemplateName`    | `try_new()` — nested subdirectory (`daily/standup`)                                     | Subdirectory-qualified name                 |
| 6   | `aggregate.rs` | `TemplateName`    | empty stem → `TemplateNameError::Derivation`                                          | Domain validation boundary                  |
| 7   | `aggregate.rs` | `TemplateName`    | rkyv round-trip                                                                     | Aggregate storage proof                     |
| 8   | `aggregate.rs` | `TemplateBody`    | `try_new("valid")` + `as_str()` + `AsRef<str>`                                            | Simple wrapper, accessors                   |
| 9   | `aggregate.rs` | `TemplateBody`    | `try_new("")` → `TemplateBodyError::Empty`                                              | Validation invariant                        |
| 10  | `error.rs`     | error Display   | `TemplateNameError` Display + `TemplateBodyError` Display + `TemplateError` `#[from]` chain | Error ergonomics (tested via cycles 6,9,16) |
| 11  | `raw.rs`       | `RawTemplate`     | `new(content)` + `AsRef<str>` + `into_inner()`                                            | Thin DTO, no validation                     |
| 12  | `views.rs`     | `RawTemplateView` | `new(path, hash, metadata, recorded_at)` + accessors                                  | Flat struct shape                           |
| 13  | `views.rs`     | `RawTemplateView` | `HasContentHash` + `HasContentHashMut`                                                  | Trait impls                                 |
| 14  | `views.rs`     | `RawTemplateView` | rkyv round-trip                                                                     | Serialization                               |
| 15  | `aggregate.rs` | `Template`        | `new(id, path, name, body)` + all accessors + `recorded_at` set internally              | Aggregate constructor, accessors            |
| 16  | `aggregate.rs` | `Template`        | rkyv round-trip                                                                     | Full aggregate serialization, `AsUnixTime`    |
| 17  | `template/`    | Integration     | No `minijinja` imports anywhere in `template/`                                          | Policy enforcement                          |

### Test Suite Per File

**`aggregate.rs`:**

```
#[cfg(test)]
mod tests {
    mod template_id {
        mod constructor    // new(), parse(), as_uuid_v7()
        mod defaults       // Default
        mod formatting     // Display
        mod conversions    // From<UuidV7>, TryFrom<Uuid>
        mod serialization  // rkyv
    }
    mod template_name {
        mod constructor    // try_new() flat, nested
        mod validation     // empty stem → Derivation
        mod serialization  // rkyv
    }
    mod template_body {
        mod constructor    // try_new() valid, as_str(), AsRef
        mod validation     // empty → Empty
    }
    mod template {
        mod constructor    // new() sets recorded_at internally
        mod accessors      // id(), path(), name(), body(), recorded_at()
        mod serialization  // rkyv, AsUnixTime, #[non_exhaustive]
    }
}
```

**`raw.rs`:**

```
#[cfg(test)]
mod tests {
    mod raw_template {
        mod constructor    // new(String), AsRef<str>, into_inner()
    }
}
```

**`views.rs`:**

```
#[cfg(test)]
mod tests {
    mod raw_template_view {
        mod constructor           // new(), accessors
        mod has_content_hash      // trait impl
        mod has_content_hash_mut  // trait impl
        mod serialization         // rkyv
    }
}
```

**`error.rs`:**

```
#[cfg(test)]
mod tests {
    mod template_name_error  // Display format
    mod template_body_error  // Display format
    mod template_error       // #[from] conversion chain
}
```
