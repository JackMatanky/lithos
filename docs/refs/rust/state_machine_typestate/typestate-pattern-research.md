# Rust Typestate Pattern: Best Practices Research

**Research Date:** March 26, 2026
**Sources:** Official Rust docs, authoritative blog posts, production code examples

---

## Executive Summary

The typestate pattern encodes runtime state machine logic into the type system, enabling compile-time verification of state transitions. In Rust, this pattern leverages **move semantics** and **phantom types** to prevent illegal state transitions from compiling. This research synthesizes best practices from authoritative sources including the Embedded Rust Book, established Rust community blogs, and production libraries like serde and diesel.

---

## 1. API Design Patterns for Typestate Machines

### 1.1 Core Principles

The typestate pattern requires three components:

1. **Operations available only in certain states** - methods defined only on specific state types
2. **Type-level state encoding** - compile-time representation of runtime states
3. **State transition operations** - methods that consume one state and produce another

**Source:** Cliffle's "The Typestate Pattern in Rust" (2019)

### 1.2 Two Primary Encoding Approaches

#### Approach A: Separate Struct Per State

**Best for:** Simple state machines with 2-4 states, minimal shared data

```rust
struct ReadingFile { inner: File }
struct EofFile { inner: File }

impl ReadingFile {
    pub fn read(self) -> ReadResult {
        match self.inner.read() {
            Some(bytes) => ReadResult::Read(self, bytes),
            None => ReadResult::Eof(EofFile { inner: self.inner })
        }
    }
}

impl EofFile {
    pub fn close(self) { self.inner.close(); }
}
```

**Pros:**
- Crystal clear - each state is an obvious type
- Easy to add state-specific fields
- Natural Rust idioms

**Cons:**
- Scattered documentation (one rustdoc page per type)
- More boilerplate for many states
- Harder to implement operations valid in multiple states

**Source:** Will Crichton, "Type-Driven API Design in Rust"

#### Approach B: Generic Struct with State Type Parameter

**Best for:** Complex state machines with 5+ states, many cross-state operations

```rust
struct State<S> {
    inner: File,
    _marker: PhantomData<S>
}

struct Reading;
struct Eof;

impl State<Reading> {
    pub fn read(self) -> ReadResult { /* ... */ }
}

impl State<Eof> {
    pub fn close(self) { /* ... */ }
}

// Operations valid in ALL states
impl<S> State<S> {
    pub fn bytes_so_far(&self) -> usize { /* ... */ }
}

// Operations valid in SUBSET of states
trait SendingState {}
impl SendingState for Reading {}

impl<S: SendingState> State<S> {
    pub fn flush(&mut self) { /* ... */ }
}
```

**Pros:**
- Single rustdoc page shows all states and transitions
- Easy to add operations valid in multiple states
- Less duplication of shared infrastructure

**Cons:**
- Requires `PhantomData` (compiler noise)
- More complex for beginners
- Type constraints can get hairy

**Source:** Cliffle (2019), Yoric "Typestates in Rust" (2018)

### 1.3 State-Specific Data Storage

States can carry different data by storing the state marker as a **concrete type** (not phantom):

```rust
struct State<S> {
    inner: File,
    extra: S  // NOT PhantomData<S>
}

struct Reading;  // No fields
struct Headers { response_code: u8 }  // Has field

impl State<Reading> {
    fn send_status(self, code: u8) -> State<Headers> {
        State {
            inner: self.inner,
            extra: Headers { response_code: code }
        }
    }
}

impl State<Headers> {
    fn response_code(&self) -> u8 {
        self.extra.response_code  // Available only in Headers state
    }
}
```

**Benefits:**
- Eliminates `Option<T>` wrapper overhead
- Makes data availability compiler-checked
- Reduces memory footprint in early states

**Source:** Cliffle (2019)

---

## 2. Branching/Control Flow Approaches

### 2.1 The Challenge

Typestate machines struggle with branching because different branches produce different types:

```rust
let mut conn = Connection::new();  // Type: Connection<Disconnected>

if needs_ssl {
    conn = conn.start_tls();  // Type: Connection<Encrypted>
} else {
    conn = conn.start_plain();  // Type: Connection<Plain>
}

// ERROR: conn has different types in different branches!
```

