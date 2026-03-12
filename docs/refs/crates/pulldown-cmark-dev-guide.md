# Developer Guide

pulldown-cmark uses a two-pass parsing strategy with a pull parser architecture to efficiently parse Markdown into HTML. This guide explains the internal workings of the library for developers who want to contribute or better understand how it works.

## High-Level Architecture

The parser operates in two main passes:

1. **First Pass (Block Structure)**: The first pass scans the document and builds a tree structure representing block-level elements like paragraphs, lists, code blocks, etc. This establishes the hierarchical structure of the document.

2. **Second Pass (Inline Processing)**: The second pass processes inline elements like emphasis, links and code spans within the blocks identified by the first pass. This is done in a streaming fashion as events are requested.

The library uses a pull parser design, which means:

- Instead of pushing events to a callback or building a complete AST, it provides an iterator interface that lets consumers pull events as needed
- It enables flexible transformation of the event stream before rendering

Key components:

- `Parser`: The main entry point that implements the Iterator trait for Events
- `Tree`: A Vec-based data structure that holds the block structure
- `Event`: An enum representing the different Markdown elements
- `HtmlWriter`: Renders the event stream as HTML

## Performance Characteristics

The parser is designed for high performance:

- Performance is intended to be linear with respect to the size of the input text
- String handling uses copy-on-write semantics to avoid unnecessary allocations
- SIMD optimizations are available for scanning text on x86_64

## Extending the Parser

The parser can be extended in several ways:

- New syntax extensions can be added by implementing new scan functions
- The event stream can be transformed using Iterator adaptors
- Custom renderers can be built by consuming events
- The HTML renderer can be customized through options

## Directory Structure

```
src/
  firstpass.rs   - First pass block structure parsing
  scanners.rs    - Low-level text scanning functions
  parse.rs       - Main parser implementation
  html.rs        - HTML renderer
  tree.rs        - Tree data structure
  entities.rs    - HTML entity handling
  strings.rs     - String types and utilities
```

Subsequent chapters cover each of these components in detail:

1. [Block Structure Parsing](./dev/block-parsing.md)
2. [Inline Processing](./dev/inline-processing.md)
3. [String Handling](./dev/string-handling.md)
4. [HTML Generation](./dev/html-generation.md)
5. [Performance Optimizations](./dev/performance.md)
6. [Adding Extensions](./dev/extensions.md)

# Block Structure Parsing

The first pass of pulldown-cmark's parsing process handles block-level elements and constructs the basic document structure.

