---
title: "Implementation Patterns & Consistency Rules"
description: "Development patterns, naming conventions, and consistency rules for Lithos implementation"
author: "Jack"
date: "2026-01-23"
last_updated: "2026-02-15"
section: "Implementation Standards"
---

# Implementation Patterns & Consistency Rules

## Pattern Categories Defined

**Critical Conflict Points Identified:** 30+ areas where AI agents could make different choices in async Rust CLI applications with modular boundaries, functional pipelines, and type-driven design.

## Naming Patterns

**Rust Naming Conventions:**

- **Crate/Package Names:** Cargo _package_ names are `kebab-case` (e.g., `lithos-core`, `lithos-cli`). In Rust code, the crate import path is `snake_case` (e.g., `lithos_core`).
- **Modules & Files:** `snake_case` (e.g., `vault_indexer.rs`, `frontmatter_service.rs`)
- **Functions & Variables:** `snake_case` (e.g., `execute_template`, `vault_path`)
- **Structs & Enums:** `PascalCase` (e.g., `Note`, `DomainError`, `TemplateEngine`)
- **Traits:** `PascalCase` ending with trait name (e.g., `CacheWriter`, `VaultReader`) or `Port` (e.g., `StoragePort`)
- **Constants:** `SCREAMING_SNAKE_CASE` (e.g., `MAX_VAULT_SIZE`, `DEFAULT_TIMEOUT`)
- **Test Functions:** `snake_case` with descriptive names that read like a sentence (e.g., `returns_blob_when_larger_than_b`). Avoid generic `test_*` prefixes; use `mod <unit_under_test>` to group related tests when helpful.
- **Macros:** `snake_case` (e.g., `my_macro!`)

**Naming Discipline (Semantic Consistency):**

- **Say what it means:** Prefer names that make roles and direction obvious (e.g., `needle`/`haystack`, `source`/`destination`, `before`/`after`).
- **Consistent word order:** Pick a project-wide pattern and stick to it. If most functions read as `verb_noun`, new APIs should follow `verb_noun` unless there is a strong reason not to.
- **Be concise (but not cryptic):** Avoid nonstandard abbreviations and "inside jokes"; shorten only when meaning remains obvious.
- **Use simple, correct words:** Prefer the smallest set of small words that preserve meaning; avoid terms that can be read two ways.
- **Unify concept names:** One term per concept. If we choose "vault" (not "repo"/"workspace") for the user's note collection, use "vault" consistently in APIs, docs, and modules.
- **Avoid type-noise in names:** Don't encode types (e.g., `*_str`, `*_vec`) unless it disambiguates two values with the same conceptual meaning.

**Pattern-Match Variable Naming:**

- When destructuring, keep field names whenever possible, and avoid renaming to single letters.
- Use struct field shorthand to preserve the domain vocabulary.

✅ Prefer:

```rust
if let Some(response) = response { /* ... */ }
let Self { name, path } = self;
match state {
    State::Reading(file) => { /* ... */ }
    State::Evaluating { workload, .. } => { /* ... */ }
}
```

⚠️ Avoid:

```rust
if let Some(r) = response { /* ... */ }
let Self { name: some_name, path: name } = self;
match state {
    State::Reading(data_source) => { /* ... */ }
    State::Evaluating { workload: to_eval, .. } => { /* ... */ }
}
```

**Pattern Matching Discipline:**

- **Match exhaustively to draw attention:** Prefer destructuring structs/enums to make it obvious which fields are considered. This helps the compiler alert us when structures evolve.
- **Don't pattern-match references:** Prefer explicit dereferencing (`|x| *x`) over `|&x| x`.
- **Avoid numeric tuple indexing:** Prefer destructuring into named values (`let (x, y) = point`) over `.0`/`.1`.
- **Avoid pattern-matching in `fn` parameters:** Unpack on the first line inside the function; keep signatures clean.

✅ Prefer:

```rust
// Exhaustive destructuring (future-proofing)
let Self { name, path, .. } = self;

// Explicit deref, not matching references
let values: Vec<_> = refs.iter().map(|x| *x).collect();

// Name tuple elements
let (x1, y1) = point1;
let (x2, y2) = point2;

// Keep function signature clean
fn new(config: ServerConfig) {
    let ServerConfig { db_path, working_path } = config;
    // ...
}
```

⚠️ Avoid:

```rust
// Implicit access hides evolution of the type
let (name, path) = (&self.name, &self.path);

// Reference pattern matching obscures deref
let values: Vec<_> = refs.iter().map(|&x| x).collect();

// Tuple indexing loses semantics
let gradient = (point2.1 - point1.1) / (point2.0 - point1.0);

// Pattern matching in parameters adds noise to signatures
fn new(ServerConfig { db_path, working_path }: ServerConfig) {
    // ...
}
```

**Generic Type Parameter Naming:**

- Generic type parameters should be single-letter to avoid looking like concrete types (common: `T`, `E`, `K`, `V`).

**Lifetime Parameter Naming:**

- Use lifetimes as "documentation": pick names derived from what is being borrowed (e.g., `'db`, `'tx`, `'bytes`, `'src`).
- Avoid `'a`/`'b` unless there is a compelling reason; avoid numbers in lifetime names.

✅ Prefer:

```rust
pub struct ArchivedGuard<'db, T> { /* ... */ }
pub struct SchemaView<'bytes> { /* ... */ }
```

⚠️ Avoid:

```rust
pub struct ArchivedGuard<'a, T> { /* ... */ }
pub struct SchemaView<'a> { /* ... */ }
```

