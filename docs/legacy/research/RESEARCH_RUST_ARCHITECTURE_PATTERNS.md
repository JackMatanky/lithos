# Lightweight Architectural Patterns in Rust
## Research Report: Alternatives to CQRS for File-Based Systems

**Date:** March 10, 2026
**Focus:** Lean, performant, idiomatic Rust patterns that provide separation of concerns without CQRS overhead

---

## Executive Summary

After analyzing major Rust projects (Cargo, Zola, mdBook, rust-analyzer) and architectural literature, the consensus is clear: **CQRS is heavyweight overkill for most Rust applications**. The Rust ecosystem favors simpler patterns that leverage the type system, ownership model, and trait system for separation of concerns.

### Key Finding
Rust projects achieve separation of concerns through:
1. **Module boundaries with clear ownership**
2. **Trait-based abstraction over concrete storage** (without event sourcing)
3. **Pipeline architectures** using iterators and type state
4. **Simple service layers** with functional composition
5. **Zero-cost abstractions** that compile to optimal code

---

## 1. File-Based Systems: What They Actually Use

### 1.1 Zola (Static Site Generator) - Component-Based Architecture

**Structure:**
```
components/
  ├── config/      # Configuration management
  ├── content/     # Content parsing (markdown)
  ├── imageproc/   # Image processing
  ├── link_checker/
  ├── markdown/
  ├── search/
  ├── site/        # Orchestration
  ├── templates/   # Template rendering
  └── utils/
```

**Key Patterns:**
- **Bounded contexts as crates**: Each component is isolated
- **Service orchestration in `site/`**: Coordinates components without god-objects
- **Pipeline pattern**: File → Parse → Transform → Render → Write
- **No CQRS, no events**: Direct function calls with `Result<T, E>`

**Example Pattern:**
```rust
// NOT CQRS - just functional composition
pub struct Site {
    config: Config,
    content: Content,
    templates: Templates,
}

impl Site {
    pub fn build(&self) -> Result<(), Error> {
        let pages = self.content.parse_markdown()?;
        let rendered = self.templates.render(pages)?;
        self.write_output(rendered)?;
        Ok(())
    }
}
```

### 1.2 mdBook - Layered Architecture

**Core Pattern:** "Build, Render, Output" pipeline

```rust
// From Rust Book Ch. 12: "Separation of Concerns for Binary Projects"
fn main() {
    let config = Config::build(&args)?;
    run(config)?;  // All logic in separate function
}

fn run(config: Config) -> Result<(), Box<dyn Error>> {
    let contents = fs::read_to_string(config.file_path)?;
    // Business logic here
    Ok(())
}
```

**Key Principles:**
1. **main.rs** handles CLI, setup, error formatting
2. **lib.rs** contains all testable logic
3. **No complex ports**: Just `pub fn` boundaries
4. **Result<T, E>** for error flow, not events

### 1.3 Cargo - Domain-Driven Modules

**Structure:**
```
cargo/core/
  ├── compiler/
  ├── dependency.rs
  ├── manifest.rs
  ├── package.rs
  ├── profiles.rs
  ├── registry.rs
  ├── resolver/
  ├── source_id.rs
  └── workspace.rs
```

**Key Patterns:**
- **Domain types as first-class values**: `Package`, `Manifest`, `Dependency`
- **No command/query split**: Just methods on structs
- **Trait-based extension**: `Source` trait for different registries
- **Module privacy**: Internal complexity hidden behind public API

**Example:**
```rust
pub struct Package {
    manifest: Manifest,
    summary: Summary,
}

impl Package {
    // Simple, direct methods - no CQRS ceremony
    pub fn manifest(&self) -> &Manifest { &self.manifest }
    pub fn dependencies(&self) -> &[Dependency] { ... }
    pub fn resolve(&self, resolver: &Resolver) -> Result<Resolution> { ... }
}
```

---

## 2. Service Layer Pattern in Rust

### 2.1 The Rust Approach (NOT C# Service Layer)

**Bad (Heavyweight):**
```rust
// AVOID: C#-style service layers
trait SchemaService {
    fn create(&mut self, schema: Schema) -> Result<()>;
    fn update(&mut self, id: Id, schema: Schema) -> Result<()>;
    fn delete(&mut self, id: Id) -> Result<()>;
    fn find_by_id(&self, id: Id) -> Result<Option<Schema>>;
    fn list_all(&self) -> Result<Vec<Schema>>;
}
```

