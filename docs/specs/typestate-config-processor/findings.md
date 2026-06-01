# Findings: Config Typestate Refactoring

**Context**: Analyzing current config implementation to design typestate replacement for figment.

---

## Current Implementation

### Figment Usage Location

**File**: `lithos-core/src/config/loader.rs`
**Lines**: 22, 415-439

```rust
// Line 22: Import
use figment::{Figment, providers::Serialized};

// Lines 415-439: merge_raw_configs method
fn merge_raw_configs(
    global: Option<&RawGlobalConfig>,
    vault: Option<&RawVaultConfig>,
) -> Result<RawConfig, ConfigIngestError> {
    // Layer 1: Compiled defaults
    let mut figment = Figment::from(Serialized::defaults(RawConfig::default()));

    // Layer 2: Global config (if exists)
    if let Some(global_config) = global {
        figment = figment.merge(Serialized::defaults(RawConfig::from(global_config.clone())));
    }

    // Layer 3: Vault config (if exists)
    if let Some(vault_config) = vault {
        figment = figment.merge(Serialized::defaults(RawConfig::from(vault_config.clone())));
    }

    // Extract merged config
    figment.extract().map_err(ConfigIngestError::from)
}
```

### Merging Semantics

**Precedence Order** (lowest to highest priority):
1. `RawConfig::default()` - compiled-in defaults
2. `RawGlobalConfig` - system-wide settings
3. `RawVaultConfig` - project-specific overrides

**Key Observations**:
- Figment only used for this single 25-line method
- All config types already implement `From` conversions to `RawConfig`
- Merging is field-by-field override (later values win)
- No complex validation or transformation during merge

### Current Flow

```text
Loader::load()
  └─> rebuild_config() / rebuild_with_cached_vault() / rebuild_with_cached_global()
      └─> merge_raw_configs()  <-- FIGMENT USAGE HERE
          └─> Config::build()
```

---

## Existing Typestate Patterns

### Note Processor Pattern

**File**: `lithos-core/src/note/processor.rs`

**Stages**:
1. Discovery - repository lookup
2. Comparison - metadata staleness check
3. Analysis - parse markdown
4. Construction - build domain + persist
5. Completed - terminal state

**Statuses**: Unknown, Missing, Present, Suspect, New, Changed, Ready

**Key Features**:
- Single-dimension typestate (stage only)
- Status types carry accumulated data
- Transition methods return new typed states
- `PhantomData<P>` for stage marker

### Property Bank Processor Pattern

**File**: `lithos-core/src/schema/property_bank_processor.rs`

**Stages**:
1. Discovery - check cached view
2. Comparison - timestamp/hash checks
3. Analysis - property-level comparison
4. Refresh - metadata sync
5. Construction - create/update domain
6. Completed - terminal state

**Statuses**: Unknown, Missing, Present, Suspect, StaleTimestamps, StaleContent, Fresh, etc.

**Key Features**:
- Dual-dimension typestate (stage + status)
- Branch enums for orchestration (`ComparisonBranch`, `TimestampBranch`, etc.)
- Early-exit optimizations (fresh path vs. rebuild)
- More complex than note processor due to caching strategy

---

## Design Decisions

### Pattern Choice

**Decision**: Use **simple single-dimension typestate** like `note::processor`

**Rationale**:
- Config merging is simpler than property bank caching
- No complex caching/staleness logic in the merge itself (handled by Loader)
- Clear linear pipeline: defaults → global → vault → merged
- Matches existing note processor complexity level

### Proposed Stages

```rust
pub struct ConfigMerge;      // Merging stage
pub struct ConfigValidated;  // Post-merge validation (if needed)
pub struct ConfigCompleted;  // Terminal state
```

**Wait, simpler approach**: Config merging doesn't need full typestate pipeline. It's a pure function with no side effects or complex state transitions.

**Alternative Design**: Replace figment with **direct field merging** helper functions, no typestate needed.

---

## Revised Design: Direct Merging Functions

### Why Not Typestate?

After examining the code:
- `merge_raw_configs()` is a pure function (no I/O, no side effects)
- Only 3 layers, always in same order
- No complex branching or state accumulation
- Typestate is overkill for this use case

### Better Approach: Field-Level Merge Helpers

```rust
// New module: config/merge.rs

/// Merge two configs with right-hand precedence.
pub(crate) fn merge(base: RawConfig, overlay: RawConfig) -> RawConfig {
    RawConfig {
        logging: overlay.logging.or(base.logging),
        paths: overlay.paths.or(base.paths),
        // ... for each field
    }
}

/// Build final config from all layers.
pub(crate) fn merge_all(
    global: Option<&RawGlobalConfig>,
    vault: Option<&RawVaultConfig>,
) -> RawConfig {
    let mut merged = RawConfig::default();

    if let Some(g) = global {
        merged = merge(merged, RawConfig::from(g.clone()));
    }

    if let Some(v) = vault {
        merged = merge(merged, RawConfig::from(v.clone()));
    }

    merged
}
```

**Benefits**:
- No external dependency
- Explicit field-by-field logic (easier to debug)
- Matches Rust idiom: simple helper functions
- No unnecessary abstraction

**Tradeoffs**:
- Manual field listing (but config changes rarely)
- Less "magic" than figment

---

## Final Design Decision

**User Decision**: Implement typestate pattern for clarity and expressivity, even though simpler approaches exist.

---

## Phase 6C Incremental Migration Notes (2026-05-07)

- `Config::build` remains high-risk (`gitnexus_impact`: HIGH, 19 direct callers), so migration continues via low-risk slices.
- Removed unused low-risk adapters:
  - `impl TryFrom<&RawConfig> for Global`
  - `impl TryFrom<&RawConfig> for Vault`
- Reduced cross-context `RawConfig` coupling in tests:
  - `config/aggregate.rs` fixtures/tests now call `Config::build_from_layers(...)`
  - `note/aggregate.rs` test helpers now build from `RawVaultConfig` layers
