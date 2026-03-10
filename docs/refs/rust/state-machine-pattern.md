# State Machine Pattern in Rust

## Overview

This document explains the **type-state pattern** for state machines in Rust, based on [Hoverbear's state machine article](https://hoverbear.org/blog/rust-state-machine-pattern/). It walks through why we need state machines, the problems with naive approaches, and how to leverage Rust's type system for compile-time guarantees.

**Core Idea:** Use Rust's type system to make **invalid state transitions impossible at compile time**.

---

## What is a State Machine?

A **state machine** is any system that:
1. Has a finite set of **states**
2. Has defined **transitions** between states
3. Can only be in **one state at a time**

**Examples:**
- TCP connection: `Closed → SynSent → Established → CloseWait → Closed`
- File processing: `Discovered → Parsed → Validated → Stored`
- Schema resolution: `Raw → Validated → Graphed → Sorted → Resolved`

**Key constraint:** Not all transitions are valid. For example:
- TCP can't go from `Closed` directly to `Established`
- File processing can't skip `Parsed` and go straight to `Validated`

**Goal:** Enforce these constraints **at compile time**, not runtime.

---

## The Problem: What We Want

Before exploring solutions, let's define what we want from a state machine pattern:

### Requirements

1. **Only one state at a time** - Can't be in multiple states simultaneously
2. **State-specific data** - Each state can carry its own data
3. **Well-defined transitions** - Moving between states has clear semantics
4. **Shared context** - Some data persists across all states
5. **Restricted transitions** - Only explicitly defined transitions allowed
6. **Consuming transitions** - Old state can't be reused after transition
7. **Stack allocation** - No unnecessary heap allocations
8. **Clear error messages** - Invalid transitions caught at compile time
9. **Type system leverage** - Use Rust's types to maximum advantage
10. **Compile-time errors** - As many errors as possible before running

---

## Approach 1: Naive Enum (Runtime Validation)

### Structure

The most obvious approach is to use an enum:

```rust
enum State {
    Waiting { waiting_time: Duration },
    Filling { rate: usize },
    Done,
}

struct Machine {
    state: State,
}

impl Machine {
    fn new() -> Self {
        Machine {
            state: State::Waiting { waiting_time: Duration::from_secs(0) },
        }
    }

    fn to_filling(&mut self) {
        self.state = match self.state {
            State::Waiting { .. } => State::Filling { rate: 1 },
            _ => panic!("Invalid transition from {:?} to Filling!", self.state),
        }
    }

    fn to_done(&mut self) {
        self.state = match self.state {
            State::Filling { .. } => State::Done,
            _ => panic!("Invalid transition from {:?} to Done!", self.state),
        }
    }
}
```

### Problems

❌ **Invalid transitions fail at RUNTIME** - You won't know until the code runs
❌ **Must `match` every state in every method** - Lots of boilerplate
❌ **Internal mutation can bypass checks** - `machine.state = State::Done` works inside the module
❌ **Panics crash the program** - Even with `Result`, errors are runtime

### Benefits

✅ **Small memory footprint** - Enum size is the largest variant
✅ **Stack allocated** - No heap allocations
✅ **Defined semantics** - Transitions either work or crash

**Verdict:** This approach doesn't meet our requirements. We need compile-time guarantees.

---

## Approach 2: Separate Structs with Transitions

### Structure

Instead of an enum, use separate structs for each state:

```rust
// States as separate structs
struct Waiting {
    waiting_time: Duration,
    shared_value: usize,  // Repeated in every state!
}

struct Filling {
    rate: usize,
    shared_value: usize,  // Repeated!
}

struct Done {
    shared_value: usize,  // Repeated!
}

impl Waiting {
    fn new() -> Self {
        Waiting {
            waiting_time: Duration::from_secs(0),
            shared_value: 0,
        }
    }

    // Transition consumes self and returns new state
    fn to_filling(self) -> Filling {
        Filling {
            rate: 1,
            shared_value: self.shared_value,  // Manually transfer
        }
    }
}

impl Filling {
    fn to_done(self) -> Done {
        Done {
            shared_value: self.shared_value,  // Manually transfer
        }
    }
}
```

### Usage

```rust
let waiting = Waiting::new();
let filling = waiting.to_filling();  // Old state consumed
let done = filling.to_done();

// ❌ COMPILE ERROR: Can't skip states
// let done = waiting.to_done();  // ERROR: no method `to_done` on `Waiting`
```

### Problems

❌ **Massive code duplication** - `shared_value` repeated everywhere
❌ **Manual field transfer** - Easy to forget fields during transitions
❌ **Unclear shared state** - Which fields are shared vs state-specific?
❌ **Variable size** - Need enum wrapper to store in parent struct

### Benefits

✅ **Compile-time transition validation** - Invalid transitions can't compile
✅ **Consumes old state** - Can't accidentally reuse stale state
✅ **No match statements** - Direct method calls
✅ **Clear error messages** - Type system tells you what's wrong

**Verdict:** We're getting closer! Compile-time validation is great, but code duplication is painful.

---

## Approach 3: Generic State Machine (The Solution)

### Core Insight

**Separate the machine from its states:**
- The **machine** holds shared context
- The **state** is a generic type parameter
- Each state is a lightweight struct with state-specific data

```rust
// The machine is generic over its state
struct Machine<S> {
    shared_value: usize,  // Available in all states
    state: S,             // Current state (type changes on transition)
}

// States are lightweight - only state-specific data
struct Waiting {
    waiting_time: Duration,
}

struct Filling {
    rate: usize,
}

struct Done;
```

### Implementation Pattern

**Define initial state constructor:**

```rust
impl Machine<Waiting> {
    fn new(shared_value: usize) -> Self {
        Machine {
            shared_value,
            state: Waiting {
                waiting_time: Duration::from_secs(0),
            },
        }
    }
}
```

**Define transitions using `From`:**

```rust
// Waiting → Filling
impl From<Machine<Waiting>> for Machine<Filling> {
    fn from(val: Machine<Waiting>) -> Machine<Filling> {
        Machine {
            shared_value: val.shared_value,  // Transfer shared context
            state: Filling {
                rate: 1,  // Initialize new state data
            },
        }
    }
}

// Filling → Done
impl From<Machine<Filling>> for Machine<Done> {
    fn from(val: Machine<Filling>) -> Machine<Done> {
        Machine {
            shared_value: val.shared_value,
            state: Done,
        }
    }
}
```

### Usage

```rust
// Create machine in initial state
let in_waiting = Machine::<Waiting>::new(0);

// Explicit transition with type annotation
let in_filling = Machine::<Filling>::from(in_waiting);

// Or let type inference handle it
let in_done = in_filling.into();  // Type inferred from context

// ❌ COMPILE ERROR: Invalid transition
// let in_done = Machine::<Done>::from(in_waiting);
// ERROR: the trait `From<Machine<Waiting>>` is not implemented for `Machine<Done>`
```

### Error Messages

What happens when you try an invalid transition?

```rust
let in_waiting = Machine::<Waiting>::new(0);
let in_done = Machine::<Done>::from(in_waiting);
```

**Compiler says:**

```
error[E0277]: the trait bound `Machine<Done>: From<Machine<Waiting>>` is not satisfied
  |
  | let in_done = Machine::<Done>::from(in_waiting);
  |               ^^^^^^^^^^^^^^^^^^
  |
  = help: the following implementations were found:
  = help:   <Machine<Filling> as From<Machine<Waiting>>>
  = help:   <Machine<Done> as From<Machine<Filling>>>
```

**The compiler literally tells you the valid transitions!**

### Benefits

✅ **Compile-time validation** - Invalid transitions can't compile
✅ **Clear error messages** - Compiler suggests valid transitions
✅ **No code duplication** - Shared context lives in `Machine<S>`
✅ **Consumes old state** - Type system enforces linear usage
✅ **Stack allocated** - Memory-efficient
✅ **Type signature shows state** - `Machine<Filling>` is self-documenting
✅ **Idiomatic Rust** - Uses standard `From`/`Into` traits

### Drawbacks

⚠️ **Variable size** - `Machine<Waiting>` and `Machine<Filling>` have different sizes
⚠️ **Minor type noise** - `From` implementations are a bit verbose

**Verdict:** This is the pattern to use! Clean, type-safe, and idiomatic.

---

## Handling Parent Structures (The Enum Wrapper)

### The Problem

If you need to store a state machine in a parent struct, you face a problem:

```rust
struct Factory {
    machine: Machine<???>,  // What type goes here?
}
```

Different states have different sizes, so you can't pick a single concrete type.

### The Solution: Enum Wrapper

Use an enum to wrap all possible states:

```rust
enum MachineWrapper {
    Waiting(Machine<Waiting>),
    Filling(Machine<Filling>),
    Done(Machine<Done>),
}

struct Factory {
    machine: MachineWrapper,
}

impl Factory {
    fn new() -> Self {
        Factory {
            machine: MachineWrapper::Waiting(Machine::new(0)),
        }
    }
}
```

### Transitioning Within the Wrapper

You'll need to `match` to extract, transition, and re-wrap:

```rust
impl MachineWrapper {
    fn step(self) -> Self {
        match self {
            MachineWrapper::Waiting(m) => MachineWrapper::Filling(m.into()),
            MachineWrapper::Filling(m) => MachineWrapper::Done(m.into()),
            MachineWrapper::Done(m) => MachineWrapper::Waiting(m.into()),
        }
    }
}

// Usage
let mut factory = Factory::new();
factory.machine = factory.machine.step();
```

### Why This Isn't Terrible

**Yes, you have to match.** But this is actually good because:

1. **Exhaustiveness checking** - Compiler forces you to handle all states
2. **Explicit intent** - Clear what happens in each state
3. **Type safety preserved** - Still can't do invalid transitions

**The wrapper is ONLY needed when embedding in a parent struct.** Most of your code works with the typed `Machine<S>` directly.

---

## Lithos-Specific Patterns

### Schema Loading Pipeline

**Requirements:**
- 7 distinct phases (discover → parse → validate → graph → sort → resolve → project)
- Each phase has different data and error types
- Phases can't be skipped
- Shared context (vault root, config, filesystem reader)

**Pattern:** Generic state machine

```rust
// The loader machine
struct SchemaLoader<S> {
    vault_root: PathBuf,
    fs_reader: Arc<dyn FsReader>,
    config: Arc<Config>,
    state: S,
}

// States (lightweight, phase-specific data only)
struct Discovered {
    files: Vec<PathBuf>,
}

struct Parsed {
    raw_schemas: Vec<RawSchema>,
}

struct Validated {
    schemas: Vec<Schema>,
}

struct Graphed {
    graph: DependencyGraph,
    schemas: Vec<Schema>,
}

struct Sorted {
    order: Vec<SchemaId>,
    schemas: Vec<Schema>,
}

struct Resolved {
    resolved: Vec<ResolvedSchema>,
}

// Initial state constructor
impl SchemaLoader<Discovered> {
    fn discover(
        vault_root: PathBuf,
        fs_reader: Arc<dyn FsReader>,
        config: Arc<Config>,
    ) -> Result<Self, DiscoveryError> {
        let files = fs_reader.discover_schemas(&vault_root)?;
        Ok(SchemaLoader {
            vault_root,
            fs_reader,
            config,
            state: Discovered { files },
        })
    }
}

// Transitions
impl From<SchemaLoader<Discovered>> for Result<SchemaLoader<Parsed>, ParseError> {
    // Implementation...
}

impl From<SchemaLoader<Parsed>> for Result<SchemaLoader<Validated>, ValidationError> {
    // Implementation...
}

// Usage
let loader = SchemaLoader::discover(vault_root, fs_reader, config)?
    .into()?  // Parse
    .into()?  // Validate
    .into()?  // Build graph
    .into()?  // Topological sort
    .into()?; // Resolve

loader.project(&storage)?;
```

**Why this works:**
- ✅ Can't skip phases - Type system enforces order
- ✅ Each phase has its own error type
- ✅ Shared context (vault_root, fs_reader, config) lives in `SchemaLoader<S>`
- ✅ State-specific data lives in state types
- ✅ Compile-time guarantees

### Config Loading Pipeline

**Requirements:**
- Multiple phases (load → merge → resolve variables → validate → finalize)
- Need to merge multiple config sources
- Variable resolution has dependencies
- Final config should be immutable

**Pattern:** Generic state machine

```rust
struct ConfigLoader<S> {
    // Shared across all phases
    default_config: Config,
    state: S,
}

struct Loaded {
    global: RawConfig,
    vault: Option<RawConfig>,
}

struct Merged {
    config: RawConfig,
}

struct Resolved {
    config: RawConfig,  // Variables expanded
}

struct Validated {
    config: Config,  // Fully validated
}

struct Finalized {
    config: Arc<Config>,  // Immutable, shareable
}

impl ConfigLoader<Loaded> {
    fn load_from_defaults() -> Result<Self, LoadError> {
        // Implementation...
    }
}

// Transitions...
impl From<ConfigLoader<Loaded>> for ConfigLoader<Merged> {
    fn from(val: ConfigLoader<Loaded>) -> Self {
        // Merge global + vault configs
    }
}
```

### When NOT to Use State Machines

**Don't use for:**
- ❌ Simple boolean flags (`started: bool` is fine)
- ❌ Single-phase operations (just use a function)
- ❌ When phases can be freely reordered (use enum + runtime validation)

**DO use for:**
- ✅ Multi-phase pipelines with strict ordering
- ✅ When each phase has distinct data/operations
- ✅ When you want compile-time guarantees
- ✅ Complex workflows (schema resolution, config loading, protocol state)

---

## Testing State Machines

### Testing Individual Phases

```rust
#[test]
fn validates_cycles_at_graph_phase() {
    // Construct machine in specific state for testing
    let in_validated_state = SchemaLoader {
        vault_root: test_vault(),
        fs_reader: test_reader(),
        config: test_config(),
        state: Validated {
            schemas: vec![
                schema_a_extends_b(),
                schema_b_extends_a(),  // Cycle!
            ],
        },
    };

    // Test just the graph building phase
    let result: Result<SchemaLoader<Graphed>, _> = in_validated_state.try_into();
    assert!(matches!(result, Err(CycleError::CircularDependency { .. })));
}
```

### Testing Full Pipeline

```rust
#[test]
fn full_pipeline_integration() {
    let loader = SchemaLoader::discover(test_vault(), test_reader(), test_config())
        .unwrap()
        .into().unwrap()  // Parse
        .into().unwrap()  // Validate
        .into().unwrap()  // Graph
        .into().unwrap()  // Sort
        .into().unwrap(); // Resolve

    loader.project(&test_storage()).unwrap();

    // Verify final state
    assert_eq!(test_storage().count(), 3);
}
```

---

## Naming Conventions

### Machine Name: Domain Concept

The machine struct should describe **what it does**, not that it's a state machine:

✅ **Good:**
- `SchemaLoader<S>` - Loads schemas through multiple phases
- `ConfigLoader<S>` - Loads and merges configuration
- `Connection<S>` - Manages network connection lifecycle

❌ **Bad:**
- `SchemaState<S>` - What does "state" mean here?
- `SchemaIngestion<S>` - Only highlights one part (ingestion)
- `SchemaStateMachine<S>` - Implementation detail, not domain concept

### State Names: Phase or Status

State types should describe **where you are** in the process:

✅ **Good:**
- `Discovered`, `Parsed`, `Validated` - Clear phases
- `Connected`, `Disconnected`, `Failed` - Clear status
- `Waiting`, `Filling`, `Done` - Descriptive

❌ **Bad:**
- `StateA`, `StateB`, `StateC` - No semantic meaning
- `Step1`, `Step2`, `Step3` - Order-dependent, not descriptive

---

## Summary

### Key Takeaways

1. **Use generic state machines for multi-phase pipelines** - `Machine<S>` pattern
2. **Put shared data in the machine** - Don't duplicate across states
3. **Put phase-specific data in states** - Keep states lightweight
4. **Use `From`/`Into` for transitions** - Idiomatic and consumptive
5. **Wrap in enum only when needed** - For embedding in parent structs
6. **Let the type system enforce correctness** - Impossible states are unrepresentable

### Decision Matrix

| Use Case | Pattern | Example |
|----------|---------|---------|
| Multi-phase linear pipeline | Generic state machine | Schema loading |
| Complex workflow with branching | Generic state machine | Protocol state |
| Cyclical state transitions | Enum-based runtime state | LSP server |
| Simple flag | Boolean | `is_initialized` |
| One-time operation | Function | Template rendering |

### Pattern Template

```rust
// 1. Define the machine (generic over state)
struct Machine<S> {
    shared_context: Context,
    state: S,
}

// 2. Define states (lightweight)
struct State1 { phase1_data: Data1 }
struct State2 { phase2_data: Data2 }

// 3. Define initial state constructor
impl Machine<State1> {
    fn new(context: Context) -> Self {
        Machine { shared_context: context, state: State1 { .. } }
    }
}

// 4. Define transitions with `From`
impl From<Machine<State1>> for Machine<State2> {
    fn from(val: Machine<State1>) -> Machine<State2> {
        Machine {
            shared_context: val.shared_context,
            state: State2 { phase2_data: transform(val.state.phase1_data) },
        }
    }
}

// 5. Use it
let machine = Machine::new(context)
    .into();  // Type system enforces correct order
```

---

## References

- [Hoverbear - Rust State Machine Pattern](https://hoverbear.org/blog/rust-state-machine-pattern/) - Original article
- [Rust API Guidelines - Type Safety](https://rust-lang.github.io/api-guidelines/type-safety.html)
- [Rust Book - Generics](https://doc.rust-lang.org/book/ch10-00-generics.html)
- [Rust Book - From and Into](https://doc.rust-lang.org/std/convert/trait.From.html)
