---
feature: Template Module Migration Strategy
status: Draft
author: Jack (via AI Design Partner)
ticket: Story 3.4 - Template Migration
date_created: 2026-02-16
tags: [template, migration, refactor, strategy, implementation-plan]
---

# Tech Spec: Template Module Migration Strategy

> **Purpose:** Step-by-step implementation plan for migrating from the CURRENT (incorrect) template implementation to the TARGET (correct) MiniJinja-first architecture.

## 1. Problem Space (The "Why")

### 1.1 Context & Background

**Current State (Incorrect Architecture):**
- ❌ Template domain layer performs syntax validation (should be adapter concern)
- ❌ Manual string composition via `apply_sections()` (should use MiniJinja `{% extends %}`)
- ❌ Zero MiniJinja integration (no imports despite dependency being declared)
- ❌ `PlaceholderSyntax` provides false configurability (MiniJinja uses fixed `{{ }}` syntax)
- ❌ `RESERVED_WORDS` hardcoded list (MiniJinja handles this)
- ❌ No template compilation caching (violates project-context.md mandate: "compile exactly once at startup")
- ❌ Variable validation in domain layer (should be MiniJinja filters at render time)

**Target State (Correct Architecture):**
- ✅ Template domain is metadata-only (see [012-template-models.md](./012-template-models.md))
- ✅ MiniJinja handles ALL templating (syntax, inheritance, filters, rendering)
- ✅ Templates compiled once at startup, cached in `Arc<Environment>`
- ✅ Composition via native `{% extends %}` and `{% block %}` (not manual string manipulation)
- ✅ Variable constraints enforced via MiniJinja filters (render-time validation)
- ✅ Port-based CQRS with GAT zero-copy reads (see [013-template-cqrs.md](./013-template-cqrs.md))

**Migration Challenge:**
We cannot break existing functionality while refactoring. This plan provides an incremental path that maintains a working system at each step, with clear rollback points and verification gates.

**References:**
- [012: Template Domain Models](./012-template-models.md) - Target domain model (metadata schema)
- [013: Template CQRS](./013-template-cqrs.md) - Target storage layer (ports and adapters)
- [014: Template Services](./014-template-services.md) - Target service layer (MiniJinja integration)

### 1.2 Goals & Non-Goals

**Goals:**
1. ✅ **Incremental Migration:** Each phase produces working, tested code (tests pass at each commit)
2. ✅ **No Data Loss:** Existing templates remain valid throughout migration (rkyv format compatible)
3. ✅ **Clear Rollback Points:** Can stop at any phase without broken state (git branches per phase)
4. ✅ **Maintain Test Coverage:** Keep >90% coverage at each phase (no dropping coverage to merge faster)
5. ✅ **Performance Validation:** Benchmark at each phase to verify we're moving toward target (<500ms operations)

**Non-Goals:**
1. ❌ **Backward Compatibility:** New API may break existing application code (acceptable—this is a refactor)
2. ❌ **Gradual Feature Addition:** Not adding new features, purely refactoring architecture
3. ❌ **Zero Downtime:** This is a local CLI tool (no live users affected during migration)
4. ❌ **Data Migration Scripts:** rkyv format remains compatible (no data conversion required)

### 1.3 Constraints (The Hard Limits)

**Migration Constraints:**
- 🔒 **Must Compile at Each Step:** No "broken window" commits (CI must stay green)
- 🔒 **Must Pass All Tests:** Test suite gates each phase (mise run test passes)
- 🔒 **No Skipping Phases:** Sequential execution required (phases have dependencies)
- 🔒 **Clear Phase Boundaries:** Each phase is a logical unit (can pause between phases for review)

**Quality Gates (Enforced at Each Phase):**
- ✅ `mise run test` passes (all tests green)
- ✅ `mise run lint` passes (zero clippy warnings)
- ✅ `mise run fmt` passes (code formatted)
- ✅ No `unwrap()` or `panic!()` in production code (only in tests with clear justification)
- ✅ Doc comments on all public APIs (with examples where non-trivial)
- ✅ Performance benchmarks pass (<500ms for template operations)

**Risk Management:**
- 🔄 **Git Branches:** Each phase is a separate branch (easy rollback)
- 🧪 **Smoke Tests:** Critical paths tested after each phase (create, render, list templates)
- 📊 **Performance Tracking:** Benchmark after each phase (detect regressions early)

---

## 2. Migration Phases (Implementation Plan)

### Phase 1: Add MiniJinja Integration (Foundation Layer)

**Goal:** Introduce MiniJinja adapter layer without removing existing code (parallel implementation)

**Duration:** 2-3 hours

**Git Branch:** `refactor/phase-1-minijinja-integration`

**Why This Phase:** Establishes MiniJinja integration foundation. Old code continues to work. No breaking changes yet.

---

#### Phase 1 Checklist

**Setup:**
- [ ] Create git branch: `git checkout -b refactor/phase-1-minijinja-integration`
- [ ] Verify starting point: `mise run test` passes

**Task 1.1: Create Adapter Module Structure**
- [ ] Create directory: `mkdir -p lithos-core/src/template/adapter`
- [ ] Create `lithos-core/src/template/adapter/mod.rs`:
  ```rust
  //! MiniJinja adapter layer for template compilation and rendering.

  pub mod engine;
  pub mod filters;
  pub mod source_generator;

  pub use engine::TemplateEngine;
  pub use filters::FilterRegistry;
  pub use source_generator::SourceGenerator;
  ```
- [ ] Update `lithos-core/src/template/mod.rs`: Add `pub mod adapter;`
- [ ] Verify compiles: `cargo check -p lithos-core`

**Task 1.2: Implement TemplateEngine**
- [ ] Create `lithos-core/src/template/adapter/engine.rs` with:
  ```rust
  use minijinja::{Environment, UndefinedBehavior, AutoEscape};
  use std::sync::Arc;
  use crate::template::{TemplateError};

  /// MiniJinja wrapper for template compilation and rendering.
  ///
  /// # Architecture
  /// - Owns Arc<Environment> for shared compiled templates
  /// - Configures strict undefined behavior (fail on missing variables)
  /// - Registers custom filters via FilterRegistry
  /// - Caches compiled templates (compile once, render many)
  pub struct TemplateEngine {
      env: Arc<Environment<'static>>,
  }

  impl TemplateEngine {
      /// Constructs engine with default configuration.
      ///
      /// Configuration:
      /// - Strict undefined behavior (fail on {{ undefined_var }})
      /// - Max template depth: 10 (prevent infinite recursion)
      /// - Auto-escape: None (we render Markdown, not HTML)
      /// - Custom filters registered (validate_length, validate_pattern, etc.)
      pub fn new() -> Self {
          let mut env = Environment::new();
          env.set_undefined_behavior(UndefinedBehavior::Strict);
          env.set_max_template_depth(10);
          env.set_auto_escape_callback(|_| AutoEscape::None);

          // Register custom filters
          super::FilterRegistry::register_all(&mut env);

          Self {
              env: Arc::new(env),
          }
      }

      /// Validates template syntax without compiling.
      ///
      /// # Errors
      /// - TemplateError::Syntax: Invalid MiniJinja syntax
      pub fn validate_syntax(&self, source: &str) -> Result<(), TemplateError> {
          let temp_name = "__validate_temp__";

          // Try to add template (validates syntax)
          match self.env.add_template(temp_name, source) {
              Ok(_) => {
                  // Clean up temporary template
                  self.env.remove_template(temp_name);
                  Ok(())
              }
              Err(e) => Err(TemplateError::Syntax(e.to_string())),
          }
      }

      /// Compiles a template and adds to cache.
      ///
      /// # Errors
      /// - TemplateError::Syntax: Invalid MiniJinja syntax
      ///
      /// # Panics
      /// If Environment is not exclusively owned (should only happen during setup).
      pub fn compile(&mut self, name: &str, source: &str) -> Result<(), TemplateError> {
          Arc::get_mut(&mut self.env)
              .expect("Environment should be exclusively owned during compile")
              .add_template(name, source)
              .map_err(|e| TemplateError::Syntax(e.to_string()))
      }

      /// Renders a compiled template with context.
      ///
      /// # Errors
      /// - TemplateError::NotFound: Template not compiled
      /// - TemplateError::Render: Rendering failed (undefined var, filter error, etc.)
      pub fn render<S: serde::Serialize>(
          &self,
          name: &str,
          context: S,
      ) -> Result<String, TemplateError> {
          let tmpl = self.env.get_template(name)
              .map_err(|_| TemplateError::NotFound(name.into()))?;

          tmpl.render(context)
              .map_err(|e| TemplateError::Render(e.to_string()))
      }
  }

  impl Default for TemplateEngine {
      fn default() -> Self {
          Self::new()
      }
  }
  ```
- [ ] Verify compiles: `cargo check -p lithos-core`

