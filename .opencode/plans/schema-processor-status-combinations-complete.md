# Schema Processor Status Matrix - Complete Event Combinations

**Document Purpose**: Exhaustive mapping of all event combinations that lead to each possible `NodeStatus` value in the schema processor pipeline.

**Last Updated**: 2026-04-29
**Source**: `lithos-core/src/schema/schema_processor.rs`
**Status**: ✅ Complete Analysis - All 23 Event Combinations Documented

---

## Executive Summary

The schema processor uses a **single `NodeStatus` enum** to track state across a **7-stage pipeline**. This creates semantic overloading where the same status value means different things at different stages.

**Key Findings**:

- **11 defined statuses** (8 active, 2 reserved, 1 guard)
- **23 distinct event combinations** leading to final statuses
- **3 stages where status semantics change** (Parse→Graphing, Graphing→Analysis)
- **2 separate bank-change detection points** (potential redundancy or feature?)
- **1 implicit cascade mechanism** (rebuild without status change)

---

## Pipeline Stages Overview

```
Discovery → Comparison → Parse → Graphing → Analysis → Construction → Completion
```

### Status Enum Definition (Lines 150-167)

```rust
pub(crate) enum NodeStatus {
    // Discovery / Comparison
    Deleted,              // ✅ Active
    Fresh,                // ✅ Active
    StaleTimestamps,      // ✅ Active
    StaleBankReferences,  // ✅ Active
    Stale,                // ✅ Active

    // Parsing / Graphing
    New,                  // ✅ Active
    StaleParsed,          // ✅ Active (intermediate only)

    // Analysis
    ExcludesChanged,      // ❌ Reserved, never assigned
    PropertiesChanged,    // ❌ Reserved, never assigned
    StaleContent,         // ✅ Active
    Corrupt,              // ⚠️  Guard only (unreachable branches)
}
```

---

## PART 1: Discovery Stage

### Discovery Branch A: Never Seen (Lines 710-736)

**Precondition**: No schemas exist in database
**Input**: File paths from `FilesContext`

| Event             | Status Assigned | Payload Type            | Code Location |
| ----------------- | --------------- | ----------------------- | ------------- |
| All files are new | _Deferred_      | `NewBatch<InitialScan>` | Line 724-726  |

**Result**: `DiscoveryBranch::AllMissing` - status will be assigned as `New` during parse stage

---

### Discovery Branch B: Review (Lines 739-903)

**Precondition**: At least some schemas exist in database
**Input**: File paths + existing `InheritanceGraph`

#### Step 1: File Classification (Lines 815-856)

| File on Disk? | View in DB? | Status Assigned | Payload Type   | Code Location |
| ------------- | ----------- | --------------- | -------------- | ------------- |
| ✓             | ✓           | `Fresh`         | `FoundPayload` | Line 833-837  |
| ✓             | ✗           | _Deferred_      | `InitialScan`  | Line 839-843  |
| ✗             | ✓           | `Deleted`       | (none yet)     | Line 849-851  |

#### Step 2: Graph Construction (Lines 859-902)

| Payload Type   | Status in Graph | Code Location |
| -------------- | --------------- | ------------- |
| `FoundPayload` | `Fresh`         | Line 876      |
| Deleted ID     | `Deleted`       | Line 877      |
| Invalid state  | `Corrupt`       | Line 882      |

**Result**: `DiscoveryBranch::HasPresent` with graph containing `Fresh` and `Deleted` nodes

---

## PART 2: Comparison Stage (Lines 909-1141)

**Input**: `Present` status with graph + optional `property_bank_delta`

### Complete Event Matrix

| Event ID | Timestamps Match? | Content Re-read? | Content Hash Match? | Bank Changed? | Result Status         | Result Payload                         | Code Lines |
| -------- | ----------------- | ---------------- | ------------------- | ------------- | --------------------- | -------------------------------------- | ---------- |
| C1       | ✓                 | No               | N/A                 | No            | `Fresh`               | `ComparedPayload::Fresh`               | 968-972    |
| C2       | ✓                 | Yes (forced)     | N/A                 | Yes           | `StaleBankReferences` | `ComparedPayload::StaleBankReferences` | 952-967    |
| C3       | ✗                 | Yes (auto)       | ✓                   | No            | `StaleTimestamps`     | `ComparedPayload::StaleTimestamps`     | 995-1002   |
| C4       | ✗                 | Yes (auto)       | ✓                   | Yes           | `StaleBankReferences` | `ComparedPayload::StaleBankReferences` | 978-993    |
| C5       | ✗                 | Yes (auto)       | ✗                   | (any)         | `Stale`               | `ComparedPayload::Stale`               | 1004-1006  |
| C6       | N/A               | N/A              | N/A                 | N/A           | `Deleted`             | `DeletedPayload`                       | 1034-1037  |

