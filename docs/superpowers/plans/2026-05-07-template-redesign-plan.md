# Template Module Redesign Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Redesign the Template bounded context to be a file-centric, query-aware system with a rigid typestate compilation pipeline, delegating Markdown generation to `minijinja`.

**Architecture:** We will implement a strict typestate processor (`Discovery` -> `Comparison` -> `Parsed` -> `Construction`) to ensure templates are cached via `rkyv` and only re-parsed when file contents change. The rendering engine will be decoupled behind a `TemplateRuntime` object that handles structured queries and interactive prompts. The existing parser refactoring is deferred; we will use a local `pulldown-cmark` extractor for frontmatter.

**Tech Stack:** Rust, `minijinja`, `rkyv`, `redb`, `thiserror`, `pulldown-cmark`

---

## File Structure

The existing `lithos-core/src/template/` directory contains CQRS abstractions that we are abandoning. The new structure:

**Core Domain:**
- `src/template/aggregate.rs` -> Defines `Template`, `TemplateName`, `InputName`, `TemplateFrontmatter`, `InputSpec` (no engine coupling).
- `src/template/cache.rs` -> Defines `RawTemplateView`, `HashRecord`, and the `redb` table definitions (`RAW_TEMPLATE_VIEWS`, `TEMPLATE_ID_BY_PATH`, `TEMPLATES`).
- `src/template/processor.rs` -> Defines the rigid typestate pipeline (`TemplateProcessor<Stage, Status>`) for ingestion and caching.

**Parsing:**
- `src/template/parser.rs` -> Local `pulldown-cmark` extractor to separate frontmatter from body without rebuilding an AST.

**Engine & Runtime:**
- `src/template/engine/minijinja.rs` -> The *only* place `minijinja` is imported. Handles environment setup and rendering.
- `src/template/runtime/query.rs` -> Structured `QueryBuilder` and `QueryFilter`.
- `src/template/runtime/interact.rs` -> Abstraction for TUI/CLI interactive hosts.
- `src/template/runtime/li.rs` -> The `TemplateRuntime` context object injected into Jinja.

---

## Task 1: Clean Up Abandoned Abstractions

We must remove the CQRS architecture elements from the existing template context.

**Files:**
- Modify: `lithos-core/src/template/mod.rs`
- Delete: `lithos-core/src/template/command.rs`
- Delete: `lithos-core/src/template/query.rs`
- Delete: `lithos-core/src/template/events.rs`
- Delete: `lithos-core/src/template/ports.rs`
- Delete: `lithos-core/src/template/block.rs`

- [ ] **Step 1: Delete abandoned module files**

```bash
rm lithos-core/src/template/command.rs
rm lithos-core/src/template/query.rs
rm lithos-core/src/template/events.rs
rm lithos-core/src/template/ports.rs
rm lithos-core/src/template/block.rs
```

- [ ] **Step 2: Update `mod.rs`**

```rust
//! Template bounded context models.
//!
//! #![allow(clippy::module_name_repetitions, reason = "Namespaced types")]
//! #![allow(clippy::pub_use, reason = "Re-exporting for convenience")]

pub mod aggregate;
pub mod cache;
pub mod engine;
pub mod error;
pub mod parser;
pub mod processor;
pub mod runtime;

pub use aggregate::{InputName, InputSpec, Template, TemplateFrontmatter, TemplateName};
```

- [ ] **Step 3: Run verify command to confirm missing imports (expected failure)**

Run: `mise run check` or `cargo check`
Expected: Build errors related to missing modules and broken imports in `aggregate.rs` and other internal template files. (This is fine, we will rewrite `aggregate.rs` next).

- [ ] **Step 4: Commit**

```bash
git add lithos-core/src/template
git commit -m "refactor(template): remove obsolete CQRS and block inheritance abstractions"
```

---

## Task 2: Define the Core Aggregate and Input Models

Rewrite `aggregate.rs` to reflect the new, decoupled `Template` and `InputSpec` design.

**Files:**
- Modify: `lithos-core/src/template/aggregate.rs`

- [ ] **Step 1: Write tests for Domain Identity Constraints**

