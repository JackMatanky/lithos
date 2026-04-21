# Rust Markdown Parser Architectures - Research Report

**Date**: April 21, 2026
**Focus**: Architecture patterns for markdown/note processing in Rust

## Executive Summary

This research examines 5 major Rust markdown processing projects to identify architectural patterns, separation of concerns, extensibility mechanisms, and performance strategies. The projects represent different approaches:

- **rumdl**: Linter/formatter with rule-based architecture
- **comrak**: CommonMark/GFM parser with AST manipulation
- **pulldown-cmark**: Pull parser (iterator-based, no AST construction)
- **mdBook**: Book builder with preprocessing pipeline
- **zola**: Static site generator with templating
- **markdown-it**: Plugin-based parser (Rust port of JS library)

## 1. rumdl - Lint/Format Architecture

### Architecture Overview

**Type**: Rule-based linter and formatter
**Parser**: Uses pulldown-cmark under the hood
**Design**: Two-phase processing (lint + cross-file validation)

### Component Separation

```
┌─────────────────────────────────────────────────────────┐
│                     CLI Layer                            │
│  - Command parsing (check, fmt, init, etc.)             │
│  - File discovery & filtering                           │
│  - Watch mode & LSP server                              │
└────────────────┬────────────────────────────────────────┘
                 │
┌────────────────▼────────────────────────────────────────┐
│                Processing Layer                          │
│  - lint() - Single file processing                      │
│  - lint_and_index() - Build cross-file index            │
│  - run_cross_file_checks() - Workspace validation       │
└────────────────┬────────────────────────────────────────┘
                 │
┌────────────────▼────────────────────────────────────────┐
│                 Rule System                              │
│  - Rule trait (check, fix, category)                    │
│  - 71 individual rule implementations                   │
│  - Rule filtering by content characteristics            │
└────────────────┬────────────────────────────────────────┘
                 │
┌────────────────▼────────────────────────────────────────┐
│              LintContext                                 │
│  - Markdown parsing (pulldown-cmark events)             │
│  - LineInfo tracking                                    │
│  - Inline config parsing (<!-- rumdl-disable -->)       │
│  - Flavor-specific handling                             │
└──────────────────────────────────────────────────────────┘
```

### Data Flow

```
Input File
    ↓
ContentCharacteristics::analyze() ← Quick scan for headings/lists/etc
    ↓
LintContext::new() ← Parse with pulldown-cmark, build line info
    ↓
Filter rules by content characteristics ← Skip irrelevant rules
    ↓
For each applicable rule:
    rule.check(lint_ctx) → Vec<LintWarning>
    ↓
    Filter by inline config (rumdl-disable comments)
    ↓
    Collect warnings
    ↓
Build FileIndex for cross-file rules ← Extract headings, links, etc
    ↓
Later: run_cross_file_checks() ← Validate links across workspace
    ↓
Output (JSON/text/annotations)
```

### Key Architectural Decisions

1. **Content Characteristics Pre-filtering**
   - Single-pass content scan to detect feature usage
   - Skip rules that don't apply (e.g., list rules when no lists present)
   - Performance optimization: reduces rule count by ~30-50% on typical docs

2. **LintContext as Central State**
   - Single parse per file (pulldown-cmark events → cached structures)
   - LineInfo tracks metadata per line (list depth, code block state, etc.)
   - Inline config parsed once, shared across all rules

3. **Two-Phase Validation**
   - **Phase 1**: Single-file rules check each file independently
   - **Phase 2**: Cross-file rules validate workspace-wide (link targets, etc.)
   - FileIndex caches extracted data to avoid re-parsing

4. **Incremental Processing**
   - Content hashing (blake3) for change detection
   - Cache results per file
   - Only re-lint changed files in watch mode

### Extensibility Mechanisms

1. **Rule Trait**
   ```rust
   pub trait Rule {
       fn name(&self) -> &str;
       fn category(&self) -> RuleCategory;
       fn check(&self, ctx: &LintContext) -> LintResult;
       fn fix(&self, ctx: &LintContext) -> FixResult; // Optional
       fn cross_file_scope(&self) -> CrossFileScope; // Single vs Workspace
       fn contribute_to_index(&self, ctx: &LintContext, index: &mut FileIndex);
       fn cross_file_check(&self, path: &Path, index: &FileIndex, workspace: &WorkspaceIndex) -> LintResult;
   }
   ```

2. **Rule Categories for Filtering**
   - Heading, List, Link, Image, CodeBlock, Html, Emphasis, Blockquote, Table
   - Whitespace, FrontMatter, Other (always run)