**API Contract Naming:**

- **Trait Methods:** `snake_case` with clear action verbs (e.g., `persist_note`, `find_templates`)
- **Storage Traits:** Descriptive names ending with `Storage` (e.g., `CacheStorage`, `NoteStorage`)
- **DTO Structs:** Prefer role-based names over type-suffixes (e.g., `VaultFile`, `VaultFileRecord`, `CreateNoteRequest`). If `Dto` is used, reserve it for strict boundary/wire types (CLI/adapters/serde), not domain/core.
- **View Types:** Use the `*View` suffix for read-optimized database projections (e.g., `SchemaView`, `NoteView`).

**Builder Naming:**

- If a builder for `MyType` is provided, expose `MyType::builder() -> MyTypeBuilder` and `MyTypeBuilder::build() -> Result<MyType, _>`.
- Builder setters should read naturally and match field names where possible.

## Type-Driven Design Patterns

**Core Principle:** Make illegal states unrepresentable. Use Rust's type system to enforce invariants at compile time rather than runtime checks.

### API Design & Ownership Patterns

**Argument Ownership:**

- **Prefer borrowed arguments:** Take `&str`, `&Path`, `&[T]` instead of `String`, `PathBuf`, `Vec<T>`.
- **Take ownership only when needed:** If you need to store the data, take `T` or `impl Into<T>`.
- **Use `impl Trait` for inputs:** `fn process(input: impl Read)` is more flexible than `fn process(input: &mut File)`.

**String Efficiency:**

- **Zero-copy where possible:** Use `&str` for read-only text.
- **`Box<str>` for immutable owned:** Use `Box<str>` instead of `String` for immutable text fields (saves 8 bytes per string).
- **`Cow<'a, str>` for mixed:** Use `Cow` when data might be borrowed or owned.

**Construction Conventions:**

- **`new()`:** Infallible constructor.
- **`try_new()`:** Fallible constructor returning `Result`.
- **`with_capacity()`:** Pre-allocation for collections.
- **`from_*()`:** Conversion constructors.

### Validation Through Construction

**Pattern:** Hide direct field access, require validation at construction.

✅ **Prefer:**

```rust
/// A validated schema name (non-empty, no path separators).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SchemaName(String);  // Private field

impl SchemaName {
    /// Creates a new schema name after validation.
    ///
    /// # Errors
    /// Returns error if name is empty or contains path separators.
    pub fn new(name: impl Into<String>) -> Result<Self, ValidationError> {
        let name = name.into();

        if name.is_empty() {
            return Err(ValidationError::EmptyName);
        }

        if name.contains('/') || name.contains('\\') {
            return Err(ValidationError::PathSeparatorInName);
        }

        Ok(Self(name))
    }

    /// Returns the name as a string slice.
    pub fn as_str(&self) -> &str {
        &self.0
    }

    /// Consumes the SchemaName and returns the inner String.
    pub fn into_inner(self) -> String {
        self.0
    }
}

// Implement useful traits
impl AsRef<str> for SchemaName {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

impl std::fmt::Display for SchemaName {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}
```

❌ **Avoid:**

```rust
// Public fields allow bypassing validation
pub struct SchemaName {
    pub name: String,  // Anyone can set to invalid value!
}

impl SchemaName {
    pub fn new(name: String) -> Result<Self, ValidationError> {
        if name.is_empty() {
            return Err(ValidationError::EmptyName);
        }
        Ok(Self { name })
    }
}

// Caller can bypass validation:
let mut schema_name = SchemaName::new("valid".to_string())?;
schema_name.name = "".to_string();  // Now invalid! Type system can't prevent this.
```

### Visibility Control Strategy

**Default to Private:** Use the most restrictive visibility that works, only widen when necessary.

**Visibility Hierarchy:**

1. **Private (default):** `field: Type` - Only accessible within the defining module
2. **Crate-Internal:** `pub(crate) field: Type` - Accessible within lithos-core
3. **Parent Module:** `pub(super) field: Type` - Accessible in parent module only
4. **Public:** `pub field: Type` - Part of external API (use sparingly)

**Guidelines:**

```rust
// Domain aggregate
pub struct Note {
    // ✅ Public ID (needed for external queries)
    pub id: Uuid,

    // ✅ Controlled access through newtype
    pub path: NotePath,  // NotePath validates on construction

    // ❌ DON'T expose mutable collections directly
    // pub links: Vec<Link>,  // Caller could push invalid links

    // ✅ DO hide implementation details
    links: Vec<Link>,  // Private - use accessor methods

    // ✅ Expose as iterator, not mutable reference
    pub fn links(&self) -> impl Iterator<Item = &Link> {
        self.links.iter()
    }

    // ✅ Controlled mutation through method
    pub fn add_link(&mut self, link: Link) -> Result<(), NoteError> {
        // Can validate before adding
        if link.target().is_empty() {
            return Err(NoteError::InvalidLinkTarget);
        }
        self.links.push(link);
        Ok(())
    }
}
```

### Accessor Pattern over Direct Access

**Pattern:** Expose data through methods, not public fields.

✅ **Prefer:**

```rust
pub struct Config {
    vault_path: PathBuf,     // Private
    max_cache_size: usize,   // Private
}

impl Config {
    // Read-only access
    pub fn vault_path(&self) -> &Path {
        &self.vault_path
    }

    pub fn max_cache_size(&self) -> usize {
        self.max_cache_size
    }

    // Controlled mutation
    pub fn set_max_cache_size(&mut self, size: usize) -> Result<(), ConfigError> {
        if size == 0 {
            return Err(ConfigError::InvalidCacheSize);
        }
        self.max_cache_size = size;
        Ok(())
    }
}
```

