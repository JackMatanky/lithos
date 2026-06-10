# Template Domain Models

Status: ready-for-agent

## Parent

`.scratch/template-foundation/PRD.md`

## What to build

Define all Template domain value objects and aggregates that the rest of the Template context depends on. This is the foundational data layer — no behavior, no persistence, no engine.

Introduce:

- `TemplateId` — newtype over `UuidV7`, following the `NoteId`/`SchemaId` pattern. Derives `Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord` plus `rkyv::Archive, rkyv::Serialize, rkyv::Deserialize`.
- `TemplateName` — derived from the vault-relative path stem of the template file, qualified by subdirectory structure within the configured template directory (e.g. `templates/daily/standup.md` → `TemplateName("daily/standup")`). The `/` separator is permitted. Validated at construction from a file path, not a separate field.
- `TemplateBody` — wraps renderable source text. Structural invariants only: non-empty, valid UTF-8. Jinja syntax validity is explicitly NOT the domain's responsibility.
- `RawTemplate` — thin raw-content DTO carrying the file path and raw string content.
- `RawTemplateView` — freshness/cache view carrying `PathKey`, `Blake3Hash`, file metadata, and recorded time. Implements `HasContentHash` and `HasContentHashMut` from `support::content_hash`. Follows the versioned-history pattern of `RawSchemaView`, not a flat struct. Derives `rkyv::Archive, rkyv::Serialize, rkyv::Deserialize`.
- `Template` — primary renderable aggregate with `TemplateId`, `PathKey`, derived `TemplateName`, `TemplateBody`, and recorded ingestion time. Non-exhaustive (`#[non_exhaustive]`) for future frontmatter/query evolution. Derives `rkyv::Archive, rkyv::Serialize, rkyv::Deserialize`.

All domain types live in `lithos-core` under the `template` context. No MiniJinja types appear anywhere in this module.

## Acceptance criteria

- [ ] `TemplateId` is a newtype over `UuidV7` with `new()`, `parse()`, and `as_uuid_v7()` methods, matching the `NoteId`/`SchemaId` pattern
- [ ] `TemplateId` derives `rkyv::Archive + Serialize + Deserialize`
- [ ] `TemplateName` is constructed from a file path relative to the configured template directory, producing subdirectory-qualified stems (e.g. `daily/standup` for `templates/daily/standup.md`)
- [ ] `TemplateName` rejects paths that would produce an empty stem
- [ ] `TemplateBody` rejects empty strings at construction; valid UTF-8 is guaranteed by Rust's `String` type
- [ ] `TemplateBody` makes no claim about Jinja syntax validity
- [ ] `RawTemplate` is a thin DTO — no validation beyond what `PathKey` and `String` provide
- [ ] `RawTemplateView` holds `PathKey`, `Blake3Hash`, file metadata, and recorded time
- [ ] `RawTemplateView` implements `HasContentHash` and `HasContentHashMut`
- [ ] `RawTemplateView` derives `rkyv::Archive + Serialize + Deserialize`
- [ ] `Template` is `#[non_exhaustive]`
- [ ] `Template` derives `rkyv::Archive + Serialize + Deserialize`
- [ ] Unit tests cover: `TemplateId` construction and parse round-trip, `TemplateName` derivation (flat and subdirectory cases), `TemplateBody` empty rejection, `RawTemplateView` hash trait implementations, rkyv serialization round-trips for `Template` and `RawTemplateView`

## Blocked by

None — can start immediately.