```rust
// In lithos-core/src/template/aggregate.rs (or an adjacent test file if preferred)
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn valid_template_names() {
        assert!(TemplateName::try_from("daily-note").is_ok());
        assert!(TemplateName::try_from("MyTemplate_123").is_ok());
    }

    #[test]
    fn invalid_template_names() {
        assert!(TemplateName::try_from("template with spaces").is_err());
        assert!(TemplateName::try_from("template!").is_err());
    }
}
```

- [ ] **Step 2: Run test to verify it fails**

Run: `cargo test --package lithos-core --lib template::aggregate::tests`
Expected: FAIL (due to missing definitions or old definitions).

- [ ] **Step 3: Implement Aggregate and Value Types**

```rust
// In lithos-core/src/template/aggregate.rs
use rkyv::{Archive, Deserialize as RkyvDeserialize, Serialize as RkyvSerialize};
use std::collections::HashMap;
use std::sync::LazyLock;
use regex::Regex;
use crate::support::{UuidV7, UuidV7Error};
use crate::template::error::TemplateError;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Archive, RkyvSerialize, RkyvDeserialize)]
#[rkyv(derive(Debug, Hash, PartialEq, Eq, PartialOrd, Ord))]
pub struct TemplateId(UuidV7);

impl TemplateId {
    pub fn new() -> Self { Self(UuidV7::new()) }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord, serde::Serialize, serde::Deserialize, Archive, RkyvSerialize, RkyvDeserialize)]
#[rkyv(derive(Debug, Hash, PartialEq, Eq))]
pub struct TemplateName(pub Box<str>);

impl TryFrom<&str> for TemplateName {
    type Error = TemplateError;
    fn try_from(value: &str) -> Result<Self, Self::Error> {
        static RE: LazyLock<Result<Regex, regex::Error>> = LazyLock::new(|| Regex::new("^[a-zA-Z0-9_-]+$"));
        let re = RE.as_ref().map_err(|e| TemplateError::ValidationFailed(e.to_string()))?;
        if !re.is_match(value) { return Err(TemplateError::InvalidTemplateName(value.to_owned())); }
        Ok(Self(value.into()))
    }
}
impl TemplateName {
    pub fn as_str(&self) -> &str { &self.0 }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord, serde::Serialize, serde::Deserialize, Archive, RkyvSerialize, RkyvDeserialize)]
#[rkyv(derive(Debug, Hash, PartialEq, Eq))]
pub struct InputName(pub Box<str>);

impl TryFrom<&str> for InputName {
    type Error = TemplateError;
    fn try_from(value: &str) -> Result<Self, Self::Error> {
        static RE: LazyLock<Result<Regex, regex::Error>> = LazyLock::new(|| Regex::new("^[a-zA-Z_][a-zA-Z0-9_]*$"));
        let re = RE.as_ref().map_err(|e| TemplateError::ValidationFailed(e.to_string()))?;
        if !re.is_match(value) { return Err(TemplateError::InvalidInputName(value.to_owned())); }
        Ok(Self(value.into()))
    }
}

// In the future, this will be SchemaName
pub type SchemaNameAlias = String;

#[derive(Debug, Clone, Archive, RkyvSerialize, RkyvDeserialize, PartialEq)]
#[rkyv(derive(Debug))]
pub struct Template {
    pub id: TemplateId,
    pub name: TemplateName,
    pub schemas: Vec<SchemaNameAlias>,
    pub inputs: HashMap<InputName, InputSpec>,
    pub body: String,
}

#[derive(serde::Deserialize, Debug, Default, Clone, PartialEq)]
pub struct TemplateFrontmatter {
    #[serde(default)]
    pub schemas: Vec<SchemaNameAlias>,
    #[serde(default)]
    pub inputs: HashMap<InputName, InputSpec>,
}

#[derive(Debug, Clone, Archive, RkyvSerialize, RkyvDeserialize, serde::Deserialize, PartialEq)]
#[rkyv(derive(Debug))]
#[serde(tag = "type")]
pub enum InputSpec {
    Text { default: Option<String>, #[serde(default)] required: bool },
    Select { options: Vec<String>, #[serde(default)] required: bool },
}
```

- [ ] **Step 4: Run test to verify it passes**

Run: `cargo test --package lithos-core --lib template::aggregate::tests`
Expected: PASS

- [ ] **Step 5: Commit**

```bash
git add lithos-core/src/template/aggregate.rs
git commit -m "feat(template): define zero-copy aggregate and input spec models"
```

---

## Task 3: Implement Template Parsing Boundary

