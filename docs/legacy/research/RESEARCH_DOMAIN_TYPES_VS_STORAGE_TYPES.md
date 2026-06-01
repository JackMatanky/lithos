# Domain Types vs Storage Types in File-Based Rust Systems

## Research Report: When to Use Full DDD Aggregates vs Parsed/Storage Types

**Date:** March 11, 2026
**Focus:** Understanding the distinction between domain aggregates with behavior vs storage projections in file-based systems

---

## Executive Summary

After analyzing how successful Rust file-based systems (Cargo, rust-analyzer, Zola, mdBook, rustfmt) handle the boundary between domain logic and persistence, a clear pattern emerges:

**Key Finding:** Most Rust projects **do not separate domain types from storage types** at all. Instead, they use a single type that is both the domain model and the persistence model, with behavioral methods when needed.

The distinction between "domain aggregate" and "storage type" is primarily useful when:

1. **The domain shape is expensive to store** (needs normalization/flattening)
2. **Read queries need different shapes** than write operations (true CQRS scenarios)
3. **Complex domain invariants** require rich behavioral APIs that don't map to storage

For file-based systems where **files are the source of truth**, the pattern is:

- `Raw*` types for parsing (unvalidated, serde-only)
- **Single domain type** that is both valid entity AND storage shape
- Optional `*View` types ONLY when profiling reveals storage inefficiency

---

## 1. Cargo: Single Type, No Separation

### 1.1 The Package Type

**Source:** `cargo/core/package.rs`

```rust
/// Information about a package that is available somewhere in the file system.
///
/// A package is a `Cargo.toml` file plus all the files that are part of it.
#[derive(Clone, Debug)]
pub struct Package {
    inner: Rc<PackageInner>,
}

#[derive(Debug)]
struct PackageInner {
    /// The package's manifest
    manifest: Manifest,
    /// The root of the package
    manifest_path: PathBuf,
    /// Checksum for the package
    checksum: Option<String>,
}

impl Package {
    // BEHAVIORAL METHODS - This is a domain type with logic
    pub fn manifest(&self) -> &Manifest { &self.inner.manifest }

    pub fn dependencies(&self) -> &[Dependency] {
        self.manifest().dependencies()
    }

    pub fn targets(&self) -> &[Target] {
        self.manifest().targets()
    }

    // Domain logic: does this package publish to registries?
    pub fn publish(&self) -> &Option<Vec<String>> {
        self.manifest().publish()
    }

    // Validation logic IN the domain type
    pub fn verify(&self, config: &Config) -> CargoResult<()> {
        // Complex validation logic here
    }
}
```

**Key Observations:**

- **No separate "StoredPackage"** - `Package` is used everywhere
- **Behavioral methods** - `verify()`, `dependencies()`, etc.
- **Direct serialization** - The manifest is just stored as-is (TOML file)
- **Files are truth** - Package is reconstructed from filesystem on demand

### 1.2 The Manifest Type

```rust
/// Contains all the information about a package, as loaded from a `Cargo.toml`.
pub struct Manifest {
    summary: Summary,
    targets: Vec<Target>,
    warnings: Warnings,
    exclude: Vec<String>,
    include: Vec<String>,
    links: Option<String>,
    metadata: ManifestMetadata,
    custom_metadata: Option<toml::Value>,
    profiles: BTreeMap<InternedString, TomlProfile>,
    publish: Option<Vec<String>>,
    // ... etc
}

impl Manifest {
    // DOMAIN LOGIC in the "storage" type
    pub fn new(
        summary: Summary,
        targets: Vec<Target>,
        // ... many params
    ) -> Manifest {
        Manifest { /* ... */ }
    }

    // This is BOTH domain AND storage
    pub fn dependencies(&self) -> &[Dependency] {
        self.summary.dependencies()
    }
}
```

**Pattern:**

- Manifest is parsed from TOML (`Raw` layer)
- Manifest is validated during construction
- **Same type used for domain logic AND storage**
- No `StoredManifest` vs `Manifest` split

---

## 2. rust-analyzer: LSP Domain Types

### 2.1 The FileId Pattern

**Source:** `rust-analyzer/base-db/src/input.rs`

