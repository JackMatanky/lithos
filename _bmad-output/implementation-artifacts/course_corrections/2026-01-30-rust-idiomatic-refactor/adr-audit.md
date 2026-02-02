# ADR Audit and Review

**Date**: 2026-02-01
**Phase**: Phase 1 - Foundation & Documentation
**Task**: 1.2 ADR Audit (Proposal 7)

---

## Audit Scope

Review all 17 ADRs in `docs/adr/` to determine:
1. **Keep** - Still valid with new architecture
2. **Update** - Needs revision to reflect single-crate approach
3. **Supersede** - Replaced by course correction proposals
4. **Move to guides/** - Not an architectural decision

---

## ADR Review Matrix

| ADR  | Name                                          | Status    | Decision | Reason                                                      |
| ---- | --------------------------------------------- | --------- | -------- | ----------------------------------------------------------- |
| 0001 | ADR Process                                   | accepted  | ✅ Keep  | Process meta-doc, still valid                               |
| 0002 | Storage (redb + rkyv)                         | accepted  | ✅ Keep  | Core decision, enhanced by Proposal 4                       |
| 0003 | Template Engine                               | accepted  | ✅ Keep  | Independent of architecture                                 |
| 0004 | Markdown Parsing                              | accepted  | ✅ Keep  | Independent of architecture                                 |
| 0005 | Configuration Management                      | accepted  | ✅ Keep  | Independent of architecture                                 |
| 0006 | Error Handling & Diagnostics                  | accepted  | 📝 Update | Still valid, but references old structure                   |
| 0007 | Event Orchestration                           | accepted  | 📝 Update | Proposal 6 says "defer events", needs alignment             |
| 0008 | Event-Driven Testing Patterns                 | accepted  | 🔄 Review | Depends on ADR 0007 decision                                |
| 0009 | CQRS Testing Patterns                         | accepted  | 📝 Update | Proposal 5 changes CQRS approach                            |
| 0010 | Centralized Test Utilities                    | accepted  | ✅ Keep  | Testing patterns still valid                                |
| 0011 | Integration Testing Patterns                  | accepted  | ✅ Keep  | Testing patterns still valid                                |
| 0012 | Benchmarking Infrastructure                   | proposed  | ✅ Keep  | Needed for zero-copy validation                             |
| 0013 | Domain Serialization Strategy                 | proposed  | ❌ Supersede | Proposal 10 changes approach (feature-gated serde)          |
| 0014 | Rename Detection Strategy                     | accepted  | ✅ Keep  | Independent of architecture                                 |
| 0015 | File Loading Port Boundary                    | accepted  | 📝 Update | Port location changed (co-located), concept still valid     |
| 0016 | Caching Strategy                              | proposed  | ❌ Supersede | Proposals 4 & 5 replace with db.rs approach                 |
| 0017 | Cache Metrics & Observability                 | proposed  | 🔄 Defer  | Depends on ADR 0016, revisit after db.rs implementation     |
| NEW  | Single-Crate Architecture & Database Layer    | -         | ✅ Create | New ADR 0017 (was 0018) documenting course correction      |

---

## Detailed Review by Category

### ✅ Keep As-Is (No Changes Needed)

#### **ADR 0001: ADR Process**
- **Why Keep**: Meta-process document
- **Impact**: None - describes process, not architecture
- **Action**: None

#### **ADR 0002: Storage (redb + rkyv)**
- **Why Keep**: Core technology decision remains valid
- **Impact**: Enhanced by Proposal 4 (db.rs layer)
- **Action**: None
- **Note**: Consider adding note linking to new ADR 0017 for implementation pattern

#### **ADR 0003: Template Engine**
- **Why Keep**: Technology choice independent of crate structure
- **Impact**: None
- **Action**: None

#### **ADR 0004: Markdown Parsing**
- **Why Keep**: Technology choice independent of crate structure
- **Impact**: None
- **Action**: None

#### **ADR 0005: Configuration Management**
- **Why Keep**: Technology choice independent of crate structure
- **Impact**: None
- **Action**: None

#### **ADR 0010: Centralized Test Utilities**
- **Why Keep**: Testing pattern still valid
- **Impact**: Location changes (lithos-core/tests/) but concept valid
- **Action**: None

#### **ADR 0011: Integration Testing Patterns**
- **Why Keep**: Testing pattern still valid
- **Impact**: Simpler with single-crate (no cross-crate mocking)
- **Action**: None

#### **ADR 0012: Benchmarking Infrastructure**
- **Why Keep**: Critical for validating Proposal 4 performance claims
- **Impact**: None - needed to prove 5-10x improvement
- **Action**: Promote from "proposed" to "accepted" during Phase 2

#### **ADR 0014: Rename Detection Strategy**
- **Why Keep**: Domain logic independent of architecture
- **Impact**: None
- **Action**: None

---

### 📝 Update (Needs Revision)

#### **ADR 0006: Error Handling & Diagnostics**
- **Why Update**: References old crate structure
- **Current**: Mentions "domain errors", "adapter errors", "app errors"
- **New**: Proposal 2 (co-located errors) and Proposal 8 (thiserror + miette)
- **Action**: Update to reflect:
  - Co-located errors (note/error.rs, schema/error.rs)
  - thiserror in core, miette in CLI
  - Remove references to separate crates
- **Effort**: Low (structural references only, decision still valid)

#### **ADR 0007: Event Orchestration**
- **Why Update**: Proposal 6 says "defer events until needed"
- **Current**: Prescribes 3-tier MPSC/Broadcast/Watch from day one
- **New**: Start with direct function calls, add events when profiled
- **Action**: Update to reflect:
  - Phase 1 (CLI): Direct function calls (no event bus)
  - Phase 2 (LSP): Add events if needed for async state sync
  - Measure-first approach (not "from day one")
- **Effort**: Medium (changes recommendation timing)

#### **ADR 0009: CQRS Testing Patterns**
- **Why Update**: Proposal 5 changes CQRS approach
- **Current**: Assumes explicit CQRS traits (NoteCommands, NoteQueries)
- **New**: Static methods with naming conventions (traits optional)
- **Action**: Update to reflect:
  - Primary: Static methods (impl Note { fn find_by_id(...) })
  - Optional: Traits for testing (if needed)
  - Testing with in-memory redb (no trait mocking required)
- **Effort**: Medium (testing approach changes)

#### **ADR 0015: File Loading Port Boundary**
- **Why Update**: Port location changed
- **Current**: References ports/ folder (centralized)
- **New**: Proposal 3 (ports co-located with contexts)
- **Action**: Update to reflect:
  - Ports in context/ports.rs (e.g., note/ports.rs)
  - Concept still valid (text-only domain contract)
  - Location changed only
- **Effort**: Low (location references only)

---

### ❌ Supersede (Create New ADR)

#### **ADR 0013: Domain Serialization Strategy**
- **Why Supersede**: Proposal 10 changes approach
- **Current**: "Controlled serde allowance in domain"
- **New**: Feature-gated serde (#[cfg_attr(feature = "serde", derive(...))])
- **Action**: Create new ADR superseding 0013
  - Document feature-gate pattern
  - Reference Rust API Guidelines (C-SERDE)
  - Mark 0013 as superseded
- **Effort**: Medium (new ADR required)

#### **ADR 0016: Caching Strategy**
- **Why Supersede**: Proposals 4 & 5 replace entire approach
- **Current**: L1 (Moka) + L2 (Redb) + Coordinator
- **New**: Concrete Database type with zero-copy methods (db.rs)
- **Action**: Mark as superseded by new ADR 0017 (Single-Crate Architecture)
  - Document db.rs pattern
  - Document zero-copy primitives (get_archived, put_reserve)
  - Defer Moka to Phase 2 (LSP)
- **Effort**: None (new ADR 0017 covers this)
- **Note**: This is THE major change

---

### 🔄 Review/Defer (Decision Depends on Other ADRs)

#### **ADR 0008: Event-Driven Testing Patterns**
- **Why Review**: Depends on ADR 0007 decision
- **If ADR 0007 defers events**: This ADR should be marked "deferred to Phase 2"
- **If ADR 0007 keeps events**: Update testing patterns for single-crate
- **Action**: Wait for ADR 0007 decision

#### **ADR 0017: Cache Metrics & Observability**
- **Why Defer**: Depends on ADR 0016 (which is superseded)
- **Current**: Observability for L1/L2 cache
- **New**: db.rs doesn't have L1/L2 in Phase 1
- **Action**: Mark as "deferred to Phase 2 (LSP)"
  - When Moka is added (Phase 2), revisit this ADR
  - Rewrite for db.rs + Moka architecture
- **Effort**: None (defer decision)

---

## New ADRs Required

### ✨ **NEW ADR 0017: Single-Crate Architecture & Database Layer**

**Purpose**: Document the course correction decision

**Content**:
1. **Context**:
   - Translation Gap anti-pattern
   - Multi-crate preventing zero-copy (5-10x penalty)
   - Sub-50ms LSP target at risk

2. **Decision**:
   - Single lithos-core crate + binary crates
   - Concrete Database type (not traits)
   - Zero-copy primitives (get_archived, put_reserve, multimap)
   - Visibility-based boundaries (pub(crate))

3. **Technical Validation**:
   - Research: Matklad, Rust Performance Book, Cargo Book
   - Real projects: tokio, rust-analyzer, clap
   - Performance: 5-10x improvement on reads
   - Compilation: 1.5-2x faster builds

4. **Consequences**:
   - Positive: Zero-copy performance, faster builds, idiomatic
   - Negative: Less polymorphic, harder to mock (mitigated)
   - Supersedes: ADR 0016 (caching strategy)

**Status**: accepted (approved 2026-02-01)

**Effort**: High (comprehensive ADR, ~300-500 lines)

---

### ✨ **NEW ADR 0018: Domain Serialization with Feature Gates**

**Purpose**: Supersede ADR 0013 with new approach

**Content**:
1. **Context**:
   - Rust API Guidelines recommend serde on data structures
   - Prior ADR 0013 had "controlled serde allowance" (vague)

2. **Decision**:
   - Domain types MAY derive Serialize/Deserialize
   - MUST be feature-gated: #[cfg_attr(feature = "serde", ...)]
   - ONLY on data structures (not behaviors/services)

3. **Technical Validation**:
   - Rust API Guidelines C-SERDE
   - Serde feature-flag documentation
   - Ecosystem patterns (serde, tokio)

4. **Consequences**:
   - Positive: Idiomatic, optional dependency
   - Negative: Feature gates add boilerplate (minor)
   - Supersedes: ADR 0013

**Status**: accepted (approved 2026-02-01)

**Effort**: Medium (~150-200 lines)

---

## Renumbering Plan

After superseding and creating new ADRs:

| Current | New   | Name                                     | Action    |
| ------- | ----- | ---------------------------------------- | --------- |
| 0001    | 0001  | ADR Process                              | Keep      |
| 0002    | 0002  | Storage (redb + rkyv)                    | Keep      |
| 0003    | 0003  | Template Engine                          | Keep      |
| 0004    | 0004  | Markdown Parsing                         | Keep      |
| 0005    | 0005  | Configuration Management                 | Keep      |
| 0006    | 0006  | Error Handling & Diagnostics             | Update    |
| 0007    | 0007  | Event Orchestration                      | Update    |
| 0008    | 0008  | Event-Driven Testing Patterns            | Review    |
| 0009    | 0009  | CQRS Testing Patterns                    | Update    |
| 0010    | 0010  | Centralized Test Utilities               | Keep      |
| 0011    | 0011  | Integration Testing Patterns             | Keep      |
| 0012    | 0012  | Benchmarking Infrastructure              | Keep      |
| 0013    | 0013  | Domain Serialization Strategy            | Supersede |
| 0014    | 0014  | Rename Detection Strategy                | Keep      |
| 0015    | 0015  | File Loading Port Boundary               | Update    |
| 0016    | 0016  | Caching Strategy                         | Supersede |
| 0017    | -     | Cache Metrics & Observability            | Defer     |
| -       | 0017  | Single-Crate Architecture & Database Layer | Create    |
| -       | 0018  | Domain Serialization with Feature Gates | Create    |

**Note**: Old 0017 is deferred (not assigned a new number until Phase 2)

---

## Execution Plan

### Step 1: Review with User (This Document)
- [ ] User reviews categorization
- [ ] User approves/adjusts decisions
- [ ] User confirms which ADRs to update first

### Step 2: Supersede Old ADRs
- [ ] Update ADR 0013 status to "superseded by 0018"
- [ ] Update ADR 0016 status to "superseded by 0017"
- [ ] Move old ADR 0017 to deferred/ folder (or rename to 0017-deferred.md)

### Step 3: Create New ADRs
- [ ] Create ADR 0017: Single-Crate Architecture & Database Layer
- [ ] Create ADR 0018: Domain Serialization with Feature Gates

### Step 4: Update Existing ADRs
- [ ] Update ADR 0006 (error handling references)
- [ ] Update ADR 0007 (event timing)
- [ ] Update ADR 0009 (CQRS patterns)
- [ ] Update ADR 0015 (port location)

### Step 5: Validate
- [ ] Run `mise run adr:validate`
- [ ] Verify all references updated
- [ ] No broken links

---

## Questions for User

Before proceeding, please confirm:

1. **ADR 0007 (Events)**: Do you want to update it to "defer until needed" or keep "implement from day one"?
   - Proposal 6 recommends defer
   - Your preference?

2. **ADR 0017 (old Cache Metrics)**: Should we:
   - A) Move to `docs/adr/deferred/0017-cache-metrics.md`
   - B) Delete (can retrieve from git history)
   - C) Rename to `0017-cache-metrics-deferred.md`

3. **Update Order**: Which ADRs should I update first?
   - Suggestion: Create new ADRs first (0017, 0018), then update existing

4. **ADR 0008 (Event Testing)**: If we defer events (ADR 0007), should we:
   - Mark ADR 0008 as "deferred to Phase 2"
   - Keep it but note "not applicable in Phase 1"

---

**Status**: ⏸️ **AWAITING USER INPUT**

Please review the matrix and answer the questions above, then I'll proceed with the ADR updates.