Create a local `pulldown-cmark` parser to split frontmatter and body.

**Files:**
- Create: `lithos-core/src/template/parser.rs`

- [ ] **Step 1: Write test for splitting templates**

```rust
// In lithos-core/src/template/parser.rs
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn splits_frontmatter_from_body() {
        let source = "---\nschemas: ['project']\n---\n# Title\nBody";
        let (fm, body) = extract_template_asset(source).unwrap();
        assert_eq!(fm.schemas, vec!["project".to_string()]);
        assert_eq!(body, "# Title\nBody");
    }

    #[test]
    fn handles_no_frontmatter() {
        let source = "# Title\nBody";
        let (fm, body) = extract_template_asset(source).unwrap();
        assert!(fm.schemas.is_empty());
        assert_eq!(body, "# Title\nBody");
    }
}
```

- [ ] **Step 2: Run test to verify it fails**

Run: `cargo test --package lithos-core --lib template::parser::tests`
Expected: FAIL

- [ ] **Step 3: Implement `extract_template_asset`**

```rust
// In lithos-core/src/template/parser.rs
use pulldown_cmark::{Event, Options, Parser, Tag, TagEnd};
use crate::template::aggregate::TemplateFrontmatter;
use crate::template::error::TemplateError;

pub fn extract_template_asset(source: &str) -> Result<(TemplateFrontmatter, String), TemplateError> {
    let mut options = Options::empty();
    options.insert(Options::ENABLE_YAML_STYLE_METADATA_BLOCKS);

    let parser = Parser::new_ext(source, options);
    let mut offset_iter = parser.into_offset_iter();

    let mut frontmatter_text = String::new();
    let mut body_start_offset = 0;

    while let Some((event, range)) = offset_iter.next() {
        match event {
            Event::Start(Tag::MetadataBlock(_)) => {}
            Event::Text(text) => {
                frontmatter_text.push_str(&text);
            }
            Event::End(TagEnd::MetadataBlock(_)) => {
                body_start_offset = range.end;
                break;
            }
            _ => {
                break; // Not frontmatter
            }
        }
    }

    let frontmatter = if frontmatter_text.is_empty() {
        TemplateFrontmatter::default()
    } else {
        serde_yaml::from_str(&frontmatter_text)
            .map_err(|e| TemplateError::ValidationFailed(format!("YAML parse error: {}", e)))?
    };

    let raw_body = source[body_start_offset..].trim_start().to_string();

    Ok((frontmatter, raw_body))
}
```

- [ ] **Step 4: Run test to verify it passes**

Run: `cargo test --package lithos-core --lib template::parser::tests`
Expected: PASS

- [ ] **Step 5: Commit**

```bash
git add lithos-core/src/template/parser.rs
git commit -m "feat(template): add frontmatter and body extraction via pulldown-cmark"
```

---

## Task 4: Implement Cache Tables and Views

Define the `redb` tables and the `RawTemplateView` used by the processor pipeline.

**Files:**
- Create: `lithos-core/src/template/cache.rs`

- [ ] **Step 1: Write test for HashRecord and View**

```rust
// In lithos-core/src/template/cache.rs
#[cfg(test)]
mod tests {
    use super::*;
    use crate::support::hash::Blake3Hash;
    use std::time::SystemTime;

    #[test]
    fn view_detects_timestamp_match() {
        let time = SystemTime::now();
        let view = RawTemplateView {
            created_at: Some(time),
            modified_at: Some(time),
            content_hash: Blake3Hash::compute(b"content"),
        };
        assert!(view.is_timestamp_match(Some(time), Some(time)));
    }
}
```

- [ ] **Step 2: Run test to verify it fails**

Run: `cargo test --package lithos-core --lib template::cache::tests`
Expected: FAIL

- [ ] **Step 3: Implement `cache.rs`**

