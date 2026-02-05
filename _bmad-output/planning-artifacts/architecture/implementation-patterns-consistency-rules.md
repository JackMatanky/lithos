---
title: "Implementation Patterns & Consistency Rules"
description: "Development patterns, naming conventions, and consistency rules for Lithos implementation"
author: "Jack"
date: "2026-01-23"
last_updated: "2026-02-05"
section: "Implementation Standards"
---

# Implementation Patterns & Consistency Rules

## Pattern Categories Defined

**Critical Conflict Points Identified:** 30+ areas where AI agents could make different choices in async Rust CLI applications with port-based CQRS, bounded contexts, and event-driven patterns.

## Naming Patterns

**Rust Naming Conventions:**

- **Crate/Package Names:** Cargo *package* names are `kebab-case` (e.g., `lithos-core`, `lithos-cli`). In Rust code, the crate import path is `snake_case` (e.g., `lithos_core`).
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
- **Be concise (but not cryptic):** Avoid nonstandard abbreviations and “inside jokes”; shorten only when meaning remains obvious.
- **Use simple, correct words:** Prefer the smallest set of small words that preserve meaning; avoid terms that can be read two ways.
- **Unify concept names:** One term per concept. If we choose “vault” (not “repo”/“workspace”) for the user’s note collection, use “vault” consistently in APIs, docs, and modules.
- **Avoid type-noise in names:** Don’t encode types (e.g., `*_str`, `*_vec`) unless it disambiguates two values with the same conceptual meaning.

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
- **Don’t pattern-match references:** Prefer explicit dereferencing (`|x| *x`) over `|&x| x`.
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

- Use lifetimes as “documentation”: pick names derived from what is being borrowed (e.g., `'db`, `'tx`, `'bytes`, `'src`).
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
- **Port Traits:** Descriptive names ending with `Port` (e.g., `CacheWriterPort`, `VaultReaderPort`)
- **DTO Structs:** Prefer role-based names over type-suffixes (e.g., `VaultFile`, `VaultFileRecord`, `CreateNoteRequest`). If `Dto` is used, reserve it for strict boundary/wire types (CLI/adapters/serde), not domain/core.
- **Event Names:** `PascalCase` with past tense (e.g., `NoteIndexed`, `TemplateExecuted`)

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

## Port-Based CQRS Implementation Patterns

**Core Principle:** Separate read and write capabilities via split port traits to prevent interface bloat, enable read-only test fakes, and support future backend flexibility.

### Split Ports Pattern

**Pattern:** Define separate `QueryPort` and `CommandPort` traits in single `<context>/ports.rs` file.

**Why Split Ports:**
- Read-only use cases don't implement writes
- Test fakes only implement needed capabilities
- Future flexibility (cache reads, DB writes independently)
- Prevents "god interface" anti-pattern

✅ **Prefer:**

```rust
// <context>/ports.rs - Single file, multiple focused traits
pub trait SchemaQueryPort {
    type Error: std::error::Error;
    type Archived<'a> where Self: 'a;  // GAT for zero-copy

    // COLD TIER: Owned reads for mutations/complex operations
    fn find_owned_by_name(&self, name: &SchemaName)
        -> Result<Option<Schema>, Self::Error>;

    fn list_all_owned(&self) -> Result<Vec<Schema>, Self::Error>;

    // HOT TIER: Zero-copy closure-scoped reads (LSP hot path)
    fn with_archived_by_name<R>(
        &self,
        name: &SchemaName,
        f: impl for<'a> FnOnce(Self::Archived<'a>) -> R,
    ) -> Result<Option<R>, Self::Error>;
}

pub trait SchemaCommandPort {
    type Error: std::error::Error;

    fn save(&self, schema: &Schema) -> Result<(), Self::Error>;
    fn delete(&self, name: &SchemaName) -> Result<bool, Self::Error>;
    fn batch_save(&self, schemas: &[Schema]) -> Result<(), Self::Error>;
}
```

❌ **Avoid:**

