# Key Architecture Patterns - Quick Reference

**For**: Lithos development team
**Date**: April 21, 2026

## TL;DR - What to Steal

### 1. rumdl's Two-Phase Validation Pattern ✅

**Why it matters**: Cross-file link validation without re-parsing

```rust
// Phase 1: Build index while linting
for file in workspace {
    let (warnings, file_index) = lint_and_index(file);
    workspace_index.add(file.path, file_index);
}

// Phase 2: Cross-file checks
for file in workspace {
    let cross_warnings = validate_cross_file(file, workspace_index);
}
```

**Lithos application**:
- Phase 1: Parse note, validate schema, extract links/headings → FileIndex
- Phase 2: Validate link targets exist, no broken refs → WorkspaceIndex

### 2. Content Characteristics Pre-filtering ✅

**Why it matters**: 30-50% speedup by skipping irrelevant validators

```rust
struct ContentCharacteristics {
    has_frontmatter: bool,
    has_tags: bool,
    has_links: bool,
    has_code_blocks: bool,
}

// Quick scan (5-10ms)
let chars = ContentCharacteristics::analyze(content);

// Skip validators that don't apply
for validator in validators {
    if chars.should_skip(validator) { continue; }
    validator.check(content);
}
```

**Lithos application**:
- Scan for `---` frontmatter, `#tags`, `[[links]]`, etc.
- Skip validators when features not present
- Example: Skip tag validator when no `#` found

### 3. LintContext Pattern (Cached Parsing) ✅

**Why it matters**: Parse once, use many times

```rust
pub struct NoteContext {
    raw_content: String,
    // Cached parsed structures
    pulldown_events: Vec<Event>,
    line_info: Vec<LineInfo>,
    frontmatter: Option<Frontmatter>,
    headings: Vec<Heading>,
    links: Vec<Link>,
}

impl NoteContext {
    pub fn new(content: &str) -> Self {
        // Parse everything once
        let events = Parser::new(content).collect();
        let frontmatter = extract_frontmatter(&events);
        let headings = extract_headings(&events);
        // ...
        Self { /* all cached */ }
    }
}

// Validators just read from cache
impl Validator for RequiredFieldsValidator {
    fn check(&self, ctx: &NoteContext) -> Vec<Issue> {
        // No parsing! Just read ctx.frontmatter
    }
}
```

**Lithos application**:
- `note::NoteContext` holds all parsed data
- Validators never parse directly
- Single pulldown-cmark pass per file

### 4. Content Hashing for Change Detection ✅

**Why it matters**: Only re-process changed files

```rust
fn compute_hash(content: &str) -> String {
    blake3::hash(content.as_bytes()).to_hex()
}

struct FileIndex {
    content_hash: String,
    last_validated: Instant,
    // ... extracted data
}

// Check if file changed
if file_index.content_hash == compute_hash(new_content) {
    // Use cached results!
    return Ok(cached_warnings);
}
```

**Lithos application**:
- Hash note content on save
- Compare with stored hash in redb
- Skip validation if unchanged
- Watch mode only re-validates changed files

### 5. Workspace Index Pattern ✅

**Why it matters**: Fast cross-file queries

```rust
pub struct WorkspaceIndex {
    files: HashMap<PathBuf, FileIndex>,
}

impl WorkspaceIndex {
    pub fn find_link_target(&self, from: &Path, link: &str) -> Option<Heading> {
        let target_path = resolve_link(from, link);
        self.files.get(target_path)?.headings.find(/* ... */)
    }

    pub fn find_backlinks(&self, to: &Path) -> Vec<Link> {
        self.files.values()
            .flat_map(|idx| &idx.links)
            .filter(|link| link.target == to)
            .collect()
    }
}
```

**Lithos application**:
- `schema::WorkspaceIndex` for cross-file queries
- Find link targets without re-parsing
- Backlink queries in O(1) per file

### 6. Validator Trait with Scope ✅

**Why it matters**: Clear separation of local vs. workspace validators

