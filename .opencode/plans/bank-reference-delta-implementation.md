# Bank Reference Delta Implementation (Document C)

**Purpose**: Complete design for integrating bank property changes into property delta system

**Date**: 2026-04-29
**Status**: Design Specification

---

## Problem Statement

### Current Behavior

When property bank changes are detected (comparison stage, line 947):
```rust
let is_bank_affected = Self::bank_changed(&found_payload.view, property_bank_delta);
if is_bank_affected {
    // Mark as StaleBankReferences
    ComparedPayload::StaleBankReferences(StalePayload { ... })
}
```

**Gap**: We know the schema is affected but not **which specific properties** need re-expansion.

**Result**: Current implementation likely re-expands ALL properties (inefficient).

---

### Desired Behavior

When bank changes detected:
1. Compute **specific properties** affected (using `bank_references`)
2. Add affected properties to **property delta**
3. Use **delta expansion** strategy (not full re-expansion)

**Benefit**: Only re-expand affected refs, not all refs.

---

## Design Overview

### Three-Part Solution

1. **New Payload Type**: `StaleBankReferencesPayload` with `affected_refs` field
2. **Comparison Stage**: Compute `affected_refs` when bank changes detected
3. **Analysis Stage**: Integrate `affected_refs` into property delta

---

## Part 1: New Payload Type

### StaleBankReferencesPayload

**Location**: `schema_processor.rs` alongside other payload types

```rust
/// Payload for schemas with stale bank references.
///
/// Similar to `StalePayload` but includes the set of property names
/// that reference changed bank properties, enabling delta expansion.
#[derive(Debug, Clone, PartialEq)]
pub(crate) struct StaleBankReferencesPayload {
    /// Vault-relative schema path.
    path: RelativePath,

    /// File metadata.
    stats: FileStats,

    /// Raw file content (already read).
    content_str: Box<str>,

    /// Content hash.
    content_hash: Blake3Hash,

    /// Cached view from database.
    view: RawSchemaView,

    /// Property names that reference changed bank properties.
    ///
    /// These are schema property names (not bank property names).
    /// Computed from `view.bank_references()` intersected with
    /// `property_bank_delta`.
    affected_refs: HashSet<PropertyName>,
}
```

### Update ComparedPayload Enum

**Current**:
```rust
pub(crate) enum ComparedPayload {
    Fresh(FreshPayload),
    StaleTimestamps(FoundPayload),
    StaleBankReferences(StalePayload),  // OLD
    Stale(StalePayload),
}
```

**New**:
```rust
pub(crate) enum ComparedPayload {
    Fresh(FreshPayload),
    StaleTimestamps(FoundPayload),
    StaleBankReferences(StaleBankReferencesPayload),  // NEW
    Stale(StalePayload),
}
```

---

## Part 2: Comparison Stage Implementation

### Current Code (Lines 947-967)

```rust
TimestampBranch::Match(matched_payload) => {
    if is_bank_affected {
        let content_str = source
            .read_to_string(matched_payload.path.as_path())
            .map_err(SchemaIngestionError::from)
            .map_err(SchemaLoaderError::Ingestion)?;
        let content_hash = Blake3Hash::compute(content_str.as_bytes());
        ComparedPayload::StaleBankReferences(StalePayload {
            path: matched_payload.path,
            stats: matched_payload.stats,
            content_str: content_str.into(),
            content_hash,
            view: matched_payload.view,
        })
    } else {
        // ...
    }
}
```

### Updated Code

```rust
TimestampBranch::Match(matched_payload) => {
    let affected_refs = Self::compute_affected_refs(
        &matched_payload.view,
        property_bank_delta,
    );

    if !affected_refs.is_empty() {
        let content_str = source
            .read_to_string(matched_payload.path.as_path())
            .map_err(SchemaIngestionError::from)
            .map_err(SchemaLoaderError::Ingestion)?;
        let content_hash = Blake3Hash::compute(content_str.as_bytes());
        ComparedPayload::StaleBankReferences(StaleBankReferencesPayload {
            path: matched_payload.path,
            stats: matched_payload.stats,
            content_str: content_str.into(),
            content_hash,
            view: matched_payload.view,
            affected_refs,  // NEW
        })
    } else {
        // No affected refs - schema is actually fresh
        ComparedPayload::Fresh(FreshPayload {
            path: matched_payload.path,
            view: matched_payload.view,
        })
    }
}
```

