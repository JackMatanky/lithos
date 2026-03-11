# Refactor Plan Checklist Template (Context Modules)

**Purpose:** Single template for planning refactors across config/schema/note/template.
**Scope:** File-based architecture with Raw → Domain → Storage pipeline and context isolation.

---

## 0) Inputs and Constraints

- [ ] Read `_bmad-output/planning-artifacts/architecture/04-implementation-patterns-consistency-rules.md` (primary authority).
- [ ] Read `_bmad-output/project-context.md` and confirm latest rules.
- [ ] Read ADR 002 (Repository) only for historical context.
- [ ] Read `docs/refs/rust/naming-taxonomy.md` and confirm method naming rules.
- [ ] Confirm context isolation: no cross-imports between business contexts.
- [ ] Confirm file-based source-of-truth requirement for this refactor.

---

## 1) Full File and Component Audit (Context Inventory)

**Goal:** Enumerate everything in the context and classify what it is.

### 1.1 File Inventory
- [ ] List every file in the context directory (e.g., `lithos-core/src/<context>/`).
- [ ] For each file, record:
  - [ ] Purpose (parser, loader, resolver, adapter, raw types, domain types, storage, errors, tests)
  - [ ] Primary types/functions defined
  - [ ] External dependencies used (pulldown-cmark, rkyv, redb, etc.)
  - [ ] Ownership boundary (public API vs internal)

### 1.2 Component Inventory
- [ ] Catalog all domain types and their responsibilities.
- [ ] Catalog all Raw types (serde-only), and which files they parse.
- [ ] Catalog all stored/view/projection types and their use sites.
- [ ] Catalog all traits/ports/adapters (FsReader, Repository, etc.).
- [ ] Catalog all loaders/orchestrators and their entry points.
- [ ] Catalog all errors (enum variants, error construction patterns).

### 1.3 Cross-File Coupling Audit
- [ ] Identify cyclic dependencies across submodules.
- [ ] Identify “god modules” (too many responsibilities).
- [ ] Identify types used outside their intended layer (e.g., Raw types used in domain logic).

---

## 2) Workflow and Pipeline Audit (Behavioral Inventory)

**Goal:** Identify all pipelines and flows in the context and evaluate bloat and inefficiency.

### 2.1 Pipeline Map
- [ ] Diagram each pipeline (file → raw → domain → storage; plus any resolution phases).
- [ ] Identify all entry points (public functions, loaders, CLI handlers).
- [ ] List each stage function in order (parse, validate/resolve, transform, persist).
 - [ ] Note pipeline inputs/outputs and error boundaries per stage.

### 2.2 Bloat and Inefficiency Checks
- [ ] Duplicate parsing logic across modules?
- [ ] Redundant validation (checks repeated after domain construction)?
- [ ] Conversion churn (Raw → Domain → Stored → View without need)?
- [ ] Over-abstracted interfaces (ports/traits unused by tests)?
- [ ] Unused or dead pipeline steps?
- [ ] Unclear boundaries between parsing, validation, and resolution?

### 2.3 Modularity and Isolation Checks
- [ ] Parsing dependencies isolated in a `parser` submodule?
- [ ] External libraries isolated to a single boundary module?
- [ ] Domain logic free of I/O and parsing library types?
- [ ] Storage layer free of file I/O?
- [ ] Loader is the only orchestrator of file → domain → storage?

---

## 3) Architecture Alignment Audit

**Goal:** Compare current state to intended architecture and identify gaps.

- [ ] Raw types are serde-only (no behavior except parsing helpers).
- [ ] Domain types are validated, immutable, and used as the storage shape.
- [ ] No `Stored*` or `*View` types unless profiling justifies them.
- [ ] Repository trait is unified (no CQRS split).
- [ ] Zero-copy access uses `with_archived` closure pattern where needed.
- [ ] Context isolation respected (no cross-imports).
- [ ] Naming follows taxonomy (`parse_*`, `try_new`, `filter_*`, `all_*`, `any_*`, etc.).

---

## 4) Refactor Targets and Removal Candidates

**Goal:** Identify what to change and what to delete.

- [ ] Mark obsolete modules or types to delete.
- [ ] Mark types to merge or rename (e.g., `Stored*` → domain type).
- [ ] Mark traits to remove or collapse (CQRS → Repository).
- [ ] Mark functions to relocate to submodules (parser, resolver, index).
- [ ] Mark redundant tests or benchmarks to update/remove.

---

## 5) Proposed Module Structure (Target State)

**Goal:** Define the desired submodule layout for the context.

- [ ] Draft target file tree (mod.rs + submodules).
- [ ] Identify public API surface and re-exports in `mod.rs`.
- [ ] Define internal-only modules and keep them private.
- [ ] Specify ports/adapters and their location.

---

## 6) Target Pipeline Design (Target State)

**Goal:** Define the intended pipelines and stages for the context.

- [ ] Define the canonical pipeline (File → Raw → Domain → Storage) for this context.
- [ ] If multi-phase: list each phase and its purpose (parse, resolve, graph, sort, etc.).
- [ ] Define phase inputs/outputs as types (Raw*, Parsed*, Domain, Stored).
- [ ] Define where FsReader is used and where I/O stops.
- [ ] Define where validation occurs (TryFrom boundary) and where it must not occur.
- [ ] Define zero-copy read access points (`with_archived`) required for hot paths.
- [ ] Identify any optional view/projection types and justify them (profiling evidence).

---

## 7) Migration Plan (Ordered Steps)

**Goal:** Define the safe refactor sequence.

- [ ] Step-by-step sequence (no big-bang refactor).
- [ ] Explicit rename/move operations per file.
- [ ] Identify required test updates for each step.
- [ ] Identify temporary adapters/shims needed during transition.

---

## 8) Test and Verification Plan

**Goal:** Ensure correctness and prevent regressions.

- [ ] Unit tests for parsing boundary.
- [ ] Unit tests for semantic validation/resolution.
- [ ] Loader pipeline tests (file → raw → domain → storage).
- [ ] Repository behavior tests (fake/in-memory backend).
- [ ] Run verification tasks (fmt, lint, test). Document any exceptions.

---

## 9) Output Deliverables

**Goal:** Define what artifacts the refactor plan must produce.

- [ ] Context audit report (files, components, pipelines).
- [ ] Target module tree diagram.
- [ ] Gap analysis (current vs target).
- [ ] Ordered refactor steps with risks and mitigations.
- [ ] Test plan with coverage focus.

---

## 10) Context-Specific Notes

- [ ] Note any special parsing dependencies (pulldown-cmark, MiniJinja, YAML).
- [ ] Note any performance hot paths requiring zero-copy access.
- [ ] Note any domain invariants that must be preserved.