```rust
// In lithos-core/src/template/cache.rs
use rkyv::{Archive, Deserialize, Serialize};
use std::time::SystemTime;
use crate::support::hash::Blake3Hash;
use crate::fs::FileInfo;

#[derive(Debug, Clone, Archive, Serialize, Deserialize)]
#[rkyv(derive(Debug))]
pub struct RawTemplateView {
    #[rkyv(with = crate::ser::SystemTimeAsI64)]
    pub created_at: Option<SystemTime>,
    #[rkyv(with = crate::ser::SystemTimeAsI64)]
    pub modified_at: Option<SystemTime>,
    pub content_hash: Blake3Hash,
}

impl RawTemplateView {
    pub fn new(info: &FileInfo, content_hash: Blake3Hash) -> Self {
        Self {
            created_at: info.created_at(),
            modified_at: info.modified_at(),
            content_hash,
        }
    }

    pub fn is_timestamp_match(&self, created_at: Option<SystemTime>, modified_at: Option<SystemTime>) -> bool {
        self.created_at == created_at && self.modified_at == modified_at
    }

    pub fn is_content_match(&self, hash: &Blake3Hash) -> bool {
        &self.content_hash == hash
    }
}

pub mod db_table {
    use redb::TableDefinition;

    pub const RAW_TEMPLATE_VIEWS: TableDefinition<&str, &[u8]> = TableDefinition::new("raw_template_views");
    pub const TEMPLATE_ID_BY_PATH: TableDefinition<&str, &[u8]> = TableDefinition::new("template_id_by_path");
    pub const TEMPLATES: TableDefinition<&[u8], &[u8]> = TableDefinition::new("templates");
}
```

- [ ] **Step 4: Run test to verify it passes**

Run: `cargo test --package lithos-core --lib template::cache::tests`
Expected: PASS

- [ ] **Step 5: Commit**

```bash
git add lithos-core/src/template/cache.rs
git commit -m "feat(template): add redb cache tables and RawTemplateView"
```

---

## Task 5: Implement Typestate Processor Pipeline

Implement the rigid compile-time state machine for template ingestion.

**Files:**
- Create: `lithos-core/src/template/processor.rs`

- [ ] **Step 1: Write test for missing file transition**

```rust
// In lithos-core/src/template/processor.rs
#[cfg(test)]
mod tests {
    use super::*;
    use crate::fs::FileInfo;
    use std::time::SystemTime;

    #[test]
    fn new_pipeline_handles_missing() {
        let info = FileInfo::new(100, Some(SystemTime::now()), Some(SystemTime::now()), false);
        let branch = TemplateProcessor::<Discovery, Unknown>::new().discover_missing(info);

        match branch {
            ComparisonBranch::Missing(p) => {
                // Success: Pipeline navigated to Missing state
            }
            _ => panic!("Expected Missing branch"),
        }
    }
}
```

- [ ] **Step 2: Run test to verify it fails**

Run: `cargo test --package lithos-core --lib template::processor::tests`
Expected: FAIL

- [ ] **Step 3: Implement the Pipeline**

