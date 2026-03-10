# State Machine Pattern for Rust

## Overview

This document describes when and how to use state machine patterns in Lithos, based on the [type-state pattern](https://hoverbear.org/blog/rust-state-machine-pattern/).

**Key Principle:** Use state machines to make **invalid state transitions unrepresentable** at compile time.

---

## Pattern Decision Tree

```
Does your process have multiple distinct phases?
├─ NO → Use simple functions/structs
└─ YES → Continue

Can phases be skipped or executed out of order?
├─ YES → You need runtime validation (use enum state machine)
└─ NO → Continue

Do phases have different data/operations?
├─ YES → Use Type-State Pattern (compile-time enforcement)
└─ NO → Use simple phase tracking (enum or flag)

Is the flow cyclical (can return to previous states)?
├─ YES → Use Enum State Machine
└─ NO → Use Type-State Pattern
```

---

## Approach 1: Type-State Pattern (Compile-Time Enforcement)

**Best for:** Linear or tree-like pipelines where each phase has distinct data and operations.

### Basic Structure

```rust
/// Multi-phase schema ingestion pipeline.
pub struct SchemaIngestion<S> {
    context: IngestionContext,  // Shared data across all phases
    state: S,                    // Phase-specific data
}

/// Phase-specific state types
pub struct Discovered {
    files: Vec<PathBuf>,
}

pub struct Parsed {
    raw_schemas: Vec<RawSchema>,
}

pub struct Validated {
    schemas: Vec<Schema>,
}

pub struct GraphBuilt {
    graph: DependencyGraph,
    schemas: Vec<Schema>,
}

pub struct Resolved {
    resolved_schemas: Vec<ResolvedSchema>,
}

/// Shared context available in all phases
pub struct IngestionContext {
    vault_root: PathBuf,
    fs_reader: Arc<dyn FsReader>,
    config: Arc<Config>,
}
```

### Type-Safe Transitions

Each state has methods that **consume `self`** and return the next state:

```rust
impl SchemaIngestion<Discovered> {
    /// Discover schema files in the vault.
    pub fn discover(
        vault_root: PathBuf,
        fs_reader: Arc<dyn FsReader>,
        config: Arc<Config>,
    ) -> Result<Self, DiscoveryError> {
        let files = fs_reader.discover_schemas(&vault_root)?;

        Ok(SchemaIngestion {
            context: IngestionContext { vault_root, fs_reader, config },
            state: Discovered { files },
        })
    }

    /// Parse discovered files into raw schemas.
    pub fn parse(self) -> Result<SchemaIngestion<Parsed>, ParseError> {
        let raw_schemas = self.state.files
            .into_iter()
            .map(|path| self.context.fs_reader.read_schema(&path))
            .collect::<Result<Vec<_>, _>>()?;

        Ok(SchemaIngestion {
            context: self.context,
            state: Parsed { raw_schemas },
        })
    }
}

impl SchemaIngestion<Parsed> {
    /// Validate raw schemas.
    pub fn validate(self) -> Result<SchemaIngestion<Validated>, ValidationError> {
        let schemas = self.state.raw_schemas
            .into_iter()
            .map(|raw| Schema::try_from(raw))
            .collect::<Result<Vec<_>, _>>()?;

        Ok(SchemaIngestion {
            context: self.context,
            state: Validated { schemas },
        })
    }
}

impl SchemaIngestion<Validated> {
    /// Build dependency graph and check for cycles.
    pub fn build_graph(self) -> Result<SchemaIngestion<GraphBuilt>, CycleError> {
        let graph = DependencyGraph::build(&self.state.schemas)?;
        graph.check_cycles()?;

        Ok(SchemaIngestion {
            context: self.context,
            state: GraphBuilt {
                graph,
                schemas: self.state.schemas,
            },
        })
    }
}

impl SchemaIngestion<GraphBuilt> {
    /// Topologically sort schemas (infallible - DAG is guaranteed).
    pub fn topological_sort(self) -> SchemaIngestion<Sorted> {
        let order = self.state.graph.topological_sort();

        SchemaIngestion {
            context: self.context,
            state: Sorted {
                order,
                schemas: self.state.schemas,
            },
        }
    }
}

impl SchemaIngestion<Sorted> {
    /// Resolve schemas in topological order.
    pub fn resolve(self) -> Result<SchemaIngestion<Resolved>, ResolveError> {
        let mut resolved = Vec::new();
        let mut cache = HashMap::new();

        for id in self.state.order {
            let schema = &self.state.schemas[id];
            let resolved_schema = self.resolve_schema(schema, &cache)?;
            cache.insert(id, resolved_schema.clone());
            resolved.push(resolved_schema);
        }

        Ok(SchemaIngestion {
            context: self.context,
            state: Resolved { resolved_schemas: resolved },
        })
    }
}

impl SchemaIngestion<Resolved> {
    /// Project resolved schemas to storage (final phase).
    pub fn project(self, storage: &impl SchemaStorage) -> Result<(), StorageError> {
        for schema in self.state.resolved_schemas {
            storage.save(&schema)?;
        }
        Ok(())
    }
}
```

### Usage

```rust
// ✅ Compile-time enforced ordering
let ingestion = SchemaIngestion::discover(vault_root, fs_reader, config)?
    .parse()?           // Can only parse after discovery
    .validate()?        // Can only validate after parsing
    .build_graph()?     // Can only build graph after validation
    .topological_sort() // Can only sort after graph is built
    .resolve()?;        // Can only resolve after sorting

ingestion.project(&storage)?;

// ❌ COMPILE ERROR: Can't skip phases
let ingestion = SchemaIngestion::discover(vault_root, fs_reader, config)?;
ingestion.build_graph()?;  // ERROR: no method `build_graph` on type `SchemaIngestion<Discovered>`

// ❌ COMPILE ERROR: Can't call wrong methods
let ingestion = SchemaIngestion::discover(vault_root, fs_reader, config)?
    .parse()?;
ingestion.project(&storage)?;  // ERROR: no method `project` on type `SchemaIngestion<Parsed>`
```

### Benefits

1. **Impossible to skip phases** - Type system prevents calling methods in wrong order
2. **Clear error handling per phase** - Each phase has its own error type
3. **Testability** - Can construct and test individual phases
4. **Documentation** - Type signature shows current phase
5. **Refactoring safety** - Adding/removing phases causes compile errors

---

## Approach 2: Enum State Machine (Runtime State Tracking)

**Best for:** Cyclical state transitions, runtime state inspection, or when state needs to be serialized.

### Basic Structure

```rust
/// LSP server connection states.
#[derive(Debug, Clone)]
pub enum LspState {
    Uninitialized,
    Initializing { params: InitializeParams },
    Indexing { progress: usize, total: usize },
    Ready,
    ShuttingDown,
    Failed { error: String },
}

pub struct LspServer {
    state: LspState,
    workspace: Option<Workspace>,
    client: Option<Client>,
}

impl LspServer {
    pub fn new() -> Self {
        Self {
            state: LspState::Uninitialized,
            workspace: None,
            client: None,
        }
    }

    /// Handle initialization request.
    pub fn initialize(&mut self, params: InitializeParams) -> Result<(), LspError> {
        match self.state {
            LspState::Uninitialized => {
                self.state = LspState::Initializing { params };
                Ok(())
            }
            _ => Err(LspError::InvalidState {
                expected: "Uninitialized",
                actual: format!("{:?}", self.state),
            }),
        }
    }

    /// Start indexing the workspace.
    pub fn start_indexing(&mut self) -> Result<(), LspError> {
        match self.state {
            LspState::Initializing { .. } => {
                self.state = LspState::Indexing { progress: 0, total: 100 };
                Ok(())
            }
            _ => Err(LspError::InvalidState {
                expected: "Initializing",
                actual: format!("{:?}", self.state),
            }),
        }
    }

    /// Mark server as ready.
    pub fn mark_ready(&mut self) -> Result<(), LspError> {
        match self.state {
            LspState::Indexing { .. } => {
                self.state = LspState::Ready;
                Ok(())
            }
            _ => Err(LspError::InvalidState {
                expected: "Indexing",
                actual: format!("{:?}", self.state),
            }),
        }
    }

    /// Handle document change (only valid when Ready).
    pub fn handle_change(&mut self, change: DocumentChange) -> Result<(), LspError> {
        match self.state {
            LspState::Ready => {
                // Process change
                Ok(())
            }
            _ => Err(LspError::NotReady),
        }
    }

    /// Shutdown server (can transition from Ready or Failed).
    pub fn shutdown(&mut self) -> Result<(), LspError> {
        match self.state {
            LspState::Ready | LspState::Failed { .. } => {
                self.state = LspState::ShuttingDown;
                Ok(())
            }
            _ => Err(LspError::CannotShutdown),
        }
    }

    /// Check if server is ready to handle requests.
    pub fn is_ready(&self) -> bool {
        matches!(self.state, LspState::Ready)
    }

    /// Get current state for debugging/logging.
    pub fn current_state(&self) -> &LspState {
        &self.state
    }
}
```

### Benefits

1. **Runtime state inspection** - Can query current state
2. **Cyclical transitions** - Can return to previous states
3. **Serializable** - Can save/restore state
4. **Flexible error handling** - Can transition to error states and recover

### When to Use Enum vs Type-State

| Criterion                      | Type-State                | Enum                      |
|--------------------------------|---------------------------|---------------------------|
| Phase ordering                 | Linear/tree               | Cyclical                  |
| State inspection needed?       | No                        | Yes                       |
| Serialize state?               | No                        | Yes                       |
| Can skip phases?               | Never                     | Sometimes                 |
| Error recovery                 | Fail fast                 | Transition to error state |
| Compile-time guarantees        | Strong                    | Weak                      |

---

## Approach 3: Factory Pattern for Construction

**Pattern:** Use named constructors to create initial states clearly.

```rust
impl SchemaIngestion<Discovered> {
    /// Discover schemas from vault root.
    pub fn discover(
        vault_root: PathBuf,
        fs_reader: Arc<dyn FsReader>,
        config: Arc<Config>,
    ) -> Result<Self, DiscoveryError> {
        // ...
    }

    /// Create from already-discovered files (for testing).
    pub fn from_files(
        files: Vec<PathBuf>,
        context: IngestionContext,
    ) -> Self {
        SchemaIngestion {
            context,
            state: Discovered { files },
        }
    }
}

impl ConfigLoad<Raw> {
    /// Load from default locations.
    pub fn from_defaults() -> Result<Self, ConfigError> {
        // ...
    }

    /// Load from explicit paths.
    pub fn from_paths(
        global_path: PathBuf,
        vault_path: Option<PathBuf>,
    ) -> Result<Self, ConfigError> {
        // ...
    }
}
```

---

## Lithos-Specific Guidelines

### Schema Ingestion Pipeline

**Use Type-State Pattern:**

```rust
SchemaIngestion::discover(vault_root, fs_reader, config)?
    .parse()?           // File → RawSchema
    .validate()?        // RawSchema → Schema
    .build_graph()?     // Detect cycles
    .topological_sort() // Order dependencies
    .resolve()?         // Merge inheritance
    .project(storage)?; // Save to DB
```

**Phases:**
1. `Discovered` - Files found on filesystem
2. `Parsed` - Raw YAML/TOML parsed
3. `Validated` - Syntax validation complete
4. `GraphBuilt` - Dependency graph built, cycles checked
5. `Sorted` - Topological sort complete
6. `Resolved` - Inheritance merged, ready for storage

### Config Loading Pipeline

**Use Type-State Pattern:**

```rust
ConfigLoad::from_defaults()?
    .merge_vault_config(vault_path)?  // Merge vault-specific config
    .resolve_variables()?              // Expand ${VAR} references
    .validate()?                       // Check required fields
    .finalize()?;                      // Lock immutable config
```

### LSP Server Lifecycle

**Use Enum State Machine:**

```rust
enum LspState {
    Uninitialized,
    Initializing { params: InitializeParams },
    Indexing { progress: usize, total: usize },
    Ready,
    ShuttingDown,
    Failed { error: String },
}
```

**Why enum:** Can transition back to `Indexing` when files change, need runtime state inspection for progress reporting.

### File Watcher

**Use Enum State Machine:**

```rust
enum WatcherState {
    Idle,
    Debouncing { timer: Instant, changes: Vec<PathBuf> },
    Scanning { files: Vec<PathBuf> },
    Processing,
}
```

**Why enum:** Cyclical transitions (Idle → Debouncing → Scanning → Idle).

---

## Testing State Machines

### Type-State Pattern Testing

```rust
#[test]
fn schema_ingestion_phases_enforced() {
    // Test each phase independently
    let context = test_context();

    // Phase 1: Discovery
    let discovered = SchemaIngestion {
        context: context.clone(),
        state: Discovered {
            files: vec![PathBuf::from("schema1.yaml")],
        },
    };

    // Phase 2: Parsing
    let parsed = discovered.parse().unwrap();
    assert_eq!(parsed.state.raw_schemas.len(), 1);

    // Phase 3: Validation
    let validated = parsed.validate().unwrap();
    assert_eq!(validated.state.schemas.len(), 1);
}

#[test]
fn schema_resolution_validates_cycles() {
    let validated = test_validated_state_with_cycle();

    // Should fail at graph building phase
    let result = validated.build_graph();
    assert!(matches!(result, Err(CycleError::CircularDependency { .. })));
}
```

### Enum State Machine Testing

```rust
#[test]
fn lsp_lifecycle_transitions() {
    let mut server = LspServer::new();
    assert!(matches!(server.state, LspState::Uninitialized));

    // Initialize
    server.initialize(test_params()).unwrap();
    assert!(matches!(server.state, LspState::Initializing { .. }));

    // Start indexing
    server.start_indexing().unwrap();
    assert!(matches!(server.state, LspState::Indexing { .. }));

    // Mark ready
    server.mark_ready().unwrap();
    assert!(matches!(server.state, LspState::Ready));
}

#[test]
fn lsp_rejects_invalid_transitions() {
    let mut server = LspServer::new();

    // Can't handle changes before initialization
    let result = server.handle_change(test_change());
    assert!(matches!(result, Err(LspError::NotReady)));
}
```

---

## Anti-Patterns to Avoid

### ❌ DON'T: Use state machines for simple boolean flags

```rust
// ❌ BAD: Overkill for a simple flag
enum ProcessState {
    NotStarted,
    Started,
}

// ✅ GOOD: Use a boolean
struct Process {
    started: bool,
}
```

### ❌ DON'T: Mix enum and type-state patterns

```rust
// ❌ BAD: Confusing hybrid
struct Pipeline<S> {
    state: S,
    runtime_state: RuntimeState,  // Which one is the source of truth?
}
```

### ❌ DON'T: Create state machines for single-phase operations

```rust
// ❌ BAD: No phases, no need for state machine
struct TemplateRender<S> {
    state: S,
}

// ✅ GOOD: Just use a function
fn render_template(template: &Template, context: &Context) -> Result<String, Error>
```

### ❌ DON'T: Store state machines when you only need the result

```rust
// ❌ BAD: Storing entire pipeline state
struct SchemaResolver {
    ingestion: SchemaIngestion<Resolved>,  // Why store this?
}

// ✅ GOOD: Store the result
struct SchemaResolver {
    resolved_schemas: Vec<ResolvedSchema>,
}
```

---

## Summary

**Type-State Pattern (Compile-Time):**
- ✅ Linear/tree pipelines
- ✅ Each phase has distinct data
- ✅ Want compile-time ordering guarantees
- ✅ Cannot skip phases
- ❌ Not for cyclical transitions
- **Lithos use:** Schema ingestion, config loading

**Enum State Machine (Runtime):**
- ✅ Cyclical transitions
- ✅ Need runtime state inspection
- ✅ Need to serialize state
- ✅ Can transition to error states and recover
- ❌ No compile-time phase ordering
- **Lithos use:** LSP lifecycle, file watcher

**Simple Alternatives:**
- Boolean flags for binary states
- Plain functions for single-phase operations
- Pipeline functions for simple transforms

---

## References

- [Hoverbear - Rust State Machine Pattern](https://hoverbear.org/blog/rust-state-machine-pattern/)
- [Rust API Guidelines - Type Safety](https://rust-lang.github.io/api-guidelines/type-safety.html)
- ADR 002: Storage Pattern (Lithos)
- ADR 003: Serialization Strategy (Lithos)
