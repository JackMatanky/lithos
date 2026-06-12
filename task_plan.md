# Config Path Modules Refactor Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use `subagent-driven-development` or `executing-plans` to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Replace non-raw `Paths` domain aggregates with focused cache, template, and schema config modules while preserving the raw config file shape for a follow-up refactor.

**Architecture:** `RawPathsConfig`, `RawVaultPaths`, and `RawGlobalPaths` remain as serde DTOs for this change only. Resolved config moves from `Config { paths: Paths }` to `Config { cache: CacheConfig, template: TemplateConfig, schema: SchemaConfig }`, and downstream consumers use narrowed config specs. The old `config::paths` domain API is intentionally removed without compatibility aliases.

**Tech Stack:** Rust, rkyv, serde, crate-local value objects (`RelativeDirPath`, `RelativeFilePath`, `FileName`, `DirPath`, `PathKey`), existing `mise` tasks (`test:unit`, `test`, `fmt`, `lint`).

---

## Scope

- Create `lithos-core/src/config/cache.rs` with `CacheConfig`, `CacheDir`, and `CacheConfigSpec`.
- Create `lithos-core/src/config/template.rs` with `TemplateConfig`, `TemplateDir`, and existing `TemplateConfigSpec` moved from `paths.rs`.
- Create `lithos-core/src/config/schema.rs` with `SchemaConfig`, `SchemaDir`, `PropertyBankFile`, and existing `SchemaConfigSpec` moved from `paths.rs`.
- Remove non-raw `Paths` structs from `config::paths`, `config::global`, and `config::vault`.
- Update `Config` to store private `cache`, `template`, and `schema` fields.
- Update imports from `config::paths::*` to `config::{cache, template, schema}` modules.
- Keep `RawPathsConfig`, `RawVaultPaths`, `RawGlobalPaths`, and `schema/config.schema.json` unchanged in this refactor. A follow-up refactor updates those raw/schema inputs.

## Standards For Every Test

- Unit tests live in the same file as the implementation under `#[cfg(test)] mod tests`.
- Use Structure A from `docs/engineering/testing/unit-naming.md` for files with multiple concerns.
- Use canonical submodule names: `fixtures`, `defaults`, `constructor`, `validation`, `accessors`, `conversions`.
- Use verb-first test names such as `returns_default_cache_dir()` and `rejects_absolute_path()`.
- Arrange/Act/Assert discipline: `expect` is acceptable only in Arrange; Act captures `Result`; Assert verifies explicitly.
- Assertions include diagnostic context messages.
- Tests verify behavior through public or module-local APIs, not implementation details.
- No production `unwrap()` or `expect()` is introduced.

## File Map

- Create: `lithos-core/src/config/cache.rs`
  - Owns cache resolved config, cache directory value object, and cache-facing config spec.
- Create: `lithos-core/src/config/template.rs`
  - Owns template resolved config, template directory value object, and template-facing config spec.
- Create: `lithos-core/src/config/schema.rs`
  - Owns schema resolved config, schema directory value object, property bank file value object, and schema-facing config spec.
- Modify: `lithos-core/src/config/mod.rs`
  - Publishes new `cache`, `template`, and `schema` modules; removes or stops publishing `paths` when empty.
- Modify: `lithos-core/src/config/aggregate.rs`
  - Replaces private `paths: Paths` with private split fields and updates spec projection methods.
- Modify: `lithos-core/src/config/builder.rs`
  - Keeps raw path merge but constructs split resolved config types instead of `Paths`.
- Modify: `lithos-core/src/config/global.rs`
  - Removes non-raw `global::Paths`; stores explicit optional `template` and `schema` config overrides.
- Modify: `lithos-core/src/config/vault.rs`
  - Removes non-raw `vault::Paths`; stores explicit optional `cache`, `template`, and `schema` config overrides.
- Modify: `lithos-core/src/schema/discovery.rs`
  - Imports `SchemaConfigSpec` from `crate::config::schema`.
- Modify or delete: `lithos-core/src/config/paths.rs`
  - Move all surviving domain/spec types out. Delete the file if no symbols remain and no module needs it.
- Do not modify: `lithos-core/src/config/raw.rs`
- Do not modify: `schema/config.schema.json`

## Task 1: Create Cache Config Module

**Files:**
- Create: `lithos-core/src/config/cache.rs`
- Modify: `lithos-core/src/config/mod.rs`

- [ ] **Step 1: Write failing tests for `CacheDir` defaults, validation, accessors, and conversions**

Add this test suite to `lithos-core/src/config/cache.rs` before implementation:

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use crate::fs::path::RelativeDirPath;

    mod defaults {
        use super::*;

        #[test]
        fn returns_default_cache_dir() {
            let cache_dir = CacheDir::default();

            assert_eq!(
                cache_dir.as_relative_dir().as_str(),
                ".cache",
                "default cache dir should match the documented default"
            );
        }
    }

    mod constructor {
        use super::*;

        #[test]
        fn returns_cache_dir_when_relative_path_is_valid() {
            let result = CacheDir::try_new(std::path::Path::new(".lithos-cache"));

            assert!(
                result.is_ok(),
                "valid relative cache path should construct successfully: {:?}",
                result.err()
            );
            assert_eq!(
                result
                    .expect("result checked as ok")
                    .as_relative_dir()
                    .as_str(),
                ".lithos-cache",
                "cache dir should preserve the validated relative declaration"
            );
        }
    }

    mod validation {
        use super::*;

        #[test]
        fn rejects_empty_path() {
            let result = CacheDir::try_new(std::path::Path::new(""));

            assert!(result.is_err(), "empty cache path should be rejected");
        }

        #[test]
        fn rejects_absolute_path() {
            let result = CacheDir::try_new(std::path::Path::new("/tmp/cache"));

            assert!(result.is_err(), "absolute cache path should be rejected");
        }

        #[test]
        fn rejects_parent_traversal() {
            let result = CacheDir::try_new(std::path::Path::new("../cache"));

            assert!(
                result.is_err(),
                "cache path escaping the vault root should be rejected"
            );
        }
    }

    mod conversions {
        use super::*;

        #[test]
        fn returns_cache_dir_from_relative_dir_path() {
            let relative = RelativeDirPath::try_new("cache")
                .expect("fixture relative path should be valid");

            let cache_dir = CacheDir::new(relative);

            assert_eq!(
                cache_dir.as_relative_dir().as_str(),
                "cache",
                "constructor should retain the validated relative dir"
            );
        }
    }
}
```

- [ ] **Step 2: Run the failing cache tests**

Run: `cargo test -p lithos-core config::cache --lib`

Expected: FAIL because `config::cache`, `CacheDir`, and related methods do not exist yet.

- [ ] **Step 3: Implement minimal cache module API**

Implement only enough to satisfy Step 1:

```rust
//! Cache configuration types.

