# Policy-Based Scanner Design Research

## Executive Summary

This document researches patterns for designing a policy-based scanner for markdown metadata extraction in Lithos. Based on analysis of production systems (tree-sitter, pulldown-cmark, nom, ripgrep, clippy) and Lithos's current architecture, I recommend:

1. **When to scan**: Scan **after structured parsing** (on `RawListItem`, `RawHeading`, etc.) rather than raw text or events
2. **Architecture**: Use **trait-based rule composition** with fast-path byte checks
3. **Extension**: Support **runtime registration** via trait objects + config-driven enable/disable
4. **Performance**: Leverage **byte-level fast paths** before UTF-8 decoding, zero-copy slices

---

## 1. When to Scan? Architecture Analysis

### Option A: Raw Text Scanning (ripgrep approach)

**How it works**: Byte-level regex engine operating on raw strings before any parsing.

**Example from ripgrep**:
```rust
// Fast path: SIMD literal matching before full regex
if input.contains_literal("::") {
    regex.find_iter(input)  // Only run full matcher on candidates
}
```

**Pros**:
- Maximum performance (SIMD-friendly byte operations)
- Simple dependencies (no AST required)
- Easy to parallelize (independent chunks)

**Cons**:
- No structural context (can't skip code blocks, links)
- False positives in code spans, URLs
- Must re-validate context later

**Verdict for Lithos**: ❌ **Not recommended**. Obsidian metadata is context-sensitive:
- Tags inside code blocks should be ignored
- Inline fields inside wikilinks should be ignored
- Block refs only valid at line end

---

### Option B: Event Stream Scanning (pulldown-cmark approach)

**How it works**: Scan during event stream traversal, filtering based on event context.

**Example from pulldown-cmark**:
```rust
// Extension pattern from their codebase
for (event, range) in Parser::new_ext(text, options).into_offset_iter() {
    match event {
        Event::Text(text) if !in_code_block => scan_for_metadata(&text),
        Event::Code(_) => { /* skip */ },
        _ => {}
    }
}
```

**Pros**:
- Structural awareness (know when inside code/links)
- Can skip irrelevant events cheaply
- Natural fit for pulldown-cmark integration

**Cons**:
- Text may be fragmented across multiple events
- Must accumulate fragments before scanning
- Event stream doesn't expose all structural boundaries

**Verdict for Lithos**: ⚠️ **Partially useful**. Current architecture already does this (see `parser.rs:83-85`), but fragments are accumulated anyway.

---

### Option C: Post-Parse Structured Scanning (tree-sitter approach)

**How it works**: Build structured AST first, then scan specific nodes with clear boundaries.

**Example from tree-sitter external scanner**:
```rust
// External scanner knows parser's structural context
fn scan(lexer: &TSLexer, valid_symbols: &[bool]) -> bool {
    if valid_symbols[INDENT] {
        // Only scan for indent when parser expects it
        try_scan_indent(lexer)
    }
}
```

**Current Lithos flow**:
```rust
// parser.rs: Events → Fragments → Blocks
TextMergeWithOffset::new(normalized)
    → accumulate fragments into RawHeading/RawListItem
    → BlockExtractor::scan_fragments() on completed blocks

// extractor.rs:142-150
fn scan_fragments(&self, fragments: &[TextFragment]) -> ScannedRawArtifacts {
    let scannable_ranges: Vec<Range<usize>> = fragments
        .iter()
        .filter(|f| f.is_scannable)  // Already knows structural context!
        .map(|f| f.range)
        .collect();
    self.scanner.scan_ranges(self.source, &scannable_ranges, false)
}
```

**Pros**:
- ✅ **Clear boundaries**: `RawListItem` has exact `text: Cow<str>` + `range: SourceByteRange`
- ✅ **Context available**: Know if text is heading/paragraph/list item
- ✅ **Cacheable**: Structured data can be serialized/deserialized
- ✅ **Testable**: Can test scanner against `RawListItem` directly

**Cons**:
- Slight latency (must wait for block completion)
- Two-pass over text (parse + scan)

**Verdict for Lithos**: ✅ **RECOMMENDED**. Already partially implemented, natural fit.

---

## 2. Production System Patterns

### A. tree-sitter: External Scanners for Context-Dependent Tokens

**Key insight**: Use grammar rules for structure, external scanners for ambiguous cases.

```c
// From tree-sitter docs
bool tree_sitter_python_external_scanner_scan(
    void *payload,
    TSLexer *lexer,
    const bool *valid_symbols  // Parser tells scanner what's expected!
) {
    if (valid_symbols[INDENT] || valid_symbols[DEDENT]) {
        return scan_indent_dedent(lexer, valid_symbols);
    }
    return false;
}
```

**Lesson for Lithos**: Scanner should know **what it's allowed to find** based on context:
- Inside `RawHeading`: tags + fields allowed, block refs NOT allowed
- Inside `RawListItem`: all metadata types allowed
- Inside code spans: nothing allowed (already filtered by `is_scannable`)

---

### B. ripgrep: Fast-Path Optimization + Literal Extraction

**Key insight**: Check cheap byte-level conditions before expensive operations.

```rust
// Conceptual ripgrep pattern
impl Searcher {
    fn search(&self, haystack: &[u8]) -> Vec<Match> {
        // Fast path: literal prefilter
        let candidates = self.literal_matcher.find_candidates(haystack);

        // Slow path: only run regex on candidates
        candidates.iter()
            .filter_map(|&offset| self.regex.find_at(haystack, offset))
            .collect()
    }
}
```

**Lesson for Lithos**: Current `can_start_with(byte: u8)` is good! Extend it:

```rust
trait ScanRule {
    /// Fast path: Can this rule possibly match starting with this byte?
    fn can_start_with(&self, byte: u8) -> bool;

    /// Slow path: Full match with context
    fn try_scan(&self, ctx: &ScannerContext, cursor: &mut Cursor)
        -> Result<Option<ScannedArtifact>, NoteError>;
}

// Example: TagRule
impl ScanRule for TagRule {
    fn can_start_with(&self, byte: u8) -> bool {
        byte == b'#'  // Fast reject 99% of bytes
    }

    fn try_scan(&self, ctx, cursor) -> Result<...> {
        if cursor.prev_alnum { return Ok(None); }  // Word boundary check
        // Full UTF-8 validation only for candidates
        // ...
    }
}
```

---

### C. clippy: Lint Registration + Configuration

**Key insight**: Dynamic rule loading with declarative enable/disable.

```rust
// clippy_lints/src/lib.rs pattern
pub fn register_plugins(store: &mut LintStore) {
    store.register_lints(&[
        &borrow_deref_ref::BORROW_DEREF_REF,
        &bool_comparison::BOOL_COMPARISON,
        // ... hundreds more
    ]);

    store.register_group(true, "clippy::all", vec![
        LintId::of(&borrow_deref_ref::BORROW_DEREF_REF),
        // ...
    ]);
}
```

**Lesson for Lithos**: Support **rule groups** + **config overrides**:

```rust
#[derive(Debug)]
pub struct ScannerConfig {
    pub emoji_markers: Box<[char]>,
    pub enabled_rules: RuleSet,
}

bitflags::bitflags! {
    pub struct RuleSet: u32 {
        const TAGS           = 1 << 0;
        const INLINE_FIELDS  = 1 << 1;
        const BLOCK_REFS     = 1 << 2;
        const EMOJI_FIELDS   = 1 << 3;
        const BARE_FIELDS    = 1 << 4;

        const ALL = Self::TAGS.bits()
                  | Self::INLINE_FIELDS.bits()
                  | Self::BLOCK_REFS.bits()
                  | Self::EMOJI_FIELDS.bits()
                  | Self::BARE_FIELDS.bits();
    }
}

impl NoteScanner {
    pub fn with_rules(config: ScannerConfig, rules: Vec<Box<dyn ScanRule>>) -> Self {
        let enabled_rules: Vec<_> = rules.into_iter()
            .filter(|rule| config.enabled_rules.contains(rule.rule_set_bit()))
            .collect();
        Self { context: config.into(), rules: enabled_rules }
    }
}
```

---

### D. nom: Parser Combinator Composition

**Key insight**: Small, testable parsers composed into complex behavior.

```rust
// nom pattern
fn hex_color(input: &str) -> IResult<&str, Color> {
    let (input, _) = tag("#")(input)?;
    let (input, (r, g, b)) = tuple((
        hex_primary,  // Reusable component
        hex_primary,
        hex_primary,
    ))(input)?;
    Ok((input, Color { red: r, green: g, blue: b }))
}
```

**Lesson for Lithos**: Rules should be **composable building blocks**:

```rust
// Potential future extension: composite rules
struct CompositeFieldRule {
    inner: Vec<Box<dyn ScanRule>>,
}

impl ScanRule for CompositeFieldRule {
    fn try_scan(&self, ctx, cursor) -> Result<Option<ScannedArtifact>, NoteError> {
        for rule in &self.inner {
            if let Some(artifact) = rule.try_scan(ctx, cursor)? {
                return Ok(Some(artifact));
            }
        }
        Ok(None)
    }
}
```

---

## 3. Recommended Architecture for Lithos

### Core Design Principles

1. **Scan after structure**: Operate on `RawListItem`, `RawHeading`, etc.
2. **Trait-based composition**: Rules implement `ScanRule` trait
3. **Fast-path filtering**: `can_start_with(byte)` before full scan
4. **Zero-copy everywhere**: Rules borrow from source, return `&'source str` slices
5. **Config-driven**: Enable/disable rules, custom markers

---

### Proposed Architecture

```rust
// ═══ Core Scanner API ═══════════════════════════════════════════

pub struct NoteScanner {
    config: ScannerConfig,
    rules: RuleRegistry,
}

impl NoteScanner {
    /// Scan a structured block (already parsed).
    pub fn scan_structured_block<'source>(
        &self,
        block: &StructuredBlock<'source>,
    ) -> Result<ScannedArtifacts<'source>, NoteError> {
        // Knows context: heading vs paragraph vs list item
        let allowed_rules = self.rules_for_context(block.kind);

        for range in block.scannable_ranges() {
            let text = &block.source[range.clone()];
            let mut cursor = Cursor::new(text, range.start);

            while !cursor.is_eof() {
                for rule in allowed_rules {
                    if rule.can_start_with(cursor.peek_byte()?)
                       && let Some(artifact) = rule.try_scan(&self.config, &mut cursor)?
                    {
                        artifacts.push(artifact);
                        break;
                    }
                }
                cursor.advance_char()?;
            }
        }
        Ok(artifacts)
    }
}

// ═══ Rule Registry (Extensibility) ═══════════════════════════════

pub struct RuleRegistry {
    body_rules: Vec<Arc<dyn ScanRule>>,      // Tags, inline fields
    line_start_rules: Vec<Arc<dyn ScanRule>>, // Bare fields
    context_rules: HashMap<BlockContext, Vec<Arc<dyn ScanRule>>>,
}

impl RuleRegistry {
    pub fn register<R: ScanRule + 'static>(&mut self, rule: R, contexts: &[BlockContext]) {
        let rule = Arc::new(rule);
        for ctx in contexts {
            self.context_rules.entry(*ctx)
                .or_default()
                .push(Arc::clone(&rule));
        }
    }

    pub fn disable(&mut self, rule_id: &str) {
        // Remove by type name or ID
    }
}

// ═══ Enhanced Rule Trait ═══════════════════════════════════════════

pub trait ScanRule: std::fmt::Debug + Send + Sync {
    /// Rule identifier for config enable/disable.
    fn id(&self) -> &str;

    /// Fast path: Can this rule match starting with this byte?
    fn can_start_with(&self, byte: u8) -> bool;

    /// Which contexts is this rule valid in?
    fn valid_contexts(&self) -> &[BlockContext] {
        &[BlockContext::Heading, BlockContext::Paragraph, BlockContext::ListItem]
    }

    /// Full match with context awareness.
    fn try_scan<'source>(
        &self,
        config: &ScannerConfig,
        cursor: &mut Cursor<'source>,
        context: BlockContext,  // New: knows where it is!
    ) -> Result<Option<ScannedArtifact<'source>>, NoteError>;
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum BlockContext {
    Heading,
    Paragraph,
    ListItem,
    CodeBlock,  // For future: never scannable
}

// ═══ Config-Driven Behavior ═══════════════════════════════════════

#[derive(Debug)]
pub struct ScannerConfig {
    pub emoji_markers: Box<[char]>,
    pub bare_field_prefix_max_len: usize,  // Limit `key::value` key length
    pub enabled_rules: HashSet<String>,    // Rule IDs
}

impl ScannerConfig {
    pub fn is_enabled(&self, rule: &dyn ScanRule) -> bool {
        self.enabled_rules.is_empty() || self.enabled_rules.contains(rule.id())
    }
}
```

---

### Example: Enhanced TagRule with Context

```rust
#[derive(Debug)]
pub struct TagRule {
    allow_in_headings: bool,
}

impl ScanRule for TagRule {
    fn id(&self) -> &str { "core.tag" }

    fn can_start_with(&self, byte: u8) -> bool {
        byte == b'#'
    }

    fn valid_contexts(&self) -> &[BlockContext] {
        if self.allow_in_headings {
            &[BlockContext::Heading, BlockContext::Paragraph, BlockContext::ListItem]
        } else {
            &[BlockContext::Paragraph, BlockContext::ListItem]
        }
    }

    fn try_scan<'source>(
        &self,
        _config: &ScannerConfig,
        cursor: &mut Cursor<'source>,
        context: BlockContext,
    ) -> Result<Option<ScannedArtifact<'source>>, NoteError> {
        // Fast path already checked: cursor.peek_byte() == b'#'

        if cursor.prev_alnum {
            return Ok(None);  // Not a word boundary
        }

        // ... rest of current TagRule::try_scan logic
    }
}
```

---

### Example: Custom Plugin Rule

Users can register custom rules:

```rust
// In user's vault config
#[derive(Debug)]
struct CustomAnnotationRule;

impl ScanRule for CustomAnnotationRule {
    fn id(&self) -> &str { "custom.annotation" }

    fn can_start_with(&self, byte: u8) -> bool {
        byte == b'@'  // @todo, @fixme, etc.
    }

    fn valid_contexts(&self) -> &[BlockContext] {
        &[BlockContext::ListItem, BlockContext::Paragraph]
    }

    fn try_scan<'source>(
        &self,
        _config: &ScannerConfig,
        cursor: &mut Cursor<'source>,
        _context: BlockContext,
    ) -> Result<Option<ScannedArtifact<'source>>, NoteError> {
        if !cursor.rest.starts_with('@') {
            return Ok(None);
        }

        // Extract @annotation-name
        let mut len = 1;
        for c in cursor.rest[1..].chars() {
            if c.is_alphanumeric() || c == '-' {
                len += c.len_utf8();
            } else {
                break;
            }
        }

        if len > 1 {
            let annotation = &cursor.rest[..len];
            let range = SourceByteRange::new(
                cursor.offset,
                cursor.offset.add_offset(len)?,
            )?;
            cursor.advance(len)?;

            Ok(Some(ScannedArtifact::CustomAnnotation {
                name: annotation.into(),
                range,
            }))
        } else {
            Ok(None)
        }
    }
}

// Register in scanner
let mut scanner = NoteScanner::new(config);
scanner.register_rule(CustomAnnotationRule, &[
    BlockContext::Paragraph,
    BlockContext::ListItem,
]);
```

---

## 4. Performance Strategy

### Fast Path Optimizations

```rust
impl NoteScanner {
    fn scan_cursor<'source>(
        &self,
        cursor: &mut Cursor<'source>,
        artifacts: &mut Vec<ScannedArtifact<'source>>,
        context: BlockContext,
    ) -> Result<(), NoteError> {
        while !cursor.is_eof() {
            let byte = cursor.peek_byte()?;

            // ═══ Optimization 1: Byte-level fast path ═══
            // Skip ASCII letters/digits immediately (most common case)
            if byte.is_ascii_alphanumeric() && !cursor.prev_alnum {
                cursor.advance_ascii_word()?;
                continue;
            }

            // ═══ Optimization 2: Rule fast path ═══
            let mut matched = false;
            for rule in self.rules_for_context(context) {
                if rule.can_start_with(byte) {
                    if let Some(artifact) = rule.try_scan(&self.config, cursor, context)? {
                        artifacts.push(artifact);
                        matched = true;
                        break;
                    }
                }
            }

            // ═══ Optimization 3: Bulk skip ═══
            if !matched {
                cursor.advance_char()?;  // Skip one UTF-8 char
            }
        }
        Ok(())
    }
}

impl Cursor<'_> {
    /// Fast-forward through ASCII word characters.
    fn advance_ascii_word(&mut self) -> Result<(), NoteError> {
        let len = self.rest.bytes()
            .take_while(|&b| b.is_ascii_alphanumeric() || b == b'_')
            .count();
        self.advance(len)
    }
}
```

---

### Allocation Strategy

**Current approach**: Good! Already zero-copy with `Cow<'source, str>`.

**Optimization opportunities**:

```rust
// Pre-allocate based on heuristics
impl NoteScanner {
    pub fn scan_block<'source>(
        &self,
        text: &'source str,
        base_offset: SourceByteOffset,
    ) -> Result<Vec<ScannedArtifact<'source>>, NoteError> {
        // Heuristic: 1 artifact per 100 bytes
        let estimated_artifacts = text.len() / 100;
        let mut artifacts = Vec::with_capacity(estimated_artifacts.max(4));

        // ...
        artifacts.shrink_to_fit();  // Reclaim excess only if significantly over-allocated
        Ok(artifacts)
    }
}
```

---

### Parallelization

**For large documents**: Scan independent blocks in parallel.

```rust
use rayon::prelude::*;

impl NoteScanner {
    pub fn scan_document_parallel<'source>(
        &self,
        blocks: &[StructuredBlock<'source>],
    ) -> Result<Vec<ScannedArtifacts<'source>>, NoteError> {
        blocks.par_iter()  // rayon parallel iterator
            .map(|block| self.scan_structured_block(block))
            .collect()
    }
}
```

**Trade-offs**:
- ✅ Scales with CPU cores
- ⚠️ Only useful for docs with many blocks (e.g., 100+ list items)
- ❌ Adds dependency + complexity

**Recommendation**: Defer until profiling shows need.

---

## 5. Decision Matrix

| Criterion                  | Raw Text | Event Stream | Post-Parse (Recommended) |
|----------------------------|----------|--------------|--------------------------|
| **Performance**            | ⭐⭐⭐       | ⭐⭐            | ⭐⭐                        |
| **Context Awareness**      | ❌        | ⚠️            | ✅                        |
| **Zero-Copy Feasible**     | ✅        | ⚠️            | ✅                        |
| **Testability**            | ⭐⭐⭐       | ⭐⭐            | ⭐⭐⭐                      |
| **Extensibility**          | ⭐⭐        | ⭐⭐            | ⭐⭐⭐                      |
| **Fit for Obsidian**       | ❌        | ⚠️            | ✅                        |
| **Current Codebase Fit**   | ❌        | ⚠️            | ✅ (already started)      |

**Legend**:
- ⭐⭐⭐ Excellent
- ⭐⭐ Good
- ⭐ Acceptable
- ⚠️ Requires workarounds
- ❌ Not suitable

---

## 6. Migration Path

### Phase 1: Enhance Current Scanner (Low Risk)

**Current state** (`scanner.rs`):
- ✅ Trait-based rules
- ✅ Fast-path `can_start_with()`
- ✅ Zero-copy artifacts
- ❌ No context awareness
- ❌ BareFieldRule not using trait

**Changes**:

```diff
 pub(crate) trait ScanRule: std::fmt::Debug + Send + Sync {
+    fn id(&self) -> &str;
     fn can_start_with(&self, byte: u8) -> bool;
     fn try_scan<'source>(
         &self,
-        ctx: &ScannerContext,
+        config: &ScannerConfig,
         cursor: &mut Cursor<'source>,
+        context: BlockContext,
     ) -> Result<Option<ScannedArtifact<'source>>, NoteError>;
 }

+impl ScanRule for BareFieldRule {
+    fn id(&self) -> &str { "core.bare_field" }
+    // ... implement trait
+}
```

**Impact**: Minimal breakage, adds foundation for extensibility.

---

### Phase 2: Add Configuration Layer (Medium Risk)

```rust
// New types
pub struct ScannerConfig {
    pub emoji_markers: Box<[char]>,
    pub enabled_rules: HashSet<String>,
}

impl NoteScanner {
    pub fn with_config(config: ScannerConfig) -> Self {
        // Filter rules based on config
        let rules: Vec<Box<dyn ScanRule>> = Self::default_rules()
            .into_iter()
            .filter(|r| config.enabled_rules.is_empty()
                     || config.enabled_rules.contains(r.id()))
            .collect();
        Self { config, rules }
    }
}
```

**Impact**: Adds flexibility without breaking existing code (default config = current behavior).

---

### Phase 3: Add Rule Registry (Higher Risk)

```rust
pub struct RuleRegistry {
    rules: HashMap<String, Arc<dyn ScanRule>>,
}

impl NoteScanner {
    pub fn register_custom_rule(&mut self, rule: impl ScanRule + 'static) {
        self.rules.push(Box::new(rule));
    }
}
```

**Impact**: Enables plugins, requires careful lifetime management.

---

## 7. Code Examples from Production

### A. ripgrep's Literal Optimization

From `crates/grep/src/searcher/core.rs` (conceptual):

```rust
pub struct Searcher {
    matcher: Box<dyn Matcher>,
    config: Config,
}

impl Searcher {
    pub fn search_slice(&mut self, haystack: &[u8]) -> Result<(), Error> {
        // Fast path: multi-substring literal matching with Aho-Corasick
        if let Some(literals) = self.matcher.literals() {
            for mat in literals.find_iter(haystack) {
                // Only run full regex at literal match positions
                if self.matcher.is_match_at(haystack, mat.start())? {
                    self.sink.matched(mat)?;
                }
            }
        } else {
            // Slow path: full regex scan
            for mat in self.matcher.find_iter(haystack)? {
                self.sink.matched(mat)?;
            }
        }
        Ok(())
    }
}
```

**Key insight**: Extract common prefixes/literals, use faster algorithm to find candidates.

**Application to Lithos**:
```rust
// Potential optimization: literal set for all rule start patterns
static RULE_START_BYTES: &[u8] = b"#[(^@";  // All possible start bytes

impl NoteScanner {
    fn scan_cursor(&self, cursor: &mut Cursor) -> Result<...> {
        while !cursor.is_eof() {
            let byte = cursor.peek_byte()?;

            // Fast reject: not a potential rule start
            if !RULE_START_BYTES.contains(&byte) {
                cursor.advance_char()?;
                continue;
            }

            // Slow path: check rules
            for rule in &self.rules {
                if rule.can_start_with(byte) && ... { }
            }
        }
    }
}
```

---

### B. tree-sitter's Parser-Driven Scanning

From tree-sitter Python grammar (conceptual):

```c
// Grammar rule for indentation-sensitive blocks
block: $ => seq(
    '_INDENT',      // External token: only scannable at block start
    repeat($.statement),
    '_DEDENT'       // External token: only scannable at block end
)

// External scanner checks parser context
bool scan(TSLexer *lexer, const bool *valid_symbols) {
    // Parser tells us: "I'm expecting INDENT or DEDENT here"
    if (valid_symbols[INDENT]) {
        return try_scan_indent(lexer);
    }
    if (valid_symbols[DEDENT]) {
        return try_scan_dedent(lexer);
    }
    return false;  // Don't waste time on impossible tokens
}
```

**Application to Lithos**:
```rust
// Current code already has structural context!
// extractor.rs:68-87
fn process_leaf(&mut self, kind: LeafKind, span: BlockSpan, ...) {
    match kind {
        LeafKind::Heading(_) => {
            let scanned = self.scan_fragments(fragments)?;
            // ^^^ Could pass context: BlockContext::Heading
        }
        LeafKind::ListItem(_) => {
            let scanned = self.scan_fragments(fragments)?;
            // ^^^ Could pass context: BlockContext::ListItem
        }
    }
}

// Enhanced version:
fn scan_fragments_with_context(
    &self,
    fragments: &[TextFragment],
    context: BlockContext,  // NEW
) -> Result<ScannedRawArtifacts, NoteIngestError> {
    self.scanner.scan_ranges_with_context(
        self.source,
        &scannable_ranges,
        context,  // Pass to scanner
    )
}
```

---

## 8. Recommendations Summary

### ✅ DO: Post-Parse Structured Scanning

**Reasoning**:
1. Already partially implemented (see `extractor.rs:142-150`)
2. Provides natural context (`RawHeading`, `RawListItem`, etc.)
3. Clean separation: parsing → structure → scanning
4. Testable in isolation

**Implementation**:
```rust
// Enhanced scanner signature
impl NoteScanner {
    pub fn scan_structured<'source>(
        &self,
        text: &'source str,
        ranges: &[Range<usize>],
        context: BlockContext,  // NEW: knows structure
    ) -> Result<ScannedArtifacts<'source>, NoteError>
}
```

---

### ✅ DO: Trait-Based Rule Composition

**Keep current `ScanRule` trait**, enhance with:
- `fn id(&self) -> &str` for config enable/disable
- `fn valid_contexts(&self) -> &[BlockContext]` for context filtering
- Make `BareFieldRule` implement the trait

---

### ✅ DO: Fast-Path Byte Checks

**Keep current `can_start_with(byte: u8)`**, enhance with:
- Shared literal set for all rules: `&[b'#', b'[', b'(', b'^']`
- Bulk skip ASCII words: `cursor.advance_ascii_word()`

---

### ⚠️ CONSIDER: Config-Driven Rule Enable/Disable

**Low risk**, high value for users who want to customize:
```toml
# .lithos/config.toml
[scanner]
enabled_rules = ["core.tag", "core.inline_field"]  # Disable block refs
emoji_markers = ["📌", "⏰", "🎯"]
```

---

### ❌ DON'T: Scan Raw Text Without Structure

**Reason**: Too many false positives (tags in code blocks, fields in links).

---

### ❌ DON'T: Parallelize Prematurely

**Wait for**:
- Profiling data showing scanner is bottleneck
- Documents with 1000+ scannable blocks

---

## 9. Testing Strategy

### Unit Tests: Rule Isolation

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tag_rule_respects_word_boundaries() {
        let config = ScannerConfig::default();
        let mut cursor = Cursor::new("not#tag #real-tag", SourceByteOffset::new(0));

        let rule = TagRule;
        cursor.advance(3).unwrap();  // Position at '#tag'
        cursor.prev_alnum = true;    // Simulate previous char 't'

        assert_eq!(
            rule.try_scan(&config, &mut cursor, BlockContext::Paragraph).unwrap(),
            None,  // Should reject: word boundary violated
        );
    }

    #[test]
    fn tag_rule_accepts_valid_tag() {
        let config = ScannerConfig::default();
        let mut cursor = Cursor::new("#valid-tag", SourceByteOffset::new(0));
        let rule = TagRule;

        let artifact = rule.try_scan(&config, &mut cursor, BlockContext::Paragraph)
            .unwrap()
            .expect("Should match valid tag");

        match artifact {
            ScannedArtifact::Tag(tag) => {
                assert_eq!(tag.value.as_ref(), "#valid-tag");
            }
            _ => panic!("Expected Tag"),
        }
    }
}
```

---

### Integration Tests: Structured Block Scanning

```rust
#[test]
fn scan_list_item_with_metadata() {
    let source = "- [ ] Task with #tag and [key:: value]";
    let scanner = NoteScanner::new(vec!['📌']);

    // Simulate parser output
    let list_item = RawListItem {
        kind: RawListKind::Unordered,
        text: "Task with #tag and [key:: value]".into(),
        range: SourceByteRange::new(
            SourceByteOffset::new(6),
            SourceByteOffset::new(39),
        ).unwrap(),
        // ...
    };

    let scanned = scanner.scan_structured_block(
        source,
        &list_item.text_range(),
        BlockContext::ListItem,
    ).unwrap();

    assert_eq!(scanned.tags.len(), 1);
    assert_eq!(scanned.tags[0].value.as_ref(), "#tag");
    assert_eq!(scanned.inline_fields.len(), 1);
    assert_eq!(scanned.inline_fields[0].key.as_ref(), "key");
}
```

---

### Property-Based Tests: Fuzz Testing

```rust
use proptest::prelude::*;

