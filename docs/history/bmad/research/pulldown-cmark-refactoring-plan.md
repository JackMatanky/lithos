# pulldown-cmark Enhancement Refactoring Plan

**Version**: pulldown-cmark 0.13.0
**Date**: 2026-02-10
**Status**: Ready for Implementation
**Branch**: `refactor/note-context`

---

## Executive Summary

This refactoring plan consolidates our comprehensive analysis of pulldown-cmark 0.13.0 and provides a concrete implementation roadmap. We discovered that **we're using less than 10% of the parser's capabilities** and have native support for WikiLinks, frontmatter, and other Obsidian features built into the library.

### Key Findings

1. ✅ **Native WikiLink Support** - `[[link]]`, `[[link|alias]]`, `![[embed]]` syntax is BUILT-IN
2. ✅ **Native Frontmatter Support** - YAML metadata blocks are BUILT-IN
3. ✅ **Domain Models Exist** - `Heading` (457 lines), `Link` (618 lines), `Frontmatter` (1011 lines) are ready but unpopulated
4. ❌ **Parser Gap** - Zero code connects pulldown-cmark events to domain models

### Impact

- **Current State**: Using 1 of 15 options (6.7% of library capabilities)
- **Proposed State**: Using 8 of 15 options (53% - all Obsidian-relevant features)
- **Performance Impact**: ~5-8% overhead (acceptable for functionality gain)
- **Code Reduction**: ~200 lines eliminated (no custom regex parsing needed)
- **Functionality Gain**: 10x increase (headings, links, frontmatter, tables, math, etc.)

---

## Reference Documentation

### Complete Documentation

- **Full Reference**: [`docs/refs/crates/pulldown-cmark.md`](../docs/refs/crates/pulldown-cmark.md)
  - All 15 option flags documented (lines 197-215)
  - WikiLink parsing details (lines 742-820)
  - Frontmatter/MetadataBlock handling (lines 849-929)
  - Performance impact table (lines 995-1005)
  - Critical gotchas and patterns (lines 1255-1440)

### Analysis Documents

