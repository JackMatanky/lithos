# pulldown-cmark - Reference Documentation

**Version:** 0.13.0
**Official Docs:** https://docs.rs/pulldown-cmark/0.13.0/pulldown_cmark/
**Guide:** https://pulldown-cmark.github.io/pulldown-cmark/
**Repository:** https://github.com/pulldown-cmark/pulldown-cmark
**License:** MIT

## Overview

pulldown-cmark is a pull parser for CommonMark Markdown written in Rust. It provides an iterator-based API that generates parsing events, enabling memory-efficient document processing without building a full AST. The parser supports CommonMark with optional extensions like tables, footnotes, strikethrough, and task lists.

The parser implements the Rust Iterator trait directly, yielding `Event` enums that represent the document structure. This pull-parsing approach uses dramatically less memory than AST construction while being easier to use than push parsers.
See https://pulldown-cmark.github.io/pulldown-cmark/ for conceptual guide on pull parsing architecture.

## Core Features for High Performance

### 1. Pull Parser Architecture

#### Iterator-Based Event Stream

**Key Concept:** Parse Markdown by iterating over events without building a full document tree.

```rust
use pulldown_cmark::{Parser, Event};

let markdown = "# Hello\n\nWorld *emphasis*";
let parser = Parser::new(markdown);

for event in parser {
    match event {
        Event::Start(tag) => println!("Start: {:?}", tag),
        Event::End(tag_end) => println!("End: {:?}", tag_end),
        Event::Text(text) => println!("Text: {}", text),
        _ => {}
    }
}
```

**Performance Characteristics:**

- Minimal memory footprint - no AST allocation
- Streaming processing - start consuming before parsing completes
- Zero-copy text fragments via copy-on-write strings
- Efficient for large documents

**Architecture Benefits:**

- Can drive push interfaces with minimal memory
- Easy to construct ASTs if needed
- Source-map information readily available via `into_offset_iter()`
- Transformations via iterator combinators (map, filter, etc.)

### 2. Copy-on-Write String Optimization

#### [`CowStr`](https://docs.rs/pulldown-cmark/0.13.0/pulldown_cmark/enum.CowStr.html) - Zero-Allocation Text

```rust
pub enum CowStr<'a> {
    Borrowed(&'a str),    // Zero-copy reference to source
    Boxed(Box<str>),      // Owned allocated string
    Inlined(InlineStr),   // Small string optimization
}
```

**Zero-Copy Path (Most Common):**

- Text events return slices of source document
- No allocation or copying during parsing
- Direct memory reference with lifetime `'a`
- Single copy from source to output when rendering

**Allocated Path (When Needed):**

- Escaped characters or entity references
- Text transformations
- Boxed for larger strings, inlined for ~3 words

**Performance Note:** The vast majority of text fragments use `Borrowed` variant, requiring no allocation.

**Usage Example:**

```rust
use pulldown_cmark::{Parser, Event, CowStr};

let parser = Parser::new("Hello *world*");

for event in parser {
    if let Event::Text(text) = event {
        // text is CowStr - usually borrowed from source
        let s: &str = &text;  // Deref to &str
    }
}
```

### 3. Source Offset Tracking

#### [`OffsetIter`](https://docs.rs/pulldown-cmark/0.13.0/pulldown_cmark/struct.OffsetIter.html) - Event Range Mapping

```rust
use pulldown_cmark::Parser;
use std::ops::Range;

let markdown = "# Heading\n\nParagraph";
let parser = Parser::new(markdown);

// Get events with source ranges
for (event, range) in parser.into_offset_iter() {
    println!("Event {:?} at bytes {}..{}", event, range.start, range.end);
}
```

**Use Cases:**

- Error reporting with line/column info
- Syntax highlighting with position data
- Document transformation tracking
- Source map generation

**Important:** `into_offset_iter()` consumes the parser and returns `(Event, Range<usize>)` pairs.

## Parser API

### [`Parser`](https://docs.rs/pulldown-cmark/0.13.0/pulldown_cmark/struct.Parser.html) - Core Iterator

#### Basic Construction

```rust
use pulldown_cmark::Parser;

// CommonMark only (default)
let parser = Parser::new("**bold**");

// With extensions
use pulldown_cmark::Options;
let mut options = Options::empty();
options.insert(Options::ENABLE_TABLES);
options.insert(Options::ENABLE_STRIKETHROUGH);
let parser = Parser::new_ext("~~strikethrough~~", options);
```

**Construction Methods:**

- `Parser::new(text: &str)` - CommonMark only
- `Parser::new_ext(text: &str, options: Options)` - With extensions
- `Parser::new_with_broken_link_callback(...)` - Custom link resolution

#### Key Methods

```rust
impl<'a> Parser<'a> {
    // Convert to offset iterator
    pub fn into_offset_iter(self) -> OffsetIter<'a>;

    // Get reference definitions
    pub fn reference_definitions(&self) -> &RefDefs;
}
```