### 2.2 Solution Patterns

#### Pattern 1: Enums for Branch Results

**Best for:** Limited number of outcomes (2-3), different operations per branch

```rust
enum Connected {
    Encrypted(Connection<Encrypted>),
    Plain(Connection<Plain>)
}

fn connect(use_ssl: bool) -> Connected {
    let conn = Connection::new();
    if use_ssl {
        Connected::Encrypted(conn.start_tls())
    } else {
        Connected::Plain(conn.start_plain())
    }
}

// Caller handles via match
match connect(true) {
    Connected::Encrypted(c) => c.send_encrypted(data),
    Connected::Plain(c) => c.send_plain(data),
}
```

**Source:** Discussion on Rust Users Forum "Typestate pattern and branching" (2024)

#### Pattern 2: Trait Objects for Common Interface

**Best for:** Many branches that share common operations, runtime dispatch acceptable

```rust
trait Connection {
    fn send(&mut self, data: &[u8]);
}

impl Connection for TlsConnection { /* ... */ }
impl Connection for PlainConnection { /* ... */ }

fn connect(use_ssl: bool) -> Box<dyn Connection> {
    if use_ssl {
        Box::new(TlsConnection::new())
    } else {
        Box::new(PlainConnection::new())
    }
}
```

**Tradeoff:** Loses compile-time typestate guarantees for dynamic dispatch convenience.

#### Pattern 3: Closure-Based State Machines

**Best for:** Complex decision trees, async workflows

```rust
impl Builder {
    pub fn build<F>(self, configurator: F) -> Result<Built, Error>
    where
        F: FnOnce(Configuring) -> Result<Configured, Error>
    {
        let configuring = Configuring { /* ... */ };
        let configured = configurator(configuring)?;
        Ok(Built { inner: configured.finalize() })
    }
}

// Usage
let built = Builder::new().build(|mut cfg| {
    if some_condition {
        cfg.enable_feature_a();
    } else {
        cfg.enable_feature_b();
    }
    Ok(cfg.finish())
})?;
```

**Source:** Common pattern in hyper, reqwest builders

#### Pattern 4: State Converters to Common Type

**Best for:** Branches that ultimately produce same final state

```rust
trait IntoReady {
    fn into_ready(self) -> Ready;
}

impl IntoReady for Encrypted { /* ... */ }
impl IntoReady for Plain { /* ... */ }

fn setup(use_ssl: bool) -> Ready {
    if use_ssl {
        Encrypted::new().into_ready()
    } else {
        Plain::new().into_ready()
    }
}
```

---

## 3. Ergonomics vs Safety Tradeoffs

### 3.1 Method Chaining Ergonomics

**Consuming self vs &mut self:**

```rust
// Consuming self - maximal safety, awkward loops
impl Builder<NeedsFoo> {
    pub fn set_foo(self, val: Foo) -> Builder<HasFoo> { /* ... */ }
}

// Usage in loop is annoying
let mut builder = Builder::new();
for item in items {
    builder = builder.add_item(item);  // Must reassign!
}

// &mut self - less safe, ergonomic loops
impl Builder<HasFoo> {
    pub fn add_bar(&mut self, val: Bar) { /* ... */ }
}

// Usage in loop is natural
let mut builder = Builder::new().set_foo(foo);
for item in items {
    builder.add_bar(item);  // No reassignment
}
```

**Guidelines:**
- State *transitions* should consume `self` (safety critical)
- Within-state *modifications* can use `&mut self` (ergonomics)
- Return `self` from `&mut self` methods to enable optional chaining

**Source:** Cliffle (2019), common pattern in std::fs, diesel query builders

### 3.2 Stored State Machines

**The Problem:** Typestates can't be stored in structs if states change:

```rust
struct App<S> {
    connection: Connection<S>  // S is fixed at App construction
}

// ERROR: Can't change S after App is created!
impl<S> App<S> {
    fn reconnect(&mut self) {
        self.connection = Connection::new();  // Different S!
    }
}
```

**Solutions:**

**A. Enum Wrapper (recommended for stored typestates)**