❌ **Avoid:**

```rust
pub struct Config {
    pub vault_path: PathBuf,     // Can be set to anything
    pub max_cache_size: usize,   // Can be set to 0
}
```

**Benefits:**

- Can add validation to setters later without breaking API
- Can change internal representation without breaking callers
- Clear ownership semantics (reference vs owned)

### Newtype Pattern for Domain Constraints

**Pattern:** Wrap primitive types to encode domain constraints.

✅ **Examples:**

```rust
/// A note ID (UUID v7, time-ordered).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct NoteId(Uuid);

impl NoteId {
    /// Creates a new time-ordered note ID.
    pub fn new() -> Self {
        Self(Uuid::now_v7())
    }

    /// Parses a note ID from a string.
    pub fn parse(s: &str) -> Result<Self, ParseError> {
        let uuid = Uuid::parse_str(s)?;
        Ok(Self(uuid))
    }

    pub fn as_uuid(&self) -> &Uuid {
        &self.0
    }
}

/// A positive, non-zero count.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Count(NonZeroUsize);

impl Count {
    pub fn new(value: usize) -> Result<Self, ValidationError> {
        NonZeroUsize::new(value)
            .map(Self)
            .ok_or(ValidationError::ZeroCount)
    }

    pub fn get(&self) -> usize {
        self.0.get()
    }
}

/// A vault-relative path (validated, no traversal).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VaultPath(PathBuf);

impl VaultPath {
    pub fn new(path: impl AsRef<Path>) -> Result<Self, PathError> {
        let path = path.as_ref();

        if path.is_absolute() {
            return Err(PathError::AbsolutePath);
        }

        if path.components().any(|c| c == Component::ParentDir) {
            return Err(PathError::PathTraversal);
        }

        Ok(Self(path.to_path_buf()))
    }

    pub fn as_path(&self) -> &Path {
        &self.0
    }
}
```

### Non-Exhaustive Types for Evolution

**Pattern:** Mark types as `#[non_exhaustive]` when they might grow.

✅ **Use for:**

```rust
// Public enums that might gain variants
#[non_exhaustive]
#[derive(Debug, Clone, PartialEq)]
pub enum LinkStyle {
    WikiLink,
    Markdown,
    Embed,
    // Future: Audio, Video, etc.
}

// Public structs with optional fields
#[non_exhaustive]
#[derive(Debug, Clone)]
pub struct TemplateContext {
    pub title: String,
    pub date: DateTime<Utc>,
    // Can add fields without breaking existing code
}
```

**Benefits:**

- Prevents external code from exhaustive matching (forces wildcard)
- Prevents external code from direct struct construction
- Allows adding variants/fields in minor versions

### Builder Pattern for Complex Construction

**When to use:** Types with many optional fields or complex validation.

✅ **Pattern:**

```rust
#[derive(Debug, Clone)]
pub struct Schema {
    name: SchemaName,
    properties: Vec<Property>,
    extends: Option<SchemaName>,
    version: u32,
}

pub struct SchemaBuilder {
    name: Option<SchemaName>,
    properties: Vec<Property>,
    extends: Option<SchemaName>,
    version: u32,
}

impl Schema {
    pub fn builder() -> SchemaBuilder {
        SchemaBuilder {
            name: None,
            properties: Vec::new(),
            extends: None,
            version: 1,
        }
    }
}

impl SchemaBuilder {
    pub fn name(mut self, name: SchemaName) -> Self {
        self.name = Some(name);
        self
    }

    pub fn add_property(mut self, property: Property) -> Self {
        self.properties.push(property);
        self
    }

    pub fn extends(mut self, parent: SchemaName) -> Self {
        self.extends = Some(parent);
        self
    }

    pub fn version(mut self, version: u32) -> Self {
        self.version = version;
        self
    }

    pub fn build(self) -> Result<Schema, BuildError> {
        let name = self.name.ok_or(BuildError::MissingName)?;

        if self.properties.is_empty() {
            return Err(BuildError::NoProperties);
        }

        Ok(Schema {
            name,
            properties: self.properties,
            extends: self.extends,
            version: self.version,
        })
    }
}
```

### Summary: Type Safety Checklist

When designing a type, ask:

- [ ] Are all fields private unless truly needed public?
- [ ] Do I use newtypes for domain constraints (IDs, validated strings)?
- [ ] Do I provide constructors that enforce invariants?
- [ ] Do I expose accessors instead of direct field access?
- [ ] Is the type `#[non_exhaustive]` if it might evolve?
- [ ] Do I use builder pattern for complex optional construction?
- [ ] Do validation errors have helpful messages?
- [ ] Can I make illegal states unrepresentable?

**Anti-Patterns to Avoid:**

❌ Public mutable fields on domain aggregates
❌ Using `String`/`usize`/`PathBuf` directly without newtype wrappers
❌ Exposing `Vec<T>` directly (use iterators or controlled mutation)
❌ Validation in functions instead of type constructors
❌ `pub struct` with all `pub` fields as default

**Quick Wins:**

✅ Mark all fields private by default
✅ Wrap validated strings in newtypes (SchemaName, NotePath)
✅ Use NonZero* types from std for positive numbers
✅ Expose collections via `&[T]` or iterators, not `&mut Vec<T>`
✅ Use `#[non_exhaustive]` on public enums

## Module & Storage Patterns