```rust
/// `FileId` is an integer which uniquely identifies a file.
/// File paths are messy and system-dependent, so rust-analyzer
/// uses `FileId` as a handle instead.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct FileId(pub u32);

/// The set of "source roots" (aka crates and other compilation units)
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct SourceRoot {
    /// Files in this source root
    pub(crate) files: FxHashMap<FileId, VfsPath>,
}

impl SourceRoot {
    // BEHAVIORAL method - domain logic
    pub fn new_local(files: FxHashMap<FileId, VfsPath>) -> SourceRoot {
        SourceRoot { files }
    }

    // QUERY method - also domain logic
    pub fn path_for_file(&self, file: &FileId) -> Option<&VfsPath> {
        self.files.get(file)
    }

    pub fn file_for_path(&self, path: &VfsPath) -> Option<FileId> {
        self.files.iter()
            .find_map(|(id, p)| if p == path { Some(*id) } else { None })
    }
}
```

**Key Observations:**

- **No storage layer separation** - `SourceRoot` is stored directly in salsa database
- **Behavioral methods** - `file_for_path()`, `path_for_file()`
- **Domain invariants** - `FileId` is a newtype, not a raw `u32`
- **Same type everywhere** - No `StoredSourceRoot` vs `SourceRoot`

### 2.2 The Crate Graph

```rust
/// A set of Rust crates, with dependencies between them.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct CrateGraph {
    arena: FxHashMap<CrateId, CrateData>,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct CrateData {
    pub root_file_id: FileId,
    pub edition: Edition,
    pub version: Option<String>,
    pub display_name: Option<CrateDisplayName>,
    pub cfg_options: CfgOptions,
    pub potential_cfg_options: CfgOptions,
    pub env: Env,
    pub dependencies: Vec<Dependency>,
    pub proc_macro: Vec<ProcMacro>,
}

impl CrateGraph {
    // DOMAIN BEHAVIOR - mutation logic
    pub fn add_crate_root(&mut self, file_id: FileId) -> CrateId {
        let id = CrateId(self.arena.len() as u32);
        let data = CrateData::default(file_id);
        self.arena.insert(id, data);
        id
    }

    // DOMAIN QUERY - graph traversal
    pub fn transitive_deps(&self, of: CrateId) -> impl Iterator<Item = CrateId> + '_ {
        // Complex graph traversal logic
    }
}
```

**Pattern:**

- `CrateGraph` has complex behavioral methods
- Still stored directly (via salsa)
- **No separation** between domain and storage
- Files reconstructed from VFS, graph is cache

---

## 3. Zola: Content Processing Types

### 3.1 The Page Type

**Source:** `zola/components/content/src/page.rs`

```rust
/// A parsed page in the site
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct Page {
    /// The front matter meta
    pub meta: PageFrontMatter,
    /// The actual content of the page, in markdown
    pub raw_content: String,
    /// The HTML rendered version of the page
    pub content: String,
    /// The path to this page on the filesystem
    pub file: FileInfo,
    /// The slug of that page.
    pub slug: String,
    /// The URL path of the page
    pub path: String,
    /// The full URL for that page
    pub permalink: String,
    /// The summary for the article, defaults to None
    pub summary: Option<String>,
    // ... many more fields
}

impl Page {
    /// Parse a page given the content of the .md file
    /// Files without front matter are considered draft
    pub fn parse(
        file_path: &Path,
        content: &str,
        config: &Config,
        base_path: &Path,
    ) -> Result<Page> {
        // 1. Parse front matter
        let (meta, content_offset) = PageFrontMatter::parse(&content)?;

        // 2. Extract raw content
        let raw_content = &content[content_offset..];

        // 3. Render markdown
        let rendered = render_markdown(raw_content, &context)?;

        // 4. Construct domain type
        Ok(Page {
            meta,
            raw_content: raw_content.to_string(),
            content: rendered.body,
            file: FileInfo::new_page(file_path, base_path),
            slug: meta.slug.clone().unwrap_or_else(|| slugify(&title)),
            // ... compute all fields
        })
    }

    /// Read and parse a .md file into a Page struct
    pub fn from_file<P: AsRef<Path>>(
        path: P,
        config: &Config,
        base_path: &Path,
    ) -> Result<Page> {
        let content = read_file(&path)?;
        Self::parse(path.as_ref(), &content, config, base_path)
    }
}
```

**Key Observations:**

- **Parsing returns domain type directly** - No `RawPage` → `Page` → `StoredPage`
- **Rich construction logic** - `parse()` and `from_file()` are smart constructors
- **Behavioral method** - `parse()` does complex transformation
- **Serialization for templates** - `#[derive(Serialize)]` for rendering, not storage
- **Files are truth** - Pages are rebuilt from markdown on every build

