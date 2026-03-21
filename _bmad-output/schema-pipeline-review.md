# Schema Module Pipeline Review: Complete Analysis for State Machine Redesign

**Date**: 2026-03-19
**Purpose**: Comprehensive review of all pipeline stages to inform typestate pattern implementation
**Scope**: PropertyBank and Schema pipelines in `lithos-core/src/schema/`

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
2. `SchemaStateMachine` - 10 states, complex branching with staleness optimizations

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

| Path         | Trigger                         | File I/O         | Parsing             | DB Reads  | DB Writes   | Notes                  |
| ------------ | ------------------------------- | ---------------- | ------------------- | --------- | ----------- | ---------------------- |
| **NEW**      | No view in DB                   | ✅ Full read     | ✅ Parse + validate | -         | Bank + View | First time seeing file |
| **FreshTimestamp** | Timestamps match          | ❌ None          | ❌ None             | Bank only | -           | Fastest path (cached)  |
| **FreshContent**   | Hash matches, timestamps differ | ✅ Read for hash | ❌ None             | Bank only | View only   | Clock skew handling    |
| **STALE**    | Hash differs                    | ✅ Full read     | ✅ Parse + validate | Bank only | Bank + View | Incremental update     |

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
/// Ensures the path is properly formatted (e.g., `property_bank#/name`)
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
            .strip_prefix("property_bank#/")
            .ok_or_else(|| serde::de::Error::custom("Must start with 'property_bank#/'"))?;

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
┌──────────────────────────────────────────────────────────────────────────┐
│                                Discovery                                 │
│                     (Determines SchemaPipelinePath)                      │
│               (NEW / FreshTimestamp / FreshContent / STALE)              │
└───────┬─────────────────┬─────────────────┬──────────────────┬───────────┘
        │                 │                 │                  │
        ▼                 ▼                 ▼                  ▼
    ┌───────┐     ┌──────────────┐   ┌────────────┐        ┌───────┐
    │  NEW  │     │FreshTimestamp│   │FreshContent│        │ STALE │
    └───┬───┘     └───────┬──────┘   └──────┬─────┘        └───┬───┘
        │                 │                 │                  │
        ▼                 │                 │                  ▼
  ┌───────────┐           │                 │            ┌───────────┐
  │FileParsed │           │                 │            │FileParsed │
  └─────┬─────┘           │                 │            └─────┬─────┘
        │                 │                 │                  │
        │                 │                 │                  ▼
        │                 │                 │          ┌───────────────────┐
        │                 │                 │          │SchemaPropertyDelta│
        │                 │                 │          └────────┬──────────┘
        │                 │                 │                   │
        ▼                 ▼                 ▼                   ▼
 ┌──────────────┐ ┌──────────────┐  ┌──────────────┐   ┌──────────────┐
 │RawConstructed│ │RawConstructed│  │RawConstructed│   │RawConstructed│
 │(from scratch)│ │  (from DB)   │  │ (+upd times) │   │  (from DB)   │
 └──────┬───────┘ └───────┬──────┘  └───────┬──────┘   └───────┬──────┘
        │                 │                 │                  │
        │                 ▼                 ▼                  ▼
        │        (If PB STALE) ┌────────────────────────┐ (If PB STALE)
        │        ┌─────────────┤   BankReferenceDelta   ├─────────────┐
        │        │             └────────────────────────┘             │
        │        │                                                    ▼
        │        │                                              ┌────────────┐
        │        │                                              │DeltaApplied│
        │        │                                              └─────┬──────┘
        │        │                                                    │
        ▼        ▼                                                    ▼
┌────────────────────────────────────────────────────────────────────────────┐
│                                RefsExpanded                                │
│            (Full, Partial, or Zero expansion depending on path)            │
└─────────────────────────────────────┬──────────────────────────────────────┘
                                      │
                                      ▼
                          (Proceeds to Tree Building)