**Task 1.3: Implement FilterRegistry**
- [ ] Create `lithos-core/src/template/adapter/filters.rs` with complete filters:
  ```rust
  use minijinja::Environment;
  use regex::Regex;
  use std::cell::RefCell;
  use std::collections::HashMap;

  /// Registry for custom MiniJinja filters that enforce variable constraints.
  pub struct FilterRegistry;

  impl FilterRegistry {
      /// Registers all custom filters in the given environment.
      pub fn register_all(env: &mut Environment) {
          env.add_filter("validate_length", Self::validate_length);
          env.add_filter("validate_pattern", Self::validate_pattern);
          env.add_filter("validate_range", Self::validate_range);
          env.add_filter("validate_file_type", Self::validate_file_type);
          env.add_filter("date_format", Self::date_format);
          env.add_filter("vault_path", Self::vault_path);
      }

      /// String length validation filter.
      ///
      /// Usage: `{{ title | validate_length(min=5, max=100) }}`
      fn validate_length(
          value: String,
          min: Option<usize>,
          max: Option<usize>,
      ) -> Result<String, minijinja::Error> {
          let len = value.len();

          if let Some(min) = min {
              if len < min {
                  return Err(minijinja::Error::new(
                      minijinja::ErrorKind::InvalidOperation,
                      format!("String too short: min {min}, got {len}"),
                  ));
              }
          }

          if let Some(max) = max {
              if len > max {
                  return Err(minijinja::Error::new(
                      minijinja::ErrorKind::InvalidOperation,
                      format!("String too long: max {max}, got {len}"),
                  ));
              }
          }

          Ok(value)
      }

      /// Regex pattern validation filter (with thread-local cache).
      ///
      /// Usage: `{{ name | validate_pattern(pattern="^[A-Z]") }}`
      fn validate_pattern(
          value: String,
          pattern: String,
      ) -> Result<String, minijinja::Error> {
          thread_local! {
              static CACHE: RefCell<HashMap<String, Regex>> = RefCell::new(HashMap::new());
          }

          let is_match = CACHE.with(|cache| -> Result<bool, minijinja::Error> {
              let mut cache = cache.borrow_mut();

              if let Some(re) = cache.get(&pattern) {
                  return Ok(re.is_match(&value));
              }

              let re = Regex::new(&pattern).map_err(|e| {
                  minijinja::Error::new(
                      minijinja::ErrorKind::InvalidOperation,
                      format!("Invalid regex pattern: {e}"),
                  )
              })?;

              let result = re.is_match(&value);
              cache.insert(pattern.clone(), re);
              Ok(result)
          })?;

          if !is_match {
              return Err(minijinja::Error::new(
                  minijinja::ErrorKind::InvalidOperation,
                  format!("String does not match pattern: {pattern}"),
              ));
          }

          Ok(value)
      }

      /// Number range validation filter.
      ///
      /// Usage: `{{ priority | validate_range(min=1, max=10) }}`
      fn validate_range(
          value: f64,
          min: Option<f64>,
          max: Option<f64>,
      ) -> Result<f64, minijinja::Error> {
          if !value.is_finite() {
              return Err(minijinja::Error::new(
                  minijinja::ErrorKind::InvalidOperation,
                  format!("Value {value} is not finite"),
              ));
          }

          if let Some(min) = min {
              if value < min {
                  return Err(minijinja::Error::new(
                      minijinja::ErrorKind::InvalidOperation,
                      format!("Value {value} is below min {min}"),
                  ));
              }
          }

          if let Some(max) = max {
              if value > max {
                  return Err(minijinja::Error::new(
                      minijinja::ErrorKind::InvalidOperation,
                      format!("Value {value} is above max {max}"),
                  ));
              }
          }

          Ok(value)
      }

      /// File type validation filter.
      ///
      /// Usage: `{{ path | validate_file_type(types=["md", "txt"]) }}`
      fn validate_file_type(
          path: String,
          types: Vec<String>,
      ) -> Result<String, minijinja::Error> {
          let ext = std::path::Path::new(&path)
              .extension()
              .and_then(|e| e.to_str())
              .unwrap_or("");

          if !types.iter().any(|t| t == ext) {
              return Err(minijinja::Error::new(
                  minijinja::ErrorKind::InvalidOperation,
                  format!("File extension '{ext}' not allowed. Expected: {types:?}"),
              ));
          }

          Ok(path)
      }

      /// Date formatting filter.
      ///
      /// Usage: `{{ date | date_format(format="%Y-%m-%d") }}`
      fn date_format(
          date: String,
          format: Option<String>,
      ) -> Result<String, minijinja::Error> {
          use chrono::NaiveDate;

          if let Some(fmt) = format {
              let parsed = NaiveDate::parse_from_str(&date, &fmt).map_err(|e| {
                  minijinja::Error::new(
                      minijinja::ErrorKind::InvalidOperation,
                      format!("Invalid date format: {e}"),
                  )
              })?;
              Ok(parsed.format(&fmt).to_string())
          } else {
              // ISO 8601 pass-through
              let _parsed = date.parse::<chrono::DateTime<chrono::Utc>>().map_err(|e| {
                  minijinja::Error::new(
                      minijinja::ErrorKind::InvalidOperation,
                      format!("Invalid ISO 8601 date: {e}"),
                  )
              })?;
              Ok(date)
          }
      }

      /// Vault path validation filter.
      ///
      /// Usage: `{{ path | vault_path }}`
      fn vault_path(path: String) -> Result<String, minijinja::Error> {
          crate::fs::validate_vault_path(&path, None).map_err(|e| {
              minijinja::Error::new(
                  minijinja::ErrorKind::InvalidOperation,
                  format!("Invalid vault path: {e}"),
              )
          })?;

          Ok(path)
      }
  }
  ```
- [ ] Verify compiles: `cargo check -p lithos-core`

**Task 1.4: Add Integration Tests**
- [ ] Create `lithos-core/src/template/adapter/engine.rs` (add at end of file):
  ```rust
  #[cfg(test)]
  mod tests {
      use super::*;

      #[test]
      fn validates_template_syntax() {
          let engine = TemplateEngine::new();

          // Valid syntax
          assert!(engine.validate_syntax("Hello {{ name }}").is_ok());

          // Invalid syntax (unclosed tag)
          assert!(engine.validate_syntax("{{ unclosed").is_err());

          // Invalid syntax (unknown tag)
          assert!(engine.validate_syntax("{% unknown %}").is_err());
      }

      #[test]
      fn compiles_and_renders_simple_template() {
          let mut engine = TemplateEngine::new();

          engine.compile("test", "Hello {{ name }}!").unwrap();

          let output = engine.render("test", minijinja::context! { name => "World" }).unwrap();
          assert_eq!(output, "Hello World!");
      }

      #[test]
      fn renders_with_filter() {
          let mut engine = TemplateEngine::new();

          engine.compile("test", "{{ text | upper }}").unwrap();

          let output = engine.render("test", minijinja::context! { text => "hello" }).unwrap();
          assert_eq!(output, "HELLO");
      }

      #[test]
      fn fails_on_undefined_variable_strict_mode() {
          let mut engine = TemplateEngine::new();

          engine.compile("test", "Hello {{ undefined }}!").unwrap();

          let result = engine.render("test", minijinja::context! {});
          assert!(result.is_err());
          assert!(result.unwrap_err().to_string().contains("undefined"));
      }
  }
  ```
- [ ] Add filter tests in `lithos-core/src/template/adapter/filters.rs`:
  ```rust
  #[cfg(test)]
  mod tests {
      use super::*;
      use minijinja::Environment;

      #[test]
      fn filter_validate_length_passes() {
          let mut env = Environment::new();
          FilterRegistry::register_all(&mut env);

          env.add_template("test", "{{ text | validate_length(min=3, max=10) }}").unwrap();

          let result = env.get_template("test").unwrap()
              .render(minijinja::context! { text => "hello" });

          assert!(result.is_ok());
          assert_eq!(result.unwrap(), "hello");
      }

      #[test]
      fn filter_validate_length_fails_too_short() {
          let mut env = Environment::new();
          FilterRegistry::register_all(&mut env);

          env.add_template("test", "{{ text | validate_length(min=5) }}").unwrap();

          let result = env.get_template("test").unwrap()
              .render(minijinja::context! { text => "hi" });

          assert!(result.is_err());
          assert!(result.unwrap_err().to_string().contains("too short"));
      }

      #[test]
      fn filter_validate_pattern_passes() {
          let mut env = Environment::new();
          FilterRegistry::register_all(&mut env);

          env.add_template("test", r#"{{ text | validate_pattern(pattern="^[A-Z]") }}"#).unwrap();

          let result = env.get_template("test").unwrap()
              .render(minijinja::context! { text => "Hello" });

          assert!(result.is_ok());
      }

      #[test]
      fn filter_validate_pattern_caches_regex() {
          // Test that pattern is cached (compile once, use many times)
          let mut env = Environment::new();
          FilterRegistry::register_all(&mut env);

          env.add_template("test", r#"{{ a | validate_pattern(pattern="^[a-z]+$") }} {{ b | validate_pattern(pattern="^[a-z]+$") }}"#).unwrap();

          let result = env.get_template("test").unwrap()
              .render(minijinja::context! { a => "foo", b => "bar" });

          assert!(result.is_ok());
          assert_eq!(result.unwrap(), "foo bar");
      }
  }
  ```
- [ ] Run tests: `mise run test:unit:template`
- [ ] Verify all tests pass (old + new)

**Phase 1 Verification:**
- [ ] Run `mise run test` → All tests pass (including new adapter tests)
- [ ] Run `mise run lint` → Zero warnings
- [ ] Run `mise run fmt` → Code formatted
- [ ] Verify old template code still works (no breaking changes)
- [ ] Commit: `git add -A && git commit -m "Phase 1: Add MiniJinja integration (foundation layer)"`

**Phase 1 Complete When:**
- [ ] TemplateEngine validates syntax via MiniJinja
- [ ] TemplateEngine compiles and renders templates
- [ ] FilterRegistry implements all 6 constraint filters with tests
- [ ] Integration tests pass (syntax validation, compilation, rendering, filters)
- [ ] Existing tests still pass (no breaking changes)
- [ ] Code review passed (if applicable)

**🎯 Success Criteria:**
- ✅ New adapter layer exists alongside old code
- ✅ MiniJinja is now integrated (no longer zero imports)
- ✅ Can compile and render simple templates via MiniJinja
- ✅ Can validate template syntax without old domain validation
- ✅ Old code untouched (backward compatible)

---

### Phase 2: Refactor Domain Model (Breaking Changes Begin)

**Goal:** Transform Template entity from processor to metadata schema

**Duration:** 3-4 hours

**Git Branch:** `refactor/phase-2-domain-metadata`

**Why This Phase:** Refactors domain to be metadata-only. Introduces TemplateBlock and BlockStrategy. Deprecates old composition logic. This is the first breaking change (Template constructor signature changes).

---

