# Rust Idioms (Rust Design Patterns Book)

**Source:** https://rust-unofficial.github.io/patterns/idioms/index.html
**Book repo:** https://github.com/rust-unofficial/patterns
**Purpose:** A paraphrased reference of the _15 idioms_ listed in the Rust Design Patterns book’s Idioms chapter (treating the FFI subsection as one idiom group).

For code organization and style guidance (separate from “idioms”), see [docs/refs/rust/style.md](style.md).

This document is intentionally written in our own words and focuses on “what to do” and “what to avoid” when writing Rust.

## Quick Index (15)

1. Use borrowed types for arguments
2. Concatenate strings with `format!`
3. Constructors (`new`) and “named constructors”
4. The `Default` trait
5. Collections are smart pointers (`Deref` to borrowed views)
6. Finalisation in destructors (`Drop` as a `finally`-like hook)
7. `mem::{take, replace}` to move out of `&mut` / enum variants
8. On-stack dynamic dispatch (`&dyn Trait` / `&mut dyn Trait`)
9. Foreign Function Interface (FFI) idioms (group)
10. Iterating over an `Option`
11. Pass variables to closure (scope rebinding)
12. Privacy for extensibility (`#[non_exhaustive]`, private fields)
13. Easy doc initialization (rustdoc helper patterns)
14. Temporary mutability (make it immutable when done)
15. Return consumed argument on error

---

## 1) Use borrowed types for arguments

**Core idea:** Accept the most general *borrowed view* of data, not a borrowed owning container.

### What it looks like

Prefer:
- `fn f(name: &str)` over `fn f(name: &String)`
- `fn f(items: &[T])` over `fn f(items: &Vec<T>)`
- `fn f(path: &Path)` over `fn f(path: &PathBuf)`

This composes with Rust’s *deref coercions* and with APIs that naturally yield borrowed views (e.g., `split()` yields `&str`, slice indexing yields `&[T]`).

### Why this is idiomatic

- **More callers:** `&String` does not accept `"literal"` or `&str` directly, but `&str` accepts both `String` and `&'static str` via coercion.
- **Avoids extra indirection:** `String` is already an owning pointer-ish container. `&String` adds another layer; `&str` is the “thin” borrowed view.
- **Clear ownership semantics:** The signature communicates that you only need read access, not ownership.

### Guidance

- If you only need to read: accept a borrowed view.
- If you need ownership: accept `String`/`Vec<T>`/`PathBuf>`.
- If you want “owned or borrowed”: prefer `Cow<'a, str>` / `Cow<'a, [T]>`.
- If you just need “something that can be referenced as”: consider `impl AsRef<Path>` or `impl AsRef<str>` for APIs that are called a lot from many contexts.

### Common pitfalls

- Accepting `&String` tends to force allocations at call sites (e.g., `split()` callers can’t pass a `&String` without collecting into owned `String`s).
- Taking `String` when you only read it forces callers to allocate/clones or give up ownership.

**Example**

```rust
use std::path::Path;

fn render_title(title: &str, out_dir: &Path) {
    let _ = (title, out_dir);
}
```

Note: Prefer `&Path`/`PathBuf` rather than `String` paths.

---

## 2) Concatenate strings with `format!`

**Core idea:** Prefer the most readable construction for the context. `format!` is usually the clearest; manual pushing can be faster when building large strings in tight loops.

### When to use `format!`

- Mixed literals + variables.
- Anything user-facing (errors, diagnostics, logging) where clarity matters.
- Places where you don’t want to think about capacity management.

```rust
fn greet(name: &str) -> String {
    format!("Hello {name}!")
}
```

### When *not* to use it

If you are building large strings in a hot path:
- consider pre-allocating and pushing (`String::with_capacity`, `push_str`, `push`).
- or use `std::fmt::Write` (`write!(&mut s, ...)`) to append formatted content without creating intermediate `String`s.

```rust
use std::fmt::Write;

fn join_with_commas(items: &[&str]) -> String {
    let mut s = String::new();
    for (i, item) in items.iter().enumerate() {
        if i != 0 {
            s.push(',');
        }
        // Append formatted content directly into `s`.
        let _ = write!(&mut s, "{item}");
    }
    s
}
```

### Trade-offs

- `format!` is **usually** the best readability/performance compromise.
- Repeated `format!` in a loop can allocate each iteration; the “push/write into a buffer” approach avoids that.

Note: Prefer readability by default; only optimize after profiling.

---

## 3) Constructors (`new`) and named constructors

**Core idea:** Rust doesn’t have constructors as a language feature, so communities converge on conventions.