```

**State Details (Phase 1: Discovery to Expansion)**:

| State | Data Structure | Used By Paths | Notes |
|-------|---------------|---------------|-------|
| **1. Discovery** | `PropertyBankPath`, Config | All | Batches files, queries DB, determines `SchemaPipelinePath` |
| **2. FileParsed** | `RawSchema`, metadata | NEW, STALE | Parses file, validates, builds `bank_references` map |
| **3. SchemaPropertyDelta** | Schema delta info | STALE | Finds new/modified/removed properties in schema file |
| **4. RawConstructed** | `RawSchemaView`, `RawSchema` | All | Fetches baseline from DB or builds from scratch |
| **5. BankReferenceDelta** | PB refs to re-expand | FRESH*(+PB STALE), STALE(+PB STALE) | Intersects `bank_references` with PB's `PropertyDelta` |
| **6. DeltaApplied** | `RawSchema` (updated) | STALE | Applies schema file changes to baseline |
| **7. RefsExpanded** | `RefExpandedSchema` | All | Expands refs, constructs `SchemaVersion`, persists view |

### 2.2 Phase 1: File Discovery to Expansion

The action taken on each schema depends on a combination of **its own staleness** AND the **Property Bank's staleness**. The pipeline branches into 5 distinct flows.

#### **State 1: Discovery** (All Paths)
- **Input:** `PropertyBankPath` (with `PropertyDelta` if PB is STALE), Config schema dir
- **Operations:**
  - Scan directory (excluding `property_bank`).
  - Fetch `RawSchemaView`s from DB for all files.
  - Determine `SchemaPipelinePath` (`New`, `FreshTimestamp`, `FreshContent`, `Stale`) for each file via timestamp and content hash checks.
- **Output:** Branches into one of the following 5 flows.

---

#### **Flow A: Schema is NEW**
*No cached view exists. Must do full expansion.*
- **State 2 (FileParsed):**
  - Read file, parse into `RawSchema`.
  - Extract metadata, compute content hash.
- **State 4 (RawConstructed):**
  - Build `RawSchemaView` from scratch using the parsed `RawSchema`.
  - Compute `bank_references` (`HashMap<PropertyName, PropertyName>`) inside `SchemaVersion::new` by iterating over `RawSchema.properties` and extracting the `target_name` from any `RawPropertyRefPath`.
- **State 7 (RefsExpanded):**
  - Do full reference expansion on all properties against the `PropertyBank`.
  - Construct final `SchemaVersion` embedding `expanded_properties` and `bank_references`.
  - Update `RawSchemaView` and persist it.

---

#### **Flow B: Schema is Fresh* AND Property Bank is Fresh***
*Zero changes. Skip parsing entirely.*
- **State 7 (RefsExpanded):**
  - Instantly construct `RefExpandedSchema` using top-level metadata from the `RawSchemaView` (`name`, `extends`, `excludes`) and its cached `expanded_properties`.
  - *No `RawSchema` deserialization occurs!*
  - No DB persistence needed.

---

#### **Flow C: Schema is Fresh* AND Property Bank is STALE**
*Schema file didn't change, but some of its PB references might have.*
- **State 4 (RawConstructed):**
  - Fetch cached `RawSchemaView`.
  - If `FreshContent`, update view timestamps.
- **State 5 (BankReferenceDelta):**
  - Intersect the view's `bank_references` map with the PB's `PropertyDelta.changed` list.
  - Output: specific schema property names needing re-expansion.
  - *If intersection is empty, jump straight to RefsExpanded.*
- **State 7 (RefsExpanded):**
  - Deserialize `raw_properties` from JSON bytes (only because we must re-expand).
  - Re-run expansion *only* on the affected properties identified in State 5.
  - Update those specific keys in the cached `expanded_properties`.
  - Construct new `SchemaVersion`, update view, and persist.

---

#### **Flow D: Schema is STALE AND Property Bank is Fresh***
*Schema changed, PB didn't. Only expand schema changes.*
- **State 2 (FileParsed):**
  - Read new file, parse to `RawSchema`.
- **State 3 (SchemaPropertyDelta):**
  - Compare new property hashes against cached view to find new/modified/removed schema properties.
- **State 4 (RawConstructed):**
  - Fetch cached `RawSchemaView` as baseline.
- **State 6 (DeltaApplied):**
  - Apply `SchemaPropertyDelta` to baseline raw properties.
  - Build new `RawSchemaView` from the updated `RawSchema`, computing the new `bank_references` map.
- **State 7 (RefsExpanded):**
  - Expand only the NEW or MODIFIED properties from State 3.
  - Construct final `SchemaVersion` with updated `expanded_properties`, update view, and persist.

---

#### **Flow E: Schema is STALE AND Property Bank is STALE**
*Both changed. Expand schema changes PLUS affected PB references.*
- **State 2 (FileParsed):** Parse new file.
- **State 3 (SchemaPropertyDelta):** Compute schema file changes.
- **State 4 (RawConstructed):** Fetch cached `RawSchemaView` baseline.
- **State 5 (BankReferenceDelta):** Intersect *cached* `bank_references` with PB's `PropertyDelta`.
- **State 6 (DeltaApplied):**
  - Apply `SchemaPropertyDelta` to baseline.
  - Build new `RawSchemaView` from the updated `RawSchema`, computing the new `bank_references` map.
- **State 7 (RefsExpanded):**
  - Expand NEW/MODIFIED properties from State 3.
  - Re-expand unmodified schema properties flagged by State 5.
  - Construct final `SchemaVersion`, update view, persist.

---

### 2.3 Phase 2: Tree Building and Resolution

#### State 8: Inheritance Tree Building

**Input**:

- `Vec<(SchemaId, RefExpandedSchema)>` (stale schemas)
- `HashMap<SchemaId, Schema>` (known_parents, from DB)

**Output**: `SchemaTree` (topologically ordered)

**Operations** (6 phases in `Extender::build()` lines 222-250):

**Phase 1**: Build name indexes

- `name_to_id: HashMap<Box<str>, SchemaId>` - forward lookup
- `id_to_name: HashMap<SchemaId, Box<str>>` - reverse lookup
- Includes both stale schemas and known_parents
- Detect duplicate names

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

#### State 9: Property Merging (Schema Resolution)

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

#### State 10: Persistence

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

### 2.3 Cached Expansion Optimization

*(This concept is now natively handled by Flow C: Schema is Fresh AND PB is STALE, where we use the `BankReferenceDelta` to perform partial expansion.)*

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

**Typestate Pattern Application** (more complex):

```rust
// Primary states
struct FileList;          // Vec<PathBuf>
struct BulkStaleness;     // HashMap<Path, SchemaResult>
struct Partitioned;       // Three vectors split
struct Expanded;          // RefExpandedSchema
struct Tree;              // SchemaTree
struct Resolved;          // Vec<Schema>
struct Persisted;         // Saved to DB
struct Completed;         // Done