- `Config::build_from_layers(...)` no longer constructs an intermediate `RawConfig`; it now validates/constructs domain fields directly.
- Residual `RawConfig` usage is now concentrated in:
  - `config/aggregate.rs` (`Config::build(&RawConfig, ...)` legacy entrypoint)
  - `config/merger.rs` test helper and related tests
  - `config/raw.rs` DTO + conversion impls + raw DTO tests

### Phase 6C Slice 3 updates

- Removed `RawConfig`-based helper/tests from `config/merger.rs`.
- Removed legacy `Config::build(&RawConfig, ...)` from `config/aggregate.rs`.
- Deleted `RawConfig` DTO and its conversion impls from `config/raw.rs`.
- Reworked raw DTO tests to validate `RawGlobalConfig` / `RawVaultConfig` directly.
- Remaining references to `RawConfig` in `lithos-core/src` are now zero (code path fully migrated).
- Verification after deletion: `cargo test --lib` passed (985/0).

### Phase 6D verification checkpoint

- Renamed `config/loader.rs` to `config/builder.rs` and migrated callsites/docs to `config::builder`.
- Moved layered construction API from `Config` impl to builder module (`build_from_layers`).
- Added `Config::new(...)` as the aggregate constructor boundary.
- Full quality gate now passes after migration: `mise run verify` green (unit + integration + doctests).

### Pre-commit scope check

- Ran `gitnexus_detect_changes(scope: all)` before commit prep.
- Reported risk level: `critical`, driven by broad symbol touch count (docs + tests + config core module edits).
- Runtime validation evidence remains green (`mise run verify` all pass), so the change set appears intentionally wide but behaviorally stable.

---

## Post-commit reassessment (user-requested)

The prior "complete" mark was technically premature. A focused audit found real functional gaps:

1. `config/processor.rs` has incomplete analysis logic:
   - `TODO` at `lithos-core/src/config/processor.rs:645`.
   - Current behavior assumes all fields changed when a view exists, so `NoChanges` is effectively unreachable for stale-view comparisons.
   - This defeats the intended incremental optimization.

2. `config/merger.rs` still contains persistence stubs:
   - `TODO` at `lithos-core/src/config/merger.rs:195`, `lithos-core/src/config/merger.rs:208`, `lithos-core/src/config/merger.rs:220`.
   - View-update branches currently fall back to `load_cached_config()` without writing refreshed views.

3. `config/discovery.rs` still has a repository integration TODO:
   - `TODO` at `lithos-core/src/config/discovery.rs:313` claims view-query methods are missing, but `Repository` already has `get_raw_global_view` and `get_raw_vault_view`.
   - This indicates design drift between implementation and comments/assumptions.

4. Responsibility boundaries need tightening:
   - Current `ConfigMerger` both resolves outcomes and performs rebuild/persistence.
   - User intent: merger/resolver should resolve settings from already-constructed field inputs, while builder orchestrates final resolved `Config` construction.

Conclusion: code compiles and tests pass, but architecture + behavior are not fully aligned with intended end-state.

### Phase 6E implementation notes (in progress)

- `processor.rs` gap reduced:
  - Added a type-level hook `content_hash_matches(view, raw)` to `ConfigType`.
  - Analysis now emits `NoChanges` when stale timestamp + equal content hash is detected.
  - Added regression coverage for this path (`analyze_stale_timestamp_same_content_returns_no_changes`).

- `merger.rs` TODO stubs removed:
  - `update_both_views_and_load`, `update_global_view_and_load`, `update_vault_view_and_load` now update raw views before loading cached config.
  - Added helper conversion from raw configs to `RawFileVersion` with explicit validation errors when metadata is missing.

- `discovery.rs` TODO drift corrected:
  - Replaced placeholder query with real repository view reads.
  - Vault view lookup now resolves `VaultId` from path mapping first.

Remaining alignment work: responsibility split (`ConfigMerger` as resolver vs builder orchestration boundary).

### Phase 6E validation findings

- Implementation uncovered a latent test-double defect:
  - `InMemoryRepository` stored global views in a `HashMap<VaultId, ...>` but keyed reads/writes with fresh `VaultId::new()` values.
  - This made global-view retrieval nondeterministically impossible for update flows.
- Corrected by modeling global view as singleton state (`Option<RawGlobalConfigView>`), which matches domain semantics (single environment config view).

- After fixes:
  - targeted config tests pass,
  - full `mise run verify` passes.

Remaining work is now architectural (naming and stricter role boundaries), not correctness of current implemented flows.

### Naming alignment

- Renamed `ConfigMerger` to `ConfigResolver` to match intended responsibility language.
- Current resolver still performs persistence actions for update/rebuild flows; further separation into pure resolution vs persistence orchestration remains optional follow-up depending on desired strictness.

Verification after rename/update work:
- `cargo test -p lithos-core config::merger::tests` passed.
- `mise run verify` passed.

### Boundary split completed

- `config/merger.rs` was refactored from side-effecting orchestration into pure resolution.
- `ConfigResolver` now emits `ResolutionPlan` (`UseCached`, `UpdateViews`, `Rebuild`) instead of touching repository state.
- Persistence and construction responsibilities moved to `config/builder.rs`:
  - cached config loading,
  - view update persistence,
  - rebuild versioning + save flow.
- This aligns with intent: resolver resolves outcomes; builder orchestrates I/O + construction boundaries.

### Scope/risk checkpoint after boundary split (2026-05-08)

- Ran `gitnexus_detect_changes(scope: all)` before commit preparation.
- Current result reports:
  - changed symbols: 51
  - changed files: 8
  - risk level: low
  - affected indexed processes: 0 in this run
- Interpretation: residual change set is concentrated and lower-risk than earlier broad migration checkpoints.

**Rationale**: Pattern consistency across contexts (note, schema, config) improves codebase navigability and maintains architectural coherence.

### Typestate Design for Config Processing (DRAFT - NEEDS APPROVAL)

**QUESTION FROM USER**: What about discovery and comparison stages?