**Good (Idiomatic Rust):**
```rust
// Split into focused modules with clear ownership
pub mod schema {
    // Domain types
    pub struct Schema { /* ... */ }

    // Repository abstraction (simple trait)
    pub trait Repository {
        fn get(&self, id: &Id) -> Result<Option<&Schema>>;
        fn insert(&mut self, schema: Schema) -> Result<Id>;
    }

    // Business logic as functions
    pub fn validate(schema: &Schema) -> Result<()> { /* ... */ }
    pub fn load_from_file(path: &Path) -> Result<Schema> { /* ... */ }
}
```

### 2.2 Avoiding God Objects

**From rust-analyzer's architecture:**
> "Each subsystem is a *module* or a *crate*, not a trait. Traits are used for abstraction over I/O (databases, file systems), not for business logic."

**Anti-Pattern:**
```rust
// DON'T: God object with all operations
struct SchemaManager {
    Repository: Box<dyn Repository>,
    validator: Box<dyn Validator>,
    loader: Box<dyn Loader>,
    cache: Box<dyn Cache>,
}
```

**Better Pattern:**
```rust
// DO: Compose focused functions
pub fn load_and_validate(path: &Path) -> Result<Schema> {
    let raw = fs::read_to_string(path)?;
    let schema = parse(&raw)?;
    validate(&schema)?;
    Ok(schema)
}

pub fn save_to_db(schema: &Schema, db: &mut impl Repository) -> Result<()> {
    db.insert(schema.clone())
}
```

### 2.3 Real Example: Elegant APIs

**From Pascal Hertleif's "Elegant Library APIs in Rust":**

```rust
// Builder pattern for complex construction
pub struct OpenOptions {
    read: bool,
    write: bool,
    append: bool,
}

impl OpenOptions {
    pub fn new() -> Self { /* defaults */ }
    pub fn read(mut self, read: bool) -> Self {
        self.read = read;
        self
    }
    pub fn write(mut self, write: bool) -> Self {
        self.write = write;
        self
    }
    pub fn open(&self, path: &Path) -> io::Result<File> { /* ... */ }
}

// Usage: clean and chainable
let file = OpenOptions::new()
    .read(true)
    .write(true)
    .open("file.txt")?;
```

---

## 3. Pipeline / Chain of Responsibility

### 3.1 Iterator-Based Pipelines

**Core Pattern:** Use iterators for lazy, composable data transformation

```rust
// File ingestion pipeline - zero allocations until collect()
pub fn process_files(dir: &Path) -> Result<Vec<Schema>> {
    std::fs::read_dir(dir)?
        .filter_map(Result::ok)                    // Stage 1: Filter errors
        .filter(|e| e.path().extension() == Some("md"))  // Stage 2: Filter files
        .map(|e| fs::read_to_string(e.path()))     // Stage 3: Read
        .collect::<Result<Vec<_>, _>>()?          // Handle errors
        .into_iter()
        .map(|s| parse_schema(&s))                 // Stage 4: Parse
        .collect::<Result<Vec<_>, _>>()           // Stage 5: Collect results
}
```

**Key Benefits:**
- **Lazy evaluation**: No intermediate allocations
- **Type-driven**: Each stage transforms types explicitly
- **Clear separation**: Each `map`/`filter` is a distinct stage
- **Composable**: Easy to add/remove stages

### 3.2 Type-State Pattern for Staged Processing

**From Yoshua Wuyts' "State Machines in Rust":**

```rust
// Each state is a distinct type
struct Raw<T>(T);
struct Validated<T>(T);
struct Stored<T>(T);

impl Raw<String> {
    fn validate(self) -> Result<Validated<Schema>, ParseError> {
        let schema = parse(&self.0)?;
        Ok(Validated(schema))
    }
}

impl Validated<Schema> {
    fn store(self, db: &mut impl Repository) -> Result<Stored<Id>, DbError> {
        let id = db.insert(self.0)?;
        Ok(Stored(id))
    }
}

// Usage: type system enforces order
let raw = Raw(file_content);
let validated = raw.validate()?;      // Can't skip this
let stored = validated.store(&mut db)?;  // Must validate first
```

