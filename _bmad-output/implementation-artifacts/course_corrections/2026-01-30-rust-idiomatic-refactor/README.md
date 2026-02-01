# Course Correction: Rust Idiomatic Architecture Refactor

**Date**: 2026-01-30
**Status**: Approved (2026-02-01)
**Scope**: MAJOR / STOP-THE-WORLD
**Approver**: Jack (Product Owner)

---

## Overview

This course correction addresses a fundamental architectural misalignment: treating Rust as a generic implementation detail rather than a distinct system. The multi-crate workspace structure was preventing zero-copy optimizations and causing a 5-10x performance penalty.

**Decision**: Pivot to a single-crate "Core" architecture (`lithos-core`) + separate binary crates (`lithos-cli`, `lithos-lsp`).

---

## Document Structure

This folder contains the complete documentation for the Rust Idiomatic Architecture Refactor course correction:

### 1. `discovery-context-summary.md`
**Purpose**: High-level context of how the issue was discovered
**Audience**: Quick reference for understanding the "why"
**Contains**:
- The core realization (Translation Gap anti-pattern)
- Symptom identification (Cache performance issues)
- Research phase (Rust ecosystem analysis)
- Decision to pivot to holistic architecture review

### 2. `critical-architecture-review.md`
**Purpose**: Comprehensive research findings and analysis
**Audience**: Detailed technical reference
**Contains**:
- Complete conversation flow (9 phases of discovery)
- Research sources and evidence (Matklad, Rust docs, real projects)
- Performance impact analysis (zero-copy, compilation, binary size)
- Consolidated findings by priority (P0, P1, P2)
- Specific changes required by document
- Migration path and risk assessment

### 3. `proposal.md` (Main Document)
**Purpose**: Formal Sprint Change Proposal for implementation
**Audience**: Implementation teams (Architect, Dev, Tea agents)
**Contains**:
- Issue Summary (Translation Gap anti-pattern)
- Impact Analysis (100% code displacement, stop-the-world)
- Recommended Approach (Strategic Pivot)
- **10 Detailed Change Proposals**:
  1. Workspace Structure (P0)
  2. Error Type Consolidation (P1)
  3. Module Organization Strategy (P1)
  4. Database Layer: Zero-Copy Infrastructure (P0)
  5. CQRS Pattern: Optional Trait Boundaries (P2)
  6. Async vs. Sync Model (P1)
  7. ADR Audit & Cleanup (P2)
  8. Error Handling Standardization (P1)
  9. Naming & Style Cleanup (P2)
  10. Domain Serialization Strategy (P0)
- **5-Phase Implementation Handoff**:
  - Phase 1: Foundation & Documentation (Architect, 1-2 days)
  - Phase 2: Database Layer Implementation (Dev, 2-3 days)
  - Phase 3: Domain Migration & CQRS (Dev, 3-4 days)
  - Phase 4: CLI & Sync Refactor (Dev, 2-3 days)
  - Phase 5: Verification & Validation (Tea, 1-2 days)

---

## Key Metrics

| Metric                | Before (Multi-Crate) | After (Single-Core) | Improvement |
|-----------------------|----------------------|---------------------|-------------|
| Zero-copy read speed  | 50-100ns             | 10-20ns             | 5-10x       |
| Clean build time      | 45-60s               | 30-35s              | 1.5-2x      |
| Binary size           | 8-10 MB              | 6-7 MB              | 25-30%      |
| LSP autocomplete      | 48-52ms (tight)      | 35-40ms (comfortable)| 20-30%     |

---

## Reading Order

**For Quick Understanding**:
1. Start with `discovery-context-summary.md` (5 min read)
2. Read proposal Section 1-3 (Issue Summary, Impact, Approach)
3. Skim proposal Section 4 (10 proposals, focus on P0 items)

**For Implementation**:
1. Read `proposal.md` Section 5 (Implementation Handoff)
2. Reference specific proposals as needed (Section 4)
3. Consult `critical-architecture-review.md` for deep technical details

**For Research/Background**:
1. Read `critical-architecture-review.md` (full conversation flow)
2. Review research sources and evidence
3. Understand performance analysis and trade-offs

---

## Timeline

- **2026-01-30**: Discovery and research (critical-architecture-review.md)
- **2026-01-30**: Course correction workflow execution (proposal.md)
- **2026-02-01**: Approval received from Jack (Product Owner)
- **Next**: Phase 1 implementation (Foundation & Documentation)

**Estimated Duration**: 9-14 days (5 phases)

---

## Success Criteria

From proposal Section 5 (Implementation Handoff):

| Metric                    | Target                              | Validation Method           |
|---------------------------|-------------------------------------|-----------------------------|
| Zero-copy speedup         | 5-10x faster reads                  | Benchmark comparison        |
| Build time                | 1.5-2x faster compilation           | `cargo build --timings`     |
| Test coverage             | Critical paths 100% covered         | Manual review               |
| Code reduction            | -1000+ lines (removed abstractions) | `git diff --stat`           |
| Vault indexing            | <2s for 1000 notes                  | Integration test            |
| CLI operations            | <500ms per command                  | End-to-end test             |
| Quality gates             | `mise run verify` 100% green        | CI pass                     |
| Documentation compliance  | All ADRs validated                  | `mise run adr:validate` pass|

---

## Related Documents

- **ADR 0002**: Storage (redb + rkyv) - Zero-copy requirements
- **ADR 0013**: Domain Serialization - Will be updated to remove serde
- **ADR 0016**: Caching Strategy - Superseded by Proposals 4 & 5
- **New ADR 0017**: Single-Crate Architecture & Database Layer (to be created in Phase 1)

---

## Workflow Compliance

This course correction follows the official workflow:
- **Workflow**: `@_bmad/bmm/workflows/4-implementation/correct-course/`
- **Template**: Sprint Change Proposal (5 sections)
- **Checklist**: All 6 sections completed
- **Approval**: Explicit user approval obtained (2026-02-01)

---

## Notes

- This is a **fundamental re-architecture**, not a simple refactor
- **Stop-the-world event**: No feature work until complete
- All 16 epics affected (scope: ALL)
- Risk level: HIGH (100% code displacement)
- Mitigation: 5-phase approach with rollback strategy

---

**Status**: ✅ **APPROVED - READY FOR PHASE 1 IMPLEMENTATION**
