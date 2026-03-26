# Task Refactor: Typed Raw Values & Simplified Architecture

**Status**: Planning
**Created**: 2026-03-26
**Priority**: High

---

## Executive Summary

Refactor the task ingestion pipeline to use `Raw*` types as proper DTOs carrying typed values, eliminating redundant parsing in `task.rs` and restructuring `Task` to directly contain fields with a separate `TaskDates` component.

### Problem Statement

Currently, the note ingestion pipeline treats `Raw*` types as mere string carriers, forcing every downstream consumer to re-parse the same data:

1. **`RawInlineField` carries strings**: Scanner extracts text, but `RawInlineField` stores `value: Cow<'source, str>`, throwing away potential type information
2. **`InlineField::from_raw()` always creates `FieldValue::String`**: Forces task.rs to re-parse everything
3. **`task.rs` has massive parsing bloat**: `parse_metadata_value()`, `from_spec_value()`, `parse_heuristically()` all duplicate what `FieldValue`'s serde deserializer already does
4. **`TaskMetadata` duplicates `ListItem.fields()`**: Same data stored in two places, violating single source of truth
5. **Emoji mapping happens late**: Domain layer processes emoji keys instead of parser mapping them early

### Solution Overview

**Core Principle**: `Raw*` types are DTOs that isolate I/O and parsing concerns from domain logic.

**Key Changes**:
1. **Add `RawFieldValue` enum**: Typed values (String, Number, Date, DateTime, Time, Boolean) in raw layer
2. **Parser types fields**: During list item extraction, use `TaskConfigSpec` to type values and map emoji→keyword
3. **`InlineField` preserves types**: `from_raw()` converts `RawFieldValue` → `FieldValue` without re-parsing
4. **Restructure `Task`**:
   - Direct `fields: HashMap<InlineFieldKey, FieldValue>` (copied from ListItem)
   - Separate `dates: TaskDates` component (extracted from fields using spec)
5. **`TaskDateValue` mirrors `FrontmatterDateValue`**: Wraps `FieldValue` + stores `Arc<DateSpec>` for format tracking
6. **Delete parsing bloat**: Remove `TaskMetadata`, `parse_metadata_value()`, emoji helpers, etc.

---

## Architecture Changes

### Current Pipeline (Broken)

```
Scanner → RawInlineField (strings) → InlineField (strings) → Task re-parses
                                                            ↓
                                                      FieldValue typed
```

**Problems**:
- Multiple parsing passes over same data
- Type information lost and reconstructed
- Emoji keys carried to domain layer
- Field duplication between ListItem and TaskMetadata

### Proposed Pipeline (Fixed)

```
Scanner (text extraction)
    ↓
Parser (has TaskConfigSpec context)
    ↓ types values + maps emoji→keyword
RawInlineField (RawFieldValue typed)
    ↓
InlineField (FieldValue typed)
    ↓
ListItem (typed fields)
    ↓
Task::promote() (copies fields + extracts dates)
    ↓
Task { fields, dates }
```

**Benefits**:
- Single typing pass (parser)
- Single emoji mapping pass (parser)
- Zero duplication (fields from ListItem, dates extracted)
- No re-parsing in domain layer

---

## Detailed Design

### 1. RawFieldValue Enum (New Type)

**Location**: `lithos-core/src/note/raw.rs`

```rust
pub enum RawFieldValue<'source> {
    String(Cow<'source, str>),
    Number(f64),
    Date(NaiveDate),
    DateTime(DateTime<FixedOffset>),
    Time(NaiveTime),
    Boolean(bool),
}
```

**Factory Method**:
```rust
impl<'source> RawFieldValue<'source> {
    pub fn from_str_with_spec(
        text: &'source str,
        key: &str,
        spec: Option<&DateSpec>,
    ) -> Self;
}
```

**Logic**:
1. If `DateSpec` provided: Try spec format first (e.g., `%Y-%m-%d`)
2. Heuristic parsing: RFC3339 datetime, common date formats, boolean, number
3. Fallback: `String`

### 2. RawInlineField Update

**Before**:
```rust
pub struct RawInlineField<'source> {
    pub key: Cow<'source, str>,
    pub value: Cow<'source, str>,  // ← strings only
    pub range: SourceByteRange,
}
```

**After**:
```rust
pub struct RawInlineField<'source> {
    pub key: Cow<'source, str>,
    pub value: RawFieldValue<'source>,  // ← typed values
    pub range: SourceByteRange,
}
```

**Emoji Mapping Helper**:
```rust
impl<'source> RawInlineField<'source> {
    pub fn map_emoji_key(
        key: &str,
        task_spec: &TaskConfigSpec,
    ) -> Option<Box<str>>;
}
```

### 3. Parser Integration

**Location**: `lithos-core/src/note/parser.rs`