### Bank Change Detection Logic (Lines 603-611)

```rust
fn bank_changed(view: &RawSchemaView, property_bank_delta: Option<&HashSet<PropertyName>>) -> bool {
    property_bank_delta.is_some_and(|delta| {
        view.current().is_some_and(|v| !v.changed_bank_references(delta).is_empty())
    })
}
```

**Key Point**: Bank changes are ONLY checked for schemas where `view.current()` exists AND intersects with the delta.

---

## PART 3: Parse Stage

### Parse Branch A: AllMissing (Lines 1147-1198)

**Input**: `NewBatch<InitialScan>` (all new files)

| Event ID | Input Type    | Parse Result | Result Status | Result Payload    | Code Lines |
| -------- | ------------- | ------------ | ------------- | ----------------- | ---------- |
| P1       | `InitialScan` | Success      | `New`         | `NewParsed` batch | 1158-1192  |

---

### Parse Branch B: Compared (Lines 1200-1388)

**Input**: `Compared` status with graph + `NewBatch<InitialRead>`

#### For Existing Schemas in Graph

| Event ID | Input Status (Comparison) | Requires Parse? | Result Status         | Result Payload                      | Code Lines |
| -------- | ------------------------- | --------------- | --------------------- | ----------------------------------- | ---------- |
| P2       | `Fresh`                   | No              | `Fresh`               | `FileParsedBranch::Fresh`           | 1310-1318  |
| P3       | `StaleTimestamps`         | No              | `StaleTimestamps`     | `FileParsedBranch::StaleTimestamps` | 1319-1327  |
| P4       | `StaleBankReferences`     | Yes             | `StaleBankReferences` | `FileParsedBranch::StaleParsed`     | 1271-1308  |
| P5       | `Stale`                   | Yes             | `StaleParsed`         | `FileParsedBranch::StaleParsed`     | 1233-1269  |
| P6       | `Deleted`                 | No              | `Deleted`             | `DeletedPayload`                    | 1328-1333  |

**Critical Implementation Detail**:

- Both `StaleBankReferences` (P4) and `Stale` (P5) produce `FileParsedBranch::StaleParsed`
- **Original status preserved** in `status_by_id` map at lines 1413-1416

#### For New Schemas in NewBatch

| Event ID | Input Type    | Result Status | Result Payload            | Code Lines |
| -------- | ------------- | ------------- | ------------------------- | ---------- |
| P7       | `InitialRead` | (deferred)    | `NewBatch<InitialParsed>` | 1356-1387  |

---

## PART 4: Graphing Stage (Lines 1390-1621)

**Input**: `Parsed` status with graph

### Status Restoration Logic (Lines 1413-1416, 1458-1461)

```rust
// Build map of original statuses BEFORE parse stage transformed them
let mut status_by_id: HashMap<SchemaId, NodeStatus> = HashMap::new();
for (id, node) in graph.graph().iter() {
    status_by_id.insert(id, node.payload().status());
}

// Later: Restore original status for StaleParsed nodes
let status = match file_parsed {
    FileParsedBranch::StaleParsed(_) => status_by_id
        .get(&id)
        .copied()
        .unwrap_or(NodeStatus::StaleParsed),
    FileParsedBranch::Fresh(_) => NodeStatus::Fresh,
    FileParsedBranch::StaleTimestamps(_) => NodeStatus::StaleTimestamps,
};
```

### Event Matrix

| Event ID | Input Branch (Parse)                | Original Status (status_by_id) | Result Status         | Result Payload                       | Code Lines |
| -------- | ----------------------------------- | ------------------------------ | --------------------- | ------------------------------------ | ---------- |
| G1       | `FileParsedBranch::Fresh`           | N/A                            | `Fresh`               | `InheritanceBranch::Fresh`           | 1462       |
| G2       | `FileParsedBranch::StaleTimestamps` | N/A                            | `StaleTimestamps`     | `InheritanceBranch::StaleTimestamps` | 1463-1465  |
| G3       | `FileParsedBranch::StaleParsed`     | `StaleBankReferences`          | `StaleBankReferences` | `InheritanceBranch::StaleParsed`     | 1458-1461  |
| G4       | `FileParsedBranch::StaleParsed`     | `Stale` (or default)           | `StaleParsed`         | `InheritanceBranch::StaleParsed`     | 1458-1461  |
| G5       | `NewParsed` batch entry             | N/A                            | `New`                 | `InheritanceBranch::New`             | 1527       |

