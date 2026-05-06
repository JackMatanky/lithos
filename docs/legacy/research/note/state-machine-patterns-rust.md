# State Machine Orchestration Patterns in Rust

Research compiled: 2026-03-26

## Overview

This document summarizes research on state machine patterns in Rust, covering typestate builders, error handling, testing strategies, and real-world examples. The research focuses on compile-time guarantees through Rust's type system.

---

## 1. Builder Pattern with Typestate

### Core Concept

The typestate pattern encodes runtime state into compile-time types, preventing invalid state transitions at compile time rather than runtime.

### Basic Implementation

**Separate Types Per State:**

```rust
struct Waiting {
    waiting_time: Duration,
}

struct Filling {
    rate: usize,
}

struct Done;

impl Waiting {
    fn new() -> Self {
        Waiting { waiting_time: Duration::new(0, 0) }
    }

    // Consumes self, returns new state
    fn to_filling(self) -> Filling {
        Filling { rate: 1 }
    }
}

impl Filling {
    fn to_done(self) -> Done {
        Done
    }
}
```

**Key characteristics:**
- Each state is a distinct type
- Transitions consume the old state via `self` (not `&self`)
- Invalid transitions don't compile (e.g., `Waiting::to_done()` doesn't exist)
- Zero runtime overhead - all checks happen at compile time

### Generic Typestate Pattern

More flexible approach using generics:

```rust
struct StateMachine<S> {
    shared_value: usize,
    state: S,  // State stored as type parameter
}

// Zero-sized state marker types
struct Start;
struct Processing;
struct Complete;

impl StateMachine<Start> {
    fn new() -> Self {
        StateMachine {
            shared_value: 0,
            state: Start,
        }
    }

    fn begin(self) -> StateMachine<Processing> {
        StateMachine {
            shared_value: self.shared_value,
            state: Processing,
        }
    }
}

impl StateMachine<Processing> {
    fn finish(self) -> StateMachine<Complete> {
        StateMachine {
            shared_value: self.shared_value,
            state: Complete,
        }
    }
}
```

**Advantages:**
- Single struct definition with state parameter
- State-specific methods via `impl StateMachine<SpecificState>`
- Shared fields across all states
- Type signature shows state: `StateMachine<Processing>`
- Better error messages (compiler suggests valid transitions)

### State Types with Data

States can carry their own data:

```rust
struct HttpResponse<S> {
    state: Box<ActualState>,
    extra: S,  // State-specific data
}

struct Start;  // No extra data

struct Headers {
    response_code: u8,  // Only available in Headers state
}

impl HttpResponse<Start> {
    fn status_line(self, code: u8, msg: &str) -> HttpResponse<Headers> {
        HttpResponse {
            state: self.state,
            extra: Headers { response_code: code },
        }
    }
}

impl HttpResponse<Headers> {
    fn response_code(&self) -> u8 {
        self.extra.response_code  // Direct access, no Option
    }
}
```

**Benefits:**
- Memory efficient: only allocate space for current state's data
- Type-safe access: `response_code()` only exists in `Headers` state
- No `Option` wrapping for state-specific fields

### Builder Pattern Integration

Typestate works naturally with builders:

```rust
struct Builder<S> {
    config: Config,
    state: PhantomData<S>,
}

struct Unconfigured;
struct Configured;

impl Builder<Unconfigured> {
    fn new() -> Self {
        Builder {
            config: Config::default(),
            state: PhantomData,
        }
    }

    fn with_timeout(mut self, timeout: Duration) -> Builder<Configured> {
        self.config.timeout = Some(timeout);
        Builder {
            config: self.config,
            state: PhantomData,
        }
    }
}

impl Builder<Configured> {
    fn build(self) -> Result<Connection, Error> {
        // Only callable when configured
        Connection::new(self.config)
    }
}
```

**Pattern:** Required configuration steps are encoded as state transitions. The `build()` method only exists on the fully configured state.

### Real-World Example: serde