use rkyv::{Archive, Deserialize, Serialize};

use super::error::ConfigError;
use crate::fs::path::RelativeDirPath;

/// Declarative cache directory configuration.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Archive, Serialize, Deserialize)]
#[rkyv(compare(PartialEq), derive(Debug))]
pub struct CacheDir(RelativeDirPath);

impl CacheDir {
    #[inline]
    #[must_use]
    pub const fn new(cache_dir: RelativeDirPath) -> Self {
        Self(cache_dir)
    }

    #[inline]
    pub fn try_new(path: &std::path::Path) -> Result<Self, ConfigError> {
        let value = path.to_str().ok_or_else(|| ConfigError::ValidationFailed {
            field: "cache_dir".into(),
            message: "Non-UTF-8 path".into(),
        })?;

        RelativeDirPath::try_new(value)
            .map(Self)
            .map_err(|error| ConfigError::ValidationFailed {
                field: "cache_dir".into(),
                message: error.to_string().into(),
            })
    }

    #[inline]
    #[must_use]
    pub const fn as_relative_dir(&self) -> &RelativeDirPath {
        &self.0
    }
}

impl Default for CacheDir {
    #[inline]
    #[expect(
        clippy::expect_used,
        reason = "Default directory literal is guaranteed valid"
    )]
    fn default() -> Self {
        Self(RelativeDirPath::try_new(".cache").expect("default path literal must be valid"))
    }
}
```

Also add `pub mod cache;` to `lithos-core/src/config/mod.rs`.

- [ ] **Step 4: Run the cache tests again**

Run: `cargo test -p lithos-core config::cache --lib`

Expected: PASS for the `CacheDir` tests.

- [ ] **Step 5: Write failing tests for `CacheConfig` and `CacheConfigSpec`**

Append these modules to `lithos-core/src/config/cache.rs` tests:

```rust
mod cache_config {
    use super::*;

    #[test]
    fn returns_default_cache_dir() {
        let config = CacheConfig::default();

        assert_eq!(
            config.cache_dir().as_relative_dir().as_str(),
            ".cache",
            "cache config should expose the default cache directory"
        );
    }

    #[test]
    fn returns_configured_cache_dir() {
        let cache_dir = CacheDir::try_new(std::path::Path::new("custom-cache"))
            .expect("fixture cache dir should be valid");

        let config = CacheConfig::new(cache_dir);

        assert_eq!(
            config.cache_dir().as_relative_dir().as_str(),
            "custom-cache",
            "cache config should retain the configured cache directory"
        );
    }
}

mod cache_config_spec {
    use super::*;
    use crate::fs::DirPath;

    #[test]
    fn returns_relative_dir_without_requiring_target_to_exist() {
        let root = tempfile::tempdir().expect("temp dir should be created");
        let root = DirPath::try_from(root.path().to_path_buf())
            .expect("temp root should be valid");
        let directory = RelativeDirPath::try_new(".cache")
            .expect("fixture relative path should be valid");

        let spec = CacheConfigSpec::new(root, directory);

        assert_eq!(
            spec.as_relative_dir().as_str(),
            ".cache",
            "cache spec should retain declarative relative directory"
        );
    }

    #[test]
    fn returns_dir_path_when_root_and_relative_dir_are_valid() {
        let root = tempfile::tempdir().expect("temp dir should be created");
        let cache_path = root.path().join(".cache");
        std::fs::create_dir_all(&cache_path)
            .expect("cache dir fixture should be created");
        let root = DirPath::try_from(root.path().to_path_buf())
            .expect("temp root should be valid");
        let directory = RelativeDirPath::try_new(".cache")
            .expect("fixture relative path should be valid");

        let spec = CacheConfigSpec::new(root, directory);

        let result = spec.to_dir_path();

        assert!(
            result.is_ok(),
            "existing cache dir should resolve successfully: {:?}",
            result.err()
        );
        assert_eq!(
            result.expect("result checked as ok").as_path(),
            cache_path.as_path(),
            "cache spec should resolve relative directory against vault root"
        );
    }

    #[test]
    fn returns_path_key_when_root_scoped_dir_is_valid() {
        let root = tempfile::tempdir().expect("temp dir should be created");
        let cache_path = root.path().join(".cache");
        std::fs::create_dir_all(&cache_path)
            .expect("cache dir fixture should be created");
        let root = DirPath::try_from(root.path().to_path_buf())
            .expect("temp root should be valid");
        let directory = RelativeDirPath::try_new(".cache")
            .expect("fixture relative path should be valid");

        let spec = CacheConfigSpec::new(root, directory);

        let result = spec.to_path_key();

        assert!(
            result.is_ok(),
            "existing cache dir should convert to path key: {:?}",
            result.err()
        );
        assert_eq!(
            result.expect("result checked as ok").as_str(),
            ".cache",
            "cache spec should return vault-relative path key"
        );
    }
}
```

- [ ] **Step 6: Run the failing cache config/spec tests**

Run: `cargo test -p lithos-core config::cache --lib`

Expected: FAIL because `CacheConfig` and `CacheConfigSpec` do not exist yet.

- [ ] **Step 7: Implement minimal `CacheConfig` and `CacheConfigSpec`**

Add these public types to `lithos-core/src/config/cache.rs`:

```rust
use crate::fs::{DirPath, PathKey};

/// Resolved cache configuration.
#[derive(Debug, Clone, PartialEq, Archive, Serialize, Deserialize)]
#[rkyv(compare(PartialEq), derive(Debug))]
#[non_exhaustive]
pub struct CacheConfig {
    cache_dir: CacheDir,
}

impl CacheConfig {
    #[inline]
    #[must_use]
    pub const fn new(cache_dir: CacheDir) -> Self {
        Self { cache_dir }
    }

    #[inline]
    #[must_use]
    pub const fn cache_dir(&self) -> &CacheDir {
        &self.cache_dir
    }
}

impl Default for CacheConfig {
    #[inline]
    fn default() -> Self {
        Self::new(CacheDir::default())
    }
}

/// Cache configuration specification for cache consumers.
#[derive(Debug, Clone, PartialEq)]
#[non_exhaustive]
pub struct CacheConfigSpec {
    root: DirPath,
    directory: RelativeDirPath,
}

