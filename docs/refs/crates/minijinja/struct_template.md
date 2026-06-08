# minijinja: Struct `Template`

[Source](https://docs.rs/minijinja/latest/minijinja/struct.Template.html)

```rust
pub struct Template<'env: 'source, 'source> { /* private fields */ }
```
Expand description

Templates are stored in the [`Environment`](https://docs.rs/minijinja/latest/minijinja/struct.Environment.html "struct minijinja::Environment") as bytecode instructions. With the [`Environment::get_template`](https://docs.rs/minijinja/latest/minijinja/struct.Environment.html#method.get_template "method minijinja::Environment::get_template") method that is looked up and returned in form of this handle. Such a template can be cheaply copied as it only holds references.

To render the [`render`](https://docs.rs/minijinja/latest/minijinja/struct.Template.html#method.render "method minijinja::Template::render") method can be used.

## Implementations

[Source](https://docs.rs/minijinja/latest/src/minijinja/template.rs.html#145-485)

### impl<'env, 'source> Template<'env, 'source>

Returns the name of the template.

Returns the source code of the template.

Renders the template into a string.

The provided value is used as the initial context for the template. It can be any object that implements [`Serialize`](https://docs.rs/serde/1.0.219/x86_64-unknown-linux-gnu/serde/ser/trait.Serialize.html "trait serde::ser::Serialize"). You can either create your own struct and derive `Serialize` for it or the [`context!`](https://docs.rs/minijinja/latest/minijinja/macro.context.html "macro minijinja::context") macro can be used to create an ad-hoc context.

For very large contexts and to avoid the overhead of serialization of potentially unused values, you might consider using a dynamic [`Object`](https://docs.rs/minijinja/latest/minijinja/value/trait.Object.html "trait minijinja::value::Object") as value. For more information see [Map as Context](https://docs.rs/minijinja/latest/minijinja/value/trait.Object.html#map-as-context "trait minijinja::value::Object").

```rust
let tmpl = env.get_template("hello").unwrap();
println!("{}", tmpl.render(context!(name => "John")).unwrap());
```

To render a single block use [`render_captured`](https://docs.rs/minijinja/latest/minijinja/struct.Template.html#method.render_captured "method minijinja::Template::render_captured") in combination with [`State::render_block`](https://docs.rs/minijinja/latest/minijinja/struct.State.html#method.render_block "method minijinja::State::render_block").

**Note on values:** The [`Value`](https://docs.rs/minijinja/latest/minijinja/value/struct.Value.html "struct minijinja::value::Value") type implements `Serialize` and can be efficiently passed to render. It does not undergo actual serialization.

👎Deprecated since 2.18.0:

use render\_captured instead

Like [`render`](https://docs.rs/minijinja/latest/minijinja/struct.Template.html#method.render "method minijinja::Template::render") but also return the evaluated [`State`](https://docs.rs/minijinja/latest/minijinja/struct.State.html "struct minijinja::State").

This can be used to inspect the [`State`](https://docs.rs/minijinja/latest/minijinja/struct.State.html "struct minijinja::State") of the template post evaluation for instance to get fuel consumption numbers or to access globally set variables.

```rust
let tmpl = env.template_from_str("{% set x = 42 %}Hello !").unwrap();
let (rv, state) = tmpl.render_and_return_state(context!{ what => "World" }).unwrap();
assert_eq!(rv, "Hello World!");
assert_eq!(state.lookup("x"), Some(Value::from(42)));
```

**Note on values:** The [`Value`](https://docs.rs/minijinja/latest/minijinja/value/struct.Value.html "struct minijinja::value::Value") type implements `Serialize` and can be efficiently passed to render. It does not undergo actual serialization.

Like [`render`](https://docs.rs/minijinja/latest/minijinja/struct.Template.html#method.render "method minijinja::Template::render") but also returns the evaluated [`State`](https://docs.rs/minijinja/latest/minijinja/struct.State.html "struct minijinja::State") while keeping the template alive with the returned state.

This is primarily useful when working with temporary template handles, as the resulting [`State`](https://docs.rs/minijinja/latest/minijinja/struct.State.html "struct minijinja::State") can continue to be used through the returned wrapper.

```rust
let env = Environment::new();
let rendered = env
    .template_from_str("{% set x = 42 %}")
    .unwrap()
    .render_captured(())
    .unwrap();
assert_eq!(rendered.output(), "");
assert_eq!(rendered.state().lookup("x"), Some(Value::from(42)));
```

[Source](https://docs.rs/minijinja/latest/src/minijinja/template.rs.html#264-274)

#### pub fn <S: Serialize, W: Write>( &self, ctx: S, w: W, ) -> Result<Captured<'source>, Error>

Like [`render`](https://docs.rs/minijinja/latest/minijinja/struct.Template.html#method.render "method minijinja::Template::render") but writes to an [`io::Write`](https://doc.rust-lang.org/nightly/std/io/trait.Write.html "trait std::io::Write") and keeps the template alive with the returned state.

This is useful when working with temporary template handles and the state needs to be inspected afterwards. The [`output`](https://docs.rs/minijinja/latest/minijinja/struct.Captured.html#method.output "method minijinja::Captured::output") of the returned [`Captured`](https://docs.rs/minijinja/latest/minijinja/struct.Captured.html "struct minijinja::Captured") will be an empty string since the output was written to the provided writer.

```rust
let env = Environment::new();
let mut buf = Vec::new();
let captured = env
    .template_from_str("{% set x = 42 %}Hello!")
    .unwrap()
    .render_captured_to((), &mut buf)
    .unwrap();
assert_eq!(std::str::from_utf8(&buf).unwrap(), "Hello!");
assert_eq!(captured.output(), "");
```

[Source](https://docs.rs/minijinja/latest/src/minijinja/template.rs.html#340-349)

#### pub fn <S: Serialize, W: Write>( &self, ctx: S, w: W, ) -> Result<State<'\_, 'env>, Error>

👎Deprecated since 2.18.0:

use render\_captured\_to instead

Renders the template into an [`io::Write`](https://doc.rust-lang.org/nightly/std/io/trait.Write.html "trait std::io::Write").

This works exactly like [`render`](https://docs.rs/minijinja/latest/minijinja/struct.Template.html#method.render "method minijinja::Template::render"), but writes the template into an [`io::Write`](https://doc.rust-lang.org/nightly/std/io/trait.Write.html "trait std::io::Write") as it is evaluated.

```rust
use std::io::stdout;

let tmpl = env.get_template("hello").unwrap();
tmpl.render_to_write(context!(name => "John"), &mut stdout()).unwrap();
```

**Note on values:** The [`Value`](https://docs.rs/minijinja/latest/minijinja/value/struct.Value.html "struct minijinja::value::Value") type implements `Serialize` and can be efficiently passed to render. It does not undergo actual serialization.

[Source](https://docs.rs/minijinja/latest/src/minijinja/template.rs.html#373-386)

#### pub fn <S: Serialize>( &self, ctx: S, ) -> Result<State<'\_, 'env>, Error>

👎Deprecated since 2.18.0:

use render\_captured instead

Evaluates the template into a [`State`](https://docs.rs/minijinja/latest/minijinja/struct.State.html "struct minijinja::State").

This evaluates the template, discards the output and returns the final `State` for introspection. From there global variables or blocks can be accessed. What this does is quite similar to how the engine internally works with templates that are extended or imported from.

```rust
let tmpl = env.get_template("hello")?;
let state = tmpl.eval_to_state(context!(name => "John"))?;
println!("{:?}", state.exports());
```

If you also want to render, use [`render_captured`](https://docs.rs/minijinja/latest/minijinja/struct.Template.html#method.render_captured "method minijinja::Template::render_captured").

For more information see [`State`](https://docs.rs/minijinja/latest/minijinja/struct.State.html "struct minijinja::State").

Returns a set of all undeclared variables in the template.

This returns a set of all variables that might be looked up at runtime by the template. Since this runs a static analysis, the actual control flow is not considered. This also cannot take into account what happens due to includes, imports or extending. If `nested` is set to `true`, then also nested trivial attribute lookups are considered and returned.

```rust
let mut env = Environment::new();
env.add_template("x", "{% set x = foo %}").unwrap();
let tmpl = env.get_template("x").unwrap();
let undeclared = tmpl.undeclared_variables(false);
// returns ["foo", "bar"]
let undeclared = tmpl.undeclared_variables(true);
// returns ["foo", "bar.baz"]
```

Note that this does not special case global variables. This means that for instance a template that uses `namespace()` will return `namespace` in the return value.

[Source](https://docs.rs/minijinja/latest/src/minijinja/template.rs.html#443-450)

#### pub fn (&self) -> State<'\_, 'env>

Creates an empty [`State`](https://docs.rs/minijinja/latest/minijinja/struct.State.html "struct minijinja::State") for this template.

It’s very rare that you need to actually do this but it can be useful when testing values or working with macros or other callable objects from outside the template environment.

## Trait Implementations

[Source](https://docs.rs/minijinja/latest/src/minijinja/template.rs.html#54)

### impl<'env: 'source, 'source> Clone for Template<'env, 'source>

Returns a duplicate of the value. [Read more](https://doc.rust-lang.org/nightly/core/clone/trait.Clone.html#tymethod.clone)

1.0.0 (const: [unstable](https://github.com/rust-lang/rust/issues/142757 "Tracking issue for const_clone")) · [Source](https://doc.rust-lang.org/nightly/src/core/clone.rs.html#245-247)

#### fn clone\_from(&mut self, source: &Self)

Performs copy-assignment from `source`. [Read more](https://doc.rust-lang.org/nightly/core/clone/trait.Clone.html#method.clone_from)

Formats the value using the given formatter. [Read more](https://doc.rust-lang.org/nightly/core/fmt/trait.Debug.html#tymethod.fmt)

## Auto Trait Implementations

### impl<'env, 'source> Freeze for Template<'env, 'source>

### impl<'env, 'source>!RefUnwindSafe for Template<'env, 'source>

### impl<'env, 'source> Send for Template<'env, 'source>

### impl<'env, 'source> Sync for Template<'env, 'source>

### impl<'env, 'source> Unpin for Template<'env, 'source>

### impl<'env, 'source> UnsafeUnpin for Template<'env, 'source>

### impl<'env, 'source>!UnwindSafe for Template<'env, 'source>

## Blanket Implementations

[Source](https://doc.rust-lang.org/nightly/src/core/any.rs.html#141)

### impl<T> Any for Twhere T: 'static +?Sized,

Gets the `TypeId` of `self`. [Read more](https://doc.rust-lang.org/nightly/core/any/trait.Any.html#tymethod.type_id)

[Source](https://doc.rust-lang.org/nightly/src/core/borrow.rs.html#212)

### impl<T> Borrow<T> for Twhere T:?Sized,

Immutably borrows from an owned value. [Read more](https://doc.rust-lang.org/nightly/core/borrow/trait.Borrow.html#tymethod.borrow)

[Source](https://doc.rust-lang.org/nightly/src/core/borrow.rs.html#221)

### impl<T> BorrowMut<T> for Twhere T:?Sized,

[Source](https://doc.rust-lang.org/nightly/src/core/borrow.rs.html#222)

#### fn borrow\_mut(&mut self) -> &mut T

Mutably borrows from an owned value. [Read more](https://doc.rust-lang.org/nightly/core/borrow/trait.BorrowMut.html#tymethod.borrow_mut)

[Source](https://doc.rust-lang.org/nightly/src/core/clone.rs.html#547)

### impl<T> CloneToUninit for Twhere T: Clone,

🔬This is a nightly-only experimental API. (`clone_to_uninit`)

Performs copy-assignment from `self` to `dest`. [Read more](https://doc.rust-lang.org/nightly/core/clone/trait.CloneToUninit.html#tymethod.clone_to_uninit)

[Source](https://doc.rust-lang.org/nightly/src/core/convert/mod.rs.html#786)

### impl<T> From<T> for T

[Source](https://doc.rust-lang.org/nightly/src/core/convert/mod.rs.html#789)

#### fn from(t: T) -> T

Returns the argument unchanged.

[Source](https://doc.rust-lang.org/nightly/src/core/convert/mod.rs.html#768-770)

### impl<T, U> Into<U> for Twhere U: From<T>,

[Source](https://doc.rust-lang.org/nightly/src/core/convert/mod.rs.html#778)

#### fn into(self) -> U

Calls `U::from(self)`.

That is, this conversion is whatever the implementation of `From<T> for U` chooses to do.

[Source](https://doc.rust-lang.org/nightly/src/alloc/borrow.rs.html#72-74)

### impl<T> ToOwned for Twhere T: Clone,

[Source](https://doc.rust-lang.org/nightly/src/alloc/borrow.rs.html#76)

#### type Owned = T

The resulting type after obtaining ownership.

[Source](https://doc.rust-lang.org/nightly/src/alloc/borrow.rs.html#77)

#### fn to\_owned(&self) -> T

Creates owned data from borrowed data, usually by cloning. [Read more](https://doc.rust-lang.org/nightly/alloc/borrow/trait.ToOwned.html#tymethod.to_owned)

Uses borrowed data to replace owned data, usually by cloning. [Read more](https://doc.rust-lang.org/nightly/alloc/borrow/trait.ToOwned.html#method.clone_into)

[Source](https://doc.rust-lang.org/nightly/src/core/convert/mod.rs.html#828-830)

### impl<T, U> TryFrom<U> for Twhere U: Into<T>,

The type returned in the event of a conversion error.

Performs the conversion.

[Source](https://doc.rust-lang.org/nightly/src/core/convert/mod.rs.html#812-814)

### impl<T, U> TryInto<U> for Twhere U: TryFrom<T>,

The type returned in the event of a conversion error.

Performs the conversion.
