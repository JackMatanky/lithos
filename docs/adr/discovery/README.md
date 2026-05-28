# Discovery Architecture Decision Records

This directory contains Architecture Decision Records (ADRs) for the centralized discovery processor refactor.

## Overview

Lithos previously duplicated filesystem discovery logic across multiple contexts (Schema, Config), creating inconsistent behavior and maintenance overhead. The centralized discovery processor refactor consolidates all filesystem scanning into a single, shared discovery engine that runs once and provides results to all downstream contexts.

## Decision Records (In Order)

### Core Identity & Storage

1. **[0001: FileId as Universal Identity](./0001-fileid-as-universal-identity.md)**
   - Removes context-specific IDs (`SchemaId`, `NoteId`)
   - Uses `FileId` everywhere for file-backed entities
   - Eliminates duplicate path indexes

2. **[0003: Discovery Owns All Vault Tables](./0003-discovery-table-ownership.md)**
   - Discovery is sole writer to vault tables
   - Contexts are read-only consumers
   - Enforced via segregated repository traits

### Pipeline Architecture

3. **[0002: Config as Prerequisite Lens](./0002-config-as-prerequisite-lens.md)**
   - Config resolved BEFORE discovery runs
   - Ascending Discovery algorithm for vault root
   - 5-phase pipeline: Context Resolution → Config → State Rehydration → Discovery → Contexts

4. **[0004: Context-Specific Event Sourcing](./0004-context-specific-event-sourcing.md)**
   - Separate event tables per context (bounded isolation)
   - Projector pattern for crash recovery
   - Intermediate event tracking (full pipeline lifecycle)

### Performance & Optimization

5. **[0005: Freshness Checking and Full Scan Triggers](./0005-freshness-checking-and-full-scan-triggers.md)**
   - Metadata-based freshness checks by default
   - Explicit full scan triggers (uninitialized DB, --force, corruption, config changes)
   - 100x speedup for typical edits

6. **[0006: Parallel Context Processing with MVCC](./0006-parallel-context-processing-with-mvcc.md)**
   - Schema/Note/Template run in parallel
   - Decentralized MVCC commits (no scatter-gather bottleneck)
   - 3x speedup for CPU-bound processing

## Dependencies Between Decisions

```
0001 (FileId Identity)
  ├─> 0003 (Table Ownership)  # FileId enables central path index
  └─> 0004 (Event Sourcing)   # FileId used in event types

0002 (Config as Prerequisite)
  ├─> 0005 (Freshness Checking)  # Config defines discovery boundaries
  └─> 0006 (Parallel Processing) # Config hydration enables phase 5

0004 (Event Sourcing)
  └─> 0006 (Parallel Processing) # Events embedded in typestate transitions
```

## Key Architectural Principles

1. **Bounded Context Isolation**: Each context (Discovery, Schema, Note, Template, Config) owns its own event table and aggregate storage
2. **Single-Writer Consistency**: Discovery is the only writer to vault tables; contexts are read-only consumers
3. **Config-First Execution**: Config must be resolved before discovery runs (config defines the "lens")
4. **Fail-Fast on Errors**: Config errors are fatal; no fallback to defaults
5. **Freshness by Default**: All scans perform metadata-based freshness checks unless explicitly bypassed

## Implementation Status

**Status**: Design fully locked (2026-05-28)

All architectural blind spots resolved. Ready for implementation.

## References

- **Primary PRD**: `.scratch/centralized-discovery-processor/PRD.md`
- **Cross-Platform Paths Research**: `.scratch/CROSS_PLATFORM_PATH_FINDINGS.md`
- **Pipeline Restartability Research**: `.scratch/pipeline-restartability-research.md`
- **Grilling Session Handoff**: `/var/folders/9w/3qn47_qj3m9b27gkxwr5_k9m0000gn/T/opencode/handoff-centralized-discovery-continued.md`