**Core Principle:** Reject CQRS and event sourcing in favor of module boundaries, Iterator-based ingestion pipelines, and simple `Storage` trait abstractions for I/O.

### Module Isolation over Trait Isolation

**Pattern:** Business contexts (note, schema, template) isolate business logic through Rust modules. Traits are reserved strictly for abstracting I/O, not for separating domain services.

**Why Module Boundaries:**
- Simpler dependency management.
- Prevents god-objects through strict ownership and visibility rules.
- Aligns with idiomatic Rust module structure.

### Unified Storage Trait Pattern

**Pattern:** Define a single, simple `Storage` trait per domain module for all persistence operations.

✅ **Prefer:**

```rust
pub trait SchemaStorage {
    type Error: std::error::Error;

    fn get(&self, name: &SchemaName) -> Result<Option<Schema>, Self::Error>;
    fn save(&self, schema: &Schema) -> Result<(), Self::Error>;
    fn list(&self) -> Result<Vec<Schema>, Self::Error>;
}
```

❌ **Avoid:**

```rust
// Split CQRS ports that create interface bloat
pub trait SchemaQueryPort {
    // ...
}
pub trait SchemaCommandPort {
    // ...
}
```

### Functional Composition Pipeline

**Pattern:** Use functional composition and Iterator pipelines for processing data, rather than complex Command/Event routing.

✅ **Prefer:**

```text
File → parse (Raw) → validate (Domain) → project (Storage)
```

Data transitions through discrete stages:
1. `parse()`: File contents to unvalidated `Raw*` struct.
2. `validate()`: `TryFrom<Raw*>` to validated `Domain` struct.
3. `project()`: Persist to database via `Storage` trait.

## Serialization & Storage Patterns

**Serialization Patterns:**

- **Feature Flag:** Use `#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]`.
- **Optional:** Serde is optional for domain types, required for DTOs/Config.
- **Zero-Copy:** Use `rkyv` for performance-critical storage types.

### Three-Shape Serialization Pattern

**Core Principle:** Separate concerns of parsing, validation, and storage optimization through three distinct type layers.

**Architecture Flow:**

```text
External Input (TOML/YAML/JSON/User Input)
    ↓ serde::Deserialize
Raw* Types (unvalidated, optional fields)
    ↓ TryFrom<Raw*> (VALIDATION BOUNDARY)
Domain Types (validated invariants, typed enums)
    ↓ rkyv::Archive (if persisted)
Database (redb, zero-copy bytes)
```

```text
┌─────────────────────────────────────────┐
│ File System (YAML/JSON)                 │
│ - User-editable vault files             │
└─────────────────┬───────────────────────┘
                  │
                  ▼ parse (serde)
┌─────────────────────────────────────────┐
│ Raw* (serde derives)                    │
│ - Unvalidated input representation      │
│ - Location: <context>/raw.rs            │
│ - Nullable fields for better errors     │
└─────────────────┬───────────────────────┘
                  │
                  ▼ validate & compile
┌─────────────────────────────────────────┐
│ Domain (rkyv + serde feature-gated)     │
│ - Validated, invariant-preserving       │
│ - Location: <context>/aggregate.rs      │
│ - Used throughout application           │
│ - Has rkyv derives for zero-copy DB     │
└─────────────────┬───────────────────────┘
                  │
                  ▼ project/adapt (optional, only when needed)
┌─────────────────────────────────────────┐
│ *View (rkyv derives, optional)          │
│ - Read-optimized cache representation   │
│ - Location: <context>/view.rs           │
│ - Only when queries need projection     │
└─────────────────┬───────────────────────┘
                  │
                  ▼ serialize (rkyv)
┌─────────────────────────────────────────┐
│ Database (redb)                         │
│ - Zero-copy archived access             │
└─────────────────────────────────────────┘
```

---

#### Shape 1: Raw* Types (Parsing Boundary)

**Purpose:** Accept flexible, unvalidated input from external sources (files, API requests, CLI args).

**Characteristics:**

- **Zero methods** - no `impl` blocks except `Derive`
- **All fields `Option<T>`** - missing fields are `None`
- **String enums** - accept any string, validation happens in `TryFrom`
- **Public fields** - deserialization requires pub access
- **Not persisted** - never stored in database
- **Location:** `<context>/raw.rs` or co-located with domain type

**Example:**

```rust
/// Raw frontmatter configuration (unvalidated).
#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
#[non_exhaustive]
pub struct RawFrontmatter {
    pub alias_key: Option<String>,        // Could be empty ""
    pub title_key: Option<String>,        // Could be empty ""
    pub date_created_key: Option<String>, // Could be empty ""
    // NO impl blocks, NO methods
}
```

**What NOT to do:**

```rust
impl RawFrontmatter {
    // ❌ DON'T: No validation methods on Raw types
    pub fn validate(&self) -> Result<(), ConfigError> { ... }

    // ❌ DON'T: No parsing methods on Raw types
    pub fn parse(&self) -> Result<Frontmatter, ConfigError> { ... }
}
```

**Acceptable deviations (parsing helpers only):**

```rust
impl RawGlobal {
    // ✅ OK: Format-specific parsing (not validation)
    pub fn from_toml_str(s: &str) -> Result<Self, toml::de::Error> {
        toml::from_str(s)  // Just deserialization, not validation
    }
}
```

---

#### Shape 2: Domain Types (Application Core)

**Purpose:** Represent valid business entities with guaranteed invariants.

**Characteristics:**