---

## 4. mdBook: Book and Chapter Types

### 4.1 The Book Structure

**Source:** `mdbook/src/book/mod.rs`

```rust
/// The representation of a book in memory.
pub struct Book {
    /// The sections in this book.
    pub sections: Vec<BookItem>,
}

/// An item in the book, either a chapter or a separator.
#[derive(Debug, Clone, PartialEq)]
pub enum BookItem {
    Chapter(Chapter),
    Separator,
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct Chapter {
    /// The chapter's name.
    pub name: String,
    /// The chapter's content.
    pub content: String,
    /// The chapter's section number, if it has one.
    pub number: Option<SectionNumber>,
    /// The location of the source file.
    pub path: Option<PathBuf>,
    /// The chapter's sub-items, if any.
    pub sub_items: Vec<BookItem>,
}

impl Book {
    /// Create a new book from a `book.toml` file.
    pub fn build_from_toml(src_dir: &Path, config: &Config) -> Result<Book> {
        // 1. Parse SUMMARY.md
        let summary = Summary::parse(src_dir)?;

        // 2. Load all chapters
        let mut book = Book::new();
        for item in summary.numbered_chapters() {
            let chapter = Chapter::load_from_disk(item, src_dir)?;
            book.sections.push(BookItem::Chapter(chapter));
        }

        Ok(book)
    }
}

impl Chapter {
    /// Load a chapter from a file
    fn load_from_disk(/* ... */) -> Result<Chapter> {
        let content = fs::read_to_string(&path)?;
        Ok(Chapter {
            name: name.to_string(),
            content,
            path: Some(path),
            // ...
        })
    }
}
```

**Key Observations:**

- **No storage layer** - `Book` is reconstructed from files every time
- **Domain logic in constructors** - `build_from_toml()`, `load_from_disk()`
- **Single type** - No `StoredChapter` vs `Chapter` distinction
- **Files are source of truth** - Memory representation is ephemeral

---

## 5. When Separation DOES Occur: The View Pattern

### 5.1 diesel: Queryable vs Insertable

**Source:** Diesel ORM patterns

```rust
// Domain type (what you work with in code)
#[derive(Debug, Clone)]
pub struct User {
    id: UserId,
    email: Email,      // Validated newtype
    created_at: DateTime<Utc>,
}

// Storage representation (what DB sees)
#[derive(Queryable)]
struct UserRow {
    id: i64,           // DB doesn't know about UserId
    email: String,     // DB doesn't know about Email newtype
    created_at: NaiveDateTime,
}

// Conversion between domain and storage
impl From<UserRow> for User {
    fn from(row: UserRow) -> Self {
        User {
            id: UserId(row.id),
            email: Email::from_trusted(row.email), // Already validated
            created_at: DateTime::from_utc(row.created_at, Utc),
        }
    }
}
```

**When this pattern is used:**

- Database schema differs from domain (SQL types vs Rust types)
- Need to normalize/denormalize (e.g., JSON columns)
- Different read shapes (joins, aggregates)

### 5.2 rkyv: Archived Types (Zero-Copy)

**Source:** `rkyv` documentation

```rust
#[derive(Archive, Serialize, Deserialize)]
pub struct Schema {
    name: String,
    properties: Vec<Property>,
}

// rkyv auto-generates ArchivedSchema
// pub struct ArchivedSchema {
//     name: ArchivedString,
//     properties: ArchivedVec<ArchivedProperty>,
// }

// Usage: zero-copy reads
storage.with_archived(id, |archived: &ArchivedSchema| {
    // Work with archived.name, archived.properties directly
    // No deserialization!
    archived.name.as_str()
})?;
```

**When this pattern is used:**

- Performance-critical reads (LSP, hot paths)
- Archived types are **automatic** (generated by rkyv)
- NOT a separate domain model - just a different memory layout

---

## 6. Decision Framework: Domain Type vs Storage Type

### 6.1 Single Type (Most Common)

**Use a single type that is BOTH domain and storage when:**

✅ **The domain shape matches storage needs**
✅ **Files are the source of truth** (can rebuild from files)
✅ **Domain logic is simple** (validation at construction, mostly queries)
✅ **No complex behavioral state machines**

**Examples:**

- Cargo's `Package` - stored as TOML, same type everywhere
- mdBook's `Chapter` - stored as markdown, same type everywhere
- Zola's `Page` - stored as markdown, same type everywhere