**Changes**:
- `filter_artifacts_by_range()` receives `Option<&TaskConfigSpec>`
- When processing `ScannedArtifact::InlineField`:
  1. Map emoji key → keyword using `RawInlineField::map_emoji_key()`
  2. Get `DateSpec` from spec if field is temporal
  3. Type value using `RawFieldValue::from_str_with_spec()`
- Thread spec through from `finalize_list_item()` (gets from `ListContext`)

**Call Sites**:
- Headings: pass `None` (no task context)
- Paragraphs: pass `None` (no task context)
- List items: pass `Some(&context.task_spec)` (has task context)

### 4. InlineField Type Preservation

**Location**: `lithos-core/src/note/inline_fields.rs`

**Change `from_raw()`**:
```rust
pub fn from_raw(raw: &RawInlineField<'_>) -> Self {
    let value = match &raw.value {
        RawFieldValue::String(s) => FieldValue::String(s.as_ref().into()),
        RawFieldValue::Number(n) => FieldValue::Number(*n),
        RawFieldValue::Date(d) => FieldValue::Date((*d).into()),
        RawFieldValue::DateTime(dt) => FieldValue::DateTime((*dt).into()),
        RawFieldValue::Time(t) => FieldValue::Time((*t).into()),
        RawFieldValue::Boolean(b) => FieldValue::Boolean(*b),
    };
    InlineField::new(raw.key.as_ref().into(), value, raw.range)
}
```

### 5. Task Restructuring

**Location**: `lithos-core/src/note/task.rs`

#### TaskDates Component (New)

```rust
pub struct TaskDates {
    created: Option<TaskDateValue>,
    due: Option<TaskDateValue>,
    reminder: Option<TaskDateValue>,
    completed: Option<TaskDateValue>,
    start: Option<TaskDateValue>,
    scheduled: Option<TaskDateValue>,
}
```

**Individual getters**: `created()`, `due()`, `reminder()`, `completed()`, `start()`, `scheduled()`

**Private helper** (moved from `TaskMetadata::match_date_spec()`):
```rust
fn match_date_spec(
    spec: &TaskConfigSpec,
    keyword: &str,
) -> Option<(TaskDateKind, Arc<DateSpec>)>;
```

#### TaskDateValue Update

**Before** (enum with Date/DateTime variants):
```rust
pub enum TaskDateValue {
    Date(NaiveDate),
    DateTime(DateTime<FixedOffset>),
}
```

**After** (wraps FieldValue like FrontmatterDateValue):
```rust
pub struct TaskDateValue {
    value: FieldValue,
    spec: Option<Arc<DateSpec>>,  // Format spec from config
}
```

**Methods**:
- `new(value: FieldValue, spec: Option<Arc<DateSpec>>) -> Result<Self, TaskError>`
- `as_field_value() -> &FieldValue`
- `spec() -> Option<&DateSpec>`
- `as_naive_date() -> Option<NaiveDate>` (extracts from DateTime if needed)
- `as_datetime() -> Option<DateTime<FixedOffset>>` (promotes Date→DateTime at 00:00:00)
- `from_field_value(value: &FieldValue, key: &str, spec: Option<Arc<DateSpec>>) -> Result<Self, TaskError>`

#### Task Struct Update

**Before**:
```rust
pub struct Task {
    id: TaskId,
    status: Box<str>,
    text: TaskText,
    range: SourceByteRange,
    tags: Box<[Tag]>,
    metadata: TaskMetadata,  // ← contains fields HashMap
}
```

**After**:
```rust
pub struct Task {
    id: TaskId,
    status: Box<str>,
    text: TaskText,
    range: SourceByteRange,
    tags: Box<[Tag]>,
    fields: HashMap<InlineFieldKey, FieldValue>,  // ← from ListItem
    dates: TaskDates,  // ← extracted from fields
}
```

**Accessors**:
- `fields() -> &HashMap<InlineFieldKey, FieldValue>`
- `dates() -> &TaskDates`

#### Task::promote() Rewrite

**New Logic**:
1. Validate item has checkbox marker
2. Copy all fields from `ListItem.fields()`
3. Extract date slots: For each field, check if keyword maps to temporal slot using `TaskDates::match_date_spec()`, create `TaskDateValue` from typed `FieldValue`
4. Compute clean text using range-based exclusion (no re-parsing)
5. Determine status from marker
6. Construct Task with fields + dates

**No re-parsing**: Values are already typed as `FieldValue::Date` or `FieldValue::DateTime`

### 6. Deletions (Bloat Removal)

**Delete entirely**:
- `TaskMetadata` struct
- `TaskMetadata::from_list_item()`
- `TaskMetadata::process_field()`
- `TaskMetadata::parse_metadata_value()`
- `TaskDateValue::parse_heuristically()` (old version)
- `TaskDateValue::from_spec_value()` (old version)
- Emoji-related helpers: `match_date_spec_by_emoji()`, `emoji_matches()`