**Current Loader Pipeline** (from `loader.rs::load()`):
```rust
1. Discovery:     get_or_create_vault_id()
2. Ingestion:     ingestor.global_config() / ingestor.vault_config()
3. Comparison:    is_global_stale() / is_vault_stale()
4. Decision:      match (global_stale, vault_stale) branches
5. Rebuild:       rebuild_config() / rebuild_with_cached_*()
   └─> Merging:   merge_raw_configs()  <-- figment usage
   └─> Building:  Config::build()
   └─> Storage:   repository.save_config()
```

**Two Options for Typestate Design**:

#### Option A: Full Pipeline (Discovery → Comparison → Merge → Build)

Matches `note::processor` and `property_bank_processor` patterns:

**Stages**:
```rust
pub struct Discovery;      // Load raw configs from ingestor
pub struct Comparison;     // Check staleness (timestamps + hashes)
pub struct Merge;          // Combine layers (replaces figment)
pub struct Construction;   // Build domain + persist
pub struct Completed;      // Terminal state
```

**Statuses**:
```rust
// Discovery outcomes
pub struct Unknown;
pub struct Loaded {
    vault_id: VaultId,
    global: Option<RawGlobalConfig>,
    vault: Option<RawVaultConfig>
}

// Comparison outcomes
pub struct Fresh { vault_id: VaultId, version: Version }  // Can load from DB
pub struct Stale {
    vault_id: VaultId,
    global: Option<RawGlobalConfig>,
    vault: Option<RawVaultConfig>,
    which_stale: StaleLayers  // Both | GlobalOnly | VaultOnly
}

// Merge outcomes
pub struct Merged {
    vault_id: VaultId,
    config: RawConfig
}

// Construction outcomes
pub struct Built { config: Config }
```

**Branch Enums** (like property_bank_processor):
```rust
pub enum ComparisonBranch {
    Fresh(ConfigProcessor<Construction, Fresh>),    // Fast path: load from DB
    Stale(ConfigProcessor<Merge, Stale>),          // Slow path: rebuild
}

pub enum StaleLayers {
    Both,
    GlobalOnly,
    VaultOnly,
}
```

**Benefits**:
- ✅ Complete pipeline modeling
- ✅ Fast path optimization (fresh → skip merge/rebuild)
- ✅ Matches existing processor patterns
- ✅ Explicit staleness branches

**Drawbacks**:
- ⚠️ More complex than merge-only
- ⚠️ Loader orchestration logic duplicated in processor
- ⚠️ Larger API surface

---

#### Option B: Merge-Only Processor (Minimal Scope)

**Scope**: Only replace the `merge_raw_configs()` figment usage.
**Leave**: Discovery, comparison, storage in `Loader` (as-is).

**Stages**:
```rust
pub struct Defaults;      // Start with defaults
pub struct GlobalMerge;   // After merging global
pub struct VaultMerge;    // After merging vault
pub struct Completed;     // Terminal state
```

**Statuses**:
```rust
pub struct WithDefaults { config: RawConfig }
pub struct WithGlobal { config: RawConfig }
pub struct WithVault { config: RawConfig }
pub struct Ready { config: RawConfig }
```

**API**:
```rust
// Called by Loader after staleness check
let merged = ConfigProcessor::<Defaults, WithDefaults>::new()
    .merge_global(global)
    .merge_vault(vault)
    .finalize()
    .into_config();
```

**Benefits**:
- ✅ Minimal scope change
- ✅ Replaces figment cleanly
- ✅ Simple, focused responsibility
- ✅ Loader keeps orchestration

**Drawbacks**:
- ⚠️ Doesn't model full pipeline
- ⚠️ Less pattern consistency with note/schema processors

---

### APPROVED DESIGN: Full Pipeline with Property Analysis

**User Decisions**:
1. ✅ **Option A**: Full pipeline typestate
2. ✅ **4 staleness cases** with branch enums
3. ✅ **Pattern**: Like `property_bank_processor` (dual-dimension)
4. ✅ **Add property analysis stage** for fine-grained change detection

---

### Final Typestate Design

**Pattern**: Dual-dimension typestate (Stage × Status) like `property_bank_processor`

**Pipeline Stages**:
```rust
pub struct Discovery;      // Load raw configs + check repository
pub struct Comparison;     // Timestamp + hash staleness detection
pub struct Analysis;       // Property-level change detection
pub struct Merge;          // Combine config layers
pub struct Construction;   // Build domain + persist
pub struct Completed;      // Terminal state
```

**Status Types**:
```rust
// Discovery
pub struct Unknown;
pub struct Discovered {
    vault_id: VaultId,
    vault_root: VaultRoot,
    global: Option<RawGlobalConfig>,
    vault: Option<RawVaultConfig>,
    global_view: Option<RawGlobalConfigView>,  // From repository
    vault_view: Option<RawVaultConfigView>,    // From repository
}

// Comparison outcomes (4 staleness cases)
pub struct Fresh {
    vault_id: VaultId,
    version: Version,
    // Both configs match their views - can load from DB
}

pub struct BothStale {
    vault_id: VaultId,
    vault_root: VaultRoot,
    global: Option<RawGlobalConfig>,
    vault: Option<RawVaultConfig>,
    global_view: Option<RawGlobalConfigView>,  // Old version for comparison
    vault_view: Option<RawVaultConfigView>,    // Old version for comparison
}

pub struct GlobalStale {
    vault_id: VaultId,
    vault_root: VaultRoot,
    global: Option<RawGlobalConfig>,
    global_view: Option<RawGlobalConfigView>,  // Old version for comparison
    // vault is fresh in DB - can reuse cached vault
}

pub struct VaultStale {
    vault_id: VaultId,
    vault_root: VaultRoot,
    vault: Option<RawVaultConfig>,
    vault_view: Option<RawVaultConfigView>,    // Old version for comparison
    // global is fresh in DB - can reuse cached global
}

// Analysis outcomes
pub struct NoChanges {
    vault_id: VaultId,
    vault_root: VaultRoot,
    version: Version,
    // Timestamps/hash changed but actual properties unchanged
    // Need to update views but can keep cached Config
    global: Option<RawGlobalConfig>,  // For view update
    vault: Option<RawVaultConfig>,    // For view update
}

pub struct PropertyChanges {
    vault_id: VaultId,
    vault_root: VaultRoot,
    changed_fields: HashSet<ConfigField>,  // Which fields changed
    global: Option<RawGlobalConfig>,
    vault: Option<RawVaultConfig>,
}

// Merge outcome
pub struct Merged {
    vault_id: VaultId,
    vault_root: VaultRoot,
    version: Version,
    config: RawConfig,
}

// Construction outcome
pub struct Built {
    config: Config,
}
```

