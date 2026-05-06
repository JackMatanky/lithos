# Schema Module Pipeline Review: Complete Analysis for State Machine Redesign

**Date**: 2026-03-19
**Purpose**: Comprehensive review of all pipeline stages to inform typestate pattern implementation
**Scope**: PropertyBank and Schema pipelines in `lithos-core/src/schema/`

---

## Revision History

**2026-03-23 (Evening)**: FINALIZED OPTIMIZED PIPELINE ARCHITECTURE

- **BREAKING OPTIMIZATION**: Early tree construction with fail-fast validation
- Moved InheritanceGraph construction BEFORE raw properties deserialization
- Eliminated `RefExpandedSchema` intermediate type (simplified pipeline)
- Confirmed level-by-level expansion + merging with staleness optimizations
- Finalized hash strategy: `u64` for ancestors_hash, `[u8; 16]` for content/properties
- Renamed `SchemaTree` → `InheritanceGraph` (reflects lightweight structure)
- **Final state count**: PropertyBank: 6, Schema: 9 (reduced from 11)
- `expanded_properties` stores locally expanded properties (no parent merge)
- Two-state design for expansion/merging (RefsExpanded → PropertiesMerged)

**2026-03-23 (Morning)**: Major architectural clarifications and redb research integration

- Redesigned inheritance views for efficient tree queries
- Clarified Builder mutability and infrastructure separation
- Added delta structures with old/new tracking for extends
- Integrated redb research findings for optimal storage schema
- Updated state counts (PropertyBank: 6, Schema: 11)

---

## Executive Summary

The schema module implements two parallel pipelines:

1. **PropertyBank Pipeline**: Discovery → Branching (NEW/FreshTimestamp/STALE) → Construction → Persistence
2. **Schema Pipeline**: File → Raw → Expanded → Tree → Merged → Storage

Both pipelines share infrastructure (Ingestor, Repository) and follow distinct stages with clear inputs/outputs but **lack explicit state machine enforcement**. This leads to:

- **Unorganized orchestration** in `Loader`
- **Implicit state transitions** scattered across modules
- **Difficult-to-track** intermediate data flows
- **Complex staleness detection** interleaved with processing
- **"Validate, then use" anti-pattern** in Raw types (violates "parse, don't validate")
- **Redundant abstraction layers** (`Ingestor` acting as an unnecessary middleman)

**Recommendation**: Implement **two separate state machines**, eliminate the `Ingestor`, and refactor `Loader` into a `Builder` facade.

**Prerequisites**:

1. Refactor `RawPropertyBank` and `RawSchema` to use "parse, don't validate" pattern
   - Move validation into parsing constructors
   - Make fields private, add public accessors
   - Guarantee validity at type level

**Architectural Shifts**:

1. **Eliminate `Ingestor`**: Move all filesystem and DB access directly into state machine transition logic.
2. **Refactor `Loader` to `Builder`**: Convert the complex orchestrator into a thin 20-line facade that simply instantiates and drives the state machines.

**State Machines**:

1. `PropertyBankStateMachine` - 6 states, branching based on staleness (NEW/FreshTimestamp/FreshContent/STALE paths)
2. `SchemaStateMachine` - 9 states (optimized from 11), with fail-fast validation and incremental updates

---

## 0. OPTIMIZED PIPELINE ARCHITECTURE (FINAL - 2026-03-23)

This section documents the **finalized optimized pipeline** with all architectural decisions confirmed.

### 0.1 Key Optimizations

#### Critical Breakthrough: Early Tree Construction

The pipeline has been optimized to **construct the inheritance graph BEFORE deserializing raw properties**. This enables:

1. **Fail-Fast Validation**: Structural errors (cycles, missing parents) detected immediately
2. **Minimal Deserialization**: Only deserialize properties for schemas that actually need processing
3. **Simplified Pipeline**: Eliminates `InheritanceEvaluated` state (folded into `TreeConstructed`)
4. **Reduced State Count**: Schema pipeline reduced from 11 states to 9 states

#### Level-by-Level Expansion + Merging with Staleness Optimization

Instead of batch expansion → tree building → merging, the optimized pipeline:

1. **Constructs lightweight InheritanceGraph** (just `SchemaId` relationships, no properties)
2. **Processes schemas level-by-level** in topological order
3. **For each level**:
   - **FRESH schemas**: Retrieve fully merged properties from DB (skip expansion & merging)
   - **STALE/NEW schemas**:
     - Expand $refs in local properties
     - Immediately merge with parent's ALREADY-RESOLVED properties (from DB or previous level)
4. **Single pass through the tree**: No separate batch operations

This approach respects incremental updates: fresh schemas at any level can bypass processing entirely.

### 0.2 Finalized Design Decisions

| Decision Area                      | Choice                                         | Rationale                                                                                        |
| ---------------------------------- | ---------------------------------------------- | ------------------------------------------------------------------------------------------------ |
| **`ancestors_hash` type**          | `u64` (8 bytes)                                | Cache invalidation tolerates rare false positives (P=2.7×10⁻⁶ @ 10K schemas); better performance |
| **`HashMetadata.content` hash**    | `[u8; 16]` (128-bit)                           | Cryptographically sound (P=1.5×10⁻¹⁸ @ 1M items); 50% storage savings vs 256-bit                 |
| **`HashMetadata.properties` hash** | `[u8; 16]` (128-bit)                           | Negligible collision risk (P=1.5×10⁻²³ @ 100K items); memory efficient                           |
| **Tree structure naming**          | `InheritanceGraph` (renamed from `SchemaTree`) | Lightweight structure (just IDs); name reflects purpose                                          |
| **`RefExpandedSchema` type**       | **ELIMINATED**                                 | No longer needed; properties populated directly in nodes during expansion                        |
| **`expanded_properties` cache**    | **Locally expanded properties** (Option B)     | Stores properties with $refs resolved, BEFORE parent merge; enables incremental updates          |
| **Expansion/Merge states**         | **Two separate states** (Design A)             | RefsExpanded → PropertiesMerged; easier debugging despite single-loop implementation             |
| **InheritanceGraph construction**  | **BEFORE raw properties deserialization**      | Fail-fast on structural errors; minimal deserialization                                          |

### 0.3 Optimized Schema Pipeline State Sequence

The finalized 9-state pipeline:

```
1. Discovery
   ↓
2. FileParsed (TOML/JSON/YAML → raw bytes)
   ↓
3. SchemaDelta (detect which schemas changed via timestamps/hashes)
   ↓
4. TreeConstructed (build InheritanceGraph with just SchemaIds)
   ├─ Cycle detection
   ├─ Parent verification
   └─ FAIL FAST if structural errors
   ↓
5. RawPropertiesDeserialized (only for schemas that need processing)
   ↓
6. BankReferenceDelta (which schemas reference changed PropertyBank properties)
   ↓
7. RefsExpanded (level-by-level $ref expansion)
   ├─ FRESH schemas: skip (retrieve from DB)
   └─ STALE/NEW schemas: expand local properties
   ↓
8. PropertiesMerged (level-by-level inheritance merge)
   ├─ FRESH schemas: retrieve merged result from DB
   └─ STALE/NEW schemas: merge with parent's resolved properties
   ↓
9. Persisted
```

**Key Change from Original Design**:

- **Original**: Discovery → FileParsed → SchemaDelta → RawPropertiesDeserialized → InheritanceEvaluated → BankReferenceDelta → RefsExpanded → TreeConstructed → PropertiesMerged → Persisted (11 states)
- **Optimized**: TreeConstructed moved BEFORE RawPropertiesDeserialized, InheritanceEvaluated folded into TreeConstructed (9 states)

### 0.4 InheritanceGraph Structure (Renamed from SchemaTree)

The `InheritanceGraph` is now a **lightweight structure** containing only:

```rust
pub struct InheritanceGraph {
    /// Topologically ordered schema IDs (parents before children)
    order: Vec<SchemaId>,

    /// Node metadata (NO properties yet)
    nodes: HashMap<SchemaId, InheritanceNode>,
}

pub struct InheritanceNode {
    id: SchemaId,
    name: SchemaName,
    parent_id: Option<SchemaId>,
    children: Vec<SchemaId>,
    depth: usize,
    excludes: Vec<PropertyName>,  // From RawSchema metadata

    // Properties are NOT stored here!
    // They're populated during RefsExpanded/PropertiesMerged states
}
```

**Why Lightweight?**

- Enables early construction with minimal deserialization
- Properties added later during expansion/merging phases
- Structural validation (cycles, parents) happens immediately

### 0.5 Incremental Staleness Handling

The level-by-level algorithm respects incremental updates:

```
For each level L in topological order:
    For each schema S in level L:
        If S is FRESH AND parent(S) is FRESH:
            # Zero processing needed
            merged_properties ← retrieve from DB (SCHEMA_BY_ID)
            Skip expansion and merging

        Else if S is FRESH AND parent(S) is STALE:
            # Parent changed, child's local properties unchanged
            expanded_properties ← retrieve from DB (SchemaVersion.expanded_properties)
            merged_properties ← merge(expanded_properties, parent.merged_properties)

        Else if S is STALE:
            # Schema file changed (or NEW)
            If PropertyBank is STALE AND schema references changed bank properties:
                # Re-expand affected $refs
                Apply PropertyDelta to cached expanded_properties
            Else:
                # Only expand NEW/MODIFIED local properties
                Apply SchemaDelta to cached expanded_properties

            merged_properties ← merge(expanded_properties, parent.merged_properties)
```

**Key Insight**: Fresh schemas can skip processing at ANY level, as long as their parents are also fresh (or have been freshly resolved in a previous level).

### 0.6 Data Structures

#### SchemaVersion (RawSchemaView)

```rust
pub struct SchemaVersion {
    file_times: FileTimesMetadata,
    hashes: HashMetadata,  // content: [u8; 16], properties: HashMap<PropertyName, [u8; 16]>
    version: Box<str>,
    extends: Option<SchemaName>,
    excludes: Vec<PropertyName>,
    raw_properties: Vec<u8>,  // Serde-serialized RawPropertyMap
    bank_references: HashMap<PropertyName, PropertyName>,  // Extracted from $refs

    /// CRITICAL: Stores locally expanded properties (with $refs resolved)
    /// Does NOT include parent-inherited properties (those come from merging)
    expanded_properties: Option<HashMap<PropertyName, Property>>,
}
```

#### SchemaInheritanceView

```rust
pub struct SchemaInheritanceView {
    parent_id: Option<SchemaId>,
    depth: usize,  // Pre-computed during tree building
    ancestors_hash: u64,  // Fast cache invalidation
}
```

### 0.8 Simplified Extender (Now Builds InheritanceGraph)

The `Extender` module is drastically simplified:

**Old Responsibility**: Build full `SchemaTree` with properties in `SchemaNode`
**New Responsibility**: Build lightweight `InheritanceGraph` with just IDs and metadata

**Operations**:

1. Build node map (SchemaId → InheritanceNode)
2. DFS cycle detection
3. Populate children lists
4. Compute depths (BFS)
5. Kahn's topological sort

**Input**: `Vec<(SchemaId, SchemaName, Option<SchemaName>, Vec<PropertyName>)>` (id, name, extends, excludes)
**Output**: `InheritanceGraph` (topologically ordered IDs)

**NO LONGER DOES**:

- Property expansion
- Property merging
- `RefExpandedSchema` construction

### 0.9 Implementation Phases

**Phase 1: Prerequisites** (Already Complete)

- ✅ `RawPropertyMap<T>` wrapper (validates keys during parsing)
- ✅ `RawPropertyRefPath` type (validates $ref format during parsing)

**Phase 2: Rename SchemaTree → InheritanceGraph**

- Rename type throughout codebase
- Update documentation

**Phase 3: Simplify Extender**

- Remove property handling
- Build lightweight `InheritanceGraph`
- Move to early pipeline stage (before deserialization)

**Phase 4: Implement PropertyBank State Machine**

- 6 states as documented in Section 1

**Phase 5: Implement Schema State Machine**

- 9 states with optimized sequence
- Level-by-level expansion + merging
- Incremental staleness handling

**Phase 6: Refactor Loader → Builder**

- Thin facade (20 lines)
- Instantiate and drive state machines

**Phase 7: Remove Ingestor**

- Move I/O into state machine transitions
- Direct Repository and FsReader usage

---

## 1. PropertyBank Pipeline Stages

### 1.1 State Identification

The PropertyBank follows a **branching pipeline** with 6 distinct states, determined by staleness detection:

```
┌──────────────────────────────────────────────────────────────────────┐
│                              Discovery                               │
│                   (Query DB for RawPropertyBankView)                 │
│             (Determine NEW/FreshTimestamp/FreshContent/STALE)        │
└───────┬─────────────────┬────────────────┬─────────────────┬─────────┘
        │                 │                │                 │
        ▼                 ▼                ▼                 ▼
    ┌───────┐     ┌──────────────┐  ┌────────────┐       ┌───────┐
    │  NEW  │     │FreshTimestamp│  │FreshContent│       │ STALE │
    └───┬───┘     └───────┬──────┘  └──────┬─────┘       └───┬───┘
        │                 │                │                 │
        ▼                 │                │                 ▼
  ┌───────────┐           │                │           ┌───────────┐
  │FileParsed │           │                │           │FileParsed │
  │  (+view)  │           │                │           │  (+view)  │
  └─────┬─────┘           │                │           └─────┬─────┘
        │                 │                │                 │
        │                 │                │                 ▼
        │                 │                │         ┌──────────────┐
        │                 │                │         │PropertyDelta │
        │                 │                │         └──────┬───────┘
        │                 │                │                │
        ▼                 ▼                ▼                ▼
 ┌──────────────┐ ┌──────────────┐ ┌──────────────┐  ┌──────────────┐
 │BaseConstructed││BaseConstructed││BaseConstructed│ │BaseConstructed│
 │(from scratch)│ │  (from DB)   │ │ (+upd times) │  │  (from DB)   │
 └──────┬───────┘ └───────┬──────┘ └───────┬──────┘  └───────┬──────┘
        │                 │                │                 │
        │                 │                │                 ▼
        │                 │                │           ┌────────────┐
        │                 │                │           │DeltaApplied│
        │                 │                │           │  (+view)   │
        │                 │                │           └─────┬──────┘
        │                 │                │                 │
        ▼                 ▼                ▼                 ▼
┌──────────────────────────────────────────────────────────────────────┐
│                              Completed                               │
└──────────────────────────────────────────────────────────────────────┘
```

**State Details**:

| State                  | Data Structure                                   | Location                | Transitions                        | Notes                                      |
| ---------------------- | ------------------------------------------------ | ----------------------- | ---------------------------------- | ------------------------------------------ |
| **1. Discovery**       | `&Path` + `Option<RawPropertyBankView>`          | Entry point             | → PropertyBankPath enum            | Query DB, determine staleness              |
| **2. FileParsed**      | `RawPropertyBank` + `RawPropertyBankView`        | NEW/STALE paths only    | → BaseConstructed or PropertyDelta | File parsed + validated + view created     |
| **3. PropertyDelta**   | Delta info (new/modified/removed)                | STALE path only         | → BaseConstructed                  | Property-level hash comparison             |
| **4. BaseConstructed** | `PropertyBank`                                   | All paths converge here | → DeltaApplied or Completed        | Fetched from DB or built from scratch      |
| **5. DeltaApplied**    | `PropertyBank` (updated) + `RawPropertyBankView` | STALE path only         | → Completed                        | Incremental updates applied + view updated |
| **6. Completed**       | `PropertyBank` (persisted)                       | Terminal state          | -                                  | Both bank and view persisted to DB         |

**Key Changes from Original Analysis**:

- **"Validated" state eliminated** - validation now happens during parsing (see Section 1.4)
- **"ViewUpdated" state eliminated** - view creation folded into FileParsed and DeltaApplied states
- **6 states instead of 7** - more cohesive state groupings

### 1.2 Stage-by-Stage Breakdown

The PropertyBank pipeline branches into **four distinct paths** based on staleness detection:

#### **Discovery Phase** (State 1)

**Operations**:

1. Query `Repository::get_raw_property_bank_view()` → `Option<RawPropertyBankView>`
2. If `None` → **NEW** path
3. If `Some(view)`:
   - Extract file timestamps (`created_at`, `modified_at`)
   - Compare with `view.file_times()`
   - **If timestamps match** → **FreshTimestamp** path
   - **If timestamps differ**:
     - Read file content with `FsReader::read_with()`
     - Compute `blake3::hash()` of content
     - Compare with `view.hashes().content_hash()`
     - **If hash matches** → **FreshContent** path (timestamp update needed)
     - **If hash differs** → **STALE** path

**Errors**: `SchemaRepositoryError`, `SchemaFileError`
**Location**: `Ingestor::property_bank()` lines 473-510

**Output**: `PropertyBankPath` enum with one of:

- `PropertyBankPath::New(PropertyBankPipeline<FileParsed>)`
- `PropertyBankPath::FreshTimestamp(PropertyBankPipeline<BaseConstructed>)`
- `PropertyBankPath::FreshContent(PropertyBankPipeline<BaseConstructed>)`
- `PropertyBankPath::Stale(PropertyBankPipeline<FileParsed>)`

---

#### **Path 1: NEW** (No cached view exists)

**Steps**:

**State 1→2: File Parsing + View Creation**

- **Input**: `&Path` to property bank file
- **Output**: `RawPropertyBank` (validated) + `RawPropertyBankView`
- **Operations**:
  - Read file with `FsReader::read_with()`
  - Parse via `FsReader::parse_structured_from_str()` (uses custom `RawPropertyMap<T>` deserializer)
  - Property names validated during deserialization (guaranteed valid `PropertyName` keys)
  - Version validated separately via `raw.validate_version()`
  - Create `RawPropertyBankView::try_from_with_content()` with all metadata
- **Errors**: `SchemaParseError`, `SchemaVersionError`, `PropertyNameError`
- **Location**: `Ingestor::ingest_new_property_bank()` line 522-527
- **Note**: View creation happens in FileParsed state (all data available)

**State 2→4: Base Construction (from scratch)**

- **Input**: `RawPropertyBank` (validated)
- **Output**: `PropertyBank`
- **Operations**:
  - `PropertyBank::try_from(RawPropertyBank)` - convert all raw entries to `Property`
  - For each property:
    - Convert `RawPropertySpec` → `PropertySpec`
    - Create `PropertyId::new()` (UUID v7)
    - Keys are already `PropertyName` (no need for `try_new()`)
    - Set `Optionality`, `Multiplicity`
    - Call `PropertyBank::register()`
- **Errors**: `SchemaError::PropertyBank(DuplicatePropertyName)`, `PropertySpec` conversion errors
- **Location**: `PropertyBank::try_from()` in `bank.rs:313-367`

**State 4→6: Persist & Complete**

- **Operations**:
  - `Repository::save_property_bank(&bank)`
  - `Repository::save_raw_property_bank_view(&view)`
- **Location**: Lines 530-545
- **Note**: Both bank and view persisted together

---

#### **Path 2: FreshTimestamp** (Timestamps match)

**Steps**:

**State 1→4: Base Construction (from DB)**

- **Input**: None (view timestamps match)
- **Output**: `PropertyBank`
- **Operations**:
  - `Repository::get_property_bank()` - fetch cached bank
- **Errors**: `SchemaStorageError::PropertyBankNotFound` (if DB inconsistent)
- **Location**: `Ingestor::property_bank()` lines 498-505

**State 4→6: Complete (no persistence needed)**

- **Operations**: Return cached `PropertyBank`
- **Note**: No view update, no bank persistence (everything is fresh - skip straight to Completed)

---

#### **Path 3: FreshContent** (Content hash matches but timestamps differ)

**Steps**:

**State 1→4: Base Construction (from DB) + View Update**

- **Input**: Existing `RawPropertyBankView`, new timestamps
- **Output**: `PropertyBank` + Updated `RawPropertyBankView`
- **Operations**:
  - `Repository::get_property_bank()` - fetch cached bank
  - Clone existing view
  - Update `file_times()` with new timestamps
  - Keep content hash, property hashes, and compressed content unchanged
- **Note**: View update happens in BaseConstructed state (no separate ViewUpdated state)

**State 4→6: Persist View Only & Complete**

- **Operations**:
  - `Repository::save_raw_property_bank_view(&updated_view)`
  - **NO** `save_property_bank()` (bank unchanged)

---

#### **Path 4: STALE** (Content hash differs)

**Steps**:

**State 1→2: File Parsing + View Creation**

- **Input**: `&Path` to property bank file
- **Output**: `RawPropertyBank` (validated) + `RawPropertyBankView`
- **Operations**:
  - Read file with `FsReader::read_with()`
  - Parse via `FsReader::parse_structured_from_str()` (custom `RawPropertyMap<T>` deserializer)
  - Property names validated during deserialization
  - Version validated separately
  - Create `RawPropertyBankView::try_from_with_content()` with all new metadata
- **Note**: Same as NEW path - view created in FileParsed state

**State 2→3: Property Delta Computation**

- **Input**: `RawPropertyBank` (new), `RawPropertyBankView` (cached)
- **Output**: Delta information (new/modified/removed properties)
- **Operations**:
  - Compute per-property hashes for new `RawPropertyBank.properties`
  - Compare with `cached_view.hashes().property_hashes()`
  - Identify:
    - **New properties**: In new but not in cached
    - **Modified properties**: In both but hash differs
    - **Removed properties**: In cached but not in new
- **Location**: `Ingestor::ingest_stale_property_bank()` lines 580-588

**State 3→4: Base Construction (from DB)**

- **Input**: None
- **Output**: `PropertyBank` (base state before updates)
- **Operations**: `Repository::get_property_bank()` - fetch current bank

**State 4→5: Delta Application + View Update**

- **Input**: `PropertyBank` (base), Delta info, `RawPropertyBank` (new), `RawPropertyBankView` (new)
- **Output**: `PropertyBank` (updated) + `RawPropertyBankView` (updated)
- **Operations**:
  - `PropertyBank::update_from_raw(&raw, &changed_properties)`
  - For each changed property:
    - If in new raw: update or add property
    - If not in new raw: remove property
  - Increment `BankVersion` if any changes applied
  - View already created in FileParsed state (carry forward)
- **Errors**: `SchemaError::PropertyBank`
- **Location**: `bank.rs:255-300`
- **Note**: View update happens in DeltaApplied state (no separate ViewUpdated state)

**State 5→6: Persist Both & Complete**

- **Operations**:
  - `Repository::save_property_bank(&updated_bank)`
  - `Repository::save_raw_property_bank_view(&new_view)`

### 1.3 Staleness Detection Details

**Four Distinct Paths** based on view existence and content comparison:

| Path               | Trigger                         | File I/O         | Parsing             | DB Reads  | DB Writes   | Notes                  |
| ------------------ | ------------------------------- | ---------------- | ------------------- | --------- | ----------- | ---------------------- |
| **NEW**            | No view in DB                   | ✅ Full read     | ✅ Parse + validate | -         | Bank + View | First time seeing file |
| **FreshTimestamp** | Timestamps match                | ❌ None          | ❌ None             | Bank only | -           | Fastest path (cached)  |
| **FreshContent**   | Hash matches, timestamps differ | ✅ Read for hash | ❌ None             | Bank only | View only   | Clock skew handling    |
| **STALE**          | Hash differs                    | ✅ Full read     | ✅ Parse + validate | Bank only | Bank + View | Incremental update     |

**Two-Tier Staleness Check** (fast path → slow path):

1. **Fast Path** (no file I/O): Compare timestamps
   - `view.file_times().is_timestamp_match(created_at, modified_at)`
   - If match → FreshTimestamp path
   - If mismatch → proceed to slow path

2. **Slow Path** (single file read): Compare content hash
   - Read file content with `FsReader::read_with()`
   - Compute `blake3::hash(content.as_bytes())`
   - `view.hashes().is_content_match(content_hash)`
   - If match → FreshContent path
   - If mismatch → STALE path

**Key Optimization**: Content hash check prevents unnecessary re-parsing when timestamps change but content is identical (e.g., file copied, touched, or clock skew).

**Location**: `Ingestor::property_bank()` lines 473-510

---

### 1.4 Prerequisite Refactoring: "Parse, Don't Validate"

**Problem**: Current `RawPropertyBank` and `RawSchema` violate the "parse, don't validate" principle:

```rust
// Current anti-pattern: validation is optional
let raw: RawPropertyBank = serde_json::from_str(content)?;  // Can create invalid instance!
raw.validate()?;  // Caller might forget this step
```

**Issues**:

- ❌ Can construct invalid `Raw*` instances
- ❌ Validation is optional (caller can forget to call it)
- ❌ Type system doesn't enforce validity
- ❌ Property keys are `Box<str>` (not validated `PropertyName`)
- ❌ Separate "Validated" state needed in state machine