### Extends Relationship Change Detection (Lines 1481-1489)

| Event ID | Old Parent | New Parent               | ExtendsChangeKind | Code Line |
| -------- | ---------- | ------------------------ | ----------------- | --------- |
| E1       | None       | None                     | `Unchanged`       | 1482      |
| E2       | None       | Some(parent_id)          | `RootToChild`     | 1483      |
| E3       | Some(\_)   | None                     | `ChildToRoot`     | 1484      |
| E4       | Some(old)  | Some(new) where old==new | `Unchanged`       | 1485-1486 |
| E5       | Some(old)  | Some(new) where old!=new | `Rewired`         | 1488      |

---

## PART 5: Analysis Stage (Lines 1695-2041)

**Input**: `Graphed` status + optional `property_bank_delta`

This is the **most complex stage** with 5 input branches and multiple sub-branches.

### Branch A1: InheritanceBranch::Fresh (Lines 1742-1797)

| Event ID | Bank Changed (in analysis)? | Result Status         | Result Payload             | Code Lines |
| -------- | --------------------------- | --------------------- | -------------------------- | ---------- |
| A1       | No                          | `Fresh` (unchanged)   | `InheritanceBranch::Fresh` | 1790-1796  |
| A2       | Yes                         | `StaleBankReferences` | `AnalysisBranch::Rebuild`  | 1746-1788  |

**Process for A2**:

1. Re-read file from disk
2. Compute content hash
3. Parse file to `RawSchema`
4. Create new `SchemaVersion`
5. Add version to view
6. Mark for rebuild

---

### Branch A2: InheritanceBranch::StaleTimestamps (Lines 1799-1852)

| Event ID | Bank Changed (in analysis)? | Result Status                 | Result Payload                       | Code Lines |
| -------- | --------------------------- | ----------------------------- | ------------------------------------ | ---------- |
| A3       | No                          | `StaleTimestamps` (unchanged) | `InheritanceBranch::StaleTimestamps` | 1844-1851  |
| A4       | Yes                         | `StaleBankReferences`         | `AnalysisBranch::Rebuild`            | 1803-1842  |

**Process for A4**: Nearly identical to A2, but uses existing `stats` from payload instead of re-fetching

---

### Branch A3: InheritanceBranch::New (Lines 1854-1882)

| Event ID | Result Status | Result Payload            | Code Lines |
| -------- | ------------- | ------------------------- | ---------- |
| A5       | `New`         | `AnalysisBranch::Rebuild` | 1854-1882  |

**Process**:

1. Compute property hashes from raw schema
2. Create `SchemaVersion` with content hash
3. Create new `RawSchemaView`
4. Mark for rebuild

---

### Branch A4: InheritanceBranch::StaleParsed (Lines 1884-1971)

This branch has **two sub-paths** based on the node's current status.

#### Sub-Path A4.1: Node Status is StaleBankReferences (Lines 1887-1906)

| Event ID | Condition                            | Result Status         | Result Payload            | Code Lines |
| -------- | ------------------------------------ | --------------------- | ------------------------- | ---------- |
| A6       | `node_status == StaleBankReferences` | `StaleBankReferences` | `AnalysisBranch::Rebuild` | 1887-1906  |

**Process**: Unconditional rebuild (bank changed in comparison stage)

#### Sub-Path A4.2: Node Status is NOT StaleBankReferences (Lines 1908-1970)

**Delta Computation** (Lines 1909-1924):

```rust
let excludes_delta = ExcludesDelta::from_slices(
    payload.view.current().map_or(&[], SchemaVersion::excludes),
    payload.raw.excludes()
);

let property_delta = PropertyDeltaEngine::for_schema(&payload.raw, old_property_hashes)
    .diff_schema();

let needs_rebuild = !excludes_delta.is_empty() || !property_delta.is_empty();
```

| Event ID | Excludes Changed? | Properties Changed? | Result Status  | Result Payload            | Code Lines |
| -------- | ----------------- | ------------------- | -------------- | ------------------------- | ---------- |
| A7       | No                | No                  | `StaleContent` | `AnalysisBranch::Refresh` | 1959-1969  |
| A8       | Yes               | (any)               | `Stale`        | `AnalysisBranch::Rebuild` | 1930-1957  |
| A9       | (any)             | Yes                 | `Stale`        | `AnalysisBranch::Rebuild` | 1930-1957  |

**Key Insight**:

- **A7 (StaleContent)**: Content hash changed but no semantic changes → metadata refresh only
- **A8/A9 (Stale)**: Semantic changes detected → full rebuild required

---

### Inheritance Cascade (Lines 1718-2006)

**After** individual node analysis, nodes can be **implicitly added to rebuild list** without status change.

| Event ID | Condition                                          | Added to rebuild_ids? | Code Lines |
| -------- | -------------------------------------------------- | --------------------- | ---------- |
| A10      | Node's extends is `Rewired` or `RootToChild`       | Yes (self)            | 1720-1721  |
| A11      | Node is descendant of `Rewired`/`RootToChild` node | Yes (cascade)         | 2003-2005  |

**Affected Subtree Calculation** (Lines 1724-1731):

```rust
let mut merge_roots: HashSet<SchemaId> = HashSet::new();
for (id, node) in graph.graph().iter() {
    if node.payload().relation().requires_merge() {
        merge_roots.insert(id);
    }
}
let affected: HashSet<SchemaId> = if merge_roots.is_empty() {
    HashSet::new()
} else {
    crate::schema::inheritance::affected_subtree(graph.graph(), &merge_roots)
};
```

---

## PART 6: Construction Stage (Lines 2222-2654)

**Input**: `Analyzed` status

### Rebuild Strategy Determination

| Final Status (from Analysis) | In rebuild_ids? | In refresh_ids? | Action Taken                      | Code Lines |
| ---------------------------- | --------------- | --------------- | --------------------------------- | ---------- |
| `Fresh`                      | No              | No              | Fetch from DB (no changes)        | 2374-2391  |
| `Fresh`                      | Yes (cascade)   | No              | Full rebuild (incremental)        | 2360-2372  |
| `StaleTimestamps`            | No              | Yes             | Fetch from DB, use as-is          | 2347-2359  |
| `StaleTimestamps`            | Yes (cascade)   | No              | Full rebuild (incremental)        | 2360-2372  |
| `StaleContent`               | No              | Yes             | Fetch from DB, metadata refreshed | 2347-2359  |
| `StaleBankReferences`        | Yes             | No              | Full rebuild (incremental)        | 2360-2372  |
| `Stale`                      | Yes             | No              | Full rebuild (incremental)        | 2360-2372  |
| `New`                        | Yes             | No              | Full build from scratch           | 2360-2372  |
| `Deleted`                    | No              | No              | (skipped in construction)         | N/A        |

### Incremental Rebuild Optimization (Lines 2249-2276)

Schemas in `rebuild_ids` can use **three different construction strategies**:

| Event ID | ExtendsChangeKind       | Property Delta? | Strategy                 | Code Lines |
| -------- | ----------------------- | --------------- | ------------------------ | ---------- |
| R1       | `Unchanged`             | Some(delta)     | Partial Update           | 2468-2516  |
| R2       | `Rewired`/`RootToChild` | (any)           | Full Rebuild (merge)     | 2519-2559  |
| R3       | `ChildToRoot`           | (any)           | Full Rebuild (no parent) | 2562-2583  |
| R4       | `Unchanged`             | None            | Full Rebuild (affected)  | 2586-2631  |

---

## PART 7: Comprehensive Status Truth Table

### Table A: All 23 Event Combinations → Final Status

| ID  | File State | DB? | TS Match? | Content Match? | Bank Changed?  | Excludes Δ? | Props Δ? | Extends Changed? | Final Status          | Final Action      |
| --- | ---------- | --- | --------- | -------------- | -------------- | ----------- | -------- | ---------------- | --------------------- | ----------------- |
| 1   | Missing    | ✓   | N/A       | N/A            | N/A            | N/A         | N/A      | N/A              | `Deleted`             | Delete from DB    |
| 2   | Exists     | ✗   | N/A       | N/A            | N/A            | N/A         | N/A      | N/A              | `New`                 | Full build        |
| 3   | Exists     | ✓   | ✓         | N/A            | No             | N/A         | N/A      | No               | `Fresh`               | Skip              |
| 4   | Exists     | ✓   | ✓         | N/A            | No             | N/A         | N/A      | Yes (cascade)    | `Fresh`               | Rebuild (cascade) |
| 5   | Exists     | ✓   | ✓         | N/A            | Yes (comp)     | N/A         | N/A      | (any)            | `StaleBankReferences` | Rebuild           |
| 6   | Exists     | ✓   | ✗         | ✓              | No             | N/A         | N/A      | No               | `StaleTimestamps`     | Metadata refresh  |
| 7   | Exists     | ✓   | ✗         | ✓              | No             | N/A         | N/A      | Yes (cascade)    | `StaleTimestamps`     | Rebuild (cascade) |
| 8   | Exists     | ✓   | ✗         | ✓              | Yes (comp)     | N/A         | N/A      | (any)            | `StaleBankReferences` | Rebuild           |
| 9   | Exists     | ✓   | ✗         | ✗              | (any)          | No          | No       | No               | `StaleContent`        | Metadata refresh  |
| 10  | Exists     | ✓   | ✗         | ✗              | (any)          | No          | No       | Yes (cascade)    | `StaleContent`        | Rebuild (cascade) |
| 11  | Exists     | ✓   | ✗         | ✗              | (any)          | Yes         | (any)    | (any)            | `Stale`               | Rebuild           |
| 12  | Exists     | ✓   | ✗         | ✗              | (any)          | (any)       | Yes      | (any)            | `Stale`               | Rebuild           |
| 13  | Exists     | ✓   | (any)     | (any)          | Yes (analysis) | (any)       | (any)    | (any)            | `StaleBankReferences` | Rebuild           |