### Primary constructor

- If there is an “obvious” way to build a type, expose `Type::new(...)`.
- If the type can be created with no parameters, it’s common to support both `new()` and `Default`.

### Alternate constructors

Prefer explicit naming to convey semantics:
- `from_parts(...)`, `from_bytes(...)`, `with_capacity(...)`, `from_path(...)`

### Fallible construction

Do not hide fallibility:
- Use `TryFrom` / `TryInto` for conversions.
- Or use `try_new(...) -> Result<Self, Error>` when the API reads better.

### Guidance

- Keep `new` small and unsurprising.
- Put invariants in one place: have other constructors delegate to the core validation path.
- If construction has “many knobs”, prefer the Builder pattern (outside the idioms chapter, but commonly used in Rust).

---

## 4) The `Default` trait

**Core idea:** `Default` is Rust’s standard abstraction for “a sensible baseline value”. It composes in generic contexts where `new()` cannot be expressed.

### Why it matters

- Enables `..Default::default()` struct update syntax.
- Enables `Option::unwrap_or_default`, `Result::unwrap_or_default`, and many `*_or_default()` APIs.
- Makes configuration structs easy to partially specify.

### Good uses

- Config structs with optional fields.
- “Builder-like” accumulator structs.
- Collections and caches (empty baseline state).

```rust
#[derive(Default, Debug)]
struct Config {
    verbose: bool,
    max_items: usize,
}

fn config_for_cli() -> Config {
    Config {
        verbose: true,
        ..Default::default()
    }
}
```

### Bad uses

- Domain entities where “default” would violate invariants or be ambiguous.

Note: Prefer `Default` for config-like types, but keep invariants explicit.

---

## 5) Collections are smart pointers (`Deref` to a borrowed view)

**Core idea:** When a type owns data, it’s often helpful to also provide a borrowed view of that data and funnel most methods onto that view.

### The standard pattern

- Owning type implements `Deref<Target = BorrowedView>`.
- Most methods live on the borrowed view type.

Common examples:
- `Vec<T>` derefs to `[T]`.
- `String` derefs to `str`.

### Why it matters

- Reduces duplication: implement methods once on the borrowed view.
- Improves ergonomics: methods on the view “show up” on the owning type through auto-deref.
- Supports APIs that take borrowed views (`&[T]`, `&str`).

### Guidance

- Consider `AsRef`/`Borrow` alongside `Deref` when you need trait bounds in generic contexts.
- Consider `Index<Range>` for slicing syntax if your type supports it.

### Pitfall (important)

- Don’t use `Deref` to emulate inheritance or “method forwarding” between unrelated types. That’s the deref-polymorphism anti-pattern.

---

## 6) Finalisation in destructors (`Drop` as a `finally`-like hook)

**Core idea:** If you need cleanup on all exit paths (including early returns and `?`), bind a guard value and let `Drop` do the work.

### When it’s useful

- Releasing resources (locks, file handles, temporary directories).
- Restoring global state (temporarily changing a setting, then reverting).
- Ensuring spans/metrics/notifications are emitted on function exit.

### The guard pattern

Key details that trip people up:
- The guard must live until the end of the scope.
- Bind it to a variable (otherwise it drops immediately).
- If it’s unused, name it with a leading underscore like `_guard` (but not exactly `_`).

```rust
struct Guard;

impl Drop for Guard {
    fn drop(&mut self) {
        // Must not panic.
    }
}

fn do_work() {
    let _guard = Guard;
    // ... any early return here still runs `drop`.
}
```

### Limitations

- `Drop` is very reliable but not absolute (e.g., aborts, double panics).
- Never put “must-run” external side effects exclusively in `Drop`.

Note: `drop` implementations should not panic.

---

## 7) `mem::{take, replace}` to move out of `&mut` / enum variants

**Core idea:** You cannot move an owned field out of a `&mut` reference without leaving something behind. Use `mem::take` (default) or `mem::replace` (explicit placeholder) to swap.

### Typical scenario

- You have `&mut Enum` and want to rewrite it into another variant while reusing an owned field (e.g., `String`) without cloning.

```rust
use std::mem;

enum State {
    A { name: String, x: u8 },
    B { name: String },
}

fn rewrite(s: &mut State) {
    if let State::A { name, x: 0 } = s {
        *s = State::B {
            name: mem::take(name),
        };
    }
}
```

### Guidance

- Prefer `Option::take()` when the field is `Option<T>`.
- Use `mem::replace` when `T: Default` is not available or default is not appropriate.

### Trade-offs