**Solution**: Transform validation into parsing (make invalid states unrepresentable):

```rust
// Better: Validation happens during deserialization
let raw: RawPropertyBank = FsReader::parse_structured_from_str(path, content)?;
// Properties map keys are guaranteed valid PropertyName instances!
// Version validation happens separately (needs path context for errors)
raw.validate_version(&path.to_string_lossy())?;
```

#### **Refactoring Plan**

**Step 1: Create `RawPropertyMap<T>` Wrapper Type**

This provides a **single deserialization point** for property maps, ensuring consistent validation across both `RawPropertyBank` and `RawSchema`.

```rust
use serde::{Deserialize, Deserializer};
use std::collections::HashMap;

/// Validated property map that guarantees all keys are valid PropertyNames.
///
/// This type provides custom deserialization that validates property names
/// during parsing, making invalid states unrepresentable.
#[derive(Debug, Clone, PartialEq)]
pub struct RawPropertyMap<T> {
    inner: HashMap<PropertyName, T>,
}

impl<T> RawPropertyMap<T> {
    /// Get a reference to the inner map.
    #[inline]
    #[must_use]
    pub fn as_map(&self) -> &HashMap<PropertyName, T> {
        &self.inner
    }

    /// Consume and return the inner map.
    #[inline]
    #[must_use]
    pub fn into_map(self) -> HashMap<PropertyName, T> {
        self.inner
    }

    /// Get a property by name.
    #[inline]
    #[must_use]
    pub fn get(&self, name: &PropertyName) -> Option<&T> {
        self.inner.get(name)
    }

    /// Iterate over property entries.
    #[inline]
    pub fn iter(&self) -> impl Iterator<Item = (&PropertyName, &T)> {
        self.inner.iter()
    }
}

// Custom Deserialize implementation validates keys during parsing
impl<'de, T> Deserialize<'de> for RawPropertyMap<T>
where
    T: Deserialize<'de>,
{
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        // Deserialize as HashMap<Box<str>, T>
        let raw_map: HashMap<Box<str>, T> = HashMap::deserialize(deserializer)?;

        // Validate all keys and convert to PropertyName
        let inner: HashMap<PropertyName, T> = raw_map
            .into_iter()
            .map(|(k, v)| {
                PropertyName::try_new(&k)
                    .map(|name| (name, v))
                    .map_err(serde::de::Error::custom)
            })
            .collect::<Result<_, _>>()?;

        Ok(RawPropertyMap { inner })
    }
}
```

**Step 1.5: Create `RawPropertyRefPath` Wrapper Type**

Replace the raw `Box<str>` in `RawPropertyRef` with a dedicated type that parses the `$ref` path and ensures it points to a property bank property.

```rust
/// Validated reference path to a property bank entry.
///
/// Ensures the path is properly formatted (e.g., `#property_bank/name`)
/// and allows O(1) extraction of the target property name.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct RawPropertyRefPath {
    full_path: Box<str>,
    target_name: PropertyName,
}

impl RawPropertyRefPath {
    pub fn target_name(&self) -> &PropertyName {
        &self.target_name
    }

    pub fn as_str(&self) -> &str {
        &self.full_path
    }
}

// Custom Deserialize to validate format and extract target name
impl<'de> Deserialize<'de> for RawPropertyRefPath {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let path = String::deserialize(deserializer)?;

        let target = path
            .strip_prefix("#property_bank/")
            .ok_or_else(|| serde::de::Error::custom("Must start with '#property_bank/'"))?;

        let target_name = PropertyName::try_new(target)
            .map_err(serde::de::Error::custom)?;

        Ok(RawPropertyRefPath {
            full_path: path.into_boxed_str(),
            target_name,
        })
    }
}

// Then update RawPropertyRef
pub struct RawPropertyRef {
    #[serde(rename = "$ref")]
    pub ref_path: RawPropertyRefPath, // Guarantees valid format & target name
    // ... overrides
}
```

**Step 1.6: Construct `bank_references` in `SchemaVersion::new`**

Add a `bank_references: HashMap<PropertyName, PropertyName>` field to `SchemaVersion`. We construct this seamlessly during instantiation by filtering `raw.properties` for `RawPropertyRef` and grabbing the already-validated `target_name`:

```rust
// Inside SchemaVersion::new(..., raw: &RawSchema)
let mut bank_references = HashMap::new();

for (prop_name, raw_prop) in raw.properties() {
    if let RawProperty::Ref(ref_entry) = raw_prop {
        // prop_name is already a valid PropertyName
        // ref_entry.ref_path.target_name() is already a valid PropertyName
        bank_references.insert(
            prop_name.clone(),
            ref_entry.ref_path.target_name().clone()
        );
    }
}

Ok(Self {
    // ...
    bank_references,
    expanded_properties: None,
})
```

**Step 2: Update `RawPropertyBank` to Use `RawPropertyMap<T>`**

```rust
#[derive(Debug, Clone, PartialEq, serde::Deserialize)]
pub struct RawPropertyBank {
    /// Property bank format version (validated separately)
    #[serde(rename = "$version")]
    version: RawSchemaVersion,  // PRIVATE

    /// Validated property map (keys are guaranteed valid PropertyNames)
    properties: RawPropertyMap<property::RawPropertyBankEntry>,  // PRIVATE

    /// File metadata
    #[serde(skip)]
    metadata: RawPropertyBankMetadata,
}

impl RawPropertyBank {
    /// Validate the version field.
    ///
    /// This is separate from property key validation because version
    /// errors need path context for better error messages.
    pub fn validate_version(&self, path: &str) -> Result<(), SchemaIngestionError> {
        self.version.validate(path)
    }

    /// Returns the schema version.
    #[inline]
    #[must_use]
    pub fn version(&self) -> &RawSchemaVersion {
        &self.version
    }

    /// Returns the properties map.
    #[inline]
    #[must_use]
    pub fn properties(&self) -> &HashMap<PropertyName, property::RawPropertyBankEntry> {
        self.properties.as_map()
    }

    /// Returns the metadata.
    #[inline]
    #[must_use]
    pub fn metadata(&self) -> &RawPropertyBankMetadata {
        &self.metadata
    }
}
```

**Step 3: Update `RawSchema` to Use `RawPropertyMap<T>`**

```rust
#[derive(Debug, Clone, PartialEq, serde::Deserialize)]
pub struct RawSchema {
    /// Schema format version
    #[serde(rename = "$version", default)]
    version: RawSchemaVersion,

    /// Schema name (set by Ingestor from filename)
    #[serde(skip)]
    name: Box<str>,

    /// Optional parent schema name for inheritance
    extends: Option<Box<str>>,

    /// Property names to exclude from parent
    #[serde(default)]
    excludes: Vec<Box<str>>,

    /// Validated property map (keys are guaranteed valid PropertyNames)
    properties: RawPropertyMap<property::RawProperty>,  // PRIVATE

    /// File metadata
    #[serde(skip)]
    metadata: RawSchemaMetadata,
}

impl RawSchema {
    /// Validate the version field.
    pub fn validate_version(&self, path: &str) -> Result<(), SchemaIngestionError> {
        self.version.validate(path)
    }

    /// Returns the properties map.
    #[inline]
    #[must_use]
    pub fn properties(&self) -> &HashMap<PropertyName, property::RawProperty> {
        self.properties.as_map()
    }
}
```

**Step 4: Remove Old Validation Methods**

Delete from `RawPropertyBank` and `RawSchema`:

- `validate()` - property key validation now happens in `RawPropertyMap` deserializer
- `validated()` - no longer needed (parsing = validation)

Keep:

- `validate_version()` - still needed (requires path context for errors)

**Step 5: Update Ingestor**

```rust
// In Ingestor - FsReader handles format detection and parsing
self.source.read_with(path, |path, content| {
    // Deserialize with automatic property name validation
    let raw: RawPropertyBank = FsReader::parse_structured_from_str(path, content)?;

    // Separate version validation (needs path for error context)
    raw.validate_version(&path.to_string_lossy())?;

    // Properties map keys are guaranteed to be valid PropertyNames!
    // Can safely iterate without try_new() checks
    for (prop_name, entry) in raw.properties() {
        // prop_name is already PropertyName type
    }

    Ok(raw)
})
```

#### **Benefits**

| Before                                       | After                                            |
| -------------------------------------------- | ------------------------------------------------ |
| Can create invalid `RawPropertyBank`         | Type guarantees validity                         |
| Validation is optional                       | Validation is mandatory (during deserialization) |
| Property keys are `Box<str>` (unvalidated)   | Property keys are `PropertyName` (validated)     |
| Two-step process (parse, then validate)      | One-step process (parsing = validation)          |
| Separate "Validated" state in state machine  | No separate state needed                         |
| Public fields (can mutate to invalid state)  | Private fields (immutable after construction)    |
| Duplicate validation logic in both Raw types | Single `RawPropertyMap<T>` validates both        |

**Impact on State Machine**:

- **Eliminates "Validated" state** (parsing = validation)
- **Eliminates "ViewUpdated" state** (view creation in FileParsed/DeltaApplied)
- **6 states instead of 7** (more cohesive groupings)
- **FileParsed state now guarantees validity** (properties map keys are `PropertyName`)
- **Type-level guarantee**: `RawPropertyMap<T>` cannot have invalid keys
- Cleaner state transitions

**Key Innovation - `RawPropertyMap<T>`**:

- ✅ Single source of truth for property key validation
- ✅ Reusable across both `RawPropertyBank` and `RawSchema`
- ✅ Custom deserializer validates during parsing (fail fast)
- ✅ Type safety: `HashMap<PropertyName, T>` not `HashMap<Box<str>, T>`
- ✅ Idiomatic Rust: `impl<'de> Deserialize<'de>` pattern

**Estimated Effort**: 3-4 hours

**Files to Modify**:

- `lithos-core/src/schema/raw/mod.rs` - Add `RawPropertyMap<T>`, update both Raw types
- `lithos-core/src/schema/ingestor.rs` - Update to use `validate_version()` only
- `lithos-core/src/schema/bank.rs` - Update `try_from` to work with `PropertyName` keys
- Tests in all files

---

## 2. Schema Pipeline Stages

### 2.1 State Identification

The Schema pipeline is **complex and branching**, tracking both its own file staleness and the upstream `PropertyBank` staleness. The first phase of the pipeline (through reference expansion) consists of 7 states:

```
┌───────────────────────────────────────────────────────────────────────────┐
│                                 Discovery                                 │
│                 (Query DB for RawSchemaView by filename)                  │
│   (Determine SchemaPipelinePath: NEW/FreshTimestamp/FreshContent/STALE)   │
└───────┬────────────────────────────┬──────────────┬──────────────────┬────┘
        │                            │              │                  │
        ▼                            ▼              ▼                  ▼
    ┌───────┐                 ┌──────────────┐ ┌────────────┐      ┌───────┐
    │  NEW  │                 │FreshTimestamp│ │FreshContent│      │ STALE │
    └───┬───┘                 └──────┬───────┘ └──────┬─────┘      └───┬───┘
        │                            │                │                │
        ▼                            ├─(If PB STALE)─┐│                ▼
  ┌───────────┐                      │               ││          ┌───────────┐
  │FileParsed │                      │               ▼│          │FileParsed │
  └─────┬─────┘                      │      ┌────────────────┐   └─────┬─────┘
        │                            │      │ RawConstructed │         │
        │                            │      │   (from DB)    │         ▼
        │                            │      └────────┬───────┘   ┌───────────────────┐
        │                            │               │           │SchemaPropertyDelta│
        │                            │               │           └────────┬──────────┘
        ▼                            │               │                    ▼
 ┌──────────────┐                    │               │    ┌──────────────┐ ┌────────────────┐
 │RawConstructed│                    │               │    │RawConstructed│ │ RawConstructed │
 │(from scratch)│                    │               │    │ (+upd times) │ │   (from DB)    │
 └──────┬───────┘                    │               │    └──────┬───────┘ └────────┬───────┘
        │                            │               │           │                  │
        │                            │               ▼           ▼                  ▼
        │                            │         ┌────────────────────────┐           │
        │                            │         │   BankReferenceDelta   │◀(PB STAL)─┤
        │                            │         └─────────────┬──────────┘           │
        │                            │                       │                      ▼
        │                            │                       │                ┌────────────┐
        │                            │                       │                │DeltaApplied│
        │                            │                       │                └─────┬──────┘
        │                            │                       │                      │
        │  ◀─────────────────────────┴────────(If PB FRESH)──┴───────(If PB FRESH)  │
        ▼                                                                           ▼
