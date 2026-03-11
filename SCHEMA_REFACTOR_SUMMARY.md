# Schema Refactor: Executive Summary

**Status**: Ready to Execute
**Date**: 2026-03-06
**Duration**: 46 hours (~6 working days)

---

## What We're Doing

Refactoring the schema module from a **fake DDD aggregate** to a **file-centric read model** with:
- Raw file versioning (5 versions, zstd-compressed, Blake3-hashed)
- Hash-based staleness detection (timestamp fast path, hash slow path)
- Event-driven pipeline (observability for LSP)
- Flat module structure (remove 1204 lines of boilerplate)
- Incremental property resolution (10x faster for small changes)

---

## Why We're Doing It

### Current Problems

1. **`Schema` aggregate has no behavior** - just a parsed data structure pretending to be DDD
2. **1204 lines of wrapper boilerplate** - `query.rs`, `command.rs` do nothing but error conversion
3. **Nested `adapter/` folder** - confusing structure (ports vs adapters vs wrappers?)
4. **No raw file cache** - can't diff changes, can't rollback, can't detect renames
5. **Coarse staleness detection** - timestamp-only misses file identity changes
6. **Full re-resolution on PropertyBank change** - wasteful (should be property-level)
7. **Orchestration in wrong layer** - `application/schema.rs` should be `schema/loader.rs`

### Benefits of Refactor

✅ **Simpler**: 1204 fewer lines, flat structure, clear responsibilities
✅ **Faster**: Hash-based staleness, incremental property resolution (10x speedup)
✅ **Observable**: Event-driven pipeline (ready for LSP integration)
✅ **File-centric**: Raw file versioning enables diffs, rollback, offline work
✅ **Zero-copy preserved**: GATs stay (2-33x faster operations)
✅ **Cohesive**: All schema code in one module (`schema/loader.rs`, not `application/`)

---

## Key Architectural Decisions

| Decision | Choice | Rationale |
|----------|--------|-----------|
| **Wrappers** | Remove | Error conversion via `From` trait (automatic) |
| **Module structure** | Flat | No circular deps, clear separation |
| **Loader location** | `schema/loader.rs` | Cohesion, matches Rust conventions |
| **GATs** | Keep + expand | Zero-copy performance (2-33x faster) |
| **Event storage** | Transient | Sufficient for CLI/LSP, can add persistent later |
| **PropertyBank cascade** | Individual events | Consistent, granular, performant |
| **Validation** | Multi-layer | Size + depth + regex limits prevent attacks |
| **Hashing** | Blake3 | 10x faster than SHA-256, cryptographically secure |
| **Compression** | Zstd level 3 | 60% compression, 500 MB/s encode |
| **Ring buffer** | 5 versions | Covers undo/redo/compare workflows |

---

## Before vs. After

### Module Structure

**Before** (nested, confusing):
```
lithos-core/src/
├── application/
│   └── schema.rs              # Orchestration (343 lines)
└── schema/
    ├── adapter/
    │   ├── query.rs           # Port impl
    │   ├── command.rs         # Port impl
    │   ├── ingestor.rs        # File scanning
    │   └── stored.rs          # Storage types
    ├── query.rs               # Generic wrapper (810 lines)
    ├── command.rs             # Generic wrapper (394 lines)
    ├── aggregate.rs           # Fake DDD aggregate
    └── ...
```

**After** (flat, cohesive):
```
lithos-core/src/
└── schema/
    ├── loader.rs              # Orchestration (was application/schema.rs)
    ├── db_query.rs            # Port impl (was adapter/query.rs)
    ├── db_command.rs          # Port impl (was adapter/command.rs)
    ├── ingestor.rs            # File scanning (was adapter/ingestor.rs)
    ├── stored.rs              # Storage types (was adapter/stored.rs)
    ├── ports.rs               # Port traits (WITH GATs)
    ├── raw.rs, bank.rs, ...   # Domain types
    └── expander.rs, extender.rs, resolver.rs  # Pipeline stages
```

**Lines removed**: 1204 (query.rs + command.rs wrappers)
**Folders removed**: `application/`, `adapter/`

---

### Data Flow

**Before** (opaque):
```
File → Parse → Schema (fake aggregate) → StoredSchema → DB
            ↑ no versioning, no events, no incremental updates
```

**After** (transparent, versioned):
```
File → Hash + Compress → RawSchemaFile (versioned) → DB
    ↓
Parse + Validate → Deref → Extend → Resolve → StoredSchema → DB
                    ↑
              Events emitted at each stage
```

---

## Phase Breakdown