### 3.3 Newtype Pattern for Domain Boundaries

```rust
// Separate parsing from validation
pub struct RawSchema(String);         // Just text
pub struct ParsedSchema { /* AST */ } // Parsed structure
pub struct ValidatedSchema(ParsedSchema); // Guaranteed valid

impl RawSchema {
    pub fn parse(self) -> Result<ParsedSchema, ParseError> { /* ... */ }
}

impl ParsedSchema {
    pub fn validate(self) -> Result<ValidatedSchema, ValidationError> { /* ... */ }
}

impl ValidatedSchema {
    // Only valid schemas can be stored
    pub fn store(&self, db: &mut impl Repository) -> Result<Id> { /* ... */ }
}
```

---

## 4. Repository Pattern WITHOUT CQRS

### 4.1 Simple Read/Write Trait Split

**Not CQRS** (no events, no separate models):

```rust
// Repository abstraction - ONE model, split operations
pub trait SchemaQuery {
    fn get(&self, id: &Id) -> Result<Option<&Schema>>;
    fn list(&self) -> Result<Vec<&Schema>>;
    fn find_by_name(&self, name: &str) -> Result<Option<&Schema>>;
}

pub trait SchemaCommand {
    fn insert(&mut self, schema: Schema) -> Result<Id>;
    fn update(&mut self, id: &Id, schema: Schema) -> Result<()>;
    fn delete(&mut self, id: &Id) -> Result<()>;
}

// Implementation for in-memory storage
pub struct InMemorySchemas {
    data: HashMap<Id, Schema>,
}

impl SchemaQuery for InMemorySchemas {
    fn get(&self, id: &Id) -> Result<Option<&Schema>> {
        Ok(self.data.get(id))
    }
}

impl SchemaCommand for InMemorySchemas {
    fn insert(&mut self, schema: Schema) -> Result<Id> {
        let id = Id::new();
        self.data.insert(id, schema);
        Ok(id)
    }
}
```

**Why This Works:**
- **Split interface, not models**: Same `Schema` type everywhere
- **No events**: Direct mutations with `Result<T, E>` error handling
- **Trait bounds**: Functions can require `impl SchemaQuery` or `impl SchemaCommand`
- **Zero-cost**: Monomorphization means no vtable overhead

### 4.2 GAT-Based Zero-Copy Access

**For performance-critical paths:**

```rust
// Generic Associated Types for borrowed access
pub trait Repository {
    type Stored<'a> where Self: 'a;

    fn with_item<R>(&self, id: &Id, f: impl FnOnce(&Self::Stored<'_>) -> R)
        -> Result<Option<R>>;
}

// Usage: avoid clones in hot paths
repository.with_item(&id, |schema| {
    // Work with borrowed data, no allocation
    schema.validate()
})?;
```

### 4.3 Real Example: File System Abstraction

**From filesystem-based tools:**

```rust
// Trait for filesystem operations (enables testing)
pub trait FileSystem {
    fn read(&self, path: &Path) -> io::Result<Vec<u8>>;
    fn write(&self, path: &Path, contents: &[u8]) -> io::Result<()>;
    fn list_dir(&self, path: &Path) -> io::Result<Vec<PathBuf>>;
}

// Production impl
pub struct RealFs;

impl FileSystem for RealFs {
    fn read(&self, path: &Path) -> io::Result<Vec<u8>> {
        std::fs::read(path)
    }
    // ...
}

// Test impl
pub struct MemoryFs {
    files: HashMap<PathBuf, Vec<u8>>,
}

impl FileSystem for MemoryFs {
    fn read(&self, path: &Path) -> io::Result<Vec<u8>> {
        self.files.get(path)
            .cloned()
            .ok_or_else(|| io::Error::new(io::ErrorKind::NotFound, "file not found"))
    }
}
```

---

## 5. Actor Model Alternatives

### 5.1 Channel-Based Message Passing (Lightweight)

**Instead of full actor frameworks:**