The `serde` crate uses typestate extensively:

```rust
trait Serializer {
    type SerializeStruct: SerializeStruct;

    fn serialize_struct(self, name: &str, len: usize)
        -> Result<Self::SerializeStruct, Self::Error>;
}

trait SerializeStruct {
    fn serialize_field<T>(&mut self, key: &str, value: &T)
        -> Result<(), Self::Error>;

    fn end(self) -> Result<(), Self::Error>;  // Consumes self
}
```

You cannot:
- Call `serialize_struct` twice
- Add fields after calling `end()`
- Mix struct and sequence serialization

All enforced at compile time.

---

## 2. Continuation-Passing Style (CPS)

### Concept

Instead of returning state directly, pass a continuation (closure) that receives the next state.

### Basic Pattern

```rust
struct Machine<S> {
    state: S,
}

impl<S> Machine<S> {
    fn transition<Next, F>(self, f: F) -> Machine<Next>
    where
        F: FnOnce(S) -> Next,
    {
        Machine {
            state: f(self.state),
        }
    }
}

// Usage
let machine = Machine { state: Start }
    .transition(|start| Processing { data: start.init_data() })
    .transition(|proc| Complete { result: proc.compute() });
```

### Async Continuation Pattern

CPS maps naturally to async/await:

```rust
impl Machine<Start> {
    async fn begin<F, Fut>(self, f: F) -> Machine<Processing>
    where
        F: FnOnce(Start) -> Fut,
        Fut: Future<Output = Processing>,
    {
        Machine {
            state: f(self.state).await,
        }
    }
}

// Usage
let machine = Machine::new()
    .begin(|start| async move {
        let data = fetch_data().await;
        Processing { data }
    })
    .await;
```

### Builder with Continuation

```rust
impl Builder<Unconfigured> {
    fn configure<F>(self, f: F) -> Builder<Configured>
    where
        F: FnOnce(&mut Config),
    {
        let mut config = self.config;
        f(&mut config);
        Builder {
            config,
            state: PhantomData,
        }
    }
}

// Usage
let builder = Builder::new()
    .configure(|cfg| {
        cfg.timeout = Some(Duration::from_secs(30));
        cfg.retry_count = 3;
    });
```

**When to use CPS:**
- Complex initialization logic
- Need to capture local variables in transitions
- Async operations between states
- Want to provide callback hooks during transitions

**Trade-offs:**
- More flexible than direct state transitions
- Can be harder to reason about control flow
- May obscure the state machine structure
- Useful when transitions need context from calling code

---

## 3. Internal vs External Iteration Over States

### External Iteration (Pull Model)

State machine exposes an iterator:

```rust
enum State {
    Start,
    Processing,
    Complete,
}

struct StateMachine {
    current: State,
}

impl Iterator for StateMachine {
    type Item = State;

    fn next(&mut self) -> Option<Self::Item> {
        let next_state = match &self.current {
            State::Start => State::Processing,
            State::Processing => State::Complete,
            State::Complete => return None,
        };
        Some(std::mem::replace(&mut self.current, next_state))
    }
}

// Usage
for state in state_machine {
    match state {
        State::Start => println!("Starting..."),
        State::Processing => println!("Processing..."),
        State::Complete => println!("Done!"),
    }
}
```

**Characteristics:**
- Caller controls iteration pace
- State machine is mutable
- Cannot use typestate pattern (need single enum type)
- Good for linear state progressions
- Useful when external events drive transitions

### Internal Iteration (Push Model)

Caller provides callbacks for state changes:

```rust
trait StateVisitor {
    fn visit_start(&mut self, state: &Start);
    fn visit_processing(&mut self, state: &Processing);
    fn visit_complete(&mut self, state: &Complete);
}

impl<S> StateMachine<S> {
    fn visit<V: StateVisitor>(self, visitor: &mut V) -> Result<(), Error> {
        // State machine drives the iteration
        // Calls visitor methods as it transitions
    }
}

// Usage
struct Logger;
impl StateVisitor for Logger {
    fn visit_start(&mut self, _: &Start) {
        println!("Entered start state");
    }
    // ... other methods
}

let mut logger = Logger;
state_machine.visit(&mut logger)?;
```