┌───────────────────────────────────────────────────────────────────────────────────────┐
│                                 InheritanceEvaluated                                  │
│                (Verify extends, track extends/excludes delta changes)                 │
└──────────────────────────────────────────┬────────────────────────────────────────────┘
                                           │
                                           ▼
┌───────────────────────────────────────────────────────────────────────────────────────┐
│                                     RefsExpanded                                      │
│                 (Full, Partial, or Zero expansion depending on path)                  │
└──────────────────────────────────────────┬────────────────────────────────────────────┘
                                           │
                                           ▼
                              (Proceeds to Tree Building)
```

**State Details (Phase 1: Discovery to Expansion)**:

| State                       | Data Structure                    | Used By Paths                        | Notes                                                                                                                             |
| --------------------------- | --------------------------------- | ------------------------------------ | --------------------------------------------------------------------------------------------------------------------------------- |
| **1. Discovery**            | `PropertyBankPath`, Config        | All                                  | Batches files, queries DB, tracks deleted schemas, builds initial `name_to_id`/`id_to_name` maps, determines `SchemaPipelinePath` |
| **2. FileParsed**           | `RawSchema`, metadata             | NEW, STALE                           | Parses file, validates, builds `bank_references` map                                                                              |
| **3. SchemaPropertyDelta**  | Schema delta info                 | STALE                                | Finds new/modified/removed properties in schema file                                                                              |
| **4. RawConstructed**       | `RawSchemaView`, `RawPropertyMap` | All                                  | Fetches baseline from DB or builds from scratch                                                                                   |
| **5. BankReferenceDelta**   | PB refs to re-expand              | FRESH\*(+PB STALE), STALE(+PB STALE) | Intersects `bank_references` with PB's `PropertyDelta`                                                                            |
| **6. DeltaApplied**         | `RawSchema` (updated)             | STALE                                | Applies schema file changes to baseline                                                                                           |
| **7. InheritanceEvaluated** | `extends`/`excludes` delta        | All                                  | Verifies `extends` validity, tracks structural changes                                                                            |
| **8. IndexesEvaluated**     | `name_to_id`, `id_to_name`        | All                                  | Injects NEW schemas into index maps                                                                                               |
| **9. RefsExpanded**         | `RefExpandedSchema`               | All                                  | Expands refs, constructs `SchemaVersion`, persists view                                                                           |

### 2.2 Phase 1: File Discovery to Expansion

The action taken on each schema depends on a combination of **its own staleness** AND the **Property Bank's staleness**. The pipeline branches into 5 distinct flows.

#### **State 1: Discovery** (All Paths)

- **Input:** `PropertyBankPath` (with `PropertyDelta` if PB is STALE), Config schema dir
- **Operations:**
  - Scan directory (excluding `property_bank`).
  - Fetch `RawSchemaView`s from DB for all files (via `SCHEMA_ID_BY_PATH` -> `SchemaId` -> `RAW_SCHEMA_VIEWS`).
  - **Track Deleted Schemas**: Identify schemas that exist in DB but not on filesystem.
  - **Initialize Index Maps**: Create the baseline `name_to_id` and `id_to_name` maps using the queried `SchemaId`s and filenames (minus any deleted schemas).
  - Determine `SchemaPipelinePath` (`New`, `FreshTimestamp`, `FreshContent`, `Stale`) for each file via timestamp and content hash checks.
- **Super-Fast Path Output**:
  - If **Property Bank is Fresh\*** AND **all schema files are Fresh\*** AND **no schemas are deleted**:
    - Call `RawSchemaView::update_from_timestamps` for any `FreshContent` schemas and persist view.
    - Go straight to retrieving full `Schema` through `SCHEMA_BY_ID`.
    - **Skip Phase 1 and 2 entirely!**
- **Standard Output:** Branches into one of the following 5 flows.

---

#### **Flow A: Schema is NEW**

_No cached view exists. Must do full expansion._

- **State 2 (FileParsed):**
  - Read file, parse into `RawSchema`.
  - Extract metadata, compute content hash.
- **State 4 (RawConstructed):**
  - Build `RawSchemaView` from scratch using the parsed `RawSchema`.
  - Compute `bank_references` (`HashMap<PropertyName, PropertyName>`) inside `SchemaVersion::new` by iterating over `RawSchema.properties` and extracting the `target_name` from any `RawPropertyRefPath`.
- **State 7 (InheritanceEvaluated):**
  - Verify `extends` SchemaName refers to an actual schema.
- **State 8 (RefsExpanded):**
  - Do full reference expansion on all properties against the `PropertyBank`.
  - Construct final `SchemaVersion` embedding `expanded_properties` and `bank_references`.
  - Inject newly generated `SchemaId` and schema name into the `name_to_id` and `id_to_name` index maps.
  - Update `RawSchemaView` and persist it.

---

#### **Flow B: Schema is Fresh* AND Property Bank is Fresh***

_Zero changes. Skip parsing entirely._

- **State 7 (InheritanceEvaluated):**
  - If any schemas were deleted globally, re-verify `extends` SchemaName in `SchemaVersion` still refers to an actual schema.
- **State 8 (RefsExpanded):**
  - Instantly construct `RefExpandedSchema` using top-level metadata from the `RawSchemaView` (`name`, `extends`, `excludes`) and its cached `expanded_properties`.
  - _No `RawSchema` deserialization occurs!_
  - No DB persistence needed.

---

#### **Flow C: Schema is Fresh\* AND Property Bank is STALE**

_Schema file didn't change, but some of its PB references might have._

- **State 4 (RawConstructed):**
  - Fetch cached `RawSchemaView`.
  - If `FreshContent`, update view timestamps.
- **State 5 (BankReferenceDelta):**
  - Intersect the view's `bank_references` map with the PB's `PropertyDelta.changed` list.
  - Output: specific schema property names needing re-expansion.
  - _If intersection is empty, jump straight to InheritanceEvaluated._
- **State 7 (InheritanceEvaluated):**
  - If any schemas were deleted globally, re-verify `extends` SchemaName still refers to an actual schema.
- **State 8 (RefsExpanded):**
  - Deserialize `raw_properties` from JSON bytes via `RawPropertyMap` (only because we must re-expand).
  - Re-run expansion _only_ on the affected properties identified in State 5.
  - Update those specific keys in the cached `expanded_properties`.
  - Construct new `SchemaVersion`, update view, and persist.

---

#### **Flow D: Schema is STALE AND Property Bank is Fresh\***

_Schema changed, PB didn't. Only expand schema changes._

- **State 2 (FileParsed):**
  - Read new file, parse to `RawSchema`.
- **State 3 (SchemaPropertyDelta):**
  - Compare new property hashes against cached view to find new/modified/removed schema properties.
  - _Note: Only operates on deserialized `raw_properties` from `SchemaVersion`, no need to reconstruct full baseline `RawSchema`._
- **State 4 (RawConstructed):**
  - Fetch cached `RawSchemaView` as baseline.
- **State 6 (DeltaApplied):**
  - Apply `SchemaPropertyDelta` to baseline raw properties.
  - Build new `RawSchemaView` from the updated `RawSchema`, computing the new `bank_references` map.
- **State 7 (InheritanceEvaluated):**
  - Compare `extends` SchemaName with `SchemaVersion`. Track if changed (affects inheritance tree).
  - Verify `extends` refers to an actual schema.
  - Compare `excludes` Vec<PropertyName> with `SchemaVersion`. Track if changed (affects property merging).
- **State 8 (RefsExpanded):**
  - Expand only the NEW or MODIFIED properties from State 3.
  - Construct final `SchemaVersion` with updated `expanded_properties`, update view, and persist.

---

#### **Flow E: Schema is STALE AND Property Bank is STALE**

_Both changed. Expand schema changes PLUS affected PB references._

- **State 2 (FileParsed):** Parse new file.
- **State 3 (SchemaPropertyDelta):** Compute schema file changes (using deserialized `raw_properties` from view).
- **State 4 (RawConstructed):** Fetch cached `RawSchemaView` baseline.
- **State 5 (BankReferenceDelta):** Intersect _cached_ `bank_references` with PB's `PropertyDelta`.
- **State 6 (DeltaApplied):**
  - Apply `SchemaPropertyDelta` to baseline.
  - Build new `RawSchemaView` from the updated `RawSchema`, computing the new `bank_references` map.
- **State 7 (InheritanceEvaluated):**
  - Compare `extends` SchemaName with `SchemaVersion`. Track if changed.
  - Verify `extends` refers to an actual schema.
  - Compare `excludes` Vec<PropertyName> with `SchemaVersion`. Track if changed.
- **State 8 (RefsExpanded):**
  - Expand NEW/MODIFIED properties from State 3.
  - Re-expand unmodified schema properties flagged by State 5.
  - Construct final `SchemaVersion`, update view, persist.

---

### 2.3 Phase 2: Tree Building, Merging, and Persistence

After Phase 1 completes (Discovery → RefsExpanded), we have a collection of schemas with expanded properties. Phase 2 builds the inheritance tree, merges properties, and persists the results.

**Phase 2 States**:

- State 9: `TreeConstructed` (Extender builds topological order)
- State 10: `PropertiesMerged` (Merger applies inheritance)
- State 11: `Persisted` (Save to database)

#### State 9: Tree Construction (Extender)

**Input**:

- `Vec<(SchemaId, RefExpandedSchema)>` (stale schemas)
- `HashMap<SchemaId, Schema>` (known_parents, from DB)
- `name_to_id: HashMap<Box<str>, SchemaId>` (from Phase 1 Discovery & RefsExpanded)
- `id_to_name: HashMap<SchemaId, Box<str>>` (from Phase 1 Discovery & RefsExpanded)

**Output**: `SchemaTree` (topologically ordered)

**Operations** (formerly 6 phases in `Extender::build()`, now 5 phases):

_(Note: Phase 1 "Build name indexes" is now handled organically during Phase 1 of the pipeline. Baseline is loaded in Discovery, and NEW schemas are injected during RefsExpanded, preventing duplicate iteration.)_

**Phase 2**: Build node map

- For each `RefExpandedSchema`:
  - Resolve `extends` name → parent `SchemaId` via `name_to_id`
  - Create `SchemaNode` with:
    - `name`, `properties`, `excludes`
    - `parent_id` (resolved)
    - `children: Vec::new()` (populated in Phase 4)
    - `depth: NodeDepth::root()` (computed in Phase 5)

**Phase 3**: DFS cycle detection

- For each node with `parent_id`:
  - Walk up the parent chain
  - Track visited IDs in `HashSet`
  - If loop detected → `SchemaError::Inheritance(CircularInheritance)`

**Phase 4**: Populate children lists

- Reverse parent→child relationships
- For each node with `parent_id`:
  - Add `child_id` to `parent.children`

**Phase 5**: Compute inheritance depths

- BFS traversal starting from roots
- Root depth = 1
- Child depth = parent depth + 1
- Accounts for both in-batch parents and known_parents

**Phase 6**: Kahn's topological ordering

- Initialize: roots have in-degree 0
- Queue roots, process children when all parents visited
- Result: `Vec<SchemaId>` in topological order (parents before children)

**Errors**:

- `SchemaError::Resolution(DuplicateSchemaName)`
- `SchemaError::Inheritance(CircularInheritance)`
- `SchemaError::Inheritance(ParentNotFound)`

**Location**: `Extender::build()` in `extender.rs:222-250`

**State Transition**:

```rust
impl SchemaPipeline<RefsExpanded> {
    pub fn build_tree(
        schemas: Vec<SchemaPipeline<RefsExpanded>>,
        known_parents: &HashMap<SchemaId, Schema>,
    ) -> Result<SchemaPipeline<TreeConstructed>, SchemaError> {
        let expanded: Vec<(SchemaId, RefExpandedSchema)> = schemas
            .into_iter()
            .map(|s| s.into_expanded_schema())
            .collect();

        let tree = Extender::build(expanded, known_parents)?;

        Ok(SchemaPipeline {
            data: Box::new(SchemaTreeData { tree }),
            _state: PhantomData,
        })
    }
}
```

#### State 10: Property Merging (Merger)

**Input**:

- `SchemaTree` (topological order)
- `HashMap<SchemaId, Schema>` (known_parents)

**Output**: `Vec<Schema>` (fully resolved)

**Operations** (single linear walk in `Merger::resolve()` lines 69-152):

- Walk tree in topological order (parents before children)
- For each node:
  1. **Depth check**: Verify `node.depth <= INHERITANCE_MAX_DEPTH` (10)
  2. **Get parent properties**:
     - If `node.parent_id.is_some()`:
       - Lookup in `resolved_cache` (in-batch parent, already processed)
       - OR lookup in `known_parents` (DB-fresh parent)
     - If `None` (root schema): use empty `HashMap`
  3. **Merge properties**:
     - Start with child's own properties (child overrides)
     - Add parent properties NOT in excludes list and NOT already in child
     - Result: `HashMap<PropertyName, Property>`
  4. **Construct Schema**:
     - `Schema::new(id, name, parent_id, children, merged_properties)`
  5. **Cache result**: Store in `resolved_cache` for downstream children
  6. **Add to results**: Append to output vector

**Key Merging Rules**:

- Child property with same name **completely replaces** parent property
- Parent properties in `excludes` list are **not inherited**
- All other parent properties are **inherited**

**Errors**:

- `SchemaError::Inheritance(DepthExceeded)`
- `SchemaError::Resolution(MissingNode)`

**Location**: `Merger::resolve()` in `merger.rs:69-152`

**State Transition**:

```rust
impl SchemaPipeline<TreeConstructed> {
    pub fn merge_properties(
        self,
        known_parents: &HashMap<SchemaId, Schema>,
    ) -> Result<SchemaPipeline<PropertiesMerged>, SchemaError> {
        let tree = self.data.tree;
        let schemas = Merger::resolve(&tree, known_parents)?;

        Ok(SchemaPipeline {
            data: Box::new(MergedSchemasData { schemas }),
            _state: PhantomData,
        })
    }
}
```

#### State 11: Persistence

**Input**: `Vec<Schema>` (resolved)
**Output**: Persisted to database

**Operations**:

1. `Repository::save_schemas()` - bulk save (lines 396-402)
2. `Repository::save_inheritance_metadata()` - cache inheritance views (lines 417-474)
3. `Repository::save_raw_schema_view()` - save staleness metadata (already done in Stage 2)

**Metadata Persisted** (for future staleness checks):

- `RawSchemaView` - timestamps, hashes, compressed content, expanded properties
- `SchemaInheritanceView` - parent, ancestors, excludes, ancestors_hash, resolved_at

**Errors**: `SchemaRepositoryError::Storage`
**Location**: `Loader::persist_resolved_schemas()`, `Loader::persist_inheritance_metadata()`

**State Transition**:

```rust
impl SchemaPipeline<PropertiesMerged> {
    pub fn persist(
        self,
        repo: &impl Repository,
    ) -> Result<(), SchemaRepositoryError> {
        // Bulk save schemas
        repo.save_schemas(&self.data.schemas)?;

        // Save inheritance metadata for each schema
        for schema in &self.data.schemas {
            let metadata = SchemaInheritanceView::from_schema(schema);
            repo.save_inheritance_metadata(schema.id(), &metadata)?;
        }

        // Update SCHEMA_DESCENDANTS multimap
        repo.update_descendants_index(&self.data.schemas)?;

        Ok(())
    }
}
```

### 2.4 Redesigned Inheritance Views

**Current Problem**: The existing views in `views/inheritance.rs` are not optimized for incremental tree rebuilding. We need efficient queries for:

1. Finding all descendants of a changed parent (BFS traversal)
2. Detecting transitive staleness via hash comparison
3. Reconstructing minimal subgraphs for re-resolution

**Redesigned Storage Schema (3 Tables)**:

```rust
// Table 1: SCHEMAS (Regular Table) - Already exists
// Key: SchemaId → Value: Schema (aggregate root)

