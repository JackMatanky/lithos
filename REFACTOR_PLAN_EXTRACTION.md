# Extraction Refactor Implementation Plan

**Status**: Ready for Implementation
**Created**: 2026-03-03
**Estimated Effort**: 4-5 engineer-days (TDD adds 1 day)
**Goal**: Replace 2,186-line god-object with composable extractor architecture
**Methodology**: **Test-Driven Development (TDD)** - Red → Green → Refactor

---

## Table of Contents

1. [Overview](#overview)
2. [TDD Framework](#tdd-framework)
3. [Architecture Changes](#architecture-changes)
4. [File Structure](#file-structure)
5. [Implementation Phases (TDD Cycles)](#implementation-phases-tdd-cycles)
6. [Detailed Component Specifications](#detailed-component-specifications)
7. [Edge Cases & Error Handling](#edge-cases--error-handling)
8. [Testing Strategy](#testing-strategy)
9. [Migration Checklist](#migration-checklist)
10. [Rollback Plan](#rollback-plan)

---

## TDD Framework

### TDD Principles for This Refactor

This refactor follows **strict Test-Driven Development**:

1. **RED**: Write a failing test first
2. **GREEN**: Write minimal code to make the test pass
3. **REFACTOR**: Improve code quality without changing behavior

### TDD Rules (Non-Negotiable)

✅ **ALWAYS write test before implementation**
✅ **NEVER write production code without a failing test**
✅ **Write the simplest code to make test pass**
✅ **Refactor only when tests are green**
✅ **Run tests after every change**
✅ **Commit on green, not on red**

### TDD Cycle Template

```rust
// 1. RED: Write failing test
#[test]
fn extracts_simple_heading() {
    let mut extractor = HeadingExtractor::new();  // ← Doesn't exist yet
    // ... test implementation
    assert_eq!(heading.text(), "Title");  // ← Will fail
}

// Run: cargo test extracts_simple_heading
// Expected: Compilation error (HeadingExtractor doesn't exist)

// 2. GREEN: Minimal implementation
pub struct HeadingExtractor {
    current: Option<HeadingBuilder>,
}

impl HeadingExtractor {
    pub fn new() -> Self {
        Self { current: None }
    }
}

impl Extractor for HeadingExtractor {
    // ... minimal implementation to pass test
}

// Run: cargo test extracts_simple_heading
// Expected: Test passes

// 3. REFACTOR: Improve code
// - Extract helper functions
// - Add documentation
// - Improve naming
// - Remove duplication

// Run: cargo test
// Expected: All tests still pass
```

### TDD Benefits for This Refactor

1. **Prevents regression**: Old behavior captured in tests
2. **Living documentation**: Tests show how extractors work
3. **Confident refactoring**: Green tests = safe to refactor
4. **Better design**: Test-first encourages composability
5. **Fast feedback**: Know immediately when something breaks

### Test Organization

```
lithos-core/
├── src/note/adapter/
│   ├── extract_list.rs
│   │   └── #[cfg(test)] mod tests { ... }  ← Unit tests
│   ├── extract_link.rs
│   │   └── #[cfg(test)] mod tests { ... }
│   └── ...
│
├── tests/
│   ├── note_extraction_integration.rs  ← Integration tests
│   └── note_extraction_property.rs      ← Property-based tests
│
└── benches/
    └── note_extraction.rs               ← Performance tests
```

### TDD Metrics (Track These)

| Metric | Target | Actual |
|--------|--------|--------|
| Test-first coverage | 100% | ___ |
| Tests written before code | 100% | ___ |
| Red-green-refactor cycles | All | ___ |
| Time in red state | <5 min/cycle | ___ |
| Commits on green | 100% | ___ |

---

## Overview

### Problem Statement

Current `reader.rs` (2,186 lines) is a god-object with:
- 12 interdependent collectors
- No isolation for testing
- Cross-collector state coupling
- Duplicated tag/task parsing logic
- Impossible to compose (parse all or nothing)

### Solution Architecture

**Extractor Pattern**: Each markdown element type gets a focused extractor that:
1. Implements `Extractor` trait (defined in `reader.rs`)
2. Processes pulldown-cmark events independently
3. Emits domain entities when patterns complete
4. Uses `ExtractionContext` for shared global state (no cross-extractor coupling)

### Key Design Decisions

| Decision | Rationale |
|----------|-----------|
| **Flat structure** | Rust convention: subdirectory only needed for 10+ related files |
| **Protocol in `reader.rs`** | Reader orchestrates extraction; extractors are implementation details |
| **`pub(super)` visibility** | Protocol visible to sibling `extract_*.rs` but not outside `adapter/` |
| **Unified list extraction** | Checkboxes ARE list items; task promotion is post-processing |
| **`CowStr<'_>` everywhere** | Zero-copy text handling from pulldown-cmark |
| **`EmbedType` in domain** | Extension detection is domain logic, not adapter concern |
| **Bidirectional `FieldValue`** | Add `to_yaml_value()` and `to_toml_value()` for round-tripping |

---

## Architecture Changes

### Before

```
note/adapter/
├── mod.rs
├── reader.rs           (2,186 lines - god-object)
├── tag_scanner.rs      (54 lines)
├── task_parser.rs      (483 lines)
├── command.rs          (storage adapter)
└── query.rs            (storage adapter)
```

**Problems**:
- All extraction logic in one file
- No isolation for testing
- Duplicated tag scanning (inline and frontmatter)
- Task parser tightly coupled to reader

### After

```
note/adapter/
├── mod.rs                   (Module organization, public API)
├── reader.rs                (~200 lines: protocol + orchestration)
├── extract_list.rs          (~300 lines: lists + checkboxes + tasks)
├── extract_link.rs          (~150 lines: links + anchors)
├── extract_heading.rs       (~100 lines: headings)
├── extract_section.rs       (~150 lines: sections)
├── extract_frontmatter.rs   (~120 lines: YAML/TOML)
├── extract_tag.rs           (~80 lines: tag scanning)
├── command.rs               (unchanged)
└── query.rs                 (unchanged)
```

**Benefits**:
- Each extractor testable in isolation
- Clear separation of concerns
- Zero cross-extractor coupling
- Composable (can extract subsets)
- ~1,100 lines vs 2,186 (48% reduction)

---

## File Structure

### Module Organization

```rust
// note/adapter/mod.rs

//! Note adapters for storage and markdown ingestion.

// PUBLIC API
pub use reader::{NoteReader, ParseOutcome};

// Storage adapters (keep as-is)
pub mod command;
pub mod query;

// Internal extraction infrastructure
mod reader;                // Protocol + orchestration
mod extract_list;          // Lists + checkboxes + task promotion
mod extract_link;          // Links + anchors
mod extract_heading;       // Headings
mod extract_section;       // Section boundaries
mod extract_frontmatter;   // YAML/TOML frontmatter
mod extract_tag;           // Tag scanning
```

### Domain Enhancements

```rust
// note/link.rs (add to existing file)

impl EmbedType {
    /// Determine embed type from file extension.
    pub fn from_extension(path: &str) -> Self {
        // Move logic from LinkCollector::determine_embed_type
    }
}

// note/value.rs (add to existing file)

impl FieldValue {
    /// Convert to `serde_yaml::Value` for round-tripping.
    pub fn to_yaml_value(&self) -> serde_yaml::Value {
        // Inverse of from_yaml
    }

    /// Convert to `toml::Value` for round-tripping.
    pub fn to_toml_value(&self) -> Result<toml::Value, FieldValueConversionError> {
        // Inverse of from_json/from_yaml
    }
}
```

---

## Implementation Phases (TDD Cycles)

**CRITICAL**: Every task follows Red → Green → Refactor cycle

### Phase 1: Foundation (Day 1, Morning - 4 hours)

**TDD Approach**: Write tests for domain enhancements first

**Goal**: Establish extraction protocol and domain enhancements

---

#### Task 1.1: `EmbedType::from_extension` (TDD - 60 min)

##### RED Phase (15 min)

**Write tests FIRST** in `lithos-core/src/note/link.rs`:

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn detects_image_extensions() {
        assert_eq!(EmbedType::from_extension("image.png"), EmbedType::Image);
        assert_eq!(EmbedType::from_extension("photo.jpg"), EmbedType::Image);
        assert_eq!(EmbedType::from_extension("pic.jpeg"), EmbedType::Image);
        assert_eq!(EmbedType::from_extension("icon.gif"), EmbedType::Image);
        assert_eq!(EmbedType::from_extension("logo.svg"), EmbedType::Image);
        assert_eq!(EmbedType::from_extension("hero.webp"), EmbedType::Image);
    }

    #[test]
    fn detects_video_extensions() {
        assert_eq!(EmbedType::from_extension("video.mp4"), EmbedType::Video);
        assert_eq!(EmbedType::from_extension("clip.webm"), EmbedType::Video);
        assert_eq!(EmbedType::from_extension("movie.ogv"), EmbedType::Video);
        assert_eq!(EmbedType::from_extension("recording.mov"), EmbedType::Video);
    }

    #[test]
    fn detects_audio_extensions() {
        assert_eq!(EmbedType::from_extension("song.mp3"), EmbedType::Audio);
        assert_eq!(EmbedType::from_extension("sound.wav"), EmbedType::Audio);
        assert_eq!(EmbedType::from_extension("audio.ogg"), EmbedType::Audio);
        assert_eq!(EmbedType::from_extension("track.m4a"), EmbedType::Audio);
    }

    #[test]
    fn detects_pdf_extension() {
        assert_eq!(EmbedType::from_extension("doc.pdf"), EmbedType::Pdf);
        assert_eq!(EmbedType::from_extension("Doc.PDF"), EmbedType::Pdf);
    }

    #[test]
    fn case_insensitive_matching() {
        assert_eq!(EmbedType::from_extension("IMAGE.PNG"), EmbedType::Image);
        assert_eq!(EmbedType::from_extension("Video.MP4"), EmbedType::Video);
        assert_eq!(EmbedType::from_extension("Audio.MP3"), EmbedType::Audio);
    }

    #[test]
    fn no_extension_defaults_to_note() {
        assert_eq!(EmbedType::from_extension("filename"), EmbedType::Note);
    }

    #[test]
    fn unknown_extension_defaults_to_note() {
        assert_eq!(EmbedType::from_extension("file.txt"), EmbedType::Note);
        assert_eq!(EmbedType::from_extension("doc.docx"), EmbedType::Note);
    }
}
```

**Run**: `cargo test detects_image_extensions`
**Expected**: ❌ Compilation error - `EmbedType::from_extension` doesn't exist

##### GREEN Phase (20 min)

**Add minimal implementation** to make tests pass:

```rust
// note/link.rs

impl EmbedType {
    /// Determine embed type from file extension.
    ///
    /// Uses case-insensitive matching without allocation.
    #[must_use]
    pub fn from_extension(path: &str) -> Self {
        let Some((_, ext)) = path.rsplit_once('.') else {
            return Self::Note;
        };

        if matches_any_ignore_case(ext, &["png", "jpg", "jpeg", "gif", "svg", "webp"]) {
            return Self::Image;
        }
        if matches_any_ignore_case(ext, &["mp4", "webm", "ogv", "mov"]) {
            return Self::Video;
        }
        if matches_any_ignore_case(ext, &["mp3", "wav", "ogg", "m4a"]) {
            return Self::Audio;
        }
        if ext.eq_ignore_ascii_case("pdf") {
            return Self::Pdf;
        }
        Self::Note
    }
}

fn matches_any_ignore_case(s: &str, candidates: &[&str]) -> bool {
    candidates.iter().any(|c| s.eq_ignore_ascii_case(c))
}
```

**Run**: `cargo test -p lithos-core --lib note::link::tests`
**Expected**: ✅ All tests pass

##### REFACTOR Phase (15 min)

- Add doc comments with examples
- Extract `matches_any_ignore_case` to module-level (if used elsewhere)
- Add `#[inline]` where appropriate
- Run clippy: `cargo clippy --tests`

**Run**: `cargo test -p lithos-core --lib note::link::tests`
**Expected**: ✅ All tests still pass

##### Commit (5 min)

```bash
git add lithos-core/src/note/link.rs
git commit -m "feat(note): add EmbedType::from_extension

- Add case-insensitive extension matching
- Support image/video/audio/pdf detection
- Default to Note for unknown extensions
- Add 7 unit tests covering all cases

TDD: Red → Green → Refactor"
```

---

#### Task 1.2: `FieldValue` Conversion Methods (TDD - 90 min)

##### RED Phase (30 min)

**Write tests FIRST** in `lithos-core/src/note/value.rs`:

```rust
#[cfg(test)]
mod conversion_tests {
    use super::*;

    #[test]
    fn yaml_round_trip_string() {
        let original = FieldValue::String("test".into());
        let yaml = original.to_yaml_value();
        let round_trip = FieldValue::from_yaml(&yaml).unwrap();
        assert_eq!(original, round_trip);
    }

    #[test]
    fn yaml_round_trip_number() {
        let original = FieldValue::Number(42.5);
        let yaml = original.to_yaml_value();
        let round_trip = FieldValue::from_yaml(&yaml).unwrap();
        assert_eq!(original, round_trip);
    }

    #[test]
    fn yaml_round_trip_boolean() {
        let original = FieldValue::Boolean(true);
        let yaml = original.to_yaml_value();
        let round_trip = FieldValue::from_yaml(&yaml).unwrap();
        assert_eq!(original, round_trip);
    }

    #[test]
    fn yaml_round_trip_array() {
        let original = FieldValue::Array(vec![
            FieldValue::String("a".into()),
            FieldValue::Number(1.0),
            FieldValue::Boolean(false),
        ]);
        let yaml = original.to_yaml_value();
        let round_trip = FieldValue::from_yaml(&yaml).unwrap();
        assert_eq!(original, round_trip);
    }

    #[test]
    fn yaml_round_trip_object() {
        let mut map = HashMap::new();
        map.insert("key".into(), FieldValue::String("value".into()));
        let original = FieldValue::Object(map);
        let yaml = original.to_yaml_value();
        let round_trip = FieldValue::from_yaml(&yaml).unwrap();
        assert_eq!(original, round_trip);
    }

    #[test]
    fn toml_round_trip_string() {
        let original = FieldValue::String("test".into());
        let toml = original.to_toml_value().unwrap();
        // TOML → JSON → FieldValue (existing path)
        let json = serde_json::to_value(&toml).unwrap();
        let round_trip = FieldValue::from_json(&json).unwrap();
        assert_eq!(original, round_trip);
    }

    #[test]
    fn toml_converts_whole_numbers_to_integer() {
        let original = FieldValue::Number(42.0);
        let toml = original.to_toml_value().unwrap();
        assert!(matches!(toml, toml::Value::Integer(42)));
    }

    #[test]
    fn toml_converts_floats() {
        let original = FieldValue::Number(42.5);
        let toml = original.to_toml_value().unwrap();
        assert!(matches!(toml, toml::Value::Float(_)));
    }
}
```

**Run**: `cargo test conversion_tests`
**Expected**: ❌ Compilation error - methods don't exist

##### GREEN Phase (40 min)

**Add minimal implementation**:

```rust
// note/value.rs

impl FieldValue {
    /// Convert this `FieldValue` to a `serde_yaml::Value`.
    ///
    /// Useful for writing updated frontmatter back to files.
    #[must_use]
    pub fn to_yaml_value(&self) -> serde_yaml::Value {
        match self {
            Self::String(s) => serde_yaml::Value::String(s.to_string()),
            Self::Number(n) => serde_yaml::Value::Number(
                serde_yaml::Number::from(*n)
            ),
            Self::Boolean(b) => serde_yaml::Value::Bool(*b),
            Self::Date(ts) => {
                let datetime = chrono::DateTime::from_timestamp(*ts, 0)
                    .unwrap_or_default();
                serde_yaml::Value::String(datetime.to_rfc3339())
            }
            Self::Array(arr) => {
                let seq: Vec<_> = arr.iter().map(|v| v.to_yaml_value()).collect();
                serde_yaml::Value::Sequence(seq)
            }
            Self::Object(obj) => {
                let mut map = serde_yaml::Mapping::new();
                for (key, value) in obj {
                    map.insert(
                        serde_yaml::Value::String(key.to_string()),
                        value.to_yaml_value(),
                    );
                }
                serde_yaml::Value::Mapping(map)
            }
        }
    }

    /// Convert this `FieldValue` to a `toml::Value`.
    ///
    /// # Errors
    /// Returns error if timestamp cannot be converted to TOML datetime.
    pub fn to_toml_value(&self) -> Result<toml::Value, FieldValueConversionError> {
        match self {
            Self::String(s) => Ok(toml::Value::String(s.to_string())),
            Self::Number(n) => {
                if n.is_finite() && n.trunc() == *n {
                    Ok(toml::Value::Integer(*n as i64))
                } else {
                    Ok(toml::Value::Float(*n))
                }
            }
            Self::Boolean(b) => Ok(toml::Value::Boolean(*b)),
            Self::Date(ts) => {
                let datetime = chrono::DateTime::from_timestamp(*ts, 0)
                    .ok_or(FieldValueConversionError::InvalidTimestamp)?;
                Ok(toml::Value::Datetime(toml::value::Datetime {
                    date: Some(toml::value::Date {
                        year: datetime.year() as u16,
                        month: datetime.month() as u8,
                        day: datetime.day() as u8,
                    }),
                    time: Some(toml::value::Time {
                        hour: datetime.hour() as u8,
                        minute: datetime.minute() as u8,
                        second: datetime.second() as u8,
                        nanosecond: datetime.nanosecond(),
                    }),
                    offset: None,
                }))
            }
            Self::Array(arr) => {
                let vec: Result<Vec<_>, _> = arr.iter().map(|v| v.to_toml_value()).collect();
                Ok(toml::Value::Array(vec?))
            }
            Self::Object(obj) => {
                let mut map = toml::map::Map::new();
                for (key, value) in obj {
                    map.insert(key.to_string(), value.to_toml_value()?);
                }
                Ok(toml::Value::Table(map))
            }
        }
    }
}

// Add error type
#[derive(Debug)]
pub enum FieldValueConversionError {
    InvalidTimestamp,
}

impl std::fmt::Display for FieldValueConversionError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InvalidTimestamp => write!(f, "invalid timestamp for conversion"),
        }
    }
}

impl std::error::Error for FieldValueConversionError {}
```

**Run**: `cargo test conversion_tests`
**Expected**: ✅ All tests pass

##### REFACTOR Phase (10 min)

- Add documentation examples
- Improve error messages
- Add `#[inline]` for small functions
- Run clippy

**Run**: `cargo test conversion_tests`
**Expected**: ✅ All tests still pass

##### Commit (5 min)

```bash
git add lithos-core/src/note/value.rs
git commit -m "feat(note): add FieldValue bidirectional conversion

- Add to_yaml_value() for YAML serialization
- Add to_toml_value() for TOML serialization
- Add FieldValueConversionError for conversion failures
- Add 8 round-trip tests

TDD: Red → Green → Refactor"
```

---

#### Task 1.3: Extraction Protocol (TDD - 60 min)

##### RED Phase (20 min)

**Write tests FIRST** in `lithos-core/src/note/adapter/reader.rs`:

```rust
#[cfg(test)]
mod protocol_tests {
    use super::*;

    #[test]
    fn extraction_context_defaults() {
        let ctx = ExtractionContext::default();
        assert!(!ctx.inside_link);
        assert!(!ctx.inside_code_block);
        assert_eq!(ctx.list_depth, 0);
    }

    #[test]
    fn extraction_state_is_continue() {
        let state: ExtractionState<String> = ExtractionState::Continue;
        assert!(matches!(state, ExtractionState::Continue));
    }

    #[test]
    fn extraction_state_is_emit() {
        let state = ExtractionState::Emit("value".to_string());
        assert!(matches!(state, ExtractionState::Emit(_)));
    }

    // Mock extractor for testing protocol
    struct MockExtractor {
        calls: usize,
    }

    impl Extractor for MockExtractor {
        type Output = String;
        type Error = NoteError;

        fn process(
            &mut self,
            _event: &Event<'_>,
            _text: CowStr<'_>,
            _range: Range<usize>,
            _ctx: &ExtractionContext,
        ) -> Result<ExtractionState<String>, NoteError> {
            self.calls += 1;
            if self.calls == 3 {
                Ok(ExtractionState::Emit("entity".to_string()))
            } else {
                Ok(ExtractionState::Continue)
            }
        }

        fn finish(self) -> Result<Vec<String>, NoteError> {
            Ok(vec![])
        }
    }

    #[test]
    fn mock_extractor_emits_on_third_call() {
        let mut extractor = MockExtractor { calls: 0 };
        let ctx = ExtractionContext::default();
        let event = Event::Text(CowStr::Borrowed("test"));

        // First call
        let result = extractor.process(&event, CowStr::Borrowed("test"), 0..4, &ctx).unwrap();
        assert!(matches!(result, ExtractionState::Continue));

        // Second call
        let result = extractor.process(&event, CowStr::Borrowed("test"), 4..8, &ctx).unwrap();
        assert!(matches!(result, ExtractionState::Continue));

        // Third call - should emit
        let result = extractor.process(&event, CowStr::Borrowed("test"), 8..12, &ctx).unwrap();
        match result {
            ExtractionState::Emit(value) => assert_eq!(value, "entity"),
            _ => panic!("Expected Emit"),
        }
    }
}
```

**Run**: `cargo test protocol_tests`
**Expected**: ❌ Compilation error - types don't exist

##### GREEN Phase (25 min)

**Add protocol types** to `reader.rs`:

```rust
// note/adapter/reader.rs

/// Extraction context shared across all extractors.
#[derive(Debug, Default, Clone)]
pub(super) struct ExtractionContext {
    pub inside_link: bool,
    pub inside_code_block: bool,
    pub list_depth: usize,
}

/// Extraction state returned after processing an event.
#[derive(Debug)]
pub(super) enum ExtractionState<T> {
    Continue,
    Emit(T),
}

/// Extracts typed domain entities from pulldown-cmark event stream.
pub(super) trait Extractor {
    type Output;
    type Error: Into<NoteError>;

    fn process(
        &mut self,
        event: &Event<'_>,
        text: CowStr<'_>,
        range: Range<usize>,
        ctx: &ExtractionContext,
    ) -> Result<ExtractionState<Self::Output>, Self::Error>;

    fn finish(self) -> Result<Vec<Self::Output>, Self::Error>;
}
```

**Run**: `cargo test protocol_tests`
**Expected**: ✅ All tests pass

##### REFACTOR Phase (10 min)

- Add comprehensive documentation with examples
- Add doc comments explaining each field/method
- Ensure `pub(super)` visibility is correct

**Run**: `cargo test protocol_tests`
**Expected**: ✅ All tests still pass

##### Commit (5 min)

```bash
git add lithos-core/src/note/adapter/reader.rs
git commit -m "feat(adapter): add extraction protocol

- Add ExtractionContext for shared state
- Add ExtractionState enum for results
- Add Extractor trait with process/finish methods
- Use pub(super) for module-local visibility
- Add protocol tests with mock extractor

TDD: Red → Green → Refactor"
```

#### Phase 1 Summary

✅ **Foundation complete in ~4 hours**
✅ **All tests written BEFORE implementation**
✅ **3 commits, all on green**
✅ **Ready for Phase 2**

---

---

### Phase 2: Core Extractors (Day 1 Afternoon - Day 2 Morning - 8 hours)

**TDD Approach**: Test-first for all extractor functionality

**Goal**: Implement high-value extractors (lists, links, headings)

---

#### Task 2.1: List Extractor (TDD - 3 hours)

**CRITICAL**: This is the most complex extractor - follow TDD strictly

**File**: `note/adapter/extract_list.rs`

##### RED Phase 1: Basic List Extraction (30 min)

**Write tests FIRST**:

```rust
// lithos-core/src/note/adapter/extract_list.rs

#[cfg(test)]
mod tests {
    use super::*;
    use pulldown_cmark::{CowStr, Event, Tag as CmarkTag, TagEnd};

    #[test]
    fn extracts_plain_unordered_list() {
        let mut extractor = ListExtractor::new(&test_config());
        let ctx = ExtractionContext::default();

        // Start list
        let result = extractor.process(
            &Event::Start(CmarkTag::List(None)),
            CowStr::Borrowed(""),
            0..2,
            &ctx,
        ).unwrap();
        assert!(matches!(result, ExtractionState::Continue));

        // Start item
        extractor.process(
            &Event::Start(CmarkTag::Item),
            CowStr::Borrowed(""),
            2..4,
            &ctx,
        ).unwrap();

        // Item text
        extractor.process(
            &Event::Text(CowStr::Borrowed("Buy milk")),
            CowStr::Borrowed("Buy milk"),
            4..12,
            &ctx,
        ).unwrap();

        // End item
        extractor.process(
            &Event::End(TagEnd::Item),
            CowStr::Borrowed(""),
            12..13,
            &ctx,
        ).unwrap();

        // End list - should emit
        let result = extractor.process(
            &Event::End(TagEnd::List(false)),
            CowStr::Borrowed(""),
            13..14,
            &ctx,
        ).unwrap();

        match result {
            ExtractionState::Emit(ExtractionOutput::List(list)) => {
                assert_eq!(list.items().count(), 1);
                assert!(matches!(list.list_type(), ListType::Unordered));
            }
            _ => panic!("Expected list emission"),
        }
    }

    #[test]
    fn extracts_ordered_list() {
        let mut extractor = ListExtractor::new(&test_config());
        let ctx = ExtractionContext::default();

        // Start list with start number
        extractor.process(
            &Event::Start(CmarkTag::List(Some(1))),
            CowStr::Borrowed(""),
            0..2,
            &ctx,
        ).unwrap();

        // ... add items ...

        let result = extractor.process(
            &Event::End(TagEnd::List(true)),
            CowStr::Borrowed(""),
            20..21,
            &ctx,
        ).unwrap();

        match result {
            ExtractionState::Emit(ExtractionOutput::List(list)) => {
                assert!(matches!(list.list_type(), ListType::Ordered { start: 1 }));
            }
            _ => panic!("Expected list emission"),
        }
    }

    fn test_config() -> Config {
        // Helper to create test config
    }
}
```

**Run**: `cargo test extract_list`
**Expected**: ❌ Compilation error - `ListExtractor` doesn't exist

##### GREEN Phase 1: Minimal List Implementation (45 min)

**Add minimal structs and basic implementation**:

```rust
// lithos-core/src/note/adapter/extract_list.rs

use pulldown_cmark::{CowStr, Event, Tag as CmarkTag, TagEnd};
use std::ops::Range;

use crate::{
    config::Config,
    note::{
        error::NoteError,
        list::{List, ListDepth, ListItem, ListType},
        position::SourceByteOffset,
    },
};

use super::reader::{Extractor, ExtractionContext, ExtractionState};

pub struct ListExtractor<'config> {
    config: &'config Config,
    list_stack: Vec<List>,
    completed_lists: Vec<List>,
    current_item: Option<ItemBuilder>,
}

struct ItemBuilder {
    position: SourceByteOffset,
    text: String,
    is_checkbox: bool,
}

impl ItemBuilder {
    fn new(position: SourceByteOffset) -> Self {
        Self {
            position,
            text: String::new(),
            is_checkbox: false,
        }
    }
}

pub enum ExtractionOutput {
    List(List),
    Task(Task),
}

impl<'config> ListExtractor<'config> {
    pub fn new(config: &'config Config) -> Self {
        Self {
            config,
            list_stack: Vec::new(),
            completed_lists: Vec::new(),
            current_item: None,
        }
    }
}

impl<'config> Extractor for ListExtractor<'config> {
    type Output = ExtractionOutput;
    type Error = NoteError;

    fn process(
        &mut self,
        event: &Event<'_>,
        text: CowStr<'_>,
        range: Range<usize>,
        _ctx: &ExtractionContext,
    ) -> Result<ExtractionState<ExtractionOutput>, NoteError> {
        match event {
            Event::Start(CmarkTag::List(start)) => {
                let depth = ListDepth::try_new(self.list_stack.len())?;
                let list_type = match start {
                    Some(start) => ListType::Ordered { start: *start },
                    None => ListType::Unordered,
                };
                self.list_stack.push(List::with_depth(list_type, depth));
                Ok(ExtractionState::Continue)
            }

            Event::Start(CmarkTag::Item) => {
                let position = SourceByteOffset::try_from_usize(range.start)?;
                self.current_item = Some(ItemBuilder::new(position));
                Ok(ExtractionState::Continue)
            }

            Event::Text(_) => {
                if let Some(item) = self.current_item.as_mut() {
                    item.text.push_str(&text);
                }
                Ok(ExtractionState::Continue)
            }

            Event::End(TagEnd::Item) => {
                if let Some(item) = self.current_item.take() {
                    if let Some(list) = self.list_stack.last_mut() {
                        list.add_item(ListItem::Plain {
                            text: item.text.trim().into(),
                            position: item.position,
                        });
                    }
                }
                Ok(ExtractionState::Continue)
            }

            Event::End(TagEnd::List(_)) => {
                if let Some(list) = self.list_stack.pop() {
                    return Ok(ExtractionState::Emit(ExtractionOutput::List(list)));
                }
                Ok(ExtractionState::Continue)
            }

            _ => Ok(ExtractionState::Continue),
        }
    }

    fn finish(self) -> Result<Vec<ExtractionOutput>, NoteError> {
        Ok(self.list_stack.into_iter().map(ExtractionOutput::List).collect())
    }
}
```

**Run**: `cargo test extract_list`
**Expected**: ✅ Tests pass

##### REFACTOR Phase 1 (15 min)

- Extract helper methods
- Add documentation
- Improve naming

**Run**: `cargo test extract_list`
**Expected**: ✅ Tests still pass

##### RED Phase 2: Checkbox Support (20 min)

**Add checkbox tests**:

```rust
#[test]
fn extracts_checkbox_without_promotion() {
    let mut extractor = ListExtractor::new(&test_config());
    let ctx = ExtractionContext::default();

    // ... list start ...

    // Item start
    extractor.process(&Event::Start(CmarkTag::Item), ...).unwrap();

    // Checkbox marker
    extractor.process(&Event::TaskListMarker(false), ...).unwrap();

    // Text
    extractor.process(&Event::Text(CowStr::Borrowed("Buy milk")), ...).unwrap();

    // Item end
    extractor.process(&Event::End(TagEnd::Item), ...).unwrap();

    // List end
    let result = extractor.process(&Event::End(TagEnd::List(false)), ...).unwrap();

    match result {
        ExtractionState::Emit(ExtractionOutput::List(list)) => {
            let item = list.items().next().unwrap();
            assert!(matches!(item, ListItem::Checkbox { .. }));
            assert!(item.task_id().is_none()); // Not promoted
        }
        _ => panic!("Expected list"),
    }
}
```

**Run**: `cargo test extracts_checkbox_without_promotion`
**Expected**: ❌ Test fails - checkbox not handled

##### GREEN Phase 2: Checkbox Implementation (30 min)

**Add checkbox handling**:

```rust
// Update ItemBuilder
struct ItemBuilder {
    position: SourceByteOffset,
    text: String,
    tag_scan_text: String,
    is_checkbox: bool,
    status_symbol: Option<char>,
}

impl ItemBuilder {
    fn mark_as_checkbox(&mut self, checked: bool) {
        self.is_checkbox = true;
        self.status_symbol = Some(if checked { 'x' } else { ' ' });
    }
}

// Update process method
Event::TaskListMarker(checked) => {
    if let Some(item) = self.current_item.as_mut() {
        item.mark_as_checkbox(*checked);
    }
    Ok(ExtractionState::Continue)
}

Event::End(TagEnd::Item) => {
    if let Some(item) = self.current_item.take() {
        if let Some(list) = self.list_stack.last_mut() {
            if item.is_checkbox {
                let status = crate::config::task::StatusSymbol::try_new(
                    item.status_symbol.unwrap_or(' ')
                )?;
                list.add_item(ListItem::Checkbox {
                    text: item.text.trim().into(),
                    status,
                    position: item.position,
                    task_id: None,
                });
            } else {
                list.add_item(ListItem::Plain {
                    text: item.text.trim().into(),
                    position: item.position,
                });
            }
        }
    }
    Ok(ExtractionState::Continue)
}
```

**Run**: `cargo test extract_list`
**Expected**: ✅ All tests pass

##### RED Phase 3: Task Promotion (30 min)

**Add task promotion test**:

```rust
#[test]
fn promotes_checkbox_with_task_tag() {
    let mut extractor = ListExtractor::new(&test_config_with_task_tag());
    let ctx = ExtractionContext::default();

    // ... list and item start ...

    // Checkbox marker
    extractor.process(&Event::TaskListMarker(false), ...).unwrap();

    // Text with promotion tag
    extractor.process(
        &Event::Text(CowStr::Borrowed("#task Review PR")),
        CowStr::Borrowed("#task Review PR"),
        ...,
    ).unwrap();

    // Item end - should emit task
    let result = extractor.process(&Event::End(TagEnd::Item), ...).unwrap();

    match result {
        ExtractionState::Emit(ExtractionOutput::Task(task)) => {
            assert_eq!(task.text(), "Review PR");
        }
        _ => panic!("Expected task emission"),
    }
}
```

**Run**: `cargo test promotes_checkbox_with_task_tag`
**Expected**: ❌ Test fails - promotion not implemented

##### GREEN Phase 3: Task Promotion Implementation (45 min)

**Add promotion logic** (reuse from task_parser.rs):

```rust
// Add to ItemBuilder
fn build(self, config: &Config) -> Result<(ListItem, Option<Task>), NoteError> {
    let text = self.text.trim();

    if !self.is_checkbox {
        return Ok((
            ListItem::Plain {
                text: text.into(),
                position: self.position,
            },
            None,
        ));
    }

    let status = crate::config::task::StatusSymbol::try_new(
        self.status_symbol.unwrap_or(' ')
    )?;

    // Scan for tags
    let tags = TagScanner::new(&self.tag_scan_text).collect_tags();

    // Check promotion
    let promoted_task = if should_promote_to_task(&tags, config) {
        Some(promote_checkbox_to_task(text, tags, status, self.position, config)?)
    } else {
        None
    };

    let task_id = promoted_task.as_ref().map(|t| t.id());
    let list_item = ListItem::Checkbox {
        text: text.into(),
        status,
        position: self.position,
        task_id,
    };

    Ok((list_item, promoted_task))
}
```

**Run**: `cargo test extract_list`
**Expected**: ✅ All tests pass

##### REFACTOR Phase 3 (20 min)

- Extract helper functions
- Add comprehensive documentation
- Clean up code

##### Commit (5 min)

```bash
git add lithos-core/src/note/adapter/extract_list.rs
git commit -m "feat(adapter): add list extractor with task promotion

- Extract plain lists (ordered/unordered)
- Extract checkboxes with status
- Promote checkboxes to tasks based on tags
- Link tasks to list items via task_id
- Add 10+ unit tests covering all cases

TDD: Multiple Red-Green-Refactor cycles"
```

**Key Types**:

```rust
pub struct ListExtractor<'config> {
    config: &'config Config,
    list_stack: Vec<List>,
    completed_lists: Vec<List>,
    current_item: Option<ItemBuilder>,
}

struct ItemBuilder {
    position: SourceByteOffset,
    text: String,
    tag_scan_text: String,  // Excludes code/links
    is_checkbox: bool,
    status_symbol: Option<char>,
}

pub enum ExtractionOutput {
    List(List),
    Task(Task),
}
```

**Edge Cases**:
- Nested lists (depth tracking)
- Checkbox without status marker
- Empty checkbox text (should error)
- Checkbox with only whitespace
- Code blocks inside list items
- Links inside list items (don't scan for tags)
- Multiple consecutive lists
- Unclosed list at EOF

**Tests**:
```rust
#[test] fn extracts_plain_list()
#[test] fn extracts_ordered_list()
#[test] fn extracts_nested_lists()
#[test] fn extracts_checkbox_without_promotion()
#[test] fn promotes_checkbox_with_task_tag()
#[test] fn links_checkbox_to_task_via_task_id()
#[test] fn ignores_tags_in_code()
#[test] fn ignores_tags_in_links()
#[test] fn handles_unclosed_list()
#[test] fn handles_empty_checkbox_text()
```

---

#### Task 2.2: Link Extractor

**File**: `note/adapter/extract_link.rs`

**Responsibilities**:
1. Parse wiki-links: `[[target]]`, `[[target|alias]]`
2. Parse markdown links: `[text](url)`
3. Parse embeds: `![[image.png]]`, `![alt](image.jpg)`
4. Parse anchors: `[[note#heading]]`, `[[note#^block-id]]`
5. Detect external URLs (scheme-based)
6. Collect alias text across multiple Text events
7. Use `EmbedType::from_extension` for embed classification

**Key Types**:

```rust
pub struct LinkExtractor<'config> {
    config: &'config Config,
    current: Option<LinkBuilder>,
}

struct LinkBuilder {
    target: Box<str>,
    alias: Option<String>,
    position: SourceByteOffset,
    is_embed: bool,
    is_wikilink: bool,
    is_markdown_image: bool,
    collect_alias: bool,
}
```

**Edge Cases**:
- Wiki-link with pothole: `[[target|alias]]`
- Anchor parsing: `#heading` vs `#^block-ref`
- Empty target (error)
- External URL detection (http://, https://, ftp://, mailto:, etc.)
- Wiki-link without closing `]]`
- Markdown link without closing `)`
- Nested links (invalid markdown, but pulldown-cmark may emit)
- Image embeds vs regular images

**Tests**:
```rust
#[test] fn extracts_simple_wikilink()
#[test] fn extracts_wikilink_with_alias()
#[test] fn extracts_wikilink_with_heading_anchor()
#[test] fn extracts_wikilink_with_block_anchor()
#[test] fn extracts_markdown_link()
#[test] fn extracts_markdown_image()
#[test] fn extracts_embed_with_image_extension()
#[test] fn extracts_embed_with_video_extension()
#[test] fn extracts_embed_with_audio_extension()
#[test] fn detects_external_url()
#[test] fn handles_unclosed_link()
#[test] fn handles_empty_target()
```

---

#### Task 2.3: Heading Extractor

**File**: `note/adapter/extract_heading.rs`

**Responsibilities**:
1. Detect heading start (H1-H6)
2. Accumulate heading text across multiple Text events
3. Handle breaks in heading text
4. Emit heading when closed
5. Validate heading level (1-6)

**Key Types**:

```rust
pub struct HeadingExtractor {
    current: Option<HeadingBuilder>,
}

struct HeadingBuilder {
    level: HeadingLevel,
    text: String,
    position: SourceByteOffset,
}
```

**Edge Cases**:
- Empty heading text (allowed by CommonMark)
- Heading with only whitespace
- Heading with inline code
- Heading with links
- Heading with breaks (convert to space)
- Unclosed heading at EOF

**Tests**:
```rust
#[test] fn extracts_h1_through_h6()
#[test] fn accumulates_text_across_events()
#[test] fn converts_breaks_to_spaces()
#[test] fn handles_empty_heading()
#[test] fn handles_heading_with_code()
#[test] fn handles_heading_with_link()
#[test] fn handles_unclosed_heading()
```

---

### Phase 3: Secondary Extractors (Day 2, Afternoon)

**Goal**: Implement section, frontmatter, and tag extractors

#### Task 3.1: Section Extractor

**File**: `note/adapter/extract_section.rs`

**Responsibilities**:
1. Track block boundaries (paragraphs, lists, code blocks, etc.)
2. Associate headings with subsequent sections
3. Track byte offsets for section ranges
4. Handle rule sections (horizontal rules)
5. Validate UTF-8 boundaries

**Key Types**:

```rust
pub struct SectionExtractor<'source> {
    source: &'source str,
    block_depth: u32,
    current: Option<SectionBuilder>,
    last_offset: usize,
    sections: Vec<Section>,
}

struct SectionBuilder {
    start: SourceByteOffset,
    heading: Option<Heading>,
    awaiting_heading: bool,
}
```

**Edge Cases**:
- Nested blocks (track depth)
- Section without heading
- Multiple sections with same heading (allowed)
- Empty section (heading with no content)
- Section at EOF (no closing block)
- Invalid UTF-8 boundaries (should error)

**Tests**:
```rust
#[test] fn creates_section_for_heading()
#[test] fn creates_section_without_heading()
#[test] fn handles_nested_blocks()
#[test] fn handles_rule_section()
#[test] fn handles_section_at_eof()
#[test] fn validates_utf8_boundaries()
```

---

#### Task 3.2: Frontmatter Extractor

**File**: `note/adapter/extract_frontmatter.rs`

**Responsibilities**:
1. Detect YAML frontmatter (`---`)
2. Detect TOML frontmatter (`+++`)
3. Accumulate text within metadata block
4. Parse YAML/TOML to `serde_yaml::Value`/`toml::Value`
5. Convert to `HashMap<Box<str>, FieldValue>`
6. Use `FieldValue::from_yaml`/`from_json` (TOML via JSON)
7. Handle parse errors gracefully

**Key Types**:

```rust
pub struct FrontmatterExtractor {
    kind: Option<pulldown_cmark::MetadataBlockKind>,
    text: String,
}
```

**Edge Cases**:
- Invalid YAML (syntax error)
- Invalid TOML (syntax error)
- Non-mapping root (YAML list at root)
- Non-string keys
- Nested structures
- Empty frontmatter block
- Multiple frontmatter blocks (only first should be used)
- Frontmatter with breaks (preserve as newlines)

**Tests**:
```rust
#[test] fn parses_yaml_frontmatter()
#[test] fn parses_toml_frontmatter()
#[test] fn handles_nested_structures()
#[test] fn handles_array_values()
#[test] fn errors_on_invalid_yaml()
#[test] fn errors_on_invalid_toml()
#[test] fn errors_on_non_mapping_root()
#[test] fn ignores_subsequent_blocks()
```

---

#### Task 3.3: Tag Extractor

**File**: `note/adapter/extract_tag.rs`

**Responsibilities**:
1. Scan text for Obsidian-style tags (`#tag`, `#nested/tag`)
2. Ignore tags in code blocks
3. Ignore tags in links
4. Extract tags from frontmatter (if present)
5. Deduplicate tags
6. Validate tag format

**Key Types**:

```rust
pub struct TagExtractor {
    tags: Vec<NoteTag>,
    tag_set: HashSet<Box<str>>,
    frontmatter_tags: Vec<NoteTag>,
}
```

**Edge Cases**:
- Tag at start of line
- Tag after whitespace
- Tag after punctuation
- Tag with numbers (`#tag123`)
- Tag with hyphens (`#my-tag`)
- Tag with underscores (`#my_tag`)
- Nested tags (`#parent/child`)
- Tag followed by punctuation
- Tags in code (ignore)
- Tags in links (ignore)
- Duplicate tags (deduplicate)
- Invalid tag characters

**Tests**:
```rust
#[test] fn extracts_simple_tag()
#[test] fn extracts_nested_tag()
#[test] fn extracts_tag_with_numbers()
#[test] fn extracts_tag_with_hyphens()
#[test] fn extracts_tag_with_underscores()
#[test] fn ignores_tag_in_code()
#[test] fn ignores_tag_in_link()
#[test] fn deduplicates_tags()
#[test] fn extracts_tags_from_frontmatter()
#[test] fn handles_invalid_tag_characters()
```

---

### Phase 4: Reader Orchestration (Day 3, Morning)

**Goal**: Wire extractors into reader orchestration

#### Task 4.1: Update `reader.rs`

**Changes**:

1. **Add extraction protocol** (already covered in Phase 1)

2. **Refactor `parse_str` method**:
   ```rust
   pub(crate) fn parse_str(&self, markdown: &str) -> Result<ParseOutcome, NoteError> {
       // Initialize extractors
       let mut link_ext = super::extract_link::LinkExtractor::new(self.config);
       let mut list_ext = super::extract_list::ListExtractor::new(self.config);
       let mut heading_ext = super::extract_heading::HeadingExtractor::new();
       let mut section_ext = super::extract_section::SectionExtractor::new(markdown);
       let mut frontmatter_ext = super::extract_frontmatter::FrontmatterExtractor::new();
       let mut tag_ext = super::extract_tag::TagExtractor::new();

       // Accumulators
       let mut links = Vec::new();
       let mut lists = Vec::new();
       let mut tasks = Vec::new();
       let mut headings = Vec::new();
       let mut sections = Vec::new();
       let mut frontmatter = None;
       let mut tags = Vec::new();

       // Extraction context
       let mut ctx = ExtractionContext::default();

       // Parse events with text merging
       let events = Parser::new_ext(markdown, obsidian_options()).into_offset_iter();
       let merged = TextMergeWithOffset::new(events);

       for (event, range) in merged {
           update_context(&mut ctx, &event);

           let text = match &event {
               Event::Text(t) | Event::Code(t) => t.clone(),
               _ => CowStr::Borrowed(""),
           };

           // Route to extractors
           if let ExtractionState::Emit(link) = link_ext.process(&event, text.clone(), range.clone(), &ctx)? {
               links.push(link);
           }

           if let ExtractionState::Emit(output) = list_ext.process(&event, text.clone(), range.clone(), &ctx)? {
               use super::extract_list::ExtractionOutput;
               match output {
                   ExtractionOutput::List(list) => lists.push(list),
                   ExtractionOutput::Task(task) => tasks.push(task),
               }
           }

           if let ExtractionState::Emit(heading) = heading_ext.process(&event, text.clone(), range.clone(), &ctx)? {
               headings.push(heading);
           }

           section_ext.process(&event, text.clone(), range.clone(), &ctx)?;
           frontmatter_ext.process(&event, text.clone(), range.clone(), &ctx)?;
           tag_ext.process(&event, text, range, &ctx)?;
       }

       // Finalize all extractors
       links.extend(link_ext.finish()?);

       for output in list_ext.finish()? {
           use super::extract_list::ExtractionOutput;
           match output {
               ExtractionOutput::List(list) => lists.push(list),
               ExtractionOutput::Task(task) => tasks.push(task),
           }
       }

       headings.extend(heading_ext.finish()?);
       sections.extend(section_ext.finish()?);
       if let Some(fm) = frontmatter_ext.finish()?.into_iter().next() {
           frontmatter = Some(fm);
       }
       tags.extend(tag_ext.finish()?);

       Ok(ParseOutcome {
           lists,
           tasks,
           headings,
           sections,
           frontmatter,
           links,
           tags,
       })
   }
   ```

3. **Add `update_context` helper**:
   ```rust
   fn update_context(ctx: &mut ExtractionContext, event: &Event<'_>) {
       use pulldown_cmark::{Tag as CmarkTag, TagEnd};

       match event {
           Event::Start(CmarkTag::Link { .. }) | Event::Start(CmarkTag::Image { .. }) => {
               ctx.inside_link = true;
           }
           Event::End(TagEnd::Link) | Event::End(TagEnd::Image) => {
               ctx.inside_link = false;
           }
           Event::Start(CmarkTag::CodeBlock(_)) => {
               ctx.inside_code_block = true;
           }
           Event::End(TagEnd::CodeBlock) => {
               ctx.inside_code_block = false;
           }
           Event::Start(CmarkTag::List(_)) => {
               ctx.list_depth = ctx.list_depth.saturating_add(1);
           }
           Event::End(TagEnd::List(_)) => {
               ctx.list_depth = ctx.list_depth.saturating_sub(1);
           }
           _ => {}
       }
   }
   ```

**Edge Cases**:
- Event cloning overhead (CowStr is cheap to clone)
- Error propagation from extractors
- Order of extractor finalization (some depend on others)

**Tests**:
```rust
#[test] fn orchestrates_all_extractors()
#[test] fn handles_extractor_errors()
#[test] fn updates_context_correctly()
#[test] fn finalizes_extractors_in_order()
```

---

### Phase 5: Integration & Testing (Day 4 - 6 hours)

**Goal**: Comprehensive testing and validation

**TDD Approach**: Characterization tests to ensure behavior matches current implementation

#### Task 5.1: Integration Tests (TDD - 2 hours)

**File**: `lithos-core/tests/note_extraction_integration.rs`

**TDD Approach**: Use existing passing tests as baseline, then verify new implementation

##### RED Phase (30 min)

**Capture current behavior**:

```rust
// First, run ALL existing tests with current implementation
// cargo test -p lithos-core --test '*'
// Document which tests pass and their exact assertions

#[test]
fn characterization_complex_note_with_all_elements() {
    // Copy markdown from current test
    let markdown = include_str!("fixtures/complex_note.md");
    let config = test_config();
    let reader = NoteReader::new(&config);

    // Parse with CURRENT implementation - record results
    let outcome = reader.parse_str(markdown).unwrap();

    // Document exact counts (these become our acceptance criteria)
    assert_eq!(outcome.headings().len(), 5, "heading count must match");
    assert_eq!(outcome.tasks().len(), 12, "task count must match");
    assert_eq!(outcome.links().len(), 8, "link count must match");
    assert_eq!(outcome.tags().len(), 6, "tag count must match");
    assert_eq!(outcome.lists().len(), 3, "list count must match");

    // Verify specific entity properties
    let first_task = &outcome.tasks()[0];
    assert_eq!(first_task.text(), "Review PR", "task text must match");
    // ... more assertions documenting current behavior
}
```

**Create characterization tests for**:
1. Complex note with all elements
2. Deeply nested lists (5 levels)
3. Real-world Obsidian note
4. Task-to-ListItem linkage
5. Frontmatter tag extraction
6. Unicode handling (emoji, CJK, RTL)
7. Malformed markdown edge cases

##### GREEN Phase (45 min)

**After refactor, run characterization tests**:
- All assertions should still pass
- Entity counts must match exactly
- Entity properties must match exactly
- No behavior changes allowed

##### REFACTOR Phase (15 min)

**Add new integration tests** that weren't possible before:
- Test individual extractors in isolation
- Test partial parsing (links only, tasks only)

```rust
#[test]
fn can_extract_links_only() {
    // This was impossible with old architecture
    let mut extractor = LinkExtractor::new(&config);
    // ... test that we can get links without parsing entire document
}
```

---

#### Task 5.2: Property-Based Tests (TDD - 2 hours)

**File**: `lithos-core/tests/note_extraction_property.rs`

**TDD Approach**: Write property tests to catch edge cases

##### RED Phase (45 min)

**Define properties that must hold**:

```rust
use proptest::prelude::*;

proptest! {
    // Property: All valid headings parse successfully
    #[test]
    fn extracts_any_valid_heading(text in ".*", level in 1u8..=6) {
        let markdown = format!("{} {}", "#".repeat(level as usize), text);
        let outcome = reader.parse_str(&markdown)?;

        // Property: Exactly one heading extracted
        assert_eq!(outcome.headings().len(), 1);
        // Property: Level preserved
        assert_eq!(outcome.headings()[0].level().as_u8(), level);
    }

    // Property: All valid tag names parse successfully
    #[test]
    fn handles_any_tag_name(name in "[a-z0-9_/-]{1,50}") {
        let markdown = format!("#{}", name);
        let outcome = reader.parse_str(&markdown)?;

        // Property: Exactly one tag extracted
        assert_eq!(outcome.tags().len(), 1);
        // Property: Name preserved
        assert_eq!(outcome.tags()[0].full_path(), name);
    }

    // Property: Parser never panics
    #[test]
    fn parser_never_panics(markdown in ".*") {
        // Property: Any input results in Ok or predictable Err
        let _ = reader.parse_str(&markdown);
        // If we get here without panic, property holds
    }

    // Property: Unicode safety (no invalid slicing)
    #[test]
    fn handles_any_unicode(text in "\\PC*") {
        let markdown = format!("# {}", text);
        let result = reader.parse_str(&markdown);

        // Property: No panic, either Ok or Err
        match result {
            Ok(_) | Err(_) => {} // Both are acceptable
        }
    }
}
```

##### GREEN Phase (60 min)

**Run property tests** - fix any failures:
- Most should pass immediately
- Unicode edge cases may reveal bugs
- Adjust properties if needed

##### REFACTOR Phase (15 min)

**Add more properties**:
- List nesting never exceeds 255 levels
- Task IDs in lists match task entities
- Section ranges are valid UTF-8 boundaries

---

#### Task 5.3: Performance Benchmarks

**File**: `lithos-core/benches/note_parsing.rs` (EXISTING - verify still comprehensive)

**Current Benchmarks** (already in place):
- `ingest_markdown/simple` - 91 bytes, ~13-14 µs, ~7 MiB/s
- `ingest_markdown/medium` - 500 bytes, ~18-19 µs, ~27 MiB/s
- `ingest_markdown/complex` - 2419 bytes, ~47-48 µs, ~50 MiB/s

**Refactor Verification Steps**:

1. **Run baseline before refactor**:
   ```bash
   cargo bench --bench note_parsing -- --save-baseline before_refactor
   ```

2. **After refactor, compare**:
   ```bash
   cargo bench --bench note_parsing -- --baseline before_refactor
   ```

3. **Verify no regression**:
   - Simple: Should remain ~13-14 µs (±10%)
   - Medium: Should remain ~18-19 µs (±10%)
   - Complex: Should remain ~47-48 µs (±10%)
   - Throughput should scale linearly: O(n)

**Acceptance Criteria**:
- [ ] No performance regression >10% for any benchmark
- [ ] Throughput scaling remains O(n) (sub-linear latency growth)
- [ ] All benchmarks complete successfully
- [ ] Baseline comparison shows "no change" or improvement

**If Regression Detected** (>10%):
1. Profile with `cargo flamegraph --bench note_parsing`
2. Identify hot path changes
3. Check for:
   - Extra allocations in extractors
   - Inefficient event cloning
   - Redundant validation
4. Optimize before merging

---

### Phase 6: Cleanup & Documentation (Day 4)

**Goal**: Remove old code, update documentation

#### Task 6.1: Delete Old Files

```bash
rm lithos-core/src/note/adapter/tag_scanner.rs
rm lithos-core/src/note/adapter/task_parser.rs
```

#### Task 6.2: Update `mod.rs`

Remove references to deleted modules.

#### Task 6.3: Update Documentation

**Files to update**:
- `lithos-core/src/note/adapter/mod.rs` - Module-level docs
- `ARCHITECTURE.md` - Update extraction section
- `CHANGELOG.md` - Add entry for refactor

#### Task 6.4: Update Tests

**Search for tests using old API**:
```bash
rg "tag_scanner::TagScanner" --type rust
rg "task_parser::TaskParser" --type rust
```

**Update to use new extractors** (if any test directly used old modules).

---

## Detailed Component Specifications

### Extraction Protocol

#### `ExtractionContext`

**Purpose**: Global parsing state shared across extractors

```rust
#[derive(Debug, Default, Clone)]
pub(super) struct ExtractionContext {
    /// Whether we're currently inside a link/image element.
    /// Used by list extractor to avoid scanning tags in link text.
    pub inside_link: bool,

    /// Whether we're currently inside a code block.
    /// Used by tag extractor to avoid scanning code.
    pub inside_code_block: bool,

    /// Current list nesting depth (0 = not in list).
    /// Can be used for validation or debugging.
    pub list_depth: usize,
}
```

**Invariants**:
- `list_depth` never exceeds 255 (validated by `ListDepth::try_new`)
- `inside_link` and `inside_code_block` are mutually exclusive (for tag scanning purposes)

---

#### `ExtractionState<T>`

**Purpose**: Return value from `Extractor::process`

```rust
#[derive(Debug)]
pub(super) enum ExtractionState<T> {
    /// Continue processing events.
    Continue,

    /// Emit a complete entity and continue processing.
    Emit(T),
}
```

**Usage**:
```rust
// In reader orchestration
if let ExtractionState::Emit(entity) = extractor.process(...)? {
    entities.push(entity);
}
```

---

#### `Extractor` Trait

**Purpose**: Protocol for all extraction strategies

```rust
pub(super) trait Extractor {
    /// The domain entity type this extractor produces.
    type Output;

    /// The error type for extraction failures.
    type Error: Into<NoteError>;

    /// Process a single event, updating internal state.
    ///
    /// # Arguments
    /// * `event` - pulldown-cmark event reference
    /// * `text` - CowStr for Text/Code events (empty otherwise)
    /// * `range` - Byte range in source markdown
    /// * `ctx` - Shared extraction context
    ///
    /// # Returns
    /// * `Continue` if no entity ready
    /// * `Emit(entity)` if entity completed
    fn process(
        &mut self,
        event: &Event<'_>,
        text: CowStr<'_>,
        range: Range<usize>,
        ctx: &ExtractionContext,
    ) -> Result<ExtractionState<Self::Output>, Self::Error>;

    /// Finalize extraction after all events processed.
    ///
    /// Returns any remaining entities that were being accumulated.
    /// For well-formed markdown, this typically returns empty Vec.
    fn finish(self) -> Result<Vec<Self::Output>, Self::Error>;
}
```

**Invariants**:
- `finish()` consumes `self` (no further processing possible)
- `process()` may emit entities at any time (not just at end)
- Extractors must handle malformed markdown gracefully

---

### List Extractor Deep Dive

#### State Machine

```
[Idle]
  ↓ Event::Start(List)
[InList] ← stack.push(List)
  ↓ Event::Start(Item)
[InItem] ← current_item = Some(ItemBuilder::new)
  ↓ Event::TaskListMarker(checked)
[InCheckbox] ← mark_as_checkbox(checked)
  ↓ Event::Text / Code / Break
[AccumulatingText] ← push_text()
  ↓ Event::End(Item)
[ItemComplete] ← build() → (ListItem, Option<Task>)
  ├─→ Add ListItem to current list
  └─→ If Task, emit via ExtractionState::Emit
[InList]
  ↓ Event::End(List)
[ListComplete] ← stack.pop() → emit List
[Idle]
```

#### Task Promotion Logic

```rust
fn should_promote_to_task(tags: &[Tag], config: &Config) -> bool {
    config.task().tags().iter().any(|promotion_tag| {
        tags.iter().any(|tag| {
            promotion_tag
                .as_str()
                .strip_prefix('#')
                .is_some_and(|raw| raw == tag.full_path())
        })
    })
}
```

**Promotion happens when**:
1. Item is checkbox (has TaskListMarker)
2. Item text contains at least one promotion tag (e.g., `#task`)
3. Status symbol is valid

**Task attributes parsed from**:
1. Inline fields: `[key:: value]`
2. Emoji dates: `📅 2026-03-15`
3. Tags in text: `#urgent`, `#project/lithos`

---

### Link Extractor Deep Dive

#### Anchor Parsing

```rust
fn parse_anchor(raw: &str) -> Option<Anchor> {
    if let Some(block_ref) = raw.strip_prefix("#^") {
        let block_ref = block_ref.trim();
        if block_ref.is_empty() {
            None
        } else {
            Some(Anchor::block_ref(block_ref)?)
        }
    } else {
        let heading = raw.strip_prefix('#').unwrap_or(raw).trim();
        if heading.is_empty() {
            None
        } else {
            Some(Anchor::heading(heading)?)
        }
    }
}
```

**Anchor types**:
- `#heading-text` → `Anchor::Heading("heading-text")`
- `#^block-id` → `Anchor::BlockRef("block-id")`

#### External URL Detection

```rust
fn is_external_link(link_type: LinkType, target: &str) -> bool {
    matches!(
        link_type,
        LinkType::Autolink | LinkType::Email
    ) || has_scheme(target)
}

fn has_scheme(target: &str) -> bool {
    let mut chars = target.chars();
    let Some(first) = chars.next() else {
        return false;
    };
    if !first.is_ascii_alphabetic() {
        return false;
    }
    for ch in chars {
        if ch == ':' {
            return true;
        }
        if !(ch.is_ascii_alphanumeric() || matches!(ch, '+' | '-' | '.')) {
            return false;
        }
    }
    false
}
```

**Schemes**: `http:`, `https:`, `ftp:`, `mailto:`, `file:`, `data:`, etc.

---

## Edge Cases & Error Handling

### Markdown Edge Cases

| Case | Behavior | Test |
|------|----------|------|
| Unclosed list at EOF | Close and emit in `finish()` | ✅ |
| Unclosed item at EOF | Error (malformed) | ✅ |
| Unclosed link at EOF | Error (malformed) | ✅ |
| Unclosed heading at EOF | Close and emit in `finish()` | ✅ |
| Empty checkbox text | Error (`TaskError::EmptyText`) | ✅ |
| Nested lists (5+ levels) | Track depth, validate ≤255 | ✅ |
| List without items | Empty list (valid) | ✅ |
| Heading without text | Empty heading (valid) | ✅ |
| Multiple frontmatters | Only first used | ✅ |
| Invalid YAML | Parse error | ✅ |
| Invalid TOML | Parse error | ✅ |

### Unicode Edge Cases

| Case | Behavior | Test |
|------|----------|------|
| Emoji in text | Preserved correctly | ✅ |
| CJK characters | UTF-8 boundaries respected | ✅ |
| RTL text | Preserved as-is | ✅ |
| Zero-width characters | Preserved | ✅ |
| Combining characters | Preserved | ✅ |

### Extraction Context Edge Cases

| Case | Behavior | Test |
|------|----------|------|
| Nested code blocks | Track depth (saturating add) | ✅ |
| Link inside link | pulldown-cmark prevents | N/A |
| Code inside link | `inside_link` takes precedence | ✅ |
| Deeply nested lists | `list_depth` tracks correctly | ✅ |

### Error Propagation

All extractors return `Result<_, Self::Error>` where `Self::Error: Into<NoteError>`.

**Error conversion chain**:
```rust
LinkExtractor::Error (LinkExtractionError)
    ↓ Into<NoteError>
NoteError::Link(LinkError)
    ↓ Propagated by ?
reader.parse_str() → Result<ParseOutcome, NoteError>
```

---

## Testing Strategy

### Unit Tests (per extractor)

**Location**: Each `extract_*.rs` file has `#[cfg(test)] mod tests`

**Coverage targets**:
- Happy path: Valid input produces expected output
- Edge cases: Boundary conditions, empty inputs
- Error cases: Invalid input produces expected error
- Context awareness: Correctly uses `ExtractionContext`

**Example structure**:
```rust
#[cfg(test)]
mod tests {
    use super::*;
    use pulldown_cmark::{Event, CowStr};

    fn make_event(event_type: &str) -> Event<'static> {
        // Helper to create test events
    }

    #[test]
    fn extracts_simple_case() {
        let mut extractor = MyExtractor::new(&config);
        let ctx = ExtractionContext::default();

        let result = extractor.process(
            &make_event("start"),
            CowStr::Borrowed("text"),
            0..4,
            &ctx,
        )?;

        assert!(matches!(result, ExtractionState::Continue));

        let result = extractor.process(
            &make_event("end"),
            CowStr::Borrowed(""),
            4..5,
            &ctx,
        )?;

        assert!(matches!(result, ExtractionState::Emit(_)));
    }
}
```

### Integration Tests

**Location**: `lithos-core/tests/note_extraction_integration.rs`

**Strategy**:
- Full markdown documents (not isolated events)
- Real-world Obsidian notes
- Verify interactions between extractors
- Cross-entity relationships (task_id links)

### Property-Based Tests

**Location**: `lithos-core/tests/note_extraction_property.rs`

**Strategy**:
- Use `proptest` to generate random markdown
- Verify invariants hold for all inputs
- Catch edge cases not covered by unit tests

**Properties to test**:
- Parse never panics
- Extracted entities count ≤ source element count
- Round-trip: Note → markdown → Note preserves entities
- Unicode safety: No invalid UTF-8 slicing

### Benchmark Tests

**Location**: `lithos-core/benches/note_extraction.rs`

**Strategy**:
- Small (100 lines), medium (1000 lines), large (5000 lines) notes
- Measure throughput (notes/sec)
- Compare to baseline (current implementation)

**Acceptance criteria**:
- No more than 5% performance regression
- Linear scaling with note size

---

## Migration Checklist

### Pre-Implementation

- [ ] Review architecture plan with team
- [ ] Create feature branch: `refactor/extraction-architecture`
- [ ] Set up tracking issue with sub-tasks
- [ ] Identify test fixtures for integration tests

### Phase 1: Foundation

- [ ] Add `EmbedType::from_extension` to `note/link.rs`
- [ ] Add unit tests for `EmbedType::from_extension`
- [ ] Add `FieldValue::to_yaml_value()` to `note/value.rs`
- [ ] Add `FieldValue::to_toml_value()` to `note/value.rs`
- [ ] Add round-trip tests for `FieldValue` conversion
- [ ] Add extraction protocol to `reader.rs` (pub(super))
- [ ] Document protocol with examples
- [ ] Run: `mise run test` → all pass
- [ ] Run: `mise run lint` → clean

### Phase 2: Core Extractors

#### List Extractor
- [ ] Create `extract_list.rs` with `ListExtractor` struct
- [ ] Implement `Extractor` trait for `ListExtractor`
- [ ] Add `ItemBuilder` helper struct
- [ ] Implement task promotion logic
- [ ] Add unit tests (10+ test cases)
- [ ] Run: `mise run test:unit:note` → all pass

#### Link Extractor
- [ ] Create `extract_link.rs` with `LinkExtractor` struct
- [ ] Implement `Extractor` trait for `LinkExtractor`
- [ ] Add `LinkBuilder` helper struct
- [ ] Implement anchor parsing
- [ ] Use `EmbedType::from_extension` for embed type
- [ ] Add unit tests (10+ test cases)
- [ ] Run: `mise run test:unit:note` → all pass

#### Heading Extractor
- [ ] Create `extract_heading.rs` with `HeadingExtractor` struct
- [ ] Implement `Extractor` trait for `HeadingExtractor`
- [ ] Add `HeadingBuilder` helper struct
- [ ] Add unit tests (7+ test cases)
- [ ] Run: `mise run test:unit:note` → all pass

### Phase 3: Secondary Extractors

#### Section Extractor
- [ ] Create `extract_section.rs` with `SectionExtractor` struct
- [ ] Implement `Extractor` trait for `SectionExtractor`
- [ ] Add `SectionBuilder` helper struct
- [ ] Add unit tests (6+ test cases)
- [ ] Run: `mise run test:unit:note` → all pass

#### Frontmatter Extractor
- [ ] Create `extract_frontmatter.rs` with `FrontmatterExtractor` struct
- [ ] Implement `Extractor` trait for `FrontmatterExtractor`
- [ ] Use `FieldValue::from_yaml` / `from_json`
- [ ] Add unit tests (8+ test cases)
- [ ] Run: `mise run test:unit:note` → all pass

#### Tag Extractor
- [ ] Create `extract_tag.rs` with `TagExtractor` struct
- [ ] Implement `Extractor` trait for `TagExtractor`
- [ ] Reuse tag scanning logic from `tag_scanner.rs`
- [ ] Add unit tests (10+ test cases)
- [ ] Run: `mise run test:unit:note` → all pass

### Phase 4: Reader Orchestration

- [ ] Refactor `reader.rs` `parse_str` method
- [ ] Initialize all extractors
- [ ] Implement event routing loop
- [ ] Add `update_context` helper
- [ ] Call `finish()` on all extractors
- [ ] Update unit tests for `NoteReader`
- [ ] Run: `mise run test:unit:note` → all pass

### Phase 5: Integration & Testing

- [ ] Capture baseline: `cargo bench --bench note_parsing -- --save-baseline before_refactor`
- [ ] Create `tests/note_extraction_integration.rs`
- [ ] Write characterization tests documenting current behavior
- [ ] Add 7+ integration test scenarios
- [ ] Create `tests/note_extraction_property.rs`
- [ ] Add 4+ property-based tests (proptest)
- [ ] Run: `mise run test` → all pass
- [ ] Verify benchmark: `cargo bench --bench note_parsing -- --baseline before_refactor`
- [ ] Ensure no regression >10% in any benchmark
- [ ] Update benchmark docs if needed

### Phase 6: Cleanup & Documentation

- [ ] Delete `tag_scanner.rs`
- [ ] Delete `task_parser.rs`
- [ ] Update `mod.rs` (remove old modules, add new ones)
- [ ] Update module-level documentation
- [ ] Update `ARCHITECTURE.md`
- [ ] Add entry to `CHANGELOG.md`
- [ ] Search for references to deleted modules: `rg "tag_scanner|task_parser"`
- [ ] Update any tests using old API
- [ ] Run: `mise run verify` → 100% pass

### Pre-Merge

- [ ] Code review by team
- [ ] All tests passing in CI
- [ ] No performance regression
- [ ] Documentation complete
- [ ] CHANGELOG updated
- [ ] Squash commits into logical units
- [ ] Write detailed merge commit message

### Post-Merge

- [ ] Monitor production metrics (if applicable)
- [ ] Close tracking issue
- [ ] Update project board
- [ ] Share learnings with team

---

## Rollback Plan

### If Integration Tests Fail

1. **Revert reader orchestration changes**
   - Git revert Phase 4 commits
   - Keep new extractors (can be tested independently)
   - Unblock main branch

2. **Debug in isolation**
   - Add more logging to extractors
   - Create minimal reproduction test
   - Fix and re-integrate

### If Performance Regresses

1. **Profile with `cargo flamegraph`**
   ```bash
   cargo flamegraph --bench note_extraction
   ```

2. **Identify hot path**
   - Event cloning?
   - Text accumulation?
   - Collection allocations?

3. **Optimize hot path**
   - Reduce clones (already using `CowStr`)
   - Pre-allocate collections with capacity
   - Use `SmallVec` for small collections

4. **If unfixable, revert and redesign**

### If Tests Are Flaky

1. **Add determinism checks**
   - Ensure no `HashMap` iteration order dependency
   - Ensure no timing dependencies
   - Add `run_pending_tasks()` for cache tests (if applicable)

2. **Run tests with `--test-threads=1`**
   ```bash
   cargo nextest run --test-threads=1
   ```

3. **Fix race conditions or non-determinism**

---

## Success Criteria

### Functional Requirements

- [ ] All existing tests pass
- [ ] All new tests pass
- [ ] No functionality regression (feature parity)
- [ ] Extracted entities match current implementation

### Non-Functional Requirements

- [ ] No performance regression (≤5% slower)
- [ ] Code size reduced (48% reduction: 2,186 → ~1,100 lines)
- [ ] Cyclomatic complexity reduced (≤10 per function)
- [ ] Test coverage maintained or improved
- [ ] Documentation complete and clear

### Architectural Requirements

- [ ] Zero cross-extractor coupling (context only)
- [ ] Each extractor testable in isolation
- [ ] Flat module structure (no subdirectories)
- [ ] Protocol in `reader.rs` with `pub(super)`
- [ ] `CowStr` used throughout (zero-copy)

---

## Risk Assessment

### High Risk

| Risk | Mitigation | Owner |
|------|------------|-------|
| Performance regression | Benchmark suite, profiling | Dev |
| Subtle behavior change | Integration tests, manual QA | Dev + QA |
| Incomplete error handling | Comprehensive error tests | Dev |

### Medium Risk

| Risk | Mitigation | Owner |
|------|------------|-------|
| Edge cases missed | Property-based tests | Dev |
| Flaky tests | Determinism checks | Dev |
| Documentation gaps | Review before merge | Team |

### Low Risk

| Risk | Mitigation | Owner |
|------|------------|-------|
| Merge conflicts | Small, focused PRs | Dev |
| Tooling issues | CI validates | CI |

---

## Notes for Implementer

### Code Style

- Use `pub(super)` for extraction protocol (not `pub(crate)`)
- Use `CowStr::Borrowed("")` for empty text (not `CowStr::Inlined`)
- Use `super::module::Type` for sibling imports (not `crate::note::adapter::...`)
- Use descriptive variable names (`list_ext` not `le`)
- Document public API with examples
- Document edge case handling in comments

### Common Pitfalls

1. **Forgetting to handle `finish()`**
   - Extractors must handle unclosed elements in `finish()`
   - Test with truncated markdown

2. **Not using context correctly**
   - Always check `ctx.inside_link` before scanning tags
   - Always check `ctx.inside_code_block` before scanning tags

3. **Clone confusion**
   - `CowStr::clone()` is cheap (clone reference, not data)
   - `Range::clone()` is cheap (Copy type)
   - `Event::clone()` is acceptable (mostly references)

4. **Error propagation**
   - Use `?` operator consistently
   - Implement `Into<NoteError>` for custom errors
   - Don't use `unwrap()` or `expect()` in production code

### Debugging Tips

1. **Enable tracing**
   ```rust
   #[tracing::instrument(skip(self, event), level = "trace")]
   fn process(...) -> Result<...> {
       tracing::trace!(?event, ?range, "processing event");
       // ...
   }
   ```

2. **Print event stream**
   ```bash
   RUST_LOG=trace cargo test test_name -- --nocapture
   ```

3. **Use `dbg!` macro**
   ```rust
   dbg!(&self.current_state);
   ```

---

---

## TDD Checklist (Must Follow)

### For Every Component

- [ ] **Write test FIRST** (Red phase)
- [ ] **Run test** - verify it fails for the right reason
- [ ] **Write minimal code** to pass test (Green phase)
- [ ] **Run test** - verify it passes
- [ ] **Refactor** code while keeping tests green
- [ ] **Run all tests** - verify nothing broke
- [ ] **Commit on green** with descriptive message
- [ ] **Never commit on red**

### Test Coverage Requirements

| Component | Unit Tests | Integration Tests | Property Tests |
|-----------|------------|-------------------|----------------|
| extract_list.rs | 10+ | Included in full parse | Yes |
| extract_link.rs | 10+ | Included in full parse | Yes |
| extract_heading.rs | 7+ | Included in full parse | No |
| extract_section.rs | 6+ | Included in full parse | No |
| extract_frontmatter.rs | 8+ | Included in full parse | Yes |
| extract_tag.rs | 10+ | Included in full parse | Yes |
| reader.rs (orchestration) | 5+ | Yes (full document) | Yes |

### TDD Anti-Patterns to Avoid

❌ **Writing implementation before test**
❌ **Writing multiple tests before any implementation**
❌ **Skipping refactor phase**
❌ **Committing failing tests**
❌ **Testing implementation details instead of behavior**
❌ **Making tests depend on each other**
❌ **Not running tests frequently**

### TDD Best Practices

✅ **One test at a time** - red, green, refactor, repeat
✅ **Test behavior, not implementation** - test outcomes, not internals
✅ **Keep tests independent** - any order should work
✅ **Use descriptive test names** - `extracts_checkbox_with_task_tag`
✅ **Arrange-Act-Assert pattern** - clear test structure
✅ **Fast tests** - unit tests should run in milliseconds
✅ **Commit frequently** - every green is a commit opportunity

---

## Abbreviated Remaining Phases

**All remaining phases follow the same TDD pattern as shown in Phase 1 and Phase 2.1**

### Phase 2.2-2.3: Link and Heading Extractors (Day 2 - 5 hours)

**TDD Cycle**: Red (write tests) → Green (implement) → Refactor (improve)

- Link Extractor: 10+ tests, 3 TDD cycles (wiki-links, anchors, embeds)
- Heading Extractor: 7+ tests, 2 TDD cycles (levels, text accumulation)

### Phase 3: Secondary Extractors (Day 2-3 - 6 hours)

**TDD Cycle**: Red → Green → Refactor for each extractor

- Section Extractor: 6+ tests, 2 TDD cycles
- Frontmatter Extractor: 8+ tests, 3 TDD cycles
- Tag Extractor: 10+ tests, 2 TDD cycles

### Phase 4: Reader Orchestration (Day 3 - 3 hours)

**TDD Approach**: Integration tests BEFORE wiring

1. **RED**: Write integration test with all extractors
2. **GREEN**: Wire extractors into reader.parse_str()
3. **REFACTOR**: Extract helper functions, improve readability

### Phase 5: Integration & Property Tests (Day 4 - 4 hours)

**TDD Approach**: Characterization testing

1. **RED**: Write test asserting current behavior
2. **GREEN**: Verify new implementation matches
3. **REFACTOR**: Optimize without changing behavior

### Phase 6: Cleanup (Day 4 - 2 hours)

- Delete old files after all tests pass
- Update documentation
- Final verification

---

## Estimated Timeline

- Day 1: Foundation + Core extractors (list, link, heading)
- Day 2: Secondary extractors + Reader orchestration
- Day 3: Integration tests + Cleanup

### Realistic (4 days)

- Day 1: Foundation + List extractor
- Day 2: Link + Heading + Section extractors
- Day 3: Frontmatter + Tag extractors + Reader orchestration
- Day 4: Integration tests + Performance tuning + Cleanup

### Pessimistic (5 days)

- Day 1: Foundation + List extractor + debugging
- Day 2: Link + Heading extractors
- Day 3: Section + Frontmatter + Tag extractors
- Day 4: Reader orchestration + debugging
- Day 5: Integration tests + Performance fixes + Cleanup

**Recommended**: Plan for 4 days, buffer 1 day for unforeseen issues.

---

## Appendix: Example Extractor Implementation

```rust
// note/adapter/extract_heading.rs

use pulldown_cmark::{CowStr, Event, Tag as CmarkTag, TagEnd};
use std::ops::Range;

use crate::note::{
    error::NoteError,
    structure::{Heading, HeadingLevel},
    position::SourceByteOffset,
};

use super::reader::{Extractor, ExtractionContext, ExtractionState};

/// Extracts headings from markdown.
pub struct HeadingExtractor {
    current: Option<HeadingBuilder>,
}

struct HeadingBuilder {
    level: HeadingLevel,
    text: String,
    position: SourceByteOffset,
}

impl HeadingExtractor {
    pub fn new() -> Self {
        Self { current: None }
    }
}

impl Extractor for HeadingExtractor {
    type Output = Heading;
    type Error = NoteError;

    fn process(
        &mut self,
        event: &Event<'_>,
        text: CowStr<'_>,
        range: Range<usize>,
        _ctx: &ExtractionContext,
    ) -> Result<ExtractionState<Heading>, NoteError> {
        match event {
            Event::Start(CmarkTag::Heading { level, .. }) => {
                let level = convert_heading_level(*level)?;
                let position = SourceByteOffset::try_from_usize(range.start)?;

                self.current = Some(HeadingBuilder {
                    level,
                    text: String::new(),
                    position,
                });

                Ok(ExtractionState::Continue)
            }

            Event::Text(_) | Event::Code(_) => {
                if let Some(builder) = self.current.as_mut() {
                    builder.text.push_str(&text);
                }
                Ok(ExtractionState::Continue)
            }

            Event::SoftBreak | Event::HardBreak => {
                if let Some(builder) = self.current.as_mut() {
                    builder.text.push(' ');
                }
                Ok(ExtractionState::Continue)
            }

            Event::End(TagEnd::Heading(_)) => {
                let Some(builder) = self.current.take() else {
                    return Ok(ExtractionState::Continue);
                };

                let heading = Heading::new(
                    builder.level,
                    builder.text,
                    builder.position,
                )?;

                Ok(ExtractionState::Emit(heading))
            }

            _ => Ok(ExtractionState::Continue),
        }
    }

    fn finish(self) -> Result<Vec<Heading>, NoteError> {
        // If there's an unclosed heading, that's malformed markdown
        // but we can handle it gracefully by emitting it
        if let Some(builder) = self.current {
            let heading = Heading::new(
                builder.level,
                builder.text,
                builder.position,
            )?;
            Ok(vec![heading])
        } else {
            Ok(Vec::new())
        }
    }
}

fn convert_heading_level(
    level: pulldown_cmark::HeadingLevel,
) -> Result<HeadingLevel, NoteError> {
    use pulldown_cmark::HeadingLevel as PLevel;

    let level_num = match level {
        PLevel::H1 => 1,
        PLevel::H2 => 2,
        PLevel::H3 => 3,
        PLevel::H4 => 4,
        PLevel::H5 => 5,
        PLevel::H6 => 6,
    };

    HeadingLevel::try_new(level_num)
}

#[cfg(test)]
mod tests {
    use super::*;
    use pulldown_cmark::HeadingLevel as PLevel;

    #[test]
    fn extracts_h1_heading() {
        let mut extractor = HeadingExtractor::new();
        let ctx = ExtractionContext::default();

        // Start heading
        let result = extractor.process(
            &Event::Start(CmarkTag::Heading {
                level: PLevel::H1,
                id: None,
                classes: vec![],
                attrs: vec![],
            }),
            CowStr::Borrowed(""),
            0..2,
            &ctx,
        ).unwrap();
        assert!(matches!(result, ExtractionState::Continue));

        // Add text
        let result = extractor.process(
            &Event::Text(CowStr::Borrowed("Title")),
            CowStr::Borrowed("Title"),
            2..7,
            &ctx,
        ).unwrap();
        assert!(matches!(result, ExtractionState::Continue));

        // End heading
        let result = extractor.process(
            &Event::End(TagEnd::Heading(PLevel::H1)),
            CowStr::Borrowed(""),
            7..8,
            &ctx,
        ).unwrap();

        match result {
            ExtractionState::Emit(heading) => {
                assert_eq!(heading.text(), "Title");
                assert_eq!(heading.level().as_u8(), 1);
            }
            _ => panic!("Expected Emit"),
        }
    }

    #[test]
    fn handles_unclosed_heading() {
        let mut extractor = HeadingExtractor::new();
        let ctx = ExtractionContext::default();

        // Start heading
        extractor.process(
            &Event::Start(CmarkTag::Heading {
                level: PLevel::H2,
                id: None,
                classes: vec![],
                attrs: vec![],
            }),
            CowStr::Borrowed(""),
            0..3,
            &ctx,
        ).unwrap();

        // Add text
        extractor.process(
            &Event::Text(CowStr::Borrowed("Unclosed")),
            CowStr::Borrowed("Unclosed"),
            3..11,
            &ctx,
        ).unwrap();

        // Finish without closing
        let result = extractor.finish().unwrap();
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].text(), "Unclosed");
    }
}
```

---

## Questions for Review

1. **Extraction order**: Should section extractor run after heading extractor to associate headings? (Current: they coordinate via heading emission)

2. **Error granularity**: Should each extractor have its own error type, or use `NoteError` directly? (Current: extractor-specific errors that convert to `NoteError`)

3. **Memory pressure**: Should we add capacity hints to vectors based on document size? (Current: use default capacity)

4. **Streaming API**: Should we add a streaming iterator variant for very large notes? (Current: collect all entities before returning)

---

---

## TDD Success Metrics

Track these metrics throughout implementation:

| Metric | Target | How to Measure |
|--------|--------|----------------|
| **Test-first %** | 100% | Count tests written before code |
| **Red-Green-Refactor cycles** | All | Count in commit messages |
| **Time in red** | <5 min/cycle | Time from test write to test pass |
| **Commits on green** | 100% | Never commit failing tests |
| **Test coverage** | >95% | `cargo tarpaulin` |
| **Refactor iterations** | ≥1 per feature | Code quality improvements |

### Daily TDD Retrospective

At end of each day, ask:

1. Did I write tests before code? (Yes/No for each feature)
2. How many red-green-refactor cycles completed?
3. What was my average time in red state?
4. Did I commit any failing tests?
5. What TDD practices helped most today?
6. What TDD challenges did I face?

---

## End of Plan

**Last Updated**: 2026-03-03
**Version**: 2.0 (TDD Framework)
**Status**: Ready for TDD Implementation
**Methodology**: Red → Green → Refactor

**Remember**: Test-first is non-negotiable. Every line of production code must be justified by a failing test.