- **Private fields** - enforce invariants via getters
- **Typed enums** - `LogLevel`, not `String`
- **Newtypes** - `FrontmatterKey`, `VaultRoot`, not raw `String`/`PathBuf`
- **Methods allowed** - business logic, queries, commands
- **Persisted** - stored in database via `rkyv::Archive`
- **Location:** `<context>/aggregate.rs`, `<context>/<entity>.rs`
- **Derives:** `rkyv::Archive + Serialize + Deserialize`, optionally `serde` (feature-gated)

**Example:**

```rust
/// Validated frontmatter configuration.
#[derive(Debug, Clone, PartialEq,
         rkyv::Archive, rkyv::Serialize, rkyv::Deserialize)]
#[non_exhaustive]
pub struct Frontmatter {
    alias_key: FrontmatterKey,        // Validated (non-empty)
    title_key: FrontmatterKey,        // Validated (non-empty)
    date_created_key: FrontmatterKey, // Validated (non-empty)
    // Private fields
}

impl Frontmatter {
    // ✅ DO: Constructors enforce invariants
    pub fn new(
        alias_key: FrontmatterKey,
        title_key: FrontmatterKey,
        date_created_key: FrontmatterKey,
    ) -> Self {
        Self { alias_key, title_key, date_created_key }
    }

    // ✅ DO: Getters for private fields
    pub fn alias_key(&self) -> &FrontmatterKey {
        &self.alias_key
    }
}
```

---

#### Validation Boundary: TryFrom

**Purpose:** Explicit, single validation point from unvalidated → validated.

**Pattern:** Use `TryFrom<Raw*>` trait as the validation boundary.

**Why TryFrom (not methods on Raw types):**

- ✅ **Rust convention** - standard validation pattern
- ✅ **Single point** - all validation in one place
- ✅ **Explicit** - caller must call `try_into()` or `TryFrom::try_from()`
- ✅ **Testable** - easy to construct invalid Raw types for tests
- ✅ **Type-safe** - compiler enforces validation before domain use

**Example:**

```rust
impl TryFrom<RawFrontmatter> for Frontmatter {
    type Error = ConfigError;

    fn try_from(raw: RawFrontmatter) -> Result<Self, Self::Error> {
        // Extract with defaults
        let alias_key = raw.alias_key.unwrap_or_else(|| "aliases".to_owned());
        let title_key = raw.title_key.unwrap_or_else(|| "title".to_owned());

        // Validate (empty strings are invalid)
        let alias_key = FrontmatterKey::try_new(alias_key)?;
        let title_key = FrontmatterKey::try_new(title_key)?;

        // Construct validated domain type
        Ok(Frontmatter::new(alias_key, title_key, /* ... */))
    }
}
```

**Usage:**

```rust
// External input → Raw → Domain
let raw: RawFrontmatter = toml::from_str(contents)?;  // Parse
let validated: Frontmatter = raw.try_into()?;          // Validate (explicit)
```

---

#### Shape 3: `*View` Types (Read-Optimized Projections)

**Purpose:** Represent read-optimized projections in the expendable database cache.

**When to Create:**
Introduce `*View` types when mapping files to database queries:

- ✅ When database needs a flattened or indexing-friendly projection.
- ✅ When domain shape is inefficient for storage.
- ✅ When projecting data from multiple files.

**Default Strategy:** Store domain types directly if they are simple, but use `*View` for read-optimized queries.

**Location:** `<context>/view.rs` or `<context>/views/<view_name>.rs`
**Derives:** `rkyv::Archive + Serialize + Deserialize`

**Example:**

```rust
// schema/view.rs
#[derive(rkyv::Archive, rkyv::Serialize, rkyv::Deserialize)]
pub struct SchemaView {
    pub name: Box<str>,  // Flattened from SchemaName newtype
    pub properties: Vec<PropertyView>,
}

impl From<Schema> for SchemaView { /* ... */ }
impl TryFrom<SchemaView> for Schema { /* ... */ }
```

---

#### Conversion Flow Pattern

✅ **Prefer:**

```rust
// fs/parsers.rs - File → Raw → Domain
fn parse_schema_yaml(path: &Path) -> Result<Schema, ParseError> {
    let content = fs::read_to_string(path)?;
    let raw: RawSchema = serde_yaml::from_str(&content)?;  // Parse
    Schema::try_from(raw)  // Validate (TryFrom boundary)
}

// schema/adapters/storage.rs - Domain → Storage
impl SchemaStorage for RedbSchemaStorage<'_> {
    fn save(&self, schema: &Schema) -> Result<(), DbError> {
        // Option 1: Store domain directly if simple
        self.db.put("schemas", schema.name().as_ref(), schema)

        // Option 2: Convert to *View for read optimization
        // let view = SchemaView::from(schema);
        // self.db.put("schemas", schema.name().as_ref(), &view)
    }
}
```

---

#### Design Rationale

**Why separate Raw from Domain?**

**Problems with direct deserialization:**

```rust
// ❌ BAD: Domain type directly deserialized
#[derive(Deserialize)]
pub struct Frontmatter {
    alias_key: String,  // Can be empty! Invalid states representable
}
```

**Benefits of Raw/Domain split:**

1. **Invalid states unrepresentable** - domain types enforce invariants
2. **Clear error messages** - distinguish parse errors from validation errors
3. **Flexible parsing** - Raw types accept typos, wrong types, partial configs
4. **Easy testing** - construct invalid Raw types for test fixtures
5. **Independent evolution** - file format can change without breaking domain

**Why NOT methods on Raw types?**

**Problem with this approach:**

