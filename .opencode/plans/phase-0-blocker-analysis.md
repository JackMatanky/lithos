# Phase 0 Blocker Analysis: TemporalMapping Structure

**Date**: 2026-03-26
**Status**: Requires Decision
**Blocker For**: Main refactoring plan

---

## Problem Statement

The current `TemporalMapping` type stores temporal field configuration as:

```rust
pub type TemporalMapping = HashMap<Box<str>, (TemporalSlot, String, Option<char>)>;
//                                               ^^^^^^^^^  ^^^^^^
//                                               slot enum   format string
```

However, our refactoring plan requires access to `Arc<DateSpec>` because:

1. **`TaskDateValue` needs `Arc<DateSpec>`**: The design mirrors `FrontmatterDateValue` but adds spec tracking:
   ```rust
   pub struct TaskDateValue {
       value: FieldValue,
       spec: Option<Arc<DateSpec>>,  // ← Need this!
   }
   ```

2. **`RawFieldValue::from_str_with_spec()` needs `DateSpec`**: For spec-aware parsing:
   ```rust
   pub fn from_str_with_spec(
       text: &'source str,
       key: &str,
       spec: Option<&DateSpec>,  // ← Need this!
   ) -> Self;
   ```

## Current Config Structure

The `Task` config struct (high-level) stores individual `DateSpec` fields:

```rust
pub struct Task {
    enabled: bool,
    // ...
    due: Option<DateSpec>,
    created: Option<DateSpec>,
    reminder: Option<DateSpec>,
    completed: Option<DateSpec>,
    start: Option<DateSpec>,
    scheduled: Option<DateSpec>,
    // ...
}
```

But `TaskConfigSpec` (lightweight spec) stores `TemporalMapping` with just format strings:

```rust
pub struct TaskConfigSpec {
    // ...
    pub temporal_specs: TemporalMapping,  // String format only
    // ...
}
```

## Root Cause

There's a **conversion** from `Task` → `TaskConfigSpec` where full `DateSpec` objects are flattened into `(TemporalSlot, String, Option<char>)` tuples, losing the `DateSpec` reference.

## Solution Options

### Option A: Update TemporalMapping to Store Arc<DateSpec> (RECOMMENDED)

**Change**:
```rust
// Before
pub type TemporalMapping = HashMap<Box<str>, (TemporalSlot, String, Option<char>)>;

// After
pub type TemporalMapping = HashMap<Box<str>, (TemporalSlot, Arc<DateSpec>, Option<char>)>;
//                                                          ^^^^^^^^^^^^^^^
```

**Pros**:
- Consistent with design (TaskDateValue stores Arc<DateSpec>)
- No on-the-fly DateSpec construction
- Single source of truth for format + keyword + emoji

**Cons**:
- Requires updating config layer conversion logic
- All consumers of TemporalMapping need updates
- Adds Arc reference (minimal overhead)

**Impact**: Medium - requires config layer changes first

### Option B: Construct DateSpec On-The-Fly

**Implementation**:
```rust
// When needed, create DateSpec from TemporalMapping tuple
let (slot, format_str, emoji) = temporal_specs.get(keyword)?;
let date_spec = Arc::new(DateSpec::from_format(format_str, emoji));
```

**Pros**:
- No config layer changes needed
- Simpler immediate path

**Cons**:
- Allocates DateSpec multiple times (inefficient)
- No `DateSpec::from_format()` constructor exists (need to add)
- `DateSpec` has more than just format (has `FieldName` keyword too)

**Impact**: Low upfront, but creates technical debt

### Option C: Store Separate DateSpec Mapping

**Change**:
```rust
pub struct TaskConfigSpec {
    // Keep existing
    pub temporal_specs: TemporalMapping,
    // Add new mapping
    pub date_specs: HashMap<Box<str>, Arc<DateSpec>>,  // NEW
}
```

**Pros**:
- No breaking changes to TemporalMapping
- Parallel data structure

**Cons**:
- **Duplication**: Same keyword→format mapping in two places
- **Sync risk**: Can get out of sync
- **Verbose**: Requires managing two maps

**Impact**: Low upfront, high maintenance cost

## Recommendation

**Option A** is the cleanest solution because:

1. **Single source of truth**: DateSpec contains keyword, format, and emoji
2. **No duplication**: TemporalMapping stores the full spec
3. **Consistent design**: Matches TaskDateValue's Arc<DateSpec> field
4. **Efficient**: DateSpec allocated once during config parsing, shared via Arc

## Required Changes for Option A

### Config Layer (lithos-core/src/config/task.rs)

1. **Update TemporalMapping type**:
   ```rust
   pub type TemporalMapping = HashMap<Box<str>, (TemporalSlot, Arc<DateSpec>, Option<char>)>;
   ```

2. **Update Task → TaskConfigSpec conversion** (wherever it happens):
   ```rust
   // Before
   temporal_specs.insert(
       keyword.clone(),
       (slot, date_spec.format().to_string(), date_spec.emoji())
   );

   // After
   temporal_specs.insert(
       keyword.clone(),
       (slot, Arc::new(date_spec), date_spec.emoji())
   );
   ```

3. **Update all TemporalMapping consumers** to extract Arc<DateSpec> from tuple

### Affected Code Locations

Search for:
- `temporal_specs.get(`
- `temporal_specs.insert(`
- Pattern matching on TemporalMapping tuples

Estimate: 10-15 locations across config and task modules

## Decision Required

**Question for User**: Should we proceed with Option A (update TemporalMapping), or prefer Option B/C?

**My Recommendation**: Option A, with Phase 0 dedicated to config layer update.

## Updated Phase 0 Tasks

If Option A is chosen:

```
Phase 0.1: Update TemporalMapping type definition in config/task.rs
Phase 0.2: Update Task → TaskConfigSpec conversion logic
Phase 0.3: Update all TemporalMapping usage sites
Phase 0.4: Update tests that construct TaskConfigSpec
Phase 0.5: Verify config layer builds and tests pass
```

**Estimate**: 2-3 hours

**Total Project Estimate** (with Phase 0): 17-24 hours (was 15-21 hours)

---

## Next Steps

1. **User Decision**: Choose Option A, B, or C
2. **If Option A**: Add Phase 0 tasks to todo list
3. **If Option B or C**: Update main plan to reflect workaround
4. **Proceed with implementation**: Once blocker is resolved
