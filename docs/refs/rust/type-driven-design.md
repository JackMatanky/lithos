# Type-Driven Design Principles for Lithos

Sources:
- "Parse, don't validate" by Alexis King
- "Type-Driven Development in Rust" by Ruggero Rebellato

This reference distills key type-driven design principles relevant to the Lithos file-based refactor.

## Core Principle: Parse, Don't Validate

**Definition**: Parsers consume less-structured input and produce more-structured output, preserving information gained during validation in the type system.

### The Problem with Validation

```rust
// ❌ BAD: Validation throws information away
fn validate_non_empty(list: &[T]) -> Result<(), Error> {
    if list.is_empty() {
        Err(Error::Empty)
    } else {
        Ok(())  // Information lost!
    }
}

// ✅ GOOD: Parsing preserves information in types
fn parse_non_empty(list: Vec<T>) -> Result<NonEmpty<T>, Error> {
    NonEmpty::try_from(list)  // Type carries proof
}
```

**Key insight**: `validate_non_empty` checks the list but returns `()`, forcing callers to re-check. `parse_non_empty` returns a `NonEmpty<T>` type that *proves* the list is non-empty, eliminating redundant checks downstream.

### Benefits of Parsing

1. **Eliminate redundant checks**: Once parsed, downstream code can assume the invariant holds.
2. **Prevent shotgun parsing**: Validation scattered across code makes state unpredictable; parsing stratifies into parse phase and execution phase.
3. **Make illegal states unrepresentable**: If a type can't represent an invalid state, that bug can't be written.

### Practical Guidelines

1. **Use a data structure that makes illegal states unrepresentable**
   - Prefer `NonEmpty<T>` over `Vec<T>` when empty isn't valid
   - Use `BTreeMap` instead of `Vec<(K, V)>` to prevent duplicate keys
   - Model states with enums, not booleans or magic values

2. **Push proof upward as far as possible, but no further**
   - Parse at system boundaries (file I/O, HTTP, CLI args)
   - Transform data into the most precise representation early
   - Use sum types to adapt to control flow

3. **Let datatypes inform code, not vice versa**
   - Don't add a `bool` to a struct just because one function needs it
   - Refactor to use the right representation; the type system ensures completeness

4. **Treat `fn() -> Result<(), E>` with suspicion**
   - These often indicate validation instead of parsing
   - If the primary effect is raising an error, there's likely a better way

5. **Parse data in multiple passes when needed**
   - Context-sensitive parsing is fine
   - Just don't *act* on data before it's fully parsed

6. **Avoid denormalized representations, especially if mutable**
   - Duplication introduces trivially representable illegal states (out of sync)
   - Keep denormalized data behind abstraction boundaries

7. **Use abstract types to make validators "look like" parsers**
   - `newtype` with smart constructor for invariants the type system can't express
   - Example: `struct Percentage(u8)` with `new(u8) -> Result<Self, Error>` checking 0-100

## Rust-Specific Patterns for Type Safety

### 1. Ownership as Linear Types

Rust's ownership system is an affine type system (values used at most once):
- **Move semantics** prevent use-after-free and double-free
- **Borrowing rules** prevent data races (`&T` shared, `&mut T` exclusive)
- **Lifetimes** prevent dangling pointers

**Type-driven benefit**: Memory safety and thread safety are compile-time guarantees. The type system encodes resource ownership.

### 2. Algebraic Data Types (ADTs)

**Product types** (`struct`): Bundle related data
**Sum types** (`enum`): Represent choices between variants

```rust
// ❌ BAD: Boolean state is easy to misuse
struct User {
    name: String,
    logged_in: bool,
    session_token: Option<String>,
}

// ✅ GOOD: Enum makes invalid states impossible
enum User {
    LoggedOut { name: String },
    LoggedIn { name: String, session_token: String },
}
```

**Key insight**: With the enum, you *cannot* have `logged_in: false` but `session_token: Some(...)`. Pattern matching forces handling both cases.

### 3. Newtype Pattern

Wrap primitives to give them semantic meaning and prevent misuse:

```rust
struct UserId(u64);
struct OrderId(u64);

fn cancel_order(order_id: OrderId) { ... }

// Type error: can't pass UserId where OrderId expected
let user = UserId(42);
cancel_order(user);  // ❌ Compile error
```

**Type-driven benefit**: Prevents mixing up semantically different IDs. The compiler catches the mistake.

### 4. Typestate Pattern

Use phantom types to track state transitions at compile time:

```rust
struct Closed;
struct Open;

struct Connection<State> {
    _state: PhantomData<State>,
    // ... actual fields
}

impl Connection<Closed> {
    fn open(self) -> Connection<Open> { ... }
}

impl Connection<Open> {
    fn send(&self, data: &[u8]) { ... }
    fn close(self) -> Connection<Closed> { ... }
}

// ✅ Type system enforces correct usage
let conn = Connection::<Closed>::new();
// conn.send(data);  // ❌ Compile error: no `send` on Closed
let conn = conn.open();
conn.send(data);  // ✅ OK
```

**Type-driven benefit**: Illegal state transitions are compile errors, not runtime panics.

### 5. Builder Pattern with Required Fields

Use typestate to ensure required fields are set before `build()`:

```rust
struct Config<Name, Port> {
    name: Name,
    port: Port,
}

struct Unset;

impl Config<Unset, Unset> {
    fn new() -> Self { ... }
}

impl<Port> Config<Unset, Port> {
    fn name(self, name: String) -> Config<String, Port> { ... }
}

impl<Name> Config<Name, Unset> {
    fn port(self, port: u16) -> Config<Name, u16> { ... }
}

impl Config<String, u16> {
    fn build(self) -> FinalConfig { ... }
}

// ✅ Forces setting both name and port
let config = Config::new()
    .name("lithos".into())
    .port(8080)
    .build();
```

**Type-driven benefit**: `build()` only exists when all required fields are set. Forgetting a field is a compile error.

### 6. Const Generics for Compile-Time Checks

Use const generics to encode size constraints:

```rust
struct Matrix<T, const M: usize, const N: usize> {
    data: [[T; N]; M],
}

impl<T, const M: usize, const N: usize, const P: usize>
    Matrix<T, M, N>
{
    fn multiply(self, other: Matrix<T, N, P>) -> Matrix<T, M, P> {
        // N must match at compile time
    }
}
```

**Type-driven benefit**: Matrix dimension mismatches are compile errors, not runtime panics.

### 7. Zero-Cost Abstractions

Rust's monomorphization means using rich types has no runtime cost:
- Newtypes optimize to underlying type
- Generic trait bounds monomorphize to specialized code
- Phantom types have zero size

**Type-driven benefit**: Can freely use types for safety without performance penalty.

## Application to Lithos Refactor

### File-Based Source of Truth

**Before (CQRS)**:
- Events as source of truth
- Complex event sourcing pipeline
- Hard to reason about state

**After (File-Based)**:
- Files are source of truth
- Database is projection/cache
- Parsing pipeline validates once at boundary

**Type-driven approach**:

1. **Parse at file boundary**: `RawSchema` → validation → `StoredSchema`
2. **Use types to track validation state**:
   - `RawSchema`: syntax-only validation (serde-facing)
   - `Schema`: semantically validated (refs resolved, no cycles)
   - `StoredSchema`: ready for database (with metadata)

3. **Make invalid states unrepresentable**:
   - Use `SchemaId` and `SchemaName` newtypes (not bare `Uuid` or `String`)
   - Use `PropertyId` for stable identity (not property name strings)
   - Use `NonEmpty` for lists that can't be empty

4. **Push validation to boundaries**:
   - Ingestor: file → `RawSchema` (syntax)
   - Dereferencer: `RawSchema` → resolve `$ref` (semantics)
   - Resolver: inheritance → final `StoredSchema`
   - Storage: only stores validated data

5. **Eliminate shotgun validation**:
   - No scattered `if prop_exists(...)` checks
   - Parse once, pass typed values everywhere
   - If DB has `StoredSchema`, assume it's valid

### Unified Repository Pattern

**Type-driven insight**: Instead of separate Query/Command traits, use a single `Repository` trait that provides both reads and writes. The trait itself encodes the contract.

```rust
// ✅ Single trait, clear contract
pub trait Repository {
    // Reads
    fn get(&self, id: SchemaId) -> Result<Option<StoredSchema>, Error>;
    fn with_archived<F, R>(&self, id: SchemaId, f: F) -> Result<Option<R>, Error>
    where
        F: for<'a> FnOnce(&'a ArchivedSchema) -> R;

    // Writes
    fn save(&self, schema: &StoredSchema) -> Result<(), Error>;
    fn delete(&self, id: SchemaId) -> Result<(), Error>;
}
```

**Benefits**:
- Type system ensures all implementations provide the full contract
- No need to wire up separate query/command objects
- Easier to reason about (one abstraction per context)

### Property Bank as Parsed Registry

The Property Bank is a parser, not a validator:

```rust
// ❌ BAD: Validate and hope
fn validate_property_bank(bank: &RawPropertyBank) -> Result<(), Error> { ... }

// ✅ GOOD: Parse into validated representation
impl PropertyBank {
    pub fn try_from_raw(
        raw: RawPropertyBank,
        existing: Option<&Self>
    ) -> Result<Self, SchemaError> {
        // Parse once, return typed bank
        // All downstream code can assume properties are valid
    }
}
```

**Type-driven benefit**: Once you have a `PropertyBank`, it's guaranteed to be valid. No re-checking needed.

## Summary

Type-driven design in Lithos means:

1. **Parse files into typed representations at system boundaries**
2. **Use newtypes and ADTs to make illegal states unrepresentable**
3. **Let the compiler enforce invariants** (not runtime checks)
4. **Push validation as far upstream as possible** (at file ingestion)
5. **Use types to document contracts** (traits, associated types)
6. **Leverage Rust's zero-cost abstractions** (safety without overhead)

The refactor from CQRS to file-based follows these principles:
- Files → parse → validated types → database (parse, don't validate)
- Unified Repository trait (type encodes contract)
- Property Bank as parsed registry (validate once, use everywhere)
- Domain types carry proofs (SchemaId vs Uuid, PropertyId vs String)

**Result**: If it compiles, it's memory-safe, thread-safe, and respects domain invariants. Bugs are caught at compile time, not in production.