impl CacheConfigSpec {
    #[inline]
    #[must_use]
    pub const fn new(root: DirPath, directory: RelativeDirPath) -> Self {
        Self { root, directory }
    }

    #[inline]
    #[must_use]
    pub const fn root(&self) -> &DirPath {
        &self.root
    }

    #[inline]
    #[must_use]
    pub const fn as_relative_dir(&self) -> &RelativeDirPath {
        &self.directory
    }

    #[inline]
    pub fn to_dir_path(&self) -> Result<DirPath, crate::fs::PathError> {
        self.root.append_dir(&self.directory)
    }

    #[inline]
    pub fn to_path_key(&self) -> Result<PathKey, crate::fs::PathError> {
        self.to_dir_path()?.as_key(self.root())
    }
}
```

- [ ] **Step 8: Run cache module tests**

Run: `cargo test -p lithos-core config::cache --lib`

Expected: PASS.

## Task 2: Create Template Config Module

**Files:**
- Create: `lithos-core/src/config/template.rs`
- Modify: `lithos-core/src/config/mod.rs`
- Modify: `lithos-core/src/config/paths.rs`

- [ ] **Step 1: Write failing tests for `TemplateDir` and `TemplateConfig`**

Create `lithos-core/src/config/template.rs` with tests first:

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use crate::fs::path::RelativeDirPath;

    mod defaults {
        use super::*;

        #[test]
        fn returns_default_template_dir() {
            let template_dir = TemplateDir::default();

            assert_eq!(
                template_dir.as_relative_dir().as_str(),
                "templates",
                "default template dir should match the documented default"
            );
        }
    }

    mod constructor {
        use super::*;

        #[test]
        fn returns_template_dir_when_relative_path_is_valid() {
            let result = TemplateDir::try_new(std::path::Path::new("custom-templates"));

            assert!(
                result.is_ok(),
                "valid relative template path should construct successfully: {:?}",
                result.err()
            );
            assert_eq!(
                result
                    .expect("result checked as ok")
                    .as_relative_dir()
                    .as_str(),
                "custom-templates",
                "template dir should preserve the validated relative declaration"
            );
        }
    }

    mod validation {
        use super::*;

        #[test]
        fn rejects_empty_path() {
            let result = TemplateDir::try_new(std::path::Path::new(""));

            assert!(result.is_err(), "empty template path should be rejected");
        }

        #[test]
        fn rejects_absolute_path() {
            let result = TemplateDir::try_new(std::path::Path::new("/tmp/templates"));

            assert!(result.is_err(), "absolute template path should be rejected");
        }

        #[test]
        fn rejects_parent_traversal() {
            let result = TemplateDir::try_new(std::path::Path::new("../templates"));

            assert!(
                result.is_err(),
                "template path escaping the vault root should be rejected"
            );
        }
    }

    mod template_config {
        use super::*;

        #[test]
        fn returns_default_template_dir() {
            let config = TemplateConfig::default();

            assert_eq!(
                config.template_dir().as_relative_dir().as_str(),
                "templates",
                "template config should expose the default template directory"
            );
        }

        #[test]
        fn returns_configured_template_dir() {
            let template_dir = TemplateDir::try_new(std::path::Path::new("custom-templates"))
                .expect("fixture template dir should be valid");

            let config = TemplateConfig::new(template_dir);

            assert_eq!(
                config.template_dir().as_relative_dir().as_str(),
                "custom-templates",
                "template config should retain the configured template directory"
            );
        }
    }

    mod conversions {
        use super::*;

        #[test]
        fn returns_template_dir_from_relative_dir_path() {
            let relative = RelativeDirPath::try_new("templates")
                .expect("fixture relative path should be valid");

            let template_dir = TemplateDir::new(relative);

            assert_eq!(
                template_dir.as_relative_dir().as_str(),
                "templates",
                "constructor should retain the validated relative dir"
            );
        }
    }
}
```

- [ ] **Step 2: Run the failing template tests**

Run: `cargo test -p lithos-core config::template --lib`

Expected: FAIL because `config::template`, `TemplateDir`, and `TemplateConfig` do not exist yet.

- [ ] **Step 3: Implement minimal template module API and move `TemplateConfigSpec`**

Implement `TemplateDir` with the same validation behavior as the old `paths::Template`: convert `&Path` to UTF-8, validate with `RelativeDirPath::try_new`, and map failures to `ConfigError::ValidationFailed { field: "templates_dir", .. }`. Implement `TemplateConfig` as a private-field wrapper around `TemplateDir`. Move the existing `TemplateConfigSpec` from `paths.rs` into `template.rs` unchanged except for module imports.

The public method names must be:

```rust
impl TemplateDir {
    pub const fn new(templates_dir: RelativeDirPath) -> Self;
    pub fn try_new(path: &std::path::Path) -> Result<Self, ConfigError>;
    pub const fn as_relative_dir(&self) -> &RelativeDirPath;
}

impl TemplateConfig {
    pub const fn new(template_dir: TemplateDir) -> Self;
    pub const fn template_dir(&self) -> &TemplateDir;
}

impl TemplateConfigSpec {
    pub const fn new(root: DirPath, directory: RelativeDirPath) -> Self;
    pub const fn root(&self) -> &DirPath;
    pub const fn as_relative_dir(&self) -> &RelativeDirPath;
    pub fn to_dir_path(&self) -> Result<DirPath, crate::fs::PathError>;
    pub fn to_path_key(&self) -> Result<PathKey, crate::fs::PathError>;
}
```

Also add `pub mod template;` to `lithos-core/src/config/mod.rs`.

- [ ] **Step 4: Move existing `TemplateConfigSpec` tests into `template.rs`**

Move the existing `template_config_spec` tests from `paths.rs` into the `tests` module in `template.rs`. Ensure names remain verb-first and modules remain canonical:

```rust
mod template_config_spec {
    use super::*;
    use crate::fs::DirPath;

    #[test]
    fn returns_relative_dir_without_requiring_target_to_exist() {
        let root = tempfile::tempdir().expect("temp dir should be created");
        let root = DirPath::try_from(root.path().to_path_buf())
            .expect("temp root should be valid");
        let directory = RelativeDirPath::try_new("templates")
            .expect("fixture relative path should be valid");

        let spec = TemplateConfigSpec::new(root, directory);

        assert_eq!(
            spec.as_relative_dir().as_str(),
            "templates",
            "template spec should retain declarative relative directory"
        );
    }

    #[test]
    fn returns_dir_path_when_root_and_relative_dir_are_valid() {
        let root = tempfile::tempdir().expect("temp dir should be created");
        let template_path = root.path().join("templates");
        std::fs::create_dir_all(&template_path)
            .expect("template dir fixture should be created");
        let root = DirPath::try_from(root.path().to_path_buf())
            .expect("temp root should be valid");
        let directory = RelativeDirPath::try_new("templates")
            .expect("fixture relative path should be valid");

        let spec = TemplateConfigSpec::new(root, directory);

        let result = spec.to_dir_path();

        assert!(
            result.is_ok(),
            "existing template dir should resolve successfully: {:?}",
            result.err()
        );
        assert_eq!(
            result.expect("result checked as ok").as_path(),
            template_path.as_path(),
            "template spec should resolve relative directory against vault root"
        );
    }

    #[test]
    fn returns_path_key_when_root_scoped_dir_is_valid() {
        let root = tempfile::tempdir().expect("temp dir should be created");
        let template_path = root.path().join("templates");
        std::fs::create_dir_all(&template_path)
            .expect("template dir fixture should be created");
        let root = DirPath::try_from(root.path().to_path_buf())
            .expect("temp root should be valid");
        let directory = RelativeDirPath::try_new("templates")
            .expect("fixture relative path should be valid");

        let spec = TemplateConfigSpec::new(root, directory);

        let result = spec.to_path_key();

        assert!(
            result.is_ok(),
            "existing template dir should convert to path key: {:?}",
            result.err()
        );
        assert_eq!(
            result.expect("result checked as ok").as_str(),
            "templates",
            "template spec should return vault-relative path key"
        );
    }
}
```

- [ ] **Step 5: Run template module tests**

Run: `cargo test -p lithos-core config::template --lib`

Expected: PASS.

## Task 3: Create Schema Config Module

**Files:**
- Create: `lithos-core/src/config/schema.rs`
- Modify: `lithos-core/src/config/mod.rs`
- Modify: `lithos-core/src/config/paths.rs`

- [ ] **Step 1: Write failing tests for schema value objects and config ownership**

Create `lithos-core/src/config/schema.rs` with tests first:

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use crate::fs::{FileName, path::RelativeDirPath};

    mod defaults {
        use super::*;

        #[test]
        fn returns_default_schema_dir() {
            let schema_dir = SchemaDir::default();

            assert_eq!(
                schema_dir.as_relative_dir().as_str(),
                "schemas",
                "default schema dir should match the documented default"
            );
        }

        #[test]
        fn returns_default_property_bank_file() {
            let property_bank = PropertyBankFile::default();

            assert_eq!(
                property_bank.as_str(),
                "property_bank.json",
                "default property bank file should match the documented default"
            );
        }
    }

    mod validation {
        use super::*;

        #[test]
        fn rejects_empty_schema_dir() {
            let result = SchemaDir::try_new(std::path::Path::new(""));

            assert!(result.is_err(), "empty schema path should be rejected");
        }

        #[test]
        fn rejects_absolute_schema_dir() {
            let result = SchemaDir::try_new(std::path::Path::new("/tmp/schemas"));

            assert!(result.is_err(), "absolute schema path should be rejected");
        }

        #[test]
        fn rejects_parent_traversal_schema_dir() {
            let result = SchemaDir::try_new(std::path::Path::new("../schemas"));

            assert!(
                result.is_err(),
                "schema path escaping the vault root should be rejected"
            );
        }

        #[test]
        fn rejects_property_bank_file_with_path_separator() {
            let result = PropertyBankFile::try_new("schemas/bank.json");

            assert!(
                result.is_err(),
                "property bank file should reject path separators"
            );
        }
    }

    mod schema_config {
        use super::*;

        #[test]
        fn returns_default_schema_dir_and_property_bank_file() {
            let config = SchemaConfig::default();

            assert_eq!(
                config.schema_dir().as_relative_dir().as_str(),
                "schemas",
                "schema config should expose the default schema directory"
            );
            assert_eq!(
                config.property_bank_file().as_str(),
                "property_bank.json",
                "schema config should expose the default property bank file"
            );
        }

        #[test]
        fn returns_property_bank_relative_path_under_schema_dir() {
            let schema_dir = SchemaDir::try_new(std::path::Path::new("custom-schemas"))
                .expect("fixture schema dir should be valid");
            let property_bank = PropertyBankFile::try_new("bank.json")
                .expect("fixture property bank file should be valid");
            let config = SchemaConfig::new(schema_dir, property_bank);

            let result = config.property_bank_relative_path();

            assert_eq!(
                result,
                std::path::PathBuf::from("custom-schemas").join("bank.json"),
                "schema config should derive property bank path under schema dir"
            );
        }
    }

    mod conversions {
        use super::*;

        #[test]
        fn returns_property_bank_file_from_file_name() {
            let file_name = FileName::try_from(std::path::Path::new("bank.json"))
                .expect("fixture filename should be valid");

            let property_bank = PropertyBankFile::from(file_name);

            assert_eq!(
                property_bank.as_str(),
                "bank.json",
                "property bank file should retain the validated file name"
            );
        }

        #[test]
        fn returns_string_from_property_bank_file() {
            let property_bank = PropertyBankFile::try_new("bank.json")
                .expect("fixture property bank file should be valid");

            let value = String::from(property_bank);

            assert_eq!(
                value,
                "bank.json",
                "property bank file should convert back to owned string"
            );
        }
    }
}
```

- [ ] **Step 2: Run the failing schema tests**

Run: `cargo test -p lithos-core config::schema --lib`

Expected: FAIL because `config::schema`, `SchemaDir`, `PropertyBankFile`, and `SchemaConfig` do not exist yet.

- [ ] **Step 3: Implement minimal schema value objects and `SchemaConfig`**

Move the old `paths::Schema` behavior into `SchemaDir`: convert `&Path` to UTF-8, validate with `RelativeDirPath::try_new`, and map failures to `ConfigError::ValidationFailed { field: "schemas_dir", .. }`. Move the old `paths::PropertyBank` behavior into `PropertyBankFile`: validate through `FileName`, expose `as_str()`, and preserve `From<FileName>`, `TryFrom<String>`, `From<PropertyBankFile> for String`, and `Display`. Introduce `SchemaConfig` with this public API:

```rust
impl SchemaDir {
    pub const fn new(schema_dir: RelativeDirPath) -> Self;
    pub fn try_new(path: &std::path::Path) -> Result<Self, ConfigError>;
    pub const fn as_relative_dir(&self) -> &RelativeDirPath;
}