- More verbose than cloning.
- Sometimes the compiler can’t optimize away the “double store” as well as the hand-written unsafe-language version.

---

## 8) On-stack dynamic dispatch (`&dyn Trait` / `&mut dyn Trait`)

**Core idea:** Use trait objects by reference when you want dynamic dispatch without allocating a `Box`.

### When it’s a good fit

- “Plumbing” code (CLI, IO routing, adapters) where avoiding monomorphization reduces compile-time and code size.
- When you have a small number of possible concrete types but want one variable.

```rust
use std::fs;
use std::io::{self, Read};

fn read_from_arg(arg: &str) -> io::Result<Vec<u8>> {
    let mut input: &mut dyn Read = if arg == "-" {
        &mut io::stdin()
    } else {
        &mut fs::File::open(arg)?
    };

    let mut buf = Vec::new();
    input.read_to_end(&mut buf)?;
    Ok(buf)
}
```

### Trade-offs

- Slight runtime overhead per call due to dynamic dispatch.
- Some APIs become object-safety constrained.

---

## 9) Foreign Function Interface (FFI) idioms (group)

**Core idea:** FFI is primarily about *defining and defending invariants* at a boundary where Rust’s type system cannot help you.

Note: This section is still worth understanding for external integration work and for recognizing good boundary design.

### 9.1) Idiomatic errors across FFI boundaries

**Goal:** C callers need simple, stable results; Rust wants structured errors.

Guidelines:
- Return an **integer error code** (or a `#[repr(C)]` flat enum) from exported functions.
- If you need richer detail, expose a separate “error description” accessor that returns a C string (or writes into a caller-provided buffer).
- Keep the error code domain small, stable, and well-documented.

Pitfalls:
- Returning Rust-specific layouts (e.g., `Result<T, E>` directly) is not ABI-stable.
- Returning pointers without a clear ownership/freeing story causes leaks or UAF.

### 9.2) Accepting strings from FFI

**Goal:** Minimize unsafe and treat foreign strings as borrowed data.

Guidelines:
- Convert `*const c_char` to `&CStr` as early as possible.
- Convert `&CStr` to `&str` with fallible UTF-8 conversion (`to_str()`); decide what to do on invalid UTF-8.
- Keep unsafe blocks small and isolated.

Pitfalls:
- Doing manual `strlen`/pointer arithmetic is error-prone.
- Assuming UTF-8 without checking is a correctness bug.

### 9.3) Passing strings to FFI

**Goal:** Ensure the backing buffer lives long enough and that ownership does not accidentally transfer.

Guidelines:
- Bind `CString` to a local, then pass `.as_ptr()`.
- Never take `.as_ptr()` from a temporary `CString`.
- If the foreign API mutates the buffer, don’t use `CString`—use a `Vec<u8>` buffer you own.

Pitfall:
- Dangling pointers from temporaries are a classic “looks fine, is UB” bug.

---

## 10) Iterating over an `Option`

**Core idea:** `Option<T>` is a “0-or-1 element container” and participates naturally in iterator-based code.

### Useful patterns

- Append if present:

```rust
let mut v = vec![1, 2, 3];
let maybe = Some(4);
v.extend(maybe);
```

- Chain onto existing iteration:

```rust
let items = vec!["a", "b"];
let maybe = Some("c");
for s in items.iter().chain(maybe.iter()) {
    let _ = s;
}
```

### Guidance

- Prefer `if let Some(x) = opt { ... }` for standalone control flow.
- Prefer iterator adapters when it makes surrounding iterator code *simpler*.

### Pitfalls

- Be explicit about ownership:
  - `opt.iter()` yields `&T`
  - `opt.iter_mut()` yields `&mut T`
  - `opt.into_iter()` yields `T` (moves)
- If you always have exactly one element, `std::iter::once(x)` is clearer than `Some(x).into_iter()`.

---

## 11) Pass variables to closure (scope rebinding)

**Core idea:** Control how a closure captures its environment (move/clone/borrow) by rebinding inside a small scope.

### Why it’s useful

- Keeps “capture shaping” colocated with the closure definition.
- Avoids a trail of `*_cloned` / `*_borrowed` variables.
- Ensures any temporary clones are dropped with the closure.

```rust
use std::sync::Arc;

fn make_handler(shared: Arc<String>, prefix: String) -> impl Fn() + Send + Sync + 'static {
    let handler = {
        let shared = Arc::clone(&shared); // explicit clone of the Arc, not the inner String
        let prefix = prefix; // move `prefix`
        move || {
            let _ = (&shared, &prefix);
        }
    };
    handler
}
```

### Guidance