```rust
pub enum ValidationScope {
    Local,      // Single file only
    Workspace,  // Needs other files
}

pub trait Validator {
    fn scope(&self) -> ValidationScope;
    fn check_local(&self, note: &Note) -> Vec<Issue>;
    fn check_workspace(&self, note: &Note, workspace: &WorkspaceIndex) -> Vec<Issue>;
}

// Example: Local validator
struct RequiredFieldsValidator;
impl Validator for RequiredFieldsValidator {
    fn scope(&self) -> ValidationScope { ValidationScope::Local }
    fn check_local(&self, note: &Note) -> Vec<Issue> {
        // Check frontmatter fields
    }
}

// Example: Workspace validator
struct LinkTargetValidator;
impl Validator for LinkTargetValidator {
    fn scope(&self) -> ValidationScope { ValidationScope::Workspace }
    fn check_workspace(&self, note: &Note, workspace: &WorkspaceIndex) -> Vec<Issue> {
        // Check link targets exist
    }
}
```

**Lithos application**:
- `schema::Validator` trait
- Local: schema structure, required fields, types
- Workspace: link targets, ref resolution, uniqueness

---

## Parser Foundation: pulldown-cmark

**Why pulldown-cmark?**
- Industry standard (used by cargo doc, crates.io, docs.rs)
- Event stream API (flexible)
- Zero-copy where possible
- Well-tested against CommonMark spec

**How to use**:
```rust
use pulldown_cmark::{Parser, Event, Tag};

fn extract_links(markdown: &str) -> Vec<Link> {
    let parser = Parser::new(markdown);
    let mut links = Vec::new();
    let mut in_link = false;
    let mut link_url = String::new();

    for event in parser {
        match event {
            Event::Start(Tag::Link { dest_url, .. }) => {
                in_link = true;
                link_url = dest_url.to_string();
            }
            Event::Text(text) if in_link => {
                links.push(Link {
                    text: text.to_string(),
                    url: link_url.clone(),
                });
            }
            Event::End(TagEnd::Link) => {
                in_link = false;
            }
            _ => {}
        }
    }
    links
}
```

---

## Anti-Patterns to Avoid

### ❌ DON'T: Parse for every validator

```rust
// BAD: Re-parsing for each validator
impl Validator for TagValidator {
    fn check(&self, content: &str) -> Vec<Issue> {
        let events = Parser::new(content).collect(); // ❌ Wasteful!
        // ...
    }
}
```

**Instead**: Use NoteContext pattern (parse once)

### ❌ DON'T: Cross-file checks during file scan

```rust
// BAD: Immediate cross-file validation
for file in files {
    let note = parse(file);
    for link in note.links {
        // ❌ This requires parsing other files mid-scan!
        if !target_exists(link.target) {
            errors.push("Broken link");
        }
    }
}
```

**Instead**: Two-phase validation (build index first)

### ❌ DON'T: Manual traversal of pulldown-cmark events

```rust
// BAD: Complex state machine for extracting data
let mut state = ComplexState::default();
for event in parser {
    // 50 lines of state transitions...
}
```

**Instead**: Use helper functions or existing libraries (like comrak if you need AST)

---

## Quick Decision Matrix

| Need | Use |
|------|-----|
| Parse markdown | pulldown-cmark |
| AST manipulation | comrak (or build on pulldown-cmark) |
| Cached parsing | NoteContext pattern (rumdl) |
| Cross-file validation | Two-phase + WorkspaceIndex |
| Performance boost | Content filtering |
| Change detection | Content hashing (blake3) |
| Parallel processing | Rayon |

---

## Lithos-Specific Recommendations

### File Ingestion Pipeline

```
File I/O (fs::source)
    ↓
Raw content → Hash
    ↓
Check cache (redb)
    ├─ Hit → Return cached
    └─ Miss ↓
Parse frontmatter (TOML)
    ↓
Validate frontmatter syntax
    ↓
Parse markdown (pulldown-cmark)
    ↓
Build NoteContext (cache structures)
    ↓
Run local validators
    ↓
Contribute to FileIndex
    ↓
Store in redb
```

### Cross-File Validation

```
All files ingested
    ↓
Build WorkspaceIndex from FileIndexes
    ↓
For each file:
    Run workspace validators
    ├─ Link targets exist?
    ├─ No duplicate IDs?
    └─ Refs resolve?
    ↓
Collect issues
```

### Note Context Structure

```rust
pub struct NoteContext {
    // Raw data
    raw_content: String,
    content_hash: String,

    // Parsed structures (cached)
    frontmatter: Frontmatter,
    events: Vec<Event<'static>>,

    // Extracted metadata
    headings: Vec<Heading>,
    links: Vec<Link>,
    tags: Vec<Tag>,

    // Index data
    line_info: Vec<LineInfo>,
}

impl NoteContext {
    pub fn new(content: &str) -> Self {
        // Parse everything once
        let hash = blake3::hash(content.as_bytes()).to_hex();
        let frontmatter = parse_frontmatter(content)?;
        let events = Parser::new(content).collect();
        let headings = extract_headings(&events);
        let links = extract_links(&events);
        let tags = extract_tags(&events);

        Self { /* all cached */ }
    }
}
```