**Characteristics:**
- State machine controls iteration
- Can use typestate pattern with visitor
- Better encapsulation of state transitions
- Useful for event-driven architectures

### Hybrid: Async Streams

Combine pull iteration with typestate:

```rust
struct StateMachine<S> {
    state: S,
}

impl StateMachine<Start> {
    fn into_stream(self) -> impl Stream<Item = StateEvent> {
        stream! {
            yield StateEvent::Started;
            let processing = self.to_processing();
            yield StateEvent::Processing;
            let complete = processing.to_complete();
            yield StateEvent::Complete(complete.result);
        }
    }
}

// Usage
let mut stream = state_machine.into_stream();
while let Some(event) = stream.next().await {
    handle_event(event);
}
```

**Benefits:**
- Typestate safety
- Async-friendly
- Backpressure support
- Composable with other streams

### When to Use Each

**External Iteration:**
- Simple linear progressions
- Need to pause/resume iteration
- External events trigger transitions
- Don't need compile-time state validation

**Internal Iteration:**
- Complex state machines with branching
- Want to encapsulate transition logic
- Observers pattern (multiple listeners)
- Can tolerate runtime state checks

**Async Streams:**
- I/O-bound state transitions
- Need backpressure handling
- Want compositional state machines
- Require cancellation support

---

## 4. Error Handling in Typestate Transitions

### Fallible Constructors

```rust
impl StateMachine<Unconfigured> {
    // Infallible - state always created
    fn new() -> Self {
        StateMachine {
            state: Unconfigured,
        }
    }
}

impl StateMachine<Configured> {
    // Fallible - might fail to transition
    fn try_new(config: Config) -> Result<Self, ConfigError> {
        config.validate()?;
        Ok(StateMachine {
            state: Configured { config },
        })
    }
}
```

### Fallible Transitions

**Option: Return original state on error**

```rust
impl StateMachine<Processing> {
    fn try_complete(self) -> Result<StateMachine<Complete>, Self> {
        if self.state.is_valid() {
            Ok(StateMachine {
                state: Complete { result: self.state.result },
            })
        } else {
            Err(self)  // Return original state
        }
    }
}

// Usage
match machine.try_complete() {
    Ok(complete) => handle_complete(complete),
    Err(still_processing) => {
        // Can retry or handle error
        still_processing.fix_and_retry()
    }
}
```

**Benefits:**
- Can recover from failed transitions
- Don't lose machine state on error
- Can retry with fixes

**Option: Return new error state**

```rust
#[derive(Debug)]
enum State {
    Start,
    Processing,
    Complete,
    Error(ErrorInfo),
}

struct StateMachine<S> {
    state: S,
}

impl StateMachine<Processing> {
    fn complete(self) -> StateMachine<StateEnum> {
        if self.state.is_valid() {
            StateMachine {
                state: StateEnum::Complete(Complete { ... }),
            }
        } else {
            StateMachine {
                state: StateEnum::Error(ErrorInfo { ... }),
            }
        }
    }
}
```

**Trade-off:** Lose typestate benefits after error, but can continue execution.

### Non-Deterministic Transitions

Use enums to represent multiple possible end states:

```rust
enum TransitionResult {
    Success(StateMachine<Success>),
    Retry(StateMachine<Retry>),
    Failed(StateMachine<Failed>),
}

impl StateMachine<Attempting> {
    fn attempt(self) -> TransitionResult {
        match self.state.try_operation() {
            Ok(result) => TransitionResult::Success(
                StateMachine { state: Success { result } }
            ),
            Err(e) if e.is_retryable() => TransitionResult::Retry(
                StateMachine { state: Retry { attempt: self.state.attempt + 1 } }
            ),
            Err(e) => TransitionResult::Failed(
                StateMachine { state: Failed { error: e } }
            ),
        }
    }
}

// Usage
match machine.attempt() {
    TransitionResult::Success(s) => handle_success(s),
    TransitionResult::Retry(r) => schedule_retry(r),
    TransitionResult::Failed(f) => handle_failure(f),
}
```

