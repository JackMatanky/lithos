# Parser and Scanner Optimization Plan

**Date:** 2026-04-07
**Status:** Ready for Implementation
**Goal:** Reduce parser passes, fix scanner boundary semantics, improve cohesion between parser and scanner, and optimize pulldown-cmark usage.

---

## Table of Contents

1. [Executive Summary](#executive-summary)
2. [Current Architecture Overview](#current-architecture-overview)
3. [Detailed Findings](#detailed-findings)
4. [Solution Design](#solution-design)
5. [Implementation Plan](#implementation-plan)
6. [Testing Strategy](#testing-strategy)
7. [Risk Assessment](#risk-assessment)

---

## Executive Summary

### Problems Identified

1. **Multiple full-document passes**: `metadata_ranges()` and `reference_definitions_map()` add 2 extra passes
2. **Redundant per-block scans**: list items are scanned twice (once for artifacts, once for task markers)
3. **Scanner boundary semantics broken**: `scan_ranges()` carries state across disjoint ranges, causing false positives
4. **Block-ref tail scan issues**: fallback scan can detect refs in code/link text
5. **Suboptimal pulldown-cmark usage**: not using built-in RefDefs API, enabling unnecessary extensions

### Expected Impact

- **Performance**: Reduce from 3+ passes to 1 streaming pass for reference-heavy documents
- **Correctness**: Fix tag/block-ref detection near skipped text (links, code)
- **Maintainability**: Remove ~400 lines of manual reference parsing code
- **Code quality**: Better separation of concerns between parser and scanner

---

## Current Architecture Overview

### File Structure

```
lithos-core/src/note/
├── parser.rs         # Orchestrates pulldown-cmark, creates Raw* structures
├── scanner.rs        # State machine for inline metadata (tags, fields, refs)
└── raw/              # Zero-copy Raw* types
    ├── mod.rs
    ├── aggregate.rs  # RawNote
    ├── inline_field.rs
    ├── value.rs
    ├── tag.rs
    ├── block_ref.rs
    ├── list.rs
    ├── section.rs
    ├── link.rs
    ├── heading.rs
    └── frontmatter.rs
```

### Current Flow

```
1. Parse markdown → pulldown-cmark event stream
   ├── Pre-pass: metadata_ranges() (PASS 1)
   └── When reference links detected:
       └── Manual scan: reference_definitions_map() (PASS 2)

2. Main event loop (PASS 3 for content)
   ├── Track blocks via block_stack
   ├── Track scannable ranges (text outside code/links)
   └── On block end:
       ├── Call scan_block_artifacts() → NoteScanner
       │   ├── Scans scannable_ranges
       │   └── Fallback: scans last 512 bytes if no block refs
       └── For list items with checkboxes:
           └── Call scan_task_marker() → NoteScanner (REDUNDANT SCAN)

3. NoteScanner.scan_ranges()
   ├── Preserves prev_alnum/mode across ranges (BUGGY)
   └── Extracts tags, fields, block refs, task markers
```

### Key Issues

**Issue 1: Multiple Passes**
- Location: `parser.rs:294-326` (metadata_ranges), `parser.rs:1187-1333` (reference scanning)
- Impact: 2 extra full-document passes before main parse
- Root cause: Manual reference definition scanning instead of using pulldown-cmark API

**Issue 2: Redundant Block Scans**
- Location: `parser.rs:950-956` (task marker rescan in finalize_list_item)
- Impact: Every list item with checkbox scanned twice
- Root cause: Separate task marker detection instead of unified scan

**Issue 3: Scanner Boundary Semantics**
- Location: `scanner.rs:176-201` (scan_ranges)
- Impact: False positives for tags/refs adjacent to skipped text
- Root cause: `prev_alnum`/`mode` carried across disjoint ranges without checking actual source boundaries

**Issue 4: Block-ref Tail Scan**
- Location: `parser.rs:769-783` (block_ref_tail_range in scan_block_artifacts)
- Impact: Block refs detected in code/link text that was intentionally excluded
- Root cause: Fallback scan doesn't respect scannable_ranges contract

---

## Detailed Findings

### Finding 1: Manual Reference Definition Scanning

**Current Implementation:**
```rust
// parser.rs:294-326
fn metadata_ranges(markdown: &str) -> Vec<std::ops::Range<usize>> {
    let parser = Parser::new_ext(markdown, Self::obsidian_options());
    // Full pass to find metadata blocks
    // ...
}

// parser.rs:1187-1333
fn scan_reference_definitions(
    markdown: &str,
    skip_ranges: &[std::ops::Range<usize>],
) -> Vec<ReferenceDef> {
    // Manual line-by-line parsing
    // - Track fence state
    // - Parse [label]: destination
    // - Handle multiline definitions
    // ~150 lines of complex parsing logic
}
```

**Problem:**
- Two full-document passes (metadata_ranges + scan_reference_definitions)
- ~150 lines of duplicate parsing logic that pulldown-cmark already does
- Potential divergence from CommonMark spec (case folding, title parsing, etc.)

**pulldown-cmark Solution:**
```rust
// RefDefs API (docs.rs/pulldown-cmark/latest/pulldown_cmark/struct.RefDefs.html)
let parser = Parser::new_ext(markdown, options);
let offset_iter = parser.into_offset_iter();

// Get reference definitions BEFORE consuming iterator
let ref_defs = offset_iter.reference_definitions();

// Access any reference
if let Some(link_def) = ref_defs.get("my-ref") {
    let url = link_def.dest.as_ref();
    let title = link_def.title.as_deref();
}

// Or iterate all
for (label, link_def) in ref_defs.iter() {
    // Process reference
}

// Then consume events
for (event, range) in offset_iter {
    // Main parse
}
```

**Benefits:**
- Zero extra passes (references collected during main parse)
- Correct CommonMark case-insensitive matching
- No manual parsing code to maintain

### Finding 2: Redundant Task Marker Scanning

**Current Implementation:**
```rust
// parser.rs:922-1026 (finalize_list_item)
fn finalize_list_item(...) {
    // First scan: artifacts
    let (scan_tags, scan_fields, scan_refs, _) =
        Self::scan_block_artifacts(
            scanner,
            markdown,
            &block.scannable_ranges,  // Scans text ranges
            false,
            block_range,
            task_spec,
        )?;

    // Second scan: task marker (if checkbox present)
    let task_marker = if is_checked.is_some() {
        scanner
            .scan_task_marker(markdown, block_range)  // Rescans FULL block
            .map_err(NoteIngestError::Domain)?
    } else {
        None
    };
    // ...
}
```

**Problem:**
- `scan_task_marker()` rescans the entire `block_range` even though `scan_block_artifacts()` just scanned it
- Task markers are always near line start (within first ~20 bytes), but we scan the entire block

**Solution:**
- Option A: Pass `include_task_marker: bool` to `scan_block_artifacts()` and scan once
- Option B: Create a small prefix range (first line or first N bytes) and scan only that for markers

**Recommended Approach: Option B (smaller scan)**
```rust
// Only scan first line for task marker
let task_marker = if is_checked.is_some() {
    let first_line_len = markdown[block_range.start().as_usize()..]
        .find('\n')
        .unwrap_or(80)
        .min(80);  // Cap at 80 chars
    let prefix_range = block_range.start().as_usize()
        ..(block_range.start().as_usize() + first_line_len);
    scanner
        .scan_task_marker(markdown, SourceByteRange::try_from(prefix_range)?)
        .map_err(NoteIngestError::Domain)?
} else {
    None
};
```

### Finding 3: Scanner Boundary Semantics

**Current Implementation:**
```rust
// scanner.rs:176-201
pub fn scan_ranges<'source>(
    &self,
    text: &'source str,
    ranges: &[std::ops::Range<usize>],
    artifacts: &mut Vec<ScannedArtifact<'source>>,
) -> Result<(), NoteError> {
    let mut cursor = Cursor::new("", SourceByteOffset::new(0));
    for range in ranges {
        // ...
        cursor.reset(segment, base_offset);
        self.scan_cursor(&mut cursor, artifacts)?;
        // ^^^ cursor.prev_alnum and cursor.mode persist across ranges
    }
    Ok(())
}
```

**Problem Scenario:**
```markdown
Check [[wikilink]]#tag
```

Parser creates scannable_ranges:
- Range 1: "Check " (before link)
- Range 2: "#tag" (after link)

Scanner sees:
1. Range 1: processes "Check ", sets `prev_alnum = false` (space)
2. Range 2: sees `#`, checks `prev_alnum == false`, extracts "#tag" ✅

But this is **wrong** because in the actual source, "#" comes immediately after "]" (no space). The tag should **not** be detected because there's no word boundary.

**Solution:**
Derive boundary state from actual source:

```rust
pub fn scan_ranges<'source>(
    &self,
    text: &'source str,
    ranges: &[std::ops::Range<usize>],
    artifacts: &mut Vec<ScannedArtifact<'source>>,
) -> Result<(), NoteError> {
    for range in ranges {
        if range.is_empty() {
            continue;
        }
        let Some(segment) = text.get(range.clone()) else {
            continue;
        };
        let base_offset = SourceByteOffset::try_from(range.start)?;

        // Derive boundary state from ACTUAL source
        let prev_alnum = if range.start > 0 {
            text.as_bytes()
                .get(range.start - 1)
                .map(|&b| b.is_ascii_alphanumeric())
                .unwrap_or(false)
        } else {
            false
        };

        let mode = if range.start == 0
            || text.as_bytes().get(range.start - 1) == Some(&b'\n')
            || text.as_bytes().get(range.start - 1) == Some(&b'\r')
        {
            ScanMode::AtLineStart
        } else {
            ScanMode::InBody
        };

        let mut cursor = Cursor::new_with_state(segment, base_offset, mode, prev_alnum);
        self.scan_cursor(&mut cursor, artifacts)?;
    }
    Ok(())
}
```

**Impact:**
- Fixes false positives for tags/block-refs adjacent to links
- Maintains correct word-boundary semantics across skipped segments

### Finding 4: Block-ref Tail Scan Issues

**Current Implementation:**
```rust
// parser.rs:769-783
fn scan_block_artifacts(...) {
let raw_tokens = scanner
         .scan_ranges(source, scannable_ranges, include_task_marker)?;

    let mut block_refs = raw_tokens.block_refs;

    // Fallback: scan last 512 bytes if no block refs found
    if block_refs.is_empty()
        && let Some(tail_range) = Self::block_ref_tail_range(source, block_range)
    {
let tail_tokens = scanner
             .scan_ranges(source, std::slice::from_ref(&tail_range), false)?;
        block_refs.extend(tail_tokens.block_refs);
    }
    // ...
}
```

**Problem:**
The `block_ref_tail_range()` creates a range over the last 512 bytes of the block, **ignoring** the `scannable_ranges` contract. This means:
- Block refs in inline code at end of block are detected: `` `code ^ref` ``
- Block refs in links are detected: `[[note#section^ref]]`
- Violates the parser's semantic rule that code/links are not scannable

**Solution: Constrain to Last Line Only**
```rust
fn block_ref_tail_range(
    source: &str,
    block_range: SourceByteRange,
    scannable_ranges: &[std::ops::Range<usize>],
) -> Option<std::ops::Range<usize>> {
    let start = block_range.start().as_usize();
    let end = block_range.end().as_usize();

    // Find last newline in block
    let block_text = source.get(start..end)?;
    let last_line_start = block_text
        .rfind('\n')
        .map(|pos| start + pos + 1)
        .unwrap_or(start);

    // Only scan if last line overlaps with scannable ranges
    let last_line_range = last_line_start..end;
    let overlaps_scannable = scannable_ranges
        .iter()
        .any(|r| r.start < last_line_range.end && r.end > last_line_range.start);

    if overlaps_scannable && block_text.get((last_line_start - start)..)?.contains('^') {
        Some(last_line_range)
    } else {
        None
    }
}
```

**Benefits:**
- Respects scannable_ranges contract
- Only scans last line (typical location for block refs)
- Avoids detecting refs in code/links

### Finding 5: pulldown-cmark Options

**Current Implementation:**
```rust
// parser.rs:282-292
pub const fn obsidian_options() -> Options {
    Options::ENABLE_TASKLISTS
        .union(Options::ENABLE_WIKILINKS)
        .union(Options::ENABLE_YAML_STYLE_METADATA_BLOCKS)
        .union(Options::ENABLE_PLUSES_DELIMITED_METADATA_BLOCKS)
        .union(Options::ENABLE_HEADING_ATTRIBUTES)  // ❌ Not needed
        .union(Options::ENABLE_TABLES)              // ❌ Not needed
        .union(Options::ENABLE_FOOTNOTES)           // ❌ Not needed
        .union(Options::ENABLE_STRIKETHROUGH)       // ✅ Keep (helps separate)
        .union(Options::ENABLE_MATH)                // ❌ Not needed
}
```

**User Decisions:**
1. Keep only: tasklists, wikilinks, metadata blocks, strikethrough
2. Rename `obsidian_options()` → `extension_options()`
3. Keep unused options as conditional flags for future user configuration

**Solution:**
```rust
pub const fn extension_options() -> Options {
    let mut opts = Options::ENABLE_TASKLISTS
        .union(Options::ENABLE_WIKILINKS)
        .union(Options::ENABLE_YAML_STYLE_METADATA_BLOCKS)
        .union(Options::ENABLE_PLUSES_DELIMITED_METADATA_BLOCKS)
        .union(Options::ENABLE_STRIKETHROUGH);

    // Future: make these configurable via user settings
    if cfg!(feature = "tables") {
        opts = opts.union(Options::ENABLE_TABLES);
    }
    if cfg!(feature = "footnotes") {
        opts = opts.union(Options::ENABLE_FOOTNOTES);
    }
    if cfg!(feature = "heading-attributes") {
        opts = opts.union(Options::ENABLE_HEADING_ATTRIBUTES);
    }
    if cfg!(feature = "math") {
        opts = opts.union(Options::ENABLE_MATH);
    }

    opts
}
```

**Alternative (simpler for now):**
```rust
pub const fn extension_options() -> Options {
    Options::ENABLE_TASKLISTS
        .union(Options::ENABLE_WIKILINKS)
        .union(Options::ENABLE_YAML_STYLE_METADATA_BLOCKS)
        .union(Options::ENABLE_PLUSES_DELIMITED_METADATA_BLOCKS)
        .union(Options::ENABLE_STRIKETHROUGH)
}
```

Keep the conditional logic for when we add user configuration support.

---

## Solution Design

### Architecture Changes

```
BEFORE:
┌─────────────────────────────────────────────┐
│ MarkdownParser::parse()                     │
├─────────────────────────────────────────────┤
│ 1. metadata_ranges() - PASS 1               │
│ 2. if has_ref_links:                        │
│      reference_definitions_map() - PASS 2   │
│ 3. Main event loop - PASS 3                 │
│    ├─ Track blocks & scannable_ranges       │
│    └─ On block end:                         │
│       ├─ scan_block_artifacts()             │
│       │  ├─ scan scannable_ranges           │
│       │  └─ fallback: scan last 512 bytes   │
│       └─ For list items:                    │
│          └─ scan_task_marker() - RESCAN     │
└─────────────────────────────────────────────┘
          ↓
   NoteScanner.scan_ranges()
   (buggy boundary state)

AFTER:
┌─────────────────────────────────────────────┐
│ MarkdownParser::parse()                     │
├─────────────────────────────────────────────┤
│ 1. Create Parser + OffsetIter               │
│ 2. Get ref_defs = iter.reference_defs()     │
│ 3. Main event loop - SINGLE PASS            │
│    ├─ Track blocks & scannable_ranges       │
│    └─ On block end:                         │
│       ├─ scan_block_artifacts()             │
│       │  ├─ scan scannable_ranges           │
│       │  ├─ if list item + checkbox:        │
│       │  │   scan FIRST LINE for marker     │
│       │  └─ fallback: scan LAST LINE only   │
│       └─ No separate task marker scan       │
└─────────────────────────────────────────────┘
          ↓
   NoteScanner.scan_ranges()
   (correct boundary state from actual source)
```

### Data Flow Changes

**Reference Link Resolution:**
```rust
// BEFORE: Manual scanning
if is_reference_link_type(link_type) && reference_map.is_none() {
    reference_map = Some(reference_definitions_map(markdown, &metadata_ranges));
}
let map = reference_map.as_ref().unwrap_or(&empty_reference_map);

// AFTER: Use pulldown-cmark RefDefs
let ref_defs = offset_iter.reference_definitions();
if is_reference_link_type(link_type) {
    let target = ref_defs
        .get(dest_url.as_ref())
        .map(|def| Cow::Owned(def.dest.to_string()))
        .unwrap_or_else(|| cow_str_to_cow(dest_url));
}
```

**Task Marker Detection:**
```rust
// BEFORE: Full block rescan
let task_marker = if is_checked.is_some() {
    scanner.scan_task_marker(markdown, block_range)?  // Full block
} else {
    None
};

// AFTER: Scan first line only
let task_marker = if is_checked.is_some() {
    let first_line_end = markdown[block_start..]
        .find('\n')
        .map(|pos| block_start + pos)
        .unwrap_or(block_end)
        .min(block_start + 80);  // Cap at 80 chars
    let prefix_range = SourceByteRange::new(
        block_range.start(),
        SourceByteOffset::try_from(first_line_end)?
    )?;
    scanner.scan_task_marker(markdown, prefix_range)?
} else {
    None
};
```

**Scanner Boundary State:**
```rust
// BEFORE: State carried across ranges
let mut cursor = Cursor::new("", SourceByteOffset::new(0));
for range in ranges {
    cursor.reset(segment, base_offset);  // prev_alnum persists
    self.scan_cursor(&mut cursor, artifacts)?;
}

// AFTER: State derived from actual source
for range in ranges {
    let prev_alnum = if range.start > 0 {
        text.as_bytes()
            .get(range.start - 1)
            .map(|&b| b.is_ascii_alphanumeric())
            .unwrap_or(false)
    } else {
        false
    };
    let mode = derive_mode_from_source(text, range.start);
    let mut cursor = Cursor::new_with_state(segment, base_offset, mode, prev_alnum);
    self.scan_cursor(&mut cursor, artifacts)?;
}
```

---

## Implementation Plan

### Phase 1: Preparation (No Breaking Changes)

**Step 1.1: Add `Cursor::new_with_state()` constructor**
- File: `lithos-core/src/note/scanner.rs`
- Add method to create cursor with explicit `mode` and `prev_alnum`
- Keep existing `new()` for backward compatibility
- Estimated LOC: +10

```rust
impl<'source> Cursor<'source> {
    // Existing
    pub fn new(text: &'source str, base_offset: SourceByteOffset) -> Self {
        Self::new_with_state(text, base_offset, ScanMode::AtLineStart, false)
    }

    // New
    pub fn new_with_state(
        text: &'source str,
        base_offset: SourceByteOffset,
        mode: ScanMode,
        prev_alnum: bool,
    ) -> Self {
        Self {
            rest: text,
            offset: base_offset,
            mode,
            prev_alnum,
        }
    }
}
```

**Step 1.2: Make `ScanMode` pub(crate)**
- File: `lithos-core/src/note/scanner.rs`
- Change `enum ScanMode` visibility from private to `pub(crate)`
- Needed for parser to set initial mode
- Estimated LOC: 1 line change

```rust
// Change from:
enum ScanMode { ... }
// To:
pub(crate) enum ScanMode { ... }
```

**Step 1.3: Add tests for boundary scenarios**
- File: `lithos-core/src/note/scanner.rs` (tests module)
- Test cases for tags/refs adjacent to skipped text
- Estimated LOC: +50

```rust
#[test]
fn should_not_detect_tag_adjacent_to_link() {
    let scanner = scanner_fixture();
    let text = "[[link]]#tag";
    // Ranges: skip "[[link]]", only scan "#tag"
    let ranges = vec![8..12]; // "#tag"
    let mut artifacts = Vec::new();

    // Before fix: would incorrectly detect #tag
    // After fix: should NOT detect (no word boundary)
    scanner.scan_ranges(text, &ranges, &mut artifacts).unwrap();
    assert!(artifacts.is_empty(), "Should not detect tag without word boundary");
}
```

### Phase 2: Scanner Boundary Fix

**Step 2.1: Update `NoteScanner::scan_ranges()` implementation**
- File: `lithos-core/src/note/scanner.rs:176-201`
- Derive `prev_alnum` and `mode` from actual source boundaries
- Estimated LOC: ~30 (replace existing loop body)

```rust
pub fn scan_ranges<'source>(
    &self,
    text: &'source str,
    ranges: &[std::ops::Range<usize>],
    artifacts: &mut Vec<ScannedArtifact<'source>>,
) -> Result<(), NoteError> {
    for range in ranges {
        if range.is_empty() {
            continue;
        }
        let Some(segment) = text.get(range.clone()) else {
            continue;
        };
        let base_offset = SourceByteOffset::try_from(range.start)
            .map_err(|_err| {
                crate::note::error::StructureError::OutOfBounds {
                    offset: range.start,
                    source_len: text.len(),
                }
            })?;

        // Derive boundary state from actual source
        let prev_alnum = if range.start > 0 {
            text.as_bytes()
                .get(range.start - 1)
                .map(|&b| b.is_ascii_alphanumeric())
                .unwrap_or(false)
        } else {
            false
        };

        let mode = if range.start == 0 {
            ScanMode::AtLineStart
        } else if let Some(&prev_byte) = text.as_bytes().get(range.start - 1) {
            if prev_byte == b'\n' || prev_byte == b'\r' {
                ScanMode::AtLineStart
            } else {
                ScanMode::InBody
            }
        } else {
            ScanMode::InBody
        };

        let mut cursor = Cursor::new_with_state(segment, base_offset, mode, prev_alnum);
        self.scan_cursor(&mut cursor, artifacts)?;
    }
    Ok(())
}
```

**Step 2.2: Run scanner tests**
- Command: `mise run test:unit:note -- scanner`
- Verify boundary fix works correctly
- Expected: New boundary test passes, existing tests still pass

### Phase 3: Reference Definition Refactor

**Step 3.1: Remove manual reference scanning code**
- File: `lithos-core/src/note/parser.rs`
- Remove functions (lines to delete: ~250):
  - `metadata_ranges()` (294-326)
  - `reference_definitions_map()` (1187-1196)
  - `scan_reference_definitions()` (1238-1333)
  - `parse_reference_destination()` (1336-1346)
  - `parse_multiline_destination()` (1348-1367)
  - `is_line_in_ranges()` (1369-1375)
  - `advance_line_offset()` (1377-1386)
  - `is_indented_code_line()` (1388-1390)
  - `split_leading_spaces()` (1392-1406)
  - `line_fence_marker()` (1408-1424)
  - `normalize_reference_label()` (1206-1236)
  - Helper types: `FenceState`, `ReferenceDef`, etc.

**Step 3.2: Update `parse()` to use RefDefs**
- File: `lithos-core/src/note/parser.rs:63-276`
- Changes:
  1. Remove `metadata_ranges` variable and call
  2. Remove `reference_map` variable (Option<ReferenceMap>)
  3. Get `ref_defs` from offset_iter before consuming
  4. Update reference resolution in `handle_start_tag()`

```rust
// Remove these lines:
let metadata_ranges = Self::metadata_ranges(markdown);
let mut reference_map: Option<ReferenceMap> = None;
let empty_reference_map = ReferenceMap::new();

// Add after creating offset_iter:
let parser = Parser::new_ext(markdown, Self::extension_options());
let offset_iter = parser.into_offset_iter();

// Get reference definitions BEFORE consuming iterator
let ref_defs = offset_iter.reference_definitions();

// Update TextMergeWithOffset to use offset_iter:
let iter = TextMergeWithOffset::new(offset_iter);
```

**Step 3.3: Update `handle_start_tag()` reference resolution**
- File: `lithos-core/src/note/parser.rs:345-461`
- Replace reference_map lookup with ref_defs
- Update function signature to accept `ref_defs: &RefDefs<'source>`

```rust
// Change signature from:
fn handle_start_tag<'source>(
    tag: pulldown_cmark::Tag<'source>,
    start_pos: SourceByteOffset,
    reference_map: &ReferenceMap,  // Remove
    // ...
)

// To:
fn handle_start_tag<'source>(
    tag: pulldown_cmark::Tag<'source>,
    start_pos: SourceByteOffset,
    ref_defs: &pulldown_cmark::RefDefs<'source>,  // Add
    // ...
)

// Update Link/Image handling:
pulldown_cmark::Tag::Link { link_type, dest_url, .. } => {
    let target = resolve_reference_target(link_type, dest_url, ref_defs);
    // ...
}
```

**Step 3.4: Update `resolve_reference_target()` helper**
- File: `lithos-core/src/note/parser.rs:1173-1185`
- Change from HashMap lookup to RefDefs.get()

```rust
fn resolve_reference_target<'source>(
    link_type: pulldown_cmark::LinkType,
    dest_url: CowStr<'source>,
    ref_defs: &pulldown_cmark::RefDefs<'source>,
) -> Cow<'source, str> {
    if is_reference_link_type(link_type) {
        if let Some(link_def) = ref_defs.get(dest_url.as_ref()) {
            return Cow::Owned(link_def.dest.to_string());
        }
    }
    cow_str_to_cow(dest_url)
}
```

**Step 3.5: Rename `obsidian_options()` → `extension_options()`**
- File: `lithos-core/src/note/parser.rs:278-292`
- Rename method
- Update all call sites
- Remove unused options (tables, footnotes, math, heading_attributes)

```rust
pub const fn extension_options() -> Options {
    Options::ENABLE_TASKLISTS
        .union(Options::ENABLE_WIKILINKS)
        .union(Options::ENABLE_YAML_STYLE_METADATA_BLOCKS)
        .union(Options::ENABLE_PLUSES_DELIMITED_METADATA_BLOCKS)
        .union(Options::ENABLE_STRIKETHROUGH)
}
```

**Step 3.6: Run parser tests**
- Command: `mise run test:unit:note -- parser`
- Verify reference link resolution still works
- Expected: All tests pass, reference links resolved correctly

### Phase 4: Task Marker Optimization

**Step 4.1: Update `finalize_list_item()` to scan first line only**
- File: `lithos-core/src/note/parser.rs:922-1026`
- Replace full-block scan with first-line scan
- Estimated LOC: ~15 (replace task_marker detection logic)

```rust
// In finalize_list_item(), replace lines 950-956 with:
let task_marker = if is_checked.is_some() {
    // Only scan first line (where checkbox must be)
    let block_start = block_range.start().as_usize();
    let block_slice = markdown.get(block_start..).unwrap_or("");
    let first_line_len = block_slice
        .find('\n')
        .unwrap_or(block_slice.len())
        .min(80);  // Cap at 80 chars (checkboxes are always near start)

    let first_line_end = SourceByteOffset::try_from(block_start + first_line_len)
        .map_err(|_err| {
            #[expect(clippy::as_conversions, reason = "u32::MAX fits in usize")]
            NoteIngestError::Domain(
                StructureError::OutOfBounds {
                    offset: block_start + first_line_len,
                    source_len: u32::MAX as usize,
                }
                .into(),
            )
        })?;

    let prefix_range = SourceByteRange::new(block_range.start(), first_line_end)
        .map_err(NoteIngestError::Domain)?;

    scanner
        .scan_task_marker(markdown, prefix_range)
        .map_err(NoteIngestError::Domain)?
} else {
    None
};
```

**Step 4.2: Add test for task marker in long list items**
- File: `lithos-core/src/note/parser.rs` (tests module)
- Verify marker detected even with 100+ char content
- Estimated LOC: +15

```rust
#[test]
fn should_detect_task_marker_in_long_item() {
    let long_text = "x".repeat(200);
    let md = format!("- [x] {}", long_text);
    let raw = parse_raw(&md);
    assert_eq!(raw.list_items.len(), 1);
    let item = &raw.list_items[0];
    assert!(matches!(
        item.task_marker,
        Some(RawTaskMarker::Checked('x'))
    ));
}
```

### Phase 5: Block-ref Tail Scan Fix

**Step 5.1: Update `block_ref_tail_range()` signature and logic**
- File: `lithos-core/src/note/parser.rs:820-844`
- Add `scannable_ranges` parameter
- Constrain to last line only
- Check overlap with scannable_ranges
- Estimated LOC: ~40 (replace existing function)

```rust
fn block_ref_tail_range(
    source: &str,
    block_range: SourceByteRange,
    scannable_ranges: &[std::ops::Range<usize>],
) -> Option<std::ops::Range<usize>> {
    let start = block_range.start().as_usize();
    let end = block_range.end().as_usize();
    if end <= start {
        return None;
    }

    let block_slice = source.get(start..end)?;

    // Find last newline to identify last line
    let last_line_offset = block_slice
        .rfind('\n')
        .map(|pos| pos + 1)
        .unwrap_or(0);
    let last_line_start = start + last_line_offset;

    // Check if last line contains '^'
    let last_line_slice = block_slice.get(last_line_offset..)?;
    if !last_line_slice.contains('^') {
        return None;
    }

    // Verify last line overlaps with scannable ranges
    let last_line_range = last_line_start..end;
    let overlaps_scannable = scannable_ranges
        .iter()
        .any(|r| {
            r.start < last_line_range.end && r.end > last_line_range.start
        });

    if overlaps_scannable {
        Some(last_line_range)
    } else {
        None
    }
}
```

**Step 5.2: Update call site in `scan_block_artifacts()`**
- File: `lithos-core/src/note/parser.rs:752-784`
- Pass `scannable_ranges` to `block_ref_tail_range()`

```rust
if block_refs.is_empty()
    && let Some(tail_range) =
        Self::block_ref_tail_range(source, block_range, scannable_ranges)
{
    let tail_tokens = scanner
        .scan_ranges_raw(source, std::slice::from_ref(&tail_range), false)
        .map_err(NoteIngestError::Domain)?;
    block_refs.extend(tail_tokens.block_refs);
}
```

**Step 5.3: Add test for block-ref in code at end**
- File: `lithos-core/src/note/parser.rs` (tests module)
- Verify block refs in code aren't detected
- Estimated LOC: +15

```rust
#[test]
fn should_not_detect_block_ref_in_trailing_code() {
    let md = "Paragraph text\n`code ^not-a-ref`";
    let raw = parse_raw(md);
    assert!(
        raw.block_refs.is_empty(),
        "Should not detect block ref in code"
    );
}

#[test]
fn should_detect_block_ref_on_last_line() {
    let md = "Line 1\nLine 2 ^actual-ref";
    let raw = parse_raw(md);
    assert_eq!(raw.block_refs.len(), 1);
    assert_eq!(raw.block_refs[0].id, "actual-ref");
}
```

### Phase 6: Integration & Cleanup

**Step 6.1: Remove unused types and functions**
- File: `lithos-core/src/note/parser.rs`
- Remove:
  - `type ReferenceMap = std::collections::HashMap<Box<str>, Box<str>>`
  - `type ReferenceDef = (Box<str>, Box<str>)`
  - `struct FenceState`
- Estimated LOC: -10

**Step 6.2: Update all call sites**
- Search for `obsidian_options()` → replace with `extension_options()`
- Verify all `handle_start_tag()` calls updated
- Files to check:
  - `lithos-core/src/note/parser.rs`
  - Any integration tests

**Step 6.3: Run full test suite**
- Command: `mise run test`
- Verify all tests pass
- Fix any test failures

**Step 6.4: Run linting and formatting**
- Commands:
  - `mise run fmt`
  - `mise run lint`
- Fix any clippy warnings

**Step 6.5: Update documentation**
- File: `lithos-core/src/note/parser.rs` (module doc)
- Update to reflect single-pass design
- Mention RefDefs usage

```rust
//! Markdown parser and extraction.
//!
//! This module provides the primary ingestion engine for Obsidian-compatible
//! markdown files. It uses a **single-pass** event stream driven by
//! `pulldown-cmark` to extract both structural components (headings, sections,
//! lists) and specialized metadata (tags, inline fields, block references,
//! frontmatter).
//!
//! Reference link definitions are obtained directly from pulldown-cmark's
//! `RefDefs` API, eliminating the need for manual scanning.
//!
//! The main entry point is [`MarkdownParser`].
```

---

## Testing Strategy

### Unit Tests

**Scanner Boundary Tests** (`scanner.rs`)
- `should_not_detect_tag_adjacent_to_link()`
- `should_not_detect_block_ref_after_link()`
- `should_detect_tag_with_actual_word_boundary()`
- `should_respect_line_start_after_newline_in_ranges()`

**Reference Resolution Tests** (`parser.rs`)
- `should_resolve_reference_links_with_refdefs()`
- `should_handle_case_insensitive_references()`
- `should_handle_reference_with_title()`
- `should_fallback_for_undefined_references()`

**Task Marker Tests** (`parser.rs`)
- `should_detect_task_marker_in_first_line()`
- `should_detect_task_marker_in_long_item()`
- `should_not_rescan_full_block_for_marker()`

**Block-ref Tail Tests** (`parser.rs`)
- `should_detect_block_ref_on_last_line()`
- `should_not_detect_block_ref_in_trailing_code()`
- `should_not_detect_block_ref_in_trailing_link()`
- `should_constrain_tail_scan_to_last_line()`

### Integration Tests

**End-to-End Parsing** (`lithos-core/tests/note_ingestion.rs`)
- Parse complete note with all features
- Verify reference links resolved correctly
- Verify tags/fields/refs detected correctly with boundary semantics
- Verify task markers detected in list items

### Performance Tests

**Benchmark Pass Count**
- Before: measure passes via instrumentation
- After: verify single pass with RefDefs
- Expected: 2-3x faster on reference-heavy documents

### Regression Tests

**Existing Test Suite**
- Run `mise run test:unit:note`
- Ensure no existing tests break
- Fix any tests that relied on old behavior

---

## Risk Assessment

### High Risk

**Risk: RefDefs API behavior differs from manual parsing**
- Mitigation: Extensive testing with reference link edge cases
- Rollback: Keep old functions commented out for one release

**Risk: Scanner boundary fix breaks existing valid detections**
- Mitigation: Comprehensive boundary test coverage
- Rollback: Revert scanner.rs changes if tests fail

### Medium Risk

**Risk: Task marker optimization misses markers in unusual formatting**
- Mitigation: Test with various list formats, indentation, long lines
- Rollback: Easy to revert to full-block scan if needed

**Risk: Block-ref tail scan constraint misses valid refs**
- Mitigation: Analyze existing note corpus for ref placement patterns
- Rollback: Can widen scan scope if false negatives found

### Low Risk

**Risk: Options change breaks compatibility**
- Mitigation: Keep strikethrough for separation; only remove unused extensions
- Rollback: Easy to re-add options if needed

---

## Success Criteria

1. **Performance**: Single streaming pass (no metadata_ranges or reference scanning)
2. **Correctness**: Scanner boundary tests pass (no false positives near links)
3. **Efficiency**: Task markers found without full-block rescans
4. **Quality**: All existing tests pass, no regressions
5. **Code reduction**: ~250 lines of reference parsing code removed

---

## Implementation Checklist

- [ ] Phase 1: Preparation
  - [ ] Add `Cursor::new_with_state()`
  - [ ] Make `ScanMode` pub(crate)
  - [ ] Add boundary test cases
- [ ] Phase 2: Scanner Boundary Fix
  - [ ] Update `scan_ranges()` implementation
  - [ ] Run scanner tests
- [ ] Phase 3: Reference Definition Refactor
  - [ ] Remove manual reference scanning code
  - [ ] Update `parse()` to use RefDefs
  - [ ] Update `handle_start_tag()` signature
  - [ ] Update `resolve_reference_target()`
  - [ ] Rename `obsidian_options()` → `extension_options()`
  - [ ] Run parser tests
- [ ] Phase 4: Task Marker Optimization
  - [ ] Update `finalize_list_item()`
  - [ ] Add long list item test
- [ ] Phase 5: Block-ref Tail Scan Fix
  - [ ] Update `block_ref_tail_range()`
  - [ ] Update call site
  - [ ] Add tail scan tests
- [ ] Phase 6: Integration & Cleanup
  - [ ] Remove unused types
  - [ ] Update call sites
  - [ ] Run full test suite
  - [ ] Run fmt/lint
  - [ ] Update documentation

---

## Post-Implementation

### Metrics to Track

- Parser pass count (expect: 1)
- Parse time for reference-heavy documents (expect: 2-3x improvement)
- Memory usage (expect: slight decrease from removed HashMap allocations)
- Lines of code (expect: ~250 line reduction)

### Follow-up Tasks

1. Consider making extensions user-configurable (tables, footnotes, math)
2. Benchmark scanner boundary derivation overhead (may need caching)
3. Profile block-ref tail scan usage (consider removing if rarely needed)
4. Document RefDefs API usage in codebase style guide

---

## Appendix: Code Size Impact

| File | Before (LOC) | After (LOC) | Change |
|------|--------------|-------------|--------|
| `parser.rs` | ~1510 | ~1280 | -230 |
| `scanner.rs` | ~1095 | ~1125 | +30 |
| Tests | ~400 | ~480 | +80 |
| **Total** | ~3005 | ~2885 | **-120** |

**Summary**: Net reduction of ~120 lines while adding tests and fixing bugs.