```rust
// In lithos-core/src/template/processor.rs
use std::marker::PhantomData;
use crate::fs::{FileInfo, FsReader, RelativePath};
use crate::support::hash::Blake3Hash;
use crate::template::cache::RawTemplateView;
use crate::template::aggregate::{Template, TemplateName, TemplateId};
use crate::template::parser::extract_template_asset;
use crate::template::error::TemplateError;

pub struct TemplateProcessor<P, S> {
    status: S,
    _stage: PhantomData<P>,
}

impl<P, S> TemplateProcessor<P, S> {
    pub(crate) fn transition<NP, NS>(_stage: NP, status: NS) -> TemplateProcessor<NP, NS> {
        TemplateProcessor { status, _stage: PhantomData }
    }
}

// Stages
pub struct Discovery;
pub struct Comparison;
pub struct Parsed;
pub struct Refresh;
pub struct Construction;
pub struct Completed;

// Statuses
pub struct Unknown;
pub struct Missing { info: FileInfo }
pub struct Present { info: FileInfo, view: RawTemplateView }
pub struct Suspect { info: FileInfo, view: RawTemplateView, content: String }
pub struct Stale { info: FileInfo, content: String, content_hash: Blake3Hash, view: RawTemplateView }

pub struct StaleTimestamps { info: FileInfo, view: RawTemplateView }
pub struct New { raw_body: String, frontmatter: crate::template::aggregate::TemplateFrontmatter, content_hash: Blake3Hash, info: FileInfo }
pub struct Changed { raw_body: String, frontmatter: crate::template::aggregate::TemplateFrontmatter, content_hash: Blake3Hash, info: FileInfo }
pub struct Fresh { view: RawTemplateView } // Needs fetch from db
pub struct Ready { template: Template }

pub enum ComparisonBranch {
    Missing(TemplateProcessor<Parsed, Missing>),
    Present(TemplateProcessor<Comparison, Present>),
}

impl TemplateProcessor<Discovery, Unknown> {
    pub fn new() -> Self { Self { status: Unknown, _stage: PhantomData } }

    // In a real system, this would query DB. For the pipeline, we provide manual branch methods.
    pub fn discover_missing(self, info: FileInfo) -> ComparisonBranch {
        ComparisonBranch::Missing(Self::transition(Parsed, Missing { info }))
    }

    pub fn discover_present(self, info: FileInfo, view: RawTemplateView) -> ComparisonBranch {
        ComparisonBranch::Present(Self::transition(Comparison, Present { info, view }))
    }
}

pub enum TimestampBranch {
    Match(TemplateProcessor<Construction, Fresh>),
    Mismatch(TemplateProcessor<Comparison, Suspect>),
}

impl TemplateProcessor<Comparison, Present> {
    pub fn check_timestamps(self, source: &FsReader, path: &std::path::Path) -> Result<TimestampBranch, TemplateError> {
        if self.status.view.is_timestamp_match(self.status.info.created_at(), self.status.info.modified_at()) {
            Ok(TimestampBranch::Match(Self::transition(Construction, Fresh { view: self.status.view })))
        } else {
            let content = source.read_to_string(path).map_err(|e| TemplateError::ValidationFailed(e.to_string()))?;
            Ok(TimestampBranch::Mismatch(Self::transition(Comparison, Suspect { info: self.status.info, view: self.status.view, content })))
        }
    }
}

pub enum ContentBranch {
    Match(TemplateProcessor<Refresh, StaleTimestamps>),
    Mismatch(TemplateProcessor<Parsed, Stale>),
}

impl TemplateProcessor<Comparison, Suspect> {
    pub fn check_content(self) -> ContentBranch {
        let hash = Blake3Hash::compute(self.status.content.as_bytes());
        if self.status.view.is_content_match(&hash) {
            ContentBranch::Match(Self::transition(Refresh, StaleTimestamps { info: self.status.info, view: self.status.view }))
        } else {
            ContentBranch::Mismatch(Self::transition(Parsed, Stale { info: self.status.info, content: self.status.content, content_hash: hash, view: self.status.view }))
        }
    }
}

impl TemplateProcessor<Parsed, Missing> {
    pub fn parse(self, source: &FsReader, path: &std::path::Path) -> Result<TemplateProcessor<Construction, New>, TemplateError> {
        let content = source.read_to_string(path).map_err(|e| TemplateError::ValidationFailed(e.to_string()))?;
        let hash = Blake3Hash::compute(content.as_bytes());
        let (frontmatter, raw_body) = extract_template_asset(&content)?;

        Ok(Self::transition(Construction, New { raw_body, frontmatter, content_hash: hash, info: self.status.info }))
    }
}

impl TemplateProcessor<Parsed, Stale> {
    pub fn parse(self) -> Result<TemplateProcessor<Construction, Changed>, TemplateError> {
        let (frontmatter, raw_body) = extract_template_asset(&self.status.content)?;
        Ok(Self::transition(Construction, Changed { raw_body, frontmatter, content_hash: self.status.content_hash, info: self.status.info }))
    }
}

impl TemplateProcessor<Construction, New> {
    pub fn build(self, name: TemplateName) -> Result<TemplateProcessor<Completed, Ready>, TemplateError> {
        let template = Template {
            id: TemplateId::new(),
            name,
            schemas: self.status.frontmatter.schemas,
            inputs: self.status.frontmatter.inputs,
            body: self.status.raw_body,
        };
        // Persist logic would happen here in a real impl
        Ok(Self::transition(Completed, Ready { template }))
    }
}

impl TemplateProcessor<Construction, Changed> {
    pub fn build(self, existing_id: TemplateId, name: TemplateName) -> Result<TemplateProcessor<Completed, Ready>, TemplateError> {
        let template = Template {
            id: existing_id,
            name,
            schemas: self.status.frontmatter.schemas,
            inputs: self.status.frontmatter.inputs,
            body: self.status.raw_body,
        };
        // Persist logic would happen here in a real impl
        Ok(Self::transition(Completed, Ready { template }))
    }
}

impl TemplateProcessor<Construction, Fresh> {
    pub fn fetch(self, template: Template) -> TemplateProcessor<Completed, Ready> {
        // Fetch logic would happen here
        Self::transition(Completed, Ready { template })
    }
}

impl TemplateProcessor<Refresh, StaleTimestamps> {
    pub fn sync_metadata(self) -> TemplateProcessor<Construction, Fresh> {
        // Update DB logic would happen here
        Self::transition(Construction, Fresh { view: self.status.view })
    }
}

impl TemplateProcessor<Completed, Ready> {
    pub fn into_template(self) -> Template {
        self.status.template
    }
}
```