**Pattern:**

```rust
// Single type with validation at construction
pub struct Schema {
    name: SchemaName,      // Validated newtype
    properties: Vec<Property>,
}

impl Schema {
    /// Parse and validate from raw input
    pub fn parse(raw: RawSchema) -> Result<Self, SchemaError> {
        // Validation logic here
        Ok(Schema { /* ... */ })
    }

    /// Query method (domain behavior)
    pub fn find_property(&self, name: &str) -> Option<&Property> {
        self.properties.iter().find(|p| p.name() == name)
    }
}

// Same type used for:
// - Domain logic (find_property)
// - Storage (rkyv serialization)
// - API (feature-gated serde for JSON output)
```

### 6.2 Domain + Storage Split (Rare)

**Use separate domain and storage types when:**

❗ **Storage shape is SIGNIFICANTLY different** (normalization required)
❗ **Rich domain behavior** that doesn't map to storage (complex state machines)
❗ **Read queries need different projections** (reporting, analytics)
❗ **Profiling shows storage inefficiency** (too large, slow queries)

**Examples:**

- Event sourcing (events stored, aggregate reconstituted)
- ORM with complex joins (SQL row ≠ domain object)
- Analytics (denormalized read models)

**Pattern:**

```rust
// Domain aggregate (rich behavior)
pub struct Order {
    id: OrderId,
    items: Vec<OrderItem>,
    state: OrderState,  // State machine
}

impl Order {
    pub fn add_item(&mut self, item: OrderItem) -> Result<(), OrderError> {
        // Complex business rules
        if self.state != OrderState::Draft {
            return Err(OrderError::CannotModifySubmittedOrder);
        }
        self.items.push(item);
        Ok(())
    }

    pub fn submit(&mut self) -> Result<(), OrderError> {
        // State transition logic
        self.state = OrderState::Submitted;
        Ok(())
    }
}

// Storage projection (flattened for queries)
#[derive(Archive, Serialize, Deserialize)]
pub struct StoredOrder {
    id: OrderId,
    item_count: usize,    // Denormalized for queries
    total: Money,         // Computed field
    state: OrderState,
}

impl From<&Order> for StoredOrder {
    fn from(order: &Order) -> Self {
        StoredOrder {
            id: order.id,
            item_count: order.items.len(),
            total: order.compute_total(),
            state: order.state,
        }
    }
}
```

### 6.3 Archived Types (Generated)

**Use rkyv's Archived types when:**

✅ **Performance-critical reads** (LSP, real-time queries)
✅ **Domain type already correct** (just need zero-copy access)
✅ **Automatic generation** (rkyv does the work)

**NOT a manual separation** - just add closure-based access:

```rust
pub trait Repository {
    /// Zero-copy access via closure
    fn with_archived<F, R>(&self, id: &Id, f: F) -> Result<Option<R>>
    where
        F: for<'a> FnOnce(&'a ArchivedSchema) -> R;
}
```

---

## 7. File-Based Systems: The Lithos Pattern

### 7.1 Core Principle

**Files are the source of truth** → Database is a **rebuildable cache**

This fundamentally changes the domain/storage relationship:

| Traditional App           | File-Based System              |
| ------------------------- | ------------------------------ |
| DB is source of truth     | Files are source of truth      |
| Domain → persist to DB    | Files → parse → cache in DB    |
| Rich domain with behavior | Simple domain with validation  |
| State machines, workflows | Mostly queries + validation    |
| CQRS for scaling writes   | No CQRS (files control writes) |

### 7.2 Recommended Type Layers for Lithos

