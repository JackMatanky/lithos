# Rust Best Practices - Canonical

Source: https://canonical.github.io/rust-best-practices/

---

## Introduction

This book provides a list of best-practices which originate from discussions with both our CTO and the company's other tech leads. All points listed here are strongly considered before merging code at Canonical. Individually, the things written here may seem unimportant or even trivial, but each of these is a crucial building block for writing good, clean code. You wouldn't want to hire a builder who has a reputation for installing rotten beams.

This book aims to facilitate peaceful collaboration by expressing an opinion on many common discrepancies, allowing debate to focus on more interesting areas. It encourages uniformity between projects and teams with the aim of allowing engineers in one part of an organisation to feel reasonably at-home in codebases from other parts. This goal is notably different from individual works in unique contexts, where the use of highly novel approaches is a healthy form of self-expression. These guidelines should be considered as a style guide for 'Canonical Rust,' and are intended to complement both the idiomatic advice laid out in Rust for Rustaceans and well thought-out, consistent API design.

If you spot any of the problems detailed here in our existing code-bases or patches, don't be afraid to fix them—a good codebase is inherently more maintainable and will cause fewer headaches and annoyances later. Remember that all code should aim to be locally-consistent—new code shouldn't stick out like a sore thumb.

*Disclaimer: this is not a complete list, more items can and likely will be added in future.*