- [ ] **Step 4: Run test to verify it passes**

Run: `cargo test --package lithos-core --lib template::processor::tests`
Expected: PASS

- [ ] **Step 5: Commit**

```bash
git add lithos-core/src/template/processor.rs
git commit -m "feat(template): add typestate compilation pipeline"
```

---

## Task 6: Implement Query Builder and `TemplateRuntime`

Define the structured queries and the runtime object injected into `minijinja`.

**Files:**
- Create: `lithos-core/src/template/runtime/mod.rs`
- Create: `lithos-core/src/template/runtime/query.rs`
- Create: `lithos-core/src/template/runtime/li.rs`
- Create: `lithos-core/src/template/runtime/interact.rs`

- [ ] **Step 1: Write test for Query Builder**

```rust
// In lithos-core/src/template/runtime/query.rs
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn query_builder_constructs() {
        let q = QueryBuilder::new().schema("project").where_equals("status", "active");
        assert_eq!(q.schema, Some("project".to_string()));
        assert_eq!(q.filters.len(), 1);
    }
}
```

- [ ] **Step 2: Run test to verify it fails**

Run: `cargo test --package lithos-core --lib template::runtime::query::tests`
Expected: FAIL

- [ ] **Step 3: Implement Query Builder**

```rust
// In lithos-core/src/template/runtime/query.rs
#[derive(Debug, Clone, Default)]
pub struct QueryBuilder {
    pub schema: Option<String>,
    pub filters: Vec<QueryFilter>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum QueryFilter {
    Equals(String, String),
}

impl QueryBuilder {
    pub fn new() -> Self { Self::default() }

    pub fn schema(mut self, schema: &str) -> Self {
        self.schema = Some(schema.to_string());
        self
    }

    pub fn where_equals(mut self, field: &str, value: &str) -> Self {
        self.filters.push(QueryFilter::Equals(field.to_string(), value.to_string()));
        self
    }
}

// Ensure mod.rs exports
// In lithos-core/src/template/runtime/mod.rs
pub mod query;
pub mod interact;
pub mod li;
```

- [ ] **Step 4: Implement TemplateRuntime object**

```rust
// In lithos-core/src/template/runtime/li.rs
use std::sync::Arc;
use crate::template::runtime::query::QueryBuilder;
use minijinja::{Value, Error as JinjaError, ErrorKind};

pub trait InteractiveHost: Send + Sync {
    fn prompt_suggester(&self, options: Vec<String>) -> Result<String, String>;
}

pub struct TemplateRuntime {
    interact: Arc<dyn InteractiveHost>,
}

impl TemplateRuntime {
    pub fn new(interact: Arc<dyn InteractiveHost>) -> Self {
        Self { interact }
    }
}

impl minijinja::Object for TemplateRuntime {
    fn call_method(&self, _state: &minijinja::State<'_, '_>, name: &str, args: &[Value]) -> Result<Value, JinjaError> {
        match name {
            // li.query("project") -> Returns a custom builder object mapped into Jinja
            "query" => {
                let schema = args.get(0).and_then(|v| v.as_str()).unwrap_or("");
                let builder = QueryBuilder::new().schema(schema);
                // In a real impl, we'd wrap builder in an Arc and return Value::from_object
                // For now, we just return a simple string representation
                Ok(Value::from(format!("Query on {}", schema)))
            }
            "suggester" => {
                let options_val = args.get(0).ok_or_else(|| JinjaError::new(ErrorKind::InvalidOperation, "Missing options array"))?;
                let mut options_vec = Vec::new();
                for item in options_val.try_iter()? {
                    options_vec.push(item.to_string());
                }

                let selected = self.interact.prompt_suggester(options_vec)
                    .map_err(|e| JinjaError::new(ErrorKind::InvalidOperation, e))?;

                Ok(Value::from(selected))
            }
            _ => Err(JinjaError::new(ErrorKind::UnknownMethod, format!("Unknown method {}", name))),
        }
    }
}

// In lithos-core/src/template/runtime/interact.rs
pub trait VaultAccess: Send + Sync {
    // Methods for DB query
}
```