```rust
enum ConnectionState {
    Disconnected(Connection<Disconnected>),
    Connected(Connection<Connected>),
}

struct App {
    connection: ConnectionState  // Can transition
}

impl App {
    fn reconnect(&mut self) {
        self.connection = match std::mem::take(&mut self.connection) {
            ConnectionState::Disconnected(c) => {
                ConnectionState::Connected(c.connect())
            }
            ConnectionState::Connected(c) => {
                ConnectionState::Disconnected(c.disconnect())
            }
        };
    }
}
```

**B. Separate Lifecycle (when possible)**

```rust
struct App {
    // Don't store the typestate machine!
}

impl App {
    fn do_work(&mut self) -> Result<()> {
        // Create, use, drop in one method
        let conn = Connection::new()
            .connect()?
            .authenticate()?;
        conn.send_data(&self.data)?;
        // conn dropped here
        Ok(())
    }
}
```

**Source:** Yoshua Wuyts "State Machines" (2020), machine crate design

### 3.3 Testing Tradeoffs

**Challenge:** Typestates make it hard to construct objects in arbitrary states for tests.

**Pattern:** Builder with `#[cfg(test)]` backdoors

```rust
#[cfg(test)]
impl<S> Connection<S> {
    /// FOR TESTING ONLY: Create connection in arbitrary state
    pub fn test_new_with_state(inner: TcpStream) -> Self {
        Connection { inner, _state: PhantomData }
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_disconnect_from_connected() {
        let conn: Connection<Connected> =
            Connection::test_new_with_state(mock_stream());
        conn.disconnect();  // Test specific state
    }
}
```

**Alternative:** Fake implementations for testing

```rust
trait ConnectionTrait {
    fn send(&mut self, data: &[u8]);
}

// Production: typestate wrapper
struct RealConnection<S>(Connection<S>);

// Test: simple fake
struct FakeConnection { sent: Vec<Vec<u8>> }

#[cfg(test)]
impl ConnectionTrait for FakeConnection { /* ... */ }
```

---

## 4. Performance Considerations

### 4.1 Zero-Cost Abstractions

**Key Insight:** Properly designed typestates have **zero runtime cost**.

```rust
// This compiles to IDENTICAL assembly:

// Version 1: Runtime check
struct File {
    fd: i32,
    is_open: bool,
}

impl File {
    fn read(&mut self) -> Result<Vec<u8>> {
        if !self.is_open {  // Runtime branch
            return Err(Error::NotOpen);
        }
        unsafe { syscall_read(self.fd) }
    }
}

// Version 2: Typestate
struct OpenFile { fd: i32 }
struct ClosedFile { fd: i32 }

impl OpenFile {
    fn read(&mut self) -> Vec<u8> {
        unsafe { syscall_read(self.fd) }  // No branch!
    }
}
```

**Assembly proof:** The typestate version eliminates the `is_open` branch entirely because the type system guarantees it's true.

**Source:** Embedded Rust Book, Cliffle (2019)

### 4.2 Move Overhead

**Concern:** Does consuming `self` copy large structs?

**Answer:** No, if you use smart pointers:

```rust
// BAD: Copies entire buffer on every state change
struct Parser<S> {
    buffer: [u8; 4096],
    _state: PhantomData<S>
}

// GOOD: Moves only pointer, buffer stays put
struct Parser<S> {
    buffer: Box<[u8; 4096]>,
    _state: PhantomData<S>
}

// BETTER: Shared ownership, zero copies
struct Parser<S> {
    buffer: Arc<[u8]>,
    _state: PhantomData<S>
}
```

**Guideline:** If state transitions are frequent, keep large data behind `Box`, `Rc`, or `Arc`.

**Source:** Common pattern in tokio, async-std builders

### 4.3 Code Size

**Monomorphization Impact:**

```rust
// Each state generates separate machine code
impl State<Reading> { fn process(&self) { /* ... */ } }
impl State<Writing> { fn process(&self) { /* ... */ } }

// 2x code size vs runtime enum dispatch
```

**When to care:**
- Embedded systems with tight flash budgets
- Hot path performance more important than code size

**Mitigation:** Extract common logic into `impl<S>` blocks or free functions.

---

## 5. Examples from Production Rust Code