### Error State Pattern

Explicit error state in the machine:

```rust
enum State {
    Working(Working),
    Error(Error),
}

struct StateMachine<S> {
    state: S,
}

impl StateMachine<Working> {
    fn process(self) -> StateMachine<State> {
        match self.state.do_work() {
            Ok(result) => StateMachine {
                state: State::Working(Working { result }),
            },
            Err(e) => StateMachine {
                state: State::Error(Error {
                    message: e.to_string(),
                    recoverable: e.is_recoverable(),
                }),
            },
        }
    }
}

impl StateMachine<State> {
    fn recover(self) -> Option<StateMachine<Working>> {
        match self.state {
            State::Error(e) if e.recoverable => {
                Some(StateMachine {
                    state: Working::default(),
                })
            }
            _ => None,
        }
    }
}
```

### Testing Error Paths

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_invalid_transition() {
        let machine = StateMachine::<Start>::new();

        // This shouldn't compile - that's what we're testing
        // machine.to_complete(); // Compile error!

        // Instead, test fallible transitions
        let machine = machine.to_processing();
        match machine.try_complete() {
            Ok(_) => panic!("Should have failed"),
            Err(original) => {
                // Verify we got the original state back
                assert_eq!(original.state.data, expected);
            }
        }
    }

    #[test]
    fn test_error_state_recovery() {
        let machine = create_failing_machine();
        let machine = machine.process(); // Goes to error state

        if let Some(recovered) = machine.recover() {
            // Successfully recovered
            assert!(recovered.is_ready());
        }
    }
}
```

---

## 5. Testing Strategies for Typestate Machines

### Compile-Time Tests

The primary benefit of typestate is compile-time verification. Use negative tests:

```rust
#[test]
fn test_invalid_transitions_dont_compile() {
    // These tests verify that invalid code doesn't compile
    // Uncomment each line to verify it fails to compile

    let machine = StateMachine::<Start>::new();

    // Cannot skip to complete directly
    // let complete = machine.to_complete();

    // Cannot use methods from wrong state
    // let result = machine.get_result();

    // Cannot reuse consumed state
    // let processing = machine.to_processing();
    // let complete = processing.to_complete();
    // let _ = processing.to_complete(); // Compile error!
}
```

Use `trybuild` crate for formal compile-fail tests:

```rust
#[test]
fn ui_tests() {
    let t = trybuild::TestCases::new();
    t.compile_fail("tests/ui/invalid_*.rs");
    t.pass("tests/ui/valid_*.rs");
}
```

### State Transition Tests

Test each valid transition:

```rust
#[test]
fn test_complete_happy_path() {
    let machine = StateMachine::<Start>::new();
    assert_eq!(machine.state.counter, 0);

    let machine = machine.to_processing();
    assert!(machine.state.started_at.is_some());

    let machine = machine.to_complete().unwrap();
    assert!(machine.state.duration > Duration::ZERO);
}
```

### Property-Based Testing

Use `proptest` to verify state machine properties:

```rust
use proptest::prelude::*;

