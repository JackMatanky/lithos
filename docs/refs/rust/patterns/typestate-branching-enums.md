# Branching Enum Patterns in Rust Typestate Machines

Research findings on when and how to use branching enums in typestate patterns, including alternatives, trade-offs, and real-world implementations.

## Table of Contents

1. [When Branching Enums Are Preferred](#when-branching-enums-are-preferred)
2. [Nested Match Ergonomics and Solutions](#nested-match-ergonomics-and-solutions)
3. [Result-like Enums for Typestate Branches](#result-like-enums-for-typestate-branches)
4. [Sealed Traits for Controlled Branching](#sealed-traits-for-controlled-branching)
5. [Macro-Based DSLs for State Machines](#macro-based-dsls-for-state-machines)
6. [Real-World Examples](#real-world-examples)

---

## When Branching Enums Are Preferred

### The Typestate Pattern Foundation

The typestate pattern encodes an object's runtime state in its compile-time type. Three key properties:

1. **State-specific operations**: Operations only available in certain states
2. **Type-level encoding**: States encoded at the type level, catching errors at compile-time
3. **State transitions**: Methods that change the type-level state (consume and return different types)

### Branching Enum vs Type Parameter Approaches

#### Type Parameter Approach (Non-Branching)

```rust
// Each state is a separate type, generic struct holds state
struct HttpResponse<S: ResponseState> {
    state: Box<ActualResponseState>,
    marker: std::marker::PhantomData<S>,
}

enum Start {} // Zero-variant enum (phantom type)
enum Headers {}

trait ResponseState {}
impl ResponseState for Start {}
impl ResponseState for Headers {}

impl HttpResponse<Start> {
    fn status_line(self, code: u8) -> HttpResponse<Headers> { /* ... */ }
}

impl HttpResponse<Headers> {
    fn header(&mut self, key: &str, value: &str) { /* ... */ }
    fn body(self, contents: &str) { /* ... */ }
}
```

**Advantages:**
- All operations appear in single rustdoc page (better discoverability)
- Easy to add operations valid in multiple states (unconstrained impl blocks)
- Can add state-specific fields by making state types concrete (not phantom)
- Linear state progression is clear from type signatures

**When to prefer:**
- States follow a clear linear progression (A → B → C → D)
- Need to store different data in different states
- Want unified documentation
- Complex state hierarchies with shared behavior

#### Branching Enum Approach

```rust
// Enum encodes all possible states
enum State {
    Green,
    Yellow,
    Red,
}

impl State {
    pub fn new() -> Self::Green { Self::Green }

    pub fn next(self: Self::Green) -> Self::Yellow { Self::Yellow }
    pub fn next(self: Self::Yellow) -> Self::Red { Self::Red }
    pub fn next(self: Self::Red) -> Self::Green { Self::Green }
}
```

**Note:** This is a *future direction* proposal (not stable Rust). Current limitation: enum variants aren't fully qualified types, so you can't use `Self::Green` as a return type or in `self` position.

**When to prefer (if available):**
- All states known upfront and unlikely to change
- States form a closed set (no extension by downstream crates)
- Branching/cyclical state transitions (not just linear)
- Want to match on state variants directly

### Current Best Practice: Separate Types + Separate Structs

The most common production pattern today:

```rust
struct HttpResponse { state: Box<ActualResponseState> }
struct HttpResponseAfterStatus { state: Box<ActualResponseState> }

impl HttpResponse {
    fn status_line(self, code: u8, msg: &str) -> HttpResponseAfterStatus {
        // Consume self, return new state
    }
}

impl HttpResponseAfterStatus {
    fn header(self, key: &str, value: &str) -> Self { /* ... */ }
    fn body(self, text: &str) { /* consume, return nothing */ }
}
```

**Advantages:**
- Works today (no experimental features)
- Zero-cost abstraction (moves consume values)
- Impossible to use wrong operations in wrong state
- Clean separation of concerns

**Trade-offs:**
- Method chaining requires returning `self` from each method
- Need to reassign in loops: `r = r.header(k, v)` (forgetting `r =` is a compile error)
- Boilerplate for multiple state types

---

## Nested Match Ergonomics and Solutions

### The Problem: Deep Nesting with Branching

Parser combinators and protocol implementations often need to branch on multiple nested states:

```rust
// nom parser example
pub type IResult<I, O, E> = Result<(I, O), Err<E>>;

pub enum Err<E> {
    Incomplete(Needed),
    Error(E),
    Failure(E),
}

// Nested matching becomes verbose
match parser1(input) {
    Ok((rest, value1)) => match parser2(rest) {
        Ok((rest2, value2)) => match parser3(rest2) {
            Ok((rest3, value3)) => Ok((rest3, combine(value1, value2, value3))),
            Err(e) => Err(e),
        },
        Err(e) => Err(e),
    },
    Err(e) => Err(e),
}
```

### Solution 1: `?` Operator + Result Chaining

```rust
fn parse_sequence(input: &str) -> IResult<&str, Output> {
    let (input, value1) = parser1(input)?;
    let (input, value2) = parser2(input)?;
    let (input, value3) = parser3(input)?;
    Ok((input, combine(value1, value2, value3)))
}
```

**Key insight:** The `?` operator works with any type implementing `Try` trait (Result, Option, etc.)

### Solution 2: Combinator Functions

nom's approach: Build complex parsers from simple combinators

```rust
use nom::branch::alt;
use nom::sequence::{preceded, terminated};
use nom::combinator::complete;

// alt() tries parsers in sequence, returns first success
let parser = alt((tag("ab"), tag("cd")));

// Composition reduces nesting
fn json_string(input: &str) -> IResult<&str, String> {
    preceded(
        char('"'),
        terminated(parse_str, char('"'))
    )(input)
}
```

**Branching combinators:**
- `alt()`: Try list of parsers, return first success
- `permutation()`: Match all parsers in any order
- `switch()`: Choose parser based on first result

### Solution 3: Poll-Based State Machines (Async Pattern)

Avoid nested branching by flattening state into a struct with `poll()` method:

```rust
pub struct Join<FutureA, FutureB> {
    // Option prevents polling after completion
    a: Option<FutureA>,
    b: Option<FutureB>,
}

impl<FutureA, FutureB> Future for Join<FutureA, FutureB> {
    type Output = ();

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<()> {
        // Attempt to complete future `a`
        if let Some(a) = &mut self.a {
            if let Poll::Ready(()) = a.poll(cx) {
                self.a.take(); // Remove completed future
            }
        }

        // Attempt to complete future `b`
        if let Some(b) = &mut self.b {
            if let Poll::Ready(()) = b.poll(cx) {
                self.b.take();
            }
        }

        if self.a.is_none() && self.b.is_none() {
            Poll::Ready(())
        } else {
            Poll::Pending
        }
    }
}
```

**Key pattern:** State stored in fields, poll logic uses early returns + flat structure

### Solution 4: `select!` Macro for Concurrent Branching

Tokio's approach to waiting on multiple branches:

```rust
use tokio::select;

loop {
    select! {
        _ = &mut future => { break; }
        Some(val) = stream.next() => {
            println!("got value = {}", val);
        }
        _ = &mut timeout => {
            println!("timeout");
            break;
        }
    }
}
```

**Pattern:** Macro generates flattened polling code from clean syntax

---

## Result-like Enums for Typestate Branches

### The Pattern: Two-Variant Success/Failure Split

Result-like enums provide a natural way to represent branching after validation or computation:

```rust
// nom's approach: 3-way branching
pub enum Err<E> {
    Incomplete(Needed),  // Need more input
    Error(E),            // Recoverable error (try next parser)
    Failure(E),          // Unrecoverable error (abort parsing)
}

pub type IResult<I, O, E> = Result<(I, O), Err<E>>;
```

**Why 3-way instead of Result?**
- `Incomplete`: Streaming parser needs more data (can't decide yet)
- `Error`: Try alternative branch (e.g., `alt()` combinator tries next option)
- `Failure`: Abort entire parse (cut commitment to current branch)

### Pattern: Nested Result for Multi-Phase Validation

Lithos-style example (from project context):

```rust
// Phase 1: Syntax validation (parsing)
pub enum RawParseError {
    InvalidFormat(String),
    MissingField(&'static str),
}

// Phase 2: Semantic validation (resolution)
pub enum ResolveError {
    UndefinedReference(Id),
    CyclicDependency(Vec<Id>),
    DepthExceeded { max: usize, actual: usize },
}

// Two-phase result
type LoadResult<T> = Result<Result<T, ResolveError>, RawParseError>;

// Usage shows natural branching
match parse_file(path) {
    Err(syntax_err) => {
        // Syntax error: can't proceed to validation
        log_parse_error(syntax_err);
    }
    Ok(Err(semantic_err)) => {
        // Valid syntax, invalid semantics
        log_resolution_error(semantic_err);
    }
    Ok(Ok(schema)) => {
        // Fully valid
        storage.save(schema)?;
    }
}
```

### Pattern: State-Carrying Error Enums

Recovery pattern: Include consumed input in error for retry:

```rust
pub enum ValidationError<T> {
    Invalid {
        input: T,           // Return consumed input
        reason: String,
    },
    Irrecoverable(String),  // Input was consumed/lost
}

// Enables retry with modifications
match validate(input) {
    Ok(result) => use_result(result),
    Err(ValidationError::Invalid { input, reason }) => {
        // Can retry with modified input
        retry_with_fallback(input, reason)
    }
    Err(ValidationError::Irrecoverable(msg)) => {
        // Input lost, must abort
        abort(msg)
    }
}
```

**From Rust idioms:** *"For fallible operations that consume an input, prefer returning the consumed value on failure when it materially improves recovery."*

---

## Sealed Traits for Controlled Branching

### The Problem: Uncontrolled Trait Implementation

Type-parameter typestate allows downstream crates to add new states:

```rust
// In your library
pub trait ResponseState {}
pub struct Start;
pub struct Headers;
impl ResponseState for Start {}
impl ResponseState for Headers {}

// Downstream user can add states you didn't anticipate!
struct TheirWeirdState;
impl ResponseState for TheirWeirdState {} // Compiles!
```

### Solution: Sealed Trait Pattern

```rust
mod private {
    pub trait Sealed {}
}

// Public trait extends private sealed trait
pub trait ResponseState: private::Sealed {}

pub struct Start;
pub struct Headers;

// Only you can impl Sealed (it's private)
impl private::Sealed for Start {}
impl private::Sealed for Headers {}

// Public impls still work
impl ResponseState for Start {}
impl ResponseState for Headers {}

// Downstream can't impl ResponseState because they can't impl Sealed!
```

**Use when:**
- State set is closed (finite, non-extensible)
- Internal invariants depend on exhaustive state knowledge
- Want to add states in semver-minor updates without breaking changes

**Reference:** [Sealed traits protect against downstream implementations (API Guidelines)](https://rust-lang.github.io/api-guidelines/future-proofing.html#c-sealed)

### Advanced: Sealed Trait Hierarchies

```rust
mod private {
    pub trait Sealed {}
}

// Base trait: sealed
pub trait State: private::Sealed {
    fn name(&self) -> &'static str;
}

// Sub-trait for states that allow sending data
pub trait SendingState: State {
    fn bytes_sent(&self) -> usize;
}

struct Start;
struct Headers { sent: usize }
struct Complete;

impl private::Sealed for Start {}
impl private::Sealed for Headers {}
impl private::Sealed for Complete {}

impl State for Start {
    fn name(&self) -> &'static str { "Start" }
}
impl State for Headers {
    fn name(&self) -> &'static str { "Headers" }
}
impl State for Complete {
    fn name(&self) -> &'static str { "Complete" }
}

// Only Headers implements SendingState
impl SendingState for Headers {
    fn bytes_sent(&self) -> usize { self.sent }
}

// Operations available only in sending states
impl<S: SendingState> HttpResponse<S> {
    fn spam_spam_spam(&mut self) { /* ... */ }
}
```

---

## Macro-Based DSLs for State Machines

### Motivation: Reduce Boilerplate

Typestate machines generate significant boilerplate:

```rust
// For each state: separate type + trait impl
struct StateA;
struct StateB;
struct StateC;

// For each transition: method consuming old state, returning new
impl StateA {
    fn to_b(self) -> StateB { StateB }
}
impl StateB {
    fn to_c(self) -> StateC { StateC }
}
```

Macros can generate this from declarative syntax.

### Approach 1: Declarative Macro (TT Muncher Pattern)

Pattern from `typestate-rs` crate (proc-macro DSL):

```rust
#[typestate]
mod TrafficLight {
    #[automaton]
    pub struct TrafficLight;

    #[state]
    pub struct Green;
    #[state]
    pub struct Yellow;
    #[state]
    pub struct Red;

    impl TrafficLight {
        pub fn next(Green) -> Yellow { /* ... */ }
        pub fn next(Yellow) -> Red { /* ... */ }
        pub fn next(Red) -> Green { /* ... */ }
    }
}
```

**TT Muncher technique:**
1. Match first item in token stream
2. Process it (invoke callback with matched tokens)
3. Recurse on remaining tokens
4. Base case: empty stream

Example from the macro tutorial:

```rust
macro_rules! visit_members {
    (
        $callback:ident;

        $( #[$attr:meta] )*
        fn $name:ident(&self $(, $arg_name:ident : $arg_ty:ty )*) $(-> $ret:ty)?;

        $( $rest:tt )*  // <-- rest of input
    ) => {
        // Invoke callback with matched function
        $callback!(
            $( #[$attr] )*
            fn $name(&self $(, $arg_name : $arg_ty )*) $(-> $ret)?
        );

        // Recurse on rest
        visit_members! { $callback; $($rest)* }
    };

    // Base case: empty
    ($callback:ident;) => {};
}
```

### Approach 2: Callback Pattern (Continuation-Passing Style)

Pass macro name as parameter to invoke later:

```rust
macro_rules! generate_trait_impls {
    (
        $( #[$attr:meta] )*
        trait $name:ident {
            $( $body:tt )*
        }
    ) => {
        // Emit trait
        $( #[$attr] )*
        trait $name { $( $body )* }

        // Generate impls via callback
        impl_trait_for_ref! {
            $( #[$attr] )*
            trait $name { $( $body )* }
        }
        impl_trait_for_box! {
            $( #[$attr] )*
            trait $name { $( $body )* }
        }
    };
}

macro_rules! impl_trait_for_ref {
    ( trait $name:ident { $( $body:tt )* } ) => {
        impl<'a, T: $name + ?Sized> $name for &'a T {
            visit_members!( call_via_deref; $($body)* );
        }
    };
}

macro_rules! call_via_deref {
    ( fn $name:ident(&self) -> $ret:ty ) => {
        fn $name(&self) -> $ret {
            (**self).$name()  // Deref twice: & and pointer
        }
    };
}
```

**Pattern benefits:**
- Separation of concerns (scanning vs generation)
- Reusable callbacks
- Incremental development (add callbacks as needed)

### Approach 3: Search-and-Replace with Conditionals

Conditional code generation via pattern matching:

```rust
/// Only invoke callback if `&mut self` NOT found in token stream
macro_rules! search_for_mut_self {
    // Found it: stop, don't invoke callback
    ($callback:ident!($($args:tt)*); &mut self $($rest:tt)*) => {};
    ($callback:ident!($($args:tt)*); (&mut self $($other_args:tt)*) $($rest:tt)*) => {};

    // Keep searching
    ($callback:ident!($($args:tt)*); $_head:tt $($rest:tt)*) => {
        search_for_mut_self!($callback!($($args)*); $($rest)*);
    };

    // Not found: invoke callback
    ($callback:ident!($($args:tt)*);) => {
        $callback!($($args)*)
    };
}

// Usage: only impl Trait for &T if no &mut self methods
search_for_mut_self! {
    impl_trait_for_ref!(trait Foo { $($body)* });

    $($body)*  // Search through trait body
}
```

### Testing Macros

From matklad's article on testing:

```rust
#[track_caller]  // Show test failure location in test, not in check()
fn check(input: &str, expected: &str) {
    let actual = my_macro_expansion(input);
    assert_eq!(expected, actual);
}

#[test]
fn test_simple_state_machine() {
    check(
        "state A -> B",
        "impl From<A> for B { ... }"
    );
}
```

**compile_error!() debugging trick:**

```rust
macro_rules! debug_tokens {
    ( $($tokens:tt)* ) => {
        compile_error!(
            concat!(
                $( stringify!($tokens), " " ),*
            )
        );
    };
}
```

---

## Real-World Examples

### Example 1: Parser Combinators (nom)

**Branching strategy:** Result-like enum with 3 outcomes

```rust
pub type IResult<I, O, E> = Result<(I, O), Err<E>>;

pub enum Err<E> {
    Incomplete(Needed),  // Streaming: need more input
    Error(E),            // Recoverable: try alternative
    Failure(E),          // Unrecoverable: abort
}
```

**State progression:**
- Input → Parse → (Success + Remaining) | (Error type)
- Streaming parsers maintain input position across `Incomplete` returns

**Branching combinators:**

```rust
use nom::branch::alt;

// Try parsers in order, return first success
let parser = alt((
    tag("GET"),
    tag("POST"),
    tag("PUT"),
));

// Equivalent to nested match:
match tag("GET")(input) {
    ok @ Ok(_) => ok,
    Err(Err::Error(_)) => match tag("POST")(input) {
        ok @ Ok(_) => ok,
        Err(Err::Error(_)) => tag("PUT")(input),
        err => err,
    },
    err => err,
}
```

### Example 2: Async Runtimes (Tokio)

**Branching strategy:** Poll-based state machine with Pin

```rust
pub trait Future {
    type Output;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output>;
}

pub enum Poll<T> {
    Ready(T),
    Pending,
}
```

**Concurrent branching with select!:**

```rust
use tokio::{pin, select};

let fut1 = async_operation_1();
let fut2 = async_operation_2();
pin!(fut1);
pin!(fut2);

select! {
    result1 = &mut fut1 => handle_result1(result1),
    result2 = &mut fut2 => handle_result2(result2),
}
```

**Key pattern:** State stored in struct fields, polled until Ready

```rust
// Generated by async fn
enum AsyncFnState {
    Start,
    AwaitingFoo { future: FooFuture },
    AwaitingBar { foo_result: Foo, future: BarFuture },
    Done,
}

impl Future for AsyncFnFuture {
    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Output> {
        loop {
            match self.state {
                Start => { /* transition to AwaitingFoo */ }
                AwaitingFoo { ref mut future } => {
                    match Pin::new(future).poll(cx) {
                        Poll::Ready(foo) => { /* transition to AwaitingBar */ }
                        Poll::Pending => return Poll::Pending,
                    }
                }
                AwaitingBar { foo_result, ref mut future } => {
                    match Pin::new(future).poll(cx) {
                        Poll::Ready(bar) => { /* transition to Done */ }
                        Poll::Pending => return Poll::Pending,
                    }
                }
                Done => panic!("polled after completion"),
            }
        }
    }
}
```

### Example 3: HTTP Protocol Implementation (Typestate)

**Branching strategy:** Linear type progression with branching at error points

```rust
struct HttpResponse<S: ResponseState> {
    state: Box<ActualState>,
    _marker: PhantomData<S>,
}

struct Start;
struct AfterStatus { code: u8 }
struct AfterHeaders { code: u8, headers: Vec<(String, String)> }

impl HttpResponse<Start> {
    fn status_line(self, code: u8, msg: &str) -> HttpResponse<AfterStatus> {
        // ...
    }
}

impl HttpResponse<AfterStatus> {
    fn header(mut self, key: &str, val: &str) -> Self { /* ... */ }
    fn into_headers(self) -> HttpResponse<AfterHeaders> { /* ... */ }
}

impl HttpResponse<AfterHeaders> {
    fn body(self, bytes: &[u8]) -> Result<(), ProtocolError> {
        // Terminal state: consumes self, returns nothing
    }
}
```

**Branching decision:** Error cases break linearity

```rust
impl HttpResponse<AfterHeaders> {
    fn body(self, bytes: &[u8]) -> Result<(), ProtocolError> {
        if bytes.len() > MAX_BODY_SIZE {
            // Error branch: can't proceed
            Err(ProtocolError::BodyTooLarge)
        } else {
            // Success: send and complete
            self.state.write_body(bytes)?;
            Ok(())
        }
    }
}
```

### Example 4: Lithos Schema Resolution (Multi-Phase Validation)

**Branching strategy:** Nested Result for syntax vs semantic validation

```rust
// From lithos-core architecture
pub mod raw {
    // Syntax-only validation
    pub struct RawSchema { /* ... */ }

    impl RawSchema {
        pub fn parse(text: &str) -> Result<Self, ParseError> {
            // Regex/type validation only
        }
    }
}

pub mod domain {
    // Semantic validation
    pub struct Schema { /* ... */ }

    impl Schema {
        pub fn resolve(raw: RawSchema, ctx: &Context)
            -> Result<Self, ResolveError>
        {
            // Check refs exist, no cycles, depth limits
        }
    }
}

// Two-phase loading
pub fn load_schema(path: &Path)
    -> Result<Result<Schema, ResolveError>, ParseError>
{
    let text = fs::read_to_string(path)?;
    let raw = RawSchema::parse(&text)?;  // Syntax
    let schema = Schema::resolve(raw, &ctx)?;  // Semantics
    Ok(Ok(schema))
}
```

**Branching insight:** Separate error types for different failure modes enables:
- Different recovery strategies (retry parse vs abort on semantic error)
- Different error reporting (syntax error points at source location, semantic error shows reference chain)
- Partial pipeline reuse (cache parsed output, re-validate on context changes)

---

## Summary: When to Use Each Pattern

| Pattern | Use When | Avoid When |
|---------|----------|------------|
| **Separate struct types** | Linear state progression, need compile-time guarantees, production code | States are dynamic, many states with shared behavior |
| **Type parameter + phantom** | Need unified docs, states share data layout, want subset operations | State data differs significantly, downstream extension needed |
| **Branching enum** (future) | Cyclical transitions, closed state set, want to match on state | Not available in stable Rust yet |
| **Result-like enum** | Multi-phase validation, recoverable errors, streaming | Only two outcomes (use Result), single validation pass |
| **Sealed traits** | Closed state set, internal invariants, prevent extension | Want downstream extensibility |
| **Macro DSL** | High boilerplate, many similar state machines, team knowledge | Simple state machine, macro expertise lacking, compile time matters |
| **Poll-based** | Async state machines, need to pause/resume, zero-cost futures | Synchronous code, simple linear flow |
| **TT muncher** | Process token stream item-by-item, callbacks, conditional generation | Simple substitution (use regular macro) |

---

## References

- [Cliffle: The Typestate Pattern in Rust](https://cliffle.com/blog/rust-typestate/)
- [Yoshua Wuyts: State Machines in Rust](https://blog.yoshuawuyts.com/state-machines/)
- [Embedded Rust Book: Typestate Programming](https://docs.rust-embedded.org/book/static-guarantees/typestate-programming.html)
- [typestate-rs: Proc-macro typestate DSL](https://github.com/rustype/typestate-rs)
- [The Little Book of Rust Macros: TT Munchers](https://danielkeep.github.io/tlborm/book/pat-incremental-tt-munchers.html)
- [Michael F. Bryan: Writing Non-Trivial Macros in Rust](https://adventures.michaelfbryan.com/posts/non-trivial-macros/)
- [Without Boats: Asynchronous Destructors](https://without.boats/blog/poll-drop/)
- [Rust Async Book: The Future Trait](https://rust-lang.github.io/async-book/02_execution/02_future.html)
- [nom: Parser Combinator Framework](https://docs.rs/nom)
- [tokio: Async Runtime Documentation](https://docs.rs/tokio)