```rust
// Single "Store" trait forces all implementers to provide everything
pub trait SchemaStore {
    // Read operations
    fn find_owned_by_name(...) -> ...;
    fn list_all_owned(...) -> ...;

    // Write operations (read-only fakes forced to implement these!)
    fn save(...) -> ...;
    fn delete(...) -> ...;
}
```

### Generic CQRS Types Pattern

**Pattern:** CQRS types generic over respective ports, hide generics via type aliases.

✅ **Prefer:**

```rust
// <context>/query.rs - Generic over QueryPort
pub struct Query<Q> {
    query_port: Q,
}

impl<Q: SchemaQueryPort> Query<Q> {
    pub fn new(query_port: Q) -> Self {
        Self { query_port }
    }

    pub fn find_owned_by_name(&self, name: &SchemaName)
        -> Result<Option<Schema>, QueryError<Q::Error>>
    {
        self.query_port.find_owned_by_name(name)
            .map_err(QueryError::Storage)
    }

    // Hot path helper
    pub fn with_archived_by_name<R>(
        &self,
        name: &SchemaName,
        f: impl for<'a> FnOnce(Q::Archived<'a>) -> R,
    ) -> Result<Option<R>, QueryError<Q::Error>>
    {
        self.query_port.with_archived_by_name(name, f)
            .map_err(QueryError::Storage)
    }
}

// <context>/command.rs - Generic over CommandPort
pub struct Command<C> {
    command_port: C,
}

impl<C: SchemaCommandPort> Command<C> {
    pub fn new(command_port: C) -> Self {
        Self { command_port }
    }

    pub fn save(&self, schema: &Schema)
        -> Result<(), CommandError<C::Error>>
    {
        self.command_port.save(schema)
            .map_err(CommandError::Storage)
    }
}
```

### Adapter Implementation Pattern

**Pattern:** Adapters live in `db/<context>_adapter.rs` and implement port traits.

✅ **Prefer:**

```rust
// db/schema_adapter.rs - Infrastructure implements domain ports
pub struct RedbSchemaQueryAdapter<'db> {
    db: &'db Database,
}

impl SchemaQueryPort for RedbSchemaQueryAdapter<'_> {
    type Error = DbError;
    type Archived<'a> = &'a ArchivedSchema;  // Domain type or StoredSchema

    fn find_owned_by_name(&self, name: &SchemaName)
        -> Result<Option<Schema>, DbError>
    {
        // Default: Store domain type directly (has rkyv derives)
        self.db.get_owned::<Schema>("schemas", name.as_ref())

        // Optional: If Stored* exists for optimization
        // let stored: Option<StoredSchema> = self.db.get_owned("schemas", name.as_ref())?;
        // Ok(stored.map(Schema::from))
    }

    fn with_archived_by_name<R>(
        &self,
        name: &SchemaName,
        f: impl for<'a> FnOnce(&'a ArchivedSchema) -> R,
    ) -> Result<Option<R>, DbError> {
        self.db.get::<Schema, _, _>("schemas", name.as_ref(), f)
    }
}

pub struct RedbSchemaCommandAdapter<'db> {
    db: &'db Database,
}

impl SchemaCommandPort for RedbSchemaCommandAdapter<'_> {
    type Error = DbError;

    fn save(&self, schema: &Schema) -> Result<(), DbError> {
        // Default: Store domain type directly
        self.db.put("schemas", schema.name().as_ref(), schema)
    }
}
```

❌ **Avoid:**

```rust
// Adapter in domain context (violates dependency flow)
// <context>/redb_adapter.rs - WRONG LOCATION!
pub struct RedbSchemaStore { /* ... */ }
impl SchemaStore for RedbSchemaStore {
    // Now domain depends on infrastructure!
}
```

### Type Aliases for Ergonomics

**Pattern:** Hide generic complexity from 99% of callers.

✅ **Prefer:**