proptest! {
    #[test]
    fn test_state_machine_invariants(
        operations in prop::collection::vec(
            any::<Operation>(),
            0..100
        )
    ) {
        let mut machine = StateMachine::new();

        for op in operations {
            machine = apply_operation(machine, op);

            // Invariants that should hold in any state
            assert!(machine.is_consistent());
            assert!(machine.counter() >= 0);
        }
    }
}
```

### Testing State-Specific Behavior

```rust
#[test]
fn test_processing_state_behavior() {
    let machine = create_processing_machine();

    // Test methods only available in Processing state
    assert!(machine.can_pause());
    assert_eq!(machine.progress(), 0.5);

    let machine = machine.add_work(10);
    assert_eq!(machine.work_remaining(), 10);
}
```

### Testing Shared Behavior

```rust
#[test]
fn test_shared_methods_across_states() {
    let start = StateMachine::<Start>::new();
    assert_eq!(start.id(), 1);

    let processing = start.to_processing();
    assert_eq!(processing.id(), 1);  // ID preserved

    let complete = processing.to_complete().unwrap();
    assert_eq!(complete.id(), 1);  // Still same ID
}
```

### Mock States for Testing

```rust
#[cfg(test)]
mod tests {
    struct MockProcessing {
        should_fail: bool,
    }

    impl MockProcessing {
        fn to_complete(self) -> Result<Complete, Self> {
            if self.should_fail {
                Err(self)
            } else {
                Ok(Complete { result: "test" })
            }
        }
    }

    #[test]
    fn test_with_mock() {
        let mock = MockProcessing { should_fail: true };
        match mock.to_complete() {
            Err(_) => { /* Expected */ },
            Ok(_) => panic!("Should have failed"),
        }
    }
}
```

### Integration Tests

Test the full state machine lifecycle:

```rust
#[test]
fn test_full_lifecycle() {
    let machine = StateMachine::new()
        .configure(|cfg| cfg.timeout = Duration::from_secs(30))
        .start()
        .expect("Failed to start");

    let machine = machine.process_batch(&data);

    let results = machine.complete()
        .expect("Failed to complete")
        .results();

    assert_eq!(results.len(), data.len());
}
```

### Fuzzing State Machines

```rust
#[cfg(fuzzing)]
mod fuzz {
    use arbitrary::Arbitrary;

    #[derive(Arbitrary, Debug)]
    enum Operation {
        Start,
        Process(Vec<u8>),
        Complete,
    }

    pub fn fuzz_state_machine(operations: Vec<Operation>) {
        let mut machine = StateMachine::new();

        for op in operations {
            machine = match (machine, op) {
                (StateMachine::Start(s), Operation::Process(data)) => {
                    s.process(data)
                }
                (StateMachine::Processing(p), Operation::Complete) => {
                    p.complete()
                }
                // Invalid transitions are ignored or logged
                (machine, _) => machine,
            };
        }
    }
}
```

---

## 6. Real-World Examples and Anti-Patterns

### Real-World Examples

#### 1. File Handle State Machine

```rust
struct File<S> {
    handle: FileHandle,
    state: PhantomData<S>,
}

struct Closed;
struct Open;

impl File<Closed> {
    fn open(path: &Path) -> io::Result<File<Open>> {
        let handle = fs::File::open(path)?;
        Ok(File {
            handle: FileHandle { inner: handle },
            state: PhantomData,
        })
    }
}

impl File<Open> {
    fn read(&mut self) -> io::Result<Vec<u8>> {
        let mut buf = Vec::new();
        self.handle.inner.read_to_end(&mut buf)?;
        Ok(buf)
    }

    fn close(self) -> File<Closed> {
        drop(self.handle);
        File {
            handle: FileHandle::closed(),
            state: PhantomData,
        }
    }
}

impl Drop for File<Open> {
    fn drop(&mut self) {
        // Auto-close on drop
    }
}
```

**Pattern:** RAII with typestate ensures files can't be read after closing.

#### 2. Database Transaction States

```rust
struct Transaction<S> {
    connection: Connection,
    state: S,
}

struct Active {
    modifications: Vec<Change>,
}

struct Prepared {
    statement: PreparedStatement,
}

struct Committed;
struct RolledBack;

impl Transaction<Active> {
    fn begin(conn: Connection) -> Self {
        Transaction {
            connection: conn,
            state: Active { modifications: vec![] },
        }
    }