**Branch Enums** (orchestration):
```rust
pub enum ComparisonBranch {
    Fresh(ConfigProcessor<Construction, Fresh>),          // Fast path: load from DB
    BothStale(ConfigProcessor<Analysis, BothStale>),      // Both changed
    GlobalStale(ConfigProcessor<Analysis, GlobalStale>),  // Global changed
    VaultStale(ConfigProcessor<Analysis, VaultStale>),    // Vault changed
}

pub enum AnalysisBranch {
    NoChanges(ConfigProcessor<Construction, NoChanges>),        // Metadata sync only
    PropertyChanges(ConfigProcessor<Merge, PropertyChanges>),   // Rebuild needed
}

pub enum ConfigField {
    Logging,
    Paths,
    Task,
    Frontmatter,
}
```

**Key View Types** (from `config/views.rs`):
- `RawGlobalConfigView` - Tracks global config file with version history
- `RawVaultConfigView` - Tracks vault config file with version history
- `RawFileVersion` - Single version snapshot with BLAKE3 hash + timestamps

**View Staleness Detection**:
- `view.is_fresh(raw)` - Hybrid check: timestamps + content hash
- `view.latest_version()` - Get most recent cached version
- `RawFileVersion::is_timestamp_match()` - Fast check
- `RawFileVersion::is_content_match()` - Accurate hash check

**Full Pipeline Flow**:
```rust
// 1. Discovery - Load raw configs + views from repository
let processor = ConfigProcessor::<Discovery, Unknown>::new(vault_root, repo);
let discovered = processor.discover(&ingestor)?;  // Loads raw + views

// 2. Comparison - Check staleness using views
match discovered.compare()? {  // Uses view.is_fresh(raw)
    ComparisonBranch::Fresh(p) => p.fetch_cached()?,  // Fast path

    ComparisonBranch::BothStale(p) => {
        match p.analyze()? {  // Field-level change detection
            AnalysisBranch::NoChanges(p) => {
                // Only timestamps changed - update views, keep cached Config
                p.sync_views()?
            }
            AnalysisBranch::PropertyChanges(p) => {
                // Actual field changes - full rebuild
                p.merge()
                 .build()?
                 .persist()?  // Saves new Config + updated views
            }
        }
    }

    ComparisonBranch::GlobalStale(p) => {
        match p.analyze()? {
            AnalysisBranch::NoChanges(p) => p.sync_metadata()?,
            AnalysisBranch::PropertyChanges(p) => {
                p.merge_global()
                 .build()?
                 .persist()?
            }
        }
    }

    ComparisonBranch::VaultStale(p) => {
        match p.analyze()? {
            AnalysisBranch::NoChanges(p) => p.sync_metadata()?,
            AnalysisBranch::PropertyChanges(p) => {
                p.merge_vault()
                 .build()?
                 .persist()?
            }
        }
    }
}
```

**Property Analysis Logic**:

The Analysis stage performs fine-grained change detection:

1. **Extract field-level data** from raw configs:
   - `global.logging` vs cached global in view
   - `global.paths` vs cached global in view
   - `vault.task` vs cached vault in view
   - `vault.frontmatter` vs cached vault in view

2. **Compare per-field hashes**:
   - Compute BLAKE3 hash for each field in new raw config
   - Compare against decompress + parse of view's cached version
   - Track which fields changed: `HashSet<ConfigField>`

3. **Optimization decision**:
   - If `changed_fields.is_empty()` → `NoChanges` (timestamps changed, content same)
   - If `!changed_fields.is_empty()` → `PropertyChanges` (actual data changed)

4. **Early exit for NoChanges**:
   - Update `RawGlobalConfigView` / `RawVaultConfigView` with new timestamps
   - Push new `RawFileVersion` to version ring buffer
   - Return cached `Config` from repository (no rebuild)

**View Update on Stale Detection**:
- When stale, create new `RawFileVersion` from raw config
- Push to view's version ring buffer (maintains max 5)
- Persist updated view to repository

**Benefits**:
- ✅ Complete pipeline modeling (matches property_bank_processor)
- ✅ 4 explicit staleness branches (compile-time safety)
- ✅ Property-level change detection (optimization)
- ✅ Fast path for fresh configs
- ✅ Metadata-only updates (timestamps changed, content same)
- ✅ Consistent pattern across all contexts

**Implementation Scope**:
- New module: `config/processor.rs` (~800-1000 lines, similar to property_bank_processor)
- Update: `config/loader.rs` to use processor instead of direct calls
- New types: Branch enums, status types, stage markers
- Tests: Transition tests, branch tests, property analysis tests

---

## REVISED DESIGN: Unified Discovery + Parallel Single-File Processors

**User Proposal** (11:30):
1. **Unified Discovery Engine** - Separate orchestrator that loads files + views
2. **Single-File Typestate Processor** - Reusable processor for one config file
3. **Parallel Execution** - Run processor for global AND vault simultaneously
4. **Merger + Construction** - After both processors complete, merge + build domain

### Architecture

```
┌─────────────────────────────────────────────────────────┐
│ Unified Discovery Engine                                │
│ - Load vault ID                                         │
│ - Ingest global config (optional)                       │
│ - Ingest vault config (optional)                        │
│ - Load global view from repository                      │
│ - Load vault view from repository                       │
└─────────────────────────────────────────────────────────┘
                    ↓
        ┌───────────┴───────────┐
        ↓                       ↓
┌──────────────────┐    ┌──────────────────┐
│ Global Processor │    │ Vault Processor  │
│ (parallel)       │    │ (parallel)       │
└──────────────────┘    └──────────────────┘
        ↓                       ↓
        └───────────┬───────────┘
                    ↓
        ┌───────────────────────┐
        │ Merger + Construction │
        │ - Combine configs     │
        │ - Build domain        │
        │ - Persist             │
        └───────────────────────┘
```