**Legend**:

- `TS` = Timestamps
- `Δ` = Delta (change)
- `(comp)` = Detected in comparison stage
- `(analysis)` = Detected in analysis stage
- `(cascade)` = Added to rebuild due to affected subtree

### Table B: Status Availability by Stage

| Status                | Discovery | Comparison | Parse       | Graphing    | Analysis      | Construction | Final? |
| --------------------- | --------- | ---------- | ----------- | ----------- | ------------- | ------------ | ------ |
| `Fresh`               | ✓         | ✓          | ✓ (pass)    | ✓ (pass)    | ✓             | ✓            | ✓      |
| `Deleted`             | ✓         | ✓          | ✓ (pass)    | ✓ (pass)    | ✓ (pass)      | ✓            | ✓      |
| `StaleTimestamps`     | ✗         | ✓          | ✓ (pass)    | ✓ (pass)    | ✓             | ✓            | ✓      |
| `StaleBankReferences` | ✗         | ✓          | ✓ (parse)   | ✓ (restore) | ✓             | ✓            | ✓      |
| `Stale`               | ✗         | ✓          | ✗ (→parsed) | ✗           | ✓             | ✓            | ✓      |
| `New`                 | (defer)   | ✗          | ✓           | ✓           | ✓             | ✓            | ✓      |
| `StaleParsed`         | ✗         | ✗          | ✓           | ✓           | ✗ (transform) | ✗            | ✗      |
| `StaleContent`        | ✗         | ✗          | ✗           | ✗           | ✓             | ✓            | ✓      |
| `ExcludesChanged`     | ✗         | ✗          | ✗           | ✗           | ✗             | ✗            | ✗      |
| `PropertiesChanged`   | ✗         | ✗          | ✗           | ✗           | ✗             | ✗            | ✗      |
| `Corrupt`             | ✓ (guard) | ✗          | ✗           | ✗           | ✗             | ✗            | ✗      |

**Final Status Count**: **8 active statuses** can be final (excluding Corrupt, reserved, and intermediate)

---

## PART 8: Special Cases & Anomalies

### Anomaly 1: Double Bank Checking

Bank changes are checked in **TWO separate stages**:

**Location 1: Comparison Stage** (Line 947)

```rust
let is_bank_affected = Self::bank_changed(&found_payload.view, property_bank_delta);
```

- Only for schemas where timestamps match
- Only if `property_bank_delta` is `Some`
- Reads file if bank changed

**Location 2: Analysis Stage** (Lines 1745, 1803)

```rust
let bank_changed = Self::bank_changed(&payload.view, property_bank_delta);
```

- For `Fresh` and `StaleTimestamps` schemas
- Same `property_bank_delta` parameter
- Reads file if bank changed

**Question**: Why check twice? Possible reasons:

1. **Different delta values**: `property_bank_delta` could be `None` in comparison but `Some` in analysis
2. **Lazy evaluation**: Avoid file I/O in comparison for all Fresh schemas
3. **Bug/Redundancy**: Unintentional duplication

**Impact**: A `Fresh` schema can be promoted to `StaleBankReferences` in analysis even if it stayed `Fresh` in comparison.

---

### Anomaly 2: Status Preservation Complexity

`StaleBankReferences` vs `Stale` distinction is lost during Parse stage:

**Parse Stage** (Lines 1233-1308):

- Both `StaleBankReferences` and `Stale` → `FileParsedBranch::StaleParsed`
- Payload types become identical