*If you find an item which you believe should be in this document, please do open an [RFC](https://github.com/canonical/rust-best-practices/issues/new/choose).*

### Further Reading

Checkout the [`rust-analyzer` style guide](https://github.com/rust-lang/rust-analyzer/blob/master/docs/book/src/contributing/style.md). Note that those style guidelines are intended for a single project, hence it is possible for its developers to make more specific requirements which only make sense in the context of that project, unlike the guidelines provided here.

### Preconditions

All new code should abide by `cargo fmt`, `cargo clippy`, and `cargo clippy --tests`. If your crate uses features, be careful to ensure that `clippy` is definitely being run on all of your code, for example by using `--all-features`.

---

## Cosmetic Discipline

### Spacing

Use blank lines semantically, rather than aesthetically. They should be used consistently, regardless of the size of a section of code, to delimit *strongly associated* sections. There are no hard and fast rules for this strong association, but the following heuristics are quite effective.

- If a variable is declared and only used in the block of code which follows it, that declaration and block are strongly associated. Do not put a blank line here.
- If a variable is used in multiple blocks of code, not just the one which follows it, that declaration is not strongly associated with the block immediately after it. Put a blank line here.
- If a variable is declared and then checked, the declaration and check are strongly associated and must not be separated by a blank line. If the check contains more than three lines, the declaration and check start to form their own strongly associated block so require a blank line after.

✅ Do this:

```rust
type Result<T> = std::result::Result<T, Error>;

enum Error {
    Invalid,
    NetworkUnavailable,
    MalformedEnvUrl {
        env_var: &'static str,
        source: Box<dyn std::error::Error>,
    },
    Unsupported,
    Unknown(Box<dyn std::error::Error>),
}

impl<E> From<E> for Error
where
    E: std::error::Error + 'static,
{
    fn from(err: E) -> Self {
        Error::Unknown(Box::new(err))
    }
}
struct Arbitrary;

impl Arbitrary {
    fn is_valid(&self) -> bool {
        true
    }
}

impl std::ops::Add for Arbitrary {
    type Output = Self;

    fn add(self, _: Self) -> Self {
        Self
    }
}

impl std::fmt::Display for Arbitrary {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Arbitrary")
    }
}

fn foo() -> Arbitrary {
    Arbitrary
}
fn bar() -> Arbitrary {
    Arbitrary
}
fn baz() -> Arbitrary {
    Arbitrary
}
fn snippet() -> Result<Arbitrary> {
let x = foo();
if !x.is_valid() {
    return Err(Error::Invalid);
}
println!("{x}");

let y = baz();
if !y.is_valid() {
    return Err(Error::Invalid);
}
return Ok(y);
}
```

⚠️ Avoid this:

```rust
type Result<T> = std::result::Result<T, Error>;

enum Error {
    Invalid,
    NetworkUnavailable,
    MalformedEnvUrl {
        env_var: &'static str,
        source: Box<dyn std::error::Error>,
    },
    Unsupported,
    Unknown(Box<dyn std::error::Error>),
}

impl<E> From<E> for Error
where
    E: std::error::Error + 'static,
{
    fn from(err: E) -> Self {
        Error::Unknown(Box::new(err))
    }
}
struct Arbitrary;

impl Arbitrary {
    fn is_valid(&self) -> bool {
        true
    }
}

impl std::ops::Add for Arbitrary {
    type Output = Self;

    fn add(self, _: Self) -> Self {
        Self
    }
}

impl std::fmt::Display for Arbitrary {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Arbitrary")
    }
}

fn foo() -> Arbitrary {
    Arbitrary
}
fn bar() -> Arbitrary {
    Arbitrary
}
fn baz() -> Arbitrary {
    Arbitrary
}
fn snippet() -> Result<Arbitrary> {
let x = foo();

if !x.is_valid() {
    return Err(Error::Invalid);
}

println!("{x}");
let y = baz();
if !y.is_valid() {
    return Err(Error::Invalid);
}

return Ok(y);
}
```

### Grouping

Don't interleave unrelated code. Remember, to a new reader, this will look deliberate and they become confused about how variables relate. Keep it clean and group together strongly intradependent sections of code.

This is particularly significant where closures are bound to variables—if a closure is defined as such halfway through a function, does not capture anything and then is only used at the end, the reader will have to keep more things in mind for no good reason. If values are captured, declare such closures close as possible to where they're needed. Otherwise, define an `fn` function, rather than a closure with `|| …`. Also, consider whether a closure is required at all—although it may be tempting to define helper closures, code may feel cleaner with a simpler, more top-to-bottom flow control pattern. Logic should always feel clean and be easy to follow.

*The following snippets assume that functions `foo`, `bar` and `baz` are free of side-effects.*

✅ Do this:

```rust
type Result<T> = std::result::Result<T, Error>;

enum Error {
    Invalid,
    NetworkUnavailable,
    MalformedEnvUrl {
        env_var: &'static str,
        source: Box<dyn std::error::Error>,
    },
    Unsupported,
    Unknown(Box<dyn std::error::Error>),
}

impl<E> From<E> for Error
where
    E: std::error::Error + 'static,
{
    fn from(err: E) -> Self {
        Error::Unknown(Box::new(err))
    }
}
struct Arbitrary;

impl Arbitrary {
    fn is_valid(&self) -> bool {
        true
    }
}

impl std::ops::Add for Arbitrary {
    type Output = Self;

    fn add(self, _: Self) -> Self {
        Self
    }
}

impl std::fmt::Display for Arbitrary {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Arbitrary")
    }
}

fn foo() -> Arbitrary {
    Arbitrary
}
fn bar() -> Arbitrary {
    Arbitrary
}
fn baz() -> Arbitrary {
    Arbitrary
}
fn snippet() -> Result<()> {
let x = foo();
let b = baz();
if !b.is_valid() {
    return Err(Error::Invalid);
}
let z = x + b;

let y = bar();
if !y.is_valid() {
    return Err(Error::Invalid);
}
Ok(())
}
```

⚠️ Avoid this:

```rust
type Result<T> = std::result::Result<T, Error>;

enum Error {
    Invalid,
    NetworkUnavailable,
    MalformedEnvUrl {
        env_var: &'static str,
        source: Box<dyn std::error::Error>,
    },
    Unsupported,
    Unknown(Box<dyn std::error::Error>),
}

impl<E> From<E> for Error
where
    E: std::error::Error + 'static,
{
    fn from(err: E) -> Self {
        Error::Unknown(Box::new(err))
    }
}
struct Arbitrary;

impl Arbitrary {
    fn is_valid(&self) -> bool {
        true
    }
}

impl std::ops::Add for Arbitrary {
    type Output = Self;

    fn add(self, _: Self) -> Self {
        Self
    }
}

impl std::fmt::Display for Arbitrary {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Arbitrary")
    }
}

fn foo() -> Arbitrary {
    Arbitrary
}
fn bar() -> Arbitrary {
    Arbitrary
}
fn baz() -> Arbitrary {
    Arbitrary
}
fn snippet() -> Result<()> {
type X = Arbitrary;
let x = foo();
let check = |x: X| {
    if !x.is_valid() {
        return Err(Error::Invalid)
    }
    Ok(x)
};
let y = bar();
let z = x + check(baz())?;
check(y)?;
Ok(())
}
```

### Hex Values

Unless there is an existing convention, hex values should be lowercase as this avoids creating visually-impenetrable rectangles. By using lowercase, we provide more 'handles' for the eye to use.

✅ Do this:

```rust
const SOME_SPECIFIC_IMPORTANT_VALUE: u64 = 0xab5c4d320974a3bc;
```

⚠️ Avoid this:

```rust
const SOME_SPECIFIC_IMPORTANT_VALUE: u64 = 0xAB5C4D320974A3BC;
```

---

## Naming Discipline

### Name Content

Naming is one of the three hardest problems in programming (along with off-by-one errors). Every variable, every function, every type and every concept requires a good name which fits into a good naming scheme. There is no one optimal way to come up with a good name, however when attempting to do so, the first place to look is for similar names in your project and to try to mimic these. This should result in a name which intuitively feels like it belongs among the rest of the code. Even doing this has its pitfalls, however, so ideally your name should:

- **say what it means** —make the name fit conceptually into the surrounding context. If a reader sees `fn is_in(a: &str, b: &str)`, the order is not as obvious as if they were to see `fn is_in(haystack: &str, needle: &str)`.
- **have a consistent word order** —inconsistency makes an API look dishevelled, unplanned and hence unprofessional. If the rest of the API uses `verb_noun` then unless there is a very good reason not to, the next function should be of the form `verb_noun`.
- **be concise** —a long name can almost always be shortened. More characters implies a need to disambiguate, so if no such need exists, reduce the cognitive load on the next reader by reducing the amount they must read. Of course, don't take this too far—the next reader must not be expected to look elsewhere to understand the full meaning of a name, as may occur if nonstandard acronyms or abbreviations are used.
- **comprise simple words** —a long word can often be replaced by a shorter one. A concise name will comprise the smallest list of the smallest words which do not lose the subject's meaning. Remember: [thesaurus.com](https://www.thesaurus.com/) and [dictionary.com](https://www.dictionary.com/) are your friends!
- **comprise correct words** —if there is any disagreement over the implications of chosen words, then there will be some reader who gets the wrong idea. It's better to spend more time discussing internally than to confuse a user. (Example: we once had a 20-minute discussion on the choice between `NOT_SAFE` vs `UNSAFE` as an empty value for a set of safety flags, where each flag had a name like `FOO_SAFE`.)
- **be unified** —there should be one and only one name for concepts used. If more are used haphazardly, it implies a difference where there is none and thus muddies the water.
- **avoid including types** —type names should be omitted unless required to discriminate between two variables of different types which roughly hold the same value. Some examples: in a finder function `needleStr` and `haystackStr` can be more concisely expressed as `needle` and `haystack`.

Canonical policy dictates that names should use UK spelling and not US or other spelling.

Good names with the help of concise doc comments do a good job of explaining a good API. However, if after much consideration, there don't seem to be any good names, this is likely caused by the API not being good. If an API cannot be easily and intuitively explained, it is not a good API and it's time for a refactor.

A good, semantically and behaviorally consistent API hidden behind a layer of bad naming is hard to distinguish from a bad API. Time spent getting the right naming will pay off.

Great care should be taken over all names, but extreme care should be taken over publicly exposed ones. These names do not have the luxury of being able to be tweaked without consequence later—any appreciation for a slightly better name an external user may have will be completely overshadowed by their irritation of having to deal with a breaking change.

### Pattern Match Variable Naming

To reduce cognitive load, pattern-matched variables should be named consistently, that is, the new variable must be either:

- The same as the variable/field it comes from
- The first letter of the variable/field it comes from

When matching structs and struct-like enum variants, try to use the original field names.

✅ Do this:

```rust
type Result<T> = std::result::Result<T, Error>;

enum Error {
    Invalid,
    NetworkUnavailable,
    MalformedEnvUrl {
        env_var: &'static str,
        source: Box<dyn std::error::Error>,
    },
    Unsupported,
    Unknown(Box<dyn std::error::Error>),
}

impl<E> From<E> for Error
where
    E: std::error::Error + 'static,
{
    fn from(err: E) -> Self {
        Error::Unknown(Box::new(err))
    }
}
struct Arbitrary;

impl Arbitrary {
    fn builder() -> Self {
        Self
    }

    fn bar(self, _: Self) -> Self {
        Self
    }

    fn build(self) -> Result<Self> {
        Ok(Self)
    }
}

struct Event {
    response: Option<()>,
}

struct File {
    name: (),
    path: (),
}
let response = Some(());
if let Some(response) = response { /* ... */ }

let event = Event { response: None };
if let Some(response) = event.response { /* ... */ }
if let Some(event_response) = event.response { /* ... */ }

impl File {
  fn f(self) {
let Self { name, path } = self;
  }
}

use std::fs;
type Workload = ();
enum State {
    Reading(fs::File),
    Evaluating {
        workload: Workload,
      other_fields: (),
        /* ... */
    }
}
let state = State::Evaluating {
    workload: (),
    other_fields: ()
};
match state {
    State::Reading(file) => { /* ... */ }
    State::Evaluating{ workload, .. } => { /* ... */ }
}
```

⚠️ Avoid this:

```rust
type Result<T> = std::result::Result<T, Error>;

enum Error {
    Invalid,
    NetworkUnavailable,
    MalformedEnvUrl {
        env_var: &'static str,
        source: Box<dyn std::error::Error>,
    },
    Unsupported,
    Unknown(Box<dyn std::error::Error>),
}

impl<E> From<E> for Error
where
    E: std::error::Error + 'static,
{
    fn from(err: E) -> Self {
        Error::Unknown(Box::new(err))
    }
}
struct Arbitrary;

impl Arbitrary {
    fn builder() -> Self {
        Self
    }

    fn bar(self, _: Self) -> Self {
        Self
    }

    fn build(self) -> Result<Self> {
        Ok(Self)
    }
}

struct Event {
    response: Option<()>,
}

struct File {
    name: (),
    path: (),
}
let response = Some(());
if let Some(r) = response { /* ... */ }

let event = Event { response: None };
if let Some(r) = event.response { /* ... */ }
if let Some(er) = event.response { /* ... */ }

impl File {
  fn f(self) {
let Self { name: some_name, path: name } = self;
  }
}

use std::fs;
type Workload = ();
enum State {
    Reading(fs::File),
    Evaluating {
        workload: Workload,
        other_fields: (),
    }
}
let state = State::Evaluating {
    workload: (),
    other_fields: ()
};
match state {
    State::Reading(data_source) => { /* ... */ }
    State::Evaluating{ workload: to_eval, .. } => { /* ... */ }
}
```

### Generic Type Parameter Naming

To avoid them being mistaken for concrete types, generic type parameters must have single-letter names.

### Lifetime Parameter Naming

Lifetime parameters give a unique opportunity to link together different parts of an API in the mind of a user. If the same data is used in multiple places in an API, make sure the lifetime names match.

Lifetime names should be derived from the reference they represent, not from the type they are passed to. Further, they should consist of either a single letter, short word or extremely well understood acronym. Numbers should not be used in lifetime names.

Single-letter lifetime names are acceptable if a structure is expected to be used very many times (e.g. a script interface may make heavy use of some `Value<'h>` which contains a reference to a heap upon which it is allocated). NB: the compiler will occasionally recommend the use `'a` as it lacks wider context information. The name `'a` is nearly always a bad one.

Lifetime parameters should aim to be concise without losing meaning. Given the difficulty new users face when understanding lifetimes in an interface, try to give them a hand by being explicit. Single-letter lifetime names should generally be avoided.

✅ Do this:

```rust
struct ASTQueryMatch<'cursor, 'tree> {
    /* ... */
ignore: &'cursor &'tree (),
}

struct Value<'h> {
    /* ... */
ignore: &'h (),
}
```

⚠️ Avoid this:

```rust
struct ASTQueryMatch<'a, 'b> {
    /* ... */
ignore: &'a &'b (),
}

struct Value<'value> {
    /* ... */
ignore: &'value (),
}
```

### Builder Naming

If a builder for a type `MyType` is provided, then it should have an associated function `builder()` which returns a `MyTypeBuilder`. This `MyTypeBuilder` must also have a fallible `.build()` method, which returns a `MyType`.

Typical usage is hence—

```rust
type Result<T> = std::result::Result<T, Error>;

enum Error {
    Invalid,
    NetworkUnavailable,
    MalformedEnvUrl {
        env_var: &'static str,
        source: Box<dyn std::error::Error>,
    },
    Unsupported,
    Unknown(Box<dyn std::error::Error>),
}

impl<E> From<E> for Error
where
    E: std::error::Error + 'static,
{
    fn from(err: E) -> Self {
        Error::Unknown(Box::new(err))
    }
}
struct Arbitrary;

impl Arbitrary {
    fn builder() -> Self {
        Self
    }

    fn bar(self, _: Self) -> Self {
        Self
    }

    fn build(self) -> Result<Self> {
        Ok(Self)
    }
}

struct Event {
    response: Option<()>,
}

struct File {
    name: (),
    path: (),
}
fn snippet() -> Result<()> {
type Foo = Arbitrary;
let bar = Arbitrary;
let foo = Foo::builder()
    .bar(bar)
    // ...
    .build()?;
Ok(())
}
```

---

## Import Discipline

### Don't Import All

In general, do not use `*` from a crate. Consider a source file which does this twice from two different dependencies, making use of items from each. Now, consider what happens when these crates are updated and some of these items are removed—the compiler will complain of undefined symbols, but will have absolutely no idea where these came from. Even more concerning is the fact that updates can now cause name-clashes! By sticking to explicit imports only, we help both ourselves and our future maintainers.

A corollary of this is that preludes, regardless of their initial convenience, should not be used by us in production code. Nevertheless, they remain a handy tool for others to use when prototyping, so we should still consider creating and exposing them where appropriate.

Do not bring enum variants into scope using `*` as this obscures the types and in some cases the fact that an enum is being handled. If the name of an enum is too long, can't reasonably be edited and the problematic usage is in a small scope, it may be renamed locally using `use … as …`. The new name should be an acronym of the type used, e.g. `TaskStatus` would be shortened to `Ts`. Due to scoping rules around `use`, these renaming statements should be placed at the top of the function definition which requires it.

The only exception to these rules is that in the context of a unit test module, inserting use `super::*` is acceptable as it is a well-defined idiom.

✅ Do this:

```rust
type Result<T> = std::result::Result<T, Error>;

enum Error {
    Invalid,
    NetworkUnavailable,
    MalformedEnvUrl {
        env_var: &'static str,
        source: Box<dyn std::error::Error>,
    },
    Unsupported,
    Unknown(Box<dyn std::error::Error>),
}

impl<E> From<E> for Error
where
    E: std::error::Error + 'static,
{
    fn from(err: E) -> Self {
        Error::Unknown(Box::new(err))
    }
}

mod some_crate {
    pub struct SpecificItem1;
    pub struct SpecificItem2;
}

mod some_other_crate {
    pub mod prelude {}
    pub struct SpecificItem3;
}

mod another_crate {
    pub enum SomeEnum {
        Variant1,
        Variant2,
    }
}

mod allocative {
    pub struct Allocative;
}

mod derive_more {
    pub struct Display;
}

mod starlark {
    pub mod environment {
        pub struct FrozenModule;
        pub struct Module;
    }

    pub mod eval {
        pub struct Evaluator;
    }

    pub mod values {
        pub struct AllocValue;
        pub struct Freeze;
        pub struct ProvidesStaticType;
        pub struct StarlarkValue;
        pub struct ValueLike;
    }
}

mod starlark_derive {
    pub fn starlark_value() {}
    pub struct NoSerialize;
    pub struct Trace;
}
use some_crate::{SpecificItem1, SpecificItem2};
use some_other_crate::SpecificItem3;
use another_crate::SomeEnum;

fn some_fn(some_enum: SomeEnum) -> SomeEnum {
    use SomeEnum as Se;
    match some_enum {
        Se::Variant1 => { /* ... */ }
        Se::Variant2 => { /* ... */ },
    }
    Se::Variant2
}
```

⚠️ Avoid this:

```rust
type Result<T> = std::result::Result<T, Error>;

enum Error {
    Invalid,
    NetworkUnavailable,
    MalformedEnvUrl {
        env_var: &'static str,
        source: Box<dyn std::error::Error>,
    },
    Unsupported,
    Unknown(Box<dyn std::error::Error>),
}

impl<E> From<E> for Error
where
    E: std::error::Error + 'static,
{
    fn from(err: E) -> Self {
        Error::Unknown(Box::new(err))
    }
}

mod some_crate {
    pub struct SpecificItem1;
    pub struct SpecificItem2;
}

mod some_other_crate {
    pub mod prelude {}
    pub struct SpecificItem3;
}

mod another_crate {
    pub enum SomeEnum {
        Variant1,
        Variant2,
    }
}

mod allocative {
    pub struct Allocative;
}

mod derive_more {
    pub struct Display;
}

mod starlark {
    pub mod environment {
        pub struct FrozenModule;
        pub struct Module;
    }

    pub mod eval {
        pub struct Evaluator;
    }

    pub mod values {
        pub struct AllocValue;
        pub struct Freeze;
        pub struct ProvidesStaticType;
        pub struct StarlarkValue;
        pub struct ValueLike;
    }
}

mod starlark_derive {
    pub fn starlark_value() {}
    pub struct NoSerialize;
    pub struct Trace;
}
use some_crate::*;
use some_other_crate::prelude::*;
use another_crate::{SomeEnum, SomeEnum::*};

fn some_fn(some_enum: SomeEnum) -> SomeEnum {
    match some_enum {
        Variant1 => { /* ... */ }
        Variant2 => { /* ... */ }
    }
    Variant2
}
```

### Import Grouping

At the time of writing, stable `rustfmt` does not yet express an opinion on the order of imports, hence for now we must do this ourselves. To clearly delimit whether an import is from the standard library, from a third party library or our own work, these imports should be split into three blocks, ordered as follows:

- `std`, `core`, `alloc`
- Third party crates
- `self`, `super`, `crate`

Note that this order follows the currently-unstable `rustfmt` option—

```toml
import_group = "StdExternCrate"
```

✅ Do this:

```rust
use std::path::PathBuf;

use camino::Utf8PathBuf;
use tokio::runtime::Runtime;

use crate::{Error, Result};
```

⚠️ Avoid this:

```rust
type Result<T> = std::result::Result<T, Error>;

enum Error {
    Invalid,
    NetworkUnavailable,
    MalformedEnvUrl {
        env_var: &'static str,
        source: Box<dyn std::error::Error>,
    },
    Unsupported,
    Unknown(Box<dyn std::error::Error>),
}

impl<E> From<E> for Error
where
    E: std::error::Error + 'static,
{
    fn from(err: E) -> Self {
        Error::Unknown(Box::new(err))
    }
}

mod some_crate {
    pub struct SpecificItem1;
    pub struct SpecificItem2;
}

mod some_other_crate {
    pub mod prelude {}
    pub struct SpecificItem3;
}

mod another_crate {
    pub enum SomeEnum {
        Variant1,
        Variant2,
    }
}

mod allocative {
    pub struct Allocative;
}

mod derive_more {
    pub struct Display;
}

mod starlark {
    pub mod environment {
        pub struct FrozenModule;
        pub struct Module;
    }

    pub mod eval {
        pub struct Evaluator;
    }

    pub mod values {
        pub struct AllocValue;
        pub struct Freeze;
        pub struct ProvidesStaticType;
        pub struct StarlarkValue;
        pub struct ValueLike;
    }
}

mod starlark_derive {
    pub fn starlark_value() {}
    pub struct NoSerialize;
    pub struct Trace;
}
use camino::Utf8PathBuf;
use crate::{Error, Result};
use std::path::PathBuf;
use tokio::runtime::Runtime;
```

### Import Form

Excessive repetition harms readability by adding lots of visual noise. For this reason, avoid Java-style imports where every single imported item gets its own line with a complete and exhaustive path and instead use the cleaner nested syntax. If a path contains *n* parts, merge the first *n-1,* so that only the final part is grouped with its siblings.

✅ Do this:

```rust
type Result<T> = std::result::Result<T, Error>;

enum Error {
    Invalid,
    NetworkUnavailable,
    MalformedEnvUrl {
        env_var: &'static str,
        source: Box<dyn std::error::Error>,
    },
    Unsupported,
    Unknown(Box<dyn std::error::Error>),
}

impl<E> From<E> for Error
where
    E: std::error::Error + 'static,
{
    fn from(err: E) -> Self {
        Error::Unknown(Box::new(err))
    }
}

mod some_crate {
    pub struct SpecificItem1;
    pub struct SpecificItem2;
}

mod some_other_crate {
    pub mod prelude {}
    pub struct SpecificItem3;
}

mod another_crate {
    pub enum SomeEnum {
        Variant1,
        Variant2,
    }
}

mod allocative {
    pub struct Allocative;
}

mod derive_more {
    pub struct Display;
}

mod starlark {
    pub mod environment {
        pub struct FrozenModule;
        pub struct Module;
    }

    pub mod eval {
        pub struct Evaluator;
    }

    pub mod values {
        pub struct AllocValue;
        pub struct Freeze;
        pub struct ProvidesStaticType;
        pub struct StarlarkValue;
        pub struct ValueLike;
    }
}

mod starlark_derive {
    pub fn starlark_value() {}
    pub struct NoSerialize;
    pub struct Trace;
}
use allocative::Allocative;
use derive_more::Display;
use starlark::environment::{FrozenModule, Module};
use starlark::eval::Evaluator;
use starlark::values::{AllocValue, Freeze, ProvidesStaticType, StarlarkValue, ValueLike};
use starlark_derive::{starlark_value, NoSerialize, Trace};
```

⚠️ Avoid this:

```rust
type Result<T> = std::result::Result<T, Error>;

enum Error {
    Invalid,
    NetworkUnavailable,
    MalformedEnvUrl {
        env_var: &'static str,
        source: Box<dyn std::error::Error>,
    },
    Unsupported,
    Unknown(Box<dyn std::error::Error>),
}

impl<E> From<E> for Error
where
    E: std::error::Error + 'static,
{
    fn from(err: E) -> Self {
        Error::Unknown(Box::new(err))
    }
}

mod some_crate {
    pub struct SpecificItem1;
    pub struct SpecificItem2;
}

mod some_other_crate {
    pub mod prelude {}
    pub struct SpecificItem3;
}

mod another_crate {
    pub enum SomeEnum {
        Variant1,
        Variant2,
    }
}

mod allocative {
    pub struct Allocative;
}

mod derive_more {
    pub struct Display;
}

mod starlark {
    pub mod environment {
        pub struct FrozenModule;
        pub struct Module;
    }

    pub mod eval {
        pub struct Evaluator;
    }

    pub mod values {
        pub struct AllocValue;
        pub struct Freeze;
        pub struct ProvidesStaticType;
        pub struct StarlarkValue;
        pub struct ValueLike;
    }
}

mod starlark_derive {
    pub fn starlark_value() {}
    pub struct NoSerialize;
    pub struct Trace;
}
use allocative::Allocative;
use derive_more::Display;
use starlark::environment::FrozenModule;
use starlark::environment::Module;
use starlark::eval::Evaluator;
use starlark::values::AllocValue;
use starlark::values::Freeze;
use starlark::values::ProvidesStaticType;
use starlark::values::StarlarkValue;
use starlark::values::ValueLike;
use starlark_derive::starlark_value;
use starlark_derive::NoSerialize;
use starlark_derive::Trace;
```

### Import Self Explicitly

When importing from a child module `foo`, always `use self::foo` as this avoids future name-clashes with a dependency also called `foo`.

✅ Do this:

```rust
mod foo;
mod bar;

pub use self::foo::Foo;
pub use self::bar::Bar;
```

⚠️ Avoid this:

```rust
mod foo;
mod bar;

pub use foo::Foo;
pub use bar::Bar;
```

---

## Pattern Matching Discipline

### Exhaustively Match to Draw Attention

Pattern matching is an excellent way to ensure that all items of data in internal structures have been considered, not only by the author of the current change, but also by the authors of any future changes. When using internal interfaces, always consider using pattern-matching to force useful compiler errors in case important, possibly new, parts of a structure haven't been considered. This in turn will draw the attention of the next maintainer and help them correctly do what they need.

✅ Do this:

```rust
type Result<T> = std::result::Result<T, Error>;

enum Error {
    Invalid,
    NetworkUnavailable,
    MalformedEnvUrl {
        env_var: &'static str,
        source: Box<dyn std::error::Error>,
    },
    Unsupported,
    Unknown(Box<dyn std::error::Error>),
}

impl<E> From<E> for Error
where
    E: std::error::Error + 'static,
{
    fn from(err: E) -> Self {
        Error::Unknown(Box::new(err))
    }
}

#[derive(Eq, PartialEq, PartialOrd)]
struct MyStruct {
    my: String,
    thing: String,
    with: String,
    some: String,
    unused: String,
    fields: String,
}

struct Line {
    gradient: f64,
    y_intercept: f64,
}

struct Server;
struct ServerConfig {
    db_path: (),
    working_path: (),
}
use std::cmp::Ordering;
impl Ord for MyStruct {
    fn cmp(&self, other: &Self) -> Ordering {
        let Self {
            my,
            thing,
            with,
            some,
            unused: _,
            fields: _,
        } = self;
        (my, thing, with, some)
            .cmp(&(&other.my, &other.thing, &other.with, &other.some))
    }
}
```

⚠️ Avoid this:

```rust
type Result<T> = std::result::Result<T, Error>;

enum Error {
    Invalid,
    NetworkUnavailable,
    MalformedEnvUrl {
        env_var: &'static str,
        source: Box<dyn std::error::Error>,
    },
    Unsupported,
    Unknown(Box<dyn std::error::Error>),
}

impl<E> From<E> for Error
where
    E: std::error::Error + 'static,
{
    fn from(err: E) -> Self {
        Error::Unknown(Box::new(err))
    }
}

#[derive(Eq, PartialEq, PartialOrd)]
struct MyStruct {
    my: String,
    thing: String,
    with: String,
    some: String,
    unused: String,
    fields: String,
}

struct Line {
    gradient: f64,
    y_intercept: f64,
}

struct Server;
struct ServerConfig {
    db_path: (),
    working_path: (),
}
use std::cmp::Ordering;
impl Ord for MyStruct {
    fn cmp(&self, other: &Self) -> Ordering {
        (&self.my, &self.thing, &self.with, &self.some)
            .cmp(&(&other.my, &other.thing, &other.with, &other.some))
    }
}
```

### Don't Pattern-match References

It is possible to pattern-match the references to a `Copy` type to obtain the value at the other end. Although it may seem convenient, it ultimately harms readability—it is clearer to explicitly dereference the reference we are given.

✅ Do this:

```rust
[0].iter()
    .map(|x| *x)
;
```

⚠️ Avoid this:

```rust
[0].iter()
    .map(|&x| x)
;
```

### Avoid Numeric Tuple-indexing

Although sometimes a handy shorthand, indexing tuples with `.0`, `.1` etc. deprives us of the opportunity to insert a good name in the same way that field-access on a struct would. Instead, prefer to use pattern-matching to give human-friendly names to the data being handled. Note that this advice does not apply in the `impl` blocks of newtype-pattern structs, i.e. tuple-structs with a single element (commonly used for wrapper types).

✅ Do this:

```rust
type Result<T> = std::result::Result<T, Error>;

enum Error {
    Invalid,
    NetworkUnavailable,
    MalformedEnvUrl {
        env_var: &'static str,
        source: Box<dyn std::error::Error>,
    },
    Unsupported,
    Unknown(Box<dyn std::error::Error>),
}

impl<E> From<E> for Error
where
    E: std::error::Error + 'static,
{
    fn from(err: E) -> Self {
        Error::Unknown(Box::new(err))
    }
}

#[derive(Eq, PartialEq, PartialOrd)]
struct MyStruct {
    my: String,
    thing: String,
    with: String,
    some: String,
    unused: String,
    fields: String,
}

struct Line {
    gradient: f64,
    y_intercept: f64,
}

struct Server;
struct ServerConfig {
    db_path: (),
    working_path: (),
}
fn line_through(point1: (f64, f64), point2: (f64, f64)) -> Line {
    let (x1, y1) = point1;
    let (x2, y2) = point2;
    let gradient = (y2 - y1) / (x2 - x1);
    let y_intercept = y1 - gradient * x1;
    Line {
        gradient,
        y_intercept,
    }
}
```

⚠️ Avoid this:

```rust
type Result<T> = std::result::Result<T, Error>;

enum Error {
    Invalid,
    NetworkUnavailable,
    MalformedEnvUrl {
        env_var: &'static str,
        source: Box<dyn std::error::Error>,
    },
    Unsupported,
    Unknown(Box<dyn std::error::Error>),
}

impl<E> From<E> for Error
where
    E: std::error::Error + 'static,
{
    fn from(err: E) -> Self {
        Error::Unknown(Box::new(err))
    }
}

#[derive(Eq, PartialEq, PartialOrd)]
struct MyStruct {
    my: String,
    thing: String,
    with: String,
    some: String,
    unused: String,
    fields: String,
}

struct Line {
    gradient: f64,
    y_intercept: f64,
}

struct Server;
struct ServerConfig {
    db_path: (),
    working_path: (),
}
fn line_through(point1: (f64, f64), point2: (f64, f64)) -> Line {
    let gradient = (point2.1 - point1.1) / (point2.0 - point1.0);
    let y_intercept = point1.1 - gradient * point1.0;
    Line {
        gradient,
        y_intercept,
    }
}
```

### Pattern-matched Parameters

Using pattern matching in `fn` parameters adds extra noise to a function's signature by duplicating definitions held elsewhere. Indeed, the fact that a particular parameter is to be pattern-matched inside of the function is not important to the user—it is an unwelcome implementation detail and should be hidden as such.

If parameters are to be unpacked, do this at the first line of a particular function.

Note that this guidance does not apply to closures, which are commonly used as small, locally-scoped helper functions, whose types are inferred.

✅ Do this:

```rust
type Result<T> = std::result::Result<T, Error>;

enum Error {
    Invalid,
    NetworkUnavailable,
    MalformedEnvUrl {
        env_var: &'static str,
        source: Box<dyn std::error::Error>,
    },
    Unsupported,
    Unknown(Box<dyn std::error::Error>),
}

impl<E> From<E> for Error
where
    E: std::error::Error + 'static,
{
    fn from(err: E) -> Self {
        Error::Unknown(Box::new(err))
    }
}

#[derive(Eq, PartialEq, PartialOrd)]
struct MyStruct {
    my: String,
    thing: String,
    with: String,
    some: String,
    unused: String,
    fields: String,
}

struct Line {
    gradient: f64,
    y_intercept: f64,
}

struct Server;
struct ServerConfig {
    db_path: (),
    working_path: (),
}
impl Server {
    fn new(config: ServerConfig) -> Result<Self> {
        let ServerConfig { db_path, working_path } = config;
        /* ... */
      Ok(Server)
    }
}
```

⚠️ Avoid this:

```rust
type Result<T> = std::result::Result<T, Error>;

enum Error {
    Invalid,
    NetworkUnavailable,
    MalformedEnvUrl {
        env_var: &'static str,
        source: Box<dyn std::error::Error>,
    },
    Unsupported,
    Unknown(Box<dyn std::error::Error>),
}

impl<E> From<E> for Error
where
    E: std::error::Error + 'static,
{
    fn from(err: E) -> Self {
        Error::Unknown(Box::new(err))
    }
}

#[derive(Eq, PartialEq, PartialOrd)]
struct MyStruct {
    my: String,
    thing: String,
    with: String,
    some: String,
    unused: String,
    fields: String,
}

struct Line {
    gradient: f64,
    y_intercept: f64,
}

struct Server;
struct ServerConfig {
    db_path: (),
    working_path: (),
}
impl Server {
    fn new(ServerConfig { db_path, working_path }: ServerConfig) -> Result<Self> {
        /* ... */
      Ok(Server)
    }
}
```

---

## Code Discipline

### When to Use `Self`

Use `Self` wherever possible to reduce the number of types which the reader must keep in mind as instead of needing to remember an extra type which may or may not be important, the reader is instead reminded that the current code strongly relates to the `impl` block it is contained in. By maximising the use of `Self` it also highlights where it *isn't* possible to use it, for example when the return type of a function has a slightly different set of generic parameters.

✅ Do this:

```rust
type Result<T> = std::result::Result<T, Error>;

enum Error {
    Invalid,
    NetworkUnavailable,
    MalformedEnvUrl {
        env_var: &'static str,
        source: Box<dyn std::error::Error>,
    },
    Unsupported,
    Unknown(Box<dyn std::error::Error>),
}

impl<E> From<E> for Error
where
    E: std::error::Error + 'static,
{
    fn from(err: E) -> Self {
        Error::Unknown(Box::new(err))
    }
}

struct Arbitrary;
impl Arbitrary {
    fn new() -> Self {
        Self
    }

    fn something_else(self) -> Self {
        self
    }

    fn another_thing(self) -> Self {
        self
    }

    fn chained_with_something_else(self) -> Result<Self> {
        Ok(self)
    }

    fn format_as(self, fmt: &str) -> String {
        String::new()
    }
}

fn some_long_computation() -> Result<Arbitrary> {
    Ok(Arbitrary)
}

fn some_other_long_computation() -> Arbitrary {
    Arbitrary
}

struct Input;
trait Responder {
    type Response;
    type Err;
    fn respond(&self, input: Input) -> std::result::Result<Self::Response, Self::Err>;
}

struct Message;

impl Message {
    fn text(&self) {}
}

struct Log {
    log_file_path: String,
}

impl Log {
    async fn transmit_log(&self, _: &str) -> Result<usize> {
        Ok(0)
    }
}
use std::cmp::Ordering;
#[derive(Eq, PartialEq)]
struct Node;
impl Node {
    pub fn new(parent: &Self) -> Self {
        // ...
      Node
    }
}

impl PartialOrd for Node {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
      Some(Ordering::Less)
    }
}
```

⚠️ Avoid this:

```rust
type Result<T> = std::result::Result<T, Error>;

enum Error {
    Invalid,
    NetworkUnavailable,
    MalformedEnvUrl {
        env_var: &'static str,
        source: Box<dyn std::error::Error>,
    },
    Unsupported,
    Unknown(Box<dyn std::error::Error>),
}

impl<E> From<E> for Error
where
    E: std::error::Error + 'static,
{
    fn from(err: E) -> Self {
        Error::Unknown(Box::new(err))
    }
}

struct Arbitrary;
impl Arbitrary {
    fn new() -> Self {
        Self
    }

    fn something_else(self) -> Self {
        self
    }

    fn another_thing(self) -> Self {
        self
    }

    fn chained_with_something_else(self) -> Result<Self> {
        Ok(self)
    }

    fn format_as(self, fmt: &str) -> String {
        String::new()
    }
}

fn some_long_computation() -> Result<Arbitrary> {
    Ok(Arbitrary)
}

fn some_other_long_computation() -> Arbitrary {
    Arbitrary
}

struct Input;
trait Responder {
    type Response;
    type Err;
    fn respond(&self, input: Input) -> std::result::Result<Self::Response, Self::Err>;
}

struct Message;

impl Message {
    fn text(&self) {}
}

struct Log {
    log_file_path: String,
}

impl Log {
    async fn transmit_log(&self, _: &str) -> Result<usize> {
        Ok(0)
    }
}
use std::cmp::Ordering;
#[derive(Eq, PartialEq)]
struct Node;
impl Node {
    pub fn new(parent: &Node) -> Node {
        // ...
      Node
    }
}

impl PartialOrd<Node> for Node { // NB: Rhs=Self is also the default for PartialOrd.
    fn partial_cmp(&self, other: &Node) -> Option<Ordering> {
        // ...
      Some(Ordering::Less)
    }
}
```

### When not to Use Self

Do not use `Self` when constructing associated types.

Traits often include type fields as part of the interface they describe. These types may be referred to with `Self::AssociatedType`. Do not use these to construct values as it prevents the next reader from understanding which type is in use and which fields and methods are available. Use these `Self::*` types to define interfaces, but use concrete types to define implementations.

The only exception is for trait items which return a `Result<_, Self::Err>`, where `Err` is set to the crate's `Error` type. In this case, it is okay to use the crate's `Result` type alias instead.

✅ Do this:

```rust
type Result<T> = std::result::Result<T, Error>;

enum Error {
    Invalid,
    NetworkUnavailable,
    MalformedEnvUrl {
        env_var: &'static str,
        source: Box<dyn std::error::Error>,
    },
    Unsupported,
    Unknown(Box<dyn std::error::Error>),
}

impl<E> From<E> for Error
where
    E: std::error::Error + 'static,
{
    fn from(err: E) -> Self {
        Error::Unknown(Box::new(err))
    }
}

struct Arbitrary;
impl Arbitrary {
    fn new() -> Self {
        Self
    }

    fn something_else(self) -> Self {
        self
    }

    fn another_thing(self) -> Self {
        self
    }

    fn chained_with_something_else(self) -> Result<Self> {
        Ok(self)
    }

    fn format_as(self, fmt: &str) -> String {
        String::new()
    }
}

fn some_long_computation() -> Result<Arbitrary> {
    Ok(Arbitrary)
}

fn some_other_long_computation() -> Arbitrary {
    Arbitrary
}

struct Input;
trait Responder {
    type Response;
    type Err;
    fn respond(&self, input: Input) -> std::result::Result<Self::Response, Self::Err>;
}

struct Message;

impl Message {
    fn text(&self) {}
}

struct Log {
    log_file_path: String,
}

impl Log {
    async fn transmit_log(&self, _: &str) -> Result<usize> {
        Ok(0)
    }
}
type MyType = Arbitrary;
struct SomeStruct {
    some: (),
    fields: (),
}
impl Responder for MyType {
    type Response = SomeStruct;
    type Err = Error;

    fn respond(&self, _input: Input) -> Result<Self::Response> {
        let some = ();
        let fields = ();
        Ok(SomeStruct{
            some,
            fields,
        })
    }
}
```

⚠️ Avoid this:

```rust
type Result<T> = std::result::Result<T, Error>;

enum Error {
    Invalid,
    NetworkUnavailable,
    MalformedEnvUrl {
        env_var: &'static str,
        source: Box<dyn std::error::Error>,
    },
    Unsupported,
    Unknown(Box<dyn std::error::Error>),
}

impl<E> From<E> for Error
where
    E: std::error::Error + 'static,
{
    fn from(err: E) -> Self {
        Error::Unknown(Box::new(err))
    }
}

struct Arbitrary;
impl Arbitrary {
    fn new() -> Self {
        Self
    }

    fn something_else(self) -> Self {
        self
    }

    fn another_thing(self) -> Self {
        self
    }

    fn chained_with_something_else(self) -> Result<Self> {
        Ok(self)
    }

    fn format_as(self, fmt: &str) -> String {
        String::new()
    }
}

fn some_long_computation() -> Result<Arbitrary> {
    Ok(Arbitrary)
}

fn some_other_long_computation() -> Arbitrary {
    Arbitrary
}

struct Input;
trait Responder {
    type Response;
    type Err;
    fn respond(&self, input: Input) -> std::result::Result<Self::Response, Self::Err>;
}

struct Message;

impl Message {
    fn text(&self) {}
}

struct Log {
    log_file_path: String,
}

impl Log {
    async fn transmit_log(&self, _: &str) -> Result<usize> {
        Ok(0)
    }
}
type MyType = Arbitrary;
struct SomeStruct {
    some: (),
    fields: (),
}
{
type Result<T, E> = std::result::Result<T, E>;
impl Responder for MyType {
    type Response = SomeStruct;
    type Err = Error;

    fn respond(&self, _input: Input) -> Result<SomeStruct, Error> {
        let some = ();
        let fields = ();
        Ok(Self::Response {
            some,
            fields,
        })
    }
}
}
```

### Struct Population

Structs, like tuples, provide an excellent way to group together related information of different types. Unlike tuples however, they force values to be named, thus avoiding any forgettable-ordering problems. When each field is given a value, there are three possible forms to choose from:

- The field has its value moved in from a variable with the same name,
- The field has its value moved in from a variable with a different name,
- The field takes its value from the result of some computation.

The first form is the cleanest for struct population as not only does it use the least characters, it also implies that the concepts being handled are very compatible and that there is a clear flow of data present. This makes the code easier to read, hence wherever reasonable, we should aim for this form.

The second form is acceptable if the name of the field and the name of the value are similar, for example one name being a sub-string of the other. If it is not possible to tune naming to make this true, then this is a sign of messy concepts at play and hence that a refactor is needed.

The third form can be the most problematic as large computations often draw too much attention, effectively hiding both smaller computations and the other two field-forms above. Drawing attention in this way indicates that the code awkwardly interleaves separate actions—the same section of code is both populating a struct and doing detailed computation to determine its contents. To avoid some fields sticking out like a sore thumb, either all fields of a struct should be computed or none should be. In case of a mix, refactor computations into a new `let` declarations, matching the order of the fields as closely as possible and use the first form above. This avoids the reader having to wade through alphabet soup.

The order in which fields are populated must be the same as the type declaration. Each line which populates a field in a given struct must be independent. If there is a dependency between field declarations, for example if some shared state is mutated during their construction, use `let` declarations and the first form instead.

✅ Do this:

```rust
type Result<T> = std::result::Result<T, Error>;

enum Error {
    Invalid,
    NetworkUnavailable,
    MalformedEnvUrl {
        env_var: &'static str,
        source: Box<dyn std::error::Error>,
    },
    Unsupported,
    Unknown(Box<dyn std::error::Error>),
}

impl<E> From<E> for Error
where
    E: std::error::Error + 'static,
{
    fn from(err: E) -> Self {
        Error::Unknown(Box::new(err))
    }
}

struct Arbitrary;
impl Arbitrary {
    fn new() -> Self {
        Self
    }

    fn something_else(self) -> Self {
        self
    }

    fn another_thing(self) -> Self {
        self
    }

    fn chained_with_something_else(self) -> Result<Self> {
        Ok(self)
    }

    fn format_as(self, fmt: &str) -> String {
        String::new()
    }
}

fn some_long_computation() -> Result<Arbitrary> {
    Ok(Arbitrary)
}

fn some_other_long_computation() -> Arbitrary {
    Arbitrary
}

struct Input;
trait Responder {
    type Response;
    type Err;
    fn respond(&self, input: Input) -> std::result::Result<Self::Response, Self::Err>;
}

struct Message;

impl Message {
    fn text(&self) {}
}

struct Log {
    log_file_path: String,
}

impl Log {
    async fn transmit_log(&self, _: &str) -> Result<usize> {
        Ok(0)
    }
}
struct Entry<K, V> {
    id: u64,
    key: K,
    value: V,
    pretty_date_modified: String,
}

type V = Arbitrary;
impl<K: Clone> Entry<K, V> {
fn get_entry(&self, key: &K) -> Result<Entry<K, V>> {
    let key = key.clone();
    let id = self.id_of(&key)?;
    let value = self.get(&key)?;
    let pretty_date_modified = self.date_modified(&key)?
        .format_as("yyyy-MM-dd@hh:mm:ss");
    Ok(Entry {
        id,
        key,
        value,
        pretty_date_modified,
    })
}

  fn get(&self, _: &K) -> Result<V> {
      Ok(Arbitrary)
  }

  fn id_of(&self, _: &K) -> Result<u64> {
      Ok(0)
  }

  fn date_modified(&self, _: &K) -> Result<Arbitrary> {
      Ok(Arbitrary)
  }
}
```

⚠️ Avoid this:

```rust
type Result<T> = std::result::Result<T, Error>;

enum Error {
    Invalid,
    NetworkUnavailable,
    MalformedEnvUrl {
        env_var: &'static str,
        source: Box<dyn std::error::Error>,
    },
    Unsupported,
    Unknown(Box<dyn std::error::Error>),
}

impl<E> From<E> for Error
where
    E: std::error::Error + 'static,
{
    fn from(err: E) -> Self {
        Error::Unknown(Box::new(err))
    }
}

struct Arbitrary;
impl Arbitrary {
    fn new() -> Self {
        Self
    }

    fn something_else(self) -> Self {
        self
    }

    fn another_thing(self) -> Self {
        self
    }

    fn chained_with_something_else(self) -> Result<Self> {
        Ok(self)
    }

    fn format_as(self, fmt: &str) -> String {
        String::new()
    }
}

fn some_long_computation() -> Result<Arbitrary> {
    Ok(Arbitrary)
}

fn some_other_long_computation() -> Arbitrary {
    Arbitrary
}

struct Input;
trait Responder {
    type Response;
    type Err;
    fn respond(&self, input: Input) -> std::result::Result<Self::Response, Self::Err>;
}

struct Message;

impl Message {
    fn text(&self) {}
}

struct Log {
    log_file_path: String,
}

impl Log {
    async fn transmit_log(&self, _: &str) -> Result<usize> {
        Ok(0)
    }
}
struct Entry<K, V> {
    id: u64,
    key: K,
    value: V,
    pretty_date_modified: String,
}

type V = Arbitrary;
impl<K: Clone> Entry<K, V> {
fn get_entry(&self, key: &K) -> Result<Entry<K, V>> {
    let value_stored_at_key = self.get(key)?;
    Ok(Entry {
        id: self.id_of(key)?,
        pretty_date_modified: self.date_modified(key)?
            .format_as("yyyy-MM-dd@hh:mm:ss"),
        key: key.clone(),
        value: value_stored_at_key,
    })
}

  fn get(&self, _: &K) -> Result<V> {
      Ok(Arbitrary)
  }

  fn id_of(&self, _: &K) -> Result<u64> {
      Ok(0)
  }

  fn date_modified(&self, _: &K) -> Result<Arbitrary> {
      Ok(Arbitrary)
  }
}
```

### Tuple Population

Tuples are most easily read when they are short as once line-breaks occur, the structure being created gets harder to discern. Keep things visually simple—if the formatter chooses to break tuple population into multiple lines, instead introduce new `let` declarations to move computation away from the tuple population.

✅ Do this:

```rust
type Result<T> = std::result::Result<T, Error>;

enum Error {
    Invalid,
    NetworkUnavailable,
    MalformedEnvUrl {
        env_var: &'static str,
        source: Box<dyn std::error::Error>,
    },
    Unsupported,
    Unknown(Box<dyn std::error::Error>),
}

impl<E> From<E> for Error
where
    E: std::error::Error + 'static,
{
    fn from(err: E) -> Self {
        Error::Unknown(Box::new(err))
    }
}

struct Arbitrary;
impl Arbitrary {
    fn new() -> Self {
        Self
    }

    fn something_else(self) -> Self {
        self
    }

    fn another_thing(self) -> Self {
        self
    }

    fn chained_with_something_else(self) -> Result<Self> {
        Ok(self)
    }

    fn format_as(self, fmt: &str) -> String {
        String::new()
    }
}

fn some_long_computation() -> Result<Arbitrary> {
    Ok(Arbitrary)
}

fn some_other_long_computation() -> Arbitrary {
    Arbitrary
}

struct Input;
trait Responder {
    type Response;
    type Err;
    fn respond(&self, input: Input) -> std::result::Result<Self::Response, Self::Err>;
}

struct Message;

impl Message {
    fn text(&self) {}
}

struct Log {
    log_file_path: String,
}

impl Log {
    async fn transmit_log(&self, _: &str) -> Result<usize> {
        Ok(0)
    }
}
fn snippet() -> Result<(Arbitrary, Arbitrary)> {
let key = some_long_computation()?
    .something_else()
    .another_thing();
let value = some_other_long_computation()
    .chained_with_something_else()?;
let ret =
(key, value)
;
Ok(ret)
}
```

⚠️ Avoid this:

```rust
type Result<T> = std::result::Result<T, Error>;

enum Error {
    Invalid,
    NetworkUnavailable,
    MalformedEnvUrl {
        env_var: &'static str,
        source: Box<dyn std::error::Error>,
    },
    Unsupported,
    Unknown(Box<dyn std::error::Error>),
}

impl<E> From<E> for Error
where
    E: std::error::Error + 'static,
{
    fn from(err: E) -> Self {
        Error::Unknown(Box::new(err))
    }
}

struct Arbitrary;
impl Arbitrary {
    fn new() -> Self {
        Self
    }

    fn something_else(self) -> Self {
        self
    }

    fn another_thing(self) -> Self {
        self
    }

    fn chained_with_something_else(self) -> Result<Self> {
        Ok(self)
    }

    fn format_as(self, fmt: &str) -> String {
        String::new()
    }
}

fn some_long_computation() -> Result<Arbitrary> {
    Ok(Arbitrary)
}

fn some_other_long_computation() -> Arbitrary {
    Arbitrary
}

struct Input;
trait Responder {
    type Response;
    type Err;
    fn respond(&self, input: Input) -> std::result::Result<Self::Response, Self::Err>;
}

struct Message;

impl Message {
    fn text(&self) {}
}

struct Log {
    log_file_path: String,
}

impl Log {
    async fn transmit_log(&self, _: &str) -> Result<usize> {
        Ok(0)
    }
}
fn snippet() -> Result<(Arbitrary, Arbitrary)> {
let ret =
(
    some_long_computation()?
        .something_else()
        .another_thing(),
    some_other_long_computation()
        .chained_with_something_else()?,
)
;
Ok(ret)
}
```

### Prefer Collect when Interacting with FromIterator

The `FromIterator` trait defines a method `from_iter` which is called by `Iterator::collect`. We therefore have two methods of collecting into an iterator, `Foo::from_iter` and `collect()` with appropriate type bounds. Prefer the latter as this makes the order of operations the same as what is read from top to bottom.

✅ Do this:

```rust
let collection = [0];
let filter_closure = |_: &i32| true;
let my_vec: Vec<_> = collection.into_iter()
    .filter(filter_closure)
    .collect();
```

⚠️ Avoid this:

```rust
let collection = [0];
let filter_closure = |_: &i32| true;
let my_vec = Vec::from_iter(
    collection.into_iter()
        .filter(filter_closure)
);
```

### Empty Vec Construction

To construct an empty `Vec`, we have three options: `vec![]`, `Vec::new()` and `Vec::with_capacity(_)`. If the size of the `Vec` resulting from an operation can be reasonably estimated, prefer `Vec::with_capacity` as this will avoid unnecessary reallocations. Otherwise, if a zero-sized `Vec` is required, use `Vec::new()` as this indicates most clearly that no operation is being performed as the docs for `Vec::new` guarantee no allocations, but those for `vec!` do not. Consider also that `vec![expr; n]` where `n` is zero will still evaluate (and then immediately drop) `expr`.

✅ Do this:

```rust
let my_vec = Vec::new();
let my_vec_constraint: Vec<i32> = my_vec;
```

⚠️ Avoid this:

```rust
let my_vec = vec![];
let my_vec_constraint: Vec<i32> = my_vec;
let my_vec = Vec::with_capacity(0);
let my_vec_constraint: Vec<i32> = my_vec;
```

### Avoid Loosely-scoped Let Mut

In many cases, mutability is used to create a given structure which is then used immutably for the remainder of its lifetime. Whenever this happens, scope the mutable declarations to just where they are needed, thus forcing a compiler error if this condition is broken in future. Doing this also makes code simpler to read as there are fewer things which can mutate at any one point.

```rust
type Result<T> = std::result::Result<T, Error>;

enum Error {
    Invalid,
    NetworkUnavailable,
    MalformedEnvUrl {
        env_var: &'static str,
        source: Box<dyn std::error::Error>,
    },
    Unsupported,
    Unknown(Box<dyn std::error::Error>),
}

impl<E> From<E> for Error
where
    E: std::error::Error + 'static,
{
    fn from(err: E) -> Self {
        Error::Unknown(Box::new(err))
    }
}

struct Arbitrary;
impl Arbitrary {
    fn new() -> Self {
        Self
    }

    fn something_else(self) -> Self {
        self
    }

    fn another_thing(self) -> Self {
        self
    }

    fn chained_with_something_else(self) -> Result<Self> {
        Ok(self)
    }

    fn format_as(self, fmt: &str) -> String {
        String::new()
    }
}

fn some_long_computation() -> Result<Arbitrary> {
    Ok(Arbitrary)
}

fn some_other_long_computation() -> Arbitrary {
    Arbitrary
}

struct Input;
trait Responder {
    type Response;
    type Err;
    fn respond(&self, input: Input) -> std::result::Result<Self::Response, Self::Err>;
}

struct Message;

impl Message {
    fn text(&self) {}
}

struct Log {
    log_file_path: String,
}

impl Log {
    async fn transmit_log(&self, _: &str) -> Result<usize> {
        Ok(0)
    }
}
type MyStructure = Arbitrary;
let my_structure = {
    let mut my_structure = MyStructure::new();
    // Mutate \`my_structure\` as required to construct it.
    my_structure
};
```

For greatest clarity, make sure the name of the outer (immutable) and inner (mutable) declarations have the same name, here `my_structure`.

If mutability is to retain some state whilst iterating through a structure, consider using a functional style instead. As a simple example, if presented with the following imperative code to count the number of spaces in a string

```rust
let my_string = "";
let mut num_spaces = 0;
for c in my_string.chars() {
    if c == ' ' {
        num_spaces += 1;
    }
}
```

Consider instead, using the functional style to avoid the mutation—

```rust
let my_string = "";
let num_spaces = my_string.chars()
    .filter(|c| *c == ' ')
    .count();
```

### Avoid Unassigned Let Declarations

Let-declarations without values indicate an odd flow of data through a function. Instead, prefer to return the required value from the block which computes it.

✅ Do this:

```rust
type Result<T> = std::result::Result<T, Error>;

enum Error {
    Invalid,
    NetworkUnavailable,
    MalformedEnvUrl {
        env_var: &'static str,
        source: Box<dyn std::error::Error>,
    },
    Unsupported,
    Unknown(Box<dyn std::error::Error>),
}

impl<E> From<E> for Error
where
    E: std::error::Error + 'static,
{
    fn from(err: E) -> Self {
        Error::Unknown(Box::new(err))
    }
}

struct Arbitrary;
impl Arbitrary {
    fn new() -> Self {
        Self
    }

    fn something_else(self) -> Self {
        self
    }

    fn another_thing(self) -> Self {
        self
    }

    fn chained_with_something_else(self) -> Result<Self> {
        Ok(self)
    }

    fn format_as(self, fmt: &str) -> String {
        String::new()
    }
}

fn some_long_computation() -> Result<Arbitrary> {
    Ok(Arbitrary)
}

fn some_other_long_computation() -> Arbitrary {
    Arbitrary
}

struct Input;
trait Responder {
    type Response;
    type Err;
    fn respond(&self, input: Input) -> std::result::Result<Self::Response, Self::Err>;
}

struct Message;

impl Message {
    fn text(&self) {}
}

struct Log {
    log_file_path: String,
}

impl Log {
    async fn transmit_log(&self, _: &str) -> Result<usize> {
        Ok(0)
    }
}
fn snippet() -> Result<()> {
let result = Result::Ok(());
let message = if result.is_ok() {
    "success!"
} else {
    "failed!"
};

fn exec_web_request() -> Result<Message> { Ok(Message) }
let mut retries = 0;
let message = loop {
    let resp = exec_web_request();
    match resp {
        Ok(m) => break m.text(),
        Err(Error::NetworkUnavailable) => {},
        Err(e) => return Err(e),
    }

    retries += 1;
    if retries > 5 {
        return Err(Error::NetworkUnavailable);
    }
};
Ok(())
}
```

⚠️ Avoid this:

```rust
type Result<T> = std::result::Result<T, Error>;

enum Error {
    Invalid,
    NetworkUnavailable,
    MalformedEnvUrl {
        env_var: &'static str,
        source: Box<dyn std::error::Error>,
    },
    Unsupported,
    Unknown(Box<dyn std::error::Error>),
}

impl<E> From<E> for Error
where
    E: std::error::Error + 'static,
{
    fn from(err: E) -> Self {
        Error::Unknown(Box::new(err))
    }
}

struct Arbitrary;
impl Arbitrary {
    fn new() -> Self {
        Self
    }

    fn something_else(self) -> Self {
        self
    }

    fn another_thing(self) -> Self {
        self
    }

    fn chained_with_something_else(self) -> Result<Self> {
        Ok(self)
    }

    fn format_as(self, fmt: &str) -> String {
        String::new()
    }
}

fn some_long_computation() -> Result<Arbitrary> {
    Ok(Arbitrary)
}

fn some_other_long_computation() -> Arbitrary {
    Arbitrary
}

struct Input;
trait Responder {
    type Response;
    type Err;
    fn respond(&self, input: Input) -> std::result::Result<Self::Response, Self::Err>;
}

struct Message;

impl Message {
    fn text(&self) {}
}

struct Log {
    log_file_path: String,
}

impl Log {
    async fn transmit_log(&self, _: &str) -> Result<usize> {
        Ok(0)
    }
}
fn snippet() -> Result<()> {
let result = Result::Ok(());
let message;
if result.is_ok() {
    message = "success!";
} else {
    message = "failed!";
}

fn exec_web_request() -> Result<Message> { Ok(Message) }
let mut retries = 0;
let message;
loop {
    let resp = exec_web_request();
    match resp {
        Ok(m) => {
            message = m.text();
            break;
        },
        Err(Error::NetworkUnavailable) => {},
        Err(e) => return Err(e),
    }

    retries += 1;
    if retries > 5 {
        return Err(Error::NetworkUnavailable);
    }
}
Ok(())
}
```

### Reference Scope

In many cases, the compiler is smart enough to create temporary storage locations to store variables which are given the value `&expr`, however, when these are passed to functions, it becomes slightly harder to follow which type is being used, especially when handling `&T` where `T` is `!Copy`. In this case, it is only a single character in the variable declaration, possibly many lines away which shows that the value `T` is not being moved, only its reference.

Instead of relying on temporary storage locations, store the value explicitly and take a reference where needed. This way, the transfer of ownership is much more explicit. As a rule of thumb, only use `&` at the start of the value of a `let` declaration when either indexing or slicing.

✅ Do this:

```rust
fn from_func() {}
fn other_func(_: &()) {}
let foo = from_func();
other_func(&foo);
```

⚠️ Avoid this:

```rust
fn from_func() {}
fn other_func(_: &()) {}
let foo = &from_func();
other_func(foo);
```

### Shadowing

When used in moderation, shadowing provides an excellent way to manipulate data or change its type whilst still highlighting that the 'same' data is being processed, however, too many levels of shadowing quickly makes code harder to follow.

Let us consider name-shadowing with variables, for which there are three cases:

1. Shadowing in contained scopes, such as `if let Some(x) = x { … }`
2. Shadowing in the same scope with the same type, such as `let x = …; let x = some_transform(x)`
3. Shadowing in the same scope with different types, such as `let x: T = x.into()`

In the first case when shadowing with different scopes, use at most one level of shadowing, for example when pattern matching enum variants—

```rust
let foo = Some(());
if let Some(foo) = foo {
    // The outer \`foo\` is now shadowed.
    // The inner \`foo\` should not be shadowed.
}
```

In the second case when shadowing in the same scope with the same type, there is no restriction placed on this and it may be done as many times as necessary. However, if this is being used to effectively mutate a value during construction with no other values being affected, instead use the scoped-mutability pattern—

```rust
type Result<T> = std::result::Result<T, Error>;

enum Error {
    Invalid,
    NetworkUnavailable,
    MalformedEnvUrl {
        env_var: &'static str,
        source: Box<dyn std::error::Error>,
    },
    Unsupported,
    Unknown(Box<dyn std::error::Error>),
}

impl<E> From<E> for Error
where
    E: std::error::Error + 'static,
{
    fn from(err: E) -> Self {
        Error::Unknown(Box::new(err))
    }
}

struct Arbitrary;
impl Arbitrary {
    fn new() -> Self {
        Self
    }

    fn something_else(self) -> Self {
        self
    }

    fn another_thing(self) -> Self {
        self
    }

    fn chained_with_something_else(self) -> Result<Self> {
        Ok(self)
    }

    fn format_as(self, fmt: &str) -> String {
        String::new()
    }
}

fn some_long_computation() -> Result<Arbitrary> {
    Ok(Arbitrary)
}

fn some_other_long_computation() -> Arbitrary {
    Arbitrary
}

struct Input;
trait Responder {
    type Response;
    type Err;
    fn respond(&self, input: Input) -> std::result::Result<Self::Response, Self::Err>;
}

struct Message;

impl Message {
    fn text(&self) {}
}

struct Log {
    log_file_path: String,
}

impl Log {
    async fn transmit_log(&self, _: &str) -> Result<usize> {
        Ok(0)
    }
}
type MyThing = Arbitrary;
let my_thing = {
    let mut my_thing = MyThing::new();
    // Mutate \`my_thing\` to construct it...
    my_thing
};
```

In the final case, when shadowing in the same but changing types (e.g. in a conversion method), shadowing can be done at most once per variable. This pattern is commonly seen in conversion functions—those which take ownership of their sole parameter and convert it into another type.

```rust
struct Store<S> {
    some_field: (),
    state: S,
}
struct New;
struct Inited;
impl Store<New> {
    fn init(self) -> Store<Inited> {
        let Self { some_field, .. } = self;
        let some_field = some_field.into();
        // ...
        Store {
            some_field,
            // ...
          state: Inited,
        }
    }
}
```

Now let us consider name-shadowing with types. Do not name-shadow with types. A key benefit of Rust's strong type system is that for any valid program, each handled value has exactly one possible type and representation, which makes it easier for the programmer to understand what is happening. If we shadow types, the reader may think they understand what they are reading until they find something which seems impossible, because they have been quietly fooled by a definition possibly many lines away which they did not read.

### Generic Type Parameter Constraints

Generic type parameter constraints should be grouped in the same place. If some constraints are complex enough to be placed into a `where` clause, then all constraints should be moved from angle-bracket declarations into the `where` clause also.

If constraints are declared in angle brackets and that declaration is longer than 30 characters, move all constraints into a `where` clause.

These rules ensure that the reader cannot miss any constraints.

✅ Do this:

```rust
struct IterWrapper<I>(I);
impl<'a, 'b, I, T> IterWrapper<I>
    where
        'b: 'a,
        I: Iterator<Item=T> + 'a,
        T: 'b,
{ /* ... */ }
```

⚠️ Avoid this:

```rust
struct IterWrapper<I>(I);
impl<'a, 'b: 'a, I: Iterator<Item=T> + 'a, T> IterWrapper<I>
    where
        T: 'b
{ /* ... */ }
```

### Type Annotations

The compiler's type inference is usually very good, but sometimes, it needs a little extra information. When adding the necessary annotations, we must be mindful of how much information we provide and where that information is provided.

Provide only the minimum amount of information required to help the compiler. If the compiler does not complain if a type annotation is removed, then it should be removed. Always make note of the return type of the function currently being written as often this provides enough information to make many possible type annotations obsolete.

Let's briefly consider collecting an iterator of `SomeLongType<…>` into a `Vec`. The compiler will complain that it does not know which collection type to collect into and although explicitly annotating `Vec<SomeLongType<…>>` works, the information the compiler needs is just that a `Vec` is being made, so it is better to use just `Vec<_>`. This has three benefits:

1. The code is cleaner as unnecessary information does not draw the eye
2. The code is more maintainable as the types we handle are allowed to change (the necessary properties are still implicitly enforced at function boundaries)
3. The coder's wrists ache less

In a function implementation, Rust provides three main ways to insert annotations:

1. On `let` / `static` / `const` declarations, e.g. `let foo: Type`
2. On functions using the turbofish, e.g. `func::<T>()`
3. On trait items using the fully-qualified syntax, e.g. `<value as Trait>::trait_item`

The above the order of preference in which these should be used; we shall explore each method presently.

The best place to put annotations is on `let` / `static` / `const` declarations, as this not only makes it clearer what the values are being, but also if such a declaration involves a long chain of calls spanning many lines, it also makes it more obvious what's being worked towards. This pattern is also the most flexible.

Next, if type information must be given in the middle of the expression, the turbofish may be used (`::<>`). One place to be careful though is if the turbofish is required on a `collect` in the middle of an expression, there is often a cleaner and more efficient way to achieve the same thing.

The tool of last-resort is the fully-qualified syntax which disambiguates between same-name items on the same type which come from different namespaces in scope. If an API you are making can cause an abundance of name-clashes, your consumers will be unhappy. In this case, refactor it as best you can.

If a type parameter seems necessary as a variable name is not sufficiently descriptive, improve the name.

✅ Do this:

```rust
let foo = [0];
let filter_closure = |_: &&i32| true;
let map_closure = |_| ();
let some_meaningful_var_name: Vec<_> = foo.iter()
    .filter(filter_closure)
    .map(map_closure)
    // ...
    .collect();
```

⚠️ Avoid this:

```rust
fn snippet<'lifetimes, With, Generics>() {
struct And<'l>(&'l ());
let foo = [0];
let filter_closure = |_: &&i32| true;
let map_closure = |x| SomeExtremelyLongType(std::marker::PhantomData);
struct SomeExtremelyLongType<W, G, A>(std::marker::PhantomData<(W, G, A)>);
let x = foo.iter()
    .filter(filter_closure)
    .map(map_closure)
    // ...
    .collect::<Vec<SomeExtremelyLongType<With, Generics, And<'lifetimes>>>>();
}
```

### Avoid Explicit Drop Calls

If a value must be dropped early, create a new scope with curly braces rather than using an explicit `drop` call. This will highlight the lifetimes of the values in use whilst also avoiding the reader missing the `drop` call, which may be nestled among many busy-looking lines of code.

The `drop` function should not be used to discard values in call chains. If a single value is to be ignored, use the 'toilet operator,' `|_| ()`, i.e. a closure which takes one argument, ignores it and returns a unit. This form is consistent with similar closures which ignore parts of their inputs, for example when extracting the key from a key-value pair—

```rust
2
    .map(|(key, _)| key)
;
```

There are two exceptions to this.

If `|_| …` is used and the ignored parameter is an `Error`, we should highlight that an error is intentionally being ignored, for example by using `.ok()` on a `Result` being handled.

If converting from some `Result<T>` to a `Result<()>` at the end of the last expression of a function, instead of the ignore marker, use the `?` operator and an explicit `Ok(())`. This highlights that we care only about side-effects, and that no information is returned in the successful case.

✅ Do this:

```rust
type Result<T> = std::result::Result<T, Error>;

enum Error {
    Invalid,
    NetworkUnavailable,
    MalformedEnvUrl {
        env_var: &'static str,
        source: Box<dyn std::error::Error>,
    },
    Unsupported,
    Unknown(Box<dyn std::error::Error>),
}

impl<E> From<E> for Error
where
    E: std::error::Error + 'static,
{
    fn from(err: E) -> Self {
        Error::Unknown(Box::new(err))
    }
}

struct Arbitrary;
impl Arbitrary {
    fn new() -> Self {
        Self
    }

    fn something_else(self) -> Self {
        self
    }

    fn another_thing(self) -> Self {
        self
    }

    fn chained_with_something_else(self) -> Result<Self> {
        Ok(self)
    }

    fn format_as(self, fmt: &str) -> String {
        String::new()
    }
}

fn some_long_computation() -> Result<Arbitrary> {
    Ok(Arbitrary)
}

fn some_other_long_computation() -> Arbitrary {
    Arbitrary
}

struct Input;
trait Responder {
    type Response;
    type Err;
    fn respond(&self, input: Input) -> std::result::Result<Self::Response, Self::Err>;
}

struct Message;

impl Message {
    fn text(&self) {}
}

struct Log {
    log_file_path: String,
}

impl Log {
    async fn transmit_log(&self, _: &str) -> Result<usize> {
        Ok(0)
    }
}
use std::fs::OpenOptions;
use std::io::Write;
impl Log {
async fn log(&self, message: String) -> Result<()> {
    {
        let mut file = OpenOptions::new()
            .append(true)
            .open(&self.log_file_path)?;
        file.write_all(message.as_bytes())?;
    }
    self.transmit_log(&message).await?;
    Ok(())
}
}
```

⚠️ Avoid this:

```rust
type Result<T> = std::result::Result<T, Error>;

enum Error {
    Invalid,
    NetworkUnavailable,
    MalformedEnvUrl {
        env_var: &'static str,
        source: Box<dyn std::error::Error>,
    },
    Unsupported,
    Unknown(Box<dyn std::error::Error>),
}

impl<E> From<E> for Error
where
    E: std::error::Error + 'static,
{
    fn from(err: E) -> Self {
        Error::Unknown(Box::new(err))
    }
}

struct Arbitrary;
impl Arbitrary {
    fn new() -> Self {
        Self
    }

    fn something_else(self) -> Self {
        self
    }

    fn another_thing(self) -> Self {
        self
    }

    fn chained_with_something_else(self) -> Result<Self> {
        Ok(self)
    }

    fn format_as(self, fmt: &str) -> String {
        String::new()
    }
}

fn some_long_computation() -> Result<Arbitrary> {
    Ok(Arbitrary)
}

fn some_other_long_computation() -> Arbitrary {
    Arbitrary
}

struct Input;
trait Responder {
    type Response;
    type Err;
    fn respond(&self, input: Input) -> std::result::Result<Self::Response, Self::Err>;
}

struct Message;

impl Message {
    fn text(&self) {}
}

struct Log {
    log_file_path: String,
}

impl Log {
    async fn transmit_log(&self, _: &str) -> Result<usize> {
        Ok(0)
    }
}
use std::fs::OpenOptions;
use std::io::Write;
impl Log {
async fn log(&self, message: String) -> Result<()> {
    let mut file = OpenOptions::new()
        .append(true)
        .open(&self.log_file_path)?;
    file.write_all(message.as_bytes())?;
    drop(file);
    self.transmit_log(&message)
        .await
        .map(drop)
}
}
```

### Constructors vs. Structs with All Fields Public

For structs created by a consumer and which represent an 'object' in the code's model (i.e. those which aren't just used to transfer data between components), prefer to expose constructor functions or a builder rather than exposing public fields. There are several benefits here:

- They format more nicely in call chains
- They allow parameter type conversions to occur implicitly
- They allow some fields to be computed in terms of others using implementation-specific details

Structs with all-public fields cannot benefit from any of the above and moreover, if it is later decided that any of these properties is beneficial, we face either a breaking change to fix it or extra complication to work around it.

For structs created by the consumer which just transfer data (e.g. `FooConfig` may be passed to `fn foo`), if there is a reasonable `Default` implementation, prefer to make all fields public to reduce boilerplate. To defend against the future addition of new fields causing breaking changes, consider marking the struct as `#[non_exhaustive]`. If there is no reasonable `Default` implementation, use a builder instead.

### Method-specific Data-structures

Avoid namespace pollution by putting helper types into the most local scope they may be reasonably defined in. Further, if common patterns start to arise, make sure to use standard names for each type, for example `Test` for test-specific builder-pattern test cases or `Message` / `Response` for serde-structs which represent a foreign API. Note that by using predictable names in very predictable, local-scale patterns, the reader is able to safely forget details unimportant to them. When helper types are local to a function, place the definitions at the bottom, below a marker comment such as `// test types` or `// serde types`. Such marker comments let the reader know that unless they wish to know the finer details about how a test works or how a foreign API is handled, they do not need to read any further.

This pattern is particularly useful to significantly reduce the blast-radius of incoming remote API changes by closely matching the expected form with a set of local (de)serialization types independent to those understood internally by our crate. Let's say we have a function called `get_image_info`, which makes a web-request to get information associated with given container image name (e.g. author, description, latest version). To nicely transfer data from some remote format into one we govern, say `ImageInfo`, add an explicit `return` at the end of `get_image_info` and *below* this, create a new type called `Response`, which implements `Deserialize`. Add a comment which says `// serde types.` to let the reader know that everything beyond this point only relates to modelling the remote API—thus saving them time as they will likely only care about these details if something is broken. Add as many new local types as are necessary to maintain a 1:1 relationship between Rust types and the remote's format—

```rust
async fn get_image_info(&self, name: &str) -> Result<ImageInfo> {
    let response: Response = serde_json::from_str(get_response().await?.text());
    let info = ImageInfo {
        name: name.to_owned(),
        version: response.metadata.version,
        authors: response.metadata.authors,
        latest_release: response.releases.last()
            .map(|release| ...),
    }
    return Ok(info);

    // serde types.
    #[derive(Deserialize)]
    struct Response {
        metadata: Metadata,
        releases: Vec<Release>,
    }

    #[derive(Deserialize)]
    struct Metadata {
        version: String,
        authors: Vec<String>,
    }

    #[derive(Deserialize)]
    struct Release {
        ...
    }
}
```

As a rule of thumb, `serde` annotations should not be present on the types used in the core of a crate, where the content of those types is taken entirely from a remote API or is destined to be sent to one. Remote APIs should not govern internal representation.

### Method Calls on Closing Brackets and Braces

Expressions which end in a `}` such as control structures and struct literals should not have methods called upon them as the formatter moves method calls onto the line below. This adds an unwelcome surprise as the scope of what the reader is currently looking at will appear to increase, adding to cognitive load and potential confusion. To avoid this, use a binding (`let some_var = …; some_var.foo()`).

When designing APIs, if a public struct will be filled by consumers for the purpose of calling a single method on it, consider instead reversing the dependency by using a free function which takes the struct as its first parameter in the fashion of a config struct.

Expressions which end in a `)` or `]` follow the same rule unless that expression is quite short.

✅ Do this:

```rust
type Result<T> = std::result::Result<T, Error>;

enum Error {
    Invalid,
    NetworkUnavailable,
    MalformedEnvUrl {
        env_var: &'static str,
        source: Box<dyn std::error::Error>,
    },
    Unsupported,
    Unknown(Box<dyn std::error::Error>),
}

impl<E> From<E> for Error
where
    E: std::error::Error + 'static,
{
    fn from(err: E) -> Self {
        Error::Unknown(Box::new(err))
    }
}

struct Arbitrary;
impl Arbitrary {
    fn new() -> Self {
        Self
    }

    fn something_else(self) -> Self {
        self
    }

    fn another_thing(self) -> Self {
        self
    }

    fn chained_with_something_else(self) -> Result<Self> {
        Ok(self)
    }

    fn format_as(self, fmt: &str) -> String {
        String::new()
    }
}

fn some_long_computation() -> Result<Arbitrary> {
    Ok(Arbitrary)
}

fn some_other_long_computation() -> Arbitrary {
    Arbitrary
}

struct Input;
trait Responder {
    type Response;
    type Err;
    fn respond(&self, input: Input) -> std::result::Result<Self::Response, Self::Err>;
}

struct Message;

impl Message {
    fn text(&self) {}
}

struct Log {
    log_file_path: String,
}

impl Log {
    async fn transmit_log(&self, _: &str) -> Result<usize> {
        Ok(0)
    }
}
fn snippet() -> Result<()> {
struct FooConfig {
    bar: &'static str,
    baz: &'static str,
}
fn foo(_: FooConfig) -> Result<()> {
    Ok(())
}
foo(FooConfig {
    bar: "asdf",
    baz: "fdsa",
})?;

let some_condition = true;
let value_a = "";
let value_b = "";
let value = if some_condition {
    value_a
} else {
    value_b
};
value.to_string()
;

// ...
['?'].iter()
    .filter(|c| {
        let exceptions = [ 'a', 'b', 'c', 'd' ];
        !exceptions.contains(c)
    })
;
// ...
Ok(())
}
```

⚠️ Avoid this:

```rust
type Result<T> = std::result::Result<T, Error>;

enum Error {
    Invalid,
    NetworkUnavailable,
    MalformedEnvUrl {
        env_var: &'static str,
        source: Box<dyn std::error::Error>,
    },
    Unsupported,
    Unknown(Box<dyn std::error::Error>),
}

impl<E> From<E> for Error
where
    E: std::error::Error + 'static,
{
    fn from(err: E) -> Self {
        Error::Unknown(Box::new(err))
    }
}

struct Arbitrary;
impl Arbitrary {
    fn new() -> Self {
        Self
    }

    fn something_else(self) -> Self {
        self
    }

    fn another_thing(self) -> Self {
        self
    }

    fn chained_with_something_else(self) -> Result<Self> {
        Ok(self)
    }

    fn format_as(self, fmt: &str) -> String {
        String::new()
    }
}

fn some_long_computation() -> Result<Arbitrary> {
    Ok(Arbitrary)
}

fn some_other_long_computation() -> Arbitrary {
    Arbitrary
}

struct Input;
trait Responder {
    type Response;
    type Err;
    fn respond(&self, input: Input) -> std::result::Result<Self::Response, Self::Err>;
}

struct Message;

impl Message {
    fn text(&self) {}
}

struct Log {
    log_file_path: String,
}

impl Log {
    async fn transmit_log(&self, _: &str) -> Result<usize> {
        Ok(0)
    }
}
fn snippet() -> Result<()> {
struct Foo {
    bar: &'static str,
    baz: &'static str,
}
impl Foo {
    fn do_thing(self) -> Result<()> {
        Ok(())
    }
}
Foo {
    bar: "asdf",
    baz: "fdsa",
}
.do_thing()?;

let some_condition = true;
let value_a = "";
let value_b = "";
if some_condition {
    value_a
} else {
    value_b
}
.to_string()
;

// ...
['?'].iter()
    .filter(|c| ![ 'a', 'b', 'c', 'd' ].contains(c))
;
// ...
Ok(())
}
```

### Format Arg Inlining

Arguments to `format!`\-like macros should aim to be as similar as possible to the string they are intended to produce. Whenever a single variable is used in a format argument, it should be inlined to avoid the reader needing to dart back and forth between the format string and its arguments (both of which may stretch over multiple lines).

*NB: older Rust versions may not support this syntax.*

✅ Do this:

```rust
let path = "";
let file = "";
format!("{path}/{file}")
;
```

⚠️ Avoid this:

```rust
let path = "";
let file = "";
format!("{}/{}", path, file)
;
```

---

## Error and Panic Discipline

### Error Messages

In Rust, it is a good idea to start writing a project by first making its `Error` type. This not only encourages message consistency, but also highlights the necessary information for these good error messages. This in turn, makes it clearer which information should be plumbed where, avoiding the awkward and altogether too-common situation where an error condition is identified, but much work must be done to get the right information there.

Error messages should be concise. Every second longer a user spends reading an error message and not understanding what went wrong is a second longer of their frustration. The form of the phrases used in error messages should be consistent—if, likely from previous messages, the user is expecting—

```text
cannot foo the bar
```

but instead reads—

```text
bar is not fooable
```

they will experience unnecessary discomfort. Keep it nice and clean.

A majority of the time, error messages should start with a verb and in most cases that verb should be prefixed with 'cannot.'

Capitalisation of error messages should be consistent. If it is possible that an error will not be a top-level error (e.g. it may be wrapped), then the first letter should be lowercase. Unexpected capital letters in the middle of a line of logging look dishevelled and imply that little thought has gone into the overall design of the implementation. This couldn't be further from the truth so let's make it appear as such.

Remember: the latin alphabet is optimised for many contiguous lowercase letters, hence lowercase is a good default to maintain. However, uppercase should be used for acronyms and standard names such as: TCP and NixOS.

When writing error messages, think about the background of the expected user/consumer of the project. Consider whether the person reading it will have good technical knowledge and tailor your approach as such.

If the reader of these error messages is expected to be a (Rust) developer (e.g. code in question is in a library or building block for something else), they will be familiar with how to fix simple problems and as such may not benefit from unnecessary advice. For these readers, a good concise error message will contain all the information needed to point them towards how to fix it. In this case, suggestions are at best superfluous and at worst unhelpful.

If the reader of these error messages is not expected to be a developer (e.g. these messages will appear in some GUI), think carefully about where you want to send that reader next. Whereas a developer might be okay to submit an issue, a non-developer will appreciate being explicitly pointed in the right direction.

Going up the stack, the reader's knowledge of low-level details becomes less reliable hence they must lean more heavily on the help we give them.

As an error passes further and further up the stack, more context messages may be added to it. Be aware of how errors will bubble up to avoid repeating the same information multiple times.

Note that all this advice applies to both error messages associated with error types and panic messages.

### Error Types

All reasonable types which implement `Error` fall into one of three categories:

- Those which erase the underlying types
- Those which preserve them, for example by enumeration
- Those which preserve them opaquely

Errors which use type-erasure (e.g. `Box<dyn Error>` and `anyhow::Error`) are often easier to use when writing code, however things become very problematic later on when attempting to inspect errors—with less help from the compiler comes far more places for subtle breakages to occur, both now and in future. Type-erased errors should only be used in prototypes where maintenance will never be a concern, otherwise, use concrete types. As a general rule, type erased errors *must not be used in library crates.* Type erasure is a very strong opinion and one which may not be shared by a crate's dependants and the process of converting from erased errors back to a concrete one is unreliable and unpleasant, and hence will irritate consumers.

Errors which preserve types (e.g those annotated with `#[derive(thiserror:: Error)]`) give Rust a unique advantage—not only can the golden path receive first-class support, but so too can the error path, thus allowing an even higher level of quality to be attained. In particular, the process of responding to particular errors is far more robust with enumerated errors.

Errors which preserve types but which represent unrecoverable errors should represent their error condition as a contained `String` which is assigned where the error is constructed. (Note that although `&'static str` may be applicable, `String` offers more flexibility and can be expected to have a negligible performance impact.) When constructing these errors, special care must be taken to ensure that the message is consistent with other errors in the codebase. The field used to hold the reason for the error in these cases should be named `reason`.

```rust
#[derive(Debug, thiserror::Error)]
pub enum Error {
    // ...

    #[error("invalid {credential}: {reason}")]
    InvalidCredentials{
        credential: String,
        reason: String,
    },
}
```

If one error wraps another, the inner error should be held in a field named `source`.

```rust
#[derive(Debug, thiserror::Error)]
pub enum Error {
    // ...

    #[error("cannot access {path}: {source}")]
    IO {
        path: PathBuf,
        source: io::Error,
    },
}
```

Note that exposing the error type originating from dependencies as these may accidentally expose internal details in a public API. In these cases, if using enumerated errors, consider adding an `Internal` variant which holds a type which hides the internal details as follows—

```rust
#[derive(Debug, thiserror::Error)]
pub enum Error {
    // ...

    #[error(transparent)]
    Internal(#[from] InternalError),
}

#[derive(Debug, thiserror::Error)]
#[error(transparent)]
pub struct InternalError(#[from] InternalErrorImpl);

#[derive(Debug, thiserror::Error)]
enum InternalErrorImpl {
    // ...
}
```

### Error Conversion

Errors returned from other crates should be converted to current the crate's `Error` type at the earliest reasonable opportunity. The intuition here is that within our crates, we should be talking our own error language and that calling functions and methods in other crates crosses an interface boundary, so to propagate their errors for too long creates a slow transition rather than a clean, abrupt change. Long call-chains often handle many different error types and converting early into the common crate-local error type will allow natural error propagation. By maintaining the same convention with short chains, our code becomes more predictable and hence easier to read.

✅ Do this:

```rust
type Result<T> = std::result::Result<T, Error>;

enum Error {
    Invalid,
    NetworkUnavailable,
    MalformedEnvUrl {
        env_var: &'static str,
        source: Box<dyn std::error::Error>,
    },
    Unsupported,
    Unknown(Box<dyn std::error::Error>),
}

impl<E> From<E> for Error
where
    E: std::error::Error + 'static,
{
    fn from(err: E) -> Self {
        Error::Unknown(Box::new(err))
    }
}

mod url {
    pub struct Url;

    impl Url {
        pub fn parse(_: &str) -> std::result::Result<Self, Box<dyn std::error::Error>> {
            Ok(Self)
        }
    }
}
fn snippet() -> Result<()> {
use std::env;
use url::Url;
let override_url = env::var("URL")
    .ok()
    .map(|override_url| {
        Url::parse(&override_url).map_err(|source| Error::MalformedEnvUrl {
            env_var: "URL",
            source,
        })
    })
    .transpose()?;
Ok(())
}
```

⚠️ Avoid this:

```rust
type Result<T> = std::result::Result<T, Error>;

enum Error {
    Invalid,
    NetworkUnavailable,
    MalformedEnvUrl {
        env_var: &'static str,
        source: Box<dyn std::error::Error>,
    },
    Unsupported,
    Unknown(Box<dyn std::error::Error>),
}

impl<E> From<E> for Error
where
    E: std::error::Error + 'static,
{
    fn from(err: E) -> Self {
        Error::Unknown(Box::new(err))
    }
}

mod url {
    pub struct Url;

    impl Url {
        pub fn parse(_: &str) -> std::result::Result<Self, Box<dyn std::error::Error>> {
            Ok(Self)
        }
    }
}
use std::env;
fn snippet() -> Result<()> {
let override_url = env::var("URL")
    .ok()
    .map(|override_url| url::Url::parse(&override_url))
    .transpose()
    .map_err(|source| Error::MalformedEnvUrl { // The error to be mapped comes from somewhere in the chain above!
        env_var: "URL",
        source,
    })?;
Ok(())
}
```

### Panic Calmly

Panics must only be used if a program enters an unrecoverable state. Further, that unrecoverable state must not be as a result of user input—a program which a user can easily crash is not a good program.

In Rust, panics are very aggressive, especially as if the default panic strategy is overridden (e.g. by a user who wants a smaller binary and hence sets `profile.release.panic = "abort"` in their `Cargo.toml`), we have no guarantee that the usual cleanup is performed. In such a situation, we must rely on the OS to do its best with the resources it understands but we have no guarantee that this will be sufficient. By default therefore, don't panic.

If, however, a panic is inevitable, be sure that the message signals who is at fault—if it's an internal error, start the message with `internal error:`.

The over-use of `.unwrap()` is a major red flag, as the resulting code is likely very fragile, relying on possibly-unclear preconditions in the code above it. If that code changes—as is very likely in code still under active development—production code may panic. As a rule of thumb, `.unwrap()` should only be used in code in tiny scopes, where errors can only possibly originate from the programmer—e.g. in `Regex::new("…").unwrap()`, where a panic can only occur if the raw regex constant is invalid. In general though, unwrapping should be replaced with either:

- Good use of the type system to allow the compiler to enforce preconditions
- Calls to `.expect` which document the required preconditions (these must follow the same convention as panic messages)

The first option is very much preferable. What follows is two common situations and solutions.

We can remove panics originating from `.unwrap()` calls by using the `?` operator to pass errors up the call-stack. In the case of unwrapping options in a function which returns a result, calling the `.ok_or` /`.ok_or_else` methods may be required.

We can also remove panics originating from `.unwrap()` calls by using pattern-matching. If a call to `x.unwrap()` is guarded by `if x.is_some()` / `if x.is_ok()`, instead make use of pattern matching: `if let Some(x) = x` or `if let Ok(x) = x`.

When calls to `.unwrap()` are removed, the surrounding code is not only more robust, but you may notice that it is often visually cleaner. This is not a coincidence—it is a nudge from the Rust developers to encourage fault-tolerant code and practices.

There are two exceptions to this avoidance of panicking: tests and panic-propagation.

If the `?` operator is used in tests, the origin of the error is lost, making the test harder to debug. A failed `.unwrap()` will result in a trace pointing to where the panic occurred.

If a thread panics whilst it has acquired a `Mutex` lock, we have no guarantee that the contents of the mutex represents a valid state and hence the lock gets *poisoned.* This means that any other thread which attempts to lock the mutex will get an error. In this case, panicking is acceptable because it effectively propagates an existing panic from another thread.

---

## Function Discipline

### No-information Returns

Any expression or statement which returns the unit (i.e. `()` such as `println!`) or which never returns anything (i.e. `!` such as `std::process::exit`) should end with a semicolon.

In the case of `()`, any block which ends with a function call which returns `()` relies on the return type of that function to never be changed to return something more useful. This is a strange dependency which may cause needless compiler errors in future, hence is best avoided. Using an explicit `;` reinforces the fact that we expect to obtain no information from a particular call.

A similar arguments holds for `!` as we expect to return no information from such a call.

If a function which returns `Result<()>` ends in a function call which also returns `Result<()>`, instead use the `?` operator and an explicit `Ok(())` return. The intuition here is that in the expected case on the golden path, we expect no information to be returned, hence we should make our code reinforce this fact.

If a `match` statement is expected to return `()`, then it is being used as a control-flow structure. Therefore, do-nothing `match` branches should be written as `.. => {}` rather than `.. => ()`.

✅ Do this:

```rust
type Result<T> = std::result::Result<T, Error>;

enum Error {
    Invalid,
    NetworkUnavailable,
    MalformedEnvUrl {
        env_var: &'static str,
        source: Box<dyn std::error::Error>,
    },
    Unsupported,
    Unknown(Box<dyn std::error::Error>),
}

impl<E> From<E> for Error
where
    E: std::error::Error + 'static,
{
    fn from(err: E) -> Self {
        Error::Unknown(Box::new(err))
    }
}

struct Arbitrary;

impl Arbitrary {
    fn new() -> Self {
        Self
    }

    fn builder() -> Self {
        Self
    }

    fn foo(self, _: &str) -> Self {
        self
    }

    fn bar(self, _: &str) -> Self {
        self
    }

    fn build(self) -> Result<Self> {
        Ok(self)
    }
}

mod some_crate {
    pub struct Foo;

    impl Foo {
        pub fn builder() -> FooBuilder {
            FooBuilder
        }
    }

    pub struct FooBuilder;

    impl FooBuilder {
        pub fn new() -> Self {
            Self
        }

        pub fn foo(self, _: &'static str) -> Self {
            self
        }

        pub fn bar(self, _: &'static str) -> Self {
            self
        }

        pub fn build(self) -> std::result::Result<Foo, Box<dyn std::error::Error>> {
            Ok(Foo)
        }
    }
}
struct Foo {
    foo_type: FooType
}
enum FooType {
    A,
}
impl Foo {
fn setup_foo(&self) -> Result<()> {
    match self.foo_type {
        FooType::A => {}
        // ...
    }
    self.setup_bar()?;
    Ok(())
}

  fn setup_bar(&self) -> Result<()> {
      Ok(())
  }
}
```

⚠️ Avoid this:

```rust
type Result<T> = std::result::Result<T, Error>;

enum Error {
    Invalid,
    NetworkUnavailable,
    MalformedEnvUrl {
        env_var: &'static str,
        source: Box<dyn std::error::Error>,
    },
    Unsupported,
    Unknown(Box<dyn std::error::Error>),
}

impl<E> From<E> for Error
where
    E: std::error::Error + 'static,
{
    fn from(err: E) -> Self {
        Error::Unknown(Box::new(err))
    }
}

struct Arbitrary;

impl Arbitrary {
    fn new() -> Self {
        Self
    }

    fn builder() -> Self {
        Self
    }

    fn foo(self, _: &str) -> Self {
        self
    }

    fn bar(self, _: &str) -> Self {
        self
    }

    fn build(self) -> Result<Self> {
        Ok(self)
    }
}

mod some_crate {
    pub struct Foo;

    impl Foo {
        pub fn builder() -> FooBuilder {
            FooBuilder
        }
    }

    pub struct FooBuilder;

    impl FooBuilder {
        pub fn new() -> Self {
            Self
        }

        pub fn foo(self, _: &'static str) -> Self {
            self
        }

        pub fn bar(self, _: &'static str) -> Self {
            self
        }

        pub fn build(self) -> std::result::Result<Foo, Box<dyn std::error::Error>> {
            Ok(Foo)
        }
    }
}
struct Foo {
    foo_type: FooType
}
enum FooType {
    A,
}
impl Foo {
fn setup_foo(&self) -> Result<()> {
    match self.foo_type {
        FooType::A => ()
        // ...
    }
    self.setup_bar()
}

  fn setup_bar(&self) -> Result<()> {
      Ok(())
  }
}
```

### Hide Generic Type Parameters

Generic type parameters add complication to an API. Where possible, hide generic parameters either through elision, syntactic sugar (`impl Trait`) or by leaving them unbound.

On `impl` blocks, only introduce strictly-necessary type-constraints. Not only will this reduce the cognitive overhead of understanding large blocks, it will also help make code more easily applicable in new scenarios.

On functions, if a generic *lifetime* parameter can be [elided](https://doc.rust-lang.org/nomicon/lifetime-elision.html), it should be by using `'_`. If a generic *type* parameter is only used once and isn't too complicated, use `impl Trait` to hide it.

Note that although it is possible to omit the unnamed lifetime (i.e. it may be possible to write `MyRef<'_>` as `MyRef`), this should never be done. A type without lifetime parameters looks completely self-contained and hence as though may be freely passed around. If a lifetime is present, *always* communicate that fact (i.e. always prefer `MyRef<'_>` to `MyRef`).

✅ Do this:

```rust
type Result<T> = std::result::Result<T, Error>;

enum Error {
    Invalid,
    NetworkUnavailable,
    MalformedEnvUrl {
        env_var: &'static str,
        source: Box<dyn std::error::Error>,
    },
    Unsupported,
    Unknown(Box<dyn std::error::Error>),
}

impl<E> From<E> for Error
where
    E: std::error::Error + 'static,
{
    fn from(err: E) -> Self {
        Error::Unknown(Box::new(err))
    }
}

struct Arbitrary;

impl Arbitrary {
    fn new() -> Self {
        Self
    }

    fn builder() -> Self {
        Self
    }

    fn foo(self, _: &str) -> Self {
        self
    }

    fn bar(self, _: &str) -> Self {
        self
    }

    fn build(self) -> Result<Self> {
        Ok(self)
    }
}

mod some_crate {
    pub struct Foo;

    impl Foo {
        pub fn builder() -> FooBuilder {
            FooBuilder
        }
    }

    pub struct FooBuilder;

    impl FooBuilder {
        pub fn new() -> Self {
            Self
        }

        pub fn foo(self, _: &'static str) -> Self {
            self
        }

        pub fn bar(self, _: &'static str) -> Self {
            self
        }

        pub fn build(self) -> std::result::Result<Foo, Box<dyn std::error::Error>> {
            Ok(Foo)
        }
    }
}
trait Transmitter {}
struct Message<'a>(&'a ());
fn transmit(tx: impl Transmitter, message: &Message<'_>) -> Result<()> {
    // ...
  Ok(())
}
```

⚠️ Avoid this:

```rust
type Result<T> = std::result::Result<T, Error>;

enum Error {
    Invalid,
    NetworkUnavailable,
    MalformedEnvUrl {
        env_var: &'static str,
        source: Box<dyn std::error::Error>,
    },
    Unsupported,
    Unknown(Box<dyn std::error::Error>),
}

impl<E> From<E> for Error
where
    E: std::error::Error + 'static,
{
    fn from(err: E) -> Self {
        Error::Unknown(Box::new(err))
    }
}

struct Arbitrary;

impl Arbitrary {
    fn new() -> Self {
        Self
    }

    fn builder() -> Self {
        Self
    }

    fn foo(self, _: &str) -> Self {
        self
    }

    fn bar(self, _: &str) -> Self {
        self
    }

    fn build(self) -> Result<Self> {
        Ok(self)
    }
}

mod some_crate {
    pub struct Foo;

    impl Foo {
        pub fn builder() -> FooBuilder {
            FooBuilder
        }
    }

    pub struct FooBuilder;

    impl FooBuilder {
        pub fn new() -> Self {
            Self
        }

        pub fn foo(self, _: &'static str) -> Self {
            self
        }

        pub fn bar(self, _: &'static str) -> Self {
            self
        }

        pub fn build(self) -> std::result::Result<Foo, Box<dyn std::error::Error>> {
            Ok(Foo)
        }
    }
}
trait Transmitter {}
struct Message<'a>(&'a ());
fn transmit<'a, T: Transmitter>(tx: T, message: &Message<'a>) -> Result<()> {
    // ...
  Ok(())
}
```

### Unused Parameters in Default Implementations

Occasionally, when a default trait function implementation is provided, not all of its parameters are used. To avoid a compiler warning, explicitly add `let _ = unused_param;` lines until all these warnings are removed.

Briefly considering other approaches, it is possible to rename the parameter to something like `_unused_param`, however this will appear in docs and look dishevelled. It is possible to use `#[allow(unused_variables)]` to suppress all unused-parameter warnings for the given function, however this will also affect the parameters we *do* expect to use, possibly causing issues later. Similarly, we could individually annotate each unused parameter, however this would make the overall signature much more difficult to read.

✅ Do this:

```rust
type Result<T> = std::result::Result<T, Error>;

enum Error {
    Invalid,
    NetworkUnavailable,
    MalformedEnvUrl {
        env_var: &'static str,
        source: Box<dyn std::error::Error>,
    },
    Unsupported,
    Unknown(Box<dyn std::error::Error>),
}

impl<E> From<E> for Error
where
    E: std::error::Error + 'static,
{
    fn from(err: E) -> Self {
        Error::Unknown(Box::new(err))
    }
}

struct Arbitrary;

impl Arbitrary {
    fn new() -> Self {
        Self
    }

    fn builder() -> Self {
        Self
    }

    fn foo(self, _: &str) -> Self {
        self
    }

    fn bar(self, _: &str) -> Self {
        self
    }

    fn build(self) -> Result<Self> {
        Ok(self)
    }
}

mod some_crate {
    pub struct Foo;

    impl Foo {
        pub fn builder() -> FooBuilder {
            FooBuilder
        }
    }

    pub struct FooBuilder;

    impl FooBuilder {
        pub fn new() -> Self {
            Self
        }

        pub fn foo(self, _: &'static str) -> Self {
            self
        }

        pub fn bar(self, _: &'static str) -> Self {
            self
        }

        pub fn build(self) -> std::result::Result<Foo, Box<dyn std::error::Error>> {
            Ok(Foo)
        }
    }
}
struct Value<'v>(&'v ());
trait CustomScriptValue<'v> {
    fn at(&self, index: Value<'v>) -> Result<Value<'v>> {
        let _ = index;
        Err(Error::Unsupported)
    }
}
```

⚠️ Avoid this:

```rust
type Result<T> = std::result::Result<T, Error>;

enum Error {
    Invalid,
    NetworkUnavailable,
    MalformedEnvUrl {
        env_var: &'static str,
        source: Box<dyn std::error::Error>,
    },
    Unsupported,
    Unknown(Box<dyn std::error::Error>),
}

impl<E> From<E> for Error
where
    E: std::error::Error + 'static,
{
    fn from(err: E) -> Self {
        Error::Unknown(Box::new(err))
    }
}

struct Arbitrary;

impl Arbitrary {
    fn new() -> Self {
        Self
    }

    fn builder() -> Self {
        Self
    }

    fn foo(self, _: &str) -> Self {
        self
    }

    fn bar(self, _: &str) -> Self {
        self
    }

    fn build(self) -> Result<Self> {
        Ok(self)
    }
}

mod some_crate {
    pub struct Foo;

    impl Foo {
        pub fn builder() -> FooBuilder {
            FooBuilder
        }
    }

    pub struct FooBuilder;

    impl FooBuilder {
        pub fn new() -> Self {
            Self
        }

        pub fn foo(self, _: &'static str) -> Self {
            self
        }

        pub fn bar(self, _: &'static str) -> Self {
            self
        }

        pub fn build(self) -> std::result::Result<Foo, Box<dyn std::error::Error>> {
            Ok(Foo)
        }
    }
}
struct Value<'v>(&'v ());
trait CustomScriptValue<'v> {
    fn at(&self, _index: Value<'v>) -> Result<Value<'v>> {
        Err(Error::Unsupported)
    }
}

    // OR

trait CustomScriptValue2<'v> {
    #[allow(unused_variables)]
    fn at(&self, index: Value<'v>) -> Result<Value<'v>> {
        Err(Error::Unsupported)
    }
}
```

### Builder Visibility

The type `MyTypeBuilder` should not have a public constructor. In typical usage, users of `MyType` shouldn't need to import `MyTypeBuilder`, it should be a seamless part of `MyType`.

✅ Do this:

```rust
type Result<T> = std::result::Result<T, Error>;

enum Error {
    Invalid,
    NetworkUnavailable,
    MalformedEnvUrl {
        env_var: &'static str,
        source: Box<dyn std::error::Error>,
    },
    Unsupported,
    Unknown(Box<dyn std::error::Error>),
}

impl<E> From<E> for Error
where
    E: std::error::Error + 'static,
{
    fn from(err: E) -> Self {
        Error::Unknown(Box::new(err))
    }
}

struct Arbitrary;

impl Arbitrary {
    fn new() -> Self {
        Self
    }

    fn builder() -> Self {
        Self
    }

    fn foo(self, _: &str) -> Self {
        self
    }

    fn bar(self, _: &str) -> Self {
        self
    }

    fn build(self) -> Result<Self> {
        Ok(self)
    }
}

mod some_crate {
    pub struct Foo;

    impl Foo {
        pub fn builder() -> FooBuilder {
            FooBuilder
        }
    }

    pub struct FooBuilder;

    impl FooBuilder {
        pub fn new() -> Self {
            Self
        }

        pub fn foo(self, _: &'static str) -> Self {
            self
        }

        pub fn bar(self, _: &'static str) -> Self {
            self
        }

        pub fn build(self) -> std::result::Result<Foo, Box<dyn std::error::Error>> {
            Ok(Foo)
        }
    }
}
fn snippet() -> std::result::Result<(), Box<dyn std::error::Error>> {
use some_crate::Foo;

let foo = Foo::builder()
    .foo("foo")
    .bar("bar")
    .build()?;
Ok(())
}
```

⚠️ Avoid this:

```rust
type Result<T> = std::result::Result<T, Error>;

enum Error {
    Invalid,
    NetworkUnavailable,
    MalformedEnvUrl {
        env_var: &'static str,
        source: Box<dyn std::error::Error>,
    },
    Unsupported,
    Unknown(Box<dyn std::error::Error>),
}

impl<E> From<E> for Error
where
    E: std::error::Error + 'static,
{
    fn from(err: E) -> Self {
        Error::Unknown(Box::new(err))
    }
}

struct Arbitrary;

impl Arbitrary {
    fn new() -> Self {
        Self
    }

    fn builder() -> Self {
        Self
    }

    fn foo(self, _: &str) -> Self {
        self
    }

    fn bar(self, _: &str) -> Self {
        self
    }

    fn build(self) -> Result<Self> {
        Ok(self)
    }
}

mod some_crate {
    pub struct Foo;

    impl Foo {
        pub fn builder() -> FooBuilder {
            FooBuilder
        }
    }

    pub struct FooBuilder;

    impl FooBuilder {
        pub fn new() -> Self {
            Self
        }

        pub fn foo(self, _: &'static str) -> Self {
            self
        }

        pub fn bar(self, _: &'static str) -> Self {
            self
        }

        pub fn build(self) -> std::result::Result<Foo, Box<dyn std::error::Error>> {
            Ok(Foo)
        }
    }
}
fn snippet() -> std::result::Result<(), Box<dyn std::error::Error>> {
use some_crate::{Foo, FooBuilder};

let foo = FooBuilder::new()
    .foo("foo")
    .bar("bar")
    .build()?;
Ok(())
}
```

### Builder Ownership

In Rust, there are two forms of the builder pattern, depending on the receiver type used. Consider the following builder—

```rust
type Result<T> = std::result::Result<T, Error>;

enum Error {
    Invalid,
    NetworkUnavailable,
    MalformedEnvUrl {
        env_var: &'static str,
        source: Box<dyn std::error::Error>,
    },
    Unsupported,
    Unknown(Box<dyn std::error::Error>),
}

impl<E> From<E> for Error
where
    E: std::error::Error + 'static,
{
    fn from(err: E) -> Self {
        Error::Unknown(Box::new(err))
    }
}

struct Arbitrary;

impl Arbitrary {
    fn new() -> Self {
        Self
    }

    fn builder() -> Self {
        Self
    }

    fn foo(self, _: &str) -> Self {
        self
    }

    fn bar(self, _: &str) -> Self {
        self
    }

    fn build(self) -> Result<Self> {
        Ok(self)
    }
}

mod some_crate {
    pub struct Foo;

    impl Foo {
        pub fn builder() -> FooBuilder {
            FooBuilder
        }
    }

    pub struct FooBuilder;

    impl FooBuilder {
        pub fn new() -> Self {
            Self
        }

        pub fn foo(self, _: &'static str) -> Self {
            self
        }

        pub fn bar(self, _: &'static str) -> Self {
            self
        }

        pub fn build(self) -> std::result::Result<Foo, Box<dyn std::error::Error>> {
            Ok(Foo)
        }
    }
}
fn snippet() -> Result<()> {
type Foo = Arbitrary;
let frobnicator = Foo::builder()
    .foo("foo")
    .bar("bar")
    .build()?;
Ok(())
}
```

The methods `foo`, `bar` and `build` can either take ownership of `self` or take `&mut self` by value.

If `self` is used, it encourages a simple flow of data, with values being neatly moved from one place to another. This should be the default ownership model.

If `&mut self` is used, it makes conditional use of each of the builder methods simpler, however either the builder must be bound to a variable or all contained data must be cloned during `.build()`. Although the optimiser may remove some unnecessary cloning, this should not be relied upon. Besides, if the `.build` method takes a reference to the builder—allowing it to be called multiple times—this feels more like a *factory* than a *builder.*

---

## Ordering Discipline

### General Definition Ordering

When read from top to bottom, a file should feel like a tour of the APIs it defines. The most important items should be defined further up the file, with their helpers below. No `impl` block should come before the type or trait to which it relates. In this way, lower-level implementation details are hidden from the reader until they wish to know more, at which point, having gained a good knowledge of the overall form of the code, they can read on to understand how it functions.

✅ Do this:

```rust
type Result<T> = std::result::Result<T, Error>;

enum Error {
    Invalid,
    NetworkUnavailable,
    MalformedEnvUrl {
        env_var: &'static str,
        source: Box<dyn std::error::Error>,
    },
    Unsupported,
    Unknown(Box<dyn std::error::Error>),
}

impl<E> From<E> for Error
where
    E: std::error::Error + 'static,
{
    fn from(err: E) -> Self {
        Error::Unknown(Box::new(err))
    }
}

struct Arbitrary;
type Foo = Arbitrary;
impl Foo {
    pub fn some_func(&self) {
        self.some_helper_func();
    }

    fn some_helper_func(&self) {
        // ...
    }
}
```

⚠️ Avoid this:

```rust
type Result<T> = std::result::Result<T, Error>;

enum Error {
    Invalid,
    NetworkUnavailable,
    MalformedEnvUrl {
        env_var: &'static str,
        source: Box<dyn std::error::Error>,
    },
    Unsupported,
    Unknown(Box<dyn std::error::Error>),
}

impl<E> From<E> for Error
where
    E: std::error::Error + 'static,
{
    fn from(err: E) -> Self {
        Error::Unknown(Box::new(err))
    }
}

struct Arbitrary;
type Foo = Arbitrary;
impl Foo {
    fn some_helper_func(&self) {
        // ...
    }

    pub fn some_func(&self) {
        self.some_helper_func();
    }
}
```

### Impl Block Placement

By default, `impl SomeType` blocks for a given type should be in the same file, immediately below where that type is defined. Trait implementations `impl SomeTrait for SomeType` should go in the file where `SomeTrait` or `SomeType` is defined. This is effectively the orphan rule but applied within crates.

### Impl Block Ordering

The `impl` blocks in the same file for `MyType` should be ordered as follows:

- `impl MyType`
- `unsafe impl StandardTrait for MyType`
- `unsafe impl MyTrait for MyType`
- `unsafe impl ThirdPartyTrait for MyType`
- `impl StandardTrait for MyType`
- `impl MyTrait for MyType`
- `impl ThirdPartyTrait for MyType`

The `impl` blocks in the same file for `MyTrait` should be ordered as follows:

- `impl MyTrait for StandardType`
- `impl MyTrait for MyType`
- `impl MyTrait for ThirdPartyType`

### Derive Ordering

Put all derive items in a single `#[derive(…)]` (the formatter will preserve readability by introducing line-breaks as it deems fit). Derived items should be ordered as follows:

- `Copy` should come first (this has the most surprising effect on how the consumer will interact with values of the given type)
- Standard library items, ordered lexicographically (these form a common expectation for basic behaviour and missing items are surprising)
- Third party items, ordered lexicographically

✅ Do this:

```rust
use external_crate::{Bar, Foo};

#[derive(Copy, Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd, Bar, Foo)]
struct MyType;
```

⚠️ Avoid this:

```rust
use external_crate::{Bar, Foo};

#[derive(Foo, Clone, Copy, PartialEq, Eq, Hash, Debug, Ord, PartialOrd, Bar)]
struct MyType;
```

### Declaration Ordering

Rust provides several different types of declaration and where these are declared consecutively in the same block, they should be ordered for visual consistency. This will help draw the reader's eye to the important parts of each declaration, rather than getting lost in some superfluous ordering.

Declarations should be ordered as follows:

- `const`
- `static`
- `let`
- `let mut`

### Struct Field Ordering

The more public a field, the more likely a user will to want to know more about it and understand it. Therefore, we should put the items they are most likely to care about nearer the top of our code, avoiding them having to skip over parts uninteresting to them. Specifically, this means that we should place:

- `pub` fields first,
- `pub(crate)` fields next,
- private fields last.

Structs neatly organised in this way make it clear when the reader has entered implementation details and hence when they are less likely to glean useful information.

If the reason for ordering fields in a different way to the above is due to a derivation such as `Ord` or `PartialOrd`, this is not a good reason for deviation from the norm. Maintaining consistency is of a higher priority than a single derivation, hence the relevant implementations should be written out by hand. The reader is more likely to be looking at the entire struct rather than just one trait implementation.

✅ Do this:

```rust
use std::collections::BTreeMap;
type StackFrame = ();
type Value<'h> = &'h ();
struct ScriptExecutionContext<'h, T> {
    pub user_data: T,

    pub(crate) global_vars: BTreeMap<String, Value<'h>>,

    stack: Vec<StackFrame>,
    max_steps: Option<u64>,
    steps: u64,
}
```

⚠️ Avoid this:

```rust
use std::collections::BTreeMap;
type StackFrame = ();
type Value<'h> = &'h ();
struct ScriptExecutionContext<'h, T> {
    stack: Vec<StackFrame>,
    pub(crate) global_vars: BTreeMap<String, Value<'h>>,
    pub user_data: T,
    steps: u64,
    max_steps: Option<u64>,
}
```

---

## Unsafe Discipline

### Minimise Unsafe

Rust's `unsafe` keyword enables a small but important number of additional powers which the compiler is unable to check. In effect, it is a 'hold my beer' marker—you tell the compiler to trust you and just watch whilst you do something either incredibly impressive or incredibly harmful. But the compiler is not the only one whose trust we require—when we use `unsafe`, we also ask our *users* to trust that we know exactly what we are doing. Under no circumstance do we want to break that trust.

Therefore, we must endeavour to minimise the use of unsafe constructs in our code. If there is a place where `unsafe` is used but not required for a strict functional requirement, drop it and replace it with a safe equivalent. If left, such a section is inherently much harder to maintain as it require not only excellent documentation but also diligence, careful consideration of preconditions and actions, and above all time. The more unsafe code a project contains, the slower it will be able to move forward in the long-run.

If `unsafe` is mandatory, minimise the scope of the `unsafe` blocks and functions in use. Even if this means adding an extra line or two, maximise the amount of help the compiler can give us whilst minimising the number of unsafe interactions which will require audit. A function of 50 lines wrapped in a single large `unsafe` block is far harder to maintain than a 56 line function containing three one-line `unsafe` blocks.

Note that 'because it's faster' is not a good reason alone to use `unsafe` constructs. Even if true, unless profiling can categorically show that safety checks add a *globally-significant* cost safety should always take a higher priority. Rust is an extremely fast language anyway.

### Document Preconditions

Both `unsafe` functions and `unsafe` blocks must document their preconditions in a comment with the prefix `// SAFETY: `. If something goes wrong in future, this will help the future maintainer understand which conditions have been violated. These comments must be carefully maintained and updated as changes are made.

---

## Structural Discipline

### How to Structure mod.rs

Files named `mod.rs` must only be used to specify the structure of a project, if definitions are added, they quickly become messy, ultimately detracting from their core purpose of declaring sub-modules and the current module's interface.

The `mod.rs` file must be separated into distinct blocks in the following order, keeping the most public items first:

1. `pub mod _` declarations
2. `pub(crate) mod _` declarations
3. `mod _` declarations
4. `pub use _` declarations
5. `pub(crate) use _` declarations
6. `pub use _` declarations

Any items gated behind a `#[cfg(…)]` must be placed at the end of the file, in the same order as the above. Like-gated items should be wrapped in a block, i.e. `#[cfg(…)] { /* items here */ }`.

No other items should be present.

Note that these guidelines also hold for `lib.rs`, with the one exception that a crate's `Error` and `Result` types are permitted in `lib.rs`, given their central importance.

### Use mod.rs to Declare a Module-root

When declaring a module `foo` which comprises multiple files, Rust allows two directory structures as follows.

Firstly, all files can be grouped into a single directory containing a `mod.rs` —

```text
foo/
├── mod.rs
├── file_in_foo.rs
└── another_file_in_foo.rs
```

Secondly, all files can be grouped into a single directory, except for the module root which is stored next to the directory and shares its name—

```text
foo.rs
foo/
├── file_in_foo.rs
└── another_file_in_foo.rs
```

Always prefer the first way as `mod.rs` files are expected to have a specific structure, hence the reader knows what they'll see when opening one of these, unlike some arbitrary `foo.rs`. Some editors group files and folders separately, so in the above example, the fact that `foo.rs` is related to the `foo/` directory will not be obvious when there are many other entries in between.

### Define Error and Result in a Standard Location

A crate's `Error` and `Result` types are likely the most important types it contains and should therefore be declared in a standard, outward-facing place.

In library crates, the `Error` type should be defined in the crate-root, `lib.rs` immediately below the `mod` and `use` declarations. The custom `Result` type alias, `type Result<T> = std::result::Result<T, Error>` must be immediately below `Error`.

In binary crates, the `Error` type should be defined `error.rs` and the `Result` alias in `result.rs`, both in the crate's root directory.

---

## Comment Discipline

### First Doc-sentence Form

When wrapped, the first sentence of a doc comment should be at most two lines. It should clearly and concisely explain the whole of the golden path of a function. After reading this first sentence, it should be clear *when* to use the given function/type—don't fall into the trap of just explaining *what* the given item does.

✅ Do this:

```rust
type Result<T> = std::result::Result<T, Error>;

enum Error {
    Invalid,
    NetworkUnavailable,
    MalformedEnvUrl {
        env_var: &'static str,
        source: Box<dyn std::error::Error>,
    },
    Unsupported,
    Unknown(Box<dyn std::error::Error>),
}

impl<E> From<E> for Error
where
    E: std::error::Error + 'static,
{
    fn from(err: E) -> Self {
        Error::Unknown(Box::new(err))
    }
}

struct Arbitrary;
impl Arbitrary {
    /// This function reports an increase in the number of steps taken by this
    /// thread.
    fn add_steps(&self, delta: i64) -> Result<()> {
        /* ... */
        Ok(())
    }
}
```

⚠️ Avoid this:

```rust
type Result<T> = std::result::Result<T, Error>;

enum Error {
    Invalid,
    NetworkUnavailable,
    MalformedEnvUrl {
        env_var: &'static str,
        source: Box<dyn std::error::Error>,
    },
    Unsupported,
    Unknown(Box<dyn std::error::Error>),
}

impl<E> From<E> for Error
where
    E: std::error::Error + 'static,
{
    fn from(err: E) -> Self {
        Error::Unknown(Box::new(err))
    }
}

struct Arbitrary;
impl Arbitrary {
    /// This function adds a given delta to the current step counter.
    fn add_steps(&self, delta: i64) -> Result<()> {
        /* ... */
        Ok(())
    }
}
```

### Definite vs. Indefinite Articles

When referring to parameters, be concrete and specific. Where possible, refer parameters by their name and if an article must be used (i.e. 'a'/'an' and 'the'), always prefer the definite article, 'the.' Leave no room for ambiguity and hence misunderstanding.

✅ Do this:

```rust
type Result<T> = std::result::Result<T, Error>;

enum Error {
    Invalid,
    NetworkUnavailable,
    MalformedEnvUrl {
        env_var: &'static str,
        source: Box<dyn std::error::Error>,
    },
    Unsupported,
    Unknown(Box<dyn std::error::Error>),
}

impl<E> From<E> for Error
where
    E: std::error::Error + 'static,
{
    fn from(err: E) -> Self {
        Error::Unknown(Box::new(err))
    }
}

struct Arbitrary;
impl Arbitrary {
    /// Increments this counter by the given \`delta\`.
    fn incr_by(&self, delta: u64) -> Result<()> {
        /* ... */
        Ok(())
    }
}
```

⚠️ Avoid this:

```rust
type Result<T> = std::result::Result<T, Error>;

enum Error {
    Invalid,
    NetworkUnavailable,
    MalformedEnvUrl {
        env_var: &'static str,
        source: Box<dyn std::error::Error>,
    },
    Unsupported,
    Unknown(Box<dyn std::error::Error>),
}

impl<E> From<E> for Error
where
    E: std::error::Error + 'static,
{
    fn from(err: E) -> Self {
        Error::Unknown(Box::new(err))
    }
}

struct Arbitrary;
impl Arbitrary {
    /// Increments a counter by a given amount.
    fn incr_by(&self, delta: u64) -> Result<()> {
        /* ... */
        Ok(())
    }
}
```