```rust
impl RawFrontmatter {
    pub fn parse(&self) -> Result<Frontmatter, ConfigError> {
        // ❌ BAD: Raw types now have behavior
    }
}
```

**Issues:**

- Breaks single responsibility (Raw types parse AND validate)
- Violates "dumb data, smart functions" principle
- Against Rust conventions (`TryFrom` is standard)
- Harder to mock/test (Raw types become stateful)
- Validation might be forgotten (not enforced by type system)

---

#### Testing Pattern

**Test invalid Raw input:**

```rust
#[test]
fn rejects_empty_alias_key() {
    let raw = RawFrontmatter {
        alias_key: Some("".to_string()), // Invalid
        ..Default::default()
    };

    let result = Frontmatter::try_from(raw);
    assert!(matches!(result, Err(ConfigError::ValidationFailed { .. })));
}
```

**Test valid conversion:**

```rust
#[test]
fn converts_valid_raw_config() -> Result<(), ConfigError> {
    let raw = RawFrontmatter {
        alias_key: Some("aliases".to_string()),
        title_key: Some("title".to_string()),
        ..Default::default()
    };

    let frontmatter = Frontmatter::try_from(raw)?;
    assert_eq!(frontmatter.alias_key().as_str(), "aliases");
    Ok(())
}
```

---

#### Anti-Patterns to Avoid

❌ **DON'T: Validate in constructors**

```rust
impl Frontmatter {
    // ❌ BAD: Makes invalid state constructible
    pub fn new(alias_key: String) -> Self {
        Self { alias_key }  // Can be empty!
    }
}
```

❌ **DON'T: Skip Raw layer**

```rust
// ❌ BAD: Deserializing directly into domain
#[derive(Deserialize)]
pub struct Frontmatter {
    alias_key: String, // Can be empty!
}
```

❌ **DON'T: Add methods to Raw types**

```rust
impl RawFrontmatter {
    // ❌ BAD: Raw types should have zero methods
    pub fn is_valid(&self) -> bool { ... }
}
```

❌ **DON'T: Persist Raw types**

```rust
// ❌ BAD: Never store unvalidated data
db.put("config", "frontmatter", &raw_frontmatter)?;
```

❌ **DON'T: Create `*View` types prematurely**

```rust
// ❌ BAD: Creating *View without query performance justification
pub struct SchemaView { /* ... */ }  // Domain works fine!
```

---

#### Summary Table

| Aspect             | Raw Types                   | Domain Types             | View Types                  |
| ------------------ | --------------------------- | ------------------------ | --------------------------- |
| **Purpose**        | Accept flexible input       | Represent valid entities | Optimize storage            |
| **Validation**     | None (can be invalid)       | Guaranteed invariants    | N/A (mechanical conversion) |
| **Fields**         | All `Option<T>`             | Typed, private           | Flattened, optimized        |
| **Methods**        | Zero (pure DTO)             | Business logic allowed   | Only From/TryFrom           |
| **Persistence**    | Never                       | Via rkyv                 | Yes (if needed)             |
| **Conversion**     | N/A                         | Via `TryFrom<Raw*>`      | Via `From<Domain>`          |
| **Testing**        | Easy to make invalid        | Hard to make invalid     | Not tested directly         |
| **When to create** | Always (for external input) | Always                   | Rarely (profiling only)     |

**Golden Rule:** Raw types are **dumb data**, validation is **explicit** via `TryFrom`, domain types are **smart** with invariants, View types are **rare optimizations**.

## Storage Patterns

Following **ADR 003 Appendix A**, minimize coupling between domain and storage format. When a `*View` representation is introduced, apply these triggers and guidelines; default remains to store domain types directly if they map well, but use `*View` to optimize database querying.

**When to Introduce View Types:**

- ✅ For read-optimized database projections (NoteView, SchemaView)
- ✅ When database needs a flattened or indexing-friendly format
- ✅ When projecting data combined from multiple domain boundaries
- ❌ Not for every type (avoid DTO explosion)

**Pattern:**

```rust
// Domain type (in context/aggregate.rs)
pub struct Schema {
    pub name: SchemaName,  // Validated newtype
    pub properties: Vec<Property>,
    // Ergonomic, behavior-rich
}

// View type (in <context>/view.rs or storage adapter)
#[derive(Archive, Serialize, Deserialize)]
#[rkyv(derive(Debug))]
pub struct SchemaView {
    pub name: Box<str>,
    pub version: u32,
    pub properties: Vec<PropertyView>,
    // Stable layout for fast queries
}

// Conversions (in storage adapter)
impl From<Schema> for SchemaView { /* ... */ }
impl TryFrom<SchemaView> for Schema { /* ... */ }
```

**Guidelines:**