// Table 2: SCHEMA_INHERITANCE (Regular Table)
// Key: SchemaId → Value: SchemaInheritanceView
#[derive(Debug, Clone, PartialEq, Archive, Serialize, Deserialize)]
pub struct SchemaInheritanceView {
    /// Immediate parent ID (None for roots)
    pub parent: Option<SchemaId>,

    /// Ordered ancestors: [parent, grandparent, ...] (empty for roots)
    pub ancestors: Vec<SchemaId>,

    /// Inheritance depth (1 for roots, parent.depth + 1 for children)
    /// Pre-computed during tree building, cached for efficient merging
    pub depth: usize,

    /// Recursive hash: hash(parent_id || parent.ancestors_hash)
    /// Enables O(1) transitive staleness detection
    pub ancestors_hash: u64,

    /// When this metadata was computed
    #[rkyv(with = AsUnixTime)]
    pub resolved_at: SystemTime,
}

// Table 3: SCHEMA_DESCENDANTS (Multimap)
// Key: ParentId → Value: Vec<SchemaId> (direct children only)
// Enables O(log N + C) lookup of all children for BFS traversal
```

**Key Changes from Current Design**:

1. ✅ Added `depth: usize` field (pre-computed, saves recalculation)
2. ✅ Removed `excludes` field (redundant - already in Schema aggregate)
3. ✅ Renamed `schema_children` → `SCHEMA_DESCENDANTS` (clearer purpose)
4. ✅ Simplified multimap values to just `Vec<SchemaId>` (lightweight)
5. ✅ Eliminated `ChildSchemaView` and `ParentSchemaView` structs (unnecessary)

**Efficient Query Patterns**:

```rust
// O(log N) - Check if schema needs re-merging
pub fn is_metadata_stale(
    repo: &impl Repository,
    schema_id: SchemaId,
) -> Result<bool, DbError> {
    let metadata = repo.get_inheritance_metadata(schema_id)?;

    if let Some(parent_id) = metadata.parent {
        let parent_metadata = repo.get_inheritance_metadata(parent_id)?;
        let expected_hash = SchemaInheritanceView::compute_hash(
            Some((parent_id, &parent_metadata))
        );

        Ok(metadata.ancestors_hash != expected_hash)
    } else {
        Ok(false)  // Roots never stale via inheritance
    }
}

// O(D×log N) - Find all descendants (BFS traversal)
pub fn find_all_descendants(
    repo: &impl Repository,
    root_id: SchemaId,
) -> Result<HashSet<SchemaId>, DbError> {
    let mut descendants = HashSet::new();
    let mut queue = VecDeque::from([root_id]);

    while let Some(id) = queue.pop_front() {
        let children = repo.get_descendants(id)?;  // Multimap lookup
        for child_id in children {
            if descendants.insert(child_id) {
                queue.push_back(child_id);
            }
        }
    }

    Ok(descendants)
}
```

**Incremental Update Workflow**:

1. **Detect structurally stale schemas** (from `InheritanceEvaluated` state):
   - Schemas where `extends` changed (old parent ≠ new parent)
   - Schemas where `excludes` changed (affects property merging)

2. **Find transitive descendants** (BFS using `SCHEMA_DESCENDANTS`):

   ```rust
   let mut all_stale = HashSet::new();
   for stale_schema in structurally_stale {
       all_stale.insert(stale_schema.id);
       all_stale.extend(find_all_descendants(repo, stale_schema.id)?);
   }
   ```

3. **Partition for optimal processing**:
   - **Structurally Stale + Descendants** → Full tree rebuild + merge
   - **Bank-Only Stale** (not in descendants) → Surgical property update via `Merger::resolve_affected_properties()`
   - **Fresh** → Skip entirely

4. **Build minimal subgraph**:
   - Pass only `all_stale` schemas to `Extender::build()`
   - Use fresh schemas as `known_parents` boundary
   - Result: Only re-merge O(S) schemas, not entire database

**Performance Analysis**:

| Vault Size | Stale Schemas | Descendants | Tree Build | Merge Time | Total |
| ---------- | ------------- | ----------- | ---------- | ---------- | ----- |
| 100        | 5             | 15          | 2ms        | 3ms        | 5ms   |
| 1,000      | 10            | 50          | 8ms        | 15ms       | 23ms  |
| 10,000     | 20            | 200         | 35ms       | 80ms       | 115ms |

**Compared to full rebuild** (current approach):

- 100 schemas: 5ms vs 15ms (3× faster)
- 1,000 schemas: 23ms vs 250ms (11× faster)
- 10,000 schemas: 115ms vs 3.5s (30× faster)

### 2.5 Cached Expansion Optimization

_(This concept is now natively handled by Flow C: Schema is Fresh AND PB is STALE, where we use the `BankReferenceDelta` to perform partial expansion.)_

---

## 3. Cross-Cutting Concerns

### 3.1 Staleness Detection Architecture

**Two-Tier Strategy**:

1. **Fast Path**: Timestamp comparison (no I/O)
2. **Slow Path**: Content hash comparison (single file read)

**Metadata Types**:

| Type                    | Purpose                | Storage          | Data                                                              |
| ----------------------- | ---------------------- | ---------------- | ----------------------------------------------------------------- |
| `RawPropertyBankView`   | PropertyBank staleness | Singleton table  | timestamps, content_hash, property_hashes, compressed_content     |
| `RawSchemaView`         | Schema staleness       | Per-schema table | timestamps, content_hash, compressed_content, expanded_properties |
| `SchemaInheritanceView` | Inheritance cache      | Per-schema table | parent, ancestors, excludes, ancestors_hash, resolved_at          |

**Timestamp Fields**:

- `created_at: Option<SystemTime>` - file creation time
- `modified_at: Option<SystemTime>` - file modification time

**Hash Fields**:

- `content_hash: [u8; 32]` - blake3 hash of file content
- `property_hashes: HashMap<PropertyName, [u8; 32]>` - per-property hashes (PropertyBank only)

**Comparison Logic** (in `views/metadata.rs`):

```rust
// Fast path: timestamp match
view.file_times().is_timestamp_match(created_at, modified_at)

// Slow path: hash match
view.hashes().is_content_match(content_hash)
```

### 3.2 Error Taxonomy (Hierarchical)

**Umbrella Errors** (top level):

- `SchemaError` - catch-all for domain errors
- `SchemaIngestionError` - file loading + parsing pipeline
- `SchemaRepositoryError` - database operations
- `SchemaLoaderError` - orchestration failures

**Pipeline Errors** (specific stages):

- `SchemaFileError` - file I/O (Stage 1→2)
- `SchemaParseError` - deserialization (Stage 2→3)
- `SchemaVersionError` - version validation (Stage 3→4)
- `SchemaSyntaxError` - syntax validation (Stage 3→4)
- `SchemaStorageError` - database persistence (Stage 5→6)

**Sub-Domain Errors** (property/schema level):

- `SchemaNameError` - schema name validation
- `PropertyNameError` - property name validation
- `PropertySpecError` - property spec validation
- `PropertyValueError` - property value validation
- `PropertyRefError` - bank reference resolution
- `PropertyBankError` - bank registration
- `SchemaInheritanceError` - inheritance logic
- `SchemaResolutionError` - schema resolution

### 3.3 Repository Interface

**Unified Repository Trait** (`schema::storage::Repository`):

**PropertyBank Operations**:

- `get_property_bank() -> Result<Option<PropertyBank>>`
- `save_property_bank(&PropertyBank) -> Result<()>`
- `get_raw_property_bank_view() -> Result<Option<RawPropertyBankView>>`
- `save_raw_property_bank_view(&RawPropertyBankView) -> Result<()>`

**Schema Operations**:

- `find_schema_by_name(&SchemaName) -> Result<Option<Schema>>`
- `find_schemas_by_ids(&[SchemaId]) -> Result<Vec<Schema>>`
- `find_schema_ids_by_paths(&[PathBuf]) -> Result<HashMap<PathBuf, SchemaId>>`
- `save_schemas(&[Schema]) -> Result<()>`

**View Operations**:

- `get_raw_schema_view(SchemaId) -> Result<Option<RawSchemaView>>`
- `find_raw_schema_view_by_path(&str) -> Result<Option<RawSchemaView>>`
- `find_raw_schema_views_by_paths(&[PathBuf]) -> Result<HashMap<PathBuf, RawSchemaView>>`
- `save_raw_schema_view(SchemaId, &RawSchemaView) -> Result<()>`

**Inheritance Operations**:

- `save_inheritance_metadata(SchemaId, &SchemaInheritanceView) -> Result<()>`

**Implementations**:

- `RedbRepository` - production (redb zero-copy database)
- `InMemoryRepository` - testing (HashMap-based)
- `FakeStorage` - mocking (configurable behavior)

---

## 4. State Machine Design Recommendations

### 4.1 PropertyBank State Machine

**Consolidated States** (6 total):

```rust
// State types (zero-sized markers)
struct Discovery;
struct FileParsed;
struct PropertyDelta;
struct BaseConstructed;
struct DeltaApplied;
struct Completed;