| Phase | Goal | Duration | Key Deliverables |
|-------|------|----------|------------------|
| **Phase 0** | Planning | 2h | Planning docs, decisions, migration plan |
| **Phase 1** | Infrastructure | 8h | Blake3Hash, RingBuffer, raw file tables |
| **Phase 2** | Staleness | 6h | Hash-based staleness detection |
| **Phase 3** | Events | 6h | Event types, handlers, emission |
| **Phase 4** | Incremental | 4h | Property-level re-resolution |
| **Phase 5** | Validation | 4h | Security limits (size, depth, regex) |
| **Phase 6** | Flatten + remove wrappers | 6h | Flat structure, delete 1204 lines |
| **Phase 7** | Remove aggregate | 6h | Delete `Schema` aggregate (BREAKING) |
| **Phase 8** | Documentation | 4h | ADR, rustdoc, diagrams |
| **Total** | | **46h** | File-centric read model architecture |

---

## Breaking Changes (Phase 7 only)

⚠️ **Coordinate with CLI team before Phase 7**

### Public API Changes

**Before**:
```rust
use lithos_core::application::schema::SchemaService;
use lithos_core::schema::{Query, Command, adapter};

let service = SchemaService::new(
    Query::new(adapter::Query::new(&db)),
    Command::new(adapter::Command::new(&db)),
);
service.load(&ingestor)?;
```

**After**:
```rust
use lithos_core::schema::{loader::Loader, db_query, db_command};

let loader = Loader::new(
    db_query::Query::new(&db),
    db_command::Command::new(&db),
);
loader.load(&ingestor)?;
```

### Migration Guide for Users

1. Replace `application::schema::SchemaService` → `schema::loader::Loader`
2. Replace `schema::Query<adapter::Query>` → `schema::db_query::Query`
3. Replace `schema::Command<adapter::Command>` → `schema::db_command::Command`
4. Replace `schema::adapter::*` → `schema::*`
5. Error conversion now automatic (via `From` trait)

---

## Risk Mitigation

### Low Risk (Phases 1-5)
- **No breaking changes** until Phase 7
- **Incremental implementation** with tests at each step
- **Git branches** for each phase (can rollback)
- **CI verification** before merge

### Medium Risk (Phase 6)
- **Breaking changes** to internal APIs
- **Mitigation**: Comprehensive test coverage, coordination with CLI team
- **Rollback**: Restore from git if needed

### High Risk (Phase 7)
- **Breaking changes** to public APIs
- **Mitigation**: Migration guide, backward-compatible shims (if needed)
- **Rollback**: Revert commits, restore old structure

---

## Success Metrics

### Performance
- ✅ Staleness detection: <1 µs per schema (timestamp fast path)
- ✅ Hash computation: <10 µs per schema (slow path)
- ✅ Incremental resolution: 10x faster than full resolution
- ✅ Raw file storage: <10 MB for 1000 schemas (zstd compression)
- ✅ Zero-copy reads: 2-33x faster (GATs preserved)

### Correctness
- ✅ All tests pass (unit + integration)
- ✅ No regressions in existing functionality
- ✅ Security validation prevents attacks

### Maintainability
- ✅ 1204 lines removed (wrappers)
- ✅ No circular dependencies
- ✅ Clear validation boundaries
- ✅ Event-driven pipeline is observable

---

## Documents

1. **`SCHEMA_REFACTOR_PLAN.md`** - Complete 8-phase refactor plan
2. **`SCHEMA_REFACTOR_DECISIONS.md`** - All architectural decisions with rationale
3. **`SCHEMA_REFACTOR_RESEARCH.md`** - Security research + technical proof
4. **`SCHEMA_REFACTOR_MIGRATION.md`** - Step-by-step migration instructions ← **START HERE**
5. **`SCHEMA_REFACTOR_TRACKING.csv`** - Task-level progress tracking
6. **`SCHEMA_REFACTOR_SUMMARY.md`** - This document (executive summary)

---

## Next Steps

1. ✅ Review all documents (especially `SCHEMA_REFACTOR_MIGRATION.md`)
2. ✅ Approve architectural decisions
3. ✅ Create feature branch: `git checkout -b refactor/schema-file-centric`
4. ✅ Copy `SCHEMA_REFACTOR_MIGRATION.md` → `MIGRATION_PROGRESS.md` (for tracking)
5. ✅ Start Phase 1: Infrastructure (raw file storage + Blake3 hashing)

---

## Questions?

- **Architecture**: See `SCHEMA_REFACTOR_PLAN.md`
- **Rationale**: See `SCHEMA_REFACTOR_DECISIONS.md`
- **Technical details**: See `SCHEMA_REFACTOR_RESEARCH.md`
- **Step-by-step**: See `SCHEMA_REFACTOR_MIGRATION.md`
- **Progress tracking**: Use `SCHEMA_REFACTOR_TRACKING.csv`

**Ready to proceed!** 🚀