```rust
// <context>/mod.rs - Public API with convenient aliases
pub type RedbSchemaQuery<'db> = Query<RedbSchemaQueryAdapter<'db>>;
pub type RedbSchemaCommand<'db> = Command<RedbSchemaCommandAdapter<'db>>;

impl<'db> RedbSchemaQuery<'db> {
    pub fn new_redb(db: &'db Database) -> Self {
        Self::new(RedbSchemaQueryAdapter::new(db))
    }
}

impl<'db> RedbSchemaCommand<'db> {
    pub fn new_redb(db: &'db Database) -> Self {
        Self::new(RedbSchemaCommandAdapter::new(db))
    }
}

// CLI code never sees generics:
let query = RedbSchemaQuery::new_redb(&db);
let schema = query.find_owned_by_name(name)?;
```

### Three-Shape Serialization Pattern

**Core Principle:** Separate concerns of parsing, validation, and storage optimization.

**Shapes:**

1. **Raw\* (parsing boundary):**
   - Location: `<context>/raw.rs`
   - Derives: `serde::Serialize + Deserialize`
   - Purpose: Tolerant parsing from files
   - Nullable fields for better error messages

2. **Domain (application core):**
   - Location: `<context>/aggregate.rs`
   - Derives: `rkyv::Archive + Serialize + Deserialize`, optionally `serde` (feature-gated)
   - Purpose: Validated entities used throughout app
   - **Has rkyv derives** for zero-copy database operations
   - Private fields with smart constructors

3. **Stored\* (storage optimization, optional):**
   - Location: `db/stored/<context>.rs`
   - Derives: `rkyv::Archive + Serialize + Deserialize`
   - Purpose: Only when domain shape inefficient for storage
   - Flattened newtypes, optimized layouts

**Conversion Flow Pattern:**

✅ **Prefer:**

```rust
// fs/parsers.rs - File → Raw → Domain
fn parse_schema_yaml(path: &Path) -> Result<Schema, ParseError> {
    let content = fs::read_to_string(path)?;
    let raw: RawSchema = serde_yaml::from_str(&content)?;  // serde
    Schema::try_from(raw)  // validation
}

// db/schema_adapter.rs - Domain → Storage
impl SchemaCommandPort for RedbSchemaCommandAdapter<'_> {
    fn save(&self, schema: &Schema) -> Result<(), DbError> {
        // Option 1: Store domain directly (preferred, default)
        self.db.put("schemas", schema.name().as_ref(), schema)

        // Option 2: Convert to Stored* only if profiling shows need
        // let stored = StoredSchema::from(schema);
        // self.db.put("schemas", schema.name().as_ref(), &stored)
    }
}
```

**When to Create Stored\* Types:**

Only introduce `Stored*` when profiling reveals:
- ✅ Wrapper newtypes (SchemaName) complicate database indexing
- ✅ Deep nesting causes excessive alignment copy overhead
- ✅ Arc<T> sharing doesn't serialize efficiently
- ✅ Storage layout differs significantly from domain representation

**Default Strategy:** Store domain types directly (they have rkyv derives).

❌ **Avoid:**

```rust
// Creating Stored* prematurely without performance justification
// db/stored/schema.rs - DON'T CREATE until needed!
pub struct StoredSchema { /* ... */ }

// Domain without rkyv derives (wrong!)
// <context>/aggregate.rs
#[derive(Debug, Clone)]  // Missing rkyv derives!
pub struct Schema { /* ... */ }
```

### Port-Based Testing Pattern

**Pattern:** Different test fakes for read vs write, minimal implementation.

✅ **Prefer:**

```rust
// <context>/query.rs tests - Read-only fake
#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    struct FakeSchemaQueryPort {
        schemas: HashMap<SchemaName, Schema>,
    }

    impl SchemaQueryPort for FakeSchemaQueryPort {
        type Error = String;
        type Archived<'a> = &'a Schema;  // Just borrow domain type

        fn find_owned_by_name(&self, name: &SchemaName)
            -> Result<Option<Schema>, String>
        {
            Ok(self.schemas.get(name).cloned())
        }

        fn with_archived_by_name<R>(
            &self,
            name: &SchemaName,
            f: impl for<'a> FnOnce(&'a Schema) -> R,
        ) -> Result<Option<R>, String> {
            Ok(self.schemas.get(name).map(f))
        }

        // No need to implement list_all_owned if test doesn't use it!
        fn list_all_owned(&self) -> Result<Vec<Schema>, String> {
            unimplemented!("not needed for this test")
        }
    }

    #[test]
    fn finds_existing_schema() {
        let mut store = FakeSchemaQueryPort {
            schemas: HashMap::new()
        };
        let schema = Schema::new("test")?;
        store.schemas.insert(schema.name().clone(), schema.clone());

        let query = Query::new(store);  // Generic Query<FakeSchemaQueryPort>

        let result = query.find_owned_by_name(schema.name())?;
        assert_eq!(result, Some(schema));
    }
}
```