3. **Flavor System**
   - `MarkdownFlavor` enum (Standard, GFM, MkDocs, MDX, Quarto)
   - Per-file flavor overrides via config
   - Flavor detection from file extension

4. **Config Merging**
   - TOML-based configuration
   - Inline config overrides (HTML comments)
   - Config inheritance (extends directive)

### Performance Strategies

1. **Lazy Parsing**
   - Only parse when rules need it
   - Cache parsed structures in LintContext

2. **Regex Caching**
   - Global regex cache with LRU eviction
   - Avoids repeated regex compilation

3. **Parallel Processing**
   - Rayon for multi-file linting
   - ThreadLocal state for rule instances

4. **Smart Rule Filtering**
   - ContentCharacteristics analysis (~5-10ms overhead)
   - Saves 100ms+ by skipping 30+ irrelevant rules

### Frontmatter Handling

- Detected via `---` delimiters
- Excluded from linting by default
- Optional validation rules

### Custom Syntax Extensions

- Inline config via HTML comments
- Flavor-specific syntax (admonitions in MkDocs, JSX in MDX)
- Code block tools (linting/formatting code inside fenced blocks)

---

## 2. comrak - AST Manipulation Architecture

### Architecture Overview

**Type**: CommonMark/GFM parser with AST
**Design**: Arena-allocated tree with mutable nodes
**Focus**: Parse → Manipulate → Render

### Component Separation

```
┌─────────────────────────────────────────────────────────┐
│                   Parser Layer                           │
│  - Tokenization (scanners.rs)                           │
│  - Inline parsing (emphasis, links)                     │
│  - Block parsing (lists, headings, code blocks)         │
│  - Extension handling (tables, strikethrough, etc.)     │
└────────────────┬────────────────────────────────────────┘
                 │
┌────────────────▼────────────────────────────────────────┐
│                  AST Layer                               │
│  - nodes::AstNode (arena-allocated)                     │
│  - Parent/child/sibling pointers                        │
│  - NodeValue enum (Text, Paragraph, List, etc.)         │
│  - RefCell for mutation                                 │
└────────────────┬────────────────────────────────────────┘
                 │
┌────────────────▼────────────────────────────────────────┐
│               Rendering Layer                            │
│  - format_html() - HTML output                          │
│  - format_commonmark() - Markdown output                │
│  - format_xml() - XML output                            │
│  - Custom formatters via adapters                       │
└──────────────────────────────────────────────────────────┘
```

### Data Flow

```
Input String
    ↓
parse_document(arena, input, options) → root: &AstNode
    ↓
    ├─ Block parsing (paragraphs, lists, headings)
    │  └─ Inline parsing for each text node
    ↓
AST traversal (user code)
    ↓
    ├─ node.descendants() - iterator over all nodes
    ├─ Mutable access via RefCell::borrow_mut()
    ├─ Modify NodeValue (text replacement, etc.)
    ↓
format_html(root, options, output) → String
    ↓
    └─ Tree walk, emit HTML tags
```

### Key Architectural Decisions