### [`Options`](https://docs.rs/pulldown-cmark/0.13.0/pulldown_cmark/struct.Options.html) - Extension Flags

```rust
use pulldown_cmark::Options;

let mut options = Options::empty();

// GFM extensions
options.insert(Options::ENABLE_TABLES);
options.insert(Options::ENABLE_STRIKETHROUGH);
options.insert(Options::ENABLE_TASKLISTS);
options.insert(Options::ENABLE_FOOTNOTES);

// Other extensions
options.insert(Options::ENABLE_SMART_PUNCTUATION);
options.insert(Options::ENABLE_HEADING_ATTRIBUTES);
options.insert(Options::ENABLE_YAML_STYLE_METADATA_BLOCKS);
options.insert(Options::ENABLE_PLUSES_DELIMITED_METADATA_BLOCKS);
options.insert(Options::ENABLE_OLD_FOOTNOTES);
options.insert(Options::ENABLE_MATH);
options.insert(Options::ENABLE_GFM);
```

**Common Patterns:**

```rust
// All GFM features
let options = Options::ENABLE_GFM;

// Multiple extensions
let options = Options::ENABLE_TABLES | Options::ENABLE_FOOTNOTES;
```

**Extension Note:** Extensions are not part of CommonMark spec. Only enable what you need.

## Event Types

### [`Event`](https://docs.rs/pulldown-cmark/0.13.0/pulldown_cmark/enum.Event.html) - Core Enum

```rust
pub enum Event<'a> {
    // Container elements
    Start(Tag<'a>),
    End(TagEnd),

    // Leaf elements
    Text(CowStr<'a>),
    Code(CowStr<'a>),
    InlineHtml(CowStr<'a>),
    Html(CowStr<'a>),

    // Breaks
    SoftBreak,
    HardBreak,

    // Other
    Rule,
    TaskListMarker(bool),
    InlineMath(CowStr<'a>),
    DisplayMath(CowStr<'a>),
}
```

**Event Flow Pattern:**

```
Document:  # Heading\n\nParagraph *text*

Events:
  Start(Heading(H1))
  Text("Heading")
  End(Heading(H1))
  Start(Paragraph)
  Text("Paragraph ")
  Start(Emphasis)
  Text("text")
  End(Emphasis)
  End(Paragraph)
```

**Consecutive Text Note:** Parser may emit consecutive `Event::Text` events. Use `TextMergeStream` to merge them.

### [`Tag`](https://docs.rs/pulldown-cmark/0.13.0/pulldown_cmark/enum.Tag.html) - Container Start

```rust
pub enum Tag<'a> {
    Paragraph,
    Heading {
        level: HeadingLevel,
        id: Option<CowStr<'a>>,
        classes: Vec<CowStr<'a>>,
        attrs: Vec<(CowStr<'a>, Option<CowStr<'a>>)>
    },
    BlockQuote(Option<BlockQuoteKind>),
    CodeBlock(CodeBlockKind<'a>),
    HtmlBlock,
    List(Option<u64>), // None = bullet, Some(n) = ordered starting at n
    Item,
    FootnoteDefinition(CowStr<'a>),
    DefinitionList,
    DefinitionListTitle,
    DefinitionListDefinition,
    Table(Vec<Alignment>),
    TableHead,
    TableRow,
    TableCell,
    Emphasis,
    Strong,
    Strikethrough,
    Link {
        link_type: LinkType,
        dest_url: CowStr<'a>,
        title: CowStr<'a>,
        id: CowStr<'a>,
    },
    Image {
        link_type: LinkType,
        dest_url: CowStr<'a>,
        title: CowStr<'a>,
        id: CowStr<'a>,
    },
    MetadataBlock(MetadataBlockKind),
}
```

**Usage Example:**

```rust
use pulldown_cmark::{Parser, Event, Tag};

let parser = Parser::new("*emphasized*");

for event in parser {
    match event {
        Event::Start(Tag::Emphasis) => print!("<em>"),
        Event::End(TagEnd::Emphasis) => print!("</em>"),
        Event::Text(text) => print!("{}", text),
        _ => {}
    }
}
```

### [`TagEnd`](https://docs.rs/pulldown-cmark/0.13.0/pulldown_cmark/enum.TagEnd.html) - Container End

```rust
pub enum TagEnd {
    Paragraph,
    Heading(HeadingLevel),
    BlockQuote,
    CodeBlock,
    HtmlBlock,
    List(bool), // true = ordered
    Item,
    FootnoteDefinition,
    DefinitionList,
    DefinitionListTitle,
    DefinitionListDefinition,
    Table,
    TableHead,
    TableRow,
    TableCell,
    Emphasis,
    Strong,
    Strikethrough,
    Link,
    Image,
    MetadataBlock(MetadataBlockKind),
}
```

**Design Note:** `TagEnd` is separate from `Tag` to avoid carrying data not needed at close tags.