### Single-File Processor Pipeline

**Generic over config type** `T` (either `RawGlobalConfig` or `RawVaultConfig`):

**Stages**:
```rust
pub struct Comparison;    // Compare raw vs view
pub struct Analysis;      // Property-level change detection
pub struct Completed;     // Terminal state
```

**Status Types**:
```rust
pub struct Unknown<T> {
    raw: Option<T>,
    view: Option<V>,  // V = RawGlobalConfigView | RawVaultConfigView
}

pub struct Fresh {
    // View matches raw - no rebuild needed
    cached_version: Version,
}

pub struct Stale<T> {
    raw: T,
    view: Option<V>,  // Old version for comparison
}

pub struct NoChanges<T> {
    // Timestamps changed, properties unchanged
    raw: T,
    view: Option<V>,
}

pub struct PropertyChanges<T> {
    raw: T,
    changed_fields: HashSet<ConfigField>,
}

pub struct Ready<T> {
    outcome: ProcessorOutcome<T>,
}
```

**Branch Enums**:
```rust
pub enum ComparisonBranch<T, V> {
    Fresh(ConfigFileProcessor<Completed, Fresh>),
    Stale(ConfigFileProcessor<Analysis, Stale<T>>),
}

pub enum AnalysisBranch<T> {
    NoChanges(ConfigFileProcessor<Completed, NoChanges<T>>),
    PropertyChanges(ConfigFileProcessor<Completed, PropertyChanges<T>>),
}

pub enum ProcessorOutcome<T> {
    UseCached,                    // Fresh - no work needed
    UpdateViewOnly,               // NoChanges - sync metadata
    Rebuild(T),                   // PropertyChanges - need merge
}
```

### Usage Pattern

```rust
// 1. Discovery (not in typestate - separate orchestrator)
let discovery = DiscoveryEngine::new(vault_root, &repo);
let discovered = discovery.discover(&ingestor)?;
// discovered = { vault_id, global_raw, vault_raw, global_view, vault_view }

// 2. Parallel processing (same processor type, different inputs)
let global_processor = ConfigFileProcessor::new(
    discovered.global_raw,
    discovered.global_view,
);
let vault_processor = ConfigFileProcessor::new(
    discovered.vault_raw,
    discovered.vault_view,
);

// Run in parallel
let global_outcome = global_processor.compare()?.analyze()?.finalize();
let vault_outcome = vault_processor.compare()?.analyze()?.finalize();

// 3. Merge + Construction (based on outcomes)
let merger = ConfigMerger::new(vault_id, vault_root, version);
let config = match (global_outcome, vault_outcome) {
    (UseCached, UseCached) => repo.get_config(vault_id, version)?,
    (Rebuild(g), Rebuild(v)) => merger.merge_both(g, v)?.build()?.persist()?,
    (Rebuild(g), UseCached) => merger.merge_global(g)?.build()?.persist()?,
    (UseCached, Rebuild(v)) => merger.merge_vault(v)?.build()?.persist()?,
    // ... other combinations with UpdateViewOnly
};
```

### Benefits

✅ **Composability**: Single processor works for both global and vault
✅ **Parallelism**: Can run both processors simultaneously (no shared state)
✅ **Separation of concerns**: Discovery / Processing / Merging are distinct
✅ **Reusability**: Same typestate pattern for any config file
✅ **Clarity**: Each component has single responsibility
✅ **Testing**: Easier to test single-file processor in isolation

### Modules

```
config/
├── discovery.rs       # DiscoveryEngine - load files + views
├── processor.rs       # ConfigFileProcessor<P, S> - single-file typestate
├── merger.rs          # ConfigMerger - combine global + vault (replaces figment)
├── loader.rs          # Orchestrates: discovery → process → merge → persist
└── views.rs           # (existing) RawGlobalConfigView, RawVaultConfigView
```

### Design Decisions (Based on Schema Views Pattern)

**Reviewing schema views architecture** (`schema/views/`):

Key patterns observed:
- `HashRecord` - dual-level hashing (content hash + per-property hashes)
- `RawSchemaView` / `RawPropertyBankView` - version ring buffers with staleness detection
- `SchemaVersion` / `PropertyBankVersion` - snapshot types with `FileInfo + HashRecord`
- Per-property hash maps enable incremental updates (only re-expand changed properties)
- Zero-copy archived types (`ArchivedRawSchemaView`) for hot-path reads
- Traits: `RawView` (mutation), `RawViewRead` (zero-copy), `Version`, `VersionRead`

**Applying to Config Context**:

We need similar view types for config files:
- `RawGlobalConfigView` - ✅ Already exists in `config/views.rs`
- `RawVaultConfigView` - ✅ Already exists in `config/views.rs`
- `RawFileVersion` - ✅ Already exists (timestamp + content hash)

**Missing**: Per-field hash tracking (like `HashRecord::properties`)

Current `RawFileVersion` only has:
- `content_hash: Blake3Hash` - whole file
- `compressed_content: Vec<u8>` - zstd compressed TOML
- Timestamps

**Need to add**: Field-level hash map for incremental analysis

**IMPORTANT CORRECTIONS**:
1. `RawFileVersion` should NOT have `compressed_content` - that's for schema views which store full content
2. Config views only need: `FileInfo` + `content_hash` + `field_hashes`
3. Use `FileInfo` (not separate `created_at`/`modified_at`)
4. `loader.rs` should be renamed to `builder.rs` (matches schema pattern)

---

### Answers to Design Questions

**1. Generics vs Enum?**

**Recommendation**: Use **trait-based generics** like schema views do.