proptest! {
    #[test]
    fn scanner_never_panics(text in "\\PC*") {
        let scanner = NoteScanner::new(vec![]);
        let result = scanner.scan_block(&text, SourceByteOffset::new(0));

        // Should either succeed or return NoteError, never panic
        match result {
            Ok(_) | Err(NoteError::PositionOverflow { .. }) => {},
            Err(e) => panic!("Unexpected error: {:?}", e),
        }
    }
}
```

---

## 10. Open Questions

### Q1: Should we support negative rules?

**Example**: "Don't scan for tags inside URLs"

```rust
trait ScanRule {
    fn should_skip(&self, context: &SkipContext) -> bool {
        false  // Default: never skip
    }
}

struct TagRule;
impl ScanRule for TagRule {
    fn should_skip(&self, context: &SkipContext) -> bool {
        context.inside_url  // Skip tags in URLs
    }
}
```

**Recommendation**: Defer until we see real use cases where this is needed.

---

### Q2: Should scanner be stateful across blocks?

**Example**: Track "last seen tag" to populate `RawNote.tags` with context:

```rust
impl NoteScanner {
    fn scan_document(&mut self, blocks: &[StructuredBlock]) {
        self.state.last_tag = None;
        for block in blocks {
            let scanned = self.scan_block(block);
            if let Some(tag) = scanned.tags.first() {
                self.state.last_tag = Some(tag.clone());
            }
        }
    }
}
```

**Recommendation**: ❌ No. Scanner should be stateless. State belongs in higher-level orchestrator (`NoteProcessor`).

---

### Q3: Should we cache compiled rules?

**Example**: Pre-compile regex patterns at scanner construction:

```rust
struct DelimitedFieldRule {
    separator_regex: Regex,  // Pre-compiled `::`
}
```

**Recommendation**: ✅ Yes, but only if profiling shows pattern matching is hot path. Current byte-level scanning is likely faster.

---

## 11. References

- [tree-sitter external scanners](https://tree-sitter.github.io/tree-sitter/creating-parsers#external-scanners)
- [pulldown-cmark architecture](https://github.com/pulldown-cmark/pulldown-cmark)
- [ripgrep performance guide](https://blog.burntsushi.net/ripgrep/)
- [nom parser combinators](https://docs.rs/nom/latest/nom/)
- [clippy lint registration](https://github.com/rust-lang/rust-clippy/blob/master/book/src/development/adding_lints.md)
- Lithos current scanner: `lithos-core/src/note/scanner.rs`
- Lithos parser integration: `lithos-core/src/note/extractor.rs:142-150`

---

## 12. Appendix: Performance Benchmarks (Hypothetical)

Based on ripgrep's published benchmarks and Lithos's use case:

| Approach              | 1KB Note | 10KB Note | 100KB Note |
|-----------------------|----------|-----------|------------|
| Raw Text (SIMD)       | 50µs     | 400µs     | 4ms        |
| Event Stream          | 80µs     | 650µs     | 7ms        |
| Post-Parse Structured | 100µs    | 800µs     | 8ms        |

**Assumptions**:
- 1KB ≈ 20 lines
- 10KB ≈ 200 lines
- 100KB ≈ 2000 lines
- Typical Obsidian note: 1-10KB

**Conclusion**: For typical notes, the 20-30µs overhead of structured scanning is negligible compared to other costs (file I/O, deserialization, UI rendering).

---

## 13. Final Recommendation

**Architecture**: Post-parse structured scanning with trait-based rule composition.

**Immediate Actions**:
1. ✅ Add `BlockContext` parameter to `ScanRule::try_scan()`
2. ✅ Make `BareFieldRule` implement `ScanRule` trait
3. ✅ Add `ScanRule::id()` for future config
4. ⚠️ Keep current `NoteScanner` API stable

**Future Extensions** (defer until needed):
1. Config-driven rule enable/disable
2. Custom rule registration API
3. Rule priority/ordering
4. Negative rules (skip contexts)

**Rationale**: This approach:
- Builds on existing code (`scanner.rs`, `extractor.rs`)
- Provides clean extension points
- Maintains zero-copy performance
- Supports testing in isolation
- Aligns with production patterns (tree-sitter, clippy)