## Document Analysis & Extraction

### Extracting Document Structure

```rust
use pulldown_cmark::{Parser, Event, Tag, TagEnd};

#[derive(Debug)]
struct DocumentStructure {
    headings: Vec<(u8, String)>,
    links: Vec<String>,
    code_blocks: Vec<(Option<String>, String)>,
}

fn analyze_document(markdown: &str) -> DocumentStructure {
    let parser = Parser::new(markdown);
    let mut structure = DocumentStructure {
        headings: Vec::new(),
        links: Vec::new(),
        code_blocks: Vec::new(),
    };

    let mut current_heading = None;
    let mut current_code_block = None;
    let mut current_text = String::new();

    for event in parser {
        match event {
            Event::Start(Tag::Heading { level, .. }) => {
                current_heading = Some(level);
                current_text.clear();
            }
            Event::Text(text) if current_heading.is_some() => {
                current_text.push_str(&text);
            }
            Event::End(TagEnd::Heading(level)) if current_heading.is_some() => {
                let level_num = match level {
                    HeadingLevel::H1 => 1,
                    HeadingLevel::H2 => 2,
                    HeadingLevel::H3 => 3,
                    HeadingLevel::H4 => 4,
                    HeadingLevel::H5 => 5,
                    HeadingLevel::H6 => 6,
                };
                structure.headings.push((level_num, current_text.clone()));
                current_heading = None;
            }
            Event::Start(Tag::Link { dest_url, .. }) => {
                structure.links.push(dest_url.to_string());
            }
            Event::Start(Tag::CodeBlock(CodeBlockKind::Fenced(lang))) => {
                current_code_block = Some(Some(lang.to_string()));
                current_text.clear();
            }
            Event::Start(Tag::CodeBlock(CodeBlockKind::Indented)) => {
                current_code_block = Some(None);
                current_text.clear();
            }
            Event::Text(text) if current_code_block.is_some() => {
                current_text.push_str(&text);
            }
            Event::End(TagEnd::CodeBlock) if current_code_block.is_some() => {
                structure.code_blocks.push((
                    current_code_block.take().flatten(),
                    current_text.clone(),
                ));
            }
            _ => {}
        }
    }

    structure
}
```

### Validating Document Structure

```rust
use pulldown_cmark::{Parser, Event, Tag, TagEnd};

#[derive(Debug)]
struct ValidationResult {
    valid: bool,
    errors: Vec<String>,
}

fn validate_heading_hierarchy(markdown: &str) -> ValidationResult {
    let parser = Parser::new(markdown);
    let mut result = ValidationResult {
        valid: true,
        errors: Vec::new(),
    };

    let mut last_level = 0u8;

    for event in parser {
        if let Event::End(TagEnd::Heading(level)) = event {
            let level_num = match level {
                HeadingLevel::H1 => 1,
                HeadingLevel::H2 => 2,
                HeadingLevel::H3 => 3,
                HeadingLevel::H4 => 4,
                HeadingLevel::H5 => 5,
                HeadingLevel::H6 => 6,
            };

            if level_num > last_level + 1 && last_level > 0 {
                result.valid = false;
                result.errors.push(format!(
                    "Heading level jump from {} to {} is too large",
                    last_level, level_num
                ));
            }

            last_level = level_num;
        }
    }

    result
}
```

## Utilities

### [`TextMergeStream`](https://docs.rs/pulldown-cmark/0.13.0/pulldown_cmark/struct.TextMergeStream.html) - Merge Consecutive Text

```rust
use pulldown_cmark::{Parser, Event, TextMergeStream};

let markdown = "hello *world*";
let parser = Parser::new(markdown);

// Wrap parser to merge consecutive Text events
let merged = TextMergeStream::new(parser);

for event in merged {
    match event {
        Event::Text(text) => {
            // text now contains merged consecutive fragments
            println!("{}", text);
        }
        _ => {}
    }
}
```

**Use Case:** Parser may emit multiple consecutive `Text` events due to internal scanning. `TextMergeStream` combines them for easier processing.

### [`RefDefs`](https://docs.rs/pulldown-cmark/0.13.0/pulldown_cmark/struct.RefDefs.html) - Reference Definitions

```rust
use pulldown_cmark::Parser;

let markdown = "[link][ref]\n\n[ref]: https://example.com";
let parser = Parser::new(markdown);

// Access reference definitions
let refs = parser.reference_definitions();
```

**Purpose:** Track reference-style links and images defined in document.

## Advanced Features

### Broken Link Handling

```rust
use pulldown_cmark::{Parser, BrokenLink, CowStr};

fn callback<'a>(broken_link: BrokenLink<'a>) -> Option<(CowStr<'a>, CowStr<'a>)> {
    // Return (url, title) for broken reference
    if broken_link.reference == "custom" {
        Some(("https://example.com".into(), "Custom Link".into()))
    } else {
        None
    }
}

let markdown = "[text][custom]";
let parser = Parser::new_with_broken_link_callback(
    markdown,
    Options::empty(),
    Some(&callback)
);
```