// Generic state machine
struct SchemaPipeline<S> {
    data: Box<SchemaData>,
    _state: PhantomData<S>,
}

// State-specific operations
impl SchemaPipeline<FileList> {
    pub fn new(paths: Vec<PathBuf>) -> Self { /* ... */ }
    pub fn check_staleness(self, repo: &Repository)
        -> Result<SchemaPipeline<BulkStaleness>, Error> { /* ... */ }
}

impl SchemaPipeline<BulkStaleness> {
    pub fn partition(self, bank_is_fresh: bool)
        -> SchemaPipeline<Partitioned> { /* ... */ }
}

impl SchemaPipeline<Partitioned> {
    pub fn expand_refs(self, bank: &PropertyBank)
        -> Result<SchemaPipeline<Expanded>, Error> { /* ... */ }

    pub fn skip_expansion_if_cached(self)
        -> Result<SchemaPipeline<Expanded>, Error> { /* ... */ }
}

impl SchemaPipeline<Expanded> {
    pub fn build_tree(self, known_parents: &HashMap<SchemaId, Schema>)
        -> Result<SchemaPipeline<Tree>, Error> { /* ... */ }
}

impl SchemaPipeline<Tree> {
    pub fn resolve(self, known_parents: &HashMap<SchemaId, Schema>)
        -> Result<SchemaPipeline<Resolved>, Error> { /* ... */ }
}

