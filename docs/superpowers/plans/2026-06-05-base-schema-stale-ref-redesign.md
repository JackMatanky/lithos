# BaseSchemaProcessor Stale Reference Redesign Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Correct the architectural placement of stale bank-reference handling by moving it to the Comparison/Analysis stages and ensuring `delta.rs` owns property hydration.

**Architecture:**
Detection moves from `Construction/Fresh` to `Comparison`. `StaleReferences` status triggers a targeted re-expansion flow where `delta.rs` injects forced properties into the semantic delta.

**Tech Stack:** Rust, Typestate Pattern, GitNexus.

---

### Task 1: Delta Engine Extension

**Files:**
- Modify: `traces-core/src/schema/delta.rs:351-413`
- Test: `traces-core/src/schema/delta.rs` (update engine tests)

- [ ] **Step 1: Update `diff_schema` signature**

```rust
// traces-core/src/schema/delta.rs
pub(crate) fn diff_schema(
    &self,
    expander: &RefExpander,
    forced_refs: &[PropertyName],
) -> Result<PropertyDelta, SchemaLoaderError>
```

- [ ] **Step 2: Implement forced reference injection**
In `diff_schema`, after `self.compute_change_set()`, iterate over `forced_refs`. If a name is not in the raw `upserts` map, fetch its raw entry from `self.properties` and insert it into the raw upsert map.

- [ ] **Step 3: Update `PropertyDeltaEngine` tests**
Add a test case where `forced_refs` contains a name whose hash has NOT changed, and verify it appears in the `PropertyDelta` upserts after hydration.

- [ ] **Step 4: Commit**

```bash
git add traces-core/src/schema/delta.rs
git commit -m "feat(delta): allow forced property injection in diff_schema"
```

---

### Task 2: Typestate Redesign (Routing)

**Files:**
- Modify: `traces-core/src/schema/base_processor.rs`

- [ ] **Step 1: Define new statuses**
Add `StaleReferences` and `ParsedStaleReferences` structs. `StaleReferences` carries `{ content: String, content_hash: Blake3Hash, view: RawSchemaView, schema_id: SchemaId, ref_delta: Vec<PropertyName> }`.

- [ ] **Step 2: Implement `impl<Parsed, StaleReferences>`**
Add `parse()` method that transitions to `ParsedStaleReferences`.

- [ ] **Step 3: Implement `impl<Analysis, ParsedStaleReferences>`**
Add `analyze()` method that calls `diff_schema(expander, &self.status.ref_delta)` and transitions to `Changed`.

- [ ] **Step 4: Update Comparison routing**
Update `check_timestamps` and `check_content` to perform the `changed_bank_references` check before transitioning to `Fresh`. If refs changed, read/use content and transition to `StaleReferences`.

- [ ] **Step 5: Commit**

```bash
git add traces-core/src/schema/base_processor.rs
git commit -m "refactor(schema): move stale ref detection to Comparison stage"
```

---

### Task 3: Consistency & Cleanup

**Files:**
- Modify: `traces-core/src/schema/base_processor.rs`

- [ ] **Step 1: Align ID preservation**
Ensure `update()` uses `upserts.clone().with_ids(&properties)` for all paths.

- [ ] **Step 2: Remove `CorruptNew` and `escalate_bank_conflict_to_new`**
Delete the status and the helper. Update `analyze` to route `None` version views to `AnalysisBranch::Corrupt(New)`.

- [ ] **Step 3: Remove bank-ref logic from `Construction/Fresh`**
Clean up the old orthogonal re-expansion methods.

- [ ] **Step 4: Commit**

```bash
git add traces-core/src/schema/base_processor.rs
git commit -m "refactor(schema): align ID stability and remove CorruptNew"
```

---

### Task 4: Integration Verification

**Files:**
- Create: `traces-core/tests/base_processor_integration.rs`

- [ ] **Step 1: Add Cold Start + Bank Change test**
Verify schema correctly transitions from `New` to `Stale` (targeted) when the bank target changes between runs.

- [ ] **Step 2: Add Multiple Schemas test**
Verify two schemas referencing the same bank target both receive targeted updates when that target changes.

- [ ] **Step 3: Run final quality gate**
Run `mise run verify` and confirm all 1600+ tests pass.

- [ ] **Step 4: Commit**

```bash
git add traces-core/tests/base_processor_integration.rs
git commit -m "test(schema): add stale bank reference integration tests"
```
