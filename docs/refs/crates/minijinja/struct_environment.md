# minijinja: Struct `Environment`

[Source](https://docs.rs/minijinja/latest/minijinja/struct.Environment.html)

```rust
pub struct Environment<'source> { /* private fields */ }
```

An abstraction that holds the engine configuration.

This object holds the central configuration state for templates. It is also the container for all loaded templates.

The environment holds references to the source the templates were created from. This makes it very inconvenient to pass around unless the templates are static strings.

There are generally two ways to construct an environment:

- [`Environment::new`](https://docs.rs/minijinja/latest/minijinja/struct.Environment.html#method.new "associated function minijinja::Environment::new") creates an environment preconfigured with sensible defaults. It will contain all built-in filters, tests and globals as well as a callback for auto escaping based on file extension.
- [`Environment::empty`](https://docs.rs/minijinja/latest/minijinja/struct.Environment.html#method.empty "associated function minijinja::Environment::empty") creates a completely blank environment.

## Implementations

[Source](https://docs.rs/minijinja/latest/src/minijinja/environment.rs.html#105-857)

### impl<'source> Environment<'source>

Creates a new environment with sensible defaults.

This environment does not yet contain any templates but it will have all the default filters, tests and globals loaded. If you do not want any default configuration you can use the alternative [`empty`](https://docs.rs/minijinja/latest/minijinja/struct.Environment.html#method.empty "associated function minijinja::Environment::empty") method.

Creates a completely empty environment.

This environment has no filters, no templates, no globals and no default logic for auto escaping configured.

[Source](https://docs.rs/minijinja/latest/src/minijinja/environment.rs.html#180-182)

#### pub fn ( &mut self, name: &'source str, source: &'source str, ) -> Result<(), Error>

Loads a template from a string into the environment.

The `name` parameter defines the name of the template which identifies it. To look up a loaded template use the [`get_template`](https://docs.rs/minijinja/latest/minijinja/struct.Environment.html#method.get_template "method minijinja::Environment::get_template") method.

```rust
let mut env = Environment::new();
env.add_template("index.html", "Hello !").unwrap();
```

This method fails if the template has a syntax error.

Note that there are situations where the interface of this method is too restrictive as you need to hold on to the strings for the lifetime of the environment. To avoid this restriction use [`add_template_owned`](https://docs.rs/minijinja/latest/minijinja/struct.Environment.html#method.add_template_owned "method minijinja::Environment::add_template_owned").

Adds a template without borrowing.

This lets you place an owned [`String`](https://doc.rust-lang.org/nightly/alloc/string/struct.String.html "struct alloc::string::String") in the environment rather than the borrowed `&str` without having to worry about lifetimes.

```rust
let mut env = Environment::new();
env.add_template_owned("index.html".to_string(), "Hello !".to_string()).unwrap();
```

**Note**: the name is a bit of a misnomer as this API also allows borrowing, as the parameters are actually [`Cow`](https://doc.rust-lang.org/nightly/alloc/borrow/enum.Cow.html "enum alloc::borrow::Cow"). This method fails if the template has a syntax error.

Register a template loader as source of templates.

When a template loader is registered, the environment gains the ability to dynamically load templates. The loader is invoked with the name of the template. If this template exists `Ok(Some(template_source))` has to be returned, otherwise `Ok(None)`. Once a template has been loaded it’s stored on the environment. This means the loader is only invoked once per template name.

For loading templates from the file system, you can use the [`path_loader`](https://docs.rs/minijinja/latest/minijinja/fn.path_loader.html "fn minijinja::path_loader") function.

##### Example

```rust
fn create_env() -> Environment<'static> {
    let mut env = Environment::new();
    env.set_loader(|name| {
        if name == "layout.html" {
            Ok(Some("...".into()))
        } else {
            Ok(None)
        }
    });
    env
}
```

[Source](https://docs.rs/minijinja/latest/src/minijinja/environment.rs.html#248-253)

#### pub fn (&mut self, yes: bool)

Preserve the trailing newline when rendering templates.

The default is `false`, which causes a single newline, if present, to be stripped from the end of the template.

This setting is used whenever a template is loaded into the environment. Changing it at a later point only affects future templates loaded.

Returns the value of the trailing newline preservation flag.

[Source](https://docs.rs/minijinja/latest/src/minijinja/environment.rs.html#267-269)

#### pub fn (&mut self, yes: bool)

Remove the first newline after a block.

If this is set to `true` then the first newline after a block is removed (block, not variable tag!). Defaults to `false`.

Returns the value of the trim blocks flag.

[Source](https://docs.rs/minijinja/latest/src/minijinja/environment.rs.html#280-282)

#### pub fn (&mut self, yes: bool)

Remove leading spaces and tabs from the start of a line to a block.

If this is set to `true` then leading spaces and tabs from the start of a line to the block tag are removed.

Returns the value of the lstrip blocks flag.

[Source](https://docs.rs/minijinja/latest/src/minijinja/environment.rs.html#290-292)

#### pub fn (&mut self, name: &str)

Removes a template by name.

[Source](https://docs.rs/minijinja/latest/src/minijinja/environment.rs.html#317-322)

#### pub fn <F>(&mut self, f: F)where F: for<'s> Fn(&'s str, &'s str) -> Cow<'s, str> + Send + Sync + 'static,

Sets a callback to join template paths.

By default this returns the template path unchanged, but it can be used to implement relative path resolution between templates. The first argument to the callback is the name of the template to be loaded, the second argument is the parent path.

The following example demonstrates how a basic path joining algorithm can be implemented.

```rust
env.set_path_join_callback(|name, parent| {
    let mut rv = parent.split('/').collect::<Vec<_>>();
    rv.pop();
    name.split('/').for_each(|segment| match segment {
        "." => {}
        ".." => { rv.pop(); }
        _ => { rv.push(segment); }
    });
    rv.join("/").into()
});
```

Sets a callback invoked for unknown methods on objects.

This registers a function with the environment that is invoked when invoking a method on a value results in a [`UnknownMethod`](https://docs.rs/minijinja/latest/minijinja/enum.ErrorKind.html#variant.UnknownMethod "variant minijinja::ErrorKind::UnknownMethod") error. In that case the callback is invoked with [`State`](https://docs.rs/minijinja/latest/minijinja/struct.State.html "struct minijinja::State"), the [`Value`](https://docs.rs/minijinja/latest/minijinja/value/struct.Value.html "struct minijinja::value::Value"), the name of the method as `&str` as well as all arguments in a slice.

This for instance implements a `.items()` method that invokes the `|items` filter:

```rust
use minijinja::value::{ValueKind, from_args};
use minijinja::{Error, ErrorKind};

env.set_unknown_method_callback(|state, value, method, args| {
    if value.kind() == ValueKind::Map && method == "items" {
        let _: () = from_args(args)?;
        state.apply_filter("items", &[value.clone()])
    } else {
        Err(Error::from(ErrorKind::UnknownMethod))
    }
});
```

This can be used to increase the compatibility with Jinja2 templates that might call Python methods on objects which are not available in minijinja. A range of common Python methods is implemented in `minijinja-contrib`. For more information see [minijinja\_contrib::pycompat](https://docs.rs/minijinja-contrib/latest/minijinja_contrib/pycompat/).

[Source](https://docs.rs/minijinja/latest/src/minijinja/environment.rs.html#364-366)

#### pub fn (&mut self)

Removes all stored templates.

This method is mainly useful when combined with a loader as it causes the loader to “reload” templates. By calling this method one can trigger a reload.

[Source](https://docs.rs/minijinja/latest/src/minijinja/environment.rs.html#383-388)

#### pub fn (&self) -> impl Iterator<Item = (&str, Template<'\_, '\_>)>

Returns an iterator over the already loaded templates and their names.

Only templates that are already loaded will be returned.

```rust
let mut env = Environment::new();
env.add_template("hello.txt", "Hello !").unwrap();
env.add_template("goodbye.txt", "Goodbye !").unwrap();

for (name, tmpl) in env.templates() {
    println!("{}", tmpl.render(context!{ name => "World" }).unwrap());
}
```

Fetches a template by name.

This requires that the template has been loaded with [`add_template`](https://docs.rs/minijinja/latest/minijinja/struct.Environment.html#method.add_template "method minijinja::Environment::add_template") beforehand. If the template was not loaded an error of kind `TemplateNotFound` is returned. If a loader was added to the engine this can also dynamically load templates.

```rust
let mut env = Environment::new();
env.add_template("hello.txt", "Hello !").unwrap();
let tmpl = env.get_template("hello.txt").unwrap();
println!("{}", tmpl.render(context!{ name => "World" }).unwrap());
```

[Source](https://docs.rs/minijinja/latest/src/minijinja/environment.rs.html#420-433)

#### pub fn ( &self, name: &'source str, source: &'source str, ) -> Result<Template<'\_, 'source>, Error>

Loads a template from a string.

In some cases you only need to work with (e.g., render) a template once.

```rust
let env = Environment::new();
let tmpl = env.template_from_named_str("template_name", "Hello ").unwrap();
let rv = tmpl.render(context! { name => "World" });
println!("{}", rv.unwrap());
```

[Source](https://docs.rs/minijinja/latest/src/minijinja/environment.rs.html#439-441)

#### pub fn ( &self, source: &'source str, ) -> Result<Template<'\_, 'source>, Error>

Loads a template from a string, with name `<string>`.

This is a shortcut to [`template_from_named_str`](https://docs.rs/minijinja/latest/minijinja/struct.Environment.html#method.template_from_named_str "method minijinja::Environment::template_from_named_str") with name set to `<string>`.

Parses and renders a template from a string in one go with name.

Like [`render_str`](https://docs.rs/minijinja/latest/minijinja/struct.Environment.html#method.render_str "method minijinja::Environment::render_str"), but provide a name for the template to be used instead of the default `<string>`. This is an alias for [`template_from_named_str`](https://docs.rs/minijinja/latest/minijinja/struct.Environment.html#method.template_from_named_str "method minijinja::Environment::template_from_named_str") paired with [`render`](https://docs.rs/minijinja/latest/minijinja/struct.Template.html#method.render "method minijinja::Template::render").

```rust
let env = Environment::new();
let rv = env.render_named_str(
    "template_name",
    "Hello ",
    context!{ name => "World" }
);
println!("{}", rv.unwrap());
```

**Note on values:** The [`Value`](https://docs.rs/minijinja/latest/minijinja/value/struct.Value.html "struct minijinja::value::Value") type implements `Serialize` and can be efficiently passed to render. It does not undergo actual serialization.

Parses and renders a template from a string in one go.

In some cases you really only need a template to be rendered once from a string and returned. The internal name of the template is `<string>`.

This is an alias for [`template_from_str`](https://docs.rs/minijinja/latest/minijinja/struct.Environment.html#method.template_from_str "method minijinja::Environment::template_from_str") paired with [`render`](https://docs.rs/minijinja/latest/minijinja/struct.Template.html#method.render "method minijinja::Template::render").

**Note on values:** The [`Value`](https://docs.rs/minijinja/latest/minijinja/value/struct.Value.html "struct minijinja::value::Value") type implements `Serialize` and can be efficiently passed to render. It does not undergo actual serialization.

[Source](https://docs.rs/minijinja/latest/src/minijinja/environment.rs.html#508-513)

#### pub fn <F>(&mut self, f: F)where F: Fn(&str) -> AutoEscape + 'static + Sync + Send,

Sets a new function to select the default auto escaping.

This function is invoked when templates are loaded into the environment to determine the default auto escaping behavior. The function is invoked with the name of the template and can make an initial auto escaping decision based on that. The default implementation ([`default_auto_escape_callback`](https://docs.rs/minijinja/latest/minijinja/fn.default_auto_escape_callback.html "fn minijinja::default_auto_escape_callback")) turns on escaping depending on the file extension.

```rust
env.set_auto_escape_callback(|name| {
    if matches!(name.rsplit('.').next().unwrap_or(""), "html" | "htm" | "aspx") {
        AutoEscape::Html
    } else {
        AutoEscape::None
    }
});
```

[Source](https://docs.rs/minijinja/latest/src/minijinja/environment.rs.html#520-522)

#### pub fn (&mut self, behavior: UndefinedBehavior)

Changes the undefined behavior.

This changes the runtime behavior of [`undefined`](https://docs.rs/minijinja/latest/minijinja/value/struct.Value.html#associatedconstant.UNDEFINED "associated constant minijinja::value::Value::UNDEFINED") values in the template engine. For more information see [`UndefinedBehavior`](https://docs.rs/minijinja/latest/minijinja/enum.UndefinedBehavior.html "enum minijinja::UndefinedBehavior"). The default is [`UndefinedBehavior::Lenient`](https://docs.rs/minijinja/latest/minijinja/enum.UndefinedBehavior.html#variant.Lenient "variant minijinja::UndefinedBehavior::Lenient").

Returns the current undefined behavior.

This is particularly useful if a filter function or similar wants to change its behavior with regards to undefined values.

Sets a different formatter function.

The formatter is invoked to format the given value into the provided [`Output`](https://docs.rs/minijinja/latest/minijinja/struct.Output.html "struct minijinja::Output"). The default implementation is [`escape_formatter`](https://docs.rs/minijinja/latest/minijinja/fn.escape_formatter.html "fn minijinja::escape_formatter").

When implementing a custom formatter it depends on if auto escaping should be supported or not. If auto escaping should be supported then it’s easiest to just wrap the default formatter. The following example swaps out `None` values before rendering for `Undefined` which renders as an empty string instead.

The current value of the auto escape flag can be retrieved directly from the [`State`](https://docs.rs/minijinja/latest/minijinja/struct.State.html "struct minijinja::State").

```rust
use minijinja::escape_formatter;
use minijinja::value::Value;

env.set_formatter(|out, state, value| {
    escape_formatter(
        out,
        state,
        if value.is_none() {
            &Value::UNDEFINED
        } else {
            value
        },
    )
});
```

[Source](https://docs.rs/minijinja/latest/src/minijinja/environment.rs.html#589-591)

#### pub fn (&mut self, enabled: bool)

Available on **crate feature `debug`** only.

Enable or disable the debug mode.

When the debug mode is enabled the engine will dump out some of the execution state together with the source information of the executing template when an error is created. The cost of this is relatively high as the data including the template source is cloned.

When this is enabled templates will print debug information with source context when the error is printed.

This requires the `debug` feature. This is enabled by default if debug assertions are enabled and false otherwise.

Available on **crate feature `debug`** only.

Returns the current value of the debug flag.

Available on **crate feature `fuel`** only.

Sets the optional fuel of the engine.

When MiniJinja is compiled with the `fuel` feature then every instruction consumes a certain amount of fuel. Usually `1`, some will consume no fuel. By default the engine has the fuel feature disabled (`None`). To turn on fuel set something like `Some(50000)` which will allow 50.000 instructions to execute before running out of fuel.

To find out how much fuel is consumed, you can access the fuel levels from the [`State`](https://docs.rs/minijinja/latest/minijinja/struct.State.html "struct minijinja::State").

Fuel consumed per-render.

Available on **crate feature `fuel`** only.

Returns the configured fuel.

[Source](https://docs.rs/minijinja/latest/src/minijinja/environment.rs.html#632-634)

#### pub fn (&mut self, syntax: SyntaxConfig)

Available on **crate feature `custom_syntax`** only.

Sets the syntax for the environment.

This setting is used whenever a template is loaded into the environment. Changing it at a later point only affects future templates loaded.

See [`SyntaxConfig`](https://docs.rs/minijinja/latest/minijinja/syntax/struct.SyntaxConfig.html "struct minijinja::syntax::SyntaxConfig") for more information.

Available on **crate feature `custom_syntax`** only.

Returns the current syntax config.

[Source](https://docs.rs/minijinja/latest/src/minijinja/environment.rs.html#660-669)

#### pub fn (&mut self, level: usize)

Reconfigures the runtime recursion limit.

This defaults to `500`. Raising it above that level requires the `stacker` feature to be enabled. Otherwise the limit is silently capped at that safe maximum. Note that the maximum is not necessarily safe if the thread uses a lot of stack space already, it’s just a value that was validated once to provide basic protection.

Every operation that requires recursion in MiniJinja increments an internal recursion counter. The actual cost attributed to that recursion depends on the cost of the operation. If statements and for loops for instance only increase the counter by 1, whereas template includes and macros might increase it to 10 or more.

**Note on stack growth:** even if the stacker feature is enabled it does not mean that in all cases stack can grow to the limits desired. For instance in WASM the maximum limits are additionally enforced by the runtime.

Returns the current max recursion limit.

[Source](https://docs.rs/minijinja/latest/src/minijinja/environment.rs.html#682-685)

#### pub fn ( &self, expr: &'source str, ) -> Result<Expression<'\_, 'source>, Error>

Compiles an expression.

This lets you compile an expression in the template language and evaluate it. This makes it possible to use the language’s expressions as a minimal scripting language. For more information and an example see [`Expression`](https://docs.rs/minijinja/latest/minijinja/struct.Expression.html "struct minijinja::Expression").

[Source](https://docs.rs/minijinja/latest/src/minijinja/environment.rs.html#691-699)

#### pub fn <E>( &self, expr: E, ) -> Result<Expression<'\_, 'source>, Error>

Compiles an expression without capturing the lifetime.

This works exactly like [`compile_expression`](https://docs.rs/minijinja/latest/minijinja/struct.Environment.html#method.compile_expression "method minijinja::Environment::compile_expression") but lets you pass an owned string without capturing the lifetime.

[Source](https://docs.rs/minijinja/latest/src/minijinja/environment.rs.html#717-725)

#### pub fn <N, F, Rv, Args>(&mut self, name: N, f: F)where N: Into<Cow<'source, str>>, F: Function<Rv, Args>, Rv: FunctionResult, Args: for<'a> FunctionArgs<'a>,

Adds a new filter function.

Filter functions are functions that can be applied to values in templates. For details about filters have a look at [`filters`](https://docs.rs/minijinja/latest/minijinja/filters/index.html "mod minijinja::filters").

[Source](https://docs.rs/minijinja/latest/src/minijinja/environment.rs.html#728-730)

#### pub fn (&mut self, name: &str)

Removes a filter by name.

[Source](https://docs.rs/minijinja/latest/src/minijinja/environment.rs.html#737-745)

#### pub fn <N, F, Rv, Args>(&mut self, name: N, f: F)where N: Into<Cow<'source, str>>, F: Function<Rv, Args>, Rv: FunctionResult, Args: for<'a> FunctionArgs<'a>,

Adds a new test function.

Test functions are similar to filters but perform a check on a value where the return value is always true or false. For details about tests have a look at [`tests`](https://docs.rs/minijinja/latest/minijinja/tests/index.html "mod minijinja::tests").

[Source](https://docs.rs/minijinja/latest/src/minijinja/environment.rs.html#748-750)

#### pub fn (&mut self, name: &str)

Removes a test by name.

[Source](https://docs.rs/minijinja/latest/src/minijinja/environment.rs.html#761-769)

#### pub fn <N, F, Rv, Args>(&mut self, name: N, f: F)where N: Into<Cow<'source, str>>, F: Function<Rv, Args>, Rv: FunctionResult, Args: for<'a> FunctionArgs<'a>,

Adds a new global function.

For details about functions have a look at [`functions`](https://docs.rs/minijinja/latest/minijinja/functions/index.html "mod minijinja::functions"). Note that functions and other global variables share the same namespace. For more details about functions have a look at [`Function`](https://docs.rs/minijinja/latest/minijinja/functions/trait.Function.html "trait minijinja::functions::Function").

This is a shortcut for calling [`add_global`](https://docs.rs/minijinja/latest/minijinja/struct.Environment.html#method.add_global "method minijinja::Environment::add_global") with the function wrapped with [`Value::from_function`](https://docs.rs/minijinja/latest/minijinja/value/struct.Value.html#method.from_function "associated function minijinja::value::Value::from_function").

[Source](https://docs.rs/minijinja/latest/src/minijinja/environment.rs.html#772-778)

#### pub fn <N, V>(&mut self, name: N, value: V)

Adds a global variable.

[Source](https://docs.rs/minijinja/latest/src/minijinja/environment.rs.html#781-783)

#### pub fn (&mut self, name: &str)

Removes a global function or variable by name.

Returns an iterator of all global variables.

[Source](https://docs.rs/minijinja/latest/src/minijinja/environment.rs.html#793-795)

#### pub fn (&self) -> State<'\_, '\_>

Returns an empty [`State`](https://docs.rs/minijinja/latest/minijinja/struct.State.html "struct minijinja::State") for testing purposes and similar.

## Trait Implementations

Returns a duplicate of the value. [Read more](https://doc.rust-lang.org/nightly/core/clone/trait.Clone.html#tymethod.clone)

1.0.0 (const: [unstable](https://github.com/rust-lang/rust/issues/142757 "Tracking issue for const_clone")) · [Source](https://doc.rust-lang.org/nightly/src/core/clone.rs.html#245-247)

#### fn clone\_from(&mut self, source: &Self)

Performs copy-assignment from `source`. [Read more](https://doc.rust-lang.org/nightly/core/clone/trait.Clone.html#method.clone_from)

Formats the value using the given formatter. [Read more](https://doc.rust-lang.org/nightly/core/fmt/trait.Debug.html#tymethod.fmt)

[Source](https://docs.rs/minijinja/latest/src/minijinja/environment.rs.html#89-91)

#### fn default() -> Self

Returns the “default value” for a type. [Read more](https://doc.rust-lang.org/nightly/core/default/trait.Default.html#tymethod.default)

## Auto Trait Implementations

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