```rust
// Define trait contract
pub trait ConfigType {
    type Raw;  // RawGlobalConfig | RawVaultConfig
    type View;  // RawGlobalConfigView | RawVaultConfigView

    fn compute_field_hashes(raw: &Self::Raw) -> ConfigFieldHashes;
}

// Implement for each config type
pub struct GlobalConfig;
impl ConfigType for GlobalConfig {
    type Raw = RawGlobalConfig;
    type View = RawGlobalConfigView;
    // ...
}

pub struct VaultConfig;
impl ConfigType for VaultConfig {
    type Raw = RawVaultConfig;
    type View = RawVaultConfigView;
    // ...
}

// Generic processor
pub struct ConfigFileProcessor<T: ConfigType, P, S> {
    _config_type: PhantomData<T>,
    _stage: PhantomData<P>,
    status: S,
}
```

**Benefits**:
- ✅ Type safety (no runtime enum matching)
- ✅ Zero-cost abstraction
- ✅ Single implementation works for both
- ✅ Matches schema views pattern

**2. Property Analysis - Field-Level Hashing**

**Recommendation**: Add `ConfigFieldHashes` to views (like `RawPropertyMapHash`).

Need to extend `config/views.rs`:

```rust
// Add to views.rs
#[derive(Debug, Clone, PartialEq, Archive, Serialize, Deserialize)]
pub struct ConfigFieldHashes(HashMap<ConfigField, Blake3Hash>);

#[derive(Debug, Clone, PartialEq, Eq, Hash, Archive, Serialize, Deserialize)]
pub enum ConfigField {
    Logging,
    Paths,
    Task,
    Frontmatter,
}

// Update RawFileVersion to include field hashes
pub struct RawFileVersion {
    /// File metadata (timestamps + size)
    file_info: FileInfo,

    /// Blake3 hash of entire file content
    content_hash: Blake3Hash,

    /// Per-field Blake3 hashes for incremental analysis
    field_hashes: ConfigFieldHashes,

    /// When this version was recorded to DB
    recorded_at: SystemTime,
}
```

**Analysis Logic** (like `SchemaVersion::changed_bank_references`):
```rust
impl ConfigFileProcessor<Analysis, Stale<T>> {
    fn analyze(&self) -> AnalysisBranch<T> {
        let old_fields = self.status.view.latest_version().field_hashes();
        let new_fields = compute_field_hashes(&self.status.raw);

        let changed: HashSet<ConfigField> = new_fields
            .iter()
            .filter_map(|(field, hash)| {
                if old_fields.get(field) != Some(hash) {
                    Some(field.clone())
                } else {
                    None
                }
            })
            .collect();

        if changed.is_empty() {
            AnalysisBranch::NoChanges(/* ... */)
        } else {
            AnalysisBranch::PropertyChanges(/* changed fields */)
        }
    }
}
```

**3. View Updates - Batch in Merger**

**Recommendation**: **Option B - Batch update in merger** (after both processors complete).

Matches schema views pattern where `Builder` updates views after schema processing.

```rust
// In ConfigMerger::persist()
fn persist(&self, global_outcome, vault_outcome) {
    // 1. Build domain config
    let config = self.build_from_outcomes(global_outcome, vault_outcome)?;

    // 2. Batch update views
    if let Some(new_version) = global_outcome.new_version() {
        repo.save_raw_global_view(&new_version)?;
    }
    if let Some(new_version) = vault_outcome.new_version() {
        repo.save_raw_vault_view(&new_version)?;
    }

    // 3. Save config
    repo.save_config(vault_id, &config)?;
}
```

**Benefits**:
- ✅ Atomic update (all or nothing)
- ✅ Processor stays pure (no side effects)
- ✅ Merger controls persistence strategy

**4. Outcome Combinations - 9 Cases with Precedence Awareness**

**Recommendation**: Handle all 9 cases with **vault override optimization**.

```rust
match (global_outcome, vault_outcome) {
    // Both fresh - load from DB
    (UseCached, UseCached) => repo.get_config(vault_id, version)?,

    // Metadata-only updates - sync views, keep config
    (UpdateViewOnly, UpdateViewOnly) => {
        sync_both_views()?;
        repo.get_config(vault_id, version)?
    }
    (UpdateViewOnly, UseCached) => {
        sync_global_view()?;
        repo.get_config(vault_id, version)?
    }
    (UseCached, UpdateViewOnly) => {
        sync_vault_view()?;
        repo.get_config(vault_id, version)?
    }

    // Vault rebuild (vault overrides global) - optimize
    (_, Rebuild(vault_raw)) => {
        // Vault changed - must rebuild
        // But: can reuse cached global if not changed
        let global_raw = match global_outcome {
            Rebuild(g) => Some(g),
            _ => None,  // Use cached/defaults
        };
        merger.merge(global_raw, Some(vault_raw))?.build()?.persist()?
    }

    // Global rebuild, vault cached - partial merge
    (Rebuild(global_raw), UseCached | UpdateViewOnly) => {
        // Global changed, vault fresh
        // Load cached vault from DB or use defaults
        merger.merge_global_only(global_raw)?.build()?.persist()?
    }
}
```

**Key Insight**: Vault always wins due to precedence, so:
- If vault rebuilds → always rebuild final config
- If only global rebuilds → merge global + cached vault
- If neither rebuilds → use cached config

---

### Updated Module Design

```
config/
├── discovery.rs       # DiscoveryEngine - load files + views (not typestate)
├── processor.rs       # ConfigFileProcessor<T, P, S> - single-file typestate
├── merger.rs          # ConfigMerger - combine outcomes + build domain
├── loader.rs          # Orchestrator: discovery → process × 2 → merge
└── views.rs           # UPDATED: Add ConfigFieldHashes to RawFileVersion
```

---

## Test Coverage Analysis

### Existing Tests

**File**: `lithos-core/src/config/loader.rs` (lines 746-777)

```rust
#[test]
fn merge_raw_configs_with_only_defaults()
fn merge_raw_configs_with_global_only()
fn merge_raw_configs_with_vault_only()
fn merge_raw_configs_with_both()
```

**Coverage**: Basic smoke tests, no field-level validation

### Required New Tests