### 5.1 serde::Serializer (Canonical Example)

**Pattern:** Complex state machine with enforced ordering

```rust
pub trait Serializer {
    type SerializeStruct: SerializeStruct;

    fn serialize_struct(self, name: &str, len: usize)
        -> Result<Self::SerializeStruct>;
}

pub trait SerializeStruct {
    fn serialize_field<T>(&mut self, key: &str, value: &T)
        -> Result<()>;

    fn end(self) -> Result<()>;  // Consumes self!
}

// Usage - can't call serialize_struct twice!
let mut ser = MySerializer::new();
let mut state = ser.serialize_struct("Point", 2)?;
state.serialize_field("x", &1)?;
state.serialize_field("y", &2)?;
state.end()?;
// state is consumed, can't use anymore
```

**Guarantees enforced:**
- Can't serialize two values where one expected
- Can't add fields after `end()`
- Can't forget to call `end()`

**Source:** serde documentation, Cliffle (2019)

### 5.2 diesel::UpdateStatement

**Pattern:** Query builder with compile-time SQL correctness

```rust
pub struct UpdateStatement<T, U, V = SetNotCalled, Ret = NoReturningClause> {
    /* private fields */
}

impl<T, U> UpdateStatement<T, U, SetNotCalled> {
    pub fn set<V>(self, values: V) -> UpdateStatement<T, U, V::Changeset>
    where V: AsChangeset<Target = T>
    { /* ... */ }
}

impl<T, U, V> UpdateStatement<T, U, V, NoReturningClause> {
    pub fn returning<E>(self, returns: E)
        -> UpdateStatement<T, U, V, ReturningClause<E>>
    { /* ... */ }
}

// Usage - can't execute without calling .set()!
diesel::update(users)
    .set(name.eq("Jim"))  // Required!
    .filter(id.eq(1))
    .returning(id)
    .get_result(&conn)?;
```

**Guarantees:**
- Can't execute UPDATE without SET clause
- Can't call `set()` twice
- `returning()` only valid on certain databases (enforced via trait bounds)

**Source:** diesel documentation

### 5.3 std::fs::File (Simple Two-State)

**Pattern:** RAII with typestate-like guarantees

```rust
pub struct File { /* private */ }

impl File {
    pub fn open(path: &Path) -> Result<File>;  // Only constructor
}

impl Drop for File {
    fn drop(&mut self) {
        // Auto-close, user can't access closed file
    }
}

// Can't use File after drop(), enforced by move semantics
fn example() {
    let f = File::open("foo.txt")?;
    drop(f);
    f.read_to_string(&mut buf)?;  // ERROR: use after move
}
```

**Source:** std documentation, Yoric (2018)

### 5.4 hyper::client::Builder (Ergonomic Builder)

**Pattern:** Optional typestate - most configs use `&mut self`, only final build consumes

```rust
pub struct Builder {
    /* config fields */
}

impl Builder {
    pub fn new() -> Builder;

    // Ergonomic: can chain or not
    pub fn pool_idle_timeout(&mut self, val: Duration) -> &mut Self;
    pub fn pool_max_idle_per_host(&mut self, val: usize) -> &mut Self;

    // Consumes: can only call once
    pub fn build<C>(self, connector: C) -> Client<C>;
}

// Usage
let client = Builder::new()
    .pool_idle_timeout(Duration::from_secs(30))
    .build(HttpConnector::new());
```

**Design note:** Doesn't enforce ordering because configuration order doesn't matter. Uses typestate only for the final "build" transition.

---

## 6. Recommendations for Lithos Schema Refactor

### 6.1 When to Use Typestates

**✅ Use typestates for:**
- Schema validation pipeline: Raw → Validated → Resolved
- File ingestion states: Source → Parsed → Stored
- Operations with strict ordering requirements
- Preventing "forgot to call initialize" bugs

**❌ Don't use typestates for:**
- Simple boolean flags
- Optional features (use `Option<T>`)
- States that change frequently in loops
- States that need to be stored in collections

### 6.2 Recommended Pattern for Lithos

**Use Approach B (generic struct) with closure-based access:**