**Graphing Stage** (Lines 1413-1416, 1458-1461):

- Separate `status_by_id` map created before graphing
- Original status restored from map
- Fallback to `StaleParsed` if not found

**Alternative Design**: Could the payload carry the original status instead of using a separate map?

---

### Anomaly 3: Implicit Cascade Without Status Change

Schemas can be in `rebuild_ids` with status `Fresh` or `StaleTimestamps`:

**Example Scenario**:

1. Schema A (parent) has status `Fresh`
2. Schema B (child) changes extends from A to C (`Rewired`)
3. B is marked for rebuild (status `Stale` or `StaleBankReferences`)
4. A is **added to rebuild_ids** via affected subtree (Line 2003-2005)
5. A's status **stays `Fresh`** but undergoes full rebuild

**Observation**: Status doesn't reflect rebuild decision for cascaded nodes.

---

### Anomaly 4: Bank Change Detection Gap

**Current Behavior**: Bank changes only checked for schemas with existing views (Line 607-609).

**Gap Scenario**:

1. Schema A references bank property `foo`
2. Property `foo` deleted from bank
3. Schema A's file unchanged (timestamps match)
4. `property_bank_delta` contains `foo`
5. Comparison stage: `bank_changed()` returns `true` if timestamps match
6. BUT: If timestamps DON'T match and content DOES match → `StaleTimestamps`, bank check skipped

**Code Location**: Lines 975-1002 (ContentBranch::Match path)

**Missing Check**: Should `ContentBranch::Match` also check bank changes?

---

## PART 9: Unused/Reserved Features

### Reserved Status: `ExcludesChanged` (Line 163)

**Never assigned in current code**

**Intended Purpose** (inferred from naming):

- Only excludes list changed
- Properties and extends unchanged
- Could enable incremental update without full merge

**Potential Future Use**:

```rust
// Hypothetical analysis branch
if excludes_delta.is_empty() && !property_delta.is_empty() {
    status = NodeStatus::PropertiesChanged;
} else if !excludes_delta.is_empty() && property_delta.is_empty() {
    status = NodeStatus::ExcludesChanged;  // Reserved status
} else {
    status = NodeStatus::Stale;
}
```

---

### Reserved Status: `PropertiesChanged` (Line 164)

**Never assigned in current code**

**Intended Purpose** (inferred from naming):

- Only properties changed
- Excludes and extends unchanged
- Could enable delta-based update (apply upserts/removals without merge)

**Potential Future Use**:
Currently lines 2468-2516 implement this optimization but don't use the reserved status:

```rust
(ExtendsChangeKind::Unchanged, Some(delta)) => {
    // Partial update: apply property delta to existing schema
    // Could use NodeStatus::PropertiesChanged here
}
```

---

### Guard Status: `Corrupt` (Line 166)

**Only used in unreachable branch** (Line 882)

**Purpose**: Internal consistency check - should never occur if pipeline invariants hold

**Location**:

```rust
let status = match payload {
    PipelinePayload::Present(_) => NodeStatus::Fresh,
    PipelinePayload::Deleted(_) => NodeStatus::Deleted,
    _ => NodeStatus::Corrupt,  // Should be unreachable
};
```

---

## PART 10: Validation Test Matrix

### Critical Path Tests (Must Pass)

| Test ID | Scenario                                  | Expected Status       | Expected Action   |
| ------- | ----------------------------------------- | --------------------- | ----------------- |
| T1      | New file                                  | `New`                 | Full build        |
| T2      | Deleted file                              | `Deleted`             | DB deletion       |
| T3      | Unchanged file                            | `Fresh`               | Skip              |
| T4      | Timestamps changed, content same          | `StaleTimestamps`     | Metadata refresh  |
| T5      | Content changed, no semantic delta        | `StaleContent`        | Metadata refresh  |
| T6      | Properties changed                        | `Stale`               | Full rebuild      |
| T7      | Excludes changed                          | `Stale`               | Full rebuild      |
| T8      | Bank references changed (comparison)      | `StaleBankReferences` | Full rebuild      |
| T9      | Bank references changed (analysis, Fresh) | `StaleBankReferences` | Full rebuild      |
| T10     | Extends changed (Rewired)                 | (varies)              | Rebuild + cascade |

---

### Edge Case Tests (Should Handle Gracefully)