    fn prepare(self, sql: &str) -> Result<Transaction<Prepared>, Error> {
        let stmt = self.connection.prepare(sql)?;
        Ok(Transaction {
            connection: self.connection,
            state: Prepared { statement: stmt },
        })
    }

    fn rollback(self) -> Transaction<RolledBack> {
        self.connection.rollback();
        Transaction {
            connection: self.connection,
            state: RolledBack,
        }
    }
}

impl Transaction<Prepared> {
    fn commit(self) -> Result<Transaction<Committed>, Error> {
        self.state.statement.execute()?;
        self.connection.commit()?;
        Ok(Transaction {
            connection: self.connection,
            state: Committed,
        })
    }
}
```

**Pattern:** Enforces transaction lifecycle: begin → prepare → commit/rollback.

#### 3. Network Protocol State Machine (Raft)

From the research, a simplified Raft implementation:

```rust
struct Raft<S> {
    node_id: NodeId,
    state: S,
}

struct Follower {
    last_heartbeat: Instant,
}

struct Candidate {
    votes_received: usize,
    election_timeout: Instant,
}

struct Leader {
    next_index: HashMap<NodeId, u64>,
}

// Follower → Candidate (election timeout)
impl From<Raft<Follower>> for Raft<Candidate> {
    fn from(val: Raft<Follower>) -> Raft<Candidate> {
        Raft {
            node_id: val.node_id,
            state: Candidate {
                votes_received: 1,  // Vote for self
                election_timeout: Instant::now() + Duration::from_millis(150),
            },
        }
    }
}

// Candidate → Leader (won election)
impl From<Raft<Candidate>> for Raft<Leader> {
    fn from(val: Raft<Candidate>) -> Raft<Leader> {
        Raft {
            node_id: val.node_id,
            state: Leader {
                next_index: HashMap::new(),
            },
        }
    }
}

// Candidate → Follower (lost election)
impl From<Raft<Candidate>> for Raft<Follower> {
    fn from(val: Raft<Candidate>) -> Raft<Follower> {
        Raft {
            node_id: val.node_id,
            state: Follower {
                last_heartbeat: Instant::now(),
            },
        }
    }
}
```

**Pattern:** Non-linear state machine with multiple valid transitions per state.

### Anti-Patterns

#### 1. Runtime State Enum Instead of Typestate

**Anti-pattern:**

```rust
enum State {
    Open,
    Closed,
}

struct File {
    state: State,
    handle: Option<fs::File>,
}

impl File {
    fn read(&mut self) -> io::Result<Vec<u8>> {
        match self.state {
            State::Open => {
                // Runtime check!
                let handle = self.handle.as_mut()
                    .expect("File not open");
                // ...
            }
            State::Closed => panic!("Cannot read closed file"),
        }
    }
}
```

**Problems:**
- Runtime panics instead of compile errors
- `Option` wrapper needed for handle
- Easy to forget state checks

**Better:**

```rust
struct File<S> {
    handle: fs::File,
    state: PhantomData<S>,
}

struct Open;
struct Closed;

impl File<Open> {
    fn read(&mut self) -> io::Result<Vec<u8>> {
        // No state check needed - type system guarantees we're open
        self.handle.read_to_end(&mut vec![])
    }
}
```

#### 2. Shared Mutable State Across Typestate

**Anti-pattern:**

```rust
struct Machine<S> {
    state: Arc<Mutex<S>>,  // Shared mutable state
}

impl<S> Clone for Machine<S> {
    fn clone(&self) -> Self {
        Machine {
            state: self.state.clone(),
        }
    }
}
```

**Problems:**
- Can't move state because it's shared
- Typestate transitions can't consume self
- Defeats the purpose of typestate

**Better:**

```rust
struct Machine<S> {
    state: S,  // Owned state
    shared_data: Arc<SharedData>,  // Only share immutable data
}

