# PropertyBankProcessor Refactor: Dimensional Typestate Pattern

## 1. Overview
The current `PropertyBankProcessor` uses a linear, single-generic typestate pattern. This leads to redundant state definitions (e.g., separate states for different metadata updates) and forces the orchestrator (`Builder`) to perform manual dispatching through nested `match` statements.

This refactor implements a **Dimensional Typestate Pattern** using two generic parameters:
1.  **Stage (P)**: The logical phase of the pipeline (Discovery, Comparison, Analysis, Refresh, Construction).
2.  **Status (S)**: The invariant-carrying knowledge state (Unknown, Missing, Present, Suspect, MetadataStale, New, Changed, Fresh).

## 2. Dimensional Model

### 2.1 Stage (`P`) - The Process Markers
Stages are marker types (empty structs) that define which actions are logically valid at any given point in the lifecycle.

| Marker Struct | Description |
| :--- | :--- |
| `Discovery` | Entry phase; checking repository for a cached view. |
| `Comparison` | Identity phase; comparing file timestamps/hashes with the cached view. |
| `Analysis` | Semantic phase; computing the property delta between file and view. |
| `Refresh` | Maintenance phase; syncing metadata (times/hashes) to the cache. |
| `Construction` | Building phase; creating, updating, or fetching the final bank. |
| `Completed` | Terminal phase; the `PropertyBank` is ready and owned. |

### 2.2 Status (`S`) - The Knowledge Carriers
Statuses are data-carrying types that store the proven invariants and the data associated with them.

| Status Struct | Carried Data | Description |
| :--- | :--- | :--- |
| `Unknown` | None | Initial state before any checks. |
| `Missing` | `RawFileTimes` | View not found; file exists. |
| `Present` | `RawFileTimes`, `RawPropertyBankView` | View found; file exists. |
| `Suspect` | `RawFileTimes`, `RawPropertyBankView`, `content` | Identity (time/hash) mismatch. |
| `MetadataStale` | `RawFileTimes`, `RawPropertyBankView`, `content_hash` | Hash changed but properties are the same. |
| `New` | `RawPropertyBank`, `String content` | View was missing; full parse required. |
| `Changed` | `RawPropertyBank`, `PropertyDelta`, `String content` | Properties changed; delta apply required. |
| `Fresh` | `Option<RawPropertyBankView>` | Local file and bank are in sync. |
| `Ready` | `PropertyBank` | Bank is fully constructed and ready to be returned. |

## 3. The Transition Matrix

The `Builder` (orchestrator) moves the `Processor<P, S>` through the matrix by calling transition methods.

| Source State | Transition Method | Resulting Branch/State |
| :--- | :--- | :--- |
| `Discovery, Unknown` | `discover(filename, repo, source)` | `Missing` → `Comparison, Missing`<br>`Exists` → `Comparison, Present` |
| `Comparison, Missing` | `parse(content)` | `Construction, New` |
| `Comparison, Present` | `check_timestamps()` | `Match` → `Construction, Fresh`<br>`Mismatch` → `Comparison, Suspect` |
| `Comparison, Suspect` | `check_content(content)` | `Match` → `Refresh, MetadataStale`<br>`Mismatch` → `Analysis, Suspect` |
| `Analysis, Suspect` | `analyze()` | `Empty` → `Refresh, MetadataStale`<br>`Delta` → `Construction, Changed` |
| `Refresh, MetadataStale` | `sync_metadata(repo)` | `Construction, Fresh` |
| `Construction, Fresh` | `fetch(repo)` | `Completed, Ready` |
| `Construction, New` | `create(repo)` | `Completed, Ready` |
| `Construction, Changed` | `update(repo)` | `Completed, Ready` |

## 4. Implementation Details

### 4.1 Trait-Driven Knowledge Access
To deduplicate logic (e.g., the `Refresh` stage's persistence logic), the `Status` types will implement marker traits:

```rust
/// Knowledge carries a RawPropertyBankView.
trait HasView {
    fn view(&self) -> &RawPropertyBankView;
    fn view_mut(&mut self) -> &mut RawPropertyBankView;
}

/// Knowledge carries file timestamps.
trait HasTimes {
    fn times(&self) -> &RawFileTimes;
}

/// Knowledge carries a RawPropertyBank and its content.
trait HasRaw {
    fn raw(&self) -> &RawPropertyBank;
    fn content(&self) -> &str;
}
```

### 4.2 Deduplicated Syncing (Refresh Stage)
Any status that implements `HasView` and `HasTimes` can use a generic `sync_metadata` method:

```rust
impl<S> PropertyBankProcessor<Refresh, S>
where S: HasView + HasTimes
{
    pub fn sync_metadata(mut self, repo: &R) -> Result<PropertyBankProcessor<Construction, Fresh>, Error> {
        let view = self.status.view_mut();
        view.update_timestamps(self.status.times());
        // ... update content hash if MetadataStale ...
        repo.save_raw_property_bank_view(view)?;
        Ok(PropertyBankProcessor::transition(Construction, Fresh { view: Some(view) }))
    }
}
```

### 4.3 Terminal Convergence (Construction Stage)
The `Construction` stage handles different statuses by specializing its methods:

- `fetch()`: Called when `Status` is `Fresh`.
- `create()`: Called when `Status` is `New`.
- `update()`: Called when `Status` is `Changed`.

## 5. Phased Implementation Plan

### Phase 1: Infrastructure
- [ ] Define `Stage` marker types (`Discovery`, `Comparison`, etc.).
- [ ] Define `Status` data-carrying types (`Unknown`, `Missing`, `Present`, etc.).
- [ ] Implement `HasView`, `HasTimes`, and `HasRaw` traits for relevant statuses.
- [ ] Implement `PropertyBankProcessor<P, S>` struct and `transition` helper.

### Phase 2: Discovery & Initial Branching
- [ ] Refactor `has_raw_view` to return `ComparisonBranch`.
- [ ] Implement `ComparisonBranch`: `Missing(Processor<Comparison, Missing>)` and `Present(Processor<Comparison, Present>)`.

### Phase 3: Identity & Semantic Analysis
- [ ] Implement `Comparison::check_timestamps`.
- [ ] Implement `Comparison::check_content`.
- [ ] Implement `Analysis::analyze` (Delta calculation).
- [ ] Implement `Refresh::sync_metadata`.

### Phase 4: Persistence & Convergence
- [ ] Implement `Construction::fetch`.
- [ ] Implement `Construction::create`.
- [ ] Implement `Construction::update`.
- [ ] Ensure `apply_delta` preserves property IDs (critical invariant).

### Phase 5: Facade Refactor
- [ ] Rewrite `Builder::load_property_bank` to utilize the linear, fluent API.
- [ ] Remove legacy `handle_content_branch` and `handle_delta_branch` private methods.

### Phase 6: Verification
- [ ] Update unit tests to reflect the new API structure.
- [ ] Run `mise run verify` to ensure quality gates pass.