```text
┌─────────────────────────────────────────────────────────────┐
│ FILE (Source of Truth)                                      │
│ - schema.yaml, note.md, template.jinja2                     │
└─────────────────┬───────────────────────────────────────────┘
                  │ Read file (FileReader)
                  ▼
┌─────────────────────────────────────────────────────────────┐
│ RAW TYPE (Syntax Parse)                                     │
│ - RawSchema, RawNote, RawTemplate                           │
│ - Derives: serde::Deserialize ONLY                          │
│ - Purpose: Separate serde concerns from domain              │
└─────────────────┬───────────────────────────────────────────┘
                  │ TryFrom<Raw> (validation)
                  ▼
┌─────────────────────────────────────────────────────────────┐
│ DOMAIN TYPE (Validated Entity)                              │
│ - Schema, Note, Template                                    │
│ - Derives: rkyv::Archive (+ optional serde feature)         │
│ - Purpose: BOTH domain logic AND storage                    │
│ - Methods: Queries, computations, accessors                 │
│ - NO mutation (files are truth, not domain objects)         │
└─────────────────┬───────────────────────────────────────────┘
                  │ Repository::save()
                  ▼
┌─────────────────────────────────────────────────────────────┐
│ DATABASE (rkyv bytes)                                        │
│ - Stores ArchivedSchema, ArchivedNote, etc.                 │
│ - Zero-copy reads via with_archived()                       │
│ - Can be wiped and rebuilt from files                       │
└─────────────────────────────────────────────────────────────┘
                  │ Optional: Only if profiling shows need
                  ▼
┌─────────────────────────────────────────────────────────────┐
│ VIEW TYPE (Read Optimization) - RARE                        │
│ - SchemaView, NoteView (only if domain shape inefficient)   │
│ - Derives: rkyv::Archive                                    │
│ - Purpose: Denormalized for specific queries                │
└─────────────────────────────────────────────────────────────┘
```

### 7.3 Lithos Type Guidelines

**For each context (schema, note, template, config):**

1. **Always create: `Raw*` type**
   - Purpose: Parse file syntax (YAML/TOML/Markdown)
   - Derives: `serde::Deserialize`
   - No behavior (zero impl blocks except parsing helpers)

2. **Always create: Domain type** (e.g., `Schema`)
   - Purpose: Validated entity used for BOTH domain logic AND storage
   - Derives: `rkyv::Archive`, `rkyv::Serialize`, `rkyv::Deserialize`
   - Optional: Feature-gated `serde::Serialize` for CLI JSON output
   - Methods: Queries, accessors, computations
   - NO mutation methods (files control state)

3. **Rarely create: `*View` type**
   - Purpose: Storage optimization ONLY
   - When: Profiling shows domain shape is inefficient
   - Example: Schema with inheritance tree needs flattened properties for fast lookup

---

## 8. Case Studies: Applying to Lithos Contexts

### 8.1 Schema Context

**Domain Characteristics:**

- Complex: Inheritance, property resolution, $ref expansion
- Mostly queries: "What properties does this schema have?"
- State: Immutable after resolution (files control changes)

**Type Decision:**

```rust
// Raw type (serde parsing)
#[derive(Deserialize)]
pub struct RawSchema {
    pub name: Option<String>,
    pub parent: Option<String>,
    pub properties: Option<Vec<RawProperty>>,
    // All Option<T> for better error messages
}

// Domain type (validated + storage)
#[derive(Archive, Serialize, Deserialize)]
#[non_exhaustive]
pub struct Schema {
    id: SchemaId,
    name: SchemaName,      // Validated newtype
    parent_id: Option<SchemaId>,
    properties: Vec<Property>,  // Resolved (inheritance applied)
}

impl Schema {
    // Domain logic: query resolved properties
    pub fn find_property(&self, name: &PropertyName) -> Option<&Property> {
        self.properties.iter().find(|p| p.name() == name)
    }

    // Domain logic: check for property
    pub fn has_property(&self, name: &PropertyName) -> bool {
        self.find_property(name).is_some()
    }

    // NO mutation - files are source of truth
    // NO add_property(), remove_property(), etc.
}

// Optional: Only if profiling shows need
#[derive(Archive, Serialize, Deserialize)]
pub struct SchemaView {
    id: SchemaId,
    name: Box<str>,
    parent_id: Option<SchemaId>,
    property_names: Vec<Box<str>>,  // Flattened for fast lookup
}
```

**Recommendation:** **Single `Schema` type** (domain + storage). Only create `SchemaView` if LSP query performance requires it.

### 8.2 Note Context

**Domain Characteristics:**

- Simple: Mostly data extraction (frontmatter, links, tags)
- Queries: "What notes link to X?", "What tags exist?"
- State: Immutable (user edits markdown files)

**Type Decision:**

```rust
// Raw type (markdown parsing)
pub struct RawNote {
    frontmatter: Option<Frontmatter>,
    content: String,
    // Parsed by pulldown-cmark
}

// Domain type (validated + storage)
#[derive(Archive, Serialize, Deserialize)]
#[non_exhaustive]
pub struct Note {
    id: NoteId,
    path: NotePath,
    frontmatter: Option<Frontmatter>,
    links: Vec<Link>,
    tags: Vec<Tag>,
    headings: Vec<Heading>,
    // Extracted metadata
}

impl Note {
    // Domain queries
    pub fn outgoing_links(&self) -> &[Link] { &self.links }
    pub fn has_tag(&self, tag: &Tag) -> bool {
        self.tags.contains(tag)
    }

    // NO mutation - files are truth
}
```