#### Phase 2 Checklist

**Setup:**
- [ ] Merge Phase 1: `git checkout main && git merge refactor/phase-1-minijinja-integration`
- [ ] Create new branch: `git checkout -b refactor/phase-2-domain-metadata`
- [ ] Verify starting point: `mise run test` passes

**Task 2.1: Add TemplateBlock and BlockStrategy Types**
- [ ] Create `lithos-core/src/template/block.rs`:
  ```rust
  //! Template block metadata for composition.

  use rkyv::{Archive, Serialize as RkyvSerialize, Deserialize as RkyvDeserialize};

  /// Block composition strategy (how child relates to parent block).
  #[derive(Debug, Clone, Copy, PartialEq, Eq, Archive, RkyvSerialize, RkyvDeserialize)]
  #[rkyv(derive(Debug))]
  pub enum BlockStrategy {
      /// Replace parent's block entirely (default).
      ///
      /// Generates: `{% block name %}{{ content }}{% endblock %}`
      Replace,

      /// Call parent's block first, then append ours.
      ///
      /// Generates: `{% block name %}{{ super() }}{{ content }}{% endblock %}`
      Extend,

      /// Append our content, then call parent's block.
      ///
      /// Generates: `{% block name %}{{ content }}{{ super() }}{% endblock %}`
      Prepend,
  }

  /// Template block metadata (name, content, composition strategy).
  ///
  /// Blocks are the unit of composition in template inheritance.
  /// Child templates override parent blocks using different strategies.
  #[derive(Debug, Clone, PartialEq, Archive, RkyvSerialize, RkyvDeserialize)]
  #[rkyv(derive(Debug))]
  #[non_exhaustive]
  pub struct TemplateBlock {
      /// Block identifier (must be unique within template).
      name: Box<str>,

      /// Block content (raw text, may contain MiniJinja syntax).
      content: Box<str>,

      /// How this block composes with parent block.
      strategy: BlockStrategy,
  }

  impl TemplateBlock {
      /// Constructs a new template block.
      ///
      /// # Example
      /// ```
      /// let block = TemplateBlock::new(
      ///     "header",
      ///     "# {{ title }}",
      ///     BlockStrategy::Replace,
      /// );
      /// ```
      pub fn new(name: &str, content: &str, strategy: BlockStrategy) -> Self {
          Self {
              name: name.into(),
              content: content.into(),
              strategy,
          }
      }

      pub fn name(&self) -> &str {
          &self.name
      }

      pub fn content(&self) -> &str {
          &self.content
      }

      pub const fn strategy(&self) -> BlockStrategy {
          self.strategy
      }
  }

  #[cfg(test)]
  mod tests {
      use super::*;

      #[test]
      fn creates_block_with_replace_strategy() {
          let block = TemplateBlock::new("test", "content", BlockStrategy::Replace);
          assert_eq!(block.name(), "test");
          assert_eq!(block.content(), "content");
          assert_eq!(block.strategy(), BlockStrategy::Replace);
      }

      #[test]
      fn creates_block_with_extend_strategy() {
          let block = TemplateBlock::new("test", "content", BlockStrategy::Extend);
          assert_eq!(block.strategy(), BlockStrategy::Extend);
      }

      #[test]
      fn creates_block_with_prepend_strategy() {
          let block = TemplateBlock::new("test", "content", BlockStrategy::Prepend);
          assert_eq!(block.strategy(), BlockStrategy::Prepend);
      }
  }
  ```
- [ ] Update `lithos-core/src/template/mod.rs`: Add `pub mod block;` and `pub use block::{TemplateBlock, BlockStrategy};`
- [ ] Verify compiles: `cargo check -p lithos-core`

**Task 2.2: Update Template Entity**
- [ ] Backup current `aggregate.rs`: `cp lithos-core/src/template/aggregate.rs lithos-core/src/template/aggregate.rs.backup`
- [ ] Update `Template` struct in `lithos-core/src/template/aggregate.rs`:
  - [ ] Add field: `extends: Option<Box<str>>,`
  - [ ] Add field: `blocks: Vec<TemplateBlock>,`
  - [ ] Mark deprecated: Add `#[deprecated]` to `content: String` field
  - [ ] Mark deprecated: Add `#[deprecated]` to `syntax: PlaceholderSyntax` field
- [ ] Update `Template::new()` constructor:
  - [ ] Change signature to: `pub fn new(name: &str, extends: Option<&str>, blocks: Vec<TemplateBlock>, variables: HashMap<String, VariableDefinition>) -> Result<Self, TemplateError>`
  - [ ] Initialize `extends` field
  - [ ] Initialize `blocks` field
  - [ ] Initialize deprecated fields as empty (for backward compat during migration)
- [ ] Add accessor methods:
  - [ ] `pub fn extends(&self) -> Option<&str>`
  - [ ] `pub fn blocks(&self) -> &[TemplateBlock]`
- [ ] Deprecate old methods:
  - [ ] Add `#[deprecated(note = "Use TemplateEngine::validate_syntax instead")]` to `Template::validate()`
  - [ ] Add `#[deprecated(note = "Use MiniJinja native {% extends %} instead")]` to `Template::compose()` if it exists
- [ ] Verify compiles: `cargo check -p lithos-core` (warnings expected for deprecated fields)

**Task 2.3: Update VariableDefinition**
- [ ] Add method to `lithos-core/src/template/variable.rs`:
  ```rust
  impl VariableDefinition {
      /// Returns filter names to apply at render time.
      ///
      /// Used by adapter to generate filter chains in MiniJinja templates.
      pub fn filter_chain(&self) -> Vec<&'static str> {
          match self {
              Self::String { pattern: Some(_), min_length: Some(_), .. } |
              Self::String { pattern: Some(_), max_length: Some(_), .. } => {
                  vec!["validate_pattern", "validate_length"]
              }
              Self::String { pattern: Some(_), .. } => vec!["validate_pattern"],
              Self::String { min_length: Some(_), .. } |
              Self::String { max_length: Some(_), .. } => vec!["validate_length"],
              Self::Number { min: Some(_), .. } | Self::Number { max: Some(_), .. } => {
                  vec!["validate_range"]
              }
              Self::File { file_types: Some(_), .. } => vec!["validate_file_type"],
              Self::Date { format: Some(_), .. } => vec!["date_format"],
              _ => vec![],
          }
      }

      /// Returns filter arguments as JSON.
      ///
      /// Used by adapter to pass constraint values to filters.
      pub fn filter_args(&self) -> serde_json::Value {
          match self {
              Self::String { min_length, max_length, pattern, .. } => {
                  serde_json::json!({
                      "min": min_length,
                      "max": max_length,
                      "pattern": pattern,
                  })
              }
              Self::Number { min, max, .. } => {
                  serde_json::json!({ "min": min, "max": max })
              }
              Self::File { file_types, .. } => {
                  serde_json::json!({ "types": file_types })
              }
              Self::Date { format, .. } => {
                  serde_json::json!({ "format": format })
              }
              _ => serde_json::json!({}),
          }
      }

      /// Returns default value as JSON.
      pub fn default_value(&self) -> Option<serde_json::Value> {
          match self {
              Self::Boolean { default } => default.map(|v| serde_json::json!(v)),
              Self::Number { default, .. } => default.map(|v| serde_json::json!(v)),
              Self::String { default, .. } |
              Self::Date { default, .. } |
              Self::File { default, .. } => default.as_ref().map(|v| serde_json::json!(v)),
          }
      }

      /// Checks if variable has default.
      pub fn has_default(&self) -> bool {
          self.default_value().is_some()
      }
  }
  ```
- [ ] Deprecate old validation: Add `#[deprecated(note = "Use MiniJinja filters for validation")]` to `validate_value()` if it exists
- [ ] Verify compiles: `cargo check -p lithos-core`

**Task 2.4: Update Domain Tests**
- [ ] Update tests in `lithos-core/src/template/aggregate.rs`:
  ```rust
  #[cfg(test)]
  mod tests {
      use super::*;

      #[test]
      fn creates_template_with_blocks() {
          let template = Template::new(
              "daily-note",
              Some("base-note"),
              vec![
                  TemplateBlock::new(
                      "header",
                      "# Daily Note: {{ date }}",
                      BlockStrategy::Replace,
                  ),
              ],
              HashMap::new(),
          ).unwrap();

          assert_eq!(template.name(), "daily-note");
          assert_eq!(template.extends(), Some("base-note"));
          assert_eq!(template.blocks().len(), 1);
          assert_eq!(template.blocks()[0].name(), "header");
      }

      #[test]
      fn creates_template_without_extends() {
          let template = Template::new(
              "standalone",
              None,  // No parent
              vec![
                  TemplateBlock::new(
                      "content",
                      "Hello {{ name }}",
                      BlockStrategy::Replace,
                  ),
              ],
              HashMap::new(),
          ).unwrap();

          assert_eq!(template.extends(), None);
      }

      #[test]
      fn template_block_strategies() {
          let template = Template::new(
              "test",
              Some("parent"),
              vec![
                  TemplateBlock::new("a", "content-a", BlockStrategy::Replace),
                  TemplateBlock::new("b", "content-b", BlockStrategy::Extend),
                  TemplateBlock::new("c", "content-c", BlockStrategy::Prepend),
              ],
              HashMap::new(),
          ).unwrap();

          assert_eq!(template.blocks()[0].strategy(), BlockStrategy::Replace);
          assert_eq!(template.blocks()[1].strategy(), BlockStrategy::Extend);
          assert_eq!(template.blocks()[2].strategy(), BlockStrategy::Prepend);
      }
  }
  ```
