# Template Module Redesign

## 1. Executive Diagnosis

The current `@lithos-core/src/template/` implementation carries technical debt from an abandoned CQRS/event-sourced architecture and fights against `minijinja` rather than delegating to it.

**What is wrong:**
- The module duplicates native `minijinja` capabilities (inheritance, blocks) via domain abstractions (`TemplateBlock`, `BlockStrategy`), creating unnecessary complexity.
- It includes CQRS infrastructure (`command.rs`, `query.rs`, `events.rs`) which does not fit the new file-centric, deterministic asset model.
- It lacks a distinct boundary between declarative configuration (frontmatter) and imperative execution (Jinja engine).
- Schema dependencies are not explicitly tracked, making caching and deterministic rendering difficult.
- It does not support interactive operations (prompts, suggesters) which are crucial for the target workflow (similar to Obsidian Templater).

**What should remain:**
- Strong domain constraints on `TemplateName` and `InputName`.
- The intent to provide a clean, zero-copy structure (`rkyv`) for caching.

## 2. Design Principles

1. **File-Centric Source of Truth:** A template is defined entirely by its file. The frontmatter provides declarative static configuration; the body provides the rendering logic.
2. **Lean Engine Boundary:** The core domain (`Template`, `InputSpec`) knows nothing about `minijinja`. The template engine is an implementation detail hidden behind a clear adapter boundary.
3. **Markdown-First:** Templates generate Markdown. The engine must not auto-escape output, and the parser must preserve whitespace accurately.
4. **Strict Typestate Lifecycle:** The template loading process enforces a rigorous pipeline (`Discovery` -> `Comparison` -> `Parsed` -> `Construction`) modeled directly on `schema/property_bank_processor.rs` to guarantee that un-parseable or stale files cannot reach the rendering engine.
5. **Interactive & Query-Aware:** The runtime environment (`TemplateRuntime`) injected into the engine exposes first-class interactive methods (suggesters) and structured database queries.

## 3. High-Level Architecture

The module will be reorganized to enforce the separation of domain logic from the rendering engine:

```text
lithos-core/src/template/
├── mod.rs               # Public exports
├── aggregate.rs         # Template domain model, UUIDs, Name constraints
├── processor.rs         # Strict Typestate Pipeline (Discovery -> Completed)
├── parser.rs            # Uses pulldown-cmark to split Frontmatter/Body
├── engine/
│   ├── mod.rs
│   └── minijinja.rs     # The ONLY location minijinja is imported
├── runtime/
│   ├── mod.rs
│   ├── li.rs            # `TemplateRuntime` context object for Jinja
│   ├── interact.rs      # Abstractions for TUI/CLI prompts
│   └── query.rs         # Structured query builder (Dataview-style)
└── cache.rs             # Database table definitions for redb
```

## 4. Typestate Pipeline

A rigid typestate pipeline orchestrates ingestion, ensuring we only pay the cost of parsing when file contents actually change.

### The Flow
- **Discovery**: Is there a cached `RawTemplateView`?
- **Comparison**: Does the file's `mtime` match the view? If no, hash the content. Does the hash match?
- **Parsed**: (Triggered only on hash mismatch or new file) Split frontmatter and body.
- **Refresh**: (Triggered on timestamp-only or content-only mismatches) Update cache metadata without full rebuild.
- **Construction**: Build the final `Template` aggregate (or fetch the existing one) and commit to cache.

### Rust Sketches

```rust
pub struct TemplateProcessor<Stage, Status> {
    status: Status,
    _stage: PhantomData<Stage>,
}

// Stages
pub struct Discovery;
pub struct Comparison;
pub struct Parsed;
pub struct Refresh;
pub struct Construction;
pub struct Completed;

// Statuses
pub struct Unknown;
pub struct Missing { info: FileInfo }
pub struct Present { info: FileInfo, view: RawTemplateView }
pub struct Suspect { info: FileInfo, view: RawTemplateView, content: String }
pub struct Stale { info: FileInfo, content: String, content_hash: Blake3Hash, view: RawTemplateView }
// ... (Refresh/Construction statuses: StaleTimestamps, StaleContent, New, Changed, Fresh)

// Result of a successful pipeline run:
pub struct CompletedTemplate {
    pub template: Template,
}
```