- [ ] Verify field precedence (vault > global > default)
- [ ] Test `Option<T>` fields merge correctly (Some overwrites None)
- [ ] Test all config field types (logging, paths, task, etc.)
- [ ] Test metadata preservation during merge
- [ ] Integration test: end-to-end config loading

---

**Last Updated**: 2026-05-06
**Status**: Analysis complete, plan revised

---

## Post-Completion Corrections (16:45)

**Time**: 16:45
**Status**: ✅ CORRECTIONS APPLIED

### Issues Fixed

1. **CONTEXT.md Updated**:
   - Added Environment Config language (system-wide from env vars/global files)
   - Added Local (Vault) Config language (vault-specific overrides)
   - Updated Precedence Chain: Environment Config < Local (Vault) Config
   - Added examples with actual file paths

2. **RawFileVersion Fixed**:
   - ✅ Removed compressed_content field (not needed for config views)
   - ✅ Replaced created_at + modified_at with FileInfo struct
   - ✅ Updated new() constructor to take FileInfo
   - ✅ Removed decompress() method
   - ✅ Updated is_timestamp_match() to use file_info
   - ✅ Added file_info() accessor method
   - ✅ All 7 view tests pass

3. **RawVaultConfigView Fixed**:
   - ✅ Removed vault_id field (domain concern, not view concern)
   - ✅ Updated new() to not take vault_id parameter
   - ✅ Removed vault_id() accessor method
   - ✅ Updated storage trait: save_raw_vault_view() takes vault_id separately
   - ✅ Updated all callers

### Commits

- Commit d50ff897: docs(config): update CONTEXT.md with environment/local config language
- Commit 1685dbf5: fix(config): update RawFileVersion to use FileInfo
- Commit c0912e0c: fix(config): remove vault_id from RawVaultConfigView

### Final Verification
- ✅ All 1032 tests pass
- ✅ All view tests pass (7 tests)
- ✅ Full test suite passes
- ✅ RawFileVersion now uses FileInfo (consistent with schema/views)
- ✅ RawVaultConfigView no longer has domain-specific vault_id

---

## Design Problems Analysis (14:45)

**Status**: ✅ RESOLVED - User provided design decisions

### Root Cause Analysis

User correctly identified that I was mechanically fixing compilation errors without addressing underlying design problems. Analysis revealed:

#### 1. RawConfig is Obsolete

**Problem**: `RawConfig` was created as intermediate merge type for Figment. Now that Figment is gone, it's redundant.

**Evidence**:
- `merger.rs:275` - `merge_raw_configs()` returns `RawConfig`
- Now: `RawGlobalConfig` + `RawVaultConfig` → `RawConfig` → `Config` (unnecessary intermediate)

**USER DECISION**: **Remove `RawConfig` completely**. Merge `RawGlobalConfig` + `RawVaultConfig` → `Config` directly.

**Additional**: Also remove `RawConfigMetadata` type - it's just `FileInfo` from `fs/file.rs`.

#### 2. Raw* Type Organization

**Problem**: `RawFrontmatter` (frontmatter.rs:246) and `RawLogging` (logging.rs:117) are in separate files.

**USER DECISION**: **Move to raw.rs** - Does not match codebase conventions. All Raw* types should be in `raw.rs`.

#### 3. Missing discovery.rs Module

**Problem**: Design plan called for `config/discovery.rs` (see findings.md:700-705), but never created. Discovery logic still in `loader.rs`.

**USER DECISION**: **Create discovery.rs** - Similar to `schema/discovery.rs`. Should handle:
- File ingestion (loading raw configs from filesystem)
- Batch fetching Raw*View types from DB
- Routing first part of typestate pattern

#### 4. loader.rs Should Be builder.rs

**Problem**: `loader.rs` name doesn't match schema pattern.

**USER DECISION**: **Rename loader.rs → builder.rs**. Also:
- Move `Config::build()` method out of `Config` type
- `Config` should only have `new()` method
- Builder should have the `build()` method

#### 5. VaultId Import Confusion

**Problem**: Imports using `aggregate::VaultId` fail (private re-export).

**DECISION**: Always import from `vault::VaultId` (source module).

#### 6. Duplicate Module Declarations & testing.rs Not Tracked

**Problem**: `mod.rs` had duplicate `value`/`views` declarations, testing.rs not staged.

**FIXED**: Duplicates removed, testing.rs staged.

### Applied Fixes (14:50)

✅ **Fixed immediate compilation blockers**:
1. Removed `#[cfg(bench)]` (not recognized) → use `#[cfg(test)]`
2. Fixed `vault_root` moved in merger test
3. Fixed `merger` consumed by `merge()` - created merger2 for second test
4. Fixed unused/shadowed variables in tests
5. Fixed non-binding `let _` with `#[must_use]` types
6. Fixed unfulfilled lint expectations in testing.rs
7. Removed duplicate module declarations
8. Staged testing.rs

✅ **All pre-commit hooks pass** (17 checks including clippy, tests, ADR validation)

### Next Phase: Architecture Redesign

Based on user decisions, need to:
1. Remove RawConfig and RawConfigMetadata types
2. Move RawFrontmatter and RawLogging to raw.rs
3. Create discovery.rs module (similar to schema/discovery.rs)
4. Rename loader.rs → builder.rs
5. Move Config::build() to builder module
6. Update Config to only have new() method

### RawConfig Usage Analysis (17:05)

**Key Usages Found:**

1. **merger.rs** - `merge_raw_configs()` returns `RawConfig`, merges fields
2. **aggregate.rs** - `Config::build()` takes `&RawConfig` parameter
3. **global.rs** - `Global::try_from(&RawConfig)` conversion
4. **vault.rs** - `Vault::try_from(&RawConfig)` conversion
5. **raw.rs** - `From<RawGlobalConfig>` and `From<RawVaultConfig>` conversions
6. **Tests** - Many tests use `RawConfig::default()` or construct from global/vault

**Impact Assessment:**

- **High Impact**: `Config::build()` signature changes from `&RawConfig` → needs new approach
- **Medium Impact**: merger.rs needs to merge RawGlobalConfig + RawVaultConfig directly
- **Low Impact**: Conversions can be replaced with direct field access