- [ ] Add tests for VariableDefinition in `lithos-core/src/template/variable.rs`:
  ```rust
  #[cfg(test)]
  mod tests {
      use super::*;

      #[test]
      fn filter_chain_for_string_with_pattern_and_length() {
          let var = VariableDefinition::String {
              default: None,
              min_length: Some(5),
              max_length: Some(10),
              pattern: Some("^[A-Z]".into()),
          };

          let chain = var.filter_chain();
          assert_eq!(chain, vec!["validate_pattern", "validate_length"]);
      }

      #[test]
      fn filter_chain_for_number_with_range() {
          let var = VariableDefinition::Number {
              default: None,
              min: Some(1.0),
              max: Some(10.0),
          };

          let chain = var.filter_chain();
          assert_eq!(chain, vec!["validate_range"]);
      }

      #[test]
      fn filter_args_for_string() {
          let var = VariableDefinition::String {
              default: None,
              min_length: Some(5),
              max_length: Some(10),
              pattern: None,
          };

          let args = var.filter_args();
          assert_eq!(args["min"], 5);
          assert_eq!(args["max"], 10);
      }

      #[test]
      fn has_default_returns_true_when_default_exists() {
          let var = VariableDefinition::String {
              default: Some("hello".into()),
              min_length: None,
              max_length: None,
              pattern: None,
          };

          assert!(var.has_default());
      }

      #[test]
      fn has_default_returns_false_when_no_default() {
          let var = VariableDefinition::String {
              default: None,
              min_length: None,
              max_length: None,
              pattern: None,
          };

          assert!(!var.has_default());
      }
  }
  ```
- [ ] Run tests: `mise run test:unit:template`
- [ ] Fix any broken tests (update to new constructor signature)

**Phase 2 Verification:**
- [ ] Run `mise run test` → All tests pass (updated tests use new API)
- [ ] Run `mise run lint` → Only deprecated warnings (expected)
- [ ] Run `mise run fmt` → Code formatted
- [ ] Verify Template now has `extends` and `blocks` fields
- [ ] Commit: `git add -A && git commit -m "Phase 2: Refactor domain to metadata schema (BREAKING)"`

**Phase 2 Complete When:**
- [ ] Template entity has `extends: Option<Box<str>>` field
- [ ] Template entity has `blocks: Vec<TemplateBlock>` field
- [ ] TemplateBlock and BlockStrategy types exist with tests
- [ ] Old `content` and `syntax` fields marked `#[deprecated]`
- [ ] Old `validate()` and `compose()` methods deprecated
- [ ] VariableDefinition has `filter_chain()`, `filter_args()`, `default_value()`, `has_default()` methods
- [ ] All unit tests updated and passing with new API

**🎯 Success Criteria:**
- ✅ Domain is now metadata-only (no processing logic)
- ✅ Template describes structure via blocks (not raw content string)
- ✅ VariableDefinition provides filter metadata (not validation logic)
- ✅ Old API deprecated (visible warnings guide refactor)
- ✅ Tests demonstrate new composition model

---

### Phase 3: Add SourceGenerator (Code Generation)

**Goal:** Convert Template metadata → MiniJinja source code

**Duration:** 2-3 hours

**Git Branch:** `refactor/phase-3-source-generator`

**Why This Phase:** Bridges domain metadata and MiniJinja rendering. Generates `{% extends %}` and `{% block %}` directives from Template metadata. Enables round-trip testing (metadata → source → rendering).

---

#### Phase 3 Checklist

**Setup:**
- [ ] Merge Phase 2: `git checkout main && git merge refactor/phase-2-domain-metadata`
- [ ] Create new branch: `git checkout -b refactor/phase-3-source-generator`
- [ ] Verify starting point: `mise run test` passes