**Use Cases:**

- Custom link resolution
- Wiki-style links
- Dynamic reference generation
- Error recovery

### Document Transformation

**Iterator Combinators:**

```rust
use pulldown_cmark::{Parser, Event};

let parser = Parser::new("text\nmore");

// Transform soft breaks to hard breaks
let transformed = parser.map(|event| match event {
    Event::SoftBreak => Event::HardBreak,
    _ => event
});
```

**Text Replacement:**

```rust
let parser = Parser::new("abbr stands for abbreviation");

let expanded = parser.map(|event| match event {
    Event::Text(text) => {
        Event::Text(text.replace("abbr", "abbreviation").into())
    }
    _ => event
});
```

**Nesting Level Tracking:**

```rust
use pulldown_cmark::{Parser, Event};

let parser = Parser::new("# Heading\n\n> Quote\n\n> > Nested");

let mut max_nesting = 0;
let mut level = 0;

for event in parser {
    match event {
        Event::Start(_) => {
            level += 1;
            max_nesting = std::cmp::max(max_nesting, level);
        }
        Event::End(_) => level -= 1,
        _ => {}
    }
}

println!("Max nesting: {}", max_nesting);
```

## Type Reference

### [`HeadingLevel`](https://docs.rs/pulldown-cmark/0.13.0/pulldown_cmark/enum.HeadingLevel.html)

```rust
pub enum HeadingLevel {
    H1, H2, H3, H4, H5, H6,
}
```

**Conversion:** Can convert from `usize` with `TryFrom`, returns `InvalidHeadingLevel` error if not 1-6.

### [`CodeBlockKind`](https://docs.rs/pulldown-cmark/0.13.0/pulldown_cmark/enum.CodeBlockKind.html)

```rust
pub enum CodeBlockKind<'a> {
    Indented,
    Fenced(CowStr<'a>), // Contains info string (language, etc.)
}
```

**Example:**

````markdown
```rust
code here
```
````

Produces: `CodeBlock(Fenced("rust"))`

### [`LinkType`](https://docs.rs/pulldown-cmark/0.13.0/pulldown_cmark/enum.LinkType.html)

```rust
pub enum LinkType {
    Inline,              // [text](url)
    Reference,           // [text][ref]
    ReferenceUnknown,    // [text][unknown]
    Collapsed,           // [text][]
    CollapsedUnknown,    // [text][] (unknown)
    Shortcut,            // [text]
    ShortcutUnknown,     // [text] (unknown)
    Autolink,            // <url>
    Email,               // <email>
}
```

### [`Alignment`](https://docs.rs/pulldown-cmark/0.13.0/pulldown_cmark/enum.Alignment.html)

```rust
pub enum Alignment {
    None,
    Left,
    Center,
    Right,
}
```

**Usage:** Table column alignment from `Tag::Table(Vec<Alignment>)`.

### [`BlockQuoteKind`](https://docs.rs/pulldown-cmark/0.13.0/pulldown_cmark/enum.BlockQuoteKind.html)

```rust
pub enum BlockQuoteKind {
    Note,
    Tip,
    Important,
    Warning,
    Caution,
}
```

**Extension:** GitHub-style alerts/admonitions.

### [`MetadataBlockKind`](https://docs.rs/pulldown-cmark/0.13.0/pulldown_cmark/enum.MetadataBlockKind.html)

```rust
pub enum MetadataBlockKind {
    YamlStyle,    // ---\n...\n---
    PlusesStyle,  // +++\n...\n+++
}
```

**Extension:** Front matter metadata blocks (YAML, TOML).

## Performance Best Practices

### 1. Memory Efficiency

**Zero-Copy Pattern:**

```rust
// ✅ GOOD: Borrows from source
let parser = Parser::new(markdown_source);
for event in parser {
    // Events contain borrowed &str via CowStr::Borrowed
}
```

**Avoid Unnecessary Allocations:**

```rust
// ❌ BAD: Unnecessary clone
Event::Text(text) => {
    let owned = text.to_string();  // Allocates even if text is borrowed
}

// ✅ GOOD: Work with borrowed data
Event::Text(text) => {
    let s: &str = &text;  // No allocation
}
```

### 2. Buffered Output

**Always Use Buffered Writers:**

```rust
// ❌ BAD: Unbuffered (slow for many small writes)
html::write_html(std::io::stdout(), parser)?;

// ✅ GOOD: Buffered
let mut output = String::new();
html::push_html(&mut output, parser);
print!("{}", output);

// ✅ GOOD: Buffered file I/O
use std::io::BufWriter;
let writer = BufWriter::new(file);
html::write_html(writer, parser)?;
```

### 3. Feature Flags

**Only Enable Needed Extensions:**

