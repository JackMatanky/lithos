# Rust Method & Function Naming Taxonomy

**Last Updated**: 2026-03-03
**Status**: Canonical Reference
**Scope**: All Rust code in lithos-core and lithos-cli

This document provides a comprehensive taxonomy of method and function naming conventions for the Lithos project, combining official Rust API Guidelines with CQRS-specific patterns for our port-based architecture.

---

## Table of Contents

1. [Core Rust Naming Conventions](#core-rust-naming-conventions)
2. [CQRS Method Naming (Query & Command Ports)](#cqrs-method-naming-query--command-ports)
3. [Conversion & Constructor Patterns](#conversion--constructor-patterns)
4. [Getter & Accessor Patterns](#getter--accessor-patterns)
5. [Iterator Naming](#iterator-naming)
6. [Boolean Predicates](#boolean-predicates)
7. [Mutability & Borrowing Indicators](#mutability--borrowing-indicators)
8. [Quick Reference Tables](#quick-reference-tables)
9. [Anti-Patterns to Avoid](#anti-patterns-to-avoid)

---

## Core Rust Naming Conventions

### Casing Rules (RFC 430)

| Item              | Convention             | Example                        |
| ----------------- | ---------------------- | ------------------------------ |
| Functions/Methods | `snake_case`           | `parse_note`, `validate_value` |
| Types/Traits      | `UpperCamelCase`       | `PropertyName`, `NoteId`       |
| Enum Variants     | `UpperCamelCase`       | `Optionality::Required`        |
| Local Variables   | `snake_case`           | `file_path`, `property_spec`   |
| Constants/Statics | `SCREAMING_SNAKE_CASE` | `MAX_SIZE`, `DEFAULT_PORT`     |
| Type Parameters   | Concise uppercase      | `T`, `K`, `V`                  |
| Lifetimes         | Short lowercase        | `'a`, `'src`, `'de`            |

**Acronym Rules**:

- `UpperCamelCase`: Treat as one word → `Uuid` not `UUID`, `Stdin` not `StdIn`
- `snake_case`: Lowercase → `is_xid_start` not `is_XID_start`
- Single letters only at end → `btree_map` not `b_tree_map`

---

## CQRS Method Naming (Query & Command Ports)

### Query Port Method Prefixes

Use these prefixes in `Query` trait definitions for read-only operations:

| Prefix          | Return Type                | Semantics                              | Examples                              |
| --------------- | -------------------------- | -------------------------------------- | ------------------------------------- |
| **`find_*`**    | `Option<T>`                | Optional single-entity lookup          | `find_by_id`, `find_by_name`          |
| **`get_*`**     | `Option<T>` or `T`         | Singleton/expected-to-exist lookup     | `get_global`, `get_property_bank`     |
| **`list_*`**    | `Vec<T>` or `Iterator`     | Enumeration of multiple entities       | `list()`, `list_name_id_pairs`        |
| **`*_many`**    | `HashMap<K,V>` (bulk I/O)  | Bulk operations in single transaction  | `find_many_by_ids`                    |
| **`are_many_*`**| `HashMap<K, bool>`         | Batch boolean queries (plural of `is_`) | `are_many_stale`                      |
| **`with_*`**    | `Option<R>` (HRTB closure) | Zero-copy closure-based access         | `with_archived`, `with_metadata`      |
| **`is_*`**      | `bool`                     | Boolean queries (staleness, existence) | `is_bank_stale`, `is_schema_stale`    |
| **`has_*`**     | `bool`                     | Possession/presence checks             | `has_name`, `has_default`             |
| **`count_*`**   | `usize`                    | Cardinality queries                    | `count_schemas` (future)              |
| **`query_*`**   | `Vec<T>` (complex)         | Multi-predicate searches               | `query_frontmatter_kv`                |
| **`cascade_*`** | Side-effect (graph ops)    | Traversal with mutations               | `cascade_staleness`                   |

**Note on `search_*` vs `find_*`**: Never use `search_*`. It is not Rust-idiomatic. Rust stdlib iterators and database crates (like SeaORM) use `find`, `filter`, and `position`.

**Note on `lookup_*`**: Do not use `lookup_*`. Consolidate index mappings and direct fetches into the `find_*` taxonomy (e.g., `find_id_by_name`).

#### `find_*` vs `get_*` Distinction

**Rule**: Use `find_*` for **optional** lookups, `get_*` for **singletons** or **expected-to-exist** cases.

```rust
// ✅ find_* for optional entity lookup
fn find_by_id(&self, id: SchemaId) -> Result<Option<Schema>, Self::Error>;
fn find_by_name(&self, name: &SchemaName) -> Result<Option<Schema>, Self::Error>;

// ✅ get_* for singleton or required lookups
fn get_global(&self) -> Result<Option<Global>, Self::Error>;           // Singleton
fn get_property_bank(&self) -> Result<Option<PropertyBank>, Self::Error>; // Singleton
fn get_vault(&self, id: VaultId) -> Result<Option<Vault>, Self::Error>; // Context-dependent
```

#### Zero-Copy `with_*` Pattern

For performance-critical paths (LSP queries, hot database reads), use closure-scoped access:

```rust
// ✅ GOOD: Closure-based zero-copy access
fn with_archived<F, R>(&self, id: SchemaId, f: F) -> Result<Option<R>, Self::Error>
where
    F: for<'a> FnOnce(&'a Archived<Schema>) -> R;

// ❌ BAD: Returning guards requires self-referential structs
fn get_archived(&self, id: SchemaId) -> Result<Option<Guard>, Self::Error>;
```

**Generic Parameter Order**: Use `<F, R>` consistently (closure first, result second).

---

### Command Port Method Verbs

Use these verbs in `Command` trait definitions for write operations:

| Verb             | Signature Pattern | Semantics                              | Examples                             |
| ---------------- | ----------------- | -------------------------------------- | ------------------------------------ |
| **`create`**     | `(params) → T`    | Insert new entity (ID generated)       | `create(&str) → Note`                |
| **`save`**       | `(&T)`            | Upsert (insert or replace, idempotent) | `save_batch(&[Schema])`              |
| **`update`**     | `(T) → T`         | Modify existing (may error)            | `update(Note) → Note`                |
| **`delete`**     | `(ID)`            | Remove entity                          | `delete(SchemaId)`                   |
| **`record_*`**   | `(&T)`            | Task-oriented write (DDD intent)       | `record_global`, `record_vault`      |
| **`activate_*`** | `→ State`         | State transition                       | `activate_version(...)`              |
| **`*_many`**     | `(&[T])`          | Bulk write operations                  | `save_many`, `delete_many`           |
| **`register_*`** | `(K, V)`          | Add to index/registry                  | `register_schema(name, id)` (future) |

#### Task-Oriented Verbs (`record_*`, `activate_*`)

**Use task-oriented verbs** when generic CRUD verbs obscure domain intent:

```rust
// ✅ GOOD: Domain-specific intent (Config context)
fn record_global(&self, config: &Global) -> Result<(), Self::Error>;
fn record_vault(&self, id: VaultId, vault: &Vault) -> Result<(), Self::Error>;
fn activate_version(&self, id: VaultId, target: ActivationTarget) -> Result<Version, Self::Error>;

// ⚠️ LESS CLEAR: Generic CRUD verbs
fn save_global(&self, config: &Global) -> Result<(), Self::Error>;
fn update_active_version(&self, id: VaultId, version: Version) -> Result<(), Self::Error>;
```

**When to use `record_*`**: Configuration management, audit logs, immutable event storage.
**When to use `activate_*`**: State machines, workflows, lifecycle transitions.

---

### Event Naming

Events follow industry-standard CQRS/DDD conventions:

| Pattern          | Example                        | Rules                           |
| ---------------- | ------------------------------ | ------------------------------- |
| **Struct name**  | `NoteCreated`, `SchemaUpdated` | PascalCase, **past tense** verb |
| **Enum variant** | `NoteEvents::NoteCreated(...)` | Wraps individual event structs  |
| **Method**       | `note.emit_created_event()`    | Returns `Vec<DomainEvent>`      |

```rust
// ✅ GOOD: Past tense, PascalCase
pub struct NoteCreated { pub id: Uuid, pub timestamp: Timestamp }
pub struct SchemaUpdated { pub id: SchemaId, pub version: Version }

pub enum NoteEvents {
    NoteCreated(NoteCreated),
    FrontmatterValidated(FrontmatterValidated),
}

// ❌ BAD: Present tense (those are commands)
pub struct CreateNote { ... }  // Wrong - this is a command, not an event
pub struct UpdateSchema { ... }
```

---

### Port & Adapter Naming

| Pattern                | Example                       | Usage                                                 |
| ---------------------- | ----------------------------- | ----------------------------------------------------- |
| **`Query` trait**      | `schema::ports::Query`        | ✅ Current standard (import as `schema_ports::Query`) |
| **`Command` trait**    | `schema::ports::Command`      | ✅ Current standard                                   |
| **`CommandState`**     | `config::ports::CommandState` | Internal read-for-write encapsulation                 |
| **Query Adapter**      | `QueryAdapter<'db>`           | ✅ Context-scoped implementation                      |
| **Command Adapter**    | `CommandAdapter<'db>`         | ✅ Context-scoped implementation                      |
| **Type Alias (Query)** | `RedbSchemaQuery<'db>`        | ✅ Ergonomic alias hiding generics                    |
| **Type Alias (Cmd)**   | `RedbSchemaCommand<'db>`      | ✅ Ergonomic alias hiding generics                    |

```rust
// ✅ GOOD: Context-scoped port traits
pub trait Query {
    type Error;
    fn find_by_id(&self, id: SchemaId) -> Result<Option<Schema>, Self::Error>;
}

pub trait Command {
    type Error;
    fn save_batch(&self, schemas: &[Schema]) -> Result<(), Self::Error>;
}

// ✅ GOOD: Context-scoped adapters
pub struct QueryAdapter<'db> { /* redb transaction */ }
impl<'db> schema::ports::Query for QueryAdapter<'db> { ... }

pub struct CommandAdapter<'db> { /* redb write transaction */ }
impl<'db> schema::ports::Command for CommandAdapter<'db> { ... }

// ✅ GOOD: Type aliases for ergonomics
pub type RedbSchemaQuery<'db> = Query<QueryAdapter<'db>>;
pub type RedbSchemaCommand<'db> = Command<CommandAdapter<'db>>;
```

---

## Conversion & Constructor Patterns

### Conversion Prefixes: `as_`, `to_`, `into_`

These prefixes follow **strict cost-based conventions**:

| Prefix      | Cost          | Ownership                     | Example                                 |
| ----------- | ------------- | ----------------------------- | --------------------------------------- |
| **`as_`**   | **Free**      | borrowed → borrowed           | `str::as_bytes()`, `Path::as_os_str()`  |
| **`to_`**   | **Expensive** | borrowed → owned (allocates)  | `str::to_lowercase()`, `Path::to_str()` |
| **`into_`** | **Variable**  | owned → owned (consumes self) | `String::into_bytes()`                  |

```rust
// ✅ GOOD: Free zero-cost view
pub fn as_bytes(&self) -> &[u8] { ... }

// ✅ GOOD: Expensive allocation
pub fn to_lowercase(&self) -> String { self.data.to_lowercase() }

// ✅ GOOD: Consumes self
pub fn into_bytes(self) -> Vec<u8> { self.data }

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
| **Fallible**                 | `fn try_new(...) -> Result<...>`  | `PropertyName::try_new(s)?`    |
| **Conversion Constructor**   | `fn from_*(...) -> Self`          | `from_raw()`, `from_bytes()`   |
| **With Config/Capacity**     | `fn with_*(...) -> Self`          | `Vec::with_capacity(100)`      |
| **I/O Resources**            | `fn open(...)`                    | `File::open()`, `Mmap::open()` |
| **Network Resources**        | `fn connect(...)`, `fn bind(...)` | `TcpStream::connect()`         |
| **Builder Pattern (fluent)** | `fn with_*(...) -> Self`          | `builder.with_name("x")`       |

```rust
// ✅ GOOD: Simple infallible constructor
impl Property {
    pub fn new(name: PropertyName, spec: PropertySpec) -> Self { ... }
}

// ✅ GOOD: Fallible constructor with validation
impl PropertyName {
    pub fn try_new(value: String) -> Result<Self, ValueError> {
        validate(&value)?;
        Ok(Self(value))
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

---

## Getter & Accessor Patterns

### Avoid `get_` Prefix on Simple Getters

**Rule**: The `get_` prefix is **not used** for simple field accessors in Rust.

```rust
// ✅ GOOD: Direct field name
pub fn name(&self) -> &PropertyName { &self.name }
pub fn name_mut(&mut self) -> &mut PropertyName { &mut self.name }

// ❌ BAD: Unnecessary get_ prefix
pub fn get_name(&self) -> &PropertyName { &self.name }
pub fn get_mut_name(&mut self) -> &mut PropertyName { &mut self.name }
```

### When to Use `get`

Use `get` **only when**:

1. Runtime validation (bounds checking)
2. Fallible access (`Option` or `Result`)

```rust
// ✅ GOOD: Runtime bounds checking warrants 'get'
fn get(&self, index: K) -> Option<&V>;
fn get_mut(&mut self, index: K) -> Option<&mut V>;
unsafe fn get_unchecked(&self, index: K) -> &V;

// ✅ GOOD: Fallible index lookup
HashMap::get(key) -> Option<&V>
Vec::get(index) -> Option<&T>
```

---

## Iterator Naming

For **homogeneous collections**:

```rust
fn iter(&self) -> Iter             // Iter implements Iterator<Item = &U>
fn iter_mut(&mut self) -> IterMut  // IterMut implements Iterator<Item = &mut U>
fn into_iter(self) -> IntoIter     // IntoIter implements Iterator<Item = U>
```

**Iterator type names match method names**:

- `iter()` → `Iter` (or `vec::Iter`, `schema::Iter`)
- `iter_mut()` → `IterMut`
- `into_iter()` → `IntoIter`
- `keys()` → `Keys`, `values()` → `Values`

**Non-homogeneous iterators**:

```rust
str::bytes() -> Bytes        // Not iter() because yields u8, not &char
str::chars() -> Chars        // Not iter() because yields char, not &u8
```

---

## Boolean Predicates

| Prefix         | Meaning               | Examples                                           |
| -------------- | --------------------- | -------------------------------------------------- |
| **`is_`**      | State/property check  | `is_required_scalar()`, `is_stale()`, `is_empty()` |
| **`has_`**     | Possession/presence   | `has_name()`, `has_default()`, `has_anchor()`      |
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

// ❌ BAD: _mut in the middle
fn as_slice_mut(&mut self) -> &mut [T] { ... }
fn mut_iter(&mut self) -> IterMut { ... }
```

### Setter Pattern (Rare in Rust)

```rust
// ⚠️ ACCEPTABLE: Explicit mutation
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

## Quick Reference Tables

### Query Port Methods

| Method Pattern           | Return Type         | Example                        |
| ------------------------ | ------------------- | ------------------------------ |
| `find_by_id(id)`         | `Option<T>`         | `find_by_id(schema_id)`        |
| `find_by_name(name)`     | `Option<T>`         | `find_by_name(&name)`          |
| `get_*`                  | `Option<T>` or `T`  | `get_property_bank()`          |
| `list()`                 | `Vec<T>`            | `list()`                       |
| `list_name_id_pairs()`   | `Vec<(K, V)>`       | `list_name_id_pairs()`         |
| `find_id_by_name(key)`   | `Option<ID>`        | `find_id_by_name(&name)`       |
| `find_many_by_ids(ids)`  | `HashMap<K, V>`     | `find_many_by_ids(&ids)`       |
| `are_many_stale(...)`    | `HashMap<ID, bool>` | `are_many_stale(&checks)`      |
| `with_archived(id, f)`   | `Option<R>`         | `with_archived(id, \|s\| ...)` |
| `is_stale(...)`          | `bool`              | `is_schema_stale(...)`         |

### Command Port Methods

| Method Pattern           | Signature           | Example                         |
| ------------------------ | ------------------- | ------------------------------- |
| `create(...)`            | `(params) → T`      | `create(&str) → Note`           |
| `save_many(items)`       | `(&[T])`            | `save_many(&schemas)`           |
| `update(item)`           | `(T) → T`           | `update(note) → Note`           |
| `delete(id)`             | `(ID)`              | `delete(schema_id)`             |
| `record_*(item)`         | `(&T)`              | `record_global(&config)`        |
| `activate_*(...)`        | `→ State`           | `activate_version(...)`         |
| `save_inheritance_many`  | `(&[Relationship])` | `save_inheritance_many(&rels)`  |

### Conversion Methods

| Pattern            | Cost      | Ownership           | Example                   |
| ------------------ | --------- | ------------------- | ------------------------- |
| `as_bytes()`       | Free      | borrowed → borrowed | `str::as_bytes()`         |
| `as_str()`         | Free      | borrowed → borrowed | `Path::as_os_str()`       |
| `to_lowercase()`   | Expensive | borrowed → owned    | `str::to_lowercase()`     |
| `to_string()`      | Expensive | borrowed → owned    | `Path::to_str()`          |
| `into_bytes()`     | Variable  | owned → owned       | `String::into_bytes()`    |
| `into_inner()`     | Variable  | wrapper → inner     | `BufReader::into_inner()` |
| `try_new(value)`   | Fallible  | -                   | `PropertyName::try_new()` |
| `from_raw(ptr)`    | Unsafe    | -                   | `Box::from_raw()`         |
| `with_capacity(n)` | Config    | -                   | `Vec::with_capacity()`    |

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

### ❌ Inconsistent Word Order

```rust
// ❌ BAD: Inconsistent with stdlib
pub struct AddrParseError;

// ✅ GOOD: Matches verb-object-error pattern
pub struct ParseAddrError;
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

### ❌ File I/O in CQRS Ports

```rust
// ❌ BAD: Violates port-based architecture
pub trait Query {
    fn load_from_file(&self, path: &Path) -> Result<Schema, Error>;
    fn scan_directory(&self, dir: &Path) -> Result<Vec<Schema>, Error>;
}

// ✅ GOOD: File I/O belongs in application services
pub trait Query {
    fn find_by_id(&self, id: SchemaId) -> Result<Option<Schema>, Error>;
}

// Application service coordinates File → Raw → Domain → DB
impl SchemaService {
    pub fn load(&self, path: &Path) -> Result<Schema, Error> {
        let raw = self.file_source.read(path)?;
        let schema = Schema::try_from(raw)?;
        self.command.save_batch(&[schema])?;
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

### CQRS-Specific

11. **Split ports**: Query (read-only) and Command (write) are separate traits
12. **Query prefixes**: `find_*`, `get_*`, `list_*`, `*_many`, `with_*`, `is_*`, `are_many_*`
13. **Command verbs**: `create`, `save`, `update`, `delete`, `record_*`, `activate_*`, `*_many`
14. **Task-oriented verbs**: Use `record_*`, `activate_*` when CRUD obscures intent
15. **Events are past tense**: `NoteCreated`, not `CreateNote`

### Mutation & Borrowing

16. **`_mut` at the end**: `iter_mut()`, `as_mut_slice()`, `get_unchecked_mut()`
17. **Setters are rare**: Prefer immutability + construction-time validation
18. **Builder methods**: Use `with_` prefix for fluent building

---

## References

- **Rust API Guidelines**: https://rust-lang.github.io/api-guidelines/naming.html
- **RFC 430** (Naming Conventions): https://github.com/rust-lang/rfcs/blob/master/text/0430-finalizing-naming-conventions.md
- **RFC 199** (Ownership Variants): https://github.com/rust-lang/rfcs/blob/master/text/0199-ownership-variants.md
- **Lithos ADR 002**: Port-Based CQRS Architecture
- **Lithos ADR 004**: Minimal Event Foundation

---

## Changelog

- **2026-03-03**: Initial taxonomy created from Rust API Guidelines + CQRS research