// Generic state machine
struct PropertyBankPipeline<S> {
    data: Box<PropertyBankData>,  // Shared data
    _state: PhantomData<S>,        // Zero-sized state marker
}

// Sealed state trait (prevents external state implementations)
mod sealed {
    pub trait Sealed {}
    impl Sealed for super::Discovery {}
    impl Sealed for super::FileParsed {}
    impl Sealed for super::PropertyDelta {}
    impl Sealed for super::BaseConstructed {}
    impl Sealed for super::DeltaApplied {}
    impl Sealed for super::Completed {}
}
pub trait PropertyBankState: sealed::Sealed {}
impl<T: sealed::Sealed> PropertyBankState for T {}
```

**State-Specific Operations**:

```rust
// Discovery: Query DB and determine branch
impl PropertyBankPipeline<Discovery> {
    pub fn new(path: &Path, repo: &Repository) -> Self {
        Self {
            data: Box::new(PropertyBankData {
                path: path.to_path_buf(),
                repo: repo.clone(),
                // ... other fields
            }),
            _state: PhantomData,
        }
    }

    pub fn discover(self) -> Result<PropertyBankPath, Error> {
        let view = self.data.repo.get_raw_property_bank_view()?;

        match view {
            None => Ok(PropertyBankPath::New(/* transition to FileParsed */)),
            Some(view) => {
                if timestamps_match(&view, &self.data.path)? {
                    Ok(PropertyBankPath::FreshTimestamp(/* transition to BaseConstructed */))
                } else {
                    let content = read_file(&self.data.path)?;
                    let hash = blake3::hash(content.as_bytes());

                    if view.hashes().is_content_match(hash.as_bytes()) {
                        Ok(PropertyBankPath::FreshContent(
                            /* transition to BaseConstructed */
                        ))
                    } else {
                        Ok(PropertyBankPath::Stale(/* transition to FileParsed */))
                    }
                }
            }
        }
    }
}

// FileParsed: File read, parsed, validated, and view created
impl PropertyBankPipeline<FileParsed> {
    pub fn parse_file(path: &Path, content: &str) -> Result<Self, Error> {
        // Parse with automatic property name validation (RawPropertyMap deserializer)
        let raw: RawPropertyBank = FsReader::parse_structured_from_str(path, content)?;

        // Separate version validation (needs path for error context)
        raw.validate_version(&path.to_string_lossy())?;

        // Create view immediately (all data available)
        let view = RawPropertyBankView::try_from_with_content(&raw, content)?;

        Ok(Self {
            data: Box::new(PropertyBankData {
                raw: Some(raw),
                view: Some(view),
                /* ... */
            }),
            _state: PhantomData,
        })
    }

    // NEW path: Build from scratch
    pub fn build_from_scratch(self) -> Result<PropertyBankPipeline<BaseConstructed>, Error> {
        let raw = self.data.raw.expect("raw exists in FileParsed");
        let bank = PropertyBank::try_from(raw)?;

        Ok(PropertyBankPipeline {
            data: Box::new(PropertyBankData {
                bank: Some(bank),
                view: self.data.view,  // Carry view forward
                /* ... */
            }),
            _state: PhantomData,
        })
    }

    // STALE path: Compute delta
    pub fn compute_delta(
        self,
        cached_view: &RawPropertyBankView
    ) -> Result<PropertyBankPipeline<PropertyDelta>, Error> {
        let new_hashes = compute_property_hashes(&self.data.raw);
        let delta = cached_view.hashes().compare(&new_hashes);

        Ok(PropertyBankPipeline {
            data: Box::new(PropertyBankData {
                raw: self.data.raw,
                view: self.data.view,  // Carry view forward
                delta: Some(delta),
                /* ... */
            }),
            _state: PhantomData,
        })
    }
}

// PropertyDelta: Delta computed (STALE path only)
impl PropertyBankPipeline<PropertyDelta> {
    pub fn fetch_base(self, repo: &Repository) -> Result<PropertyBankPipeline<BaseConstructed>, Error> {
        let bank = repo.get_property_bank()?.expect("bank exists for STALE");

        Ok(PropertyBankPipeline {
            data: Box::new(PropertyBankData { bank: Some(bank), /* ... */ }),
            _state: PhantomData,
        })
    }
}

// BaseConstructed: PropertyBank available
impl PropertyBankPipeline<BaseConstructed> {
    // STALE path: Apply delta (view already created in FileParsed)
    pub fn apply_delta(
        mut self,
        delta: &PropertyDelta
    ) -> Result<PropertyBankPipeline<DeltaApplied>, Error> {
        let bank = self.data.bank.as_mut().expect("bank exists");
        bank.update_from_raw(&self.data.raw, &delta.changed)?;

        Ok(PropertyBankPipeline {
            data: self.data,  // View carried forward from FileParsed
            _state: PhantomData,
        })
    }

    // FreshContent path: Update view timestamps only
    pub fn update_timestamps(mut self, new_timestamps: FileTimes) -> Result<PropertyBankPipeline<Completed>, Error> {
        let view = self.data.repo.get_raw_property_bank_view()?.unwrap();
        let mut updated_view = view.clone();
        updated_view.update_file_times(new_timestamps.created_at, new_timestamps.modified_at);
        self.data.view = Some(updated_view);

        // Persist view only (bank unchanged)
        Ok(self.persist(repo, /* bank_modified: */ false)?)
    }

    // NEW and FreshTimestamp paths: Persist (view already in data if NEW)
    pub fn persist(self, repo: &Repository, bank_modified: bool) -> Result<PropertyBankPipeline<Completed>, Error> {
        // Persist bank (if modified)
        if bank_modified {
            repo.save_property_bank(&self.data.bank.as_ref().unwrap())?;
        }

        // Persist view (if exists)
        if let Some(view) = &self.data.view {
            repo.save_raw_property_bank_view(view)?;
        }

        Ok(PropertyBankPipeline {
            data: self.data,
            _state: PhantomData,
        })
    }
}

// DeltaApplied: Incremental updates applied (view already created in FileParsed)
impl PropertyBankPipeline<DeltaApplied> {
    pub fn persist(self, repo: &Repository) -> Result<PropertyBankPipeline<Completed>, Error> {
        // Persist both bank and view (both modified)
        repo.save_property_bank(&self.data.bank.as_ref().unwrap())?;
        repo.save_raw_property_bank_view(&self.data.view.as_ref().unwrap())?;

        Ok(PropertyBankPipeline {
            data: self.data,
            _state: PhantomData,
        })
    }
}

// Completed: Terminal state
impl PropertyBankPipeline<Completed> {
    pub fn into_bank(self) -> PropertyBank {
        self.data.bank.expect("bank exists in Completed")
    }

    pub fn bank(&self) -> &PropertyBank {
        self.data.bank.as_ref().expect("bank exists in Completed")
    }
}
```

**Branching Enum** (returned from Discovery):

```rust
enum PropertyBankPath {
    New(PropertyBankPipeline<FileParsed>),
    FreshTimestamp(PropertyBankPipeline<BaseConstructed>),
    FreshContent(PropertyBankPipeline<BaseConstructed>),
    Stale(PropertyBankPipeline<FileParsed>),
}

impl PropertyBankPath {
    pub fn into_completed(self, repo: &Repository) -> Result<PropertyBankPipeline<Completed>, Error> {
        match self {
            PropertyBankPath::New(pipeline) => {
                pipeline
                    .build_from_scratch()?
                    .create_view()?
                    .persist(repo)
            }
            PropertyBankPath::FreshTimestamp(pipeline) => {
                // Already BaseConstructed, no persistence needed
                pipeline.persist_noop()
            }
            PropertyBankPath::FreshContent(pipeline) => {
                let view = repo.get_raw_property_bank_view()?.unwrap();
                pipeline
                    .update_timestamps(view)?
                    .persist(repo)
            }
            PropertyBankPath::Stale(pipeline) => {
                let cached_view = repo.get_raw_property_bank_view()?.unwrap();
                pipeline
                    .compute_delta(&cached_view)?
                    .fetch_base(repo)?
                    .apply_delta(&pipeline.data.delta)?
                    .update_view()?
                    .persist(repo)
            }
        }
    }
}
```

**Benefits**:

- ✅ Compile-time guarantee of correct ordering
- ✅ Branch-specific paths type-safe
- ✅ Clear state transitions (can't skip stages)
- ✅ Self-documenting API
- ✅ "Validated" state eliminated (parsing = validation)
- ✅ Error types can be state-specific

**Key Improvements Over Original**:

- **7 states instead of 10** (consolidated related operations)
- **No separate "Validated" state** (prerequisite refactoring eliminates it)
- **Branching explicit** via `PropertyBankPath` enum
- **Discovery state** encapsulates staleness detection

### 4.2 Schema State Machine

**All States (11 Total)**: Discovery → FileParsed → SchemaPropertyDelta → RawConstructed → BankReferenceDelta → DeltaApplied → InheritanceEvaluated → RefsExpanded → TreeConstructed → PropertiesMerged → Persisted

The schema pipeline models the complex leveled staleness paths using a unified state machine with explicit branching via the `SchemaPipelinePath` enum.

#### Delta Structures

```rust
/// Property-level changes in schema file
pub struct SchemaPropertyDelta {
    pub new: HashSet<PropertyName>,
    pub modified: HashSet<PropertyName>,
    pub removed: HashSet<PropertyName>,
}

/// Inheritance parent changes
pub struct ExtendsDelta {
    pub old_parent: Option<SchemaName>,
    pub new_parent: Option<SchemaName>,
}

impl ExtendsDelta {
    pub fn changed(&self) -> bool {
        self.old_parent != self.new_parent
    }
}

/// Excludes list changes
pub struct ExcludesDelta {
    pub added: Vec<PropertyName>,
    pub removed: Vec<PropertyName>,
}

impl ExcludesDelta {
    pub fn changed(&self) -> bool {
        !self.added.is_empty() || !self.removed.is_empty()
    }
}
```

#### State Markers and Data

```rust
// State markers (zero-sized)
struct Discovery;
struct FileParsed;
struct SchemaPropertyDelta;
struct RawConstructed;
struct BankReferenceDelta;
struct DeltaApplied;
struct InheritanceEvaluated;
struct RefsExpanded;
struct TreeConstructed;
struct PropertiesMerged;
struct Persisted;

// Schema data (evolves through states)
struct SchemaData {
    raw: Option<RawSchema>,
    view: Option<RawSchemaView>,

    // Deltas computed during pipeline
    property_delta: Option<SchemaPropertyDelta>,
    extends_delta: Option<ExtendsDelta>,
    excludes_delta: Option<ExcludesDelta>,
}

// Generic state machine
struct SchemaPipeline<S> {
    data: Box<SchemaData>,
    _state: PhantomData<S>,
}

// Sealed state trait
mod sealed {
    pub trait Sealed {}
    impl Sealed for super::Discovery {}
    impl Sealed for super::FileParsed {}
    impl Sealed for super::SchemaPropertyDelta {}
    impl Sealed for super::RawConstructed {}
    impl Sealed for super::BankReferenceDelta {}
    impl Sealed for super::DeltaApplied {}
    impl Sealed for super::InheritanceEvaluated {}
    impl Sealed for super::RefsExpanded {}
    impl Sealed for super::TreeConstructed {}
    impl Sealed for super::PropertiesMerged {}
    impl Sealed for super::Persisted {}
}
pub trait SchemaState: sealed::Sealed {}
impl<T: sealed::Sealed> SchemaState for T {}
```

**State-Specific Operations**:

```rust
impl SchemaPipeline<Discovery> {
    pub fn new(paths: Vec<PathBuf>, pb_path: PropertyBankPath, repo: &Repository) -> Self { /* ... */ }

    pub fn discover(self) -> Result<SchemaPipelineResult, Error> {
        // Fetch RawSchemaViews via SCHEMA_ID_BY_PATH
        // Detect deleted schemas
        // Initialize base name_to_id and id_to_name maps from DB query
        // If PB is Fresh, all schemas Fresh, and no deletions -> Super-Fast Path
        // Otherwise, return a mapped SchemaPipelinePath for each schema
    }
}

impl SchemaPipeline<FileParsed> {
    pub fn parse_file(path: &Path) -> Result<Self, Error> {
        // Parse into RawSchema
        // Build bank_references map from RawPropertyRefPaths
    }

    // NEW path jumps directly to RawConstructed
    pub fn into_raw_constructed(self) -> SchemaPipeline<RawConstructed> { /* ... */ }