```rust
// Schema validation pipeline
struct Schema<S> {
    inner: SchemaData,
    _state: PhantomData<S>
}

struct Raw;
struct Validated;
struct Resolved;

impl Schema<Raw> {
    pub fn parse(content: &str) -> Result<Self, ParseError> { /* ... */ }

    pub fn validate(self) -> Result<Schema<Validated>, ValidationError> {
        // Type transition: Raw → Validated
    }
}

impl Schema<Validated> {
    pub fn resolve<R: Repository>(
        self,
        repo: &R
    ) -> Result<Schema<Resolved>, ResolutionError> {
        // Type transition: Validated → Resolved
    }
}

impl Schema<Resolved> {
    // Only resolved schemas can be stored
    pub fn id(&self) -> &SchemaId { &self.inner.id }

    pub fn save<R: Repository>(self, repo: &mut R) -> Result<(), Error> {
        repo.save(self.inner)
    }
}

// Operations valid in all states
impl<S> Schema<S> {
    pub fn raw_content(&self) -> &str { &self.inner.raw }
}
```

### 6.3 Handling Branching in Loaders

**Use Result + early return for validation branches:**

```rust
pub fn load_schemas<R: Repository>(
    source: &impl FileReader,
    repo: &mut R
) -> Result<usize, Error> {
    let mut count = 0;

    for path in source.list_schemas()? {
        let raw = Schema::parse(&source.read(path)?)?;

        let validated = match raw.validate() {
            Ok(v) => v,
            Err(e) => {
                log::warn!("Skipping invalid schema {}: {}", path, e);
                continue;  // Different branch - skip
            }
        };

        let resolved = validated.resolve(repo)?;
        resolved.save(repo)?;
        count += 1;
    }

    Ok(count)
}
```

### 6.4 Testing Strategy

```rust
#[cfg(test)]
impl<S> Schema<S> {
    /// Test helper: construct schema in arbitrary state
    pub fn test_new(data: SchemaData) -> Self {
        Schema {
            inner: data,
            _state: PhantomData
        }
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_resolve_with_missing_ref() {
        let schema: Schema<Validated> = Schema::test_new(
            SchemaData { /* ... */ }
        );

        let repo = InMemoryRepository::new();
        let result = schema.resolve(&repo);

        assert!(matches!(result, Err(ResolutionError::MissingRef(_))));
    }
}
```

---

## 7. Key Takeaways

1. **Typestates eliminate runtime checks** - Use them when illegal states should never exist, not just be caught.

2. **Choose encoding based on complexity:**
   - 2-4 states → Separate structs
   - 5+ states or cross-state ops → Generic with PhantomData

3. **Ergonomics matter:**
   - State transitions: consume `self`
   - Within-state modifications: `&mut self`
   - Return `self` for optional chaining

4. **Branching is manageable:**
   - Enums for limited outcomes
   - Trait objects when dispatch okay
   - Closures for complex control flow

5. **Storage requires workarounds:**
   - Wrap in enum if state must change
   - Limit lifetime to single scope if possible

6. **Production examples prove value:**
   - serde prevents serialization bugs
   - diesel prevents invalid SQL
   - std::fs prevents use-after-close

7. **Zero runtime cost** - Properly designed typestates compile to same assembly as hand-written code.

---

## Sources

**Authoritative Documentation:**
- Embedded Rust Book - Typestate Programming
  https://docs.rust-embedded.org/book/static-guarantees/typestate-programming.html

**Established Community Resources:**
- Cliffle (2019) - "The Typestate Pattern in Rust"
  https://cliffle.com/blog/rust-typestate/

- Yoric (2018) - "Typestates in Rust"
  https://yoric.github.io/post/rust-typestate/

- Will Crichton - "Type-Driven API Design in Rust - Typestate"
  https://willcrichton.net/rust-api-type-patterns/typestate.html

- Yoshua Wuyts (2020) - "State Machines: Introduction"
  https://blog.yoshuawuyts.com/state-machines/

**Production Code Examples:**
- serde::Serializer trait
- diesel::UpdateStatement
- std::fs::File

**Community Discussion:**
- Rust Users Forum - "Typestate pattern and branching" (2024)
- rustype/notes - Typestate series (academic perspective)

---

**End of Research Document**