impl SchemaPipeline<Resolved> {
    pub fn persist(self, repo: &Repository)
        -> Result<SchemaPipeline<Persisted>, Error> { /* ... */ }
}

impl SchemaPipeline<Persisted> {
    pub fn complete(self) -> SchemaPipeline<Completed> { /* ... */ }
}

impl SchemaPipeline<Completed> {
    pub fn schemas(&self) -> &[Schema] { /* ... */ }
}
```

**Branch Handling**:

```rust
// Use enums for branching states
enum PartitionedSchemas {
    NeedsExpansion {
        needs_expansion: Vec<(SchemaId, RawSchema)>,
        cached_expansion: Vec<(SchemaId, HashMap<PropertyName, Property>)>,
        fresh_ids: Vec<SchemaId>,
    },
}

impl SchemaPipeline<Partitioned> {
    pub fn into_branches(self) -> PartitionedSchemas {
        // Extract three vectors
    }
}

// Process each branch through its path
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

**Challenge**: State transitions consume `self`, but we need to access data across states.

**Solution**: Store shared data in `Box<T>` or `Arc<T>`, carry it through transitions:

```rust
struct PropertyBankData {
    path: PathBuf,
    config: Config,
    repository: Repository,
    // Mutable data fields
    raw_content: Option<String>,
    raw_bank: Option<RawPropertyBank>,
    validated_bank: Option<RawPropertyBank>,
    domain_bank: Option<PropertyBank>,
}

struct PropertyBankPipeline<S> {
    data: Box<PropertyBankData>,
    _state: PhantomData<S>,
}

impl PropertyBankPipeline<Unloaded> {
    pub fn read_file(mut self) -> Result<PropertyBankPipeline<RawFile>, Error> {
        let content = std::fs::read_to_string(&self.data.path)?;
        self.data.raw_content = Some(content);

        Ok(PropertyBankPipeline {
            data: self.data,  // Move data to new state
            _state: PhantomData,
        })
    }
}
```

**Alternative**: Use `Option<T>` fields and `take()` to move data between states:

```rust
impl PropertyBankPipeline<RawFile> {
    pub fn parse(mut self) -> Result<PropertyBankPipeline<Parsed>, Error> {
        let content = self.data.raw_content.take()
            .expect("raw_content should exist in RawFile state");

        let raw_bank: RawPropertyBank = parse_content(&content)?;
        self.data.raw_bank = Some(raw_bank);

        Ok(PropertyBankPipeline {
            data: self.data,
            _state: PhantomData,
        })
    }
}
```

### 5.2 Error Handling in State Machines

**Approach 1**: State-specific error types

```rust
pub enum ReadFileError {
    Io(std::io::Error),
    InvalidPath(PathBuf),
}

pub enum ParseError {
    Json(serde_json::Error),
    Toml(toml::de::Error),
    Yaml(serde_yaml::Error),
}

impl PropertyBankPipeline<Unloaded> {
    pub fn read_file(self) -> Result<PropertyBankPipeline<RawFile>, ReadFileError> { /* ... */ }
}

impl PropertyBankPipeline<RawFile> {
    pub fn parse(self) -> Result<PropertyBankPipeline<Parsed>, ParseError> { /* ... */ }
}
```

**Approach 2**: Unified error type with context

```rust
pub enum PropertyBankError {
    ReadFile { source: std::io::Error, path: PathBuf },
    Parse { source: Box<dyn Error>, format: FileFormat },
    Validation { source: SchemaError },
    Domain { source: SchemaError },
    Storage { source: SchemaRepositoryError },
}

impl PropertyBankPipeline<Unloaded> {
    pub fn read_file(self) -> Result<PropertyBankPipeline<RawFile>, PropertyBankError> { /* ... */ }
}
```

**Recommendation**: Use Approach 2 (unified error) with `From` conversions for ergonomics.

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

- **PropertyBank**: 7 distinct states
- **Schema**: 10 primary states (with 3 branch paths)
- **Cross-cutting**: 2 staleness detection strategies, 5 error hierarchies

_Confidence Level_: **High** - Based on thorough code review of 15+ source files and 8000+ lines of code.