- **Deep Analysis**: [`_bmad-output/pulldown-cmark-deep-analysis.md`](./pulldown-cmark-deep-analysis.md)
  - Gap analysis (what we're missing)
  - Event type mapping
  - Risk assessment
  - Performance implications

- **Reference Implementation**: [`_bmad-output/pulldown-cmark-reference-impl.md`](./pulldown-cmark-reference-impl.md)
  - Complete working code examples
  - Event handler patterns
  - Test strategies

---

## Current State Analysis

### What Exists (Unused Domain Models)

| File                  | Lines | Status               | Accessor                    |
| --------------------- | ----- | -------------------- | --------------------------- |
| `note/structure.rs`   | 457   | ❌ **Not Populated** | `note.headings()` → empty   |
| `note/link.rs`        | 618   | ❌ **Not Populated** | `note.links()` → empty      |
| `note/frontmatter.rs` | 1011  | ❌ **Not Populated** | `note.frontmatter()` → None |
| `note/list.rs`        | ?     | ✅ Working           | `note.lists()` → works      |
| `note/task.rs`        | ?     | ✅ Working           | `note.tasks()` → works      |

### What's Missing (Parser Implementation)

```rust
// Current: fs/markdown.rs (line 31-35)
pub const fn with_tasklists() -> Self {
    Self {
        options: Options::ENABLE_TASKLISTS,  // ← ONLY 1 OPTION!
    }
}

// Missing parsers in note/parser.rs:
// - Heading parser (Event::Start(Tag::Heading { .. })) → 0 lines exist
// - WikiLink parser (Event::Start(Tag::Link { link_type: WikiLink, .. })) → 0 lines exist
// - Frontmatter parser (Event::Start(Tag::MetadataBlock(YamlStyle))) → 0 lines exist
// - Code block tracker (for preventing false positives) → 0 lines exist
```

### Current Event Handling (parser.rs lines 175-182)

```rust
Event::Start(_)              // ← Ignores ALL unhandled tags!
| Event::End(_)              // ← Includes Heading, Link, Image, MetadataBlock!
| Event::InlineMath(_)
| Event::DisplayMath(_)
| Event::Html(_)
| Event::InlineHtml(_)
| Event::FootnoteReference(_)
| Event::Rule => {}
```

**Critical Issue**: Parser ignores `Tag::Heading`, `Tag::Link`, `Tag::Image`, `Tag::MetadataBlock` despite domain models being ready!

---

## Implementation Phases

### Phase 0: Enable Options (1 hour - IMMEDIATE WIN)

**Goal**: Enable all Obsidian-relevant options with zero breaking changes

**File**: `lithos-core/src/fs/markdown.rs`

**Changes**:

1. Add new `with_obsidian_features()` method
2. Update `note/parser.rs` to use new method (line 80)

```rust
// Add this method to MarkdownParser:
pub const fn with_obsidian_features() -> Self {
    Self {
        options: Options::ENABLE_TASKLISTS
            | Options::ENABLE_WIKILINKS                    // ← WikiLinks!
            | Options::ENABLE_YAML_STYLE_METADATA_BLOCKS   // ← Frontmatter!
            | Options::ENABLE_HEADING_ATTRIBUTES           // ← Custom IDs!
            | Options::ENABLE_TABLES
            | Options::ENABLE_FOOTNOTES
            | Options::ENABLE_STRIKETHROUGH
            | Options::ENABLE_MATH
    }
}
```

**Testing**:

```bash
mise run test:unit:fs
mise run test:unit:note
```

**Verification**: Parser will start emitting new event types (but they'll be ignored until Phase 1)

---

### Phase 1A: Heading Parser (2 days)

**Goal**: Populate `note.headings()` with parsed heading data

**File**: `lithos-core/src/note/parser.rs`

**Changes**:

1. **Extend `ParseState`** (add 2 fields):

```rust
struct ParseState<'config> {
    headings: Vec<Heading>,                    // ← ADD
    current_heading: Option<HeadingState>,     // ← ADD
    // ... existing fields
}

struct HeadingState {
    level: HeadingLevel,
    text: String,
    position: SourceByteOffset,
}
```

2. **Add event handlers** in `handle_event()`:

```rust
Event::Start(Tag::Heading { level, .. }) => {
    self.start_heading(level, range.start)?
}
Event::End(TagEnd::Heading(_)) => {
    self.end_heading()?
}
Event::Text(text) => {
    if let Some(heading) = self.current_heading.as_mut() {
        heading.text.push_str(text.as_ref());
    } else if let Some(item) = self.current_item.as_mut() {
        item.text.push_str(text.as_ref());
    }
}
```

3. **Implement helpers** (~60 lines total):

```rust
fn start_heading(&mut self, level: pulldown_cmark::HeadingLevel, position: usize)
    -> Result<(), NoteError>
fn end_heading(&mut self) -> Result<(), NoteError>
```

4. **Update return type**:

```rust
// OLD:
type ParseOutcome = (Vec<List>, Vec<Task>);

// NEW:
type ParseOutcome = (Vec<List>, Vec<Task>, Vec<Heading>);
```

5. **Update `apply_to_note()`**:

```rust
let (lists, tasks, headings) = self.parse_lists_and_tasks(markdown)?;
for heading in headings {
    note.add_heading(heading);
}
```

**Testing**:

```rust
#[test]
fn parses_headings() -> Result<(), NoteError> {
    let config = TaskConfig::default();
    let parser = NoteParser::new(&config);

    let md = "# H1\n## H2\n### H3";
    let (_, _, headings) = parser.parse_lists_and_tasks(md)?;

    assert_eq!(headings.len(), 3);
    assert_eq!(headings[0].text(), "H1");
    assert_eq!(headings[0].level().value(), 1);
    Ok(())
}
```

**Reference**: [`pulldown-cmark-reference-impl.md` lines 445-492](./pulldown-cmark-reference-impl.md)

**Verification**:

```bash
mise run test:unit:note
```

**Impact**: ✅ `note.headings()` now returns parsed headings!

---

### Phase 1B: WikiLink Parser (3 days)

**Goal**: Populate `note.links()` with WikiLinks, embeds, and markdown links

**File**: `lithos-core/src/note/parser.rs`

**Changes**:

1. **Extend `ParseState`** (add 3 fields):

```rust
struct ParseState<'config> {
    links: Vec<Link>,                          // ← ADD
    current_link: Option<LinkState>,           // ← ADD
    in_link: bool,                             // ← ADD (context flag)
    // ... existing fields
}

struct LinkState {
    link_type: InternalLinkType,
    dest_url: Box<str>,
    alias: Option<String>,
    position: SourceByteOffset,
    is_embed: bool,
}

enum InternalLinkType {
    WikiLink { has_alias: bool },
    Markdown,
}
```

2. **Add event handlers**:

```rust
Event::Start(Tag::Link { link_type, dest_url, .. }) => {
    self.start_link(link_type, dest_url, range.start, false)?
}
Event::End(TagEnd::Link) => {
    self.end_link()?
}
Event::Start(Tag::Image { link_type, dest_url, .. }) => {
    self.start_link(link_type, dest_url, range.start, true)?  // is_embed = true
}
Event::End(TagEnd::Image) => {
    self.end_link()?
}
Event::Text(text) => {
    if let Some(link) = self.current_link.as_mut() {
        // For wikilinks with alias, this is the alias text
        if matches!(link.link_type, InternalLinkType::WikiLink { has_alias: true }) {
            link.alias = Some(text.as_ref().to_owned());
        }
    } else if let Some(heading) = self.current_heading.as_mut() {
        heading.text.push_str(text.as_ref());
    } else if let Some(item) = self.current_item.as_mut() {
        item.text.push_str(text.as_ref());
    }
}
```

3. **Implement helpers** (~150 lines):

```rust
fn start_link(&mut self, link_type: pulldown_cmark::LinkType, dest_url: CowStr<'_>,
              position: usize, is_embed: bool) -> Result<(), NoteError>
fn end_link(&mut self) -> Result<(), NoteError>

// Helper functions:
fn parse_link_destination(dest: &str) -> Result<(Box<str>, Option<Anchor>), NoteError>
fn determine_embed_type(path: &str) -> Result<EmbedType, NoteError>
fn is_external_url(url: &str) -> bool
```

4. **Update return type**:

```rust
type ParseOutcome = (Vec<List>, Vec<Task>, Vec<Heading>, Vec<Link>);
```

5. **Update `apply_to_note()`**:

```rust
let (lists, tasks, headings, links) = self.parse_lists_and_tasks(markdown)?;
for link in links {
    note.add_link(link);
}
```

**Testing**:

```rust
#[test]
fn parses_wikilink_simple() {
    let md = "[[target]]";
    let (_, _, _, links) = parser.parse_lists_and_tasks(md)?;
    assert_eq!(links.len(), 1);
    assert!(matches!(links[0].style(), Style::WikiLink));
}

#[test]
fn parses_wikilink_with_alias() {
    let md = "[[target|alias text]]";
    let (_, _, _, links) = parser.parse_lists_and_tasks(md)?;
    assert_eq!(links[0].alias(), Some("alias text"));
}

#[test]
fn parses_wikilink_with_heading_anchor() {
    let md = "[[note#Section]]";
    let (_, _, _, links) = parser.parse_lists_and_tasks(md)?;
    assert!(matches!(links[0].anchor(), Some(Anchor::Heading(text)) if text.as_ref() == "Section"));
}

#[test]
fn parses_embed() {
    let md = "![[image.png]]";
    let (_, _, _, links) = parser.parse_lists_and_tasks(md)?;
    assert!(links[0].is_embed());
    assert!(matches!(links[0].embed_type(), Some(EmbedType::Image)));
}
```

**Reference**: [`pulldown-cmark-reference-impl.md` lines 287-441](./pulldown-cmark-reference-impl.md)

**Critical Gotcha**: WikiLink alias detection via `has_pothole` flag (see [`docs/refs/crates/pulldown-cmark.md` lines 1342-1373](../docs/refs/crates/pulldown-cmark.md))

**Verification**:

```bash
mise run test:unit:note
```

**Impact**: ✅ `note.links()` now returns WikiLinks, embeds, and markdown links!

---

### Phase 1C: Frontmatter Parser (2 days)

**Goal**: Populate `note.frontmatter()` with parsed YAML metadata

**Dependencies**: Add `serde_yaml = "0.9"` to `lithos-core/Cargo.toml`

**File**: `lithos-core/src/note/parser.rs`

**Changes**:

1. **Extend `ParseState`** (add 2 fields):

```rust
struct ParseState<'config> {
    frontmatter: Option<Frontmatter>,          // ← ADD
    metadata_text: String,                     // ← ADD
    // ... existing fields
}
```

2. **Add event handlers**:

```rust
Event::Start(Tag::MetadataBlock(MetadataBlockKind::YamlStyle)) => {
    self.metadata_text.clear();
}
Event::End(TagEnd::MetadataBlock(MetadataBlockKind::YamlStyle)) => {
    self.parse_frontmatter()?;
}
Event::Text(text) => {
    // Check if we're in a metadata block
    if !self.metadata_text.is_empty() || self.metadata_text.capacity() > 0 {
        self.metadata_text.push_str(text.as_ref());
    } else if let Some(link) = self.current_link.as_mut() {
        // ... existing link handling
    } else if let Some(heading) = self.current_heading.as_mut() {
        // ... existing heading handling
    } else if let Some(item) = self.current_item.as_mut() {
        // ... existing item handling
    }
}
```

3. **Implement helpers** (~80 lines):

```rust
fn parse_frontmatter(&mut self) -> Result<(), NoteError>
fn yaml_to_field_map(yaml: &serde_yaml::Value) -> Result<HashMap<Box<str>, FieldValue>, NoteError>
fn yaml_value_to_field_value(value: &serde_yaml::Value) -> Result<FieldValue, NoteError>
```

4. **Update return type**:

```rust
type ParseOutcome = (Vec<List>, Vec<Task>, Vec<Heading>, Vec<Link>, Option<Frontmatter>);
```

5. **Update `apply_to_note()`**:

```rust
let (lists, tasks, headings, links, frontmatter) = self.parse_lists_and_tasks(markdown)?;
if let Some(fm) = frontmatter {
    note.set_frontmatter(Some(fm));
}
```

**Testing**:

```rust
#[test]
fn parses_frontmatter() {
    let md = r#"---
title: Test Note
tags: [rust, markdown]
priority: 1
---

Content"#;

    let (_, _, _, _, fm) = parser.parse_lists_and_tasks(md)?;
    let fm = fm.expect("should have frontmatter");
    assert_eq!(fm.get("title").and_then(|v| v.as_str()), Some("Test Note"));
}
```

**Reference**: [`pulldown-cmark-reference-impl.md` lines 497-581](./pulldown-cmark-reference-impl.md)

**Critical Gotcha**: Frontmatter must be at document start (see [`docs/refs/crates/pulldown-cmark.md` lines 1375-1397](../docs/refs/crates/pulldown-cmark.md))

**Verification**:

```bash
mise run test:unit:note
```

**Impact**: ✅ `note.frontmatter()` now returns parsed YAML metadata!

---

### Phase 1D: Code Block Tracking (1 day)

**Goal**: Prevent false positives in tag/task extraction inside code blocks

**File**: `lithos-core/src/note/parser.rs`

**Changes**:

1. **Extend `ParseState`** (add 1 field):

```rust
struct ParseState<'config> {
    in_code_block: bool,                       // ← ADD (context flag)
    // ... existing fields
}
```

2. **Add event handlers**:

```rust
Event::Start(Tag::CodeBlock(_)) => {
    self.in_code_block = true;
}
Event::End(TagEnd::CodeBlock) => {
    self.in_code_block = false;
}
```

3. **Update task promotion** in `note/task.rs`:

```rust
// In Task::should_promote():
pub fn should_promote(text: &str, config: &TaskConfig, in_code_block: bool) -> bool {
    if in_code_block {
        return false;  // Never promote tasks in code blocks
    }
    // ... existing logic
}
```

**Testing**:

```rust
#[test]
fn skips_tasks_in_code_blocks() {
    let md = r#"```rust
// - [ ] #task This should be ignored

// - [ ] #task This should be extracted"#;

      let (_, tasks, _, _, _) = parser.parse_lists_and_tasks(md)?;
      assert_eq!(tasks.len(), 1);
      assert!(tasks[0].text().contains("This should be extracted"));

  }

```

**Reference**: [`pulldown-cmark-reference-impl.md` lines 585-632](./pulldown-cmark-reference-impl.md)

**Verification**:
```bash
mise run test:unit:note
```

**Impact**: ✅ No more false positives for tags/tasks in code blocks!

---

### Phase 1E: Instrumentation (1 day)

**Goal**: Add `#[tracing::instrument]` to all new parser methods per architecture requirements

**File**: `lithos-core/src/note/parser.rs`

**Changes**:

```rust
#[tracing::instrument(skip(self, level, position), level = "debug")]
fn start_heading(&mut self, level: pulldown_cmark::HeadingLevel, position: usize)
    -> Result<(), NoteError>

#[tracing::instrument(skip(self), level = "debug")]
fn end_heading(&mut self) -> Result<(), NoteError>

#[tracing::instrument(skip(self, link_type, dest_url), level = "debug", fields(dest_url = %dest_url, is_embed))]
fn start_link(&mut self, link_type: pulldown_cmark::LinkType, dest_url: CowStr<'_>,
              position: usize, is_embed: bool) -> Result<(), NoteError>

#[tracing::instrument(skip(self), level = "debug")]
fn end_link(&mut self) -> Result<(), NoteError>

#[tracing::instrument(skip(self), level = "debug", fields(yaml_length = self.metadata_text.len()))]
fn parse_frontmatter(&mut self) -> Result<(), NoteError>
```

**Reference**: [`project-context.md` lines 117-134](../_bmad-output/project-context.md)

**Verification**:

```bash
RUST_LOG=debug mise run test:unit:note
```

---

## Dependencies & Setup

### Required Dependencies

```toml
# lithos-core/Cargo.toml [dependencies] section

# ALREADY PRESENT:
pulldown-cmark = "0.13"    # Event-stream markdown parser

# NEED TO ADD:
serde_yaml = "0.9"         # For frontmatter YAML parsing
```

### Commands

```bash
# Add dependency
cargo add serde_yaml --package lithos-core

# Verify it compiles
mise run build

# Run tests
mise run test:unit:note
```

---

## Testing Strategy

### Unit Tests (Phase 1)

**Location**: `lithos-core/src/note/parser.rs` (in existing `#[cfg(test)] mod tests`)

**Required Tests**:

- ✅ Existing: List parsing (5 tests)
- ✅ Existing: Task parsing (3 tests)
- 🆕 Heading parsing (3 tests):
  - `parses_headings()` - Multiple levels H1-H6
  - `parses_heading_text_with_formatting()` - Bold/italic in headings
  - `captures_heading_positions()`