impl PropertyBankFile {
    pub fn try_new<T: Into<Box<str>>>(value: T) -> Result<Self, ConfigError>;
    pub fn as_str(&self) -> &str;
}

impl SchemaConfig {
    pub const fn new(schema_dir: SchemaDir, property_bank_file: PropertyBankFile) -> Self;
    pub const fn schema_dir(&self) -> &SchemaDir;
    pub const fn property_bank_file(&self) -> &PropertyBankFile;
    pub fn property_bank_relative_path(&self) -> std::path::PathBuf;
}
```

Use the same `ConfigError::ValidationFailed` field names as the current code: `schemas_dir` and `property_bank_file`.

Also add `pub mod schema;` to `lithos-core/src/config/mod.rs`.

- [ ] **Step 4: Run schema value/config tests**

Run: `cargo test -p lithos-core config::schema --lib`

Expected: PASS for schema value/config tests, while moved spec tests may still be absent.

- [ ] **Step 5: Move `SchemaConfigSpec` and its tests**

Move existing `SchemaConfigSpec` from `paths.rs` to `schema.rs` unchanged except module imports. Move the existing `schema_config_spec` unit tests with these behavior names:

```rust
mod schema_config_spec {
    use super::*;
    use crate::fs::{DirPath, path::{RelativeDirPath, RelativeFilePath}};

    #[test]
    fn returns_relative_paths_without_requiring_targets_to_exist() {
        let root = tempfile::tempdir().expect("temp dir should be created");
        let root = DirPath::try_from(root.path().to_path_buf())
            .expect("temp root should be valid");
        let directory = RelativeDirPath::try_new("schemas")
            .expect("fixture relative dir should be valid");
        let property_bank = RelativeFilePath::try_new("schemas/bank.json")
            .expect("fixture relative file should be valid");

        let spec = SchemaConfigSpec::new(root, directory, property_bank);

        assert_eq!(
            spec.directory_relative().as_str(),
            "schemas",
            "schema spec should retain schema directory declaration"
        );
        assert_eq!(
            spec.property_bank_relative().as_str(),
            "schemas/bank.json",
            "schema spec should retain property bank declaration"
        );
    }

    #[test]
    fn returns_schema_directory_path_when_root_and_relative_dir_are_valid() {
        let root = tempfile::tempdir().expect("temp dir should be created");
        let schemas_path = root.path().join("schemas");
        std::fs::create_dir_all(&schemas_path)
            .expect("schemas dir fixture should be created");
        let root = DirPath::try_from(root.path().to_path_buf())
            .expect("temp root should be valid");
        let directory = RelativeDirPath::try_new("schemas")
            .expect("fixture relative dir should be valid");
        let property_bank = RelativeFilePath::try_new("schemas/bank.json")
            .expect("fixture relative file should be valid");

        let spec = SchemaConfigSpec::new(root, directory, property_bank);

        let result = spec.schema_directory_path();

        assert!(
            result.is_ok(),
            "existing schema dir should resolve successfully: {:?}",
            result.err()
        );
        assert_eq!(
            result.expect("result checked as ok").as_path(),
            schemas_path.as_path(),
            "schema spec should resolve schema directory against vault root"
        );
    }

    #[test]
    fn returns_property_bank_file_path_when_root_and_relative_file_are_valid() {
        let root = tempfile::tempdir().expect("temp dir should be created");
        let schemas_path = root.path().join("schemas");
        std::fs::create_dir_all(&schemas_path)
            .expect("schemas dir fixture should be created");
        let bank_path = schemas_path.join("bank.json");
        std::fs::write(&bank_path, "{}")
            .expect("property bank fixture should be writable");
        let root = DirPath::try_from(root.path().to_path_buf())
            .expect("temp root should be valid");
        let directory = RelativeDirPath::try_new("schemas")
            .expect("fixture relative dir should be valid");
        let property_bank = RelativeFilePath::try_new("schemas/bank.json")
            .expect("fixture relative file should be valid");

        let spec = SchemaConfigSpec::new(root, directory, property_bank);

        let result = spec.property_bank_file_path();

        assert!(
            result.is_ok(),
            "existing property bank file should resolve successfully: {:?}",
            result.err()
        );
        assert_eq!(
            result.expect("result checked as ok").as_path(),
            bank_path.as_path(),
            "schema spec should resolve property bank file against vault root"
        );
    }

    #[test]
    fn returns_schema_directory_key_when_root_scoped_dir_is_valid() {
        let root = tempfile::tempdir().expect("temp dir should be created");
        let schemas_path = root.path().join("schemas");
        std::fs::create_dir_all(&schemas_path)
            .expect("schemas dir fixture should be created");
        let bank_path = schemas_path.join("bank.json");
        std::fs::write(&bank_path, "{}")
            .expect("property bank fixture should be writable");
        let root = DirPath::try_from(root.path().to_path_buf())
            .expect("temp root should be valid");
        let directory = RelativeDirPath::try_new("schemas")
            .expect("fixture relative dir should be valid");
        let property_bank = RelativeFilePath::try_new("schemas/bank.json")
            .expect("fixture relative file should be valid");

        let spec = SchemaConfigSpec::new(root, directory, property_bank);

        let result = spec.schema_directory_key();

        assert!(
            result.is_ok(),
            "existing schema dir should convert to path key: {:?}",
            result.err()
        );
        assert_eq!(
            result.expect("result checked as ok").as_str(),
            "schemas",
            "schema directory key should be vault-relative"
        );
    }

    #[test]
    fn returns_property_bank_key_when_root_scoped_file_is_valid() {
        let root = tempfile::tempdir().expect("temp dir should be created");
        let schemas_path = root.path().join("schemas");
        std::fs::create_dir_all(&schemas_path)
            .expect("schemas dir fixture should be created");
        let bank_path = schemas_path.join("bank.json");
        std::fs::write(&bank_path, "{}")
            .expect("property bank fixture should be writable");
        let root = DirPath::try_from(root.path().to_path_buf())
            .expect("temp root should be valid");
        let directory = RelativeDirPath::try_new("schemas")
            .expect("fixture relative dir should be valid");
        let property_bank = RelativeFilePath::try_new("schemas/bank.json")
            .expect("fixture relative file should be valid");

        let spec = SchemaConfigSpec::new(root, directory, property_bank);

        let result = spec.property_bank_key();

        assert!(
            result.is_ok(),
            "existing property bank file should convert to path key: {:?}",
            result.err()
        );
        assert_eq!(
            result.expect("result checked as ok").as_str(),
            "schemas/bank.json",
            "property bank key should be vault-relative"
        );
    }
}
```

- [ ] **Step 6: Run schema module tests**

Run: `cargo test -p lithos-core config::schema --lib`

Expected: PASS.

## Task 4: Replace Resolved `Config` Paths Storage

**Files:**
- Modify: `lithos-core/src/config/aggregate.rs`
- Modify: `lithos-core/src/config/builder.rs`

- [ ] **Step 1: Write failing tests for split config accessors and specs**

In `lithos-core/src/config/aggregate.rs`, replace tests that assert `config.paths()` with split behavior tests:

```rust
mod resolved_path_config {
    use super::*;