**Refactoring Strategy (REVISED per user guidance):**

1. **Create discovery.rs FIRST** (enables clean metadata handling)
   - Pattern after schema/discovery.rs structure
   - Key differences from ingestor.rs:
     * Use FileReader.info() instead of created_at()/modified_at() separately
     * Return FileInfo directly (not RawConfigMetadata)
     * Batch DB queries for views in single transaction
     * Return discovery data for typestate routing

2. **Plug DiscoveryEngine into loader.rs** (replace ingestor)
   - Remove ingestor.rs dependency
   - Use DiscoveryEngine::run() for file + DB discovery

3. **Then remove RawConfig/RawConfigMetadata** (after discovery is clean)
   - Change `Config::build()` to take `Option<RawGlobalConfig>` + `Option<RawVaultConfig>`
   - Move merging logic into `Config::build()` or builder
   - Update `Global::try_from()` and `Vault::try_from()` to take separate configs
   - Remove `From<RawGlobalConfig/RawVaultConfig> for RawConfig` impls
   - Update tests to use separate configs

### Discovery Engine Pattern Analysis (17:20)

**schema/discovery.rs Key Patterns:**

1. **Result Types**:
   - `SchemaCachedState` - ID + view for existing files
   - `SchemaDiscovery` - FileEntry + Optional<CachedState>
   - `DiscoveryResult` - Aggregates all discovery data

2. **Engine Structure**:
   - Single `run()` method orchestrates pipeline
   - Filesystem scan → Batch DB query → Combine results
   - Uses FileEntry (path + FileInfo) from DirScanner

3. **Key Methods**:
   - `scan_filesystem()` - DirScanner with pattern/extensions
   - `query_cached_state()` - Single transaction for all DB reads
   - `build_result()` - Combines filesystem + DB data

**config/ingestor.rs Current Approach:**

1. **Separate methods** for global and vault
2. **Individual timestamp calls**: `created_at()`, `modified_at()`
3. **Builds RawConfigMetadata** manually
4. **Returns parsed configs** (not discovery data)

**Needed Changes for discovery.rs:**

1. Use `FileReader.info()` → returns `FileInfo` directly
2. Return discovery data (FileInfo + Optional<View>), not parsed configs
3. Batch DB queries in single transaction
4. Support global + vault in one discovery run

---

## Phase 6C Slice 1: RawConfigMetadata Removal (18:05)

### Impact Analysis (GitNexus)

- `Config::build` upstream impact: **CRITICAL** (60 impacted symbols, 5 processes, 5 modules)
- `RawConfig` upstream impact: LOW
- `RawConfigMetadata` upstream impact: LOW

Decision: proceed with low-risk slice first (`RawConfigMetadata`), then tackle critical `Config::build` refactor in smaller steps.

### Changes Applied

1. `raw.rs`
   - `RawGlobalConfig.metadata` changed from `RawConfigMetadata` to `Option<FileInfo>`
   - `RawVaultConfig.metadata` changed from `RawConfigMetadata` to `Option<FileInfo>`
   - Removed `RawConfigMetadata` type entirely

2. `processor.rs` tests
   - Removed `RawConfigMetadata` import
   - Updated fixtures to `metadata: None`

3. `merger.rs` tests
   - Removed `RawConfigMetadata` import
   - Updated fixtures to `metadata: None`

4. `views.rs`
   - Reworked freshness checks to use `Option<FileInfo>` timestamp checks
   - Added content-hash fallback by hashing serialized raw config (`toml::to_string` + `Blake3Hash::compute`)

### Verification

- `cargo test --lib` passes: **988 passed, 0 failed**
- No remaining `RawConfigMetadata` usages in `lithos-core/src`

---

## Pre-commit Fixes Applied (17:00)

**Status**: ✅ COMPLETE - All hooks pass

### Issues Found & Fixed:

1. **`ref` pattern in processor.rs:212, 223**
   - Lines still have `if let Some(ref frontmatter)` and `if let Some(ref task)`
   - My earlier sed/bash fixes didn't actually modify the file correctly
   - Fix: Change to `if let Some(frontmatter) = &raw.frontmatter`

2. **`merge_raw_configs` is now associated function (no `&self`)**
   - Removed `&self` to fix clippy unused_self warning
   - But callsites still use method syntax: `self.merge_raw_configs(...)`
   - Also in tests: `merger.merge_raw_configs(...)`
   - Fix: Change all callsites to use `ConfigMerger::merge_raw_configs(...)` syntax

3. **Lifetime rename incomplete**
   - Changed `'a` to `'repo` in struct definition and impl block
   - But line 42 still had `'a` - now fixed
   - Need to verify all references updated

### Plan:
1. ✅ Fix processor.rs ref patterns (lines 212, 223) - DONE (used as_ref())
2. ✅ Fix merger.rs syntax (added &self back with expect unused_self)
3. ✅ Fix assigning_clones in merger.rs (lines 292, 300, 303) - DONE (use clone_from)
4. ✅ Fix pattern_type_mismatch in processor.rs - DONE (use as_ref())
5. ✅ Fix syntax errors (duplicate code blocks, from_hash_map)
6. ✅ Fix unused imports and unfulfilled lint expectations
7. ✅ Fix test callsites to use method syntax (not associated function)
8. ❌ Fix default_trait_access (line 522) - IN PROGRESS
9. ❌ Fix as_conversions (lines 583, 604, 619, 638) - IN PROGRESS
10. Run pre-commit to verify all clippy warnings fixed
11. Commit with proper message (no --no-verify)


## Phase 6C Slice 2: Start migration off RawConfig (18:20)

- Added `Config::build_from_layers(...)` to `config/aggregate.rs`.
- `build_from_layers` applies precedence directly: defaults < global < vault.
- Updated `ConfigMerger::rebuild_with_configs` to call `build_from_layers` instead of `merge_raw_configs` + `Config::build`.
- This reduces production-path dependency on `RawConfig` before full type removal.

Verification: `cargo test --lib` -> 988 passed.