```rust
use std::sync::mpsc;

pub enum Message {
    Load(PathBuf),
    Store(Schema),
    Shutdown,
}

pub fn schema_worker(rx: mpsc::Receiver<Message>) {
    let mut storage = Repository::new();

    for msg in rx {
        match msg {
            Message::Load(path) => {
                if let Ok(schema) = load_from_file(&path) {
                    storage.insert(schema);
                }
            }
            Message::Store(schema) => storage.insert(schema),
            Message::Shutdown => break,
        }
    }
}

// Usage
let (tx, rx) = mpsc::channel();
std::thread::spawn(|| schema_worker(rx));
tx.send(Message::Load("schema.toml".into()))?;
```

**Benefits:**
- **Standard library**: No external dependencies
- **Type-safe**: Enum messages prevent protocol errors
- **Bounded concurrency**: Use `sync_channel(N)` for backpressure
- **Simple**: No actor framework overhead

### 5.2 State Machines (Compile-Time Checked)

**From the article review:**

```rust
// Encode state transitions in types
pub struct Open;
pub struct Sealed;
pub struct Sent;

pub struct Package<S> {
    contents: Vec<Item>,
    _state: PhantomData<S>,
}

impl Package<Open> {
    pub fn new() -> Self { /* ... */ }
    pub fn add_item(mut self, item: Item) -> Self { /* ... */ }
    pub fn seal(self) -> Package<Sealed> { /* ... */ }
}

impl Package<Sealed> {
    // Can't add items when sealed!
    pub fn send(self, address: Address) -> Package<Sent> { /* ... */ }
}

impl Package<Sent> {
    pub fn track(&self) -> TrackingInfo { /* ... */ }
}

// Usage: compiler enforces state transitions
let pkg = Package::new();
let pkg = pkg.add_item(item);
let pkg = pkg.seal();
let pkg = pkg.send(address);
// pkg.add_item(item);  // Compile error! Package is sealed
```

---

## 6. Hexagonal/Clean Architecture in Rust

### 6.1 Ports and Adapters (Simplified)

**Key insight from "Elegant APIs":**
> Use traits for I/O boundaries, not business logic

```rust
// Domain layer (pure Rust, no I/O)
pub mod domain {
    pub struct Schema { /* ... */ }

    pub fn validate(schema: &Schema) -> Result<(), ValidationError> {
        // Pure logic, no I/O
    }
}

// Ports (trait boundaries)
pub mod ports {
    use super::domain::Schema;

    pub trait Repository {
        fn load(&self, id: &str) -> Result<Schema>;
        fn save(&mut self, schema: &Schema) -> Result<()>;
    }
}

// Adapters (implementations)
pub mod adapters {
    use super::ports::Repository;

    pub struct FileSchemaRepository {
        base_path: PathBuf,
    }

    impl Repository for FileSchemaRepository {
        fn load(&self, id: &str) -> Result<Schema> {
            let path = self.base_path.join(format!("{}.json", id));
            let data = std::fs::read_to_string(path)?;
            serde_json::from_str(&data)
        }
        // ...
    }
}
```

**Critical Difference from CQRS:**
- **No event sourcing**: Direct state changes
- **No CQRS read models**: Same model for read and write
- **Traits for I/O only**: Business logic is plain functions
- **Module boundaries**: Not abstract traits

### 6.2 ARCHITECTURE.md Pattern

**From matklad's article:**

Every project should have `ARCHITECTURE.md` that explains:
1. **Bird's eye view**: What problem does this solve?
2. **Codemap**: Where is what? (no direct links, use search)
3. **Key invariants**: What must NEVER happen?
4. **Boundaries**: Where are the layer splits?

**Example structure:**
```markdown
# Architecture

## Overview
lithos is a structured notes system...

## Codemap
- `src/schema/` - Schema definition and validation
- `src/note/` - Note content management
- `src/db/` - Storage layer (redb-based)
- `src/cli/` - Command-line interface

## Invariants
- Schemas are ALWAYS validated before storage
- The DB module NEVER parses markdown
- Contexts NEVER import each other (only infrastructure)

## Boundaries
- Domain -> Ports -> Adapters
- CLI calls into domain, never accesses storage directly
```

---

## 7. Concrete Code Examples from Real Projects

### 7.1 Zola's Page Processing Pipeline