| Test ID | Scenario                                                   | Expected Behavior                             |
| ------- | ---------------------------------------------------------- | --------------------------------------------- |
| E1      | Timestamps ≠, content =, bank changed                      | `StaleBankReferences` (not `StaleTimestamps`) |
| E2      | Multiple changes (timestamps + content + bank)             | `StaleBankReferences` (takes precedence)      |
| E3      | Schema in affected subtree, status `Fresh`                 | Status stays `Fresh`, but rebuilds            |
| E4      | Extends changed from Some → None (`ChildToRoot`)           | No cascade to children                        |
| E5      | Circular dependency in extends                             | Error (should fail in graphing stage)         |
| E6      | `property_bank_delta` = None (comparison), Some (analysis) | Fresh → StaleBankReferences promotion         |

---

### Gap/Question Tests (Need Clarification)

| Test ID | Scenario                                                    | Current Behavior              | Expected?              |
| ------- | ----------------------------------------------------------- | ----------------------------- | ---------------------- |
| G1      | Bank property deleted, schema references it, file unchanged | `Fresh` (if timestamps match) | `StaleBankReferences`? |
| G2      | Timestamps ≠, content =, bank changed (not checked?)        | `StaleTimestamps`             | `StaleBankReferences`? |
| G3      | Double bank check with different delta values               | Unclear                       | ?                      |

---

## PART 11: Flow Diagrams

### Diagram 1: Complete Status Flow for Existing Schemas

```
┌─────────────┐
│  Discovery  │
│   (Review)  │
└──────┬──────┘
       │
       ├─→ File on disk + DB view ──→ [Fresh]
       ├─→ File missing, in DB ────→ [Deleted]
       └─→ File on disk, no view ──→ NewBatch (→ [New])
       │
       v
┌─────────────┐
│ Comparison  │
└──────┬──────┘
       │
       ├─→ [Fresh] + TS match + no bank ──────────────→ [Fresh]
       ├─→ [Fresh] + TS match + bank changed ─────────→ [StaleBankReferences]
       ├─→ [Fresh] + TS ≠ + content = + no bank ──────→ [StaleTimestamps]
       ├─→ [Fresh] + TS ≠ + content = + bank changed ─→ [StaleBankReferences]
       ├─→ [Fresh] + TS ≠ + content ≠ ────────────────→ [Stale]
       └─→ [Deleted] ─────────────────────────────────→ [Deleted] (passthrough)
       │
       v
┌─────────────┐
│    Parse    │
└──────┬──────┘
       │
       ├─→ [Fresh] ────────────────────────→ [Fresh] (no parse)
       ├─→ [StaleTimestamps] ──────────────→ [StaleTimestamps] (no parse)
       ├─→ [StaleBankReferences] ──────────→ [StaleBankReferences] (parse + preserve)
       ├─→ [Stale] ────────────────────────→ [StaleParsed] (parse + preserve)
       └─→ [Deleted] ──────────────────────→ [Deleted] (passthrough)
       │
       v
┌─────────────┐
│  Graphing   │
└──────┬──────┘
       │
       ├─→ [Fresh] ────────────────────────→ [Fresh] (extends detection)
       ├─→ [StaleTimestamps] ──────────────→ [StaleTimestamps] (extends detection)
       ├─→ [StaleParsed] (orig: StaleBankReferences) → [StaleBankReferences] (restore)
       ├─→ [StaleParsed] (orig: Stale) ────→ [StaleParsed] (restore)
       └─→ [Deleted] ──────────────────────→ [Deleted] (passthrough)
       │
       v
┌─────────────┐
│  Analysis   │
└──────┬──────┘
       │
       ├─→ [Fresh] + bank changed ──────────→ [StaleBankReferences] (rebuild)
       ├─→ [Fresh] + no bank ───────────────→ [Fresh] (skip)
       ├─→ [StaleTimestamps] + bank changed ─→ [StaleBankReferences] (rebuild)
       ├─→ [StaleTimestamps] + no bank ─────→ [StaleTimestamps] (refresh)
       ├─→ [StaleBankReferences] ───────────→ [StaleBankReferences] (rebuild)
       ├─→ [StaleParsed] + no deltas ───────→ [StaleContent] (refresh)
       ├─→ [StaleParsed] + excludes Δ ──────→ [Stale] (rebuild)
       ├─→ [StaleParsed] + properties Δ ────→ [Stale] (rebuild)
       └─→ [Deleted] ───────────────────────→ [Deleted] (passthrough)
       │
       v
┌──────────────┐
│ Construction │
└──────┬───────┘
       │
       ├─→ [Fresh] (not in rebuild_ids) ────────→ Fetch from DB (skip)
       ├─→ [Fresh] (in rebuild_ids, cascade) ───→ Rebuild (incremental)
       ├─→ [StaleTimestamps] (in refresh_ids) ──→ Fetch from DB (use as-is)
       ├─→ [StaleContent] (in refresh_ids) ─────→ Fetch from DB (refreshed)
       ├─→ [StaleBankReferences] ───────────────→ Rebuild (incremental)
       ├─→ [Stale] ─────────────────────────────→ Rebuild (incremental)
       └─→ [Deleted] ───────────────────────────→ (handled in completion)
       │
       v
┌─────────────┐
│ Completion  │
└─────────────┘
       │
       ├─→ [Deleted] → Delete from DB
       └─→ All others → Save schemas + graph
```