    #[test]
    fn returns_default_split_path_configs_from_empty_raw() {
        let config = crate::config::builder::build_from_layers(
            None,
            None,
            fixtures::vault_id(),
            fixtures::vault_root("/vault"),
            Version::initial(),
        )
        .expect("empty raw layers should build default config");

        assert_eq!(
            config.cache().cache_dir().as_relative_dir().as_str(),
            ".cache",
            "resolved config should expose default cache config"
        );
        assert_eq!(
            config.template().template_dir().as_relative_dir().as_str(),
            "templates",
            "resolved config should expose default template config"
        );
        assert_eq!(
            config.schema().schema_dir().as_relative_dir().as_str(),
            "schemas",
            "resolved config should expose default schema config"
        );
        assert_eq!(
            config.schema().property_bank_file().as_str(),
            "property_bank.json",
            "resolved config should expose default property bank file"
        );
    }

    #[test]
    fn applies_path_fields_from_raw_to_split_configs() {
        let vault = crate::config::raw::RawVaultConfig {
            paths: crate::config::raw::RawVaultPaths {
                cache_dir: Some(".lithos-cache".to_owned()),
                schemas_dir: Some("my-schemas".to_owned()),
                property_bank_file: Some("bank.json".to_owned()),
                templates_dir: Some("my-templates".to_owned()),
            },
            ..Default::default()
        };
        let config = crate::config::builder::build_from_layers(
            None,
            Some(&vault),
            fixtures::vault_id(),
            fixtures::vault_root("/vault"),
            Version::initial(),
        )
        .expect("raw vault paths should build resolved config");

        assert_eq!(
            config.cache().cache_dir().as_relative_dir().as_str(),
            ".lithos-cache",
            "raw cache_dir should populate CacheConfig"
        );
        assert_eq!(
            config.template().template_dir().as_relative_dir().as_str(),
            "my-templates",
            "raw templates_dir should populate TemplateConfig"
        );
        assert_eq!(
            config.schema().schema_dir().as_relative_dir().as_str(),
            "my-schemas",
            "raw schemas_dir should populate SchemaConfig"
        );
        assert_eq!(
            config.schema().property_bank_file().as_str(),
            "bank.json",
            "raw property_bank_file should populate SchemaConfig"
        );
    }
}
```

- [ ] **Step 2: Run failing aggregate tests**

Run: `cargo test -p lithos-core config::aggregate::tests::resolved_path_config --lib`

Expected: FAIL because `Config::cache()`, `Config::template()`, and `Config::schema()` do not exist and `Config` still stores `paths`.

- [ ] **Step 3: Update `Config` constructor and accessors**

Change `Config` to store:

```rust
cache: CacheConfig,
template: TemplateConfig,
schema: SchemaConfig,
```

Update `Config::new` signature to accept `cache`, `template`, and `schema` instead of `paths`. Add these accessors:

```rust
pub const fn cache(&self) -> &CacheConfig;
pub const fn template(&self) -> &TemplateConfig;
pub const fn schema(&self) -> &SchemaConfig;
```

Remove `Config::paths()` and `ArchivedConfig::paths()`.

- [ ] **Step 4: Update `build_from_layers` to construct split configs**

Keep `RawPathsConfig::merge` unchanged. Replace `Paths::try_from(&paths)?` with validation that creates:

```rust
let cache = CacheConfig::new(cache_dir);
let template = TemplateConfig::new(template_dir);
let schema = SchemaConfig::new(schema_dir, property_bank_file);
```

The validation behavior must keep the current defaults and error fields:

- `cache_dir` default: `.cache`
- `templates_dir` default: `templates`
- `schemas_dir` default: `schemas`
- `property_bank_file` default: `property_bank.json`

- [ ] **Step 5: Run aggregate split config tests**

Run: `cargo test -p lithos-core config::aggregate::tests::resolved_path_config --lib`

Expected: PASS.

## Task 5: Update Schema, Template, and Cache Spec Projection

**Files:**
- Modify: `lithos-core/src/config/aggregate.rs`
- Modify: `lithos-core/src/schema/discovery.rs`

- [ ] **Step 1: Write failing tests for `to_schema_spec`, `to_template_spec`, and `to_cache_spec`**

In `lithos-core/src/config/aggregate.rs`, update/add spec tests:

```rust
mod config_specs {
    use super::*;

    #[test]
    fn to_schema_spec_respects_custom_schema_config() {
        let root = tempfile::tempdir().expect("temp dir should be created");
        let custom_schemas = root.path().join("custom-schemas");
        std::fs::create_dir_all(&custom_schemas)
            .expect("custom schemas dir should be created");
        let custom_bank = custom_schemas.join("custom-bank.json");
        std::fs::write(&custom_bank, "{}")
            .expect("custom property bank should be writable");
        let vault = crate::config::raw::RawVaultConfig {
            paths: crate::config::raw::RawVaultPaths {
                schemas_dir: Some("custom-schemas".to_owned()),
                property_bank_file: Some("custom-bank.json".to_owned()),
                ..Default::default()
            },
            ..Default::default()
        };
        let config = crate::config::builder::build_from_layers(
            None,
            Some(&vault),
            fixtures::vault_id(),
            crate::config::vault::VaultRoot::try_new(root.path().to_path_buf())
                .expect("vault root should be valid"),
            Version::initial(),
        )
        .expect("config should build");

        let spec = config.to_schema_spec();

        assert!(spec.is_ok(), "schema spec should build: {:?}", spec.err());
        let spec = spec.expect("result checked as ok");
        assert_eq!(
            spec.schema_directory_path()
                .expect("schema directory should resolve")
                .as_path(),
            custom_schemas.as_path(),
            "schema spec should use SchemaConfig schema directory"
        );
        assert_eq!(
            spec.property_bank_file_path()
                .expect("property bank file should resolve")
                .as_path(),
            custom_bank.as_path(),
            "schema spec should derive property bank path from SchemaConfig"
        );
    }