```rust
// Simplified from Zola's actual code
pub fn process_page(path: &Path, config: &Config) -> Result<Page> {
    let raw = fs::read_to_string(path)?;

    // Pipeline: each function returns Result
    let (front_matter, content) = split_front_matter(&raw)?;
    let meta = parse_front_matter(&front_matter)?;
    let html = render_markdown(&content, config)?;
    let processed = apply_shortcodes(html, config)?;

    Ok(Page {
        meta,
        content: processed,
        path: path.to_path_buf(),
    })
}
```

### 7.2 Cargo's Source Abstraction

```rust
// Simplified from Cargo's actual trait
pub trait Source {
    fn query(&mut self, dep: &Dependency) -> Result<Vec<Summary>>;
    fn download(&mut self, pkg: PackageId) -> Result<Package>;
}

// Multiple implementations: registry, git, path
impl Source for RegistrySource { /* ... */ }
impl Source for GitSource { /* ... */ }
impl Source for PathSource { /* ... */ }

// No CQRS, just polymorphism over data sources
```

### 7.3 Iterator Composition Pattern

```rust
// From Rust community patterns
pub fn find_schemas(dir: &Path) -> impl Iterator<Item = Result<Schema>> + '_ {
    WalkDir::new(dir)
        .into_iter()
        .filter_entry(|e| !is_hidden(e))
        .filter_map(Result::ok)
        .filter(|e| e.path().extension() == Some("toml"))
        .map(move |entry| {
            let content = fs::read_to_string(entry.path())?;
            parse_schema(&content)
        })
}

// Usage: lazy, composable, zero intermediate collections
for schema in find_schemas(&base_path) {
    match schema {
        Ok(s) => println!("Found: {}", s.name),
        Err(e) => eprintln!("Error: {}", e),
    }
}
```

---

## 8. Key Anti-Patterns to Avoid

### 8.1 Over-Abstraction

**Bad:**
```rust
// AVOID: Abstract for the sake of abstract
trait EntityRepository<T, ID> {
    fn find_by_id(&self, id: ID) -> Result<Option<T>>;
    fn find_all(&self) -> Result<Vec<T>>;
    fn save(&mut self, entity: T) -> Result<ID>;
    fn delete(&mut self, id: ID) -> Result<()>;
}
```

**Good:**
```rust
// DO: Concrete types with specific needs
pub struct SchemaStorage {
    db: Database,
}

impl SchemaStorage {
    pub fn get(&self, id: &Id) -> Result<Option<Schema>> { /* ... */ }
    pub fn save(&mut self, schema: Schema) -> Result<Id> { /* ... */ }

    // Domain-specific method, not generic CRUD
    pub fn find_by_template(&self, template: &str) -> Result<Vec<Schema>> { /* ... */ }
}
```

### 8.2 String-Typed APIs

**Bad:**
```rust
// AVOID: Stringly-typed
fn set_mode(&mut self, mode: &str) { /* "read", "write", "append"? */ }
```

**Good:**
```rust
// DO: Type-safe enums
pub enum Mode {
    Read,
    Write,
    Append,
}

fn set_mode(&mut self, mode: Mode) { /* Compiler checks */ }
```

### 8.3 Hidden Allocations

**Bad:**
```rust
// AVOID: Unclear ownership
fn process(&self, id: String) -> String { /* Who owns what? */ }
```

**Good:**
```rust
// DO: Explicit about allocations
fn process(&self, id: &str) -> Result<Cow<'_, str>> {
    if needs_transform(id) {
        Ok(Cow::Owned(transform(id)))  // Explicit allocation
    } else {
        Ok(Cow::Borrowed(id))  // No allocation
    }
}
```

---

## 9. Decision Matrix: When to Use What

| Pattern | Use When | Don't Use When | Example Projects |
|---------|----------|----------------|------------------|
| **Simple modules** | < 10k LOC, clear domains | Complex cross-cutting concerns | Small CLI tools |
| **Trait-based ports** | Need multiple implementations (DB, FS, test mocks) | Only one implementation | Cargo (Source trait) |
| **Iterator pipelines** | Processing sequences, file ingestion | Need random access | Zola (file processing) |
| **Type-state** | State machines, protocol enforcement | Simple state tracking | HTTP libraries |
| **Builder pattern** | Many optional params | < 3 params | std::fs::OpenOptions |
| **Repository pattern** | Abstract over storage | Storage is trivial | Web backends |
| **CQRS** | **Event sourcing required** | **FILE-BASED SYSTEMS** | **❌ Lithos** |