**Recommendation:** **Single `Note` type**. No view needed (structure is already flat).

### 8.3 Template Context

**Domain Characteristics:**

- Behavioral: Rendering, input validation, block extraction
- Queries: "What inputs does this template need?"
- State: Templates may have lifecycle (validation → compilation)

**Type Decision:**

```rust
// Raw type (Jinja2 parsing)
#[derive(Deserialize)]
pub struct RawTemplate {
    name: Option<String>,
    inputs: Option<Vec<RawInput>>,
    // Template source is just a file path
}

// Domain type (validated + storage + behavior)
#[derive(Archive, Serialize, Deserialize)]
pub struct Template {
    id: TemplateId,
    name: TemplateName,
    inputs: Vec<InputSpec>,
    source_path: PathBuf,
}

impl Template {
    // Domain behavior: render
    pub fn render(&self, ctx: &Context) -> Result<String, TemplateError> {
        // This might be complex - loading, compiling, rendering
        let source = fs::read_to_string(&self.source_path)?;
        let env = Environment::new();
        let tmpl = env.template_from_str(&source)?;
        Ok(tmpl.render(ctx)?)
    }

    // Domain query
    pub fn required_inputs(&self) -> impl Iterator<Item = &InputSpec> {
        self.inputs.iter().filter(|i| i.is_required())
    }
}
```

**Recommendation:** **Single `Template` type** with behavioral methods. Template rendering is domain logic, not just data.

### 8.4 Config Context

**Domain Characteristics:**

- Simple: Validated configuration values
- Queries: "What's the vault path?", "What's the log level?"
- State: Immutable after load (user edits config files)

**Type Decision:**

```rust
// Raw type (TOML/YAML parsing)
#[derive(Deserialize)]
pub struct RawConfig {
    pub vault_path: Option<String>,
    pub log_level: Option<String>,
    // ...
}

// Domain type (validated + storage)
#[derive(Archive, Serialize, Deserialize)]
pub struct Config {
    vault_path: PathBuf,  // Validated to exist
    log_level: LogLevel,  // Parsed enum
    // ...
}

impl Config {
    // Simple accessors
    pub fn vault_path(&self) -> &Path { &self.vault_path }
    pub fn log_level(&self) -> LogLevel { self.log_level }
}
```

**Recommendation:** **Single `Config` type**. Very simple domain, no view needed.

---

## 9. Anti-Patterns to Avoid

### 9.1 Premature Separation

**Bad:**

```rust
// DON'T: Separate without reason
pub struct Schema { /* domain logic */ }
pub struct StoredSchema { /* storage */ }
pub struct SchemaView { /* queries */ }
pub struct SchemaAggregate { /* behavior */ }
```

**Good:**

```rust
// DO: Single type until profiling shows otherwise
pub struct Schema {
    // Fields for both domain and storage
}

impl Schema {
    // Methods for domain logic
}
```

### 9.2 Mutation in File-Based Systems

**Bad:**

```rust
// DON'T: Mutation methods when files are truth
impl Schema {
    pub fn add_property(&mut self, prop: Property) {
        self.properties.push(prop);
        // This is wrong - file hasn't changed!
    }
}
```

**Good:**

```rust
// DO: Files control mutation, domain is immutable
impl Schema {
    // Only query methods
    pub fn has_property(&self, name: &PropertyName) -> bool {
        self.properties.iter().any(|p| p.name() == name)
    }
}

// Mutation happens via file edit → reload pipeline
pub fn update_schema_file(path: &Path, edit: SchemaEdit) -> Result<()> {
    let raw = fs::read_to_string(path)?;
    let updated = apply_edit(&raw, edit)?;
    fs::write(path, updated)?;
    // System will reload from file
    Ok(())
}
```

### 9.3 Rich Domain Behavior (Wrong Context)

**Bad:**

```rust
// DON'T: Complex state machines when files are truth
impl Note {
    pub fn publish(&mut self) -> Result<(), NoteError> {
        self.state = NoteState::Published;
        // This is wrong - file controls publish state
    }
}
```