### New Helper Method

```rust
impl SchemaProcessor<Comparison, Present> {
    /// Computes which schema properties reference changed bank properties.
    ///
    /// Returns a set of schema property names (not bank property names).
    fn compute_affected_refs(
        view: &RawSchemaView,
        property_bank_delta: Option<&HashSet<PropertyName>>,
    ) -> HashSet<PropertyName> {
        let Some(delta) = property_bank_delta else {
            return HashSet::new();
        };

        let Some(current_version) = view.current() else {
            return HashSet::new();
        };

        let bank_refs = current_version.bank_references();

        // bank_refs is HashMap<PropertyName, PropertyName>
        // where key = schema property name, value = bank property name
        bank_refs
            .iter()
            .filter_map(|(schema_prop_name, bank_prop_name)| {
                if delta.contains(bank_prop_name) {
                    Some(schema_prop_name.clone())
                } else {
                    None
                }
            })
            .collect()
    }
}
```

### Update Other Comparison Branches

**Timestamp mismatch, content match, bank affected** (lines 978-993):

```rust
ContentBranch::Match(content_payload) if is_bank_affected => {
    let affected_refs = Self::compute_affected_refs(
        &content_payload.view,
        property_bank_delta,
    );

    let content_hash = Blake3Hash::compute(content_payload.content_str.as_bytes());
    ComparedPayload::StaleBankReferences(StaleBankReferencesPayload {
        path: content_payload.path,
        stats: content_payload.stats,
        content_str: content_payload.content_str,
        content_hash,
        view: content_payload.view,
        affected_refs,  // NEW
    })
}
```

---

## Part 3: Analysis Stage Integration

### Current Analysis for StaleBankReferences (Lines 1887-1906)

```rust
if node_status == NodeStatus::StaleBankReferences {
    let mut view = payload.view;
    let version = Self::build_version(&payload.raw, payload.content_hash)?;
    view.add_version(version);
    let rebuild = RebuildNodePayload {
        path: payload.path,
        stats: payload.stats,
        content_hash: payload.content_hash,
        raw: payload.raw,
        view,
        excludes_delta: None,
        property_delta: None,  // NO DELTA - this is the problem!
    };
    rebuild_ids.push(id);
    (NodeStatus::StaleBankReferences, AnalysisBranch::Rebuild(rebuild))
}
```

### Updated Analysis

```rust
if node_status == NodeStatus::StaleBankReferences {
    let mut view = payload.view;
    let version = Self::build_version(&payload.raw, payload.content_hash)?;
    view.add_version(version);

    // NEW: Create property delta from affected refs
    let property_delta = Self::create_bank_affected_delta(
        &payload.raw,
        &payload.affected_refs,
    );

    let rebuild = RebuildNodePayload {
        path: payload.path,
        stats: payload.stats,
        content_hash: payload.content_hash,
        raw: payload.raw,
        view,
        excludes_delta: None,
        property_delta: Some(property_delta),  // NEW
    };
    rebuild_ids.push(id);
    (NodeStatus::StaleBankReferences, AnalysisBranch::Rebuild(rebuild))
}
```

### New Helper Method for Bank Delta

```rust
impl SchemaProcessor<PropertyAnalysis, Graphed> {
    /// Creates a property delta containing only bank-affected refs.
    ///
    /// Marks all affected refs as upserts (they need re-expansion).
    fn create_bank_affected_delta(
        raw: &RawSchema,
        affected_refs: &HashSet<PropertyName>,
    ) -> SchemaPropertyDelta {
        let mut upserts = SchemaPropertyUpserts::default();

        // Extract ref entries for affected properties
        for (name, entry) in raw.properties().iter() {
            if affected_refs.contains(name) {
                if let RawProperty::Ref(ref_entry) = entry {
                    upserts.refs.insert(name.clone(), ref_entry.clone());
                }
            }
        }

        // No removals - bank changes don't remove properties
        SchemaPropertyDelta::new(upserts, Vec::new())
    }
}
```