---

## 10. Recommendations for Lithos

### 10.1 Suggested Architecture

```
lithos-core/
├── schema/
│   ├── mod.rs          # Public API
│   ├── types.rs        # Schema, Field, etc.
│   ├── parser.rs       # TOML → Schema
│   ├── validator.rs    # Validation logic
│   └── storage.rs      # Repository trait + impl
├── note/
│   ├── mod.rs
│   ├── types.rs        # Note, Content
│   ├── parser.rs       # Markdown → Note
│   └── storage.rs
├── db/
│   └── redb_impl.rs    # Concrete storage
└── fs/
    └── reader.rs       # File I/O abstraction
```

### 10.2 Specific Patterns to Apply

**1. Module Boundaries (Not CQRS Ports)**
```rust
// In schema/mod.rs
pub struct Schema { /* ... */ }

pub fn parse(content: &str) -> Result<Schema, ParseError> { /* ... */ }
pub fn validate(schema: &Schema) -> Result<(), ValidationError> { /* ... */ }

// Repository trait (NOT Command/Query split)
pub trait Repository {
    fn get(&self, id: &Id) -> Result<Option<Schema>>;
    fn list(&self) -> Result<Vec<Schema>>;
    fn save(&mut self, schema: Schema) -> Result<Id>;
}
```

**2. Pipeline for File Ingestion**
```rust
// In schema/loader.rs
pub fn load_schemas(dir: &Path) -> Result<Vec<Schema>> {
    std::fs::read_dir(dir)?
        .filter_map(Result::ok)
        .filter(|e| e.path().extension() == Some("toml"))
        .map(|e| {
            let content = std::fs::read_to_string(e.path())?;
            let schema = parse(&content)?;
            validate(&schema)?;
            Ok(schema)
        })
        .collect()
}
```

**3. Simple Service Layer**
```rust
// In schema/service.rs (if needed)
pub struct SchemaService<S> {
    storage: S,
}

impl<S: Repository> SchemaService<S> {
    pub fn create_from_file(&mut self, path: &Path) -> Result<Id> {
        let content = std::fs::read_to_string(path)?;
        let schema = parse(&content)?;
        validate(&schema)?;
        self.storage.save(schema)
    }
}
```

---

## 11. Further Reading

### Essential Articles
1. **matklad's "ARCHITECTURE.md"** - https://matklad.github.io/2021/02/06/ARCHITECTURE.md.html
2. **"Elegant Library APIs in Rust"** - https://deterministic.space/elegant-apis-in-rust.html
3. **"State Machines in Rust"** - https://blog.yoshuawuyts.com/state-machines/
4. **Rust Book Ch. 12** - Separation of concerns for binary projects

### Real Project Architectures
- Zola: github.com/getzola/zola (component-based)
- Cargo: github.com/rust-lang/cargo (domain modules)
- mdBook: github.com/rust-lang/mdBook (pipeline architecture)
- rust-analyzer: github.com/rust-analyzer/rust-analyzer (explicit boundaries)

### Key Takeaways
1. **Module boundaries > Trait boundaries** for business logic
2. **Iterators > Collections** for data pipelines
3. **Result<T, E> > Events** for error handling
4. **Type-state > Runtime state** for state machines
5. **Composition > Inheritance** always in Rust
6. **Zero-cost abstractions** are the goal, not purity

---

## Conclusion

**The Rust Way is NOT CQRS.** The Rust ecosystem achieves separation of concerns through:

1. **Strong module boundaries** with clear public APIs
2. **Traits for I/O abstraction** (not business logic)
3. **Type-driven design** that prevents invalid states
4. **Functional composition** of simple, focused functions
5. **Iterator-based pipelines** for data transformation
6. **Ownership system** that encodes resource management in types

For Lithos specifically: **Drop the CQRS ports**. Use simple module boundaries, trait-based storage abstraction (without Command/Query split), and pipeline patterns for file processing.

The code will be simpler, more idiomatic, easier to test, and just as maintainable - if not more so.