## 5. Data Model

The domain types use `rkyv` for zero-copy caching in `redb`.

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Archive, Serialize, Deserialize)]
pub struct TemplateId(UuidV7);

/// The terminal aggregate produced by the pipeline.
#[derive(Debug, Clone, Archive, Serialize, Deserialize)]
pub struct Template {
    pub id: TemplateId,
    pub name: TemplateName,
    pub schemas: Vec<SchemaName>, // Declared dependencies
    pub inputs: HashMap<InputName, InputSpec>,
    pub body: String,
}

/// Extracted via pulldown-cmark in the `Parsed` stage.
#[derive(serde::Deserialize)]
pub struct TemplateFrontmatter {
    #[serde(default)]
    pub schemas: Vec<SchemaName>,
    #[serde(default)]
    pub inputs: HashMap<InputName, InputSpec>,
}

#[derive(Debug, Clone, Archive, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum InputSpec {
    Text { default: Option<String>, required: bool },
    Select { options: Vec<String>, required: bool },
}
```

## 6. Query System Design

Queries are modeled as structured builders, similar to Obsidian Dataview or SQL. This avoids passing raw SQL strings to the engine, allowing the Rust backend to securely optimize and execute the query against `redb`.

```rust
/// Engine-agnostic query representation.
#[derive(Debug, Clone)]
pub struct QueryBuilder {
    pub schema: Option<SchemaName>,
    pub filters: Vec<QueryFilter>,
}

#[derive(Debug, Clone)]
pub enum QueryFilter {
    Equals(String, String),
    Contains(String, String),
}
```

## 7. The `TemplateRuntime` API

The runtime object (`li` variable in templates) bridges declarative definitions with imperative execution.

```rust
pub struct TemplateRuntime {
    vault: Arc<dyn VaultAccess>,
    interact: Arc<dyn InteractiveHost>,
}

// Injected into minijinja environment:
impl minijinja::Object for TemplateRuntime {
    // Expose: li.query("project").where("status", "active")
    fn query(&self, schema: &str) -> QueryBuilder { ... }

    // Expose: li.suggester(options)
    fn suggester(&self, options: minijinja::Value) -> Result<String, Error> {
        self.interact.prompt_suggester(...)
    }
}
```

## 8. Caching Strategy (`redb`)

To support the typestate pipeline, we define specific tables to store identity, cache views, and the compiled aggregate.

```rust
pub(crate) mod db_table {
    use redb::TableDefinition;

    // Maps path -> serialized RawTemplateView (for mtime/hash checks)
    pub(crate) const RAW_TEMPLATE_VIEWS: TableDefinition<&str, &[u8]> = TableDefinition::new("raw_template_views");

    // Maps path -> TemplateId
    pub(crate) const TEMPLATE_ID_BY_PATH: TableDefinition<&str, &[u8]> = TableDefinition::new("template_id_by_path");

    // Maps TemplateId -> serialized Template domain object
    pub(crate) const TEMPLATES: TableDefinition<&[u8], &[u8]> = TableDefinition::new("templates");
}
```
*Note: We do not cache the compiled `minijinja` AST on disk. It compiles fast enough from strings to be held in a thread-local or Mutex during a CLI session.*

## 9. Template Parsing Strategy

To decouple the template from the `note` parser without blocking progress on the `parser` refactor, the `template` module will temporarily use its own small `pulldown-cmark` based extractor to split frontmatter and body.

This perfectly isolates the `Template` domain while remaining compatible with future `fs/parser` extractions.

## 10. Migration Strategy

1. **Delete Outdated Abstractions:** Remove `command.rs`, `query.rs`, `events.rs`, `block.rs`.
2. **Implement Data Model:** Create the clean `Template`, `TemplateFrontmatter`, and `InputSpec` structures.
3. **Implement Caching/Processor:** Build the `cache.rs` tables and the `processor.rs` typestate pipeline.
4. **Implement Parsing:** Add a `parser.rs` inside `template` using `pulldown-cmark` to split the file.
5. **Implement Engine:** Build the `TemplateRuntime` and the `minijinja` engine adapter.