impl Machine<State1> {
    fn transition(self) -> Machine<State2> {
        Machine {
            state: State2,
            shared_data: self.shared_data,  // Arc clone is cheap
        }
    }
}
```

#### 3. Mixing Typestate and Dynamic Dispatch

**Anti-pattern:**

```rust
trait State {}

struct Machine {
    state: Box<dyn State>,  // Type-erased state
}

impl Machine {
    fn transition(&mut self) {
        // Runtime dispatch, no compile-time guarantees
        self.state = Box::new(NextState);
    }
}
```

**Problems:**
- Loses compile-time guarantees
- Can't restrict methods to specific states
- Defeats typestate benefits

**When dynamic dispatch is acceptable:**

```rust
// Wrapper enum for storage in collections
enum MachineState {
    State1(Machine<State1>),
    State2(Machine<State2>),
}

impl MachineState {
    fn process(self) -> Self {
        match self {
            MachineState::State1(m) => MachineState::State2(m.transition()),
            MachineState::State2(m) => MachineState::State1(m.transition()),
        }
    }
}

// Collection of machines in various states
let machines: Vec<MachineState> = vec![...];
```

#### 4. Overly Complex State Hierarchies

**Anti-pattern:**

```rust
struct Machine<S1, S2, S3, S4> {
    state1: PhantomData<S1>,
    state2: PhantomData<S2>,
    state3: PhantomData<S3>,
    state4: PhantomData<S4>,
}
```

**Problems:**
- Type signatures become unreadable
- Hard to reason about valid state combinations
- Combinatorial explosion of implementations

**Better:** Use nested state machines:

```rust
struct Outer<S> {
    inner: Inner,
    state: PhantomData<S>,
}

struct Inner<S> {
    state: PhantomData<S>,
}
```

Or use state composition:

```rust
struct Machine<S> {
    connection_state: ConnectionState,
    processing_state: S,
}
```

#### 5. Not Using Sealed Traits

**Anti-pattern:**

```rust
// Public trait - users can implement it
pub trait State {}

pub struct Machine<S: State> {
    state: S,
}
```

**Problem:** Users can create invalid states by implementing `State` for their own types.

**Better:**

```rust
mod sealed {
    pub trait Sealed {}
}

pub trait State: sealed::Sealed {}

pub struct Start;
impl sealed::Sealed for Start {}
impl State for Start {}

// Users cannot implement State for their types
```

---

## 7. When to Expose State Transitions vs Encapsulate Them

### Expose Transitions When:

1. **Library API with clear lifecycle**
   ```rust
   // User controls transition timing
   pub struct Builder<S>(S);

   impl Builder<Unconfigured> {
       pub fn configure(self, ...) -> Builder<Configured> { ... }
   }

   impl Builder<Configured> {
       pub fn build(self) -> Connection { ... }
   }
   ```

2. **State transitions have side effects users need to control**
   ```rust
   impl File<Open> {
       // User decides when to close
       pub fn close(self) -> File<Closed> { ... }
   }
   ```

3. **Complex state machines where different paths are valid**
   ```rust
   impl Transaction<Active> {
       pub fn commit(self) -> Result<Committed, Error> { ... }
       pub fn rollback(self) -> RolledBack { ... }
       // User chooses which path
   }
   ```

### Encapsulate Transitions When:

1. **Linear state progression**
   ```rust
   pub struct Process {
       // Internal state machine hidden
       state: ProcessState,
   }

   impl Process {
       pub fn run(&mut self) -> Result<Output, Error> {
           // Transitions happen internally
           self.state.advance()?;
       }
   }
   ```

2. **Implementation detail that might change**
   ```rust
   // Don't expose that HTTP request goes through multiple states
   pub fn send_request(url: &str) -> Result<Response, Error> {
       // Internal state machine:
       // Connecting → Sending → ReceivingHeaders → ReceivingBody → Complete
       // But user just sees Result
   }
   ```

3. **Automatic state management**
   ```rust
   pub struct Cache<T> {
       // States: Cold, Warming, Hot
       state: CacheState,
       data: T,
   }

   impl<T> Cache<T> {
       // User doesn't control warming - it happens automatically
       pub fn get(&mut self, key: &str) -> Option<&T> {
           // May trigger Cold -> Warming transition internally
       }
   }
   ```

### Hybrid Approach

```rust
pub struct Pipeline {
    // Exposed states
    state: PipelineState,
}

