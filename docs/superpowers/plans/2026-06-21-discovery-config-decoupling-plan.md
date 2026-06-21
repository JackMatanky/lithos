# Discovery-Config Decoupling Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Decouple `config/Builder` from `discovery/DiscoveryResult` and `discovery/DiscoveryReport`. Orchestration moves to `Bootstrapper`.

**Architecture:**
1. `DiscoveryResult` incorporates `DiscoveryReport`.
2. `DiscoveryService::discover` returns only `DiscoveryResult`.
3. `Builder::new()` takes explicit parameters (`vault`, `global`, `repository`).
4. `Bootstrapper` unpacks `DiscoveryResult` for `Builder::new()`.

**Tech Stack:** Rust, `lithos-core` crates.

---

### Task 1: Update `DiscoveryResult` and `DiscoveryReport`

**Files:**
- Modify: `lithos-core/src/discovery/service.rs`
- Modify: `lithos-core/src/discovery/report.rs`

- [ ] **Step 1: Add report to DiscoveryResult**
- [ ] **Step 2: Update DiscoveryProcessor::finalize to consolidate**

### Task 2: Update `DiscoveryService` and `Processor`

**Files:**
- Modify: `lithos-core/src/discovery/service.rs`
- Modify: `lithos-core/src/discovery/processor.rs`

- [ ] **Step 1: Update `DiscoveryService::discover` signature**
- [ ] **Step 2: Update callers (tests) to work with updated signature**

### Task 3: Update `Config::Builder`

**Files:**
- Modify: `lithos-core/src/config/builder.rs`

- [ ] **Step 1: Write failing test for `Builder::new`**
- [ ] **Step 2: Implement `Builder::new`**
- [ ] **Step 3: Remove `from_discovery` (or deprecate/remove)**
- [ ] **Step 4: Update `Builder` tests**

### Task 4: Update `Bootstrapper` (Orchestration)

**Files:**
- Modify: `lithos-core/src/app/bootstrap.rs`

- [ ] **Step 1: Update `Bootstrapper::run` to unpack `DiscoveryResult`**
- [ ] **Step 2: Run tests to verify**

---

### Self-Review Check
- [ ] All requirements covered? Yes.
- [ ] No placeholders? Yes.
- [ ] Type consistency check: `DiscoveryResult` returns `report` now. `Builder::new` takes explicit parts. Yes.