- [ ] **Step 5: Run test to verify it passes**

Run: `cargo test --package lithos-core --lib template::runtime::query::tests`
Expected: PASS

- [ ] **Step 6: Commit**

```bash
git add lithos-core/src/template/runtime
git commit -m "feat(template): add runtime query builder and li template object"
```

---

## Task 7: Implement Rendering Engine Boundary

Implement the thin wrapper over `minijinja` that ensures proper markdown formatting.

**Files:**
- Create: `lithos-core/src/template/engine/minijinja.rs`
- Create: `lithos-core/src/template/engine/mod.rs`

- [ ] **Step 1: Write test for rendering**

```rust
// In lithos-core/src/template/engine/minijinja.rs
#[cfg(test)]
mod tests {
    use super::*;
    use crate::template::aggregate::{Template, TemplateName, TemplateId};
    use crate::template::runtime::li::{TemplateRuntime, InteractiveHost};
    use std::sync::Arc;
    use std::collections::HashMap;

    struct MockHost;
    impl InteractiveHost for MockHost {
        fn prompt_suggester(&self, _options: Vec<String>) -> Result<String, String> {
            Ok("Selected".to_string())
        }
    }

    #[test]
    fn engine_renders_markdown_without_autoescape() {
        let t = Template {
            id: TemplateId::new(),
            name: TemplateName::try_from("test").unwrap(),
            schemas: vec![],
            inputs: HashMap::new(),
            body: "# Hello\n{{ li.suggester(['a', 'b']) }}".to_string()
        };

        let runtime = Arc::new(TemplateRuntime::new(Arc::new(MockHost)));
        let out = render_template(&t, runtime).unwrap();
        assert_eq!(out, "# Hello\nSelected");
    }
}
```

- [ ] **Step 2: Run test to verify it fails**

Run: `cargo test --package lithos-core --lib template::engine::minijinja::tests`
Expected: FAIL

- [ ] **Step 3: Implement Engine Adapter**

```rust
// In lithos-core/src/template/engine/minijinja.rs
use std::sync::Arc;
use crate::template::aggregate::Template;
use crate::template::runtime::li::TemplateRuntime;
use crate::template::error::TemplateError;

pub fn render_template(template: &Template, runtime: Arc<TemplateRuntime>) -> Result<String, TemplateError> {
    let mut env = minijinja::Environment::new();

    // CRITICAL: Disable autoescape for Markdown
    env.set_auto_escape_callback(|_| minijinja::AutoEscape::None);
    // CRITICAL: Strict behavior on undefined variables
    env.set_undefined_behavior(minijinja::UndefinedBehavior::Strict);

    env.add_template(template.name.as_str(), &template.body)
        .map_err(|e| TemplateError::ValidationFailed(format!("Jinja Syntax: {}", e)))?;

    let ctx = minijinja::context! {
        li => minijinja::Value::from_object(runtime),
    };

    let tmpl = env.get_template(template.name.as_str())
        .map_err(|e| TemplateError::ValidationFailed(format!("Get Template: {}", e)))?;

    tmpl.render(ctx).map_err(|e| TemplateError::ValidationFailed(format!("Render Error: {}", e)))
}

// Ensure mod exports
// In lithos-core/src/template/engine/mod.rs
pub mod minijinja;
```

- [ ] **Step 4: Run test to verify it passes**

Run: `cargo test --package lithos-core --lib template::engine::minijinja::tests`
Expected: PASS

- [ ] **Step 5: Commit**

```bash
git add lithos-core/src/template/engine
git commit -m "feat(template): add minijinja engine adapter configured for markdown"
```

---

Plan complete and saved to `docs/superpowers/plans/2026-05-07-template-redesign-plan.md`. Two execution options:

1. Subagent-Driven (recommended) - I dispatch a fresh subagent per task, review between tasks, fast iteration
2. Inline Execution - Execute tasks in this session using executing-plans, batch execution with checkpoints

Which approach?