### Validator Trait

```rust
pub trait Validator: Send + Sync {
    fn name(&self) -> &str;
    fn scope(&self) -> ValidationScope;
    fn category(&self) -> ValidatorCategory; // For content filtering

    // Local validation (single file)
    fn check_local(&self, ctx: &NoteContext) -> Result<Vec<Issue>>;

    // Workspace validation (cross-file)
    fn check_workspace(
        &self,
        ctx: &NoteContext,
        workspace: &WorkspaceIndex
    ) -> Result<Vec<Issue>> {
        Ok(Vec::new()) // Default: no workspace checks
    }

    // Contribute to index (for cross-file checks)
    fn contribute_to_index(&self, ctx: &NoteContext, index: &mut FileIndex) {
        // Default: no contribution
    }
}

pub enum ValidatorCategory {
    Frontmatter,
    Schema,
    Link,
    Tag,
    Content,
}
```

---

## Performance Targets (from rumdl benchmarks)

- **Small file (1KB)**: < 1ms per file
- **Medium file (10KB)**: < 5ms per file
- **Large file (100KB)**: < 50ms per file
- **Workspace (500 files)**: < 2s total (with parallel processing)

**How to achieve**:
1. Content filtering: Skip 30-50% of validators
2. Parallel processing: 4-8x speedup on multi-core
3. Caching: Only re-validate changed files
4. Smart indexing: O(1) cross-file lookups

---

## Testing Strategy

### 1. Snapshot Tests (comrak pattern)

```rust
#[test]
fn test_frontmatter_validation() {
    let input = r#"
---
title: Test Note
tags: [foo, bar]
---
# Content
"#;

    let ctx = NoteContext::new(input);
    let issues = RequiredFieldsValidator.check_local(&ctx);
    insta::assert_yaml_snapshot!(issues);
}
```

### 2. Property Tests (rumdl pattern)

```rust
use proptest::prelude::*;

proptest! {
    #[test]
    fn parsing_never_panics(content in ".*") {
        let _ = NoteContext::new(&content); // Should never panic
    }

    #[test]
    fn validation_is_deterministic(content in ".*") {
        let ctx = NoteContext::new(&content);
        let run1 = validator.check_local(&ctx);
        let run2 = validator.check_local(&ctx);
        assert_eq!(run1, run2); // Same input → same output
    }
}
```

### 3. Cross-File Tests

```rust
#[test]
fn test_link_validation() {
    let workspace = WorkspaceIndex::new();

    // Create target note
    workspace.add("target.md", FileIndex {
        headings: vec![Heading { level: 1, text: "Intro" }],
        ..Default::default()
    });

    // Create source note with link
    let source = NoteContext::new("[[target.md#Intro]]");
    let issues = LinkTargetValidator.check_workspace(&source, &workspace);

    assert!(issues.is_empty()); // Link should resolve
}
```

---

## Code Organization

```
lithos-core/src/
├── note/
│   ├── mod.rs           # Public API
│   ├── context.rs       # NoteContext (cached parsing)
│   ├── validators/      # Validator implementations
│   │   ├── mod.rs
│   │   ├── frontmatter.rs
│   │   ├── links.rs
│   │   └── tags.rs
│   └── index.rs         # FileIndex, WorkspaceIndex
│
├── schema/
│   ├── mod.rs
│   ├── definition.rs    # Schema types
│   ├── validators/      # Schema-specific validators
│   └── loader.rs        # Schema file loading
│
└── fs/
    ├── source.rs        # FileReader abstraction
    └── watcher.rs       # File change detection
```

---

## Next Steps

1. **Implement NoteContext** with cached pulldown-cmark parsing
2. **Define Validator trait** with Local/Workspace scopes
3. **Add content filtering** (ContentCharacteristics pattern)
4. **Build FileIndex/WorkspaceIndex** for cross-file validation
5. **Add content hashing** for change detection
6. **Implement parallel processing** with Rayon

Each step builds on the previous, following proven patterns from production Rust markdown tools.