```rust
// ❌ BAD: Kitchen sink
let options = Options::all();

// ✅ GOOD: Minimal
let options = Options::ENABLE_TABLES | Options::ENABLE_STRIKETHROUGH;
```

**SIMD Acceleration (x64):**

```toml
# Cargo.toml
[dependencies]
pulldown-cmark = { version = "0.13", features = ["simd"] }
```

**Performance Build:**

```toml
[profile.release]
lto = true
codegen-units = 1
panic = "abort"
```

### 4. Event Processing Patterns

**Collect Only When Necessary:**

```rust
// ❌ BAD: Allocates vector unnecessarily
let events: Vec<_> = parser.collect();
for event in events {
    process(event);
}

// ✅ GOOD: Stream processing
for event in parser {
    process(event);
}
```

**Buffer When You Need Lookahead:**

```rust
// ✅ ACCEPTABLE: When you need to scan ahead
let events: Vec<_> = parser.collect();
for i in 0..events.len() {
    if matches!(events[i], Event::Start(Tag::Link { .. })) {
        // Check next event
        if let Some(next) = events.get(i + 1) {
            // ...
        }
    }
}
```

## Common Parsing Patterns

### Extract All Links

```rust
use pulldown_cmark::{Parser, Event, Tag};

fn extract_links(markdown: &str) -> Vec<String> {
    let parser = Parser::new(markdown);
    let mut links = Vec::new();

    for event in parser {
        if let Event::Start(Tag::Link { dest_url, .. }) = event {
            links.push(dest_url.to_string());
        }
    }

    links
}
```

### Extract Headings with IDs

```rust
use pulldown_cmark::{Event, Options, Parser, Tag};

fn extract_headings(
    markdown: &str,
) -> Vec<(String, Option<String>)> {
    let mut options = Options::empty();
    options.insert(Options::ENABLE_HEADING_ATTRIBUTES);

    let parser = Parser::new_ext(markdown, options);

    let mut headings = Vec::new();
    let mut current_heading = None;

    for event in parser {
        match event {
            Event::Start(Tag::Heading { id, .. }) => {
                current_heading = Some(
                    id.map(|s| s.to_string())
                );
            }

            Event::Text(text)
                if current_heading.is_some() =>
            {
                headings.push((
                    text.to_string(),
                    current_heading
                        .take()
                        .flatten(),
                ));
            }

            _ => {}
        }
    }

    headings
}
```

### Extract Front Matter

```rust
use pulldown_cmark::{Parser, Event, Tag, Options, MetadataBlockKind};

fn extract_front_matter(markdown: &str) -> Option<String> {
    let mut options = Options::empty();
    options.insert(Options::ENABLE_YAML_STYLE_METADATA_BLOCKS);
    let parser = Parser::new_ext(markdown, options);

    let mut in_metadata = false;
    let mut metadata = String::new();

    for event in parser {
        match event {
            Event::Start(Tag::MetadataBlock(MetadataBlockKind::YamlStyle)) => {
                in_metadata = true;
            }
            Event::Text(text) if in_metadata => {
                metadata.push_str(&text);
            }
            Event::End(TagEnd::MetadataBlock(_)) => {
                in_metadata = false;
                return Some(metadata);
            }
            _ => {}
        }
    }

    None
}
```

### Modify Images

```rust
use pulldown_cmark::{Parser, Event, Tag, CowStr};

fn prefix_image_urls(markdown: &str, prefix: &str) -> Vec<Event> {
    let parser = Parser::new(markdown);

    parser.map(|event| match event {
        Event::Start(Tag::Image { link_type, dest_url, title, id }) => {
            let new_url = format!("{}{}", prefix, dest_url);
            Event::Start(Tag::Image {
                link_type,
                dest_url: CowStr::Boxed(new_url.into_boxed_str()),
                title,
                id,
            })
        }
        _ => event
    }).collect()
}
```

### Count Document Statistics

```rust
use pulldown_cmark::{Event, Parser, Tag, TagEnd, TextMergeStream};

#[derive(Debug, Default)]
struct DocumentStats {
    word_count: usize,
    heading_count: usize,
    link_count: usize,
    code_block_count: usize,
    list_count: usize,
    paragraph_count: usize,
}

fn count_stats(markdown: &str) -> DocumentStats {
    let parser = Parser::new(markdown);
    let merged = TextMergeStream::new(parser);
    let mut stats = DocumentStats::default();

    for event in merged {
        match event {
            Event::Text(text) => {
                stats.word_count += text.split_whitespace().count();
            }
            Event::End(TagEnd::Heading(_)) => {
                stats.heading_count += 1;
            }
            Event::Start(Tag::Link { .. }) => {
                stats.link_count += 1;
            }
            Event::End(TagEnd::CodeBlock) => {
                stats.code_block_count += 1;
            }
            Event::Start(Tag::List(_)) => {
                stats.list_count += 1;
            }
            Event::Start(Tag::Paragraph) => {
                stats.paragraph_count += 1;
            }
            _ => {}
        }
    }

    stats
}
```