It roughly corresponds to Phase 1 of the CommonMark spec appendix ["A Parsing Strategy"](https://spec.commonmark.org/0.31.2/#appendix-a-parsing-strategy).
This chapter explains how the block parsing works in pulldown-cmark.

## Overview

Block parsing is implemented in `firstpass.rs` and has two main responsibilities:

1. Identifying block-level elements like paragraphs, lists, and code blocks
2. Building a tree structure representing the nesting of these blocks

The block parser operates line-by-line, maintaining a stack of currently open blocks (called the "spine") and handling both container blocks (like blockquotes) and leaf blocks (like paragraphs).

## Block Types

The main block types handled by the first pass are:

- Container blocks:
  - Block quotes
  - Lists (ordered and unordered)
  - List items
  - Footnote definitions

- Leaf blocks:
  - Paragraphs
  - Headings (ATX and Setext style)
  - Code blocks (fenced and indented)
  - HTML blocks
  - Thematic breaks (horizontal rules)
  - Tables (with GFM extension)

## The Parsing Process

The block parsing process works like this:

1. Input text is processed line by line

2. For each line:
   - Check if it continues any blocks from the current spine
   - Scan for the start of any new blocks
   - Handle transitions between blocks
   - Track indentation and container prefixes

3. Build tree nodes for each block encountered

4. Handle tight/loose list detection

Here's a simplified example of how a nested list is parsed:

```markdown
- First item
  - Nested item
    with continuation
- Second item
```

The parser:
1. Recognizes the first `-` as starting a list and list item
2. Sees the next `-` as starting a nested list
3. Identifies the indented line as continuing the nested item
4. Recognizes the unindented `-` as closing the nested list

## Tree Construction

The block structure is stored in a `Tree<Item>` where each node contains:

```rust
struct Item {
    start: usize,      // Start byte offset
    end: usize,        // End byte offset
    body: ItemBody,    // Type and attributes
}

struct Node<T> {
    child: Option<TreeIndex>,  // First child node
    next: Option<TreeIndex>,   // Next sibling
    item: T,                   // Node data (T = Item in our tree)
}
```

The tree is built incrementally as blocks are parsed. Key operations:

- `push()`: Move down into a new block's children
- `pop()`: Move back up to the parent block
- `append()`: Add a new sibling block
- `truncate_siblings()`: End open blocks at a certain point

## Container Block Handling

Container blocks like blockquotes and lists require special handling:

1. Track container prefixes (>, -, 1., etc)
2. Calculate correct indentation levels
3. Handle lazy continuation lines
4. Determine tight/loose status for lists

The `LineStart` struct helps manage this by:
- Tracking indentation and remaining space
- Scanning container markers
- Handling tab stops correctly

## Leaf Block Processing

Leaf blocks are handled by specific scanner functions that:

1. Identify the block type
2. Calculate its bounds
3. Handle internal structure like table columns
4. Manage transitions between blocks

For example, table parsing:
```rust
pub(crate) fn scan_table_head(data: &[u8]) -> (usize, Vec<Alignment>) {
    // Check initial conditions
    let (mut i, spaces) = calc_indent(data, 4);
    if spaces > 3 || i == data.len() {
        return (0, vec![]);
    }

    // Parse cells and alignments
    let mut cols = vec![];
    let mut active_col = Alignment::None;
    // ...

    // Return parsed structure
    (i, cols)
}
```

## Error Recovery

The parser is designed to be robust and recover from invalid syntax:

- Malformed containers fall back to paragraphs
- Invalid indentation is normalized
- Unclosed blocks are implicitly closed
- HTML parsing has fallback modes

This ensures it can handle real-world Markdown without failing.

## Interfacing with Inline Parsing

The block parser prepares for inline parsing by:

1. Identifying inline-containing blocks
2. Marking potential inline boundaries (e.g. `MaybeLinkOpen`)
3. Providing context (like table cells)
4. Tracking source positions

The tree structure is then used by the inline parser to process inline elements within the appropriate blocks.

## Implementation Notes

Some key implementation details:

- Line scanning is optimized using SIMD on x86_64
- The tree structure uses indexed nodes to avoid lifetimes
- Container context is maintained in a stack-like structure
- Source positions are tracked for use by the inline parser and [`OffsetIter`](https://docs.rs/pulldown-cmark/latest/pulldown_cmark/struct.OffsetIter.html)

The block parser aims to be:
- Fast for common cases
- Memory efficient
- Robust against bad input
- Compliant with CommonMark

# Inline Processing

The second pass of pulldown-cmark's parsing process handles inline elements like emphasis, links, and code spans.

## Overview

Inline processing happens during event iteration rather than as a separate full-document pass. When the parser encounters a block that can contain inlines, it processes the inline elements on demand.

The main inline elements handled are:

- Emphasis and strong emphasis (* and _)
- Code spans (`)
- Links and images
- HTML tags and entities
- Autolinks
- Extension elements like strikethrough and math

## Processing Model

The inline processor:

1. Scans text for special characters
2. Identifies potential inline markers
3. Resolves matched pairs (like * for emphasis)
4. Handles nested elements
5. Processes escapes and entities

## Delimiter Handling

Emphasis-type elements use a sophisticated delimiter handling system:

1. Identify delimiter runs (consecutive `*`, `_`, etc)
2. Determine if they can open and/or close
3. Match pairs according to CommonMark rules
4. Handle nested cases correctly


The `InlineStack` struct manages this:

```rust
struct InlineStack {
    stack: Vec<InlineEl>,
    lower_bounds: [usize; 9],
}

struct InlineEl {
    start: TreeIndex,
    count: usize,      // Number of delimiters
    run_length: usize, // Full run length
    c: u8,            // Delimiter character
    both: bool,       // Can both open and close
}
```

## Link Processing

Link processing involves:

1. Finding link text in brackets
2. Handling different link types:
   - Inline `[text](url)`
   - Reference `[text][ref]`
   - Collapsed `[ref][]`
   - Shortcut `[ref]`

3. Resolving references in link definitions
4. Processing link destinations and titles

The link processor maintains a stack to handle nested links and images:

```rust
struct LinkStackEl {
    node: TreeIndex,
    ty: LinkStackTy,
}

enum LinkStackTy {
    Link,
    Image,
    Disabled, // For nested links
}
```

## Code Spans

Code span processing has special rules:

1. Match backtick sequences of equal length
2. Handle backslash escapes
3. Strip leading/trailing spaces according to spec
4. Prevent misinterpreting internal backticks

## HTML Processing

HTML blocks have already been recognized by the block parser. What remains is inline HTML tags between normal text. Handling this involves:

1. Identifying HTML constructs:
   - Tags
   - Comments
   - CDATA sections
   - Processing instructions

2. Validating structure
3. Preserving content exactly
4. Handling entities

The HTML processor uses a state machine to track context:

```rust
struct HtmlScanGuard {
    cdata: usize,
    processing: usize,
    declaration: usize,
    comment: usize,
}
```

## String Handling

Inline processing needs efficient string handling:

1. Copy-on-write strings to avoid allocation
2. Smart handling of escaped characters
3. Entity resolution
4. UTF-8 awareness

The `CowStr` type provides this and is documented in detail [here](./string-handling.md).

## Event Generation

As inline elements are processed, they generate events:

1. Start/end events for container elements
2. Text events for content
3. Specialized events for atomic elements
4. Source position tracking

Events are yielded in document order:

```rust
enum Event<'a> {
    Start(Tag<'a>),
    End(TagEnd),
    Text(CowStr<'a>),
    Code(CowStr<'a>),
    Html(CowStr<'a>),
    // ...
}
```
# String Handling

pulldown-cmark uses a specialized string type system optimized for the specific needs of parsing and representing Markdown content. This chapter explains the key components and design decisions of this system.

## Overview

The library uses two main custom string types:

- `CowStr`: A three-word copy-on-write string type that can be owned, borrowed, or inlined
- `InlineStr`: A small string optimized for very short content that can be stored inline

These types are designed to balance several requirements:

- Efficient memory usage for the many small string fragments in Markdown
- Zero-copy operation where possible by borrowing from input
- Good performance for string operations needed during parsing
- Safety and correct handling of Unicode

## CowStr Type

The `CowStr` enum is the primary string type used throughout the library. It has three variants:

```rust
pub enum CowStr<'a> {
    Boxed(Box<str>),
    Borrowed(&'a str),
    Inlined(InlineStr),
}
```

Each variant serves a specific purpose:

- `Borrowed`: References strings from the input text with zero copying
- `Inlined`: Stores very short strings (up to ~22 bytes on 64-bit systems) directly in the enum
- `Boxed`: Owns longer strings that need to be heap allocated

The key feature is that `CowStr` is exactly three words in size regardless of which variant is used. This fixed size makes it efficient to store and pass around.

### Example Usage

```rust
// Borrow from input when possible
let borrowed: CowStr = input[0..15].into();

// Single chars are inlined
let inline: CowStr = 'x'.into();

// Longer strings are boxed
let boxed: CowStr = "a rather long string...".to_string().into();
```

## InlineStr Type

`InlineStr` is a small string type that can store short strings inline without heap allocation. It consists of:

- A fixed-size byte array sized to three machine words minus 2 bytes
- A length field using the remaining byte

```rust
pub struct InlineStr {
    inner: [u8; MAX_INLINE_STR_LEN],
    len: u8,
}
```

The size is chosen to allow `InlineStr` to be stored directly in `CowStr` without increasing its overall size. On 64-bit systems this allows for strings up to 22 bytes.

```rust
const MAX_INLINE_STR_LEN: usize = 3 * std::mem::size_of::<isize>() - 2;
```

Key characteristics:

- Fixed size with no heap allocation
- UTF-8 encoded
- Length limited by available space
- Copy-able since it's a fixed-size type

## Converting Between Types

The library provides conversions between various string types:

```rust
// From str
let cow: CowStr = "text".into();

// From String
let cow: CowStr = string.into();

// From char
let cow: CowStr = 'x'.into();

// From std::borrow::Cow
let cow: CowStr = std_cow.into();
```

It also provides methods to convert into owned types:

```rust
let string: String = cow_str.into_string();
let static_cow: CowStr<'static> = cow_str.into_static();
```

## Performance Considerations

The string system is designed for the performance characteristics needed by a Markdown parser:

- Minimal copying of input text
- Efficient handling of many small string fragments
- Fast comparisons for link matching

## Unicode Handling

As with Rust built-in strings, these types maintain proper UTF-8 encoding:

- Input validation occurs when creating `InlineStr`
- String operations preserve valid UTF-8
- Character boundaries are respected when manipulating strings

## Implementation Details

When implementing new features that handle strings, follow these guidelines:

1. Use `CowStr` as the primary string type
2. Borrow from input when possible using `Borrowed` variant
3. Use `InlineStr` for short string literals
4. Convert to `String` only when necessary for buffering
5. Be aware of UTF-8 encoding requirements
6. Consider memory usage patterns when choosing string operations

# HTML Generation

This chapter explains how pulldown-cmark generates HTML output from Markdown events.

## Overview

HTML generation is implemented in the `html` module and consists of two main components:

1. The `HtmlWriter` struct which manages state and writes HTML tags
2. Helper functions for converting events to HTML tags and handling special cases

The HTML generation process works by:
1. Taking an iterator of Markdown events
2. Converting each event into corresponding HTML tags
3. Managing state for special cases like tables and tight lists
4. Writing the HTML tags to the provided output

## The HtmlWriter

The core type responsible for HTML generation is `HtmlWriter`:

```rust
struct HtmlWriter<'a, I, W> {
    iter: I,        // Iterator supplying events
    writer: W,      // Writer to write to
    end_newline: bool,  // Whether last write ended with newline
    in_non_writing_block: bool,  // In metadata block (no output)
    table_state: TableState,  // Current state for table processing
    table_alignments: Vec<Alignment>,  // Column alignments for current table
    table_cell_index: usize,  // Current cell index in table row
    numbers: HashMap<CowStr<'a>, usize>,  // For footnote numbering
}
```

The writer keeps track of:

- The current table state (head vs body)
- Table column alignments
- Current cell index
- Footnote numbering
- Whether we're in a non-writing block like metadata
- Whether the last write ended with a newline

## Event Processing

The main event processing loop lives in `HtmlWriter::run()`. For each event:

1. The event is matched and dispatched to the appropriate handler
2. HTML tags are written based on the event type
3. State is updated as needed

Key event handling patterns:

### Block Elements

Block elements like paragraphs, headings, lists etc. are wrapped in HTML tags:

```rust
match event {
    Start(Tag::Paragraph) => write("<p>"),
    End(EndTag::Paragraph) => write("</p>\n"),
    // etc
}
```

### Inline Elements

Inline elements like emphasis and links are handled similarly but without newlines:

```rust
match event {
    Start(Tag::Emphasis) => write("<em>"),
    End(EndTag::Emphasis) => write("</em>"),
    // etc
}
```

### Text Content

Text content is HTML escaped and written directly:

```rust
match event {
    Text(text) => escape_html_body_text(&mut writer, &text),
    // etc
}
```

### Complex Elements

More complex elements like tables require managing state:

```rust
match event {
    Start(Tag::Table(alignments)) => {
        self.table_alignments = alignments;
        self.write("<table>")?;
    }
    // etc
}
```

## HTML Safety

The functions `escape_html()` and ``escape_href()`` are used throughout the library for escaping special characters. The escaping functions live in the `pulldown-cmark-escape` crate.

## Writer Interface

The HTML writer is generic over the writer type `W`, allowing output to:

- Strings via `fmt::Write`
- Files/IO via `io::Write`

This generic design lets users choose the most efficient output method for their use case. For example:
- Using `String` is convenient for in-memory processing and testing
- Using `BufWriter<File>` is efficient for writing directly to disk
- Using a network socket allows streaming HTML over a connection
- Using a custom writer enables special handling like compression or logging

The `StrWrite` trait provides a common interface to abstract over these different writers:

```rust
pub trait StrWrite {
    type Error;
    fn write_str(&mut self, s: &str) -> Result<(), Self::Error>;
}
```

This abstraction over the writer type means the HTML generation code can focus on correct tag generation and structure without worrying about the specific output destination. It also allows users to easily integrate pulldown-cmark's HTML output into their existing I/O pipelines.

## Public API

The main public API consists of:

```rust
// Write HTML to a String
pub fn push_html<'a, I>(s: &mut String, iter: I)
where I: Iterator<Item = Event<'a>>

// Write HTML to an IO writer
pub fn write_html_io<'a, I, W>(writer: W, iter: I) -> io::Result<()>
where I: Iterator<Item = Event<'a>>,
      W: io::Write

// Write HTML to a fmt writer
pub fn write_html_fmt<'a, I, W>(writer: W, iter: I) -> fmt::Result
where I: Iterator<Item = Event<'a>>,
      W: fmt::Write
```

## Performance Considerations

HTML generation aims to be efficient by:

1. Minimizing string allocations
2. Using buffered writers
3. Avoiding recursion in the core loop

Note:

```rust
// Using unbuffered writers (like Files) will be slow
// Wrap them in BufWriter for better performance
let file = BufWriter::new(File::create("output.html")?);
write_html_io(file, parser);
```

This ensures good performance even with large documents.

## Customization

The HTML output can be customized by:

1. Using a custom writer implementation
2. Preprocessing the event stream
3. Post-processing the HTML output
4. Using the parser options to enable/disable features

# Performance Optimizations

This chapter covers the key performance optimizations implemented in pulldown-cmark. The library uses several techniques to achieve fast Markdown parsing while maintaining standards compliance and a clean architecture.

## SIMD-Accelerated Character Scanning

One of the most performance-critical operations in Markdown parsing is scanning text for special characters that may indicate inline markup. pulldown-cmark uses SIMD (Single Instruction Multiple Data) instructions on x86_64 platforms to accelerate this scanning.

The SIMD optimization is implemented in `scanners.rs` and operates by:

1. Creating a lookup table of special characters (like `*`, `_`, etc.)
2. Loading 16 bytes at a time into a SIMD register
3. Performing parallel lookups to identify special characters
4. Generating a bitmask indicating which bytes matched

```rust
// Example from firstpass.rs showing the core scanning logic
#[target_feature(enable = "ssse3")]
unsafe fn compute_mask(lut: &[u8; 16], bytes: &[u8], ix: usize) -> i32 {
    let bitmap = _mm_loadu_si128(lut.as_ptr() as *const __m128i);
    let input = _mm_loadu_si128(bytes.as_ptr().add(ix) as *const __m128i);
    let bitset = _mm_shuffle_epi8(bitmap, input);
    let higher_nibbles = _mm_and_si128(_mm_srli_epi16(input, 4), _mm_set1_epi8(0x0f));
    let bitmask = _mm_shuffle_epi8(bitmask_lookup, higher_nibbles);
    let tmp = _mm_and_si128(bitset, bitmask);
    let result = _mm_cmpeq_epi8(tmp, bitmask);
    _mm_movemask_epi8(result)
}
```

This SIMD optimization can provide significant speedups when processing large documents, since character scanning is such a common operation. The code falls back to scalar processing when SIMD is not available.

## Memory-Efficient String Storage

The library uses a custom string type `CowStr` that can represent strings. Refer to the [string handling](./string-handling.md) documentation for more details on the performance optimizations inherent to this type.

## Tree Structure Optimization

The AST (Abstract Syntax Tree) is stored in a vec-based tree structure that provides:

1. Fast node creation during parsing
2. Efficient tree traversal
3. Memory locality from vector storage

```rust
pub(crate) struct Tree<T> {
    nodes: Vec<Node<T>>,
    spine: Vec<TreeIndex>,
    cur: Option<TreeIndex>,
}

pub(crate) struct Node<T> {
    pub child: Option<TreeIndex>,
    pub next: Option<TreeIndex>,
    pub item: T,
}
```

Key optimizations in the tree structure include:

- Using indices instead of pointers for node references
- Maintaining a "spine" for fast access to ancestor nodes
- Storing nodes contiguously in a vector for better cache usage
- Using non-zero indices to save space in option types

## Protection Against Pathological Input

It is important that the parser performance remain linear with respect to the input length, otherwise the parser would find itself vulnerable to potential DOS attacks. This may not be important for all consumers, but for anyone depending on the library to handle user generated content this is critical.

Several protections are in place to prevent quadratic time or memory usage on malicious input:

1. Link nesting depth is limited:
```rust
pub(crate) const LINK_MAX_NESTED_PARENS: usize = 32;
```

2. Table column expansion is bounded:
```rust
// Limit to prevent quadratic growth from empty cells
const MAX_AUTOCOMPLETED_CELLS: usize = 1 << 18;
```

3. Link reference expansion tracking:
```rust
// Track expansion to prevent quadratic growth from reference definitions
let mut link_ref_expansion_limit: usize = text.len().max(100_000);
```


## Key Performance Considerations

When using the library, keep in mind:

1. SIMD optimizations require the `simd` feature and x86_64 platform
2. Large documents benefit most from SIMD scanning
4. The parser is designed for streaming, allowing incremental processing
5. Pathological input protection may limit processing of extremely nested or repetitive content

## Benchmarking

The library includes benchmarks to measure performance of key operations:

- String handling with different storage strategies
- Tree operations
- Full document parsing
- Pathological input cases

When making changes that could affect performance, run the benchmarks to ensure optimizations are effective:

```bash
cargo bench
```

Note that some optimizations (like SIMD) are platform-specific, so testing on multiple platforms may be necessary.

# Adding Extensions

This guide explains how to add new extensions to pulldown-cmark. Extensions allow you to parse additional Markdown syntax beyond the CommonMark specification.

If you are looking to get your extension merged upstream, it's a good idea to discuss it with the maintainers before getting to work.

## Overview

Adding an extension typically requires:

1. Adding a feature flag in the `Options` bitflags
2. Adding any new data structures needed to represent the extension's AST nodes
3. Implementing block parsing in `firstpass.rs` if the extension adds block-level elements
4. Implementing inline parsing in `parse.rs` if the extension adds inline elements
5. Adding HTML rendering support in `html.rs`
6. Adding tests to verify the extension works correctly

Let's walk through each of these steps in detail.

## Adding the Feature Flag

Extensions are controlled via the `Options` bitflags defined in `lib.rs`. Add a new constant using the next available bit:

```rust
bitflags::bitflags! {
    pub struct Options: u32 {
        // Existing options...
        const ENABLE_MY_EXTENSION = 1 << N; // N is next available bit
    }
}
```

This allows users to enable your extension with:

```rust
let mut options = Options::empty();
options.insert(Options::ENABLE_MY_EXTENSION);
```

## Adding AST Data Structures

Extensions often need new AST node types to represent their syntax. These are defined in several places:

- `Tag` enum in `lib.rs` for container elements
- `TagEnd` enum in `lib.rs` for end tags
- `Event` enum in `lib.rs` for new event types
- `ItemBody` enum in `parse.rs` for internal AST nodes

For example, the tables extension defines:

```rust
// In lib.rs
pub enum Tag<'a> {
    // ...
    Table(Vec<Alignment>),
    TableHead,
    TableRow,
    TableCell,
}

// In parse.rs
pub(crate) enum ItemBody {
    // ...
    Table(AlignmentIndex),
    TableHead,
    TableRow,
    TableCell,
}
```

Follow existing patterns for naming and make sure to implement all the necessary traits (`Debug`, `Clone`, etc.).

## Implementing Block Parsing

If your extension adds block-level elements (like tables, footnotes, etc.), you'll need to:

1. Add scanning functions in `scanners.rs` to detect your syntax
2. Add parsing logic in `firstpass.rs` to build the block structure
3. Update the `scan_containers()` function if your blocks can be nested

For example, the tables extension adds:

```rust
// In scanners.rs
pub(crate) fn scan_table_head(data: &[u8]) -> (usize, Vec<Alignment>) {
    // Scan table header row syntax...
}

// In firstpass.rs
impl<'a> FirstPass<'a, 'b> {
    fn parse_table(&mut self, ...) -> Option<usize> {
        // Parse table structure...
    }
}
```

Follow these guidelines when implementing block parsing:

- Use the `scan_` prefix for low-level scanning functions
- Make scanning functions return the number of bytes consumed
- Handle edge cases like empty lines and indentation
- Properly integrate with the container block structure
- Follow the parsing strategies used by existing extensions

## Implementing Inline Parsing

If your extension adds inline elements (like strikethrough, math, etc.), you'll need to:

1. Add marker detection in `parse_line()` in `firstpass.rs`
2. Add opener/closer matching logic in `handle_inline()`
3. Add conversion from internal AST to events

For example, the strikethrough extension adds:

```rust
// In firstpass.rs
impl<'a, 'b> FirstPass<'a, 'b> {
    fn parse_line(&mut self, ..) -> (usize, Option<Item>) {
        match byte {
            b'~' => {
                // Handle tilde markers...
            }
        }
    }
}
```

Inline parsing tips:

- Use the `MaybeX` pattern for markers that need matching
- Handle backslash escaping correctly
- Support nested inline elements
- Follow CommonMark rules for [flanking](https://spec.commonmark.org/0.31.2/#delimiter-run) conditions
- Reuse existing inline parsing infrastructure

## Adding HTML Rendering

HTML rendering is handled in `html.rs`. You'll need to:

1. Add HTML tag generation for your new elements
2. Update the `body_to_tag_end()` and `item_to_event()` functions
3. Handle any special rendering requirements

For example:

```rust
// In html.rs
impl<'a, I, W> HtmlWriter<'a, I, W> {
    fn start_tag(&mut self, tag: Tag<'a>) -> Result<(), W::Error> {
        match tag {
            Tag::MyExtension => {
                self.write("<my-extension>")
            }
            // ...
        }
    }
}
```

HTML rendering tips:

- Follow HTML5 standards
- Handle escaping properly
- Consider accessibility
- Test in different contexts

## Testing

Add tests to verify your extension works correctly. pulldown-cmark is principally tested with spec documents, which are Markdown files containing test cases. Each extension should have a file under `specs/` explaining how the feature works along with test cases. Have a look at the existing specs for inspiration.
Other kinds of testing you should consider:

1. Unit tests alongside implementation
2. Integration tests in `tests/`
3. round-trip tests
4. Edge case tests
5. Interaction tests with other extensions

For example:

```rust
#[test]
fn test_my_extension() {
    let input = "Test my extension syntax";
    let mut options = Options::empty();
    options.insert(Options::ENABLE_MY_EXTENSION);
    let parser = Parser::new_ext(input, options);
    // Test parsing result...
}
```

Testing tips:

- Test both positive and negative cases
- Test interactions with other syntax
- Test error conditions
- Test HTML output
- Test with different options enabled
- Run the different fuzzers to find crashes (`fuzz/` parse target) and performance issues (`dos-fuzzer/`)

## Example: Adding Subscript Extension

Here's a complete example of adding a hypothetical subscript extension that uses `~text~` for subscript:

```rust
// In lib.rs
bitflags::bitflags! {
    pub struct Options: u32 {
        const ENABLE_SUBSCRIPT = 1 << 15;
    }
}

pub enum Tag<'a> {
    Subscript,
}

// In parse.rs
pub(crate) enum ItemBody {
    MaybeSubscript(usize),  // For opener/closer matching
    Subscript,
}

impl<'a, F> Parser<'a, F> {
    fn parse_line(&mut self, ..) -> (usize, Option<Item>) {
        match byte {
            b'~' => {
                // Handle subscript markers...
            }
        }
    }
}

// In html.rs
impl<'a, I, W> HtmlWriter<'a, I, W> {
    fn start_tag(&mut self, tag: Tag<'a>) -> Result<(), W::Error> {
        match tag {
            Tag::Subscript => self.write("<sub>"),
        }
    }
}
```

## Tips and Best Practices

- Study existing extensions for patterns to follow
- Keep parsing efficient
- Handle edge cases gracefully
- Document your extension thoroughly
- Consider adding feature flags for subfeatures
- Follow CommonMark principles where possible
- Test extensively
- Consider compatibility with other extensions

## Common Pitfalls

- Not handling nested elements correctly
- Improper escaping in HTML output
- Not following CommonMark precedence rules
- Inefficient parsing of large documents
- Poor error recovery
- Not handling edge cases
- Breaking existing syntax
- Not documenting limitations

## Further Reading

- [CommonMark Spec](https://spec.commonmark.org/)
- [GitHub Flavored Markdown Spec](https://github.github.com/gfm/)
- [Existing pulldown-cmark extension specs](https://pulldown-cmark.github.io/pulldown-cmark/specs.html)
- [HTML5 Spec](https://html.spec.whatwg.org/)