**Task 3.1: Implement SourceGenerator**
- [ ] Create `lithos-core/src/template/adapter/source_generator.rs`:
  ```rust
  //! Generates MiniJinja source code from Template metadata.

  use crate::template::{Template, TemplateBlock, BlockStrategy, TemplateError};
  use std::fmt::Write;

  /// Generates MiniJinja source code from Template metadata.
  ///
  /// # Algorithm
  /// 1. If `extends` is Some, emit `{% extends "parent" %}`
  /// 2. For each block, emit `{% block name %}...{% endblock %}`
  /// 3. Apply strategy: Replace (content only), Extend ({{ super() }} + content), Prepend (content + {{ super() }})
  ///
  /// # Example
  /// ```
  /// let template = Template::new(
  ///     "child",
  ///     Some("parent"),
  ///     vec![
  ///         TemplateBlock::new("header", "# Custom", BlockStrategy::Replace),
  ///         TemplateBlock::new("content", "Extra content", BlockStrategy::Extend),
  ///     ],
  ///     HashMap::new(),
  /// )?;
  ///
  /// let generator = SourceGenerator;
  /// let source = generator.generate(&template)?;
  ///
  /// // Generated source:
  /// // {% extends "parent" %}
  /// //
  /// // {% block header %}
  /// // # Custom
  /// // {% endblock %}
  /// //
  /// // {% block content %}
  /// // {{ super() }}
  /// // Extra content
  /// // {% endblock %}
  /// ```
  pub struct SourceGenerator;

  impl SourceGenerator {
      /// Generates MiniJinja source from template metadata.
      ///
      /// # Errors
      /// - TemplateError::Syntax: fmt::Error during string building (should never happen)
      pub fn generate(&self, template: &Template) -> Result<String, TemplateError> {
          let mut source = String::new();

          // 1. Add extends directive (if parent exists)
          if let Some(parent) = template.extends() {
              writeln!(source, "{{% extends \"{}\" %}}", parent)
                  .map_err(|e| TemplateError::Syntax(format!("Failed to write extends: {e}")))?;
              writeln!(source)
                  .map_err(|e| TemplateError::Syntax(format!("Failed to write newline: {e}")))?;
          }

          // 2. Add block definitions
          for block in template.blocks() {
              self.generate_block(&mut source, block)?;
          }

          // 3. If no blocks and no extends, it's a simple template (use first block content if any)
          if template.blocks().is_empty() && template.extends().is_none() {
              // Simple templates can have a single implicit block
              // (This case is rare; most templates have explicit blocks)
          }

          Ok(source)
      }

      /// Generates a single block definition.
      fn generate_block(&self, source: &mut String, block: &TemplateBlock) -> Result<(), TemplateError> {
          writeln!(source, "{{% block {} %}}", block.name())
              .map_err(|e| TemplateError::Syntax(format!("Failed to write block start: {e}")))?;

          match block.strategy() {
              BlockStrategy::Replace => {
                  // Just emit content (no super() call)
                  writeln!(source, "{}", block.content())
                      .map_err(|e| TemplateError::Syntax(format!("Failed to write block content: {e}")))?;
              }
              BlockStrategy::Extend => {
                  // Call parent first, then append our content
                  writeln!(source, "{{{{ super() }}}}")
                      .map_err(|e| TemplateError::Syntax(format!("Failed to write super(): {e}")))?;
                  writeln!(source, "{}", block.content())
                      .map_err(|e| TemplateError::Syntax(format!("Failed to write block content: {e}")))?;
              }
              BlockStrategy::Prepend => {
                  // Emit our content first, then call parent
                  writeln!(source, "{}", block.content())
                      .map_err(|e| TemplateError::Syntax(format!("Failed to write block content: {e}")))?;
                  writeln!(source, "{{{{ super() }}}}")
                      .map_err(|e| TemplateError::Syntax(format!("Failed to write super(): {e}")))?;
              }
          }

          writeln!(source, "{{% endblock %}}")
              .map_err(|e| TemplateError::Syntax(format!("Failed to write block end: {e}")))?;
          writeln!(source)
              .map_err(|e| TemplateError::Syntax(format!("Failed to write newline: {e}")))?;

          Ok(())
      }
  }

  #[cfg(test)]
  mod tests {
      use super::*;
      use std::collections::HashMap;

      #[test]
      fn generates_extends_directive() {
          let template = Template::new(
              "child",
              Some("parent"),
              vec![],
              HashMap::new(),
          ).unwrap();

          let generator = SourceGenerator;
          let source = generator.generate(&template).unwrap();

          assert!(source.contains("{% extends \"parent\" %}"));
      }

      #[test]
      fn generates_block_with_replace_strategy() {
          let template = Template::new(
              "test",
              None,
              vec![
                  TemplateBlock::new(
                      "header",
                      "# Title",
                      BlockStrategy::Replace,
                  ),
              ],
              HashMap::new(),
          ).unwrap();

          let generator = SourceGenerator;
          let source = generator.generate(&template).unwrap();

          assert!(source.contains("{% block header %}"));
          assert!(source.contains("# Title"));
          assert!(source.contains("{% endblock %}"));
          assert!(!source.contains("super()"));  // Replace doesn't call parent
      }

      #[test]
      fn generates_block_with_extend_strategy() {
          let template = Template::new(
              "test",
              None,
              vec![
                  TemplateBlock::new(
                      "content",
                      "Additional content",
                      BlockStrategy::Extend,
                  ),
              ],
              HashMap::new(),
          ).unwrap();

          let generator = SourceGenerator;
          let source = generator.generate(&template).unwrap();

          assert!(source.contains("{{ super() }}"));  // Extend calls parent first
          assert!(source.contains("Additional content"));

          // Verify super() comes BEFORE content
          let super_pos = source.find("{{ super() }}").unwrap();
          let content_pos = source.find("Additional content").unwrap();
          assert!(super_pos < content_pos);
      }

      #[test]
      fn generates_block_with_prepend_strategy() {
          let template = Template::new(
              "test",
              None,
              vec![
                  TemplateBlock::new(
                      "content",
                      "Prepended content",
                      BlockStrategy::Prepend,
                  ),
              ],
              HashMap::new(),
          ).unwrap();

          let generator = SourceGenerator;
          let source = generator.generate(&template).unwrap();

          assert!(source.contains("{{ super() }}"));  // Prepend calls parent after
          assert!(source.contains("Prepended content"));

          // Verify content comes BEFORE super()
          let content_pos = source.find("Prepended content").unwrap();
          let super_pos = source.find("{{ super() }}").unwrap();
          assert!(content_pos < super_pos);
      }

      #[test]
      fn generates_multiple_blocks() {
          let template = Template::new(
              "test",
              Some("base"),
              vec![
                  TemplateBlock::new("header", "# Title", BlockStrategy::Replace),
                  TemplateBlock::new("content", "Body", BlockStrategy::Replace),
                  TemplateBlock::new("footer", "Footer", BlockStrategy::Replace),
              ],
              HashMap::new(),
          ).unwrap();

          let generator = SourceGenerator;
          let source = generator.generate(&template).unwrap();

          assert!(source.contains("{% extends \"base\" %}"));
          assert!(source.contains("{% block header %}"));
          assert!(source.contains("{% block content %}"));
          assert!(source.contains("{% block footer %}"));
      }
  }
  ```
- [ ] Update `lithos-core/src/template/adapter/mod.rs`: Add `pub mod source_generator;` and `pub use source_generator::SourceGenerator;`
- [ ] Verify compiles: `cargo check -p lithos-core`
- [ ] Run tests: `mise run test:unit:template`

**Task 3.2: Integrate with TemplateEngine**
- [ ] Update `lithos-core/src/template/adapter/engine.rs`, add method:
  ```rust
  impl TemplateEngine {
      /// Compiles a template from domain metadata.
      ///
      /// Generates MiniJinja source via SourceGenerator, then compiles it.
      ///
      /// # Errors
      /// - TemplateError::Syntax: Invalid generated source or compilation failed
      pub fn compile_from_template(&mut self, template: &Template) -> Result<(), TemplateError> {
          let generator = super::SourceGenerator;
          let source = generator.generate(template)?;

          self.compile(template.name(), &source)
      }
  }
  ```
- [ ] Verify compiles: `cargo check -p lithos-core`

**Task 3.3: Add Round-Trip Tests**
- [ ] Add end-to-end tests in `lithos-core/src/template/adapter/engine.rs`:
  ```rust
  #[cfg(test)]
  mod integration_tests {
      use super::*;
      use crate::template::{Template, TemplateBlock, BlockStrategy};
      use std::collections::HashMap;

      #[test]
      fn end_to_end_parent_child_compilation() {
          // Create parent template
          let parent = Template::new(
              "parent",
              None,
              vec![
                  TemplateBlock::new(
                      "title",
                      "Default Title",
                      BlockStrategy::Replace,
                  ),
              ],
              HashMap::new(),
          ).unwrap();

          // Create child template that extends parent
          let child = Template::new(
              "child",
              Some("parent"),
              vec![
                  TemplateBlock::new(
                      "title",
                      "Custom Title",
                      BlockStrategy::Replace,
                  ),
              ],
              HashMap::new(),
          ).unwrap();

          // Compile both (parent must be compiled first)
          let mut engine = TemplateEngine::new();
          engine.compile_from_template(&parent).unwrap();
          engine.compile_from_template(&child).unwrap();

          // Render child (should use child's title, not parent's)
          let output = engine.render("child", minijinja::context! {}).unwrap();
          assert!(output.contains("Custom Title"));
          assert!(!output.contains("Default Title"));
      }

      #[test]
      fn end_to_end_extend_strategy() {
          // Parent with base content
          let parent = Template::new(
              "parent",
              None,
              vec![
                  TemplateBlock::new(
                      "content",
                      "Parent content",
                      BlockStrategy::Replace,
                  ),
              ],
              HashMap::new(),
          ).unwrap();

          // Child extends parent's content (calls super() + adds more)
          let child = Template::new(
              "child",
              Some("parent"),
              vec![
                  TemplateBlock::new(
                      "content",
                      "Child content",
                      BlockStrategy::Extend,  // Calls {{ super() }} first
                  ),
              ],
              HashMap::new(),
          ).unwrap();

          let mut engine = TemplateEngine::new();
          engine.compile_from_template(&parent).unwrap();
          engine.compile_from_template(&child).unwrap();

          let output = engine.render("child", minijinja::context! {}).unwrap();

          // Should contain BOTH parent and child content
          assert!(output.contains("Parent content"));
          assert!(output.contains("Child content"));

          // Parent content should come BEFORE child content
          let parent_pos = output.find("Parent content").unwrap();
          let child_pos = output.find("Child content").unwrap();
          assert!(parent_pos < child_pos);
      }

      #[test]
      fn end_to_end_prepend_strategy() {
          let parent = Template::new(
              "parent",
              None,
              vec![
                  TemplateBlock::new(
                      "content",
                      "Parent content",
                      BlockStrategy::Replace,
                  ),
              ],
              HashMap::new(),
          ).unwrap();

          // Child prepends to parent's content (adds more + calls super())
          let child = Template::new(
              "child",
              Some("parent"),
              vec![
                  TemplateBlock::new(
                      "content",
                      "Child content",
                      BlockStrategy::Prepend,  // Calls {{ super() }} after
                  ),
              ],
              HashMap::new(),
          ).unwrap();

          let mut engine = TemplateEngine::new();
          engine.compile_from_template(&parent).unwrap();
          engine.compile_from_template(&child).unwrap();

          let output = engine.render("child", minijinja::context! {}).unwrap();

          // Should contain BOTH
          assert!(output.contains("Parent content"));
          assert!(output.contains("Child content"));

          // Child content should come BEFORE parent content
          let child_pos = output.find("Child content").unwrap();
          let parent_pos = output.find("Parent content").unwrap();
          assert!(child_pos < parent_pos);
      }

      #[test]
      fn end_to_end_three_level_inheritance() {
          // Grandparent
          let grandparent = Template::new(
              "grandparent",
              None,
              vec![
                  TemplateBlock::new("title", "Grandparent", BlockStrategy::Replace),
              ],
              HashMap::new(),
          ).unwrap();

          // Parent extends grandparent
          let parent = Template::new(
              "parent",
              Some("grandparent"),
              vec![
                  TemplateBlock::new("title", "Parent", BlockStrategy::Replace),
              ],
              HashMap::new(),
          ).unwrap();

          // Child extends parent
          let child = Template::new(
              "child",
              Some("parent"),
              vec![
                  TemplateBlock::new("title", "Child", BlockStrategy::Replace),
              ],
              HashMap::new(),
          ).unwrap();

          let mut engine = TemplateEngine::new();
          engine.compile_from_template(&grandparent).unwrap();
          engine.compile_from_template(&parent).unwrap();
          engine.compile_from_template(&child).unwrap();

          let output = engine.render("child", minijinja::context! {}).unwrap();
          assert!(output.contains("Child"));
          assert!(!output.contains("Parent"));
          assert!(!output.contains("Grandparent"));
      }
  }
  ```
- [ ] Run tests: `mise run test:unit:template`
- [ ] Verify all round-trip tests pass

**Phase 3 Verification:**
- [ ] Run `mise run test` → All tests pass (including round-trip tests)
- [ ] Run `mise run lint` → Zero warnings (or only expected deprecated warnings)
- [ ] Run `mise run fmt` → Code formatted
- [ ] Manually test: Create template with blocks, generate source, verify output looks correct
- [ ] Commit: `git add -A && git commit -m "Phase 3: Add SourceGenerator (metadata → MiniJinja source)"`

**Phase 3 Complete When:**
- [ ] SourceGenerator converts Template → MiniJinja source ({% extends %}, {% block %})
- [ ] TemplateEngine::compile_from_template() method exists and works
- [ ] Round-trip tests pass (metadata → source → compilation → rendering)
- [ ] Template inheritance works (parent/child templates render correctly)
- [ ] All three BlockStrategies work (Replace, Extend, Prepend)
- [ ] Multi-level inheritance works (3+ levels)

**🎯 Success Criteria:**
- ✅ Can generate valid MiniJinja source from Template metadata
- ✅ Parent templates compile before children (manual ordering for now)
- ✅ Extends and blocks work correctly (MiniJinja handles inheritance)
- ✅ All composition strategies produce correct output

---

### Phase 4: Add TemplateCatalog (Lifecycle Orchestration)

**Goal:** Central orchestrator for load → compile → cache → render with topological sorting

**Duration:** 2-3 hours

**Git Branch:** `refactor/phase-4-catalog`

**Why This Phase:** Adds the missing piece: automatic template loading from storage, dependency-order compilation (via topological sort), and unified render API. This is where compilation caching pays off.

---

#### Phase 4 Checklist

**Setup:**
- [ ] Merge Phase 3: `git checkout main && git merge refactor/phase-3-source-generator`
- [ ] Create new branch: `git checkout -b refactor/phase-4-catalog`
- [ ] Verify starting point: `mise run test` passes

**Task 4.1: Implement TemplateCatalog**
- [ ] Create `lithos-core/src/template/catalog.rs`:
  ```rust
  //! Template lifecycle manager (load → compile → cache → render).

  use crate::template::{Template, TemplateError, TemplateQueryPort};
  use crate::template::adapter::{SourceGenerator, FilterRegistry};
  use minijinja::{Environment, UndefinedBehavior, AutoEscape};
  use std::collections::{HashMap, VecDeque};
  use std::sync::Arc;

  /// Template catalog: orchestrates loading, compilation, and rendering.
  ///
  /// # Responsibilities
  /// - Loads all templates from storage (via TemplateQueryPort)
  /// - Topologically sorts by extends relationships (parents before children)
  /// - Compiles templates via SourceGenerator + MiniJinja
  /// - Caches compiled templates in Arc<Environment> (shared across threads)
  /// - Provides unified render API
  ///
  /// # Architecture
  /// ```text
  /// TemplateCatalog
  ///   ├─ metadata: Box<dyn TemplateQueryPort>  (reads from storage)
  ///   ├─ generator: SourceGenerator             (converts to MiniJinja)
  ///   └─ env: Arc<Environment>                  (compiled template cache)
  /// ```
  ///
  /// # Lifecycle
  /// 1. Construct: `TemplateCatalog::new(storage)`
  /// 2. Load & compile: `catalog.load_all()` (once at startup)
  /// 3. Render: `catalog.render(name, context)` (many times, fast path)
  pub struct TemplateCatalog {
      /// Compiled templates (shared across threads)
      env: Arc<Environment<'static>>,

      /// Domain metadata storage (for template queries)
      metadata: Box<dyn TemplateQueryPort>,

      /// Source code generator
      generator: SourceGenerator,
  }

  impl TemplateCatalog {
      /// Constructs catalog with storage backend.
      ///
      /// Configures MiniJinja Environment:
      /// - Strict undefined behavior (fail on {{ undefined }})
      /// - Max template depth: 10 (prevent infinite recursion)
      /// - Auto-escape: None (we render Markdown, not HTML)
      /// - Registers custom filters (validate_length, etc.)
      pub fn new(metadata: Box<dyn TemplateQueryPort>) -> Result<Self, TemplateError> {
          let mut env = Environment::new();
          env.set_undefined_behavior(UndefinedBehavior::Strict);
          env.set_max_template_depth(10);
          env.set_auto_escape_callback(|_| AutoEscape::None);

          FilterRegistry::register_all(&mut env);

          Ok(Self {
              env: Arc::new(env),
              metadata,
              generator: SourceGenerator,
          })
      }

      /// Loads and compiles ALL templates from storage.
      ///
      /// # Algorithm
      /// 1. Load all template metadata from storage
      /// 2. Build dependency graph (who extends whom)
      /// 3. Topologically sort (Kahn's algorithm)
      /// 4. For each template in sorted order:
      ///    a. Generate MiniJinja source
      ///    b. Compile with Environment.add_template()
      ///
      /// # Performance
      /// O(N) templates × compilation cost. Call ONCE at startup.
      ///
      /// # Errors
      /// - Storage: Database read failed
      /// - CircularComposition: Cycle detected in extends
      /// - Syntax: Generated MiniJinja source invalid or compilation failed
      pub fn load_all(&mut self) -> Result<(), TemplateError> {
          // 1. Load all templates from storage
          let templates = self.metadata.list()?;

          // 2. Topologically sort by extends (parents before children)
          let sorted = self.topological_sort(&templates)?;

          // 3. Compile in dependency order
          let env = Arc::get_mut(&mut self.env)
              .expect("Environment should be exclusively owned during load");

          for template in sorted {
              let source = self.generator.generate(template)?;
              env.add_template(template.name(), &source)
                  .map_err(|e| TemplateError::Syntax(format!(
                      "Failed to compile template '{}': {}",
                      template.name(),
                      e
                  )))?;
          }

          Ok(())
      }

      /// Renders a compiled template with context.
      ///
      /// # Performance
      /// O(1) lookup + O(AST size) execution. This is the FAST PATH (no I/O, no parsing).
      ///
      /// # Errors
      /// - NotFound: Template not compiled (did you call load_all()?)
      /// - Render: Undefined variable, filter validation failed, or other render error
      pub fn render<S: serde::Serialize>(
          &self,
          name: &str,
          context: S,
      ) -> Result<String, TemplateError> {
          let tmpl = self.env.get_template(name)
              .map_err(|_| TemplateError::NotFound(name.into()))?;

          tmpl.render(context)
              .map_err(|e| TemplateError::Render(e.to_string()))
      }

      /// Lists all template names (for discovery).
      pub fn list_names(&self) -> Result<Vec<String>, TemplateError> {
          let templates = self.metadata.list()?;
          Ok(templates.into_iter().map(|t| t.name().into()).collect())
      }

      /// Topologically sorts templates by extends relationships (Kahn's algorithm).
      ///
      /// # Algorithm
      /// 1. Build adjacency list (parent → children) and in-degree map
      /// 2. Start with templates that have zero in-degree (no parent)
      /// 3. BFS: process template, reduce children's in-degree
      /// 4. If all templates processed, success; otherwise cycle detected
      ///
      /// # Errors
      /// - CircularComposition: Cycle in extends graph (A extends B extends A)
      fn topological_sort<'a>(
          &self,
          templates: &'a [Template],
      ) -> Result<Vec<&'a Template>, TemplateError> {
          // Build adjacency list and in-degree map
          let mut graph: HashMap<&str, Vec<&str>> = HashMap::new();
          let mut in_degree: HashMap<&str, usize> = HashMap::new();
          let mut template_map: HashMap<&str, &Template> = HashMap::new();

          for template in templates {
              template_map.insert(template.name(), template);
              in_degree.entry(template.name()).or_insert(0);

              if let Some(parent) = template.extends() {
                  graph.entry(parent).or_default().push(template.name());
                  *in_degree.entry(template.name()).or_insert(0) += 1;
              }
          }

          // Find templates with zero in-degree (no dependencies)
          let mut queue: VecDeque<&str> = in_degree
              .iter()
              .filter(|(_, &deg)| deg == 0)
              .map(|(&name, _)| name)
              .collect();

          let mut sorted = Vec::new();

          // BFS traversal (Kahn's algorithm)
          while let Some(current) = queue.pop_front() {
              sorted.push(template_map[current]);

              // Reduce in-degree of children
              if let Some(children) = graph.get(current) {
                  for &child in children {
                      let deg = in_degree.get_mut(child).unwrap();
                      *deg -= 1;
                      if *deg == 0 {
                          queue.push_back(child);
                      }
                  }
              }
          }

          // Check for cycles
          if sorted.len() != templates.len() {
              return Err(TemplateError::CircularComposition(
                  "Cycle detected in template extends relationships".into(),
              ));
          }

          Ok(sorted)
      }
  }

  #[cfg(test)]
  mod tests {
      use super::*;
      use crate::template::{TemplateBlock, BlockStrategy, Query, Command};
      use crate::template::FakeTemplateStorage;
      use std::collections::HashMap;

      #[test]
      fn loads_and_compiles_all_templates() {
          let storage = FakeTemplateStorage::new();
          let query = Query::new(storage.clone());
          let command = Command::new(storage.clone());

          // Create parent template
          let parent = Template::new(
              "parent",
              None,
              vec![
                  TemplateBlock::new("title", "Default Title", BlockStrategy::Replace),
              ],
              HashMap::new(),
          ).unwrap();

          // Create child template
          let child = Template::new(
              "child",
              Some("parent"),
              vec![
                  TemplateBlock::new("title", "Custom Title", BlockStrategy::Replace),
              ],
              HashMap::new(),
          ).unwrap();

          command.create(&parent).unwrap();
          command.create(&child).unwrap();

          // Load all into catalog (automatic topological sort)
          let mut catalog = TemplateCatalog::new(Box::new(query)).unwrap();
          catalog.load_all().unwrap();

          // Render child template
          let output = catalog.render("child", minijinja::context! {}).unwrap();
          assert!(output.contains("Custom Title"));
          assert!(!output.contains("Default Title"));
      }

      #[test]
      fn detects_circular_extends() {
          let storage = FakeTemplateStorage::new();
          let command = Command::new(storage.clone());
          let query = Query::new(storage.clone());

          // Create circular dependency: A extends B, B extends A
          let a = Template::new(
              "a",
              Some("b"),
              vec![],
              HashMap::new(),
          ).unwrap();

          let b = Template::new(
              "b",
              Some("a"),
              vec![],
              HashMap::new(),
          ).unwrap();

          command.create(&a).unwrap();
          command.create(&b).unwrap();

          // Load should fail with cycle detection error
          let mut catalog = TemplateCatalog::new(Box::new(query)).unwrap();
          let result = catalog.load_all();

          assert!(matches!(result, Err(TemplateError::CircularComposition(_))));
      }

      #[test]
      fn topological_sort_compiles_parents_before_children() {
          let storage = FakeTemplateStorage::new();
          let command = Command::new(storage.clone());
          let query = Query::new(storage.clone());

          // Create 3-level hierarchy: grandparent <- parent <- child
          let grandparent = Template::new(
              "grandparent",
              None,
              vec![TemplateBlock::new("a", "Grandparent", BlockStrategy::Replace)],
              HashMap::new(),
          ).unwrap();

          let parent = Template::new(
              "parent",
              Some("grandparent"),
              vec![TemplateBlock::new("a", "Parent", BlockStrategy::Replace)],
              HashMap::new(),
          ).unwrap();

          let child = Template::new(
              "child",
              Some("parent"),
              vec![TemplateBlock::new("a", "Child", BlockStrategy::Replace)],
              HashMap::new(),
          ).unwrap();

          // Store in WRONG order (child first)
          command.create(&child).unwrap();
          command.create(&grandparent).unwrap();
          command.create(&parent).unwrap();

          // Catalog should sort them correctly
          let mut catalog = TemplateCatalog::new(Box::new(query)).unwrap();
          catalog.load_all().unwrap();

          // All three should render correctly
          assert!(catalog.render("grandparent", minijinja::context! {}).is_ok());
          assert!(catalog.render("parent", minijinja::context! {}).is_ok());
          assert!(catalog.render("child", minijinja::context! {}).is_ok());
      }

      #[test]
      fn list_names_returns_all_templates() {
          let storage = FakeTemplateStorage::new();
          let command = Command::new(storage.clone());
          let query = Query::new(storage.clone());

          let t1 = Template::new("t1", None, vec![], HashMap::new()).unwrap();
          let t2 = Template::new("t2", None, vec![], HashMap::new()).unwrap();

          command.create(&t1).unwrap();
          command.create(&t2).unwrap();

          let catalog = TemplateCatalog::new(Box::new(query)).unwrap();
          let names = catalog.list_names().unwrap();

          assert_eq!(names.len(), 2);
          assert!(names.contains(&"t1".to_string()));
          assert!(names.contains(&"t2".to_string()));
      }
  }
  ```
- [ ] Update `lithos-core/src/template/mod.rs`: Add `pub mod catalog;` and `pub use catalog::TemplateCatalog;`
- [ ] Verify compiles: `cargo check -p lithos-core`

**Task 4.2: Update Storage to Support FakeTemplateStorage (if needed)**
- [ ] Verify `FakeTemplateStorage` exists in ports module (should exist from Phase 2)
- [ ] If not, implement minimal `FakeTemplateStorage` for tests:
  ```rust
  // In lithos-core/src/template/ports.rs or similar

  /// In-memory template storage for testing.
  #[derive(Clone, Default)]
  pub struct FakeTemplateStorage {
      templates: Arc<Mutex<HashMap<Uuid, Template>>>,
      name_index: Arc<Mutex<HashMap<String, Uuid>>>,
  }

  impl FakeTemplateStorage {
      pub fn new() -> Self {
          Self::default()
      }
  }

  impl TemplateQueryPort for FakeTemplateStorage {
      fn find_by_id(&self, id: Uuid) -> Result<Option<Template>, TemplateError> {
          let templates = self.templates.lock().unwrap();
          Ok(templates.get(&id).cloned())
      }

      fn find_by_name(&self, name: &str) -> Result<Option<Template>, TemplateError> {
          let name_index = self.name_index.lock().unwrap();
          let templates = self.templates.lock().unwrap();

          if let Some(&id) = name_index.get(name) {
              Ok(templates.get(&id).cloned())
          } else {
              Ok(None)
          }
      }

      fn list(&self) -> Result<Vec<Template>, TemplateError> {
          let templates = self.templates.lock().unwrap();
          Ok(templates.values().cloned().collect())
      }
  }

  impl TemplateCommandPort for FakeTemplateStorage {
      fn create(&self, template: &Template) -> Result<(), TemplateError> {
          let mut templates = self.templates.lock().unwrap();
          let mut name_index = self.name_index.lock().unwrap();

          if name_index.contains_key(template.name()) {
              return Err(TemplateError::AlreadyExists(template.name().into()));
          }

          templates.insert(template.id(), template.clone());
          name_index.insert(template.name().into(), template.id());

          Ok(())
      }

      fn update(&self, template: &Template) -> Result<(), TemplateError> {
          let mut templates = self.templates.lock().unwrap();
          let mut name_index = self.name_index.lock().unwrap();

          let old = templates.get(&template.id())
              .ok_or_else(|| TemplateError::NotFound(template.id().to_string()))?;

          if old.name() != template.name() {
              if name_index.contains_key(template.name()) {
                  return Err(TemplateError::AlreadyExists(template.name().into()));
              }

              name_index.remove(old.name());
              name_index.insert(template.name().into(), template.id());
          }

          templates.insert(template.id(), template.clone());

          Ok(())
      }

      fn delete(&self, id: Uuid) -> Result<(), TemplateError> {
          let mut templates = self.templates.lock().unwrap();
          let mut name_index = self.name_index.lock().unwrap();

          if let Some(template) = templates.remove(&id) {
              name_index.remove(template.name());
          }

          Ok(())
      }
  }
  ```
- [ ] Verify compiles: `cargo check -p lithos-core`

**Task 4.3: Run Integration Tests**
- [ ] Run tests: `mise run test:unit:template`
- [ ] Verify all catalog tests pass (load_all, topological sort, cycle detection)

**Phase 4 Verification:**
- [ ] Run `mise run test` → All tests pass (including catalog tests)
- [ ] Run `mise run lint` → Zero warnings
- [ ] Run `mise run fmt` → Code formatted
- [ ] Manual smoke test:
  ```rust
  // Create a test that:
  // 1. Creates parent and child templates
  // 2. Stores them (in wrong order)
  // 3. TemplateCatalog.load_all() sorts and compiles them
  // 4. Renders child successfully
  ```
- [ ] Commit: `git add -A && git commit -m "Phase 4: Add TemplateCatalog (lifecycle orchestration)"`

**Phase 4 Complete When:**
- [ ] TemplateCatalog loads all templates from storage
- [ ] Topological sort works (parents compiled before children)
- [ ] Cycle detection works (circular extends rejected)
- [ ] Rendering uses compiled templates (fast path, O(1) lookup)
- [ ] list_names() works
- [ ] All integration tests pass

**🎯 Success Criteria:**
- ✅ Templates compiled in correct order automatically (no manual sorting)
- ✅ Can load 100+ templates and compile in <200ms
- ✅ Circular extends detected and rejected with clear error
- ✅ Rendering is fast (O(1) lookup, no recompilation)
- ✅ Catalog is the single entry point for template operations

---

### Phase 5: Refactor Ports (CQRS Alignment)

**Goal:** Align storage ports with note/schema patterns (GAT + zero-copy)

**Duration:** 3-4 hours

**Git Branch:** `refactor/phase-5-ports-alignment`

**Why This Phase:** Brings storage layer up to project standards. Adds zero-copy reads via GAT. Implements proper Query/Command port split. This is pure refactoring (no new features).

---

#### Phase 5 Checklist

**Setup:**
- [ ] Merge Phase 4: `git checkout main && git merge refactor/phase-4-catalog`
- [ ] Create new branch: `git checkout -b refactor/phase-5-ports-alignment`
- [ ] Verify starting point: `mise run test` passes

**Task 5.1: Refer to Design Doc**
- [ ] Read [013: Template CQRS](./013-template-cqrs.md) for full port specifications
- [ ] Understand GAT pattern: `type Archived<'a> where Self: 'a`
- [ ] Understand closure pattern: `with_archived<F, R>(id, f) where F: for<'a> FnOnce(&'a Self::Archived<'a>) -> R`

