# Phase 3: Interface Normalization & Deep Module Review Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Normalize repository trait usage to module-qualified generic names and encapsulate concrete storage implementations.

**Architecture:** Use global search and replace for trait normalization. Apply `#[doc(hidden)] pub` to `RedbRepository` and `InMemoryRepository` in all domain contexts to discourage direct coupling while maintaining testability.

**Tech Stack:** Rust, `gitnexus`, `rg`, `sed`.

---

### Task 1: Normalize Prefixed Traits in `lithos-core/src`

**Files:**
- Modify: All `.rs` files in `lithos-core/src`
- Modify: `lithos-core/src/schema/repository.rs` (ensure generic names)
- Modify: `lithos-core/src/vault/repository.rs` (ensure generic names)
- Modify: `lithos-core/src/note/repository.rs` (ensure generic names)
- Modify: `lithos-core/src/template/repository.rs` (ensure generic names)
- Modify: `lithos-core/src/config/repository.rs` (ensure generic names)

- [ ] **Step 1: Normalize `Schema` repository usage**
    - Search for `SchemaReadRepository`, `SchemaWriteRepository`, `SchemaRepository`.
    - Replace with `schema::ReadRepository`, `schema::WriteRepository`, `schema::Repository`.
    - Fix imports accordingly.

- [ ] **Step 2: Normalize `Vault` repository usage**
    - Search for `VaultReadRepository`, `VaultWriteRepository`, `VaultRepository`.
    - Replace with `vault::ReadRepository`, `vault::WriteRepository`, `vault::Repository`.
    - Fix imports accordingly.

- [ ] **Step 3: Normalize `Note` repository usage**
    - Search for `NoteReadRepository`, `NoteWriteRepository`, `NoteRepository`.
    - Replace with `note::ReadRepository`, `note::WriteRepository`, `note::Repository`.
    - Fix imports accordingly.

- [ ] **Step 4: Normalize `Template` repository usage**
    - Search for `TemplateReadRepository`, `TemplateWriteRepository`, `TemplateRepository`.
    - Replace with `template::ReadRepository`, `template::WriteRepository`, `template::Repository`.
    - Fix imports accordingly.

- [ ] **Step 5: Normalize `Config` repository usage**
    - Search for `ConfigReadRepository`, `ConfigWriteRepository`, `ConfigRepository`.
    - Replace with `config::ReadRepository`, `config::WriteRepository`, `config::Repository`.
    - Fix imports accordingly.

- [ ] **Step 6: Verify build**
    - Run: `cargo build -p lithos-core`
    - Expected: PASS

- [ ] **Step 7: Commit**
    - Run: `git add . && git commit -m "refactor(db): normalize prefixed repository traits in lithos-core/src"`

### Task 2: Normalize Prefixed Traits in Tests and Benches

**Files:**
- Modify: `lithos-core/tests/**/*.rs`
- Modify: `lithos-core/benches/**/*.rs`

- [ ] **Step 1: Normalize all prefixed traits in integration tests**
    - Apply same replacements as Task 1.
    - Special attention to `storage::RedbRepository as NoteRepository` aliases in tests.

- [ ] **Step 2: Normalize all prefixed traits in benchmarks**
    - Apply same replacements as Task 1.

- [ ] **Step 3: Verify tests**
    - Run: `cargo test -p lithos-core`
    - Expected: PASS

- [ ] **Step 4: Commit**
    - Run: `git add . && git commit -m "refactor(db): normalize prefixed repository traits in tests and benches"`

### Task 3: Normalize Prefixed Traits in Documentation

**Files:**
- Modify: `docs/**/*.md`
- Modify: `**/*.rs` (doc comments)

- [ ] **Step 1: Normalize prefixed traits in Markdown docs**
    - Search for `SchemaRepository`, etc., in `docs/` and replace with module-qualified names.

- [ ] **Step 2: Normalize prefixed traits in Rust doc comments**
    - Search for `SchemaRepository`, etc., in `///` or `//!` comments.

- [ ] **Step 3: Commit**
    - Run: `git add . && git commit -m "docs(db): normalize prefixed repository traits in documentation"`

### Task 4: Encapsulate Concrete Implementations

**Files:**
- Modify: `lithos-core/src/schema/storage/mod.rs`
- Modify: `lithos-core/src/vault/storage/mod.rs`
- Modify: `lithos-core/src/note/storage/mod.rs`
- Modify: `lithos-core/src/template/storage/mod.rs`
- Modify: `lithos-core/src/config/storage/mod.rs`
- Modify: `**/storage/testing.rs` (for `InMemoryRepository`)

- [ ] **Step 1: Encapsulate `RedbRepository` in all contexts**
    - Change `pub struct RedbRepository` to `#[doc(hidden)] pub struct RedbRepository`.

- [ ] **Step 2: Encapsulate `InMemoryRepository` in all contexts**
    - Change `pub struct InMemoryRepository` to `#[doc(hidden)] pub struct InMemoryRepository`.

- [ ] **Step 3: Verify integration tests still pass**
    - Run: `cargo test -p lithos-core`
    - Expected: PASS (Integration tests can still access `#[doc(hidden)] pub` items).

- [ ] **Step 4: Commit**
    - Run: `git add . && git commit -m "refactor(db): encapsulate concrete repository implementations with #[doc(hidden)]"`

### Task 5: Final Review & Execution Flow Check

**Files:**
- N/A

- [ ] **Step 1: Run `gitnexus_detect_changes()`**
    - Run: `gitnexus_detect_changes()`
    - Verify affected processes align with expectations.

- [ ] **Step 2: Run full quality gate**
    - Run: `mise run quality`
    - Expected: PASS

- [ ] **Step 3: Commit final changes**
    - Run: `git commit --amend --no-edit` (if applicable) or a final fixup commit.