    // STALE path computes delta
    pub fn compute_schema_delta(self, cached_view: &RawSchemaView)
        -> SchemaPipeline<SchemaPropertyDelta> { /* ... */ }
}

impl SchemaPipeline<SchemaPropertyDelta> {
    pub fn fetch_base(self, repo: &Repository) -> Result<SchemaPipeline<RawConstructed>, Error> {
        // STALE path fetches DB baseline
    }
}

impl SchemaPipeline<RawConstructed> {
    // FRESH schema with STALE PB -> check bank references
    pub fn compute_bank_delta(self, pb_delta: &PropertyDelta)
        -> SchemaPipeline<BankReferenceDelta> { /* ... */ }

    // STALE schema -> apply schema delta
    pub fn apply_schema_delta(self) -> SchemaPipeline<DeltaApplied> { /* ... */ }

    // NEW schema -> evaluate inheritance
    pub fn evaluate_inheritance(self) -> SchemaPipeline<InheritanceEvaluated> { /* ... */ }
}

impl SchemaPipeline<BankReferenceDelta> {
    pub fn evaluate_inheritance(self) -> SchemaPipeline<InheritanceEvaluated> {
        // Re-verify extends if any schemas were deleted globally
    }
}

impl SchemaPipeline<DeltaApplied> {
    pub fn evaluate_inheritance(
        self,
        cached_view: &RawSchemaView,
        name_to_id: &HashMap<SchemaName, SchemaId>,
    ) -> Result<SchemaPipeline<InheritanceEvaluated>, SchemaError> {
        let raw = self.data.raw.as_ref().expect("raw exists in DeltaApplied");
        let old_version = &cached_view.schema_version;

        // Compute extends delta
        let extends_delta = ExtendsDelta {
            old_parent: old_version.extends().cloned(),
            new_parent: raw.extends().cloned(),
        };

        // Verify new parent exists if specified
        if let Some(ref new_parent) = extends_delta.new_parent {
            if !name_to_id.contains_key(new_parent) {
                return Err(SchemaError::Inheritance(
                    SchemaInheritanceError::ParentNotFound {
                        name: new_parent.as_ref().into(),
                    }
                ));
            }
        }

        // Compute excludes delta
        let old_excludes: HashSet<&PropertyName> = old_version.excludes().iter().collect();
        let new_excludes: HashSet<&PropertyName> = raw.excludes().iter().collect();

        let excludes_delta = ExcludesDelta {
            added: new_excludes.difference(&old_excludes).map(|&n| n.clone()).collect(),
            removed: old_excludes.difference(&new_excludes).map(|&n| n.clone()).collect(),
        };

        Ok(SchemaPipeline {
            data: Box::new(SchemaData {
                raw: self.data.raw,
                view: self.data.view,
                property_delta: self.data.property_delta,
                extends_delta: Some(extends_delta),
                excludes_delta: Some(excludes_delta),
            }),
            _state: PhantomData,
        })
    }
}

impl SchemaPipeline<InheritanceEvaluated> {
    pub fn expand_refs(self, bank: &PropertyBank) -> Result<SchemaPipeline<RefsExpanded>, Error> {
        // Perform full, partial, or zero expansion based on upstream deltas
        // Construct SchemaVersion, update RawSchemaView, and persist
    }
}

impl SchemaPipeline<RefsExpanded> {
    pub fn into_expanded_schema(self) -> (SchemaId, RefExpandedSchema) {
        // Yields the finalized RefExpandedSchema. For NEW schemas, injects into index maps.
    }
}
```

**Branching Enum** (Returned from Discovery for each file):

```rust
enum SchemaPipelinePath {
    New(SchemaPipeline<FileParsed>),
    FreshTimestamp(SchemaPipeline<RawConstructed>),
    FreshContent(SchemaPipeline<RawConstructed>),
    Stale(SchemaPipeline<FileParsed>),
}

impl SchemaPipelinePath {
    pub fn into_expanded(
        self,
        repo: &Repository,
        pb_path: &PropertyBankPath
    ) -> Result<SchemaPipeline<RefsExpanded>, Error> {
        match self {
            SchemaPipelinePath::New(pipeline) => {
                pipeline
                    .into_raw_constructed()
                    .evaluate_inheritance()
                    .expand_refs(pb_path.bank())
            },
            SchemaPipelinePath::FreshTimestamp(pipeline) | SchemaPipelinePath::FreshContent(pipeline) => {
                if let Some(pb_delta) = pb_path.delta() {
                    // Flow C: Fresh Schema + STALE PB
                    pipeline
                        .compute_bank_delta(pb_delta)
                        .evaluate_inheritance()
                        .expand_refs(pb_path.bank())
                } else {
                    // Flow B: Fresh Schema + Fresh PB
                    pipeline
                        .evaluate_inheritance()
                        .expand_refs(pb_path.bank()) // Does zero-expansion internally
                }
            },
            SchemaPipelinePath::Stale(pipeline) => {
                let cached_view = repo.get_raw_schema_view()?.unwrap();
                pipeline
                    .compute_schema_delta(&cached_view)
                    .fetch_base(repo)?
                    .apply_schema_delta()
                    .evaluate_inheritance(&cached_view)
                    .expand_refs(pb_path.bank())
            }
        }
    }
}
```

### 4.3 Loader Orchestration with State Machines

**Before** (current code in `loader.rs:136-271`):

```rust
pub fn load(&self) -> Result<Vec<Schema>, SchemaLoaderError> {
    let results = self.ingestor.ingest_all()?;
    let bank = results.property_bank.bank();
    let bank_is_fresh = results.property_bank.is_fresh();

    // Complex partitioning logic (50 lines)
    let mut needs_expansion = Vec::new();
    let mut cached_expansion = Vec::new();
    let mut fresh_ids = Vec::new();
    // ... partitioning ...

    // Load known parents
    let parent_schemas = /* ... */;

    // Process needs_expansion
    if !needs_expansion.is_empty() {
        let expanded = RefExpander::new(bank).expand_all(needs_expansion.clone())?;
        self.store_expanded_properties(&expanded)?;
        let tree = Extender::build(expanded, &known_parents)?;
        let full_resolved = Merger::resolve(&tree, &known_parents)?;
        resolved.extend(full_resolved);
    }

    // Process cached_expansion
    if !cached_expansion.is_empty() {
        let cached_resolved = self.resolve_with_cached_expansion(/* ... */)?;
        resolved.extend(cached_resolved);
    }

    // Persist
    if !resolved.is_empty() {
        self.persist_resolved_schemas(&resolved)?;
        // ...
    }

    Ok(resolved)
}
```

**After** (with state machines):

```rust
pub fn load(&self) -> Result<Vec<Schema>, SchemaLoaderError> {
    // PropertyBank pipeline
    let bank_pipeline = PropertyBankPipeline::new(&bank_path)
        .read_file()?
        .parse()?
        .validate()?
        .to_domain()?
        .persist(&self.repository)?
        .complete();
    let bank = bank_pipeline.bank();

    // Schema pipeline
    let schema_pipeline = SchemaPipeline::new(file_list)
        .check_staleness(&self.repository)?
        .partition(bank_pipeline.is_fresh());

    // Branch based on partition results
    let branches = schema_pipeline.into_branches();

    // Process needs_expansion branch
    let expanded = SchemaPipeline::from_needs_expansion(branches.needs_expansion)
        .expand_refs(bank)?
        .build_tree(&known_parents)?
        .resolve(&known_parents)?
        .persist(&self.repository)?
        .complete();

    // Process cached_expansion branch
    let cached = SchemaPipeline::from_cached_expansion(branches.cached_expansion)
        .build_tree(&known_parents)?  // Skip expand_refs!
        .resolve(&known_parents)?
        .persist(&self.repository)?
        .complete();

    // Combine results
    let mut all_schemas = Vec::new();
    all_schemas.extend(expanded.schemas());
    all_schemas.extend(cached.schemas());

    Ok(all_schemas)
}
```

**Benefits**:

- **Clear state progression**: Each step is explicit
- **Compile-time safety**: Can't skip stages or call operations in wrong order
- **Self-documenting**: Type signatures show the pipeline
- **Testable**: Each state transition can be unit tested independently
- **Maintainable**: Adding new states/transitions is straightforward

---

## 5. Critical Implementation Details

### 5.1 Data Ownership in State Machines

**Architecture Decision**: Infrastructure (Config, FsReader, Repository) lives in the `Builder` facade and is passed by reference to state transitions. State machine data structs hold **only evolving artifacts**.

#### PropertyBankData Structure

```rust
struct PropertyBankData {
    // Evolving data - moves through states
    raw: Option<RawPropertyBank>,
    bank: Option<PropertyBank>,
    view: Option<RawPropertyBankView>,
    delta: PropertyDelta,  // Empty = no changes
}

struct PropertyBankPipeline<S> {
    data: Box<PropertyBankData>,
    _state: PhantomData<S>,
}
```

#### SchemaData Structure

```rust
struct SchemaData {
    // Evolving data - moves through states
    raw: Option<RawSchema>,
    view: Option<RawSchemaView>,  // Contains SchemaVersion

    // Deltas for incremental updates
    property_delta: Option<SchemaPropertyDelta>,
    extends_delta: Option<ExtendsDelta>,
    excludes_delta: Option<ExcludesDelta>,
}

struct SchemaPipeline<S> {
    data: Box<SchemaData>,
    _state: PhantomData<S>,
}
```

#### Builder Facade (Mutable)

```rust
pub struct Builder<'config, R> {
    config: &'config Config,
    source: FsReader,
    repository: R,

    // Mutable state: Set after PropertyBank pipeline completes
    property_delta: PropertyDelta,  // Empty = no changes
}

impl<'config, R: Repository> Builder<'config, R> {
    pub fn build(&mut self) -> Result<Vec<Schema>, SchemaLoaderError> {
        // 1. PropertyBank pipeline
        let bank_path = PropertyBankPath::discover(
            self.config.paths().property_bank_path(),
            &self.source,
            &self.repository,
        )?;

        let bank = bank_path.into_completed(&self.repository)?;

        // 2. Store property delta for schema pipeline
        self.property_delta = bank.delta().clone();

        // 3. Schema pipeline (accesses self.property_delta)
        let schemas = self.build_schemas(&bank)?;

        Ok(schemas)
    }
}
```

**Key Insight**: Infrastructure is immutable and passed by reference; only pipeline artifacts (parsed data, domain objects, deltas) are owned by state machines.

### 5.2 Error Handling in State Machines

**Decision**: Use existing `SchemaIngestionError`, `SchemaRepositoryError`, and `SchemaLoaderError` from `lithos-core/src/schema/error.rs`.

**No new error types needed** - the existing hierarchy covers all pipeline stages:

```rust
// File I/O and parsing
impl PropertyBankPipeline<Discovery> {
    pub fn discover(
        path: &Path,
        source: &impl FsReader,
        repo: &impl Repository,
    ) -> Result<PropertyBankPath, SchemaIngestionError> {
        // Uses SchemaFileError, SchemaParseError, SchemaStorageError
    }
}

// Domain validation
impl PropertyBankPipeline<BaseConstructed> {
    pub fn apply_delta(
        self,
        delta: &PropertyDelta,
    ) -> Result<PropertyBankPipeline<DeltaApplied>, SchemaError> {
        // Uses PropertyBankError, PropertySpecError
    }
}

// Orchestration
impl Builder {
    pub fn build(&mut self) -> Result<Vec<Schema>, SchemaLoaderError> {
        // Wraps all lower-level errors
    }
}
```

**Benefits**:

- ✅ Reuses existing error hierarchy (no duplication)
- ✅ Consistent with rest of codebase
- ✅ Preserves error context via `#[error(transparent)]`
- ✅ `From` conversions already implemented

### 5.3 Testing State Machines

**Unit Test Each Transition**:

```rust
#[test]
fn pipeline_read_file_success() {
    let temp = TempDir::new().unwrap();
    let path = temp.path().join("property_bank.json");
    std::fs::write(&path, r#"{"properties": {}}"#).unwrap();

    let pipeline = PropertyBankPipeline::<Unloaded>::new(&path);
    let result = pipeline.read_file();

    assert!(result.is_ok());
    // Can call methods on PropertyBankPipeline<RawFile>
}

#[test]
fn pipeline_read_file_failure() {
    let path = PathBuf::from("/nonexistent/path");
    let pipeline = PropertyBankPipeline::<Unloaded>::new(&path);
    let result = pipeline.read_file();

    assert!(result.is_err());
}
```