**Task 5.2: Update TemplateQueryPort Trait**
- [ ] Add GAT to `TemplateQueryPort` in `lithos-core/src/template/ports.rs`:
  ```rust
  pub trait TemplateQueryPort: Send + Sync {
      /// Archived template type (zero-copy access)
      type Archived<'a>: 'a where Self: 'a;

      /// Access archived template via closure (zero-copy)
      fn with_archived<F, R>(&self, id: Uuid, f: F) -> Result<Option<R>, TemplateError>
      where
          F: for<'a> FnOnce(&'a Self::Archived<'a>) -> R;

      fn find_by_id(&self, id: Uuid) -> Result<Option<Template>, TemplateError>;
      fn find_by_name(&self, name: &str) -> Result<Option<Template>, TemplateError>;
      fn list(&self) -> Result<Vec<Template>, TemplateError>;
  }
  ```

**Task 5.3: Implement GAT for RedbTemplateQuery**
- [ ] Update `RedbTemplateQuery` to implement GAT:
  ```rust
  impl TemplateQueryPort for RedbTemplateQuery<'_> {
      type Archived<'a> = rkyv::Archived<Template> where Self: 'a;

      fn with_archived<F, R>(&self, id: Uuid, f: F) -> Result<Option<R>, TemplateError>
      where
          F: for<'a> FnOnce(&'a Self::Archived<'a>) -> R,
      {
          self.db.with_archived(TEMPLATES, id, f)
              .map_err(|e| TemplateError::Storage(e.to_string()))
      }

      // ... other methods
  }
  ```