## Comparison with Other Parsers

### pulldown-cmark vs comrak vs markdown-rs

**pulldown-cmark:**

- Pull parsing (iterator-based)
- Zero-copy text via `CowStr`
- No AST unless you build it
- Minimal memory footprint
- Best for streaming/transformation

**comrak:**

- AST-based parsing
- Full AST always built
- Easier AST manipulation
- Higher memory usage
- Better for complex transformations

**markdown-rs:**

- Push parsing (callback-based)
- More complex state management
- Lower-level control

**When to Use pulldown-cmark:**

- Memory-constrained environments
- Streaming processing
- Simple transformations via iterators
- When you only need events, not full AST
- Performance-critical applications

## Gotchas and Common Issues

### 1. Consecutive Text Events

**Problem:** Parser emits multiple `Text` events in sequence.

**Solution:** Use `TextMergeStream`:

```rust
use pulldown_cmark::{Parser, TextMergeStream};

let merged = TextMergeStream::new(Parser::new(markdown));
```


### 2. Event Lifetime Tied to Source

**Problem:** Events borrow from source string.

```rust
// ❌ BAD: Source dropped too early
let events = {
    let markdown = String::from("# Hello");
    Parser::new(&markdown).collect::<Vec<_>>()
}; // markdown dropped - events now invalid!
```

**Solution:** Keep source alive:

```rust
// ✅ GOOD
let markdown = String::from("# Hello");
let events: Vec<_> = Parser::new(&markdown).collect();
// markdown still alive
```

### 3. Mutable Event Transformation

**Problem:** Can't modify borrowed `CowStr` in place.

**Solution:** Convert to owned:

```rust
Event::Text(text) => {
    let modified = text.replace("old", "new");
    Event::Text(CowStr::Boxed(modified.into_boxed_str()))
}
```

### 4. HTML Safety

**Problem:** `InlineHtml` and `Html` events are not sanitized.

**Solution:** Use HTML sanitization library like `ammonia`:

```rust
use pulldown_cmark::{Parser, Event};

let parser = Parser::new(markdown).map(|event| match event {
    Event::Html(html) | Event::InlineHtml(html) => {
        let clean = ammonia::clean(&html);
        Event::Html(CowStr::Boxed(clean.into_boxed_str()))
    }
    _ => event
});
```

### 5. Extension Options Required

**Problem:** Extension syntax not recognized.

**Solution:** Enable options:

```rust
// ❌ BAD: Table syntax ignored
let parser = Parser::new("| col |");

// ✅ GOOD
let parser = Parser::new_ext("| col |", Options::ENABLE_TABLES);
```



## HTML Rendering (Optional)

**Note:** This project primarily focuses on parsing. HTML rendering is available but not the primary use case.

**See Official Docs:** https://docs.rs/pulldown-cmark/0.13.0/pulldown_cmark/html/index.html for complete HTML rendering API.

### Basic HTML Output

```rust
use pulldown_cmark::{Parser, html};

let markdown = "# Hello\n\n*world*";
let parser = Parser::new(markdown);

let mut html_output = String::new();
html::push_html(&mut html_output, parser);
// Output: "<h1>Hello</h1>\n<p><em>world</em></p>\n"
```

## Integration Examples

### With Syntax Highlighting (syntect)

```rust
use pulldown_cmark::{Parser, Event, Tag, CodeBlockKind, CowStr, html};
use syntect::easy::HighlightLines;
use syntect::parsing::SyntaxSet;
use syntect::highlighting::ThemeSet;
use syntect::html::highlighted_html_for_string;

fn render_with_highlighting(markdown: &str) -> String {
    let ss = SyntaxSet::load_defaults_newlines();
    let ts = ThemeSet::load_defaults();
    let theme = &ts.themes["base16-ocean.dark"];

    let parser = Parser::new(markdown);
    let mut in_code_block = false;, TagEnd};

fn markdown_to_text(markdown: &str) -> String {
    let parser = Parser::new(markdown);
    let merged = TextMergeStream::new(parser);
    let mut output = String::new();

    for event in merged {
        match event {
            Event::Text(text) | Event::Code(text) => {
                output.push_str(&text);
            }
            Event::SoftBreak | Event::HardBreak => {
                output.push('\n');
            }
            Event::End(TagEnd::Paragraph) |
            Event::End(TagEnd::Heading(_)) => {
                output.push_str("\n\n");
            }
            _ => {}
        }
    }

    output
}
```

### Building Custom AST

