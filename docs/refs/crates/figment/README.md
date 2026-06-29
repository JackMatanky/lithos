# figment - Rust Crate

Source: <https://docs.rs/figment/latest/figment/>

---

```rust
use serde::Deserialize;
use figment::{Figment, providers::{Format, Toml, Json, Env}};

#[derive(Deserialize)]
struct Package {
    name: String,
    description: Option<String>,
    authors: Vec<String>,
    publish: Option<bool>,
    // ... and so on ...
}

#[derive(Deserialize)]
struct Config {
    package: Package,
    rustc: Option<String>,
    rustdoc: Option<String>,
    // ... and so on ...
}

let config: Config = Figment::new()
    .merge(Toml::file("Cargo.toml"))
    .merge(Env::prefixed("CARGO_"))
    .merge(Env::raw().only(&["RUSTC", "RUSTDOC"]))
    .join(Json::file("Cargo.json"))
    .extract()?;
```

## Table of Contents

- [Overview](#overview) - A brief overview of the entire crate.
- [Metadata](#metadata) - Figment’s value metadata tracking.
- [Extracting and Profiles](#extracting-and-profiles) - Semi-hierarchical “profiles”, profile selection, nesting, and extraction.
- [Crate Feature Flags](#crate-feature-flags) - Feature flags and what they enable.
- [Available Providers](#available-providers) - Table of providers provided by this and other crates.
- [For `Provider` Authors](#for-provider-authors) - Tips for writing [`Provider`](https://docs.rs/figment/latest/figment/trait.Provider.html "trait figment::Provider") s.
- [For Library Authors](#for-library-authors) - Brief guide for authors wishing to use Figment in their libraries or frameworks.
- [For Application Authors](#for-application-authors) - Brief guide for authors of applications that use libraries that use Figment.
- [For CLI Application Authors](#for-cli-application-authors) - Brief guide for authors of applications with a CLI and other configuration sources.
- [Tips](#tips) - Things to remember when working with Figment.
- [Type Index](#modules) - The real rustdocs.

## Overview

Figment is a library for declaring and combining configuration sources and extracting typed values from the combined sources. It distinguishes itself from other libraries with similar motives by seamlessly and comprehensively tracking configuration value provenance, even in the face of myriad sources. This means that error values and messages are precise and know exactly where and how misconfiguration arose.

There are two prevailing concepts:

- **Providers:** Types implementing the [`Provider`](https://docs.rs/figment/latest/figment/trait.Provider.html "trait figment::Provider") trait, which implement a configuration source.
- **Figments:** The [`Figment`](https://docs.rs/figment/latest/figment/struct.Figment.html "struct figment::Figment") type, which combines providers via [`merge`](https://docs.rs/figment/latest/figment/struct.Figment.html#method.merge "method figment::Figment::merge") or [`join`](https://docs.rs/figment/latest/figment/struct.Figment.html#method.join "method figment::Figment::join") and allows typed [`extraction`](https://docs.rs/figment/latest/figment/struct.Figment.html#method.extract "method figment::Figment::extract"). Figments are also providers themselves.

Defining a configuration consists of constructing a `Figment` and merging or joining any number of [`Provider`](https://docs.rs/figment/latest/figment/trait.Provider.html "trait figment::Provider") s. Values for duplicate keys from a *merged* provider replace those from previous providers, while no replacement occurs for *joined* providers. Sources are read eagerly, immediately upon merging and joining.

The simplest useful figment has one provider. The figment below will use all environment variables prefixed with `MY_APP_` as configuration values, after removing the prefix:

```rust
use figment::{Figment, providers::Env};

let figment = Figment::from(Env::prefixed("MY_APP_"));
```

Most figments will use more than one provider, merging and joining as necessary. The figment below reads `App.toml`, environment variables prefixed with `APP_` and fills any holes (but does not replace existing values) with values from `App.json`:

```rust
use figment::{Figment, providers::{Format, Toml, Json, Env}};

let figment = Figment::new()
    .merge(Toml::file("App.toml"))
    .merge(Env::prefixed("APP_"))
    .join(Json::file("App.json"));
```

Values can be [`extracted`](https://docs.rs/figment/latest/figment/struct.Figment.html#method.extract "method figment::Figment::extract") into any value that implements [`Deserialize`](https://docs.rs/serde_core/1.0.228/x86_64-unknown-linux-gnu/serde_core/de/trait.Deserialize.html "trait serde_core::de::Deserialize"). The [`Jail`](https://docs.rs/figment/latest/figment/struct.Jail.html "struct figment::Jail") type allows for semi-sandboxed configuration testing. The example below showcases extraction and testing:

```rust
use serde::Deserialize;
use figment::{Figment, providers::{Format, Toml, Json, Env}};

#[derive(Debug, PartialEq, Deserialize)]
struct AppConfig {
    name: String,
    count: usize,
    authors: Vec<String>,
}

figment::Jail::expect_with(|jail| {
    jail.create_file("App.toml", r#"
        name = "Just a TOML App!"
        count = 100
    "#)?;

    jail.create_file("App.json", r#"
        {
            "name": "Just a JSON App",
            "authors": ["figment", "developers"]
        }
    "#)?;

    jail.set_env("APP_COUNT", 250);

    // Sources are read _eagerly_: sources are read as soon as they are
    // merged/joined into a figment.
    let figment = Figment::new()
        .merge(Toml::file("App.toml"))
        .merge(Env::prefixed("APP_"))
        .join(Json::file("App.json"));

    let config: AppConfig = figment.extract()?;
    assert_eq!(config, AppConfig {
        name: "Just a TOML App!".into(),
        count: 250,
        authors: vec!["figment".into(), "developers".into()],
    });

    Ok(())
});
```

## Metadata

Figment takes *great* care to propagate as much information as possible about configuration sources. All values extracted from a figment are [tagged](https://docs.rs/figment/latest/figment/value/struct.Tag.html "struct figment::value::Tag") with the originating [`Metadata`](https://docs.rs/figment/latest/figment/struct.Metadata.html "struct figment::Metadata") and [`Profile`](https://docs.rs/figment/latest/figment/struct.Profile.html "struct figment::Profile"). The tag is preserved across merges, joins, and errors, which also include the [`path`](https://docs.rs/figment/latest/figment/struct.Error.html#structfield.path "field figment::Error::path") of the offending key. Precise tracking allows for rich error messages as well as [“magic”](https://docs.rs/figment/latest/figment/value/magic/index.html "mod figment::value::magic") values like [`RelativePathBuf`](https://docs.rs/figment/latest/figment/value/magic/struct.RelativePathBuf.html "struct figment::value::magic::RelativePathBuf"), which automatically creates a path relative to the configuration file in which it was declared.

A [`Metadata`](https://docs.rs/figment/latest/figment/struct.Metadata.html "struct figment::Metadata") consists of:

- The name of the configuration source.
- An [“interpolater”](https://docs.rs/figment/latest/figment/struct.Metadata.html#method.interpolate "method figment::Metadata::interpolate") that takes a path to a key and converts it into a provider-native key.
- A [`Source`](https://docs.rs/figment/latest/figment/enum.Source.html "enum figment::Source") specifying where the value was sourced from.
- A code source [`Location`](https://doc.rust-lang.org/nightly/core/panic/location/struct.Location.html "struct core::panic::location::Location") where the value’s provider was added to a [`Figment`](https://docs.rs/figment/latest/figment/struct.Figment.html "struct figment::Figment").

Along with the information in an [`Error`](https://docs.rs/figment/latest/figment/struct.Error.html "struct figment::Error"), this means figment can produce rich error values and messages:

```
error: invalid type: found string "hi", expected u16
 --> key \`debug.port\` in TOML file App.toml
```

## Extracting and Profiles

Providers *always* [produce](https://docs.rs/figment/latest/figment/trait.Provider.html#tymethod.data "method figment::Provider::data") [`Dict`](https://docs.rs/figment/latest/figment/value/type.Dict.html "type figment::value::Dict") s nested in [`Profile`](https://docs.rs/figment/latest/figment/struct.Profile.html "struct figment::Profile") s. A profile is [`selected`](https://docs.rs/figment/latest/figment/struct.Figment.html#method.select "method figment::Figment::select") when extracting, and the dictionary corresponding to that profile is deserialized into the requested type. If no profile is selected, the [`Default`](https://docs.rs/figment/latest/figment/struct.Profile.html#associatedconstant.Default "associated constant figment::Profile::Default") profile is used.

There are two built-in profiles: the aforementioned default profile and the [`Global`](https://docs.rs/figment/latest/figment/struct.Profile.html#associatedconstant.Global "associated constant figment::Profile::Global") profile. As the name implies, the default profile contains default values for all profiles. The global profile *also* contains values that correspond to all profiles, but those values supersede values of any other profile *except* the global profile, even when another source is merged.

Some providers can be configured as `nested`, which allows top-level keys in dictionaries produced by the source to be treated as profiles. The following example showcases profiles and nesting:

```rust
use serde::Deserialize;
use figment::{Figment, providers::{Format, Toml, Json, Env}};

#[derive(Debug, PartialEq, Deserialize)]
struct Config {
    name: String,
}

impl Config {
    // Note the \`nested\` option on both \`file\` providers. This makes each
    // top-level dictionary act as a profile.
    fn figment() -> Figment {
        Figment::new()
            .merge(Toml::file("Base.toml").nested())
            .merge(Toml::file("App.toml").nested())
    }
}

figment::Jail::expect_with(|jail| {
    jail.create_file("Base.toml", r#"
        [default]
        name = "Base-Default"

        [debug]
        name = "Base-Debug"
    "#)?;

    // The default profile is used...by default.
    let config: Config = Config::figment().extract()?;
    assert_eq!(config, Config { name: "Base-Default".into(), });

    // A different profile can be selected with \`select\`.
    let config: Config = Config::figment().select("debug").extract()?;
    assert_eq!(config, Config { name: "Base-Debug".into(), });

    // Selecting non-existent profiles is okay as long as we have defaults.
    let config: Config = Config::figment().select("undefined").extract()?;
    assert_eq!(config, Config { name: "Base-Default".into(), });

    // Replace the previous \`Base.toml\`. This one has a \`global\` profile.
    jail.create_file("Base.toml", r#"
        [default]
        name = "Base-Default"

        [debug]
        name = "Base-Debug"

        [global]
        name = "Base-Global"
    "#)?;

    // Global values override all profile values.
    let config_def: Config = Config::figment().extract()?;
    let config_deb: Config = Config::figment().select("debug").extract()?;
    assert_eq!(config_def, Config { name: "Base-Global".into(), });
    assert_eq!(config_deb, Config { name: "Base-Global".into(), });

    // Merges from succeeding providers take precedence, even for globals.
    jail.create_file("App.toml", r#"
        [debug]
        name = "App-Debug"

        [global]
        name = "App-Global"
    "#)?;

    let config_def: Config = Config::figment().extract()?;
    let config_deb: Config = Config::figment().select("debug").extract()?;
    assert_eq!(config_def, Config { name: "App-Global".into(), });
    assert_eq!(config_deb, Config { name: "App-Global".into(), });

    Ok(())
});
```

## Crate Feature Flags

To help with compilation times, types, modules, and providers are gated by features. They are:

| feature | gated namespace | description |
| --- | --- | --- |
| `test` | [`Jail`](https://docs.rs/figment/latest/figment/struct.Jail.html "struct figment::Jail") | Semi-sandboxed environment for testing. |
| `env` | [`providers::Env`](https://docs.rs/figment/latest/figment/providers/struct.Env.html "struct figment::providers::Env") | Environment variable [`Provider`](https://docs.rs/figment/latest/figment/trait.Provider.html "trait figment::Provider"). |
| `toml` | [`providers::Toml`](https://docs.rs/figment/latest/figment/providers/struct.Toml.html "struct figment::providers::Toml") | TOML file/string [`Provider`](https://docs.rs/figment/latest/figment/trait.Provider.html "trait figment::Provider"). |
| `json` | [`providers::Json`](https://docs.rs/figment/latest/figment/providers/struct.Json.html "struct figment::providers::Json") | JSON file/string [`Provider`](https://docs.rs/figment/latest/figment/trait.Provider.html "trait figment::Provider"). |
| `yaml` | [`providers::Yaml`](https://docs.rs/figment/latest/figment/providers/struct.Yaml.html "struct figment::providers::Yaml") | YAML file/string [`Provider`](https://docs.rs/figment/latest/figment/trait.Provider.html "trait figment::Provider"). |
| `yaml` | [`providers::YamlExtended`](https://docs.rs/figment/latest/figment/providers/struct.YamlExtended.html "struct figment::providers::YamlExtended") | [YAML Extended](https://docs.rs/figment/latest/figment/providers/struct.YamlExtended.html#method.from_str "associated function figment::providers::YamlExtended::from_str") file/string [`Provider`](https://docs.rs/figment/latest/figment/trait.Provider.html "trait figment::Provider"). |

## Available Providers

In addition to the four gated providers above, figment provides the following providers out-of-the-box:

| provider | description |
| --- | --- |
| [`providers::Serialized`](https://docs.rs/figment/latest/figment/providers/struct.Serialized.html "struct figment::providers::Serialized") | Source from any [`Serialize`](https://docs.rs/serde_core/1.0.228/x86_64-unknown-linux-gnu/serde_core/ser/trait.Serialize.html "trait serde_core::ser::Serialize") type. |
| [`(impl AsRef<str>, impl Serialize)`](https://docs.rs/figment/latest/figment/trait.Provider.html#impl-Provider-for-\(K%2C%20V\) "trait figment::Provider") | Global source from a `("key", value)`. |
| [`&T` *where* `T: Provider`](https://docs.rs/figment/latest/figment/trait.Provider.html#impl-Provider-for-%26%27_%20T "trait figment::Provider") | Source from `T` as a reference. |

Note: `key` in `(key, value)` is a *key path*, e.g. `"a"` or `"a.b.c"`, where the latter indicates a nested value `c` in `b` in `a`.

See [`Figment`](https://docs.rs/figment/latest/figment/struct.Figment.html#extraction "struct figment::Figment") and [Data (keyed)](https://docs.rs/figment/latest/figment/providers/struct.Serialized.html#provider-details "struct figment::providers::Serialized") for key path details.

#### Third-Party Providers

The following external libraries implement Figment providers:

- [`figment_file_provider_adapter`](https://crates.io/crates/figment_file_provider_adapter)
	Wraps existing providers. For any key ending in `_FILE` (configurable), emits a key without the `_FILE` suffix with a value corresponding to the contents of the file whose path is the original key’s value.

## For Provider Authors

The [`Provider`](https://docs.rs/figment/latest/figment/trait.Provider.html "trait figment::Provider") trait documentation details extensively how to implement a provider for Figment. For data format based providers, the [`Format`](https://docs.rs/figment/latest/figment/providers/trait.Format.html "trait figment::providers::Format") trait allows for even simpler implementations.

## For Library Authors

For libraries and frameworks that wish to expose customizable configuration, we encourage the following structure:

```rust
use serde::{Serialize, Deserialize};

use figment::{Figment, Provider, Error, Metadata, Profile};

// The library's required configuration.
#[derive(Debug, Deserialize, Serialize)]
struct Config { /* the library's required/expected values */ }

// The default configuration.
impl Default for Config {
    fn default() -> Self {
        Config { /* default values */ }
    }
}

impl Config {
    // Allow the configuration to be extracted from any \`Provider\`.
    fn from<T: Provider>(provider: T) -> Result<Config, Error> {
        Figment::from(provider).extract()
    }

    // Provide a default provider, a \`Figment\`.
    fn figment() -> Figment {
        use figment::providers::Env;

        // In reality, whatever the library desires.
        Figment::from(Config::default()).merge(Env::prefixed("APP_"))
    }
}

use figment::value::{Map, Dict};

// Make \`Config\` a provider itself for composability.
impl Provider for Config {
    fn metadata(&self) -> Metadata {
        Metadata::named("Library Config")
    }

    fn data(&self) -> Result<Map<Profile, Dict>, Error>  {
        figment::providers::Serialized::defaults(Config::default()).data()
    }

    fn profile(&self) -> Option<Profile> {
        // Optionally, a profile that's selected by default.
    }
}
```

This structure has the following properties:

- The library provides a `Config` structure that clearly indicates which values the library requires.
- Users can completely customize configuration via their own [`Provider`](https://docs.rs/figment/latest/figment/trait.Provider.html "trait figment::Provider").
- The library’s `Config` is itself a [`Provider`](https://docs.rs/figment/latest/figment/trait.Provider.html "trait figment::Provider") for composability.
- The library provides a `Figment` which it will use as the default configuration provider.

`Config::from(Config::figment())` can be used as the library default while allowing complete customization of the configuration sources. Developers building on the library can base their figments on `Config::default()`, `Config::figment()`, both or neither.

For frameworks, a top-level structure should expose the `Figment` that was used to extract the `Config`, allowing other libraries making use of the framework to also extract values from the same `Figment`:

```rust
use figment::{Figment, Provider, Error};

struct App {
    /// The configuration.
    pub config: Config,
    /// The figment used to extract the configuration.
    pub figment: Figment,
}

impl App {
    pub fn new() -> Result<App, Error> {
        App::custom(Config::figment())
    }

    pub fn custom<T: Provider>(provider: T) -> Result<App, Error> {
        let figment = Figment::from(provider);
        Ok(App { config: Config::from(&figment)?, figment })
    }
}
```

## For Application Authors

As an application author, you’ll need to make at least the following decisions:

1. The sources you’ll accept configuration from.
2. The precedence you’ll apply to each source.
3. Whether you’ll use profiles or not.

For special sources, you may find yourself needing to implement a custom [`Provider`](https://docs.rs/figment/latest/figment/trait.Provider.html "trait figment::Provider"). As with libraries, you’ll likely want to provide default values where possible either by providing it to the figment or by using [serde’s defaults](https://serde.rs/attr-default.html). Then, it’s simply a matter of declaring a figment and extracting the configuration from it.

A reasonable starting point might be:

```rust
use serde::{Serialize, Deserialize};
use figment::{Figment, providers::{Env, Format, Toml, Serialized}};

#[derive(Deserialize, Serialize)]
struct Config {
    key: String,
    another: u32
}

impl Default for Config {
    fn default() -> Config {
        Config {
            key: "default".into(),
            another: 100,
        }
    }
}

Figment::from(Serialized::defaults(Config::default()))
    .merge(Toml::file("App.toml"))
    .merge(Env::prefixed("APP_"));
```

## For CLI Application Authors

As an author of an application with a CLI, you may want to use Figment in combination with a library like [`clap`](https://docs.rs/clap/latest/clap/) if:

- You want to read configuration from sources outside of the CLI.
- You want flexibility in how configuration sources are combined.
- You want great error messages irrespective of how the application is configured.

If any of these conditions apply, Figment is a great choice.

If you are already using a library like [`clap`](https://docs.rs/clap/latest/clap/), you’ll likely have a configuration structure defined:

```rust
use clap::Parser;

#[derive(Parser, Debug)]
struct Config {
   /// Name of the person to greet.
   #[clap(short, long, value_parser)]
   name: String,

   /// Number of times to greet
   #[clap(short, long, value_parser, default_value_t = 1)]
   count: u8,
}
```

To enable the structure to be combined with other Figment sources, derive `Serialize` and `Deserialize` for the structure:

```
+ use serde::{Serialize, Deserialize};

- #[derive(Parser, Debug)]
+ #[derive(Parser, Debug, Serialize, Deserialize)]
struct Config {
```

It can then be combined with other sources via the [`Serialized`](https://docs.rs/figment/latest/figment/providers/struct.Serialized.html "struct figment::providers::Serialized") provider:

```rust
use clap::Parser;
use figment::{Figment, providers::{Serialized, Toml, Env, Format}};
use serde::{Serialize, Deserialize};

#[derive(Parser, Debug, Serialize, Deserialize)]
struct Config {
    // ...
}

// Parse CLI arguments. Override CLI config values with those in
// \`Config.toml\` and \`APP_\`-prefixed environment variables.
let config: Config = Figment::new()
    .merge(Serialized::defaults(Config::parse()))
    .merge(Toml::file("Config.toml"))
    .merge(Env::prefixed("APP_"))
    .extract()?;
```

See [For Application Authors](#for-application-authors) for further, general guidance on using Figment for application configuration.

## Tips

Some things to remember when working with Figment:

- Merging and joining are *eager*: sources are read immediately. It’s useful to define a function that returns a `Figment`.
- The [`util`](https://docs.rs/figment/latest/figment/util/index.html "mod figment::util") modules contains helpful serialize and deserialize implementations for defining `Config` structures.
- The [`Format`](https://docs.rs/figment/latest/figment/providers/trait.Format.html "trait figment::providers::Format") trait makes implementing data-format based [`Provider`](https://docs.rs/figment/latest/figment/trait.Provider.html "trait figment::Provider") s straight-forward.
- [`Magic`](https://docs.rs/figment/latest/figment/value/magic/index.html "mod figment::value::magic") values can significantly reduce the need to inspect a `Figment` directly.
- [`Jail`](https://docs.rs/figment/latest/figment/struct.Jail.html "struct figment::Jail") makes testing configurations straight-forward and much less error-prone.
- [`Error`](https://docs.rs/figment/latest/figment/struct.Error.html "struct figment::Error") may contain more than one error: iterate over it to retrieve all errors.
- Using `#[serde(flatten)]` [can break error attribution](https://github.com/SergioBenitez/Figment/issues/80#issuecomment-1701946622), so it’s best to avoid using it when possible.

## Modules

[error](https://docs.rs/figment/latest/figment/error/index.html "mod figment::error")

Error values produces when extracting configurations.

[providers](https://docs.rs/figment/latest/figment/providers/index.html "mod figment::providers")

Built-in [`Provider`](https://docs.rs/figment/latest/figment/trait.Provider.html "trait figment::Provider") implementations for common sources.

[util](https://docs.rs/figment/latest/figment/util/index.html "mod figment::util")

Useful functions and macros for writing figments.

[value](https://docs.rs/figment/latest/figment/value/index.html "mod figment::value")

[`Value`](https://docs.rs/figment/latest/figment/value/enum.Value.html "enum figment::value::Value") and friends: types representing valid configuration values.

## Structs

[Error](https://docs.rs/figment/latest/figment/struct.Error.html "struct figment::Error")

An error that occured while producing data or extracting a configuration.

[Figment](https://docs.rs/figment/latest/figment/struct.Figment.html "struct figment::Figment")

Combiner of [`Provider`](https://docs.rs/figment/latest/figment/trait.Provider.html "trait figment::Provider") s for configuration value extraction.

[Jail](https://docs.rs/figment/latest/figment/struct.Jail.html "struct figment::Jail") `test`

A “sandboxed” environment with isolated env and file system namespace.

[Metadata](https://docs.rs/figment/latest/figment/struct.Metadata.html "struct figment::Metadata")

Metadata about a configuration value: its source’s name and location.

[Profile](https://docs.rs/figment/latest/figment/struct.Profile.html "struct figment::Profile")

A configuration profile: effectively a case-insensitive string.

## Enums

[Source](https://docs.rs/figment/latest/figment/enum.Source.html "enum figment::Source")

The source for a configuration value.

## Traits

[Provider](https://docs.rs/figment/latest/figment/trait.Provider.html "trait figment::Provider")

Trait implemented by configuration source providers.

## Type Aliases

[Result](https://docs.rs/figment/latest/figment/type.Result.html "type figment::Result")

A simple alias to `Result` with an error type of [`Error`](https://docs.rs/figment/latest/figment/struct.Error.html "struct figment::Error").
