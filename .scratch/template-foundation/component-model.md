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

Accepted foundation shape:
 - `Template` starts as the minimal validated aggregate needed to identify and render a template asset.
 - The foundation shape is intentionally extensible; richer user-facing metadata can be added in later phases once frontmatter/query semantics are designed.
 - `recorded_at: SystemTime` is required as private ingestion/storage metadata and should use `#[rkyv(with = AsUnixTime)]`, matching the existing schema aggregate convention.

Decisions:
- `TemplateName` is a validated field stored on `Template`.
- Both `TemplateName` and `TemplatePath` are derived from `FilePath` at construction time.
- No `TemplatePath` newtype — use `PathKey` directly for vault-relative path identity.
- `FilePath` (rooted, filesystem I/O) is an adapter-layer concern — never stored on the domain aggregate.
- MiniJinja receives `(name, source)` pairs at the adapter boundary, never a filesystem path directly.
- `Template` identity: stable `TemplateId` (UUIDv7) plus canonical `path: PathKey`.

Open shape questions:
 - Body: raw Markdown template body as `String`.
- Extension registry: not stored on the aggregate; registered into `TemplateEngine` at runtime.
- Frontmatter: postponed to `template-query`; foundation should not make frontmatter semantics central.

### `RawTemplate`

The raw file-backed DTO produced from reading a template file before parsing/validation.

Foundation shape:
- `RawTemplate { content: String }` — a thin named wrapper for pipeline clarity.
- No path (ingestion context assigns it), no metadata (on `RawTemplateView`).
- Processor takes `(content, PathKey, FileMetadata)` from FS adapter and produces `Template` directly.

### `RawTemplateView`

The lightweight cache view for freshness and staleness checks.

Foundation shape:
- No `TemplatePath` newtype — use `PathKey` directly.
- `FileMetadata` replaces separate `created_at`/`modified_at` fields.
- Single `Blake3Hash` for content identity (no `Blake3HashIndex` until frontmatter/query phase).
- Must implement `HasContentHash` and `HasContentHashMut` from `lithos-core/src/support/content_hash.rs`.
- Seam kept small for future vault filesystem indexer takeover.

```rust
#[derive(Debug, Clone, PartialEq, Archive, Serialize, Deserialize)]
pub struct RawTemplateView {
    path: PathKey,
    content_hash: Blake3Hash,
    metadata: FileMetadata,
    #[rkyv(with = AsUnixTime)]
    recorded_at: SystemTime,
}

impl HasContentHash for RawTemplateView { ... }
impl HasContentHashMut for RawTemplateView { ... }
```

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