```rust
use pulldown_cmark::{Parser, Event, Tag, TagEnd, HeadingLevel};

#[derive(Debug)]
enum AstNode {
    Document(Vec<AstNode>),
    Heading { level: u8, text: String, children: Vec<AstNode> },
    Paragraph(Vec<AstNode>),
    Text(String),
    Strong(Vec<AstNode>),
    Emphasis(Vec<AstNode>),
    Link { url: String, title: String, children: Vec<AstNode> },
    CodeBlock { lang: Option<String>, code: String },
}

fn build_ast(markdown: &str) -> AstNode {
    let parser = Parser::new(markdown);
    let mut stack: Vec<AstNode> = vec![AstNode::Document(Vec::new())];

    for event in parser {
        match event {
            Event::Start(Tag::Heading { level, .. }) => {
                let level_num = match level {
                    HeadingLevel::H1 => 1,
                    HeadingLevel::H2 => 2,
                    HeadingLevel::H3 => 3,
                    HeadingLevel::H4 => 4,
                    HeadingLevel::H5 => 5,
                    HeadingLevel::H6 => 6,
                };
                stack.push(AstNode::Heading {
                    level: level_num,
                    text: String::new(),
                    children: Vec::new(),
                });
            }
            Event::Start(Tag::Paragraph) => {
                stack.push(AstNode::Paragraph(Vec::new()));
            }
            Event::Start(Tag::Strong) => {
                stack.push(AstNode::Strong(Vec::new()));
            }
            Event::Start(Tag::Emphasis) => {
                stack.push(AstNode::Emphasis(Vec::new()));
            }
            Event::Start(Tag::Link { dest_url, title, .. }) => {
                stack.push(AstNode::Link {
                    url: dest_url.to_string(),
                    title: title.to_string(),
                    children: Vec::new(),
                });
            }
            Event::Start(Tag::CodeBlock(kind)) => {
                use pulldown_cmark::CodeBlockKind;
                let lang = match kind {
                    CodeBlockKind::Fenced(l) => Some(l.to_string()),
                    CodeBlockKind::Indented => None,
                };
                stack.push(AstNode::CodeBlock {
                    lang,
                    code: String::new(),
                });
            }
            Event::Text(text) => {
                if let Some(parent) = stack.last_mut() {
                    match parent {
                        AstNode::Heading { children, .. } |
                        AstNode::Paragraph(children) |
                        AstNode::Strong(children) |
                        AstNode::Emphasis(children) |
                        AstNode::Link { children, .. } => {
                            children.push(AstNode::Text(text.to_string()));
                        }
                        AstNode::CodeBlock { code, .. } => {
                            code.push_str(&text);
                        }
                        _ => {}
                    }
                }
            }
            Event::End(_) => {
                if stack.len() > 1 {
                    let node = stack.pop().unwrap();
                    if let Some(parent) = stack.last_mut() {
                        if let AstNode::Document(children) = parent {
                            children.push(node);
                        }
                    }
                }
            }
            _ => {}
        }
    }

    stack.into_iter().next().unwrap()
}
```

### Parse Task Lists

```rust
use pulldown_cmark::{Parser, Event, Tag, Options};

#[derive(Debug)]
struct Task {
    completed: bool,
    text: String,
}

fn extract_tasks(markdown: &str) -> Vec<Task> {
    let mut options = Options::empty();
    options.insert(Options::ENABLE_TASKLISTS);
    let parser = Parser::new_ext(markdown, options);

    let mut tasks = Vec::new();
    let mut in_item = false;
    let mut current_task = None;
    let mut current_text = String::new();

    for event in parser {
        match event {
            Event::Start(Tag::Item) => {
                in_item = true;
                current_text.clear();
            }
            Event::TaskListMarker(checked) => {
                current_task = Some(checked);
            }
            Event::Text(text) if in_item => {
                current_text.push_str(&text);
            }
            Event::End(TagEnd::Item) if current_task.is_some() => {
                tasks.push(Task {
                    completed: current_task.take().unwrap(),
                    text: current_text.clone(),
                });
                in_item = false;
            }
            Event::End(TagEnd::Item) => {
                in_item = false;
            }
            _ => {}
        }
    }

    tasks
}
```

### Parse Tables

```rust
use pulldown_cmark::{Parser, Event, Tag, TagEnd, Options, Alignment};

#[derive(Debug)]
struct Table {
    headers: Vec<String>,
    alignments: Vec<Alignment>,
    rows: Vec<Vec<String>>,
}

fn extract_table(markdown: &str) -> Option<Table> {
    let mut options = Options::empty();
    options.insert(Options::ENABLE_TABLES);
    let parser = Parser::new_ext(markdown, options);

    let mut table = None;
    let mut in_header = false;
    let mut in_row = false;
    let mut current_cell = String::new();
    let mut current_row = Vec::new();

    for event in parser {
        match event {
            Event::Start(Tag::Table(alignments)) => {
                table = Some(Table {
                    headers: Vec::new(),
                    alignments: alignments.clone(),
                    rows: Vec::new(),
                });
            }
            Event::Start(Tag::TableHead) => {
                in_header = true;
            }
            Event::End(TagEnd::TableHead) => {
                in_header = false;
            }
            Event::Start(Tag::TableRow) => {
                in_row = true;
                current_row.clear();
            }
            Event::End(TagEnd::TableRow) => {
                in_row = false;
                if let Some(ref mut t) = table {
                    if !current_row.is_empty() {
                        t.rows.push(current_row.clone());
                    }
                }
            }
            Event::Start(Tag::TableCell) => {
                current_cell.clear();
            }
            Event::End(TagEnd::TableCell) => {
                if in_header {
                    if let Some(ref mut t) = table {
                        t.headers.push(current_cell.clone());
                    }
                } else if in_row {
                    current_row.push(current_cell.clone());
                }
            }
            Event::Text(text) => {
                current_cell.push_str(&text);
            }
            _ => {}
        }
    }

    tablet output = String::new();
    html::push_html(&mut output, transformed);
    output
}
```