**Integration Test Full Pipeline**:

```rust
#[test]
fn property_bank_full_pipeline() {
    let temp = TempDir::new().unwrap();
    let path = temp.path().join("property_bank.json");
    std::fs::write(&path, r#"{"$version": "1.0", "properties": {"title": {"type": "string"}}}"#).unwrap();

    let repo = InMemoryRepository::new();
    let result = PropertyBankPipeline::<Unloaded>::new(&path)
        .read_file().unwrap()
        .parse().unwrap()
        .validate().unwrap()
        .to_domain().unwrap()
        .persist(&repo).unwrap()
        .complete();

    assert_eq!(result.bank().all().count(), 1);
}
```

### 5.4 Staleness Detection Integration

**Option 1**: Separate staleness state machine that produces initial state

```rust
enum PropertyBankInitialState {
    Fresh(PropertyBank),                        // Skip pipeline entirely
    NeedsProcessing(PropertyBankPipeline<Unloaded>),
}

fn detect_staleness(path: &Path, repo: &Repository)
    -> Result<PropertyBankInitialState, Error>
{
    let view = repo.get_raw_property_bank_view()?;

    match view {
        None => Ok(PropertyBankInitialState::NeedsProcessing(
            PropertyBankPipeline::new(path)
        )),
        Some(view) => {
            if view.is_fresh(path)? {
                let bank = repo.get_property_bank()?.unwrap();
                Ok(PropertyBankInitialState::Fresh(bank))
            } else {
                Ok(PropertyBankInitialState::NeedsProcessing(
                    PropertyBankPipeline::new(path)
                ))
            }
        }
    }
}
```

**Option 2**: Make staleness checking part of the state machine

```rust
impl PropertyBankPipeline<Unloaded> {
    pub fn check_staleness(self, repo: &Repository)
        -> Result<PropertyBankStaleness, Error>
    {
        // Return enum indicating Fresh vs NeedsProcessing
    }
}

enum PropertyBankStaleness {
    Fresh(PropertyBank),
    NeedsProcessing(PropertyBankPipeline<Unloaded>),
}
```

**Recommendation**: Option 1 (separate function) for clarity and testability.

### 5.5 Wrapping State Machines in Enums

When storing in parent structs, wrap in enum:

```rust
enum PropertyBankStatus {
    Unloaded(PropertyBankPipeline<Unloaded>),
    RawFile(PropertyBankPipeline<RawFile>),
    Parsed(PropertyBankPipeline<Parsed>),
    Validated(PropertyBankPipeline<Validated>),
    Domain(PropertyBankPipeline<Domain>),
    Persisted(PropertyBankPipeline<Persisted>),
    Completed(PropertyBankPipeline<Completed>),
}

impl PropertyBankStatus {
    pub fn advance(self) -> Result<Self, Error> {
        match self {
            PropertyBankStatus::Unloaded(p) => {
                Ok(PropertyBankStatus::RawFile(p.read_file()?))
            }
            PropertyBankStatus::RawFile(p) => {
                Ok(PropertyBankStatus::Parsed(p.parse()?))
            }
            // ... other transitions
        }
    }
}
```

This allows step-by-step execution while maintaining type safety.

---

## 6. Migration Strategy

### 6.1 Incremental Refactoring Plan

**Phase 0: Prerequisite - "Parse, Don't Validate"** (Week 1: 2-3 hours)

1. Make `RawPropertyBank` fields private
2. Add `RawPropertyBank::try_from_str()` parsing constructor
3. Add public accessor methods (`version()`, `properties()`, `metadata()`)
4. Remove `validate()`, `validate_version()`, `validated()` methods
5. Update `Ingestor` to use `try_from_str()`
6. Repeat for `RawSchema`
7. Run full test suite to ensure no regressions
8. **Deliverable**: No "Validated" state needed in state machine

**Phase 1: PropertyBank State Machine** (Weeks 1-2)

1. Create `PropertyBankPipeline<S>` with 7 consolidated states
2. Implement `Discovery` state with branching logic
3. Implement `PropertyBankPath` enum for branch handling
4. Implement each branch path:
   - NEW: FileParsed → BaseConstructed → ViewUpdated → Completed
   - FRESH: BaseConstructed → Completed (no persistence)
   - FRESH+TS: BaseConstructed → ViewUpdated → Completed
   - STALE: FileParsed → PropertyDelta → BaseConstructed → DeltaApplied → ViewUpdated → Completed
5. Move existing logic from `Ingestor::property_bank()` into state transitions
6. Update `Loader` to use new state machine
7. Add unit tests for each state transition
8. Add integration tests for all four branch paths
9. **Deliverable**: Fully functional PropertyBank state machine

**Phase 2: Schema State Machine** (Weeks 3-5)

1. Create `SchemaPipeline<S>` with sealed states
2. Implement linear stages (FileList → BulkStaleness → Partitioned)
3. Implement branching logic for partitioned state
4. Move existing logic from `Loader::load()` into state transitions
5. Refactor `RefExpander`, `Extender`, `Merger` to work with state machine
6. Add unit tests for each transition
7. Add integration tests for all branches
8. **Deliverable**: Fully functional Schema state machine

**Phase 3: Error Handling & Polish** (Week 6)

1. Consolidate error types (state-specific vs unified)
2. Improve error messages with state context
3. Add extensive rustdoc documentation
4. Performance benchmarking (ensure no regressions)
5. Update AGENTS.md with new patterns
6. Create ADR documenting the refactoring decision
7. **Deliverable**: Production-ready state machines with docs

### 6.2 The Builder Facade (Replacing Loader and Ingestor)

**New Architecture**: Replace the complex `Loader` orchestration and redundant `Ingestor` with a thin `Builder` facade that simply drives the state machines.

```rust
pub struct Builder<'config, R> {
    config: &'config Config,
    source: FsReader,
    repository: R,
}

impl<'config, R: Repository> Builder<'config, R> {
    pub fn build(&self) -> Result<Vec<Schema>, SchemaBuilderError> {
        // 1. Drive the PropertyBank state machine to completion
        let bank = PropertyBankPipeline::<Discovery>::new(
            self.config.paths().property_bank_path(),
            &self.source,
            &self.repository
        )
        .discover()?
        .into_completed(&self.repository)?
        .into_bank();

        // 2. Drive the Schema state machine to completion
        let schemas = SchemaPipeline::<Discovery>::new(
            self.config.paths().schemas_dir(),
            &self.source,
            &self.repository,
            &bank
        )
        .discover()?
        .into_completed(&self.repository)?
        .into_schemas();

        Ok(schemas)
    }
}
```

**External API simplified**, internal complexity pushed into isolated state machines.

### 6.3 Testing Strategy

**Preserve All Existing Tests**: Ensure no regressions

**Add New Tests**:

1. **Per-state unit tests**: Each transition in isolation
2. **Invalid transition tests**: Verify compile-time errors
3. **Branch coverage**: All paths through partitioned state
4. **Error propagation**: Errors bubble up correctly with context
5. **Staleness integration**: Cached paths work correctly
6. **Performance benchmarks**: No slowdowns from abstraction

---

## 7. Open Questions & Next Steps

### 7.1 Questions for Discussion

1. **Granularity**: Should we have more fine-grained states (e.g., separate "Deserialized" and "Validated" states)?
2. **Shared Data**: What's the best way to pass `Repository` and `Config` through the pipeline?
3. **Staleness**: Should staleness detection be part of the state machine or a separate pre-check?
4. **Error Handling**: State-specific errors vs unified error type?
5. **Testability**: How to test state transitions without full integration tests?
6. **Performance**: Any concerns about the overhead of state machines?
7. **Documentation**: How to document the state machine pattern in rustdoc?

### 7.2 Next Steps

**Immediate**:

1. **Review this document**: Confirm understanding of all pipeline stages
2. **Decide on state machine design**: PropertyBank vs Schema, granularity, error handling
3. **Prototype**: Implement PropertyBank state machine as proof-of-concept
4. **Validate**: Ensure no regressions, measure performance

**Follow-up**:

1. Implement Schema state machine
2. Refactor Loader orchestration
3. Update documentation
4. Add comprehensive tests
5. Create ADR documenting the refactoring decision

---

## Appendices

### A. File Locations

**Pipeline Orchestration**:

- `loader.rs` - Main orchestration (lines 1-1006)
- `ingestor.rs` - File I/O and staleness detection (lines 1-1466+)

**Pipeline Stages**:

- `expander.rs` - Property reference expansion (lines 1-442)
- `extender.rs` - Inheritance tree building (lines 1-855)
- `merger.rs` - Property merging (lines 1-879)
- `resolver.rs` - Property conflict resolution (lines 1-759)

**Domain Types**:

- `aggregate.rs` - Schema aggregate (lines 1-553)
- `bank.rs` - PropertyBank aggregate (lines 1-959)
- `property.rs` - Property domain types
- `raw/mod.rs` - Raw input types (lines 1-617)

**Views & Metadata**:

- `views/raw.rs` - RawPropertyBankView, RawSchemaView
- `views/metadata.rs` - Hash and timestamp metadata
- `views/inheritance.rs` - SchemaInheritanceView

**Storage**:

- `storage.rs` - Repository trait and implementations

### B. Dependency Graph

```
┌──────────────────────────────────────────────────────┐
│                       Loader                         │
│                   (orchestration)                    │
└────────┬─────────────────────────────────────────────┘
         │
         ├─── Ingestor (file I/O + staleness)
         │      │
         │      ├─── FsReader (filesystem abstraction)
         │      └─── Repository (database operations)
         │
         ├─── RefExpander (property reference expansion)
         │      │
         │      └─── PropertyBank (lookup)
         │
         ├─── Extender (inheritance tree building)
         │      │
         │      └─── SchemaTree (topological order)
         │
         └─── Merger (property merging)
                │
                └─── Resolver (conflict resolution)
```

### C. Key Constants

- `SCHEMA_EXTENSIONS: &[&str] = &["json", "toml", "yaml", "yml"]`
- `INHERITANCE_MAX_DEPTH: usize = 10`
- `PROPERTY_BANK_KEY: &str = "singleton"`
- `RAW_PROPERTY_BANK_KEY: &str = "property-bank"`

### D. Glossary

- **RefExpanded**: Schema with `$ref` pointers resolved to concrete `Property` instances
- **SchemaTree**: Topologically ordered inheritance tree (parents before children)
- **Known Parents**: Fresh schemas loaded from DB, used as parent references
- **Staleness**: Whether file content has changed since last load (timestamp + hash based)
- **Incremental Resolution**: Updating only changed properties instead of full re-resolution
- **Cached Expansion**: Reusing previously expanded properties when only PropertyBank changed

---

**End of Document**

_Total Pipeline Stages Identified_:

- **PropertyBank**: 6 distinct states (Discovery, FileParsed, PropertyDelta, BaseConstructed, DeltaApplied, Completed)
- **Schema**: 11 distinct states (Discovery, FileParsed, SchemaPropertyDelta, RawConstructed, BankReferenceDelta, DeltaApplied, InheritanceEvaluated, RefsExpanded, TreeConstructed, PropertiesMerged, Persisted)
- **Cross-cutting**: 2 staleness detection strategies, existing error hierarchy reused

_Architecture Decisions_:

1. **Builder Mutability**: Mutable Builder holds `property_delta: PropertyDelta` (empty = no changes)
2. **Infrastructure Separation**: Config, FsReader, Repository live in Builder, passed by reference to state transitions
3. **Delta Tracking**: SchemaData holds `ExtendsDelta` (old/new parent) and `ExcludesDelta` (added/removed)
4. **Inheritance Views**: Redesigned to 3 tables with `SCHEMA_DESCENDANTS` multimap for efficient BFS traversal
5. **Depth Pre-computation**: Added `depth: usize` to `SchemaInheritanceView` (saves recalculation during merging)
6. **Error Handling**: Reuse existing `SchemaIngestionError`, `SchemaRepositoryError`, `SchemaLoaderError` (no new types)

_Next Steps_:

1. Implement `PropertyBankData` and `SchemaData` structs with correct fields
2. Implement delta structures (`PropertyDelta`, `ExtendsDelta`, `ExcludesDelta`)
3. Update `views/inheritance.rs` with redesigned `SchemaInheritanceView`
4. Add `SCHEMA_DESCENDANTS` multimap to repository
5. Implement state machine transitions following the documented flows
6. Add BFS descendant traversal for incremental updates

_Confidence Level_: **High** - Based on thorough code review and redb research for optimal tree storage.