**Good:**

```rust
// DO: State is in file frontmatter
impl Note {
    pub fn is_published(&self) -> bool {
        self.frontmatter
            .as_ref()
            .and_then(|fm| fm.get("published"))
            .and_then(|v| v.as_bool())
            .unwrap_or(false)
    }
}
```

---

## 10. Recommendations for Lithos

### 10.1 Type Architecture Per Context

| Context      | Raw Type      | Domain Type | View Type          | Reasoning                                      |
| ------------ | ------------- | ----------- | ------------------ | ---------------------------------------------- |
| **schema**   | `RawSchema`   | `Schema`    | Maybe `SchemaView` | Complex resolution, may need flat view for LSP |
| **note**     | `RawNote`     | `Note`      | No                 | Already flat, simple queries                   |
| **template** | `RawTemplate` | `Template`  | No                 | Behavioral (render), but simple                |
| **config**   | `RawConfig`   | `Config`    | No                 | Very simple, no view needed                    |

### 10.2 Implementation Checklist

For each context, follow this pattern:

- [ ] **Create `Raw*` type** in `<context>/raw.rs`
  - Derives: `serde::Deserialize` only
  - All fields `Option<T>` for better error messages
  - Zero behavior (no impl blocks)

- [ ] **Create domain type** in `<context>/mod.rs` or `<context>/domain.rs`
  - Derives: `rkyv::Archive`, `rkyv::Serialize`, `rkyv::Deserialize`
  - Private fields (validated constructors only)
  - Methods: Queries, accessors, computations
  - NO mutation methods (files are truth)

- [ ] **Create `TryFrom<Raw*>`** for validation
  - This is the parsing boundary
  - Validate syntax (regex, format)
  - Validate semantics (refs exist, no cycles)
  - Return structured errors

- [ ] **Define Repository trait** in `<context>/mod.rs`
  - Single trait (not CQRS split)
  - Methods: `get`, `list`, `save`, `delete`, `with_archived`
  - Closure-based zero-copy access

- [ ] **Only create `*View` if profiling shows need**
  - Profile LSP query performance
  - If domain shape is inefficient, create denormalized view
  - Use Loader to project Domain → View

### 10.3 File Structure Per Context

```
<context>/
├── mod.rs              # Public API, domain type
├── raw.rs              # Raw* type (serde parsing)
├── loader.rs           # File → Raw → Domain → Storage pipeline
├── error.rs            # Context-specific errors
└── view.rs             # Optional: Only if profiling shows need
```

---

## 11. Conclusion

**The Rust file-based pattern is simple:**

1. **`Raw*` types** for parsing (unvalidated, serde-only)
2. **Domain types** that are BOTH validated entities AND storage shapes
3. **Optional `*View` types** ONLY when profiling reveals inefficiency

**For Lithos specifically:**

- **Drop the CQRS Query/Command split** - Use single Repository trait
- **Use single domain type per context** - Not Domain + Stored + View + Aggregate
- **Validation at construction** - TryFrom<Raw> boundary
- **No mutation methods** - Files are source of truth
- **Add `*View` only if needed** - Profile first, optimize later

The codebase currently has:

- ❌ `StoredSchema` (should just be `Schema`)
- ❌ `StoredNote` (should just be `Note`)
- ❌ CQRS Query/Command traits (should be single `Repository`)

**Next steps:**

1. Consolidate `Stored*` → domain types
2. Replace Query/Command traits with single `Repository` trait
3. Only introduce `*View` types if profiling shows storage inefficiency

---

## 12. References

### Real Projects Analyzed

- **Cargo** - `cargo/core/` (Package, Manifest)
- **rust-analyzer** - `base-db/src/input.rs` (FileId, SourceRoot, CrateGraph)
- **Zola** - `components/content/src/page.rs` (Page)
- **mdBook** - `src/book/mod.rs` (Book, Chapter)
- **diesel** - ORM patterns (Queryable vs Insertable)
- **rkyv** - Zero-copy archived types

### Key Articles

- matklad: "ARCHITECTURE.md" - Module boundaries over trait boundaries
- Pascal Hertleif: "Elegant Library APIs in Rust" - Traits for I/O, not business logic
- Yoshua Wuyts: "State Machines in Rust" - Type-state pattern

### Core Insight

> "Most successful Rust projects use **a single type** that serves as both domain model and storage model. Separation only occurs when there's a **measurable performance or complexity reason**."