    #[test]
    fn to_template_spec_respects_custom_template_config() {
        let root = tempfile::tempdir().expect("temp dir should be created");
        let templates = root.path().join("custom-templates");
        std::fs::create_dir_all(&templates)
            .expect("templates dir should be created");
        let vault = crate::config::raw::RawVaultConfig {
            paths: crate::config::raw::RawVaultPaths {
                templates_dir: Some("custom-templates".to_owned()),
                ..Default::default()
            },
            ..Default::default()
        };
        let config = crate::config::builder::build_from_layers(
            None,
            Some(&vault),
            fixtures::vault_id(),
            crate::config::vault::VaultRoot::try_new(root.path().to_path_buf())
                .expect("vault root should be valid"),
            Version::initial(),
        )
        .expect("config should build");

        let spec = config.to_template_spec();

        assert!(spec.is_ok(), "template spec should build: {:?}", spec.err());
        assert_eq!(
            spec.expect("result checked as ok")
                .to_dir_path()
                .expect("template dir should resolve")
                .as_path(),
            templates.as_path(),
            "template spec should use TemplateConfig directory"
        );
    }

    #[test]
    fn to_cache_spec_respects_custom_cache_config() {
        let root = tempfile::tempdir().expect("temp dir should be created");
        let cache = root.path().join(".lithos-cache");
        std::fs::create_dir_all(&cache).expect("cache dir should be created");
        let vault = crate::config::raw::RawVaultConfig {
            paths: crate::config::raw::RawVaultPaths {
                cache_dir: Some(".lithos-cache".to_owned()),
                ..Default::default()
            },
            ..Default::default()
        };
        let config = crate::config::builder::build_from_layers(
            None,
            Some(&vault),
            fixtures::vault_id(),
            crate::config::vault::VaultRoot::try_new(root.path().to_path_buf())
                .expect("vault root should be valid"),
            Version::initial(),
        )
        .expect("config should build");

        let spec = config.to_cache_spec();

        assert!(spec.is_ok(), "cache spec should build: {:?}", spec.err());
        assert_eq!(
            spec.expect("result checked as ok")
                .to_dir_path()
                .expect("cache dir should resolve")
                .as_path(),
            cache.as_path(),
            "cache spec should use CacheConfig directory"
        );
    }
}
```

- [ ] **Step 2: Run failing spec projection tests**

Run: `cargo test -p lithos-core config::aggregate::tests::config_specs --lib`

Expected: FAIL because imports still point to `config::paths`, `to_cache_spec()` does not exist, and projection methods still read `self.paths`.

- [ ] **Step 3: Update spec projection methods**

Update imports in `aggregate.rs`:

```rust
use super::{
    cache::{CacheConfig, CacheConfigSpec},
    schema::{SchemaConfig, SchemaConfigSpec},
    template::{TemplateConfig, TemplateConfigSpec},
};
```

Update projections:

```rust
pub fn to_schema_spec(&self) -> Result<SchemaConfigSpec, ConfigError>;
pub fn to_template_spec(&self) -> Result<TemplateConfigSpec, ConfigError>;
pub fn to_cache_spec(&self) -> Result<CacheConfigSpec, ConfigError>;
```

Each projection uses `self.vault_metadata.root().as_dir_path().clone()` and the relevant `*Config` relative declarations. `to_schema_spec()` builds the property bank relative file from `self.schema.property_bank_relative_path()`.

- [ ] **Step 4: Update schema discovery import**

In `lithos-core/src/schema/discovery.rs`, change:

```rust
use crate::config::paths::SchemaConfigSpec;
```

to:

```rust
use crate::config::schema::SchemaConfigSpec;
```

- [ ] **Step 5: Run spec projection tests**

Run: `cargo test -p lithos-core config::aggregate::tests::config_specs --lib`

Expected: PASS.

## Task 6: Remove Non-Raw `Paths` From Global and Vault Domain Config

**Files:**
- Modify: `lithos-core/src/config/global.rs`
- Modify: `lithos-core/src/config/vault.rs`
- Modify: `lithos-core/src/config/builder.rs`

- [ ] **Step 1: Write failing global/vault domain tests**

In `global.rs`, replace `Paths`-centric tests with explicit optional field behavior:

```rust
mod path_overrides {
    use super::*;

    #[test]
    fn returns_template_and_schema_overrides_from_raw_paths() {
        let raw = crate::config::raw::RawPathsConfig {
            cache_dir: Some(".ignored".to_owned()),
            templates_dir: Some("global-templates".to_owned()),
            schemas_dir: Some("global-schemas".to_owned()),
            property_bank_file: Some("global-bank.json".to_owned()),
        };

        let global = Global::try_from_paths(&raw)
            .expect("global path overrides should validate");

        assert_eq!(
            global
                .template()
                .expect("template override should exist")
                .template_dir()
                .as_relative_dir()
                .as_str(),
            "global-templates",
            "global template override should be retained"
        );
        assert_eq!(
            global
                .schema()
                .expect("schema override should exist")
                .schema_dir()
                .as_relative_dir()
                .as_str(),
            "global-schemas",
            "global schema override should be retained"
        );
        assert_eq!(
            global
                .schema()
                .expect("schema override should exist")
                .property_bank_file()
                .as_str(),
            "global-bank.json",
            "global property bank override should be retained under schema config"
        );
    }
}
```

In `vault.rs`, replace `Paths`-centric tests with:

```rust
mod path_overrides {
    use super::*;