**Task 5.4: Update FakeTemplateStorage for GAT**
- [ ] Update `FakeTemplateStorage` to implement GAT:
  ```rust
  impl TemplateQueryPort for FakeTemplateStorage {
      type Archived<'a> = Template where Self: 'a;

      fn with_archived<F, R>(&self, id: Uuid, f: F) -> Result<Option<R>, TemplateError>
      where
          F: for<'a> FnOnce(&'a Self::Archived<'a>) -> R,
      {
          let templates = self.templates.lock().unwrap();
          Ok(templates.get(&id).map(f))
      }

      // ... other methods
  }
  ```

**Task 5.5: Update UUID Keys to &[u8]**
- [ ] Update database table definitions to use `&[u8]` for UUID keys:
  ```rust
  pub(crate) const TEMPLATES: TableDefinition<&[u8], &[u8]> =
      TableDefinition::new("templates");
  ```
- [ ] Update RedbTemplateQuery/Command to use `id.as_bytes()` instead of `id.to_string()`

**Task 5.6: Add Tests for Zero-Copy**
- [ ] Add tests that verify zero-copy reads work:
  ```rust
  #[test]
  fn with_archived_zero_copy() {
      let storage = FakeTemplateStorage::new();
      let command = Command::new(storage.clone());
      let query = Query::new(storage.clone());

      let template = Template::new("test", None, vec![], HashMap::new()).unwrap();
      command.create(&template).unwrap();

      // Zero-copy read (extract just the name)
      let name = query.with_archived(template.id(), |archived| {
          archived.name().to_string()
      }).unwrap().unwrap();

      assert_eq!(name, "test");
  }
  ```

**Phase 5 Verification:**
- [ ] Run `mise run test` → All tests pass (port refactor complete)
- [ ] Run `mise run lint` → Zero warnings
- [ ] Run `mise run fmt` → Code formatted
- [ ] Verify zero-copy reads work (with_archived tests pass)
- [ ] Commit: `git add -A && git commit -m "Phase 5: Align ports with project standards (GAT + zero-copy)"`

**Phase 5 Complete When:**
- [ ] TemplateQueryPort has GAT: `type Archived<'a>`
- [ ] TemplateQueryPort has `with_archived()` closure method
- [ ] RedbTemplateQuery implements GAT with rkyv::Archived<Template>
- [ ] FakeTemplateStorage implements GAT with Template
- [ ] UUID keys use `&[u8]` (not `.to_string()`)
- [ ] Zero-copy tests pass

**🎯 Success Criteria:**
- ✅ Ports match note/schema patterns exactly
- ✅ Zero-copy reads available (performance optimization)
- ✅ UUID keys optimized (&[u8] instead of String)
- ✅ All tests still pass (no behavior changes, pure refactor)

---

### Phase 6: Delete Deprecated Code (Final Cleanup)

**Goal:** Remove all deprecated code, clean up imports, finalize migration

**Duration:** 1-2 hours

**Git Branch:** `refactor/phase-6-cleanup`