pub enum PipelineState {
    // Expose high-level states
    Ready(Ready),
    Running(Running),
    Complete(Complete),
}

struct Running {
    // Internal state machine (hidden)
    phase: Phase,
}

enum Phase {
    // Not exposed to users
    Validating,
    Processing,
    Finalizing,
}
```

**Guidelines:**
- Expose states when users need control or visibility
- Hide states when they're implementation details
- Document state transitions clearly when exposed
- Provide escape hatches (e.g., `into_inner()`) when needed

---

## 8. Key Takeaways

### Benefits of Typestate in Rust

1. **Compile-time guarantees** - Invalid transitions don't compile
2. **Zero runtime cost** - No dynamic checks or vtables needed
3. **Self-documenting** - Type signatures show valid operations
4. **IDE support** - Autocomplete only shows valid methods
5. **Refactoring safety** - Compiler catches broken assumptions

### When to Use Typestate

✅ Use typestate when:
- State transitions have important invariants
- Invalid states would cause bugs or unsafety
- The state machine is relatively small (<10 states)
- Compile-time verification is worth the complexity

❌ Avoid typestate when:
- State transitions are entirely dynamic (user-driven)
- Need to store state machines in homogeneous collections
- State machine is very complex (>10 states)
- Rapid prototyping where flexibility matters more

### Practical Recommendations

1. **Start simple** - Begin with separate types per state before using generics
2. **Use `From`/`Into`** - Leverage standard conversion traits for transitions
3. **Document the state machine** - Generate diagrams, write docs
4. **Test both positive and negative cases** - Use compile-fail tests
5. **Consider ergonomics** - Builder pattern methods can return `&mut self` for chaining
6. **Plan for errors** - Decide early: error state, or return original state?
7. **Use enums for storage** - Wrap typed states in enum for heterogeneous collections

### Common Patterns Summary

| Pattern | Use Case | Trade-offs |
|---------|----------|------------|
| Separate Types | Simple state machines | More boilerplate, very clear |
| Generic State Parameter | Complex machines with shared data | More abstract, flexible |
| Phantom Data | States with no data | Zero memory overhead |
| States with Data | State-specific fields | Memory efficient, type-safe |
| Enum Wrapper | Mixed-state collections | Some runtime dispatch |
| Sealed Traits | Prevent user extension | More boilerplate, safer |

### Resources

- **Crate:** `typestate` - Proc macro DSL for typestates
- **Crate:** `machine` - State machine code generation
- **Pattern:** Embedded Rust Book's typestate chapter
- **Example:** `serde`'s Serializer trait hierarchy
- **Tool:** `trybuild` for compile-fail tests

---

## References

1. Hoverbear - "Pretty State Machine Patterns in Rust" (2016)
   - https://hoverbear.org/blog/rust-state-machine-pattern

2. Cliffle - "The Typestate Pattern in Rust" (2019)
   - https://cliffle.com/blog/rust-typestate

3. Yoric - "Typestates in Rust" (2018)
   - https://yoric.github.io/post/rust-typestate

4. Yoshua Wuyts - "State Machines: Introduction" (2020)
   - https://blog.yoshuawuyts.com/state-machines

5. rustype/typestate-rs - Proc macro DSL for typestates
   - https://github.com/rustype/typestate-rs

6. machine crate - State machine code generation
   - https://docs.rs/machine

7. Embedded Rust Book - Typestate Programming
   - https://docs.rust-embedded.org/book/static-guarantees/typestate-programming.html

---

*Research compiled for Lithos project, focusing on state machine patterns for schema loading and validation pipelines.*