1. **One `*View` per persisted aggregate** if beneficial for queries
2. **Keep conversions mechanical and co-located** in storage layer
3. **Use projections for new query patterns** (don't widen stored blobs)
4. **Keep archived compute closure-scoped** (never leak transaction-scoped borrows)
5. **Treat `*View` types as expendable cache representations** (can be rebuilt from files)

**Location:**

- `schema/view.rs` - SchemaView and conversions
- `note/view.rs` - NoteView and conversions
- Or `<context>/adapters/storage.rs` if storing adapter and view type together

### Zero-Copy Idioms (Footguns to Avoid)

- **rkyv format control**: Treat endianness/alignment/pointer-width feature choices as a persisted-format contract.
- **rkyv validation**: Use `rkyv::access` at trust boundaries (files/network/user input).
- **redb guards**: `AccessGuard` values borrow the transaction/table; do not return or store them beyond the transaction scope.
- **redb custom Value**: Implement `redb::Value` via local newtypes/wrappers when you need custom encoding.
- **moka determinism**: In tests, call `run_pending_tasks()` to ensure cache stats are consistent.

### Archived Types Usage (rkyv)

**Pattern:** Use `rkyv::Archived<T>` in public APIs and Storage traits. Use `Archived*` only in the defining module to add small, safe accessors for private fields on archived representations.

✅ **Prefer:**

```rust
// Public API / port signature
type NoteArchived<'a> = &'a rkyv::Archived<Note>;

// Local accessors for private fields (same module as Tag)
impl ArchivedTag {
    pub fn full_path(&self) -> &str { /* ... */ }
}
```

❌ **Avoid:**

```rust
// Leaking Archived* in public signatures
fn with_archived_tag(&self, f: impl FnOnce(&ArchivedTag) -> R) -> R;
```

**Rationale:** `rkyv::Archived<T>` keeps public APIs stable and generic. `Archived*` accessors are only for local encapsulation when archived fields are private.

## Project Structure & Module Layout

**Workspace Organization:**

- **Crate Separation:** `lithos-core` (Logic + Infra) vs `lithos-cli` (Driver).
- **Module Organization:** Within crates, contexts use `<context>/mod.rs` pattern.
  - Each context is a folder with `mod.rs` as entry point
  - Submodules organized by responsibility (aggregate, command, query, ports, error, events)
- **Test Placement:** Unit tests in same file (`#[cfg(test)]`), integration tests in `tests/`.
- **Binary Organization:** CLI crate delegating to `lithos-core`.

**File Structure Standards:**

- **Lithos Core:** `src/lib.rs`, `src/db/`, `src/fs/`, `src/<context>/` (contexts with mod.rs, errors/events/ports co-located).
- **Lithos CLI:** `src/main.rs`, `src/commands/`.
- **Common Patterns:** Group related items, keep files focused.

## Error Handling & Diagnostics

**Error Handling Standards:**

- **Core Errors:** `thiserror::Error` for typed, co-located error enums (e.g. `note::Error`).
- **Context Addition:** `anyhow::Result` only in `main.rs` if prototyping; otherwise `miette`.
- **CLI Output:** `miette` for user-facing errors with help/labels.
- **Logging:** `tracing` with structured spans.
- **Panic Avoidance:** Never use `unwrap()`, `expect()` in library code.

## Async & Concurrency Rules

**Async Patterns:**

- **Sync-First:** Core domain logic and file I/O must be synchronous.
- **Async at Edge:** `lithos-cli` uses `tokio::main`.
- **Bridging:** Use `tokio::task::spawn_blocking` for concurrent core operations.
- **No Async Traits:** Do NOT use `#[async_trait]` in `lithos-core`.

## Documentation Standards

**Documentation Standards:**

- **Item Documentation:** Use `///` for public items.
- **Module Documentation:** Use `//!` at top of `<context>.rs`.
- **Examples:** Include code examples for public APIs.

## Communication & Dependency Rules

**State Mutation & Flow:**

- **Direct Functional Calls:** Mutations happen via direct functional calls and pipeline iterators.
- **Error Propagation:** All fallible operations must return `Result<T, E>`.
- **No Event Bus:** System state mutations do not use event sourcing or an event bus.
- **Side Effects:** Handled explicitly through pipeline composition and orchestration.

**Inter-Module Communication:**

- **Context Isolation:** Business contexts (note, schema, template) do not import each other.
- **Cross-Cutting Context:** Config (user-configurable business rules) is available to all contexts.
- **Pure Infrastructure:** db, fs, patterns (generic utilities) are available to all contexts.
- **Orchestration:** Cross-context workflows happen in CLI layer or dedicated app module.

**Dependency Flow Rules:**

✅ **ALLOWED:**

```rust
// From any business context (note, schema, template):
use crate::config::Global;      // Config is cross-cutting infrastructure
use crate::config::Vault;
use crate::db;                  // Infrastructure
use crate::fs;
use crate::patterns;

// Within context:
use super::aggregate::Note;
use super::error::NoteError;
use super::storage::NoteStorage;
```

❌ **FORBIDDEN:**

```rust
// Business contexts importing each other:
use crate::note::Note;          // From schema context
use crate::schema::Schema;      // From note context
use crate::template::Template;  // From config context
```

## Process & Tooling

**Testing Standards:**

- **Unit Tests:** `#[cfg(test)]` in same file.
- **Integration Tests:** `tests/integration/` (CLI -> Core -> DB).
- **Architecture Boundaries:** Enforced via module visibility, dependency flow rules, and code review.
- **Benchmarks:** `lithos-core/benches/` (zero-copy validation).

**Configuration Management:**

- **Hierarchy:** CLI args > Config file > Defaults.
- **Validation:** Serde validation.

**Build & Development:**

- **Mise:** Task runner (`mise run verify`, `mise run build`).
- **Hooks:** Pre-commit enforcement.

**Clippy Complexity Limits:**

- **Cyclomatic Complexity:** `clippy::cognitive_complexity` threshold set to 15 (warn) and 25 (deny) to prevent overly complex functions
- **Function Length:** `clippy::too_many_lines` with limit of 100 lines per function
- **Arguments:** `clippy::too_many_arguments` with max 7 arguments
- **Nesting:** `clippy::nested_if_else` and `clippy::too_many_nested_loops` limits enforced
- **Code Quality:** Deny `clippy::unwrap_used`, `clippy::expect_used`, `clippy::todo`, `clippy::unimplemented`, `clippy::dbg_macro`
- **Performance:** Enable `clippy::inefficient_to_string`, `clippy::redundant_clone`, `clippy::needless_collect`
- **Style:** Enforce `clippy::implicit_return`, `clippy::single_match_else`, `clippy::redundant_else`

## Enforcement Guidelines

**All AI Agents MUST:**

- Follow established naming conventions without exception
- Maintain context boundaries (business contexts must not import each other; only infrastructure/cross-cutting)
- Use unified storage pattern (generic over Storage traits, not direct database coupling)
- Use async/await consistently in async-enabled edge layers with proper error handling; keep core sync-first
- Implement comprehensive error handling with typed errors and context
- Write tests for all public APIs and critical paths including async operations
- Document public traits and complex business logic with examples following Rust doc standards
- Use tracing for all logging with structured spans and consistent levels
- Keep cyclomatic complexity under 15 and cognitive complexity under 25 per function
- Never use `unwrap()`, `expect()`, `todo()`, or `unimplemented()` in production code
- Run clippy on all code with complexity limits enforced before commits via pre-commit hooks

**Pattern Enforcement:**

- **Pre-commit Hooks:** Run clippy, rustfmt, and tests before commits to maintain clean git history and catch issues early
- **Code Reviews:** Automated checks for naming violations, dependency rules, architectural boundaries, and complexity metrics; manual review for logic and API design
- **CI Pipeline:** Clippy with complexity limits, rustfmt, and custom lint enforcement with failure on violations; require green CI for merges
- **Architecture Tests:** Integration tests verifying context boundaries and dependency flow rules
- **Documentation:** Pattern violations documented in commit messages with remediation steps
- **Quality Gates:** Minimum test coverage (80%), no clippy warnings, performance regression checks, security audit passing

**Advanced Enforcement:**

- **Dependency Analysis:** Use `cargo deny` to prevent unwanted dependency introductions
- **Security Auditing:** Regular `cargo audit` runs to catch vulnerabilities
- **Performance Regression:** Automated benchmark comparisons to prevent performance degradation
- **Code Coverage:** Minimum coverage thresholds enforced in CI with `tarpaulin`
- **Style Consistency:** Automated import sorting and formatting checks; use `cargo fmt --check` in CI

## Pattern Examples

**Legacy Example (Non-Compliant, retained for context):**

This example predates current async and unwrap prohibitions. It is retained for historical context only; do not treat it as compliant guidance.

````rust
/// A note in the vault with its metadata and content.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Note {
    /// Vault-relative path serving as the unique identifier
    pub path: String,
    /// Parsed frontmatter metadata
    pub frontmatter: Frontmatter,
}