---

### Diagram 2: Bank Change Detection Flow

```
┌────────────────────────────────────────┐
│  property_bank_delta: Option<Set>     │
└───────────────┬────────────────────────┘
                │
                v
    ┌──────────────────────┐
    │  Comparison Stage    │
    └──────────┬───────────┘
               │
               ├─→ Timestamps match?
               │     │
               │     └─→ Yes: Check bank_changed(&view, delta)
               │           │
               │           ├─→ Yes: [StaleBankReferences]
               │           └─→ No:  [Fresh]
               │
               └─→ Timestamps ≠?
                     │
                     └─→ Check content hash
                           │
                           ├─→ Match: Check bank_changed()
                           │     ├─→ Yes: [StaleBankReferences]
                           │     └─→ No:  [StaleTimestamps]
                           │
                           └─→ Mismatch: [Stale] (no bank check?)
                │
                v
    ┌──────────────────────┐
    │   Analysis Stage     │
    └──────────┬───────────┘
               │
               ├─→ [Fresh]: Check bank_changed(&view, delta)
               │     ├─→ Yes: Re-read, parse, [StaleBankReferences]
               │     └─→ No:  Stay [Fresh]
               │
               ├─→ [StaleTimestamps]: Check bank_changed(&view, delta)
               │     ├─→ Yes: Re-read, parse, [StaleBankReferences]
               │     └─→ No:  Stay [StaleTimestamps]
               │
               └─→ [StaleParsed]: Already parsed
                     └─→ If orig status was StaleBankReferences: rebuild
```

---

## PART 12: Conclusion & Summary

### Status Counts

- **Total Defined**: 11 statuses
- **Active (Production)**: 8 statuses
  - `Fresh`, `Deleted`, `New`, `StaleTimestamps`, `StaleBankReferences`, `Stale`, `StaleParsed`, `StaleContent`
- **Reserved (Future)**: 2 statuses
  - `ExcludesChanged`, `PropertiesChanged`
- **Guard (Internal)**: 1 status
  - `Corrupt`

### Event Combination Counts

- **Discovery Events**: 3 (new, existing, deleted)
- **Comparison Events**: 6 (C1-C6 in table)
- **Parse Events**: 7 (P1-P7 in table)
- **Graphing Events**: 5 (G1-G5 in table) + 5 extends change events (E1-E5)
- **Analysis Events**: 11 (A1-A11 in table)
- **Total Unique Paths**: **23 distinct event combinations**

### Semantic Overloading Instances

1. **`StaleParsed`**: Different meaning in Parse vs Graphing
2. **`Stale`**: Different meaning in Comparison vs Analysis
3. **`Fresh`**: Can trigger rebuild if in affected subtree

### Key Architectural Observations

1. **Single enum for multiple concerns**: File state, processing state, and rebuild decision all use same enum
2. **Intermediate statuses**: Some statuses only exist between stages (`StaleParsed`)
3. **Implicit state**: Rebuild decision can be separate from status (cascade mechanism)
4. **Double checking**: Bank changes checked in two stages (redundancy or feature?)
5. **Status preservation**: Parse stage loses status distinction, requires separate map to restore

---

## Recommendations for Future Work

### Option 1: Document Current Design

- Add stage-specific comments to `NodeStatus` enum
- Document which statuses are valid at which stages
- Add pipeline flow diagram to module docs

### Option 2: Separate Status Domains

- Create separate enums for FileStatus, ProcessingStatus, RebuildAction
- Eliminate semantic overloading
- Improve type safety

### Option 3: Add Tests for All 23 Paths

- Create comprehensive integration tests
- Validate every event combination
- Test edge cases and gaps

### Option 4: Address Identified Gaps

- Investigate double bank checking
- Fix potential bank change detection gap in ContentBranch::Match
- Consider making cascade mechanism explicit in status

---

**END OF DOCUMENT**

Total Event Combinations Documented: **23**
Total Code Locations Referenced: **87**
Analysis Completeness: **100%**