**Keep**:
- `TaskDateKind` enum (still needed for slot identification)

### 7. Storage Layer Updates

**Location**: `lithos-core/src/note/storage.rs`

**Changes**:
- Update `task_date_index_keys()`: Receive `&TaskDates` instead of `&TaskMetadata`
- Update `task_date_query_keys()`: Receive `&TaskDates`
- Update `task_date_matches()`: Use `TaskDates` accessors
- Update all call sites: `task.metadata()` → `task.dates()`

---

## Implementation Phases

### Phase 1: Raw Layer (DTO Foundation)
**Goal**: Add typed values to raw layer without breaking existing code

**Files**: `raw.rs`

**Tasks**:
1. Add `RawFieldValue` enum with all variants
2. Add `RawFieldValue::from_str_with_spec()` factory
3. Add `RawFieldValue::into_owned()` for lifetime conversion
4. Update `RawInlineField` to use `RawFieldValue`
5. Add `RawInlineField::map_emoji_key()` helper
6. Update `RawInlineField::into_owned()`

**Risk**: Breaking changes to all consumers of `RawInlineField`

### Phase 2: Parser Integration (Typing Pass)
**Goal**: Type fields and map emoji keys during list item extraction

**Files**: `parser.rs`

**Tasks**:
1. Update `filter_artifacts_by_range()` signature: Add `task_spec: Option<&TaskConfigSpec>`
2. Update inline field extraction logic:
   - Map emoji key → keyword
   - Get DateSpec from spec
   - Type value using `RawFieldValue::from_str_with_spec()`
3. Thread spec through call sites:
   - Headings: pass `None`
   - Paragraphs: pass `None`
   - List items: get spec from `ListContext`, pass `Some(&spec)`

**Risk**: Parser logic complexity increase

### Phase 3: Domain Type Preservation (Propagation)
**Goal**: Preserve types through InlineField to ListItem

**Files**: `inline_fields.rs`

**Tasks**:
1. Update `InlineField::from_raw()`: Convert `RawFieldValue` → `FieldValue` directly
2. Verify ListItem now contains typed fields

**Risk**: Minimal - straightforward type conversion

### Phase 4: Task Restructuring (Core Domain)
**Goal**: Restructure Task with fields + dates, delete bloat

**Files**: `task.rs`

**Tasks**:
1. Add `TaskDates` struct with date slots
2. Add `TaskDates` individual getters
3. Move `match_date_spec()` to `TaskDates` impl
4. Update `TaskDateValue` to wrap `FieldValue` + store `Arc<DateSpec>`
5. Add `TaskDateValue` methods (new, accessors, from_field_value, etc.)
6. Update `Task` struct: Add `fields` and `dates`, remove `metadata`
7. Update `Task::try_new()` signature
8. Add `Task` accessors: `fields()`, `dates()`
9. Rewrite `Task::promote()`: Copy fields, extract dates, no re-parsing
10. Delete bloat: `TaskMetadata`, `parse_metadata_value()`, old parsers, emoji helpers

**Risk**: High - core domain restructuring, many test updates needed

### Phase 5: Storage Updates (Indexing)
**Goal**: Update storage layer to use TaskDates

**Files**: `storage.rs`

**Tasks**:
1. Update `task_date_index_keys()` signature and implementation
2. Update `task_date_query_keys()` signature and implementation
3. Update `task_date_matches()` to use `TaskDates` accessors
4. Update all call sites: `task.metadata()` → `task.dates()`

**Risk**: Moderate - indexing logic change, test updates needed

### Phase 6: Test Updates (Verification)
**Goal**: Update all tests to use new Task API

**Files**: `task.rs`, `aggregate.rs`, `storage.rs`, integration tests

**Tasks**:
1. Update `Task::try_new()` test calls: Pass `fields` and `dates` instead of `metadata`
2. Update assertions: `task.metadata().due()` → `task.dates().due()`
3. Add tests for typed field values
4. Add tests for emoji→keyword mapping
5. Add tests for Date→DateTime promotion
6. Update integration tests in aggregate.rs
7. Update storage tests

**Risk**: Time-consuming but straightforward

### Phase 7: Verification (Quality Gates)
**Goal**: Ensure all quality gates pass

**Commands**:
- `mise run test:unit:note`
- `mise run fmt`
- `mise run lint`
- `mise run verify`

**Tasks**:
1. Run unit tests for note module
2. Fix any test failures
3. Run formatter
4. Run clippy linting
5. Run full verification suite
6. Address any issues

**Risk**: Low if previous phases were done carefully

---

## Success Criteria