---

## Part 4: Construction Stage Integration

### No Changes Needed!

With property delta now populated for `StaleBankReferences`, the existing construction logic already handles it correctly:

**Path in decision tree**: C-4 (Property delta with valid cache)

**Strategy**: DELTA_EXPAND_BASE

**Code path** (lines 2468-2516):
```rust
(ExtendsChangeKind::Unchanged, Some(delta)) => {
    // Fetch schema + cached base properties
    let schema = fetched_by_id.get(&id)...;
    let expanded = expanded_by_id.get(&id)?;

    // Apply delta (includes bank-affected refs)
    let mut properties = schema.properties().clone();
    for (name, prop) in expanded {
        if delta.contains_upsert(name) {  // Bank-affected refs are in upserts
            properties.insert(name.clone(), prop.clone());
        }
    }

    // ... create schema
}
```

**Result**: Only affected refs are re-expanded, not all refs!

---

## Part 5: Parse Stage Updates

### Current Code (Lines 1271-1308)

Needs update to handle new `StaleBankReferencesPayload`:

```rust
PipelinePayload::Compared(ComparedPayload::StaleBankReferences(payload)) => {
    // Parse raw schema
    let schema_name = source.filename(payload.path.as_path())...;
    let stats_for_raw = payload.stats;
    let raw = FsReader::parse_structured_from_str::<RawSchema>(
        payload.path.as_path(),
        &payload.content_str,
    )...
    .with_file_stats(stats_for_raw)
    .with_name(schema_name);

    ProcessorNode::new(
        NodeStatus::StaleBankReferences,
        relation,
        PipelinePayload::FileParsed(
            FileParsedBranch::StaleParsed(StaleParsedPayload {
                path: payload.path,
                stats: payload.stats,
                content_hash: payload.content_hash,
                raw,
                view: payload.view,
                affected_refs: payload.affected_refs,  // NEW: Carry forward
            }),
        ),
    )
}
```

### Update StaleParsedPayload

```rust
#[derive(Debug, Clone, PartialEq)]
pub(crate) struct StaleParsedPayload {
    path: RelativePath,
    stats: FileStats,
    content_hash: Blake3Hash,
    raw: RawSchema,
    view: RawSchemaView,
    affected_refs: Option<HashSet<PropertyName>>,  // NEW (optional - only for bank refs)
}
```

**Alternative**: Keep `affected_refs` in the status tracking, not payload (cleaner)

---

## Part 6: Data Flow Diagram

```
┌─────────────────────────────────────────────────────────────┐
│ Comparison Stage                                            │
│                                                             │
│ 1. Check: is_bank_affected(&view, property_bank_delta)     │
│    ↓ YES                                                    │
│ 2. Compute: affected_refs = compute_affected_refs(...)     │
│    Example: bank delta = {color, size}                     │
│             bank_refs = {bg_color → color, width → size}   │
│             affected_refs = {bg_color, width}              │
│    ↓                                                        │
│ 3. Create: StaleBankReferencesPayload with affected_refs   │
└─────────────────────────────────────────────────────────────┘
                         ↓
┌─────────────────────────────────────────────────────────────┐
│ Parse Stage                                                 │
│                                                             │
│ 1. Parse raw schema from file                              │
│ 2. Carry forward affected_refs in StaleParsedPayload       │
└─────────────────────────────────────────────────────────────┘
                         ↓
┌─────────────────────────────────────────────────────────────┐
│ Graphing Stage                                              │
│                                                             │
│ 1. Preserve status (StaleBankReferences)                   │
│ 2. Carry forward affected_refs                             │
└─────────────────────────────────────────────────────────────┘
                         ↓
┌─────────────────────────────────────────────────────────────┐
│ Analysis Stage                                              │
│                                                             │
│ 1. Create property delta from affected_refs                │
│    property_delta.upserts.refs = {bg_color, width}         │
│ 2. Store in RebuildNodePayload                             │
└─────────────────────────────────────────────────────────────┘
                         ↓
┌─────────────────────────────────────────────────────────────┐
│ Construction Stage                                          │
│                                                             │
│ 1. Fetch cached base properties                            │
│ 2. Expand ONLY affected refs: {bg_color, width}            │
│ 3. Merge: cached_base - removals + {bg_color, width}       │
│ 4. Save updated base properties                            │
│ 5. Merge with parent properties → final schema             │
└─────────────────────────────────────────────────────────────┘
```