impl Note {
    /// Creates a new note with validation.
    ///
    /// # Errors
    /// Returns `DomainError` if validation fails.
    ///
    /// # Examples
    /// ```
    /// use lithos_domain::Note;
    /// let note = Note::new("path.md".to_string(), frontmatter)?;
    /// ```
    pub fn new(path: String, frontmatter: Frontmatter) -> Result<Self, DomainError> {
        if path.is_empty() {
            return Err(DomainError::InvalidPath);
        }
        Ok(Self { path, frontmatter })
    }
}

#[async_trait]
pub trait VaultWriterPort: Send + Sync {
    /// Persists a note to the vault storage.
    ///
    /// # Errors
    /// Returns `DomainError` if persistence fails.
    async fn persist_note(&self, note: Note) -> Result<(), DomainError>;
}

pub struct VaultIndexerService {
    vault_writer: Arc<dyn VaultWriterPort>,
    event_bus: Arc<dyn EventBus>,
}

impl VaultIndexerService {
    /// Indexes the vault and publishes completion events.
    ///
    /// This function maintains low complexity by delegating to helper methods.
    pub async fn index_vault(&self) -> Result<IndexStats, DomainError> {
        self.event_bus.publish(DomainEvent::VaultIndexingStarted).await?;

        let stats = self.perform_indexing().await?;

        self.event_bus.publish(DomainEvent::VaultIndexingCompleted { stats: stats.clone() }).await?;

        Ok(stats)
    }

    async fn perform_indexing(&self) -> Result<IndexStats, DomainError> {
        Ok(IndexStats::default())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DomainEvent {
    /// Fired when vault indexing begins
    VaultIndexingStarted,
    /// Fired when vault indexing completes
    VaultIndexingCompleted { stats: IndexStats },
    /// Fired when a note is indexed
    NoteIndexed { note_path: String, indexed_at: DateTime<Utc> },
    /// Fired when a template is executed
    TemplateExecuted { template_id: String, success: bool },
}

#[tokio::test]
async fn vault_indexing_succeeds() {
    let mock_writer = Arc::new(MockVaultWriter::new());
    let mock_bus = Arc::new(MockEventBus::new());
    let service = VaultIndexerService::new(mock_writer, mock_bus);

    let result = service.index_vault().await;
    assert!(result.is_ok());

    let stats = result.unwrap();
    assert_eq!(stats.total_notes, 0);
}
````

**Anti-Patterns:**

- Functions exceeding 15 cyclomatic complexity or 100 lines
- Using `unwrap()` or `expect()` in production code
- Deeply nested control structures
- Inconsistent naming or missing documentation
- Blocking operations in async functions without `spawn_blocking`
- Tests that don't cover error cases or async behavior
- Missing doc examples for public APIs
- Not running clippy or ignoring warnings

**Resource References:**

- Rust Official Documentation
- Clippy Lints Reference
- Tokio Async Patterns
