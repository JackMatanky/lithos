# minijinja - Rust Crate

[Source](https://docs.rs/minijinja/latest/minijinja/)

---

**MiniJinja: a powerful template engine for Rust with minimal dependencies**

MiniJinja is a powerful but minimal dependency template engine for Rust which is based on the syntax and behavior of the [Jinja2](https://jinja.palletsprojects.com/) template engine for Python. It’s implemented on top of [`serde`](https://docs.rs/serde/1.0.219/x86_64-unknown-linux-gnu/serde/index.html "mod serde"). The goal is to be able to render a large chunk of the Jinja2 template ecosystem from Rust with a minimal engine and to leverage an already existing ecosystem of editor integrations.

```django
{% for user in users %}
  <li></li>
{% endfor %}
```

You can play with MiniJinja online [in the browser playground](https://mitsuhiko.github.io/minijinja-playground/) powered by a WASM build of MiniJinja.

## Why MiniJinja

MiniJinja by its name wants to be a good default choice if you need a little bit of templating with minimal dependencies. It has the following goals:

- Well documented, compact API
- Minimal dependencies, reasonable compile times and decent runtime performance
- Stay close as possible to Jinja2
- Support for expression evaluation
- Support for all `serde` compatible types
- Excellent test coverage
- Support for dynamic runtime objects with methods and dynamic attributes

## Template Usage

To use MiniJinja one needs to create an [`Environment`](https://docs.rs/minijinja/latest/minijinja/struct.Environment.html "struct minijinja::Environment") and populate it with templates. Afterwards templates can be loaded and rendered. To pass data one can pass any serde serializable value. The [`context!`](https://docs.rs/minijinja/latest/minijinja/macro.context.html "macro minijinja::context") macro can be used to quickly construct a template context:

```rust
use minijinja::{Environment, context};

let mut env = Environment::new();
env.add_template("hello", "Hello !").unwrap();
let tmpl = env.get_template("hello").unwrap();
println!("{}", tmpl.render(context!(name => "John")).unwrap());
```

```
Hello John!
```

For super trivial cases where you need to render a string once, you can also use the [`render!`](https://docs.rs/minijinja/latest/minijinja/macro.render.html "macro minijinja::render") macro which acts a bit like a replacement for the [`format!`](https://doc.rust-lang.org/nightly/alloc/macro.format.html "macro alloc::format") macro.

## Expression Usage

MiniJinja — like Jinja2 — allows to be used as expression language. This can be useful to express logic in configuration files or similar things. For this purpose the [`Environment::compile_expression`](https://docs.rs/minijinja/latest/minijinja/struct.Environment.html#method.compile_expression "method minijinja::Environment::compile_expression") method can be used. It returns an expression object that can then be evaluated, returning the result:

```rust
use minijinja::{Environment, context};

let env = Environment::new();
let expr = env.compile_expression("number < 42").unwrap();
let result = expr.eval(context!(number => 23)).unwrap();
assert_eq!(result.is_true(), true);
```

This becomes particularly powerful when [dynamic objects](https://docs.rs/minijinja/latest/minijinja/value/trait.Object.html "trait minijinja::value::Object") are exposed to templates.

## Custom Filters

MiniJinja lets you register functions as filter functions (see [`Function`](https://docs.rs/minijinja/latest/minijinja/functions/trait.Function.html "trait minijinja::functions::Function")) with the engine. These can then be invoked directly from the template:

```rust
use minijinja::{Environment, context};

let mut env = Environment::new();
env.add_filter("repeat", str::repeat);
env.add_template("hello", "Na  !").unwrap();
let tmpl = env.get_template("hello").unwrap();
println!("{}", tmpl.render(context!(name => "Batman")).unwrap());
```

```
Na Na Na Batman!
```

## Learn more

- [`Environment`](https://docs.rs/minijinja/latest/minijinja/struct.Environment.html "struct minijinja::Environment"): the main API entry point. Teaches you how to configure the environment.
- [`Template`](https://docs.rs/minijinja/latest/minijinja/struct.Template.html "struct minijinja::Template"): the template object API. Shows you how templates can be rendered.
- [`syntax`](https://docs.rs/minijinja/latest/minijinja/syntax/index.html "mod minijinja::syntax"): provides documentation of the template engine syntax.
- [`filters`](https://docs.rs/minijinja/latest/minijinja/filters/index.html "mod minijinja::filters"): teaches you how to write custom filters and to see the list of built-in filters.
- [`tests`](https://docs.rs/minijinja/latest/minijinja/tests/index.html "mod minijinja::tests"): teaches you how to write custom test functions and to see the list of built-in tests.
- [`functions`](https://docs.rs/minijinja/latest/minijinja/functions/index.html "mod minijinja::functions"): teaches how to write custom functions and to see the list of built-in functions.
- For auto reloading use the [`minijinja-autoreload`](https://docs.rs/minijinja-autoreload) crate.
- For simpler embedding of templates use the [`minijinja-embed`](https://docs.rs/minijinja-embed) crate.

Additionally there is an [list of examples](https://github.com/mitsuhiko/minijinja/tree/main/examples) with many different small example programs on GitHub to explore.

## Error Handling

MiniJinja tries to give you good errors out of the box. However if you use includes or template inheritance your experience will improve greatly if you ensure to render chained errors. For more information see [`Error`](https://docs.rs/minijinja/latest/minijinja/struct.Error.html "struct minijinja::Error") with an example.

## Size and Compile Times

MiniJinja attempts to compile fast so it can be used as a sensible template engine choice when compile times matter. Because of this it’s internally modular so unnecessary bits and pieces can be removed. The default set of features tries to strike a balance but in situations where only a subset is needed (eg: `build.rs`) all default features can be be disabled.

## Optional Features

MiniJinja comes with a lot of optional features, some of which are turned on by default. If you plan on using MiniJinja in a library, please consider turning off all default features and to opt-in explicitly into the ones you actually need.

**Configurable Features**

To cut down on size of the engine some default functionality can be removed:

- **Engine Features:**
	- `builtins`: if this feature is removed the default filters, tests and functions are not implemented.
		- `macros`: when removed the `{% macro %}` tag is not included.
		- `multi_template`: when removed the templates related to imports and extends are removed (`{% from %}`, `{% import %}`, `{% include %}`, and `{% extends %}` as well as `{% block %}`).
		- `adjacent_loop_items`: when removed the `previtem` and `nextitem` attributes of the `loop` object no longer exist. Removing this feature can provide faster template execution when a lot of loops are involved.
		- `unicode`: when added unicode identifiers are supported and the `sort` filter’s case insensitive comparison changes to using unicode and not ASCII rules. Without this features only ASCII identifiers can be used for variable names and attributes.
		- `serde`: enables or disables serde support. In current versions of MiniJinja it’s not possible to disable serde but it will become possible. To prevent breakage, MiniJinja warns if this feature is disabled.
- **Rust Functionality:**
	- `debug`: if this feature is removed some debug functionality of the engine is removed as well. This mainly affects the quality of error reporting.
		- `deserialization`: when removed this disables deserialization support for the [`Value`](https://docs.rs/minijinja/latest/minijinja/value/struct.Value.html "struct minijinja::value::Value") type, removes the `ViaDeserialize` type and the error type no longer implements `serde::de::Error`.
		- `std_collections`: if this feature is removed some [`Object`](https://docs.rs/minijinja/latest/minijinja/value/trait.Object.html "trait minijinja::value::Object") implementations for standard library collections are removed. Only the ones needed for the engine to function itself are retained.

There are some additional features that provide extra functionality:

- `fuel`: enables the `fuel` feature which makes the engine track fuel consumption which can be used to better protect against expensive templates.
- `loader`: retained for backwards compatibility and now a no-op.
- `custom_syntax`: when this feature is enabled, custom delimiters are supported by the parser.
- `preserve_order`: When enable the internal value implementation uses an indexmap which preserves the original order of maps and structs.
- `json`: When enabled the `tojson` filter is added as builtin filter as well as the ability to auto escape via `AutoEscape::Json`.
- `urlencode`: When enabled the `urlencode` filter is added as builtin filter.
- `loop_controls`: enables the `{% break %}` and `{% continue %}` loop control flow tags.

Performance and memory related features:

- `stacker`: enables automatic stack growth which permits much larger levels of recursion at runtime. This does not affect the maximum recursion at parsing time which is always limited.
- `speedups`: enables all speedups, in particular it turns on the `v_htmlescape` dependency for faster HTML escaping.

Internals:

- `unstable_machinery`: exposes an unstable internal API (no semver guarantees) to parse templates and interact with the engine internals. If you need this functionality, please leave some feedback on GitHub.

## Re-exports

`pub use self::value::Value;`

## Modules

| Module | Description |
|--------|-------------|
| [filters](https://docs.rs/minijinja/latest/minijinja/filters/index.html "mod minijinja::filters") | Filter functions and abstractions. |
| [functions](https://docs.rs/minijinja/latest/minijinja/functions/index.html "mod minijinja::functions") | Global functions and abstractions. |
| [syntax](https://docs.rs/minijinja/latest/minijinja/syntax/index.html "mod minijinja::syntax") | Documents the syntax for templates and provides ways to reconfigure it. |
| [tests](https://docs.rs/minijinja/latest/minijinja/tests/index.html "mod minijinja::tests") | Test functions and abstractions. |
| [value](https://docs.rs/minijinja/latest/minijinja/value/index.html "mod minijinja::value") | Provides a dynamic value type abstraction. |

## Macros

| Macro | Description |
|-------|-------------|
| [args](https://docs.rs/minijinja/latest/minijinja/macro.args.html "macro minijinja::args") | An utility macro to create arguments for function calls. |
| [context](https://docs.rs/minijinja/latest/minijinja/macro.context.html "macro minijinja::context") | Creates a template context from keys and values or merging in another value. |
| [render](https://docs.rs/minijinja/latest/minijinja/macro.render.html "macro minijinja::render") | A macro similar to [`format!`](https://doc.rust-lang.org/nightly/alloc/macro.format.html "macro alloc::format") but that uses MiniJinja for rendering. |


## Structs

| Struct | Description |
|--------|-------------|
| [Captured](https://docs.rs/minijinja/latest/minijinja/struct.Captured.html "struct minijinja::Captured") | Represents a rendered template output together with a captured [`State`](https://docs.rs/minijinja/latest/minijinja/struct.State.html "struct minijinja::State"). |
| [Environment](https://docs.rs/minijinja/latest/minijinja/struct.Environment.html "struct minijinja::Environment") | An abstraction that holds the engine configuration. |
| [Error](https://docs.rs/minijinja/latest/minijinja/struct.Error.html "struct minijinja::Error") | Represents template errors. |
| [Expression](https://docs.rs/minijinja/latest/minijinja/struct.Expression.html "struct minijinja::Expression") | A handle to a compiled expression. |
| [HtmlEscape](https://docs.rs/minijinja/latest/minijinja/struct.HtmlEscape.html "struct minijinja::HtmlEscape") | Helper to HTML escape a string. |
| [Output](https://docs.rs/minijinja/latest/minijinja/struct.Output.html "struct minijinja::Output") | An abstraction over [`fmt::Write`](https://doc.rust-lang.org/nightly/core/fmt/trait.Write.html "trait core::fmt::Write") for the rendering. |
| [State](https://docs.rs/minijinja/latest/minijinja/struct.State.html "struct minijinja::State") | Provides access to the current execution state of the engine. |
| [Template](https://docs.rs/minijinja/latest/minijinja/struct.Template.html "struct minijinja::Template") | Represents a handle to a template. |

## Enums

| Enum | Description |
|------|-------------|
| [AutoEscape](https://docs.rs/minijinja/latest/minijinja/enum.AutoEscape.html "enum minijinja::AutoEscape") | Controls the autoescaping behavior. |
| [ErrorKind](https://docs.rs/minijinja/latest/minijinja/enum.ErrorKind.html "enum minijinja::ErrorKind") | An enum describing the error kind. |
| [FormatStyle](https://docs.rs/minijinja/latest/minijinja/enum.FormatStyle.html "enum minijinja::FormatStyle") | Controls the style of the format string. |
| [UndefinedBehavior](https://docs.rs/minijinja/latest/minijinja/enum.UndefinedBehavior.html "enum minijinja::UndefinedBehavior") | Defines the behavior of undefined values in the engine. |

## Functions

| Function | Description |
|----------|-------------|
| [default\_auto\_escape\_callback](https://docs.rs/minijinja/latest/minijinja/fn.default_auto_escape_callback.html "fn minijinja::default_auto_escape_callback") | The default logic for auto escaping based on file extension. |
| [escape\_formatter](https://docs.rs/minijinja/latest/minijinja/fn.escape_formatter.html "fn minijinja::escape_formatter") | The default formatter. |
| [format\_filter](https://docs.rs/minijinja/latest/minijinja/fn.format_filter.html "fn minijinja::format_filter") `builtins` | A helper function to apply a set of values to a given format string. |
| [path\_loader](https://docs.rs/minijinja/latest/minijinja/fn.path_loader.html "fn minijinja::path_loader") | Helper to load templates from a given directory. |
