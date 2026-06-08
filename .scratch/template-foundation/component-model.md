# Template Foundation Component Model

This file is a working design surface for the foundation grilling session. It intentionally comes before the typestate processor design.

## Design Order

1. Domain models.
2. DTOs and raw views.
3. Repository traits.
4. Registry and extension modules.
5. Typestate processor.
6. Storage adapter tables and redb/rkyv details.

## Domain Models To Design

### `Template`

The primary domain aggregate for a renderable template asset.

Open shape questions:
- Identity: `TemplateId` only, or `TemplateId` plus stable path key?
- Name: derived from path stem, frontmatter, or explicit validated `TemplateName`?
- Body: raw Markdown template body as `String`.
- Extension registry: not stored on the aggregate; registered into `TemplateEngine` at runtime.
- Frontmatter: postponed to `template-query`; foundation should not make frontmatter semantics central.

### `RawTemplate`

The raw file-backed DTO produced from reading a template file before parsing/validation.

Open shape questions:
- Include full raw text as read from disk.
- Include path identity used by repositories and cache tables.
- Include filesystem metadata only if it remains replaceable by the future vault filesystem indexer.

### `RawTemplateView`

The lightweight cache view used for freshness and staleness checks.

Required constraints:
- Use `Blake3Hash` from `lithos-core/src/support/content_hash.rs` for content identity.
- Use `Blake3HashIndex` from `lithos-core/src/support/hash_index.rs` when the view carries per-section/per-key hashes.
- Implement or expose `HasContentHash`/`HasHashIndex` where appropriate if the design benefits from common freshness comparisons.
- Keep the freshness seam small because general file freshness will later move to the vault filesystem indexer.

Open shape questions:
- Single content hash only vs. a hash index keyed by logical template sections.
- Whether to store modified time now as a temporary optimization.
- Whether archived `RawTemplateView` must support direct comparison against owned `Blake3HashIndex` via `ArchivedBlake3HashIndex::is_match_by`.

## Repository Traits

Template repositories must follow ADR 016:

- `template::ReadRepository`: reads templates, raw views, and identity mappings.
- `template::WriteRepository`: saves templates, raw views, and identity mappings.
- `template::Repository`: marker trait extending read and write traits.

Open method groups:
- Template identity lookup by path.
- Raw template view read/write.
- Template aggregate read/write.
- Possibly separate file materialization operations from persistence operations to respect the global FS isolation invariant.

## Interaction Ports

Interaction ports are not template repository traits.

Planned location:
- `lithos-core/src/interact/` or `lithos-core/src/prompt/`.

Planned ports:
- `InputProvider` for text input.
- `SelectionProvider` for single and multi-select input.

Adapter decision:
- `lithos-cli` owns the `inquire` dependency and implements these ports.

## Foundation Extensions

The full planned extension registry is captured in `planned-extensions.md`.

Foundation includes:
- `file.*`
- `path.*`
- `date.*`
- `str.*`
- `num.*`

Later phases include:
- `prompt.*` in `template-user-interaction`.
- Query and frontmatter handling in `template-query`.
