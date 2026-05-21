---
title: 06-db-testing-seam-and-in-memory-alignment
category: enhancement
label: needs-triage
status: open
date_created: 2026-05-20
---

## Type

AFK

## Labels

- needs-triage

## What to build

Establish a shared DB testing seam that standardizes in-memory testing
infrastructure across contexts while keeping context Repository Adapters local.

This slice is complete when Schema, Note, Template, and Config migrations can
rely on the same testing primitives and error semantics for in-memory adapters.

## Agent Brief (v1 - 2026-05-20)

**Category:** enhancement
**Summary:** Add `db::testing` primitives and alignment constraints for
context-local in-memory Repository Adapters.

**Current behavior:**
In-memory testing adapters are inconsistent across contexts (Schema,
Config, Template, Note), increasing risk of semantic drift in lock handling,
instrumentation, and failure injection.

**Desired behavior:**
1. Add a `db::testing` module with infra-only test primitives (no context
   business semantics).
2. Introduce `InMemoryDbError` for shared in-memory testing failures.
3. Provide failure injection and operation counters usable by all context
   in-memory adapters.
4. Document and enforce the rule: contexts own their in-memory Repository
   Adapter semantics; `db::testing` provides shared testing infrastructure.

### Explicit `db::testing` design (required for this issue)

Create a new test-only module:

- `lithos-core/src/db/testing.rs`

Compilation target:

- Prefer `#[cfg(any(test, feature = "bench"))]` if bench targets need it.
- Otherwise `#[cfg(test)]` is acceptable.

This module must expose **infra primitives only**. It must not contain context
repository behavior or domain projection semantics.

#### 1) Test store creation

- **Trait:** `TestStoreFactory`
  - `open_temp_store() -> Result<(tempfile::TempDir, Store), DbError>`
  - `open_temp_store_arc() -> Result<(tempfile::TempDir, Arc<Store>), DbError>`
- **Optional convenience struct:** `TestStore`
  - `pub fn open_temp() -> Result<(tempfile::TempDir, Store), DbError>`
  - `pub fn open_temp_arc() -> Result<(tempfile::TempDir, Arc<Store>), DbError>`
- **Purpose:** Standardized temporary `Store` setup across
  Schema/Note/Template/Config tests.

#### 2) Lock handling helpers

- **Helper functions (generic):**
  - `read_lock<T>(lock: &RwLock<T>, ctx: &'static str) -> Result<RwLockReadGuard<'_, T>, InMemoryDbError>`
  - `write_lock<T>(lock: &RwLock<T>, ctx: &'static str) -> Result<RwLockWriteGuard<'_, T>, InMemoryDbError>`
  - `mutex_lock<T>(lock: &Mutex<T>, ctx: &'static str) -> Result<MutexGuard<'_, T>, InMemoryDbError>`
- **Purpose:** Unify lock-poison error mapping so all in-memory adapters expose
  consistent failure semantics.

#### 3) Operation instrumentation

- **Struct:** `OpCounters`
  - Backed by `AtomicUsize` fields (for example: reads, writes, batches,
    deletes, injected_failures)
  - Methods: `inc_read`, `inc_write`, `inc_batch`, `snapshot`
- **Snapshot struct:** `OpCountersSnapshot`
  - Plain values for assertions.
- **Purpose:** Replace ad-hoc call counters in context tests with a shared,
  stable instrumentation seam.

#### 4) Deterministic failure injection

- **Enum:** `FailurePoint`
  - At minimum include: `BeforeRead`, `BeforeWrite`, `AfterSerialize`,
    `BeforeCommit`
  - Can be extended conservatively if tests require more precision.
- **Trait:** `FailureInjector`
  - `fail_at(point: FailurePoint) -> Result<(), InMemoryDbError>`
- **Struct:** `InMemoryHarness`
  - Holds optional injector + counters
  - Context adapters embed this harness in their state for shared behavior
- **Purpose:** Deterministic rollback/atomicity/failure-path testing across
  all context in-memory adapters.

#### 5) Shared test-infra error type

- **Enum:** `InMemoryDbError` (in `db::testing`, not production `db/error.rs`
  unless explicitly promoted later)
- Required variants:
  - `LockPoisoned { context: &'static str }`
  - `InjectedFailure { point: FailurePoint, reason: Box<str> }`
  - `InvariantViolation { message: Box<str> }`
- Context adapters map this error via `From<InMemoryDbError>` into their own
  context error surface (Schema/Config/Template/Note).

#### 6) Contract test harness utilities (infra-only)

Add a small shared test utility for context adapters to run common behavioral
contracts without introducing shared domain logic.

- Contract capabilities:
  - batch atomicity checks
  - index consistency after save/delete
  - idempotent delete checks
- Contexts provide closures/fixtures/entities; `db::testing` provides the
  orchestration scaffolding only.

### Explicit non-goals

- Do **not** add a generic `db::InMemoryRepository` domain adapter.
- Do **not** add shared domain index maps in `db`.
- Do **not** move context-specific invariants from Schema/Note/Template/Config
  into DB.

These non-goals protect DB context locality per `db/CONTEXT.md` and avoid
creating a shallow cross-context fake storage module.

**Key interfaces:**
- `db::testing` module
- `InMemoryDbError`
- Context-local in-memory Repository Adapters
- `TestStoreFactory`, `FailureInjector`, `InMemoryHarness`, `OpCounters`
- `FailurePoint`

**Acceptance criteria:**
- [ ] `db::testing` module exists with reusable infra primitives.
- [ ] `InMemoryDbError` exists and is used for shared in-memory testing
      failures.
- [ ] `TestStoreFactory` (or equivalent `TestStore`) is implemented and used by
      at least one context test module.
- [ ] Generic lock helpers are implemented with consistent `InMemoryDbError`
      mapping.
- [ ] Failure injection and operation counters are available to all contexts.
- [ ] `FailureInjector`, `FailurePoint`, and `InMemoryHarness` are implemented.
- [ ] Shared contract-test scaffolding exists for atomicity/index/idempotency.
- [ ] Context-level adapter guidance is documented and referenced by follow-up
      migration slices.
- [ ] Explicit non-goals are documented and enforced in review.

## Acceptance criteria

- [ ] Shared in-memory testing infra is available in `db::testing`.
- [ ] Context adapters (Schema/Note/Template/Config) can consume the shared
      infra without moving context invariants into DB.
- [ ] Cross-context guidance for in-memory adapter shape is documented.
- [ ] Note/Template/Config migration slices (07/08/09) reference this seam and
      apply it during implementation.

## Blocked by

- `05-cross-context-interface-depth-review.md`