### Table of Contents Generator

```rust
use pulldown_cmark::{Parser, Event, Tag, HeadingLevel};

#[derive(Debug)]
struct TocEntry {
    level: u8,
    text: String,
    id: Option<String>,
}

fn generate_toc(markdown: &str) -> Vec<TocEntry> {
    let parser = Parser::new_ext(markdown, Options::ENABLE_HEADING_ATTRIBUTES);
    let mut toc = Vec::new();
    let mut in_heading = None;
    let mut heading_text = String::new();

    for event in parser {
        match event {
            Event::Start(Tag::Heading { level, id, .. }) => {
                in_heading = Some((level, id.map(|s| s.to_string())));
                heading_text.clear();
            }
            Event::Text(text) if in_heading.is_some() => {
                heading_text.push_str(&text);
            }
            Event::End(TagEnd::Heading(_)) if in_heading.is_some() => {
                let (level, id) = in_heading.take().unwrap();
                let level_num = match level {
                    HeadingLevel::H1 => 1,
                    HeadingLevel::H2 => 2,
                    HeadingLevel::H3 => 3,
                    HeadingLevel::H4 => 4,
                    HeadingLevel::H5 => 5,
                    HeadingLevel::H6 => 6,
                };
                toc.push(TocEntry {
                    level: level_num,
                    text: heading_text.clone(),
                    id,
                });
            }
            _ => {}
        }
    }

    toc
}
```

### Markdown to Plain Text

```rust
use pulldown_cmark::{Parser, Event, TextMergeStream};

fn markdown_to_text(markdown: &str) -> String {
    let parser = Parser::new(markdown);
    let merged = TextMergeStream::new(parser);
    let mut output = String::new();

    for event in merged {
        match event {
            Event::Text(text) | Event::Code(text) => {
                output.push_str(&text);
            }
            Event::SoftBreak | Event::HardBreak => {
                output.push('\n');
            }
            Event::End(TagEnd::Paragraph) |
            Event::End(TagEnd::Heading(_)) => {
                output.push_str("\n\n");
            }
            _ => {}
        }
    }

    output
}
```

## Testing Strategies

### Event Snapshot Testing

```rust
#[cfg(test)]
mod tests {
    use pulldown_cmark::{Parser, Event};

    #[test]
    fn test_parse_events() {
        let markdown = "**bold** and *italic*";
        let events: Vec<_> = Parser::new(markdown).collect();

        insta::assert_debug_snapshot!(events);
    }
}
```

### Round-Trip Testing

```rust
#[test]
fn test_round_trip() {
    let markdown = "# Heading\n\nParagraph";

    let parser = Parser::new(markdown);
    let mut html = String::new();
    html::push_html(&mut html, parser);

    // Parse the HTML back (would need HTML parser)
    // Verify structure preserved
}
```

### Fuzzing

```rust
#![no_main]
use libfuzzer_sys::fuzz_target;
use pulldown_cmark::{Parser, html};

fuzz_target!(|data: &[u8]| {
    if let Ok(s) = std::str::from_utf8(data) {
        let parser = Parser::new(s);
        let mut output = String::new();
        html::push_html(&mut output, parser);
    }
});
```

## Additional Resources

- **CommonMark Spec:** https://spec.commonmark.org/
- **GitHub Flavored Markdown:** https://github.github.com/gfm/
- **Pull Parsing Concept:** https://www.xmlpull.org/history/index.html
- **Benchmarks:** https://github.com/pulldown-cmark/pulldown-cmark/tree/master/benches
- **Migration Guide:** https://github.com/pulldown-cmark/pulldown-cmark/blob/master/CHANGELOG.md

## Version Notes

**Version 0.13.0** (Current)

- Separate `Tag` and `TagEnd` enums
- Heading attributes support
- Metadata block support
- Math support (LaTeX-style)
- Improved GFM compatibility

**Breaking Changes from 0.9.x:**

- `Event::End(Tag)` → `Event::End(TagEnd)`
- Heading structure changed to include `id`, `classes`, `attrs`
- Some option names changed

**Migration:** Check CHANGELOG for detailed migration guide.