    #[test]
    fn returns_cache_template_and_schema_overrides_from_raw_paths() {
        let raw = crate::config::raw::RawPathsConfig {
            cache_dir: Some(".vault-cache".to_owned()),
            templates_dir: Some("vault-templates".to_owned()),
            schemas_dir: Some("vault-schemas".to_owned()),
            property_bank_file: Some("vault-bank.json".to_owned()),
        };

        let vault = Vault::try_from_paths(&raw)
            .expect("vault path overrides should validate");

        assert_eq!(
            vault
                .cache()
                .expect("cache override should exist")
                .cache_dir()
                .as_relative_dir()
                .as_str(),
            ".vault-cache",
            "vault cache override should be retained"
        );
        assert_eq!(
            vault
                .template()
                .expect("template override should exist")
                .template_dir()
                .as_relative_dir()
                .as_str(),
            "vault-templates",
            "vault template override should be retained"
        );
        assert_eq!(
            vault
                .schema()
                .expect("schema override should exist")
                .schema_dir()
                .as_relative_dir()
                .as_str(),
            "vault-schemas",
            "vault schema override should be retained"
        );
    }
}
```

- [ ] **Step 2: Run failing global/vault tests**

Run: `cargo test -p lithos-core config::global::tests::path_overrides config::vault::tests::path_overrides --lib`

Expected: FAIL because `global::Paths`, `vault::Paths`, and `paths::*` still exist and `try_from_paths`/new accessors are not implemented.

- [ ] **Step 3: Replace partial `Paths` structs with explicit optional fields**

In `Global`, replace `paths: Paths` with:

```rust
template: Option<TemplateConfig>,
schema: Option<SchemaConfig>,
```

In `Vault`, replace `paths: Paths` with:

```rust
cache: Option<CacheConfig>,
template: Option<TemplateConfig>,
schema: Option<SchemaConfig>,
```

Add accessors returning these exact borrowed option types:

```rust
impl Global {
    pub fn template(&self) -> Option<&TemplateConfig>;
    pub fn schema(&self) -> Option<&SchemaConfig>;
}

impl Vault {
    pub fn cache(&self) -> Option<&CacheConfig>;
    pub fn template(&self) -> Option<&TemplateConfig>;
    pub fn schema(&self) -> Option<&SchemaConfig>;
}
```

Add conversion helpers that validate from `RawPathsConfig` while raw DTOs remain unchanged:

```rust
impl Global {
    pub fn try_from_paths(raw: &super::raw::RawPathsConfig) -> Result<Self, ConfigError>;
}

impl Vault {
    pub fn try_from_paths(raw: &super::raw::RawPathsConfig) -> Result<Self, ConfigError>;
}
```

Use private validation helpers with these signatures to avoid duplicate parsing logic while keeping the public API narrow:

```rust
fn parse_cache_config(raw: &super::raw::RawPathsConfig) -> Result<Option<CacheConfig>, ConfigError>;
fn parse_template_config(raw: &super::raw::RawPathsConfig) -> Result<Option<TemplateConfig>, ConfigError>;
fn parse_schema_config(raw: &super::raw::RawPathsConfig) -> Result<Option<SchemaConfig>, ConfigError>;
```

- [ ] **Step 4: Run global/vault tests**

Run: `cargo test -p lithos-core config::global::tests::path_overrides config::vault::tests::path_overrides --lib`

Expected: PASS.

## Task 7: Delete or Empty `paths.rs` and Update Imports

**Files:**
- Modify/delete: `lithos-core/src/config/paths.rs`
- Modify: `lithos-core/src/config/mod.rs`
- Modify: all files importing `config::paths` symbols

- [ ] **Step 1: Search for stale path module imports**

Run: `rg "config::paths|paths::\{|crate::config::paths|super::paths" lithos-core/src lithos-core/tests`

Expected before cleanup: matches in `aggregate.rs`, `global.rs`, `vault.rs`, `schema/discovery.rs`, docs or tests.

- [ ] **Step 2: Update imports to new modules**

Replace old imports with:

```rust
use crate::config::cache::{CacheConfig, CacheConfigSpec, CacheDir};
use crate::config::template::{TemplateConfig, TemplateConfigSpec, TemplateDir};
use crate::config::schema::{PropertyBankFile, SchemaConfig, SchemaConfigSpec, SchemaDir};
```

Inside `lithos-core/src/config/*.rs` modules, prefer `super::cache`, `super::template`, and `super::schema` imports instead of `crate::config::*` imports.

- [ ] **Step 3: Remove `paths.rs` module export**

If no symbols remain in `paths.rs`, delete the file and remove `pub mod paths;` from `config/mod.rs`. If rustdoc links still need a transition, replace them with links to `cache`, `template`, and `schema` modules.

- [ ] **Step 4: Verify no stale imports remain**

Run: `rg "config::paths|paths::\{|crate::config::paths|super::paths" lithos-core/src lithos-core/tests`

Expected: no matches.

## Task 8: Update Documentation Examples and Compile-Time API Expectations

**Files:**
- Modify: `lithos-core/src/config/mod.rs`
- Modify: doc examples in changed config modules
- Do not modify: `schema/config.schema.json`

- [ ] **Step 1: Write or update doc examples to use specs/accessors**

In `config/mod.rs`, replace `config.paths().cache.cache_dir()` examples with spec or split config examples:

```rust
//! let cache_spec = config.to_cache_spec()?;
//! assert!(!cache_spec.as_relative_dir().as_str().is_empty());
```

In module docs, use:

```rust
/// use lithos_core::config::cache::CacheConfig;
/// use lithos_core::config::template::TemplateConfig;
/// use lithos_core::config::schema::SchemaConfig;
```

- [ ] **Step 2: Run doc tests for config modules**

Run: `cargo test -p lithos-core --doc config`

Expected: PASS.

## Task 9: Full Verification And Change Impact Review

**Files:**
- No planned source edits in this task unless verification reveals failures.

- [ ] **Step 1: Run formatting**

Run: `mise run fmt`

Expected: PASS; Rust files are formatted.

- [ ] **Step 2: Run unit tests**

Run: `mise run test:unit`

Expected: PASS.

- [ ] **Step 3: Run lint**

Run: `mise run lint`

Expected: PASS with no clippy warnings.

- [ ] **Step 4: Run full test suite if unit/lint are clean**

Run: `mise run test`

Expected: PASS.

- [ ] **Step 5: Run GitNexus change detection**

Run the GitNexus tool: `gitnexus_detect_changes({ scope: "all", repo: "lithos" })`

Expected: changed symbols are limited to config path refactor surfaces and expected schema/template/cache spec consumers.

## Self-Review Checklist

- [ ] Every task follows RED → GREEN → refactor sequencing.
- [ ] Tests use Structure A where files contain multiple units.
- [ ] Test function names are snake_case, verb-first, and behavior-specific.
- [ ] Tests avoid `test_` prefixes and avoid combining unrelated behaviors with `and`.
- [ ] `RawPathsConfig`, `RawVaultPaths`, `RawGlobalPaths`, and `schema/config.schema.json` are not modified in this refactor.
- [ ] `ConfigField::Paths` and raw path hashing remain unchanged unless compilation requires import-only updates.
- [ ] No compatibility aliases are added for `Paths`, `Cache`, `Template`, `Schema`, or `PropertyBank`.
- [ ] No production `unwrap()` or `expect()` is introduced.
- [ ] Public API docs point to `cache`, `template`, and `schema` modules rather than `paths`.
