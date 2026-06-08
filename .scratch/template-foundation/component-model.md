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

Foundation shape:

```rust
#[derive(Debug, Clone, PartialEq, Archive, Serialize, Deserialize)]
#[non_exhaustive]
pub struct Template {
    id: TemplateId,
    path: PathKey,
    name: TemplateName,
    body: TemplateBody,
    #[rkyv(with = AsUnixTime)]
    recorded_at: SystemTime,
}
```

- `TemplateId(UuidV7)` — stable identity, matching `NoteId`, `SchemaId`, `FileId`.
- `path: PathKey` — vault-relative canonical location (not a separate newtype).
- `TemplateName` — validated field derived from path stem at construction time.
- `TemplateBody` — validated wrapper around the renderable template source string.
- `recorded_at` — private ingestion timestamp with `#[rkyv(with = AsUnixTime)]`, matching `Schema`.
- `#[non_exhaustive]` — foundation shape is intentionally extensible for future frontmatter/query fields.

Decisions:
- `Template` starts as the minimal validated aggregate needed to identify and render a template asset.
- The foundation shape is intentionally extensible; richer user-facing metadata can be added in later phases once frontmatter/query semantics are designed.
- Extension registry is not stored on the aggregate; registered into `TemplateEngine` at runtime.
- Frontmatter postponed to `template-query`.
- Both `TemplateName` and path are derived from `FilePath` at construction time.
- MiniJinja receives `(name, source)` pairs at the adapter boundary, never a filesystem path directly.

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

Foundation trait methods:

```rust
pub trait ReadRepository {
    fn find_template_by_id(&self, id: TemplateId)
        -> Result<Option<Template>, TemplateRepositoryError>;
    fn find_template_by_path(&self, path: &PathKey)
        -> Result<Option<Template>, TemplateRepositoryError>;
    fn list_templates(&self) -> Result<Vec<Template>, TemplateRepositoryError>;
    fn get_raw_template_view(&self, path: &PathKey)
        -> Result<Option<RawTemplateView>, TemplateRepositoryError>;
    fn get_raw_template_views(&self, paths: &[PathKey])
        -> Result<Vec<Option<RawTemplateView>>, TemplateRepositoryError>;
}

pub trait WriteRepository {
    fn save_template(&self, path: &PathKey, template: &Template)
        -> Result<(), TemplateRepositoryError>;
    fn save_many_templates(&self, entries: &[(PathKey, Template)])
        -> Result<(), TemplateRepositoryError>;
    fn delete_template(&self, id: TemplateId)
        -> Result<(), TemplateRepositoryError>;
    fn save_raw_template_view(&self, path: &PathKey, view: &RawTemplateView)
        -> Result<(), TemplateRepositoryError>;
    fn save_many_raw_template_views(&self, entries: &[(PathKey, RawTemplateView)])
        -> Result<(), TemplateRepositoryError>;
    fn delete_raw_template_view(&self, path: &PathKey)
        -> Result<(), TemplateRepositoryError>;
}
```

Method rationale:
- `get_raw_template_views` (batch) added for simple batch discovery, matching schema pattern.
- `save_many_raw_template_views` added for atomic batch cache updates.
- `save_template` takes `(path, template)` to atomically write template + path index.
- FS materialization is not a repository concern; the processor reads via FS adapters and persists via repository.

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