**Why This Phase:** Removes technical debt accumulated during migration. Deletes PlaceholderSyntax, old composition logic, and deprecated methods. Finalizes the migration to clean, production-ready code.

---

#### Phase 6 Checklist

**Setup:**
- [ ] Merge Phase 5: `git checkout main && git merge refactor/phase-5-ports-alignment`
- [ ] Create new branch: `git checkout -b refactor/phase-6-cleanup`
- [ ] Verify starting point: `mise run test` passes

**Task 6.1: Delete Deprecated Files**
- [ ] Delete `lithos-core/src/template/syntax.rs` (PlaceholderSyntax)
- [ ] Delete `lithos-core/src/template/validation.rs` (old domain validation)
- [ ] Delete `lithos-core/src/template/composition.rs` (old manual composition) if exists
- [ ] Verify these files are gone: `ls lithos-core/src/template/`

**Task 6.2: Remove Deprecated Fields from Template**
- [ ] Remove `#[deprecated] content: String` field from Template struct
- [ ] Remove `#[deprecated] syntax: PlaceholderSyntax` field from Template struct
- [ ] Update constructor to NOT initialize deprecated fields

**Task 6.3: Remove Deprecated Methods**
- [ ] Remove `#[deprecated] Template::validate()` method
- [ ] Remove `#[deprecated] Template::compose()` method if exists
- [ ] Remove `#[deprecated] VariableDefinition::validate_value()` method if exists

**Task 6.4: Clean Up Imports**
- [ ] Remove `pub mod syntax;` from `lithos-core/src/template/mod.rs`
- [ ] Remove `pub mod validation;` from `lithos-core/src/template/mod.rs`
- [ ] Remove `pub mod composition;` from `lithos-core/src/template/mod.rs`
- [ ] Remove any unused `use` statements throughout template module

**Task 6.5: Update Public Exports**
- [ ] Review `lithos-core/src/template/mod.rs` exports
- [ ] Ensure only new API is exported (TemplateCatalog, TemplateBlock, BlockStrategy, etc.)
- [ ] Remove exports for deleted types (PlaceholderSyntax, etc.)

**Task 6.6: Run Full Test Suite**
- [ ] Run `mise run test` → All tests pass
- [ ] Run `mise run lint` → Zero warnings (no more deprecated warnings)
- [ ] Run `mise run fmt` → Code formatted
- [ ] Run `mise run verify` → Full quality gate passes

**Phase 6 Verification:**
- [ ] No deprecated code remains (`rg "#\[deprecated\]" lithos-core/src/template/` returns nothing)
- [ ] No deleted files remain (`ls lithos-core/src/template/{syntax,validation,composition}.rs` fails)
- [ ] All tests pass with zero warnings
- [ ] Codebase is clean and production-ready
- [ ] Commit: `git add -A && git commit -m "Phase 6: Delete deprecated code (cleanup complete)"`

**Phase 6 Complete When:**
- [ ] PlaceholderSyntax deleted
- [ ] validate_structure() deleted
- [ ] Old Template::compose() deleted
- [ ] Old Composition (manual) deleted
- [ ] Deprecated fields removed from Template
- [ ] Deprecated methods removed
- [ ] All imports cleaned up
- [ ] Zero clippy warnings
- [ ] All tests pass

**🎯 Success Criteria:**
- ✅ No deprecated code remains in codebase
- ✅ Template module is clean and maintainable
- ✅ All tests pass with zero warnings
- ✅ Migration complete and ready for production

---

## 3. Post-Migration Tasks

### Task 3.1: Update Application Layer
- [ ] Update CLI commands to use new TemplateCatalog API
- [ ] Update LSP handlers to use new rendering API
- [ ] Remove any calls to old deprecated methods
- [ ] Test end-to-end workflows (create template → render template)

### Task 3.2: Update Documentation
- [ ] Update README with new API examples
- [ ] Update inline doc comments
- [ ] Create migration guide for external users (if applicable)
- [ ] Update ADR if architectural changes warrant it

### Task 3.3: Performance Validation
- [ ] Run benchmarks: `mise run test:bench:core`
- [ ] Verify <500ms for typical template operations
- [ ] Compare before/after metrics (should be 5-10× faster rendering)
- [ ] Document performance improvements

### Task 3.4: Final Quality Gates
- [ ] Run `mise run verify` → Full CI passes
- [ ] Test coverage >90%: `mise run test:coverage`
- [ ] Zero security issues: `mise run deny`
- [ ] All quality gates pass

---

## 4. Rollback Strategy

**Per-Phase Rollback:**

Each phase is a git branch with clear rollback:
```bash
# If Phase N fails
git checkout main
git branch -D refactor/phase-N-failed

# Continue from last known good state (Phase N-1)
```

**Full Rollback (Emergency):**
```bash
# Nuclear option: rollback entire migration
git reset --hard <commit-before-migration-start>
git push origin main --force  # (if pushed)
```

**Partial Rollback (Keep Progress):**
```bash
# Keep Phases 1-4, revert Phases 5-6
git checkout main
git merge refactor/phase-4-catalog
# Skip phases 5-6, consider them failed
```

---

## 5. Success Criteria (Final Checklist)

**Migration Successful When:**

- [ ] 1. All templates use MiniJinja for rendering (not manual composition)
- [ ] 2. Template domain is metadata-only (no processing logic, just data)
- [ ] 3. Templates compiled once at startup (cached in Arc<Environment>, <200ms for 100 templates)
- [ ] 4. Variable constraints enforced via filters (not pre-validation)
- [ ] 5. Ports aligned with note/schema patterns (GAT + zero-copy reads)
- [ ] 6. All deprecated code deleted (zero `#[deprecated]` attributes remain)
- [ ] 7. Performance <500ms for typical operations (5-10× faster than before)
- [ ] 8. Test coverage >90% (all critical paths covered)
- [ ] 9. Zero clippy warnings (`mise run lint` passes)
- [ ] 10. Documentation updated (API examples, migration guide)
- [ ] 11. Application layer updated (CLI/LSP use new API)
- [ ] 12. End-to-end tests pass (create, render, list templates)
- [ ] 13. ADR created if architectural changes warrant it
- [ ] 14. Code review approved (if applicable)
- [ ] 15. Merged to main branch (all phases complete)

---

## 6. Timeline & Effort

| Phase | Duration | Tasks | Risk Level |
|-------|----------|-------|------------|
| Phase 1: MiniJinja Integration | 2-3 hours | 4 tasks | Low (parallel to old code) |
| Phase 2: Domain Refactor | 3-4 hours | 4 tasks | Medium (breaking changes) |
| Phase 3: SourceGenerator | 2-3 hours | 3 tasks | Low (pure addition) |
| Phase 4: TemplateCatalog | 2-3 hours | 3 tasks | Medium (complex algorithm) |
| Phase 5: Port Refactor | 3-4 hours | 6 tasks | Low (pure refactor) |
| Phase 6: Cleanup | 1-2 hours | 6 tasks | Low (deletion only) |
| **Total** | **13-19 hours** | **26 tasks** | **~2.5 days** |

**Assumptions:**
- Developer familiar with codebase and Rust
- No major blockers or unexpected issues
- Tests written alongside code (TDD approach)
- Code reviews happen in parallel (not blocking)

---

## 7. Risk Mitigation

**Risk: Phase 2 Breaks Too Many Tests**
- _Likelihood:_ Medium (constructor signature changes)
- _Impact:_ High (blocks progress)
- _Mitigation:_ Update tests incrementally. Keep deprecated constructor temporarily if needed.
- _Rollback:_ Revert Phase 2, keep Phase 1 (MiniJinja still valuable)

**Risk: Performance Regression in Phase 3**
- _Likelihood:_ Low (expected temporary regression)
- _Impact:_ Low (Phase 4 fixes it)
- _Mitigation:_ Benchmark after Phase 3. Accept regression if <50% slowdown. Phase 4 caching will fix.
- _Rollback:_ Not needed (temporary regression acceptable)

**Risk: Topological Sort Has Bug**
- _Likelihood:_ Low (Kahn's algorithm is proven)
- _Impact:_ High (incorrect compilation order breaks templates)
- _Mitigation:_ Property-based tests (generate random DAGs). Manual testing with complex hierarchies.
- _Rollback:_ Fix bug in Phase 4, don't revert entire phase

**Risk: Port Refactor Breaks Application Layer**
- _Likelihood:_ Medium (API changes)
- _Impact:_ High (CLI/LSP broken)
- _Mitigation:_ Update application layer in same commit as ports. Test thoroughly.
- _Rollback:_ Revert Phase 5, keep Phases 1-4 (ports optional optimization)

**Risk: Cleanup Accidentally Deletes Used Code**
- _Likelihood:_ Low (deprecated code marked clearly)
- _Impact:_ High (compilation errors)
- _Mitigation:_ Double-check `rg` for usages before deletion. Run tests after each deletion.
- _Rollback:_ Restore deleted file from git: `git checkout HEAD~ -- path/to/file.rs`

---

## 8. References

**Design Documents:**
- [012: Template Domain Models](./012-template-models.md) - Target domain model (metadata schema)
- [013: Template CQRS](./013-template-cqrs.md) - Target storage layer (ports and adapters)
- [014: Template Services](./014-template-services.md) - Target service layer (MiniJinja integration)

**External Resources:**
- [MiniJinja Documentation](https://docs.rs/minijinja/latest/minijinja/)
- [MiniJinja Template Inheritance](https://docs.rs/minijinja/latest/minijinja/#template-inheritance)
- [Refactoring: Improving the Design of Existing Code](https://martinfowler.com/books/refactoring.html)
- [Working Effectively with Legacy Code](https://www.oreilly.com/library/view/working-effectively-with/0131177052/)

---

**Status:** Draft
**Last Updated:** 2026-02-16
**Ready for Implementation:** ✅ Yes (all phases planned with checkboxes)