---

## Part 7: Edge Cases

### Edge Case 1: Bank Property Deleted

**Scenario**: Bank property `color` deleted, schema has `bg_color → color`

**Behavior**:
1. Comparison: `affected_refs = {bg_color}`
2. Analysis: `property_delta.upserts.refs = {bg_color}`
3. Construction: Expand `bg_color` ref → **expansion fails** (bank property missing)

**Question**: How to handle missing bank property?

**Options**:
- **A**: Error (strict) - schema references non-existent bank property
- **B**: Remove property (lenient) - treat as if property was removed from schema
- **C**: Keep old value (cache) - schema keeps old expanded value until manually fixed

**Recommendation**: Option A (error) - force user to fix schema

---

### Edge Case 2: Bank Property Added (Not Changed)

**Scenario**: Bank property `color` added, schema doesn't reference it yet

**Behavior**:
1. Comparison: `affected_refs = {}` (schema doesn't reference new property)
2. Result: Schema stays `Fresh`

**Expected**: Correct - new bank properties don't affect existing schemas

---

### Edge Case 3: Bank Property Changed, Schema Has Local Override

**Scenario**:
- Bank property `color` changed
- Schema has inline `color` property (overrides bank)
- Schema also has `bg_color → color` (ref to bank)

**Behavior**:
1. Comparison: `affected_refs = {bg_color}` (only the ref, not the inline)
2. Analysis: `property_delta.upserts.refs = {bg_color}`
3. Construction: Re-expand `bg_color`, keep inline `color` unchanged

**Expected**: Correct - inline properties are independent of bank

---

### Edge Case 4: Multiple Schemas Reference Same Bank Property

**Scenario**: 10 schemas reference bank property `color` (which changed)

**Behavior**:
1. Comparison: Each schema gets `StaleBankReferencesPayload` with `affected_refs = {their_prop}`
2. Analysis: Each schema gets property delta
3. Construction: Each schema delta-expands only affected props

**Optimization**: Batch expansion could cache bank property expansion

---

## Part 8: Testing Strategy

### Unit Tests

#### Test 1: compute_affected_refs

```rust
#[test]
fn compute_affected_refs_returns_empty_when_no_bank_delta() {
    let view = make_view_with_refs(hashmap! {
        "bg_color" => "color",
    });

    let result = SchemaProcessor::compute_affected_refs(&view, None);

    assert!(result.is_empty());
}

#[test]
fn compute_affected_refs_returns_schema_props_for_changed_bank_props() {
    let view = make_view_with_refs(hashmap! {
        "bg_color" => "color",
        "fg_color" => "color",
        "width" => "size",
    });

    let bank_delta = hashset!{"color".into()};

    let result = SchemaProcessor::compute_affected_refs(&view, Some(&bank_delta));

    assert_eq!(result.len(), 2);
    assert!(result.contains(&"bg_color".into()));
    assert!(result.contains(&"fg_color".into()));
    assert!(!result.contains(&"width".into()));
}
```

#### Test 2: create_bank_affected_delta

```rust
#[test]
fn create_bank_affected_delta_includes_only_affected_refs() {
    let raw = make_raw_schema(hashmap! {
        "bg_color" => RawProperty::Ref("color"),
        "width" => RawProperty::Ref("size"),
        "title" => RawProperty::Inline(string_type()),
    });

    let affected = hashset!{"bg_color".into()};

    let delta = SchemaProcessor::create_bank_affected_delta(&raw, &affected);

    assert_eq!(delta.upserts().refs().len(), 1);
    assert!(delta.upserts().refs().contains_key(&"bg_color".into()));
    assert!(delta.removals().is_empty());
}
```

---

### Integration Tests

#### Test 3: End-to-End Bank Change Flow

```rust
#[test]
fn bank_property_change_triggers_delta_expansion() {
    let mut processor = setup_processor_with_schema(/* schema with refs */);

    // Change bank property
    let bank_delta = hashset!{"color".into()};

    // Run pipeline
    let result = processor
        .compare(source, Some(&bank_delta))?
        .parse(source)?
        .build_graph()?
        .analyze_properties(source, Some(&bank_delta))?
        .construct_schemas(repo, property_bank)?;

    // Verify delta expansion was used (not full expansion)
    assert!(metrics.delta_expansions > 0);
    assert_eq!(metrics.full_expansions, 0);
}
```

---

## Part 9: Performance Implications

### Before (Current Behavior)

**Scenario**: Property bank has 100 properties, schema references 10 of them, 1 bank property changes

**Construction**:
1. Detect bank change → mark `StaleBankReferences`
2. Full expand ALL 10 refs (even though only 1 changed)

**Cost**: Expand 10 properties

---

### After (With This Implementation)

**Scenario**: Same as above

**Construction**:
1. Detect bank change → compute `affected_refs = {changed_prop}`
2. Delta expand ONLY 1 affected ref
3. Merge with cached base (other 9 refs unchanged)

**Cost**: Expand 1 property

**Savings**: 90% reduction in expansion overhead

---

### Worst Case

**Scenario**: All 10 refs changed

**Before**: Expand 10 properties
**After**: Expand 10 properties + delta merge overhead

**Overhead**: ~5-10% slower than before (negligible)

**Conclusion**: No worse than before, usually much better

---

## Part 10: Migration and Rollout

### Phase 1: Add Payload Types

**Changes**:
- Add `StaleBankReferencesPayload` struct
- Update `ComparedPayload` enum
- Update `StaleParsedPayload` (add optional `affected_refs`)

**Testing**: Compile and type-check

---

### Phase 2: Implement Comparison Logic

**Changes**:
- Add `compute_affected_refs` method
- Update comparison stage to create `StaleBankReferencesPayload`

**Testing**: Unit tests for `compute_affected_refs`

---

### Phase 3: Implement Analysis Integration

**Changes**:
- Add `create_bank_affected_delta` method
- Update analysis stage to create property delta

**Testing**: Unit tests for `create_bank_affected_delta`

---

### Phase 4: Integration Testing

**Changes**: None (construction already supports delta)

**Testing**: End-to-end integration tests

---

### Phase 5: Monitoring and Validation

**Metrics**:
- Track bank-affected schemas count
- Track delta vs full expansion ratio
- Track performance improvement

**Expected**: 50-90% reduction in expansion overhead for bank changes

---

## Summary

### Key Changes

| Component         | Change                                       | Impact       |
|-------------------|----------------------------------------------|--------------|
| Payload Types     | Add `StaleBankReferencesPayload`             | Type safety  |
| Comparison Stage  | Compute `affected_refs`                      | Optimization |
| Analysis Stage    | Create property delta from `affected_refs`   | Optimization |
| Construction      | Use delta expansion (already works)          | No change    |

### Benefits

1. **Performance**: Only re-expand affected refs (not all refs)
2. **Accuracy**: Precise tracking of what changed
3. **Consistency**: Bank changes treated same as regular property changes
4. **Simplicity**: Construction logic unchanged (reuses delta path)

### Implementation Effort

- **Phase 1** (Types): ~1 hour
- **Phase 2** (Comparison): ~2 hours
- **Phase 3** (Analysis): ~1 hour
- **Phase 4** (Testing): ~2 hours
- **Phase 5** (Monitoring): ~1 hour

**Total**: ~7 hours

### Risk Assessment

- **Low Risk**: Additive change, doesn't break existing functionality
- **Testable**: Clear unit test boundaries
- **Rollback**: Can temporarily return to full expansion if issues found

---

**END OF DOCUMENT C**
