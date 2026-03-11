# Rust Method & Function Naming Taxonomy

**Last Updated**: 2026-03-11
**Status**: Canonical Reference
**Scope**: All Rust code in lithos-core and lithos-cli

This document provides a comprehensive taxonomy of method and function naming conventions for the Lithos project, based on official Rust API Guidelines, standard library patterns, and Repository pattern best practices.

---

## Table of Contents

**Organized from most general (universal Rust) to least general (project-specific):**

1. [Core Rust Naming Conventions](#core-rust-naming-conventions) — Universal (RFC 430)
2. [Conversion & Constructor Patterns](#conversion--constructor-patterns) — Universal (API Guidelines)
3. [Getter & Accessor Patterns](#getter--accessor-patterns) — Universal (API Guidelines)
4. [Iterator Naming](#iterator-naming) — Universal (API Guidelines)
5. [Boolean Predicates](#boolean-predicates) — Universal (stdlib patterns)
6. [Mutability & Borrowing Indicators](#mutability--borrowing-indicators) — Universal (language feature)
7. [Error Handling Patterns](#error-handling-patterns) — Universal (stdlib patterns)
8. [Parse vs Validate Naming](#parse-vs-validate-naming) — Common pattern (type-driven design)
9. [Repository Pattern Method Naming](#repository-pattern-method-naming) — Project-specific (Lithos)
10. [Quick Reference Tables](#quick-reference-tables) — Summary
11. [Anti-Patterns to Avoid](#anti-patterns-to-avoid) — Summary

---

## Core Rust Naming Conventions

### Casing Rules (RFC 430)

| Item              | Convention             | Example                        |
| ----------------- | ---------------------- | ------------------------------ |
| Crates/Packages   | `kebab-case`           | `lithos-core`, `serde-json`    |
| Modules           | `snake_case`           | `note`, `schema`, `loader`     |
| Functions/Methods | `snake_case`           | `parse_schema`, `try_new`      |
| Types/Traits      | `UpperCamelCase`       | `SchemaName`, `PropertyId`     |
| Enum Variants     | `UpperCamelCase`       | `Optionality::Required`        |
| Local Variables   | `snake_case`           | `file_path`, `property_spec`   |
| Constants/Statics | `SCREAMING_SNAKE_CASE` | `MAX_DEPTH`, `DEFAULT_TIMEOUT` |
| Type Parameters   | Concise uppercase      | `T`, `K`, `V`, `E`             |
| Lifetimes         | Short lowercase        | `'a`, `'db`, `'de`, `'src`     |

**Acronym Rules**:

- `UpperCamelCase`: Treat as one word → `Uuid` not `UUID`, `HttpRequest` not `HTTPRequest`
- `snake_case`: Lowercase → `is_xid_start` not `is_XID_start`
- Single letters only at end → `btree_map` not `b_tree_map`

---

## Repository Pattern Method Naming

The Repository pattern provides data access abstraction. Use these method names in `Repository` trait definitions:

### Read Operations

| Method            | Return Type            | Semantics                            | Examples                         |
| ----------------- | ---------------------- | ------------------------------------ | -------------------------------- |
| **`find_*`**      | `Result<Option<T>, E>` | Optional entity lookup               | `find_by_id`, `find_by_name`     |
| **`get`**         | `Result<Option<T>, E>` | Fallible lookup with bounds checking | `Vec::get`, `HashMap::get`       |
| **`list`**        | `Result<Vec<T>, E>`    | Enumerate all or filtered set        | `list()`, `list_by_parent`       |
| **`find_many_*`** | `Result<HashMap<K,V>>` | Bulk lookup by keys                  | `find_many_by_ids`               |
| **`with_*`**      | `Result<Option<R>, E>` | Zero-copy closure-based access       | `with_archived`, `with_metadata` |
| **`is_*`**        | `Result<bool, E>`      | Boolean queries                      | `is_stale`, `is_empty`           |
| **`has_*`**       | `Result<bool, E>`      | Existence checks                     | `has_parent`, `has_default`      |
| **`count_*`**     | `Result<usize, E>`     | Cardinality queries                  | `count()`, `count_by_status`     |
| **`exists`**      | `Result<bool, E>`      | Existence check                      | `exists(id)`                     |

#### `find_*` vs `get` Distinction

**Rule**: Use `find_*` for **entity lookup**, `get` for **collection access with bounds checking**.

```rust
// ✅ find_* for entity lookup (repository pattern)
fn find_by_id(&self, id: SchemaId) -> Result<Option<Schema>, Self::Error>;
fn find_by_name(&self, name: &SchemaName) -> Result<Option<Schema>, Self::Error>;

// ✅ get for collection access (HashMap, Vec pattern)
fn get(&self, key: &K) -> Option<&V>;  // Like HashMap::get
Vec::get(index) -> Option<&T>           // Like Vec::get
```

#### Zero-Copy `with_*` Pattern

For performance-critical paths (LSP queries, hot database reads), use closure-scoped access:

```rust
// ✅ GOOD: Closure-based zero-copy access
fn with_archived<F, R>(&self, id: SchemaId, f: F) -> Result<Option<R>, Self::Error>
where
    F: for<'a> FnOnce(&'a Archived<Schema>) -> R;

// Usage
storage.with_archived(id, |archived| {
    archived.name()  // Zero-copy access
})?;

// ❌ BAD: Returning guards requires self-referential structs
fn get_archived(&self, id: SchemaId) -> Result<Option<Guard>, Self::Error>;
```

**Generic Parameter Order**: Use `<F, R>` consistently (closure first, result second).

---

### Write Operations

| Method            | Signature Pattern       | Semantics                      | Examples                 |
| ----------------- | ----------------------- | ------------------------------ | ------------------------ |
| **`save`**        | `(&T) -> Result<(), E>` | Upsert single entity           | `save(&schema)`          |
| **`save_many`**   | `(&[T]) -> Result<>`    | Bulk upsert atomically         | `save_many(&schemas)`    |
| **`insert`**      | `(K, V) -> Option<V>`   | Add to collection, return old  | `HashMap::insert`        |
| **`update`**      | `(T) -> Result<T, E>`   | Modify existing (may error)    | `update(schema)`         |
| **`delete`**      | `(ID) -> Result<(), E>` | Remove single entity           | `delete(schema_id)`      |
| **`delete_many`** | `(&[ID]) -> Result<>`   | Bulk remove atomically         | `delete_many(&ids)`      |
| **`remove`**      | `(K) -> Option<V>`      | Remove from collection, return | `HashMap::remove`        |
| **`create`**      | `(params) -> Result<T>` | Create new (ID generated)      | `create(name) -> Schema` |

**save vs insert vs create:**

- `save`: Repository pattern, upsert semantics (create or update)
- `insert`: Collection pattern, replaces existing (returns old value)
- `create`: Explicit creation (may error if exists)

```rust
// ✅ Repository pattern
pub trait Repository {
    fn save(&self, entity: &Schema) -> Result<(), Self::Error>;
    fn delete(&self, id: SchemaId) -> Result<(), Self::Error>;
}

// ✅ Collection pattern (HashMap, BTreeMap)
fn insert(&mut self, key: K, value: V) -> Option<V>;
fn remove(&mut self, key: &K) -> Option<V>;
```

---

## Parse vs Validate Naming

**Critical Distinction**: Parsing returns validated types, validation returns `Result<(), E>`.

### Parse Methods (Preferred)

| Pattern        | Signature                    | Usage                          | Examples                        |
| -------------- | ---------------------------- | ------------------------------ | ------------------------------- |
| **`parse`**    | `(input) -> Result<Self, E>` | Parse string to validated type | `SchemaId::parse(s)`            |
| **`try_new`**  | `(input) -> Result<Self, E>` | Validated constructor          | `SchemaName::try_new(s)`        |
| **`from_str`** | `(&str) -> Result<Self, E>`  | FromStr trait                  | `IpAddr::from_str("127.0.0.1")` |
| **`try_from`** | `(T) -> Result<Self, E>`     | TryFrom trait                  | `Schema::try_from(raw_schema)`  |

### Validate Methods (Avoid - Throws Away Info)

```rust
// ❌ BAD: Validation throws away information
fn validate_schema_name(s: &str) -> Result<(), SchemaError> {
    if !is_valid_identifier(s) {
        return Err(SchemaError::InvalidName);
    }
    Ok(())  // Information lost! Need to check again later.
}

// ✅ GOOD: Parsing preserves information in types
impl SchemaName {
    pub fn try_new(s: &str) -> Result<Self, SchemaError> {
        if !is_valid_identifier(s) {
            return Err(SchemaError::InvalidName);
        }
        Ok(Self(s.into()))  // Type carries proof of validity
    }
}

// ✅ GOOD: TryFrom is parsing, not validation
impl TryFrom<RawSchema> for Schema {
    type Error = SchemaError;
    fn try_from(raw: RawSchema) -> Result<Self, SchemaError> {
        Ok(Schema {
            name: SchemaName::try_new(&raw.name)?,  // Parse to stronger type
            properties: parse_properties(raw.properties)?,
        })
    }
}
```

**Key Principle**: Once you have a parsed type (e.g., `SchemaName`), it's guaranteed valid. No re-checking needed.

---

## Conversion & Constructor Patterns

### Conversion Prefixes: `as_`, `to_`, `into_`

These prefixes follow **strict cost-based conventions** (Rust API Guidelines C-CONV):

| Prefix      | Cost          | Ownership                     | Example                                 |
| ----------- | ------------- | ----------------------------- | --------------------------------------- |
| **`as_`**   | **Free**      | borrowed → borrowed           | `str::as_bytes()`, `Path::as_os_str()`  |
| **`to_`**   | **Expensive** | borrowed → owned (allocates)  | `str::to_lowercase()`, `Path::to_str()` |
| **`into_`** | **Variable**  | owned → owned (consumes self) | `String::into_bytes()`                  |

```rust
// ✅ GOOD: Free zero-cost view
pub fn as_bytes(&self) -> &[u8] { ... }
pub fn as_str(&self) -> &str { &self.0 }

// ✅ GOOD: Expensive allocation
pub fn to_lowercase(&self) -> String { self.data.to_lowercase() }
pub fn to_string(&self) -> String { self.data.clone() }

// ✅ GOOD: Consumes self
pub fn into_bytes(self) -> Vec<u8> { self.data }
pub fn into_inner(self) -> T { self.0 }

// ❌ BAD: 'as_' but allocates
pub fn as_lowercase(&self) -> String { self.data.to_lowercase() }

// ❌ BAD: 'into_' but doesn't consume
pub fn into_string(&self) -> String { self.data.clone() }
```

**Special Case: `into_inner()`**

Unwrap single-value wrappers:

```rust
BufReader::into_inner() -> R
Arc::into_inner() -> Option<T>  // Only if refcount == 1
Mutex::into_inner() -> T
```

---

### Constructor Naming

| Pattern                      | Signature                         | Example                        |
| ---------------------------- | --------------------------------- | ------------------------------ |
| **Infallible**               | `fn new(...) -> Self`             | `Property::new(name, spec)`    |
| **Fallible**                 | `fn try_new(...) -> Result<...>`  | `SchemaName::try_new(s)?`      |
| **Parsing Constructor**      | `fn parse(...) -> Result<...>`    | `SchemaId::parse(s)?`          |
| **Conversion Constructor**   | `fn from_*(...) -> Self`          | `from_raw()`, `from_bytes()`   |
| **With Config/Capacity**     | `fn with_*(...) -> Self`          | `Vec::with_capacity(100)`      |
| **I/O Resources**            | `fn open(...) -> Result<...>`     | `File::open()`, `Mmap::open()` |
| **Network Resources**        | `fn connect(...)`, `fn bind(...)` | `TcpStream::connect()`         |
| **Builder Pattern (fluent)** | `fn with_*(...) -> Self`          | `builder.with_name("x")`       |

```rust
// ✅ GOOD: Simple infallible constructor
impl Property {
    pub fn new(name: PropertyName, spec: PropertySpec) -> Self { ... }
}

// ✅ GOOD: Fallible constructor with validation
impl SchemaName {
    pub fn try_new(value: String) -> Result<Self, ValueError> {
        validate(&value)?;
        Ok(Self(value.into_boxed_str()))
    }
}

// ✅ GOOD: Parser constructor
impl SchemaId {
    pub fn parse(s: &str) -> Result<Self, ParseError> {
        Uuid::parse_str(s).map(Self)
    }
}

// ✅ GOOD: Builder pattern
impl SchemaBuilder {
    pub fn with_name(mut self, name: SchemaName) -> Self {
        self.name = Some(name);
        self
    }
}
```

**Default vs new():**

- Both `Default` and `new()` should exist when appropriate
- They should produce equivalent values
- `new()` is more discoverable; `Default` enables generic code

---

## Getter & Accessor Patterns

### Avoid `get_` Prefix on Simple Getters (C-GETTER)

**Rule**: The `get_` prefix is **not used** for simple field accessors in Rust.

```rust
// ✅ GOOD: Direct field name
pub fn name(&self) -> &SchemaName { &self.name }
pub fn name_mut(&mut self) -> &mut SchemaName { &mut self.name }
pub fn id(&self) -> SchemaId { self.id }

// ❌ BAD: Unnecessary get_ prefix
pub fn get_name(&self) -> &SchemaName { &self.name }
pub fn get_id(&self) -> SchemaId { self.id }
```

### When to Use `get`

Use `get` **only when**:

1. **Runtime validation** (bounds checking)
2. **Fallible access** (`Option` or `Result`)

```rust
// ✅ GOOD: Runtime bounds checking warrants 'get'
Vec::get(index) -> Option<&T>           // Bounds checking
HashMap::get(key) -> Option<&V>         // Key lookup
slice::get(index) -> Option<&T>         // Bounds checking
slice::get_unchecked(index) -> &T       // Unsafe, no checking
```

---

## Iterator Naming (C-ITER)

For **homogeneous collections**:

```rust
fn iter(&self) -> Iter             // Iterator<Item = &U>
fn iter_mut(&mut self) -> IterMut  // Iterator<Item = &mut U>
fn into_iter(self) -> IntoIter     // Iterator<Item = U>
```

**Iterator type names match method names (C-ITER-TY)**:

```rust
// ✅ GOOD: Consistent naming
Vec::iter() -> slice::Iter<T>
Vec::iter_mut() -> slice::IterMut<T>
Vec::into_iter() -> vec::IntoIter<T>
BTreeMap::keys() -> btree_map::Keys<K, V>
BTreeMap::values() -> btree_map::Values<K, V>
```

**Non-homogeneous iterators use descriptive names**:

```rust
str::bytes() -> Bytes       // Not iter() - yields u8, not char
str::chars() -> Chars       // Not iter() - yields char
str::lines() -> Lines       // Not iter() - yields &str
```

---

## Boolean Predicates

| Prefix         | Meaning               | Examples                                           |
| -------------- | --------------------- | -------------------------------------------------- |
| **`is_`**      | State/property check  | `is_required_scalar()`, `is_stale()`, `is_empty()` |
| **`has_`**     | Possession/presence   | `has_name()`, `has_default()`, `has_parent()`      |
| **`can_`**     | Capability/permission | `can_read()`, `can_write()`                        |
| **`should_`**  | Recommendation/policy | `should_promote()`, `should_retry()`               |
| **`contains`** | Membership            | `contains_key()`, `contains(&item)`                |

```rust
// ✅ GOOD: Boolean predicates
fn is_required_scalar(&self) -> bool { ... }
fn has_default(&self) -> bool { self.default.is_some() }
fn can_write(&self) -> bool { self.permissions.writable }
fn should_retry(&self) -> bool { self.attempt_count < MAX_ATTEMPTS }
fn contains_key(&self, key: &K) -> bool { self.map.contains_key(key) }
```

**Return type**: Always `bool` for predicates.

---

## Mutability & Borrowing Indicators

### `_mut` Suffix Placement

**Rule**: Always at the **end** of the base name, matching the type signature.

```rust
// ✅ GOOD: _mut at the end (matches &mut [T])
fn as_mut_slice(&mut self) -> &mut [T] { ... }
fn iter_mut(&mut self) -> IterMut { ... }
fn name_mut(&mut self) -> &mut PropertyName { ... }
fn get_mut(&mut self, key: &K) -> Option<&mut V> { ... }

// ❌ BAD: _mut in the middle
fn as_slice_mut(&mut self) -> &mut [T] { ... }
fn mut_iter(&mut self) -> IterMut { ... }
```

### Setter Pattern (Rare in Rust)

```rust
// ⚠️ ACCEPTABLE: Explicit mutation (rare)
pub fn set_path(&mut self, path: NotePath) { self.path = path; }

// ✅ PREFER: Builder pattern for construction
pub fn with_path(mut self, path: NotePath) -> Self {
    self.path = path;
    self
}

// ✅ BEST: Immutability + validation at construction
impl Note {
    pub fn new(path: NotePath, ...) -> Result<Self, Error> { ... }
}
```

---

## Error Handling Patterns

### Try Patterns

```rust
// Fallible construction
try_new() -> Result<Self, Error>
try_from() -> Result<Self, Error>     // TryFrom trait
try_into() -> Result<U, Error>        // TryInto trait

// Fallible operations
try_reserve(n) -> Result<(), Error>
try_lock() -> Result<Guard, Error>
try_recv() -> Result<T, TryRecvError>
```

### Unwrap Variants

```rust
// Panicking (avoid in production)
unwrap() -> T                          // Panic with default message
expect(msg) -> T                       // Panic with custom message

// Non-panicking (prefer these)
unwrap_or(default: T) -> T            // Provide fallback
unwrap_or_default() -> T              // Use T::default()
unwrap_or_else(f: FnOnce() -> T) -> T // Compute fallback
ok_or(err: E) -> Result<T, E>         // Option to Result
ok_or_else(f: FnOnce() -> E) -> Result<T, E>

// Result-specific
unwrap_err() -> E                      // Panic if Ok
expect_err(msg) -> E                   // Panic if Ok with message
```

### Error Type Naming

**Pattern: Verb + Object + Error** (consistent word order)

```rust
// ✅ GOOD: Consistent verb-object-error order
ParseBoolError
ParseIntError
ParseFloatError
JoinPathsError
StripPrefixError
RecvTimeoutError

// ❌ BAD: Inconsistent order (stdlib legacy)
AddrParseError  // Should be ParseAddrError
```

**Error trait implementation:**

```rust
#[derive(Debug, thiserror::Error)]
pub enum SchemaError {
    #[error("schema not found: {0}")]
    NotFound(SchemaId),

    #[error("circular dependency detected: {0}")]
    CircularDependency(String),
}
```

---

## Quick Reference Tables

### Repository Pattern Methods

| Method Pattern          | Return Type             | Example                        |
| ----------------------- | ----------------------- | ------------------------------ |
| `find_by_id(id)`        | `Result<Option<T>, E>`  | `find_by_id(schema_id)`        |
| `find_by_name(name)`    | `Result<Option<T>, E>`  | `find_by_name(&name)`          |
| `list()`                | `Result<Vec<T>, E>`     | `list()`                       |
| `list_by_parent(id)`    | `Result<Vec<T>, E>`     | `list_by_parent(parent_id)`    |
| `find_many_by_ids(ids)` | `Result<HashMap<K,V>>>` | `find_many_by_ids(&ids)`       |
| `with_archived(id, f)`  | `Result<Option<R>, E>`  | `with_archived(id, \|s\| ...)` |
| `save(entity)`          | `Result<(), E>`         | `save(&schema)`                |
| `save_many(entities)`   | `Result<(), E>`         | `save_many(&schemas)`          |
| `delete(id)`            | `Result<(), E>`         | `delete(schema_id)`            |
| `delete_many(ids)`      | `Result<(), E>`         | `delete_many(&ids)`            |
| `is_stale(...)`         | `Result<bool, E>`       | `is_stale(id)`                 |
| `exists(id)`            | `Result<bool, E>`       | `exists(id)`                   |

### Conversion Methods

| Pattern            | Cost      | Ownership           | Example                   |
| ------------------ | --------- | ------------------- | ------------------------- |
| `as_bytes()`       | Free      | borrowed → borrowed | `str::as_bytes()`         |
| `as_str()`         | Free      | borrowed → borrowed | `Path::as_os_str()`       |
| `to_lowercase()`   | Expensive | borrowed → owned    | `str::to_lowercase()`     |
| `to_string()`      | Expensive | borrowed → owned    | `Path::to_str()`          |
| `into_bytes()`     | Variable  | owned → owned       | `String::into_bytes()`    |
| `into_inner()`     | Variable  | wrapper → inner     | `BufReader::into_inner()` |
| `try_new(value)`   | Fallible  | -                   | `SchemaName::try_new()`   |
| `parse(s)`         | Fallible  | -                   | `SchemaId::parse()`       |
| `with_capacity(n)` | Config    | -                   | `Vec::with_capacity()`    |

### Constructor Patterns

| Pattern     | Signature                        | Example                 |
| ----------- | -------------------------------- | ----------------------- |
| `new()`     | `fn new() -> Self`               | `Property::new()`       |
| `try_new()` | `fn try_new() -> Result<Self>`   | `SchemaName::try_new()` |
| `parse()`   | `fn parse(&str) -> Result<Self>` | `SchemaId::parse()`     |
| `from_*()`  | `fn from_raw() -> Self`          | `Schema::from_raw()`    |
| `with_*()`  | `fn with_capacity() -> Self`     | `Vec::with_capacity()`  |
| `open()`    | `fn open(Path) -> Result<Self>`  | `File::open()`          |
| `connect()` | `fn connect() -> Result<Self>`   | `TcpStream::connect()`  |

---

## Anti-Patterns to Avoid

### ❌ `get_` on Simple Getters

```rust
// ❌ BAD
pub fn get_name(&self) -> &str { &self.name }

// ✅ GOOD
pub fn name(&self) -> &str { &self.name }
```

### ❌ Unclear Conversion Costs

```rust
// ❌ BAD: 'as_' implies free, but lowercase() allocates
pub fn as_lowercase(&self) -> String { self.data.to_lowercase() }

// ✅ GOOD: 'to_' indicates expensive operation
pub fn to_lowercase(&self) -> String { self.data.to_lowercase() }
```

### ❌ Wrong `into_` Usage

```rust
// ❌ BAD: 'into_' but doesn't consume
pub fn into_string(&self) -> String { self.data.clone() }

// ✅ GOOD: 'to_' for borrowed → owned
pub fn to_string(&self) -> String { self.data.clone() }

// ✅ GOOD: 'into_' consumes self
pub fn into_string(self) -> String { self.data }
```

### ❌ `_mut` in Wrong Position

```rust
// ❌ BAD
fn as_slice_mut(&mut self) -> &mut [T] { ... }
fn mut_iter(&mut self) -> IterMut { ... }

// ✅ GOOD
fn as_mut_slice(&mut self) -> &mut [T] { ... }
fn iter_mut(&mut self) -> IterMut { ... }
```

### ❌ Validation Instead of Parsing

```rust
// ❌ BAD: Throws away information
pub fn validate_name(s: &str) -> Result<(), Error> {
    if !is_valid(s) { return Err(...); }
    Ok(())  // Information lost!
}

// ✅ GOOD: Parsing preserves information
pub fn parse_name(s: &str) -> Result<Name, Error> {
    if !is_valid(s) { return Err(...); }
    Ok(Name(s.into()))  // Type carries proof
}
```

### ❌ Repository Methods with File I/O

```rust
// ❌ BAD: Repository shouldn't do file I/O
pub trait Repository {
    fn load_from_file(&self, path: &Path) -> Result<Schema, Error>;
}

// ✅ GOOD: Separate file ingestion from repository
pub struct SchemaLoader;
impl SchemaLoader {
    pub fn load(&self, path: &Path) -> Result<Schema, Error> {
        let raw = fs::read_to_string(path)?;
        let schema = Schema::parse(&raw)?;
        self.repository.save(&schema)?;
        Ok(schema)
    }
}
```

---

## Best Practices Summary

### Clarity & Consistency

1. **Boring names win**: Prefer `parse_schema()` over `schematize()`
2. **Consistent word order**: Pick a pattern and stick to it
3. **Match stdlib**: When similar functionality exists, follow its naming
4. **Avoid abbreviations**: `configuration` not `cfg` (except established: `db`, `ctx`)

### Ownership & Cost Transparency

5. **Prefix indicates cost**: `as_` (free), `to_` (expensive), `into_` (consumes)
6. **Getters are free**: If there's runtime cost, use `get()` or a verb
7. **No hidden allocations**: Name should indicate when work happens

### Type-Driven Design

8. **Methods over functions**: If there's a clear receiver, make it a method
9. **Predicates return bool**: `is_`, `has_`, `can_`, `should_` → `bool`
10. **Fallible operations**: Use `try_` prefix or `Result`
11. **Parse, don't validate**: Return validated types, not `Result<(), E>`

### Repository Pattern

12. **Read operations**: `find_*`, `list`, `with_*`, `is_*`, `has_*`, `count_*`
13. **Write operations**: `save`, `save_many`, `delete`, `delete_many`
14. **find vs get**: `find_*` for entity lookup, `get` for collection access
15. **Zero-copy access**: Use `with_*` closure pattern for performance

### Mutation & Borrowing

16. **`_mut` at the end**: `iter_mut()`, `as_mut_slice()`, `get_unchecked_mut()`
17. **Setters are rare**: Prefer immutability + construction-time validation
18. **Builder methods**: Use `with_` prefix for fluent building

---

## References

- **Rust API Guidelines**: https://rust-lang.github.io/api-guidelines/naming.html
- **RFC 430** (Naming Conventions): https://github.com/rust-lang/rfcs/blob/master/text/0430-finalizing-naming-conventions.md
- **RFC 199** (Ownership Variants): https://github.com/rust-lang/rfcs/blob/master/text/0199-ownership-variants.md
- **Type-Driven Design Reference**: [type-driven-design.md](./type-driven-design.md)
- **Lithos ADR 002**: Storage Pattern (Repository)
- **Lithos ADR 003**: Serialization Strategy

---

## Changelog

- **2026-03-11**: Updated to Repository pattern, removed CQRS, added parse vs validate
- **2026-03-03**: Initial taxonomy created from Rust API Guidelines
