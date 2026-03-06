# Schema Refactor: Architectural Decisions

**Last Updated**: 2026-03-06

This document tracks key architectural decisions that must be made before implementation can proceed.

---

## Decision 1: Module Structure ⚠️ DECISION NEEDED

**Context**: Where should `stored.rs` live to avoid circular dependencies?

### Option A: Flat Structure (RECOMMENDED)

```
schema/
├── stored.rs          # No dependencies
├── ports.rs           # Imports stored.rs for return types
├── db_query.rs        # Implements ports
├── db_command.rs      # Implements ports
└── ...
```

**Pros**:
- ✅ No circular dependencies
- ✅ Clear separation (data → ports → implementations)
- ✅ Easier navigation (fewer nested folders)

**Cons**:
- ❌ More files at root level (less organization)

---

### Option B: Keep Nested, Move Stored Up

```
schema/
├── stored.rs          # Moved from adapter/
├── ports.rs           # Imports ../stored.rs
└── adapter/
    ├── query.rs
    ├── command.rs
    └── ingestor.rs
```

**Pros**:
- ✅ Maintains some nesting (adapter/ contains implementations)

**Cons**:
- ❌ `stored.rs` feels out of place next to `raw.rs`
- ❌ Still some nesting (less flat than Option A)

---

### Option C: Remove Generic Wrappers

```
schema/
├── ports.rs           # Traits only
└── adapter/
    ├── stored.rs      # Can stay in adapter/
    ├── query.rs       # Concrete impl (no wrapper)
    └── command.rs     # Concrete impl (no wrapper)

// Usage
use lithos_core::schema::{
    ports::{QueryPort, CommandPort},
    adapter::{Query, Command},  // Concrete types
};
```

**Pros**:
- ✅ Simplest (no generic wrappers)
- ✅ No circular dependencies

**Cons**:
- ❌ Couples code to redb (harder to mock)
- ❌ Removes type safety (can't ensure `Query<Q: QueryPort>`)

---

**DECISION**: **Option A: Emit `SchemaEvent::SchemaStale` for Each Affected Schema**
**RATIONALE**:
- Consistency: All staleness events use same type (easy to filter/handle)
- Granularity: Handlers need per-schema detail (which properties changed)
- Performance: 100 events × 1 µs = 100 µs (negligible overhead)
- Simplicity: One event type with clear semantics
- Can add batching later if event volume becomes an issue

**Alternative considered**: Hybrid (rejected: event duplication overhead not justified)

**DATE**: 2026-03-06

---

## Decision 5: Event Storage ✅ DECIDED

**Context**: Store events in database for audit trail, or return transiently?

**DECISION**: **Transient Events** (return from service, no `SCHEMA_EVENTS` table for now)
**RATIONALE**:
- CLI use case: Log to terminal (transient sufficient)
- LSP use case: Broadcast to clients (transient sufficient)
- No audit trail requirement yet
- Can add `SCHEMA_EVENTS` table in Phase 5+ if needed
- Avoids storage overhead and complexity for now

**Future enhancement**: Add persistent event log when audit trail is needed

**DATE**: 2026-03-06

---

## Decision 6: Malicious Content Detection ✅ DECIDED

**Context**: How to detect malicious content in raw schema files?

**DECISION**: **Multi-Layer Validation** (size + depth + regex + limits)
**RATIONALE**:
- File size limit (1 MB) prevents memory exhaustion
- Nesting depth limit (10 levels) prevents stack overflow
- Regex validation prevents ReDoS attacks
- Property count limit (1000) prevents DoS
- String length limits prevent memory exhaustion
- Path traversal already handled by `FsReader::validate_path()`
- TOML/YAML/JSON are safe formats (no code execution)
- Total overhead: ~50 µs per schema (negligible)

See `SCHEMA_REFACTOR_RESEARCH.md` for implementation details.

**DATE**: 2026-03-06

---

## Summary of Decisions

| Decision | Status | Choice | Date |
|----------|--------|--------|------|
| Module Structure | ✅ DECIDED | Option A: Flat | 2026-03-06 |
| Generic Wrappers | ✅ DECIDED | Keep thin wrappers | 2026-03-06 |
| Event Handler Registration | ✅ DECIDED | Dynamic (add_handler) | 2026-03-06 |
| PropertyBank Cascade | ✅ DECIDED | Individual events (Option A) | 2026-03-06 |
| Event Storage | ✅ DECIDED | Transient (no table) | 2026-03-06 |
| Malicious Content | ✅ DECIDED | Multi-layer validation | 2026-03-06 |
| Compression Level | ✅ DECIDED | zstd Level 3 | 2026-03-06 |
| Hash Algorithm | ✅ DECIDED | Blake3 | 2026-03-06 |
| Ring Buffer Size | ✅ DECIDED | 5 versions | 2026-03-06 |
| Per-Property Hashing | ✅ DECIDED | Defer to Phase 3+ | 2026-03-06 |

---

## Next Steps

1. **Make remaining decisions** (module structure, wrappers, events)
2. **Update `SCHEMA_REFACTOR_PLAN.md`** with decisions
3. **Proceed to Phase 1 implementation**