### Port Pattern Checklist

When implementing port-based CQRS, verify:

- [ ] Ports split into `QueryPort` and `CommandPort` (not single `Store`)
- [ ] Both ports defined in single `<context>/ports.rs` file
- [ ] Query uses GAT: `type Archived<'a> where Self: 'a`
- [ ] Hot path uses HRTB: `impl for<'a> FnOnce(Self::Archived<'a>) -> R`
- [ ] CQRS types generic: `Query<Q>`, `Command<C>`
- [ ] Adapters in `db/<context>_adapter.rs` (not in domain context)
- [ ] Type aliases hide generics: `RedbSchemaQuery<'db>`
- [ ] Domain types have rkyv derives
- [ ] `Stored*` only created when profiling shows need
- [ ] Test fakes implement only needed port methods
- [ ] No unsafe blocks in CQRS or port implementations
- [ ] Context isolation maintained (domain doesn't import db/)

## Structure Patterns

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

## Format Patterns

**Error Handling Standards:**

- **Core Errors:** `thiserror::Error` for typed, co-located error enums (e.g. `note::Error`).
- **Context Addition:** `anyhow::Result` only in `main.rs` if prototyping; otherwise `miette`.
- **CLI Output:** `miette` for user-facing errors with help/labels.
- **Logging:** `tracing` with structured spans.
- **Panic Avoidance:** Never use `unwrap()`, `expect()` in library code.

**Async Patterns:**

- **Sync-First:** Core domain logic and file I/O must be synchronous.
- **Async at Edge:** `lithos-cli` uses `tokio::main`.
- **Bridging:** Use `tokio::task::spawn_blocking` for concurrent core operations.
- **No Async Traits:** Do NOT use `#[async_trait]` in `lithos-core`.

**Documentation Standards:**

- **Item Documentation:** Use `///` for public items.
- **Module Documentation:** Use `//!` at top of `<context>.rs`.
- **Examples:** Include code examples for public APIs.

**Serialization Patterns:**

- **Feature Flag:** Use `#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]`.
- **Optional:** Serde is optional for domain types, required for DTOs/Config.
- **Zero-Copy:** Use `rkyv` for performance-critical storage types.

## Communication Patterns

**Event System Standards:**

- **Event Naming:** `PascalCase` with past tense (e.g., `NoteIndexed`).
- **Co-location:** Events defined in `<context>/events.rs`.
- **Dispatch:** Deferred dispatch via `UnitOfWork` or simple callbacks (Phase 1).

**Inter-Module Communication:**

- **Context Isolation:** Business contexts (note, schema, template) do not import each other.
- **Cross-Cutting Infrastructure:** Config, db, fs, patterns are available to all contexts.
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
use super::ports::NoteStore;
```

❌ **FORBIDDEN:**
```rust
// Business contexts importing each other:
use crate::note::Note;          // From schema context
use crate::schema::Schema;      // From note context
use crate::template::Template;  // From config context
```

**Database Access Rules:**

- **Port-Based Access:** Contexts define storage port traits (e.g., `SchemaStore`, `NoteStore`)
- **Generic CQRS:** Command/Query types are generic over port: `Query<S: SchemaStore>`
- **Zero-Copy Reads:** Ports use GATs to enable closure-based archived access
- **Default Backend:** Type aliases hide generics: `RedbSchemaQuery<'db>`
- **Test Substitution:** Use `FakeSchemaStore` implementing the same port

**CQRS Pattern:**

```rust
// Port trait (in context/ports.rs)
pub trait SchemaStore {
    type Error;
    type Archived<'a> where Self: 'a;

    fn find_owned_by_name(&self, name: &SchemaName)
        -> Result<Option<Schema>, Self::Error>;

    fn with_archived_by_name<R>(
        &self,
        name: &SchemaName,
        f: impl for<'a> FnOnce(Self::Archived<'a>) -> R,
    ) -> Result<Option<R>, Self::Error>;
}

// Query implementation (in context/query.rs)
pub struct Query<S> {
    store: S,
}

impl<S: SchemaStore> Query<S> {
    pub fn new(store: S) -> Self {
        Self { store }
    }

    pub fn find_owned_by_name(&self, name: &SchemaName)
        -> Result<Option<Schema>, QueryError<S::Error>>
    {
        self.store.find_owned_by_name(name)
            .map_err(QueryError::Storage)
    }
}

// Type alias for default backend (in context/mod.rs)
pub type RedbSchemaQuery<'db> = Query<RedbSchemaStore<'db>>;
```

**CQRS Naming Conventions:**

- **Queries:** `find_*`, `get_*`, `list_*`, `count_*`.
- **Commands:** `save`, `delete`, `update`, `create`.
- **Port Traits:** `<Context>Store` (e.g., `NoteStore`, `SchemaStore`)
- **CQRS Types:** `Query<S>`, `Command<S>` generic over store port
- **Type Aliases:** `Redb<Context>Query<'db>` for ergonomic use

## Storage Patterns

Following **ADR 003 Appendix A**, minimize coupling between domain and storage format:

**When to Introduce Stored Types:**

- ✅ For persisted aggregates (Note, Schema, Template, Config)
- ✅ When domain refactors shouldn't trigger migrations
- ✅ When archived layout needs careful control (alignment, endianness)
- ❌ Not for every type (avoid DTO explosion)
- ❌ Not for value objects unless they cause migration pain

**Pattern:**

```rust
// Domain type (in context/aggregate.rs)
pub struct Schema {
    pub name: SchemaName,  // Validated newtype
    pub properties: Vec<Property>,
    // Ergonomic, behavior-rich
}

// Stored type (in db/stored/ or storage adapter)
#[derive(Archive, Serialize, Deserialize)]
#[rkyv(derive(Debug))]
pub struct StoredSchema {
    pub name: Box<str>,
    pub version: u32,
    pub properties: Vec<StoredProperty>,
    // Stable layout - changes trigger migration decisions
}

// Conversions (in storage adapter)
impl From<Schema> for StoredSchema { /* ... */ }
impl TryFrom<StoredSchema> for Schema { /* ... */ }
```

**Guidelines:**

1. **One `Stored*` per persisted aggregate** (not per value object)
2. **Keep conversions mechanical and co-located** in storage layer
3. **Use projections for new query patterns** (don't widen stored blobs)
4. **Keep archived compute closure-scoped** (never leak transaction-scoped borrows)
5. **Treat `Stored*` changes as migration decisions** (document format versions)

**Location:**

- `db/stored/schema.rs` - StoredSchema and conversions
- `db/stored/note.rs` - StoredNote and conversions
- Or `db/<context>_adapter.rs` if storing adapter and stored type together

### Zero-Copy Idioms (Footguns to Avoid)

- **rkyv format control**: Treat endianness/alignment/pointer-width feature choices as a persisted-format contract.
- **rkyv validation**: Use `rkyv::access` at trust boundaries (files/network/user input).
- **redb guards**: `AccessGuard` values borrow the transaction/table; do not return or store them beyond the transaction scope.
- **redb custom Value**: Implement `redb::Value` via local newtypes/wrappers when you need custom encoding.
- **moka determinism**: In tests, call `run_pending_tasks()` to ensure cache stats are consistent.

## Process Patterns

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
- Use port-based CQRS pattern (generic over storage traits, not direct database coupling)
- Use async/await consistently throughout the codebase with proper error handling
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

**Good Examples:**

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