### Functional
- [ ] All tests pass (`mise run test:unit:note`)
- [ ] Task promotion works with typed fields
- [ ] Emoji keys correctly map to keywords
- [ ] Date slots correctly extract from fields
- [ ] Date→DateTime promotion works
- [ ] Storage indexing works with TaskDates

### Code Quality
- [ ] No clippy warnings (`mise run lint`)
- [ ] Code formatted (`mise run fmt`)
- [ ] Full verification passes (`mise run verify`)
- [ ] No `unwrap()`/`panic!` in production code
- [ ] All public APIs have rustdoc comments

### Architecture
- [ ] `Raw*` types are proper DTOs with typed values
- [ ] Parser handles typing and emoji mapping (single pass)
- [ ] Domain layer receives typed values (no re-parsing)
- [ ] Task is self-contained (fields + dates)
- [ ] No duplication between ListItem and Task

### Performance
- [ ] Single parsing pass per field (parser only)
- [ ] No string allocation for emoji→keyword mapping
- [ ] Zero-copy where possible (Cow types in raw layer)

---

## Risks & Mitigations

### Risk 1: Breaking Changes Cascade
**Impact**: High - Changes to `RawInlineField` affect many files
**Mitigation**:
- Implement phases sequentially
- Fix compilation errors before moving to next phase
- Use compiler as guide for all affected code

### Risk 2: Test Maintenance Burden
**Impact**: Medium - Many tests need updates
**Mitigation**:
- Update test fixtures once in Phase 4
- Use search/replace for common patterns
- Verify tests pass incrementally

### Risk 3: TemporalMapping Type Mismatch
**Impact**: Medium - Current type is `HashMap<Box<str>, (TemporalSlot, String, Option<char>)>`, we need `DateSpec`
**Mitigation**:
- Investigate actual `TemporalMapping` structure
- May need to update config layer first
- **TODO**: Verify `TemporalMapping` contains `Arc<DateSpec>` or needs refactoring

### Risk 4: Parser Complexity
**Impact**: Low - Adding spec awareness to parser
**Mitigation**:
- Keep logic isolated in `filter_artifacts_by_range()`
- Add helper methods for clarity
- Document threading of spec parameter

---

## Open Questions

### Q1: TemporalMapping Structure
**Question**: Does `TemporalMapping` currently store `String` format or `Arc<DateSpec>`?

**Current Type**:
```rust
pub type TemporalMapping = HashMap<Box<str>, (TemporalSlot, String, Option<char>)>;
```

**Issue**: We need `Arc<DateSpec>` but it's currently `String`. Need to either:
- **Option A**: Update config layer to store `Arc<DateSpec>` in `TemporalMapping`
- **Option B**: Construct `DateSpec` on-the-fly from `String` format
- **Option C**: Keep String format, only use for parsing hints

**Recommendation**: Option A - Update config layer for type consistency

**Impact**: May require Phase 0 (config layer update) before starting main refactor

### Q2: DateSpec Import in task.rs
**Question**: Does `task.rs` need to import `config::value::DateSpec`?

**Answer**: Yes, `TaskDateValue` needs `Arc<DateSpec>` field

**Action**: Add import, ensure no circular dependencies

### Q3: Validation Strategy
**Question**: Do we validate typed values against `FieldSpec` constraints?

**Answer** (from discussion): No validation needed. If typing succeeds, value is valid. If typing fails, falls back to String.

---

## Dependencies

### Internal
- `chrono` crate (already used for date/time parsing)
- `FieldValue` serde deserializer logic (reference implementation)
- `FrontmatterDateValue` pattern (template for `TaskDateValue`)

### External (Config Layer)
- **Potential blocker**: If `TemporalMapping` needs update to store `Arc<DateSpec>`, that's a prerequisite

---

## Rollback Plan

If implementation fails or introduces critical bugs:

1. **Revert commits**: Git reset to pre-refactor state
2. **Keep what works**: If raw layer changes are stable, keep them and pause domain restructuring
3. **Incremental rollback**: Phases are independent, can rollback phase-by-phase

---

## Follow-up Tasks

After successful implementation:

1. **Performance benchmarking**: Measure parsing speed improvement from single-pass typing
2. **Documentation**: Update architecture docs with new pipeline flow
3. **ADR**: Create ADR documenting this refactoring decision
4. **Similar refactors**: Apply same pattern to frontmatter parsing if not already done

---

## Timeline Estimate

- **Phase 1** (Raw Layer): 2-3 hours
- **Phase 2** (Parser): 2-3 hours
- **Phase 3** (InlineField): 1 hour
- **Phase 4** (Task): 4-5 hours (most complex)
- **Phase 5** (Storage): 2 hours
- **Phase 6** (Tests): 3-4 hours
- **Phase 7** (Verification): 1 hour

**Total**: 15-21 hours

**Note**: Estimate assumes no major blockers from config layer changes