- Prefer cloning `Arc<T>` (cheap) over cloning `T` (potentially expensive).
- Be deliberate about `move`: it moves captured values into the closure.
- If you only need references, avoid `move` and borrow instead (but mind lifetimes).

---

## 12) Privacy for extensibility (`#[non_exhaustive]`, private fields)

**Core idea:** If an API must be extensible without semver-major changes, force callers to write non-exhaustive patterns.

### `#[non_exhaustive]`

- Works across crate boundaries.
- Forces `..` in struct patterns and forces a wildcard arm in enum matches.

### Private field technique

- Useful inside a crate or when you want a similar effect without `#[non_exhaustive]`.
- Add a private `()` field to prevent exhaustive construction/matching.

### Guidance

- Use deliberately: it reduces ergonomics.
- Prefer a semver-major bump when adding variants/fields is a meaningful behavior change.

Note: Use `#[non_exhaustive]` deliberately: it improves forward compatibility but reduces ergonomics.

---

## 13) Easy doc initialization (rustdoc helper patterns)

**Core idea:** Keep examples readable without drowning in setup, while still keeping examples compile-checked.

### Techniques

- **Hidden setup lines** using rustdoc’s `#` prefix.
- **Helper function wrapper** when you need `no_run` or large boilerplate.

```rust
/// # Example
/// ```
/// # fn demo(x: i32) -> i32 { x + 1 }
/// assert_eq!(2, demo(1));
/// ```
fn _placeholder() {}
```

### Important caveat

If you wrap an example in a helper function *and never call it*, assertions don’t run. The snippet is still type-checked, which is often the goal for `no_run` examples.

### Guidance

- Use `no_run` when the example depends on OS/network state.
- Prefer examples that actually run in unit tests when feasible.

Lithos note: Public traits and key APIs should have doc examples where practical (see project context rules).

---

## 14) Temporary mutability

**Core idea:** Use `mut` only during a preparation phase, then rebind to an immutable binding to make “no further mutation” a compiler-enforced property.

### Two idiomatic forms

Nested block:

```rust
let data = {
    let mut data = get_data();
    data.sort();
    data
};
```

Rebinding:

```rust
let mut data = get_data();
data.sort();
let data = data; // now immutable
```

### Why it matters

- Communicates intent to readers.
- Prevents accidental mutation later in the function.
- Can make borrow-checker interactions clearer (immutable borrows become more flexible).

---

## 15) Return consumed argument on error

**Core idea:** If an operation must take ownership of an input (consume it) and can fail, return the input back inside the error so the caller can recover without cloning.

### Why it matters

- Avoids “clone before calling, just in case”.
- Enables retry logic, fallback strategies, or logging the original value after failure.

### Canonical examples

- `std::sync::mpsc::SendError<T>` contains the unsent `T`.
- `String::from_utf8(Vec<u8>)` returns `FromUtf8Error`, which can yield the original `Vec<u8>`.

### Pattern sketch

```rust
pub struct SendError(pub String);

pub fn send(value: String) -> Result<(), SendError> {
    // ... attempt sending, may fail
    Err(SendError(value))
}
```

### Guidance

- Use when the input is expensive to clone or naturally owned.
- Keep error types small and focused: the whole point is “here is your value back”.

---

## Appendix: Adjacent practices (Apollo handbook)

The Apollo `rust-best-practices` handbook includes several “idiom-like” practices that are useful in Lithos, but they are **not** part of the Rust Design Patterns book’s canonical 15-idiom list. They’re captured here to keep this file a good one-stop reference.

### A1) Prefer lazy fallbacks to avoid early allocation

Many `Option`/`Result` helpers have an eager form and a lazy form. Prefer the lazy form when the fallback does work (allocation, formatting, I/O, etc.).

- Prefer `unwrap_or_else(...)` over `unwrap_or(...)` when the fallback allocates.
- Prefer `ok_or_else(...)` over `ok_or(...)` when the error construction allocates.
- Prefer `map_or_else(...)` over `map_or(...)` for the same reason.

### A2) Use `let ... else` for early exits when the else-branch is simple

If the “unhappy path” is a simple early return / continue / break, `let PATTERN = expr else { ... };` tends to flatten control flow and make the happy path stand out.

If the else-branch needs heavier computation, prefer `if let ... { ... } else { ... }` or `match`.

### A3) Pass small `Copy` types by value

For small `Copy` types (e.g. integers, bools, small plain-old-data structs), passing by value is often clearer and just as efficient as passing by reference. Reserve `&T` for larger values, non-`Copy` data, or when borrowing is semantically important.