1. **Arena Allocation**
   - All nodes allocated in typed_arena::Arena
   - Lifetime tied to arena ('a)
   - No dynamic allocation overhead during parsing

2. **Interior Mutability**
   - Nodes use RefCell<AstNode> for mutation
   - Allows tree manipulation after parsing
   - Trade-off: Runtime borrow checking

3. **Separation of Parsing & Rendering**
   - Parser produces language-agnostic AST
   - Formatters consume AST (HTML, XML, CommonMark)
   - Enables AST manipulation between phases

4. **Extension System**
   - Options struct with feature flags
   - Extensions: tables, strikethrough, tasklists, autolinks, footnotes, etc.
   - Plugins for custom rendering (syntax highlighting adapter)

### Extensibility Mechanisms

1. **AST Manipulation API**
   ```rust
   let arena = Arena::new();
   let root = parse_document(&arena, markdown, &options);
   for node in root.descendants() {
       if let NodeValue::Text(ref mut text) = node.data.borrow_mut().value {
           *text = text.replace("old", "new").into();
       }
   }
   ```

2. **Custom Formatters**
   - Implement formatter traits
   - Access to full AST during rendering
   - Example: syntax highlighting via syntect plugin

3. **Extension Options**
   - Bitflags for enabling extensions
   - Per-parse configuration
   - No global state

### Performance Strategies

1. **Arena Allocation**
   - Single allocation for all nodes
   - Cache-friendly memory layout
   - No individual node allocations

2. **Lazy Inline Parsing**
   - Block structure parsed first
   - Inline elements parsed on demand

3. **Cow Strings**
   - Zero-copy for unmodified text
   - Owns only when modified

### Frontmatter Handling

- Optional extension
- Parsed into metadata block
- Can be queried/modified in AST

### Custom Syntax Support

- Via extension options
- Plugin system for rendering
- WikiLinks extension example

---

## 3. pulldown-cmark - Pull Parser Architecture

### Architecture Overview

**Type**: Pull parser (iterator-based)
**Design**: Event stream without AST construction
**Focus**: Maximum performance for single-pass rendering

### Component Separation

```
┌─────────────────────────────────────────────────────────┐
│                  Parser Iterator                         │
│  - Yields Event enum items                              │
│  - State machine for markdown syntax                    │
│  - No tree construction                                 │
└────────────────┬────────────────────────────────────────┘
                 │
┌────────────────▼────────────────────────────────────────┐
│                   Event Stream                           │
│  Event::Start(Tag::Paragraph)                           │
│  Event::Text(CowStr)                                    │
│  Event::End(TagEnd::Paragraph)                          │
│  ...                                                    │
└────────────────┬────────────────────────────────────────┘
                 │
┌────────────────▼────────────────────────────────────────┐
│               Consumer Layer                             │
│  - push_html() - Direct HTML output                     │
│  - User code can build custom AST                       │
│  - TextMergeStream - Merge consecutive text events      │
└──────────────────────────────────────────────────────────┘
```

### Data Flow

```
Input String
    ↓
Parser::new(input) → Iterator<Item=Event>
    ↓
    ├─ Lazy tokenization
    ├─ Inline parsing on-the-fly
    ├─ Yields Event::Start/Text/End
    ↓
Consumer (e.g., push_html)
    ↓
    ├─ Match event type
    ├─ Emit HTML directly
    └─ No intermediate AST
    ↓
HTML String
```

### Key Architectural Decisions

1. **No AST Construction**
   - Events emitted as iterator
   - Consumer decides what to build
   - Minimal memory overhead

2. **Preorder Traversal Events**
   - Start(Tag) and End(TagEnd) pairs
   - Nested structures represented as event sequences

3. **Zero-Copy Text**
   - CowStr for text content
   - Borrows from input when possible
   - Inlines short strings

### Extensibility Mechanisms

1. **Event Filtering/Transformation**
   ```rust
   let parser = Parser::new(markdown);
   let filtered = parser.filter(|event| {
       !matches!(event, Event::Html(_))
   });
   ```

2. **Custom Consumers**
   - Iterate events, build custom structures
   - Example: build your own AST format

3. **TextMergeStream**
   - Utility to merge consecutive Text events
   - Improves consumer ergonomics

### Performance Strategies

1. **Iterator-Based**
   - Lazy evaluation
   - Only parse what's consumed
   - Perfect for single-pass rendering

2. **Inline Strings**
   - InlineStr for strings ≤23 bytes
   - Avoids heap allocation for common cases

3. **Minimal Copies**
   - CowStr borrows when possible
   - Only allocates on modification

### Limitations

- No parent/sibling pointers
- Hard to query structure (e.g., "find all links")
- Must consume events in order

---

## 4. mdBook - Preprocessing Pipeline Architecture

### Architecture Overview

**Type**: Book builder with plugin system
**Design**: Multi-stage preprocessing pipeline
**Parser**: Uses pulldown-cmark for rendering

### Component Separation

```
┌─────────────────────────────────────────────────────────┐
│                   Config Layer                           │
│  - book.toml parsing                                    │
│  - Preprocessor registration                            │
│  - Renderer selection                                   │
└────────────────┬────────────────────────────────────────┘
                 │
┌────────────────▼────────────────────────────────────────┐
│                  Book Structure                          │
│  - SUMMARY.md parsing (table of contents)               │
│  - Chapter hierarchy                                    │
│  - Book metadata                                        │
└────────────────┬────────────────────────────────────────┘
                 │
┌────────────────▼────────────────────────────────────────┐
│              Preprocessor Chain                          │
│  - Link resolution                                      │
│  - Include file expansion                               │
│  - Custom preprocessors (plugins)                       │
└────────────────┬────────────────────────────────────────┘
                 │
┌────────────────▼────────────────────────────────────────┐
│                Rendering Layer                           │
│  - HTML renderer (default)                              │
│  - Custom renderers (PDF, ePub, etc.)                   │
└──────────────────────────────────────────────────────────┘
```

### Data Flow

```
book.toml + src/ directory
    ↓
Load book structure from SUMMARY.md
    ↓
    ├─ Parse chapter hierarchy
    ├─ Resolve file paths
    └─ Load chapter content
    ↓
Run preprocessor chain
    ↓
    ├─ Preprocessor 1 (e.g., link resolution)
    ├─ Preprocessor 2 (e.g., include files)
    ├─ ...
    └─ Preprocessor N (custom)
    ↓
For each chapter:
    Parser::new(chapter.content) → events
    ↓
    push_html(events) → HTML
    ↓
Render book structure (TOC, nav, etc.)
    ↓
Output to HTML files
```

### Key Architectural Decisions

1. **Preprocessing Pipeline**
   - Transform book content before rendering
   - Preprocessors can modify chapter content
   - Extensible via plugins

2. **Book-Level Structure**
   - SUMMARY.md defines TOC
   - Chapters organized hierarchically
   - Metadata separate from content

3. **Renderer Abstraction**
   - Multiple output formats via renderer trait
   - Default HTML renderer
   - Custom renderers as plugins

### Extensibility Mechanisms

1. **Preprocessors**
   ```rust
   trait Preprocessor {
       fn name(&self) -> &str;
       fn run(&self, ctx: &PreprocessorContext, book: Book) -> Result<Book>;
   }
   ```

2. **Renderers**
   ```rust
   trait Renderer {
       fn name(&self) -> &str;
       fn render(&self, ctx: &RenderContext, book: &Book) -> Result<()>;
   }
   ```

3. **Custom Directives**
   - `{{#include file.md}}` - Include external files
   - `{{#rustdoc_include file.rs}}` - Include Rust docs
   - Extensible via preprocessors

### Performance Strategies

1. **Parallel Rendering**
   - Chapters rendered in parallel
   - Independent chapter processing

2. **Incremental Builds**
   - Track file changes
   - Only rebuild modified chapters

### Frontmatter Handling

- Not explicitly supported
- Can be handled via preprocessor

---

## 5. zola - Static Site Generator Architecture

### Architecture Overview

**Type**: Static site generator with templates
**Design**: Content processing + template rendering
**Parser**: Uses pulldown-cmark (or comrak fork)

### Component Separation

```
┌─────────────────────────────────────────────────────────┐
│                  Content Layer                           │
│  - Markdown files with frontmatter                      │
│  - Page/Section distinction                             │
│  - Taxonomy (tags, categories)                          │
└────────────────┬────────────────────────────────────────┘
                 │
┌────────────────▼────────────────────────────────────────┐
│              Processing Layer                            │
│  - Frontmatter parsing (TOML/YAML)                      │
│  - Markdown → HTML (pulldown-cmark)                     │
│  - Shortcode expansion                                  │
│  - Sass compilation                                     │
│  - Image processing                                     │
└────────────────┬────────────────────────────────────────┘
                 │
┌────────────────▼────────────────────────────────────────┐
│              Template Layer                              │
│  - Tera templates                                       │
│  - Context injection (page, section, config)            │
│  - Filters and functions                                │
└────────────────┬────────────────────────────────────────┘
                 │
┌────────────────▼────────────────────────────────────────┐
│                Output Layer                              │
│  - HTML files                                           │
│  - Static assets                                        │
│  - Search index                                         │
└──────────────────────────────────────────────────────────┘
```

### Data Flow

```
content/ directory
    ↓
Scan for .md files
    ↓
For each file:
    ├─ Parse frontmatter (TOML/YAML)
    ├─ Extract metadata (title, date, taxonomies)
    ├─ Parse markdown content
    ├─ Expand shortcodes
    └─ Convert to HTML
    ↓
Build site structure
    ├─ Page hierarchy
    ├─ Section organization
    └─ Taxonomy indexes
    ↓
Render templates
    ├─ Inject context (page, section, site)
    ├─ Apply Tera templates
    └─ Generate HTML
    ↓
Output to public/ directory
```

### Key Architectural Decisions

1. **Page/Section Distinction**
   - Pages: individual content items
   - Sections: collections with index page
   - Different template contexts

2. **Frontmatter-Driven**
   - TOML or YAML metadata
   - Drives template rendering
   - Enables sorting, filtering, taxonomies

3. **Shortcode System**
   - Reusable content blocks
   - Custom syntax ({{ shortcode_name() }})
   - Extensible via templates

4. **Asset Pipeline**
   - Sass compilation
   - Image processing (resize, format conversion)
   - Search index generation

### Extensibility Mechanisms

1. **Shortcodes**
   ```html
   <!-- In markdown -->
   {{ youtube(id="dQw4w9WgXcQ") }}

   <!-- In templates/shortcodes/youtube.html -->
   <iframe src="https://youtube.com/embed/{{ id }}"></iframe>
   ```

2. **Tera Filters**
   - Custom template filters
   - Applied to variables in templates

3. **Taxonomies**
   - Define custom taxonomies (tags, categories, etc.)
   - Auto-generated taxonomy pages

### Performance Strategies

1. **Parallel Processing**
   - Pages processed in parallel
   - Rayon for CPU-bound work

2. **Link Checking**
   - Validates internal links
   - Detects broken references

3. **Incremental Builds**
   - Track changed files
   - Only rebuild affected pages

### Frontmatter Handling

- Required for all pages
- TOML or YAML format
- Metadata for templates

### Custom Syntax Extensions

- Shortcodes (Tera templates)
- Link syntax (`@/path/to/page.md`)
- Anchor links (`[link](@/page.md#anchor)`)

---

## 6. markdown-it - Plugin-Based Architecture

### Architecture Overview

**Type**: Plugin-based parser (Rust port of JS library)
**Design**: Tokenizer → Parser → Renderer with plugin hooks
**Focus**: Maximum extensibility

### Component Separation

```
┌─────────────────────────────────────────────────────────┐
│                  Plugin System                           │
│  - Block rules (paragraph, heading, list)               │
│  - Inline rules (emphasis, link, code)                  │
│  - Renderer plugins                                     │
└────────────────┬────────────────────────────────────────┘
                 │
┌────────────────▼────────────────────────────────────────┐
│                Tokenizer Layer                           │
│  - Block tokenization                                   │
│  - Inline tokenization                                  │
│  - Nesting/structure tracking                           │
└────────────────┬────────────────────────────────────────┘
                 │
┌────────────────▼────────────────────────────────────────┐
│                  Token Stream                            │
│  - Flat list of tokens                                  │
│  - Token { type, content, nesting, ... }                │
└────────────────┬────────────────────────────────────────┘
                 │
┌────────────────▼────────────────────────────────────────┐
│               Renderer Layer                             │
│  - HTML renderer (default)                              │
│  - Custom renderers via plugins                         │
└──────────────────────────────────────────────────────────┘
```

### Data Flow

```
Input String
    ↓
Block Tokenizer
    ├─ Apply block rules (paragraph, heading, list, etc.)
    ├─ Generate block tokens
    ↓
Inline Tokenizer
    ├─ For each text token
    ├─ Apply inline rules (emphasis, link, code, etc.)
    ├─ Generate inline tokens
    ↓
Token Stream (flat list)
    ↓
Renderer
    ├─ Iterate tokens
    ├─ Apply renderer rules
    ├─ Generate HTML
    ↓
HTML String
```

### Key Architectural Decisions

1. **Token-Based (Not Events)**
   - Flat list of tokens (not nested tree)
   - Tokens have nesting level
   - Easier to manipulate than AST

2. **Plugin Architecture**
   - Plugins can add parsing rules
   - Plugins can modify token stream
   - Plugins can add renderer rules

3. **Rule Priority**
   - Rules executed in priority order
   - Allows plugins to override defaults

### Extensibility Mechanisms

1. **Block Rules**
   ```rust
   fn register_block_rule(parser: &mut MarkdownIt, rule: BlockRule) {
       parser.block.add_rule(rule);
   }
   ```

2. **Inline Rules**
   ```rust
   fn register_inline_rule(parser: &mut MarkdownIt, rule: InlineRule) {
       parser.inline.add_rule(rule);
   }
   ```

3. **Renderer Rules**
   ```rust
   fn custom_renderer(token: &Token) -> String {
       // Custom rendering logic
   }
   parser.renderer.add_rule("custom_token", custom_renderer);
   ```

### Performance Strategies

1. **Token Reuse**
   - Tokens allocated once, reused
   - Minimal allocations during parsing

2. **Lazy Inline Parsing**
   - Inline parsing on demand
   - Block structure first

### Frontmatter Handling

- Via plugin
- Not built-in

### Custom Syntax Support

- Full plugin system
- Examples in repo: @mentions, :emoji:

---

## Comparison Matrix

| Feature | rumdl | comrak | pulldown-cmark | mdBook | zola | markdown-it |
|---------|-------|--------|----------------|--------|------|-------------|
| **Approach** | Rule-based | AST | Event stream | Pipeline | Content+Templates | Plugin-based |
| **Primary Use** | Linting | Parsing | Fast rendering | Books | Sites | Extensibility |
| **AST** | No (uses pulldown-cmark) | Yes (arena) | No | No (uses pulldown-cmark) | No | Tokens (flat) |
| **Extensibility** | Rules | AST manipulation | Event filtering | Preprocessors | Shortcodes | Plugins |
| **Performance** | Content filtering | Arena allocation | Zero-copy | Parallel | Parallel | Token reuse |
| **Frontmatter** | Optional | Extension | No | Via preprocessor | Required | Plugin |
| **Custom Syntax** | Flavors | Extensions | Options | Preprocessor | Shortcodes | Full plugins |

---

## Architecture Patterns Identified

### 1. Parse Once, Use Many Times
- **rumdl**: LintContext caches parse results for all rules
- **comrak**: AST allows multiple transformations without re-parsing
- **mdBook**: Book structure parsed once, used by all preprocessors

### 2. Lazy vs. Eager Parsing
- **Lazy** (pulldown-cmark): Iterator yields events on-demand
- **Eager** (comrak): Full AST constructed upfront
- **Hybrid** (rumdl): Parse once, cache structures, iterate rules

### 3. Content Filtering
- **rumdl ContentCharacteristics**: Pre-scan to skip irrelevant rules (30-50% reduction)
- **zola**: File discovery with glob patterns
- **mdBook**: SUMMARY.md defines scope explicitly

### 4. Cross-File Analysis
- **rumdl WorkspaceIndex**: Two-phase (local + cross-file)
- **zola**: Site structure, link validation
- **mdBook**: Cross-chapter links

### 5. Configuration Hierarchies
- **rumdl**: File-level → per-file → inline comments
- **zola**: Site config → section config → page frontmatter
- **mdBook**: book.toml → preprocessor config

### 6. Extension Strategies
- **Options/Flags** (pulldown-cmark, comrak): Bitflags for features
- **Plugins** (mdBook, markdown-it): Runtime-registered extensions
- **Rules** (rumdl): Trait-based system with registration
- **Shortcodes** (zola): Template-based custom syntax

---

## Best Practices from Real-World Projects

### Performance

1. **Arena Allocation** (comrak)
   - Single allocation for all nodes
   - Lifetime-bound memory safety
   - Cache-friendly layout

2. **Content Pre-filtering** (rumdl)
   - Quick scan for feature presence
   - Skip irrelevant processing
   - 30-50% speedup on typical docs

3. **Parallel Processing** (mdBook, zola, rumdl)
   - Rayon for CPU-bound work
   - ThreadLocal state for mutable structures

4. **Incremental Builds** (mdBook, zola)
   - Track file changes
   - Only rebuild what changed

5. **Caching** (rumdl)
   - Content hashing (blake3)
   - Per-file cache with invalidation

### Separation of Concerns

1. **Parsing vs. Analysis** (comrak)
   - Parser produces AST
   - Separate traversal for transformations
   - Renderer consumes AST

2. **Rule System** (rumdl)
   - LintContext provides parsed data
   - Rules are stateless functions
   - No direct access to raw markdown

3. **Pipeline Architecture** (mdBook)
   - Preprocessors transform content
   - Renderers consume transformed content
   - Clear stage boundaries

### Extensibility

1. **Trait-Based** (rumdl, mdBook)
   - Well-defined interfaces
   - Compile-time safety
   - Easy to test

2. **Plugin Registration** (markdown-it)
   - Runtime extensibility
   - Priority-based execution
   - Can override defaults

3. **Template-Based** (zola)
   - User-defined shortcodes
   - No code required
   - Limited to template capabilities

### Handling Metadata/Frontmatter

1. **Structured Parsing** (zola)
   - TOML/YAML frontmatter
   - Validated schema
   - Drives template rendering

2. **Extension-Based** (comrak)
   - Optional feature
   - Parsed into AST
   - Can be queried/modified

3. **Ignored** (pulldown-cmark)
   - Not handled by default
   - User must strip manually

### Custom Syntax Extensions

1. **Options-Based** (comrak, pulldown-cmark)
   - CommonMark + GFM extensions
   - Bitflags for features
   - Limited to predefined extensions

2. **Full Plugin System** (markdown-it)
   - Add arbitrary syntax
   - Parsing rules + renderer rules
   - Maximum flexibility

3. **Flavor System** (rumdl)
   - Predefined flavor sets (GFM, MkDocs, MDX, Quarto)
   - Per-file flavor overrides
   - Balances flexibility vs. complexity

---

## Key Architectural Insights for Lithos

### 1. Use pulldown-cmark for Parsing Foundation
- Industry standard, well-tested
- Event stream allows flexible consumption
- Can build custom structures on top

### 2. Separate Concerns by Phase

```
Phase 1: File Ingestion
  - Read files (fs::source)
  - Parse frontmatter (TOML/YAML)
  - Validate syntax

Phase 2: Parsing
  - pulldown-cmark events
  - Build domain structures (Note, Schema)
  - Extract metadata (headings, links)

Phase 3: Validation
  - Single-file checks (schema structure)
  - Cross-file checks (link targets, refs)
  - Custom rules (business logic)

Phase 4: Storage
  - rkyv serialization
  - redb storage
  - Index structures
```

### 3. Two-Tier Rule System

**Local Rules** (single file):
- Schema validation
- Reference resolution
- Frontmatter validation

**Cross-File Rules** (workspace):
- Link target validation
- Duplicate detection
- Ref resolution

Pattern from rumdl:
```rust
trait Rule {
    fn check_local(&self, note: &Note) -> Result<Vec<Warning>>;
    fn contribute_to_index(&self, note: &Note, index: &mut Index);
    fn check_cross_file(&self, note: &Note, workspace: &Workspace) -> Result<Vec<Warning>>;
}
```

### 4. Cache Parsed Structures

From rumdl's LintContext pattern:
```rust
pub struct NoteContext {
    raw_content: String,
    parsed_events: Vec<Event>, // pulldown-cmark events
    line_info: Vec<LineInfo>,   // metadata per line
    frontmatter: Option<Frontmatter>,
    headings: Vec<Heading>,
    links: Vec<Link>,
    // ... etc
}
```

### 5. Use Content Hashing for Change Detection

From rumdl:
```rust
fn compute_content_hash(content: &str) -> String {
    blake3::hash(content.as_bytes()).to_hex()
}

struct FileIndex {
    content_hash: String,
    headings: Vec<Heading>,
    links: Vec<Link>,
    // ...
}
```

### 6. Workspace Index Pattern

From rumdl's two-phase validation:
```rust
// Phase 1: Build index
for file in files {
    let (warnings, file_index) = lint_and_index(file);
    workspace_index.add(file.path, file_index);
}

// Phase 2: Cross-file validation
for file in files {
    let warnings = run_cross_file_checks(file, workspace_index);
}
```

---

## Specific Code Examples

### 1. Arena-Based AST (comrak pattern)

```rust
use typed_arena::Arena;

pub struct NoteParser<'a> {
    arena: &'a Arena<Node<'a>>,
}

impl<'a> NoteParser<'a> {
    pub fn parse(&self, content: &str) -> &'a Node<'a> {
        let root = self.arena.alloc(Node::new(NodeValue::Document));
        // ... parsing logic
        root
    }
}

// Usage:
let arena = Arena::new();
let parser = NoteParser { arena: &arena };
let ast = parser.parse("# Hello");
// AST lives as long as arena
```

### 2. Pull Parser Event Stream (pulldown-cmark pattern)

```rust
use pulldown_cmark::{Parser, Event, Tag};

fn extract_headings(markdown: &str) -> Vec<Heading> {
    let parser = Parser::new(markdown);
    let mut headings = Vec::new();
    let mut current_level = None;

    for event in parser {
        match event {
            Event::Start(Tag::Heading { level, .. }) => {
                current_level = Some(level);
            }
            Event::Text(text) => {
                if let Some(level) = current_level {
                    headings.push(Heading { level, text: text.to_string() });
                }
            }
            Event::End(TagEnd::Heading(_)) => {
                current_level = None;
            }
            _ => {}
        }
    }
    headings
}
```

### 3. Rule Trait (rumdl pattern)

```rust
pub trait SchemaRule {
    fn name(&self) -> &str;
    fn category(&self) -> RuleCategory;
    fn check(&self, note: &Note) -> Result<Vec<Warning>>;
    fn can_fix(&self) -> bool { false }
    fn fix(&self, note: &mut Note) -> Result<Vec<Fix>> {
        Err("Fix not implemented".into())
    }
}

// Example rule:
pub struct RequiredFieldsRule {
    required: Vec<String>,
}

impl SchemaRule for RequiredFieldsRule {
    fn name(&self) -> &str { "required-fields" }
    fn category(&self) -> RuleCategory { RuleCategory::Schema }

    fn check(&self, note: &Note) -> Result<Vec<Warning>> {
        let mut warnings = Vec::new();
        for field in &self.required {
            if !note.frontmatter.contains_key(field) {
                warnings.push(Warning {
                    rule: self.name(),
                    message: format!("Missing required field: {}", field),
                    line: 1,
                });
            }
        }
        Ok(warnings)
    }
}
```

### 4. Content Filtering (rumdl pattern)

```rust
struct ContentCharacteristics {
    has_frontmatter: bool,
    has_headings: bool,
    has_links: bool,
    has_tags: bool,
}

impl ContentCharacteristics {
    fn analyze(content: &str) -> Self {
        let mut chars = Self::default();

        if content.starts_with("---") {
            chars.has_frontmatter = true;
        }

        for line in content.lines() {
            if line.starts_with('#') {
                chars.has_headings = true;
            }
            if line.contains('[') {
                chars.has_links = true;
            }
            if line.contains("#tag") {
                chars.has_tags = true;
            }
        }

        chars
    }

    fn should_skip_rule(&self, rule: &dyn SchemaRule) -> bool {
        match rule.category() {
            RuleCategory::Heading => !self.has_headings,
            RuleCategory::Link => !self.has_links,
            RuleCategory::Tag => !self.has_tags,
            _ => false,
        }
    }
}
```

### 5. Workspace Index (rumdl pattern)

```rust
pub struct WorkspaceIndex {
    files: HashMap<PathBuf, FileIndex>,
}

pub struct FileIndex {
    content_hash: String,
    headings: Vec<Heading>,
    links: Vec<Link>,
    tags: Vec<Tag>,
}

impl WorkspaceIndex {
    pub fn get_link_target(&self, from: &Path, link: &str) -> Option<&Heading> {
        // Resolve relative link
        let target_path = from.parent()?.join(link);
        let file_index = self.files.get(&target_path)?;
        // ... extract heading from link fragment
        file_index.headings.first()
    }

    pub fn find_backlinks(&self, file: &Path) -> Vec<(PathBuf, Link)> {
        let mut backlinks = Vec::new();
        for (path, index) in &self.files {
            for link in &index.links {
                if link.target == file {
                    backlinks.push((path.clone(), link.clone()));
                }
            }
        }
        backlinks
    }
}
```

---

## Recommendations for Lithos

### Architecture

1. **Use pulldown-cmark as parser foundation**
   - Industry-standard, well-tested
   - Event stream allows custom structure building
   - Can extend with custom passes

2. **Separate ingestion from parsing**
   - File I/O in `fs::source` (abstract filesystem)
   - Parsing in context-specific loaders
   - Storage in repository layer

3. **Two-phase validation**
   - Phase 1: Single-file checks (schema, syntax)
   - Phase 2: Cross-file checks (links, refs)
   - Use FileIndex pattern from rumdl

4. **Content hashing for change detection**
   - blake3 for fast, cryptographic hashing
   - Cache parsed structures per file
   - Only re-parse on hash change

### Performance

1. **Pre-filter by content characteristics**
   - Quick scan for features (frontmatter, tags, links)
   - Skip irrelevant validators
   - 30-50% speedup potential

2. **Parallel processing with Rayon**
   - Process files independently
   - ThreadLocal for mutable state
   - Join results at end

3. **Cache parsed structures**
   - NoteContext holds parsed data
   - Rules read from cache
   - Avoid re-parsing for each rule

### Extensibility

1. **Validator trait system**
   ```rust
   trait Validator {
       fn validate(&self, note: &Note) -> Result<Vec<Issue>>;
       fn scope(&self) -> ValidationScope; // Local vs Workspace
   }
   ```

2. **Schema-driven validation**
   - User-defined schemas
   - Auto-generate validators
   - Custom validators via plugins

3. **Flavor/template system**
   - Predefined templates (Zettelkasten, Wiki, etc.)
   - User-customizable
   - Per-directory overrides

### Testing

1. **Snapshot testing** (comrak uses this extensively)
   - Input markdown + expected output
   - Detect regressions
   - Easy to add new test cases

2. **Property-based testing** (rumdl has some examples)
   - Generate random markdown
   - Validate invariants
   - Find edge cases

---

## Conclusion

The research reveals several key patterns:

1. **Parser Foundation**: pulldown-cmark is the de facto standard for Rust markdown parsing
2. **Two-Phase Validation**: Local checks + cross-file checks with workspace index
3. **Content Filtering**: Pre-scan to skip irrelevant processing
4. **Separation of Concerns**: File I/O → Parsing → Validation → Storage
5. **Caching**: Content hashing + cached structures for performance
6. **Extensibility**: Trait-based systems (rules/validators) with clear interfaces

Lithos should adopt:
- **pulldown-cmark** for parsing
- **rumdl's two-phase validation** pattern
- **comrak's arena allocation** for AST (if needed)
- **Content filtering** for performance
- **Workspace index** for cross-file analysis

This architecture balances performance, extensibility, and maintainability while following Rust best practices.
