---
name: freshness-checking-and-full-scan-triggers
status: accepted
date_proposed: 2026-05-28
date_decided: 2026-05-28
stakeholders: [Core Team]
---

# ADR 0005: Freshness Checking by Default with Explicit Full Scan Triggers

## Context

Discovery must scan the filesystem to find new, modified, and deleted files. Scanning 10,000+ files on every invocation is prohibitively expensive (500ms+ of I/O). We need an incremental scanning strategy that only processes changed files.

However, certain conditions require bypassing incremental behavior and forcing a full scan: database corruption, schema migrations, configuration changes, or explicit user request.

The technical forces at play:
- **Performance**: Incremental scans are 100x faster (scan only changed files, not entire vault)
- **Correctness**: Full scans ensure index consistency after corruption or migration
- **User control**: Users must be able to force a full reindex when needed
- **Automatic detection**: System should detect when full scan is required (e.g., config changed)

Terminology is critical: "incremental" is overloaded (incremental scan? incremental updates? incremental builds?). We need precise terminology.

## Decision

**We will perform metadata-based freshness checking by default, with explicit full scan triggers.**

### Terminology (Strict Definitions)

**BANNED**: "Incremental" (overloaded term), "Schema Migration" (ambiguous: user schemas vs internal database schema)

**PRECISE TERMS**:

| Term | Definition |
|------|------------|
| **Freshness Checking** | Built-in `DirScanner` behavior comparing file metadata against `FILE_VIEWS` to yield `New`, `Fresh`, or `Stale` status |
| **Full Scan (Vault)** | Bypass freshness checks globally across entire vault (treat all files as potentially stale) |
| **Full Scan (Context)** | Bypass freshness checks within a specific context directory (e.g., schema files only) |
| **Targeted Scan** | Scan specific directory subtree (e.g., `notes/daily/`) while still performing freshness checks |
| **Event-Driven Scan** | Process specific `FileEvent` list from file watcher (skip directory traversal entirely, future enhancement) |

**Internal Architecture Changes** (precise terminology):
- **Schema Context Update**: User modifies `.md` schema files → standard processor update (NOT "schema migration")
- **Meta-Schema Migration**: Changes to `.schema.json` validation schemas
- **Object Model Migration**: Changes to Rust struct shapes/types (field additions, removals, renames)
- **Internal Database Migration**: Changes to redb table definitions/binary format (triggers full vault scan)

### Full Scan Triggers

**Default Behavior**: Freshness checking (compare `FileView.recorded_at` + `FileView.metadata.size`)

**Full Scan Overrides**:

| Trigger | Scope | Detection | Example |
|---------|-------|-----------|---------|
| **Uninitialized DB** | Full Vault | Automatic | Empty `FILE_VIEWS` table → first run |
| **Explicit `--force`** | Vault OR Context | User CLI flag | `lithos index --force` (vault)<br>`lithos schema --force` (context) |
| **Database Corruption** | Full Vault | Automatic | redb integrity check fails |
| **Internal Database Migration** | Full Vault | Automatic | Version table mismatch vs binary |
| **Config Boundary Changes** | Vault OR Context | Automatic | `DiscoveryConfigSpec` boundary hash changed |

### Config Boundary Change Detection

Config changes that affect discovery behavior (extensions, exclusions) trigger automatic full scans:

```rust
// Config hash state (embedded in config views)
pub struct ConfigHashView {
    pub content_hash: Blake3Hash,           // Full file hash
    pub entry_hashes: BTreeMap<String, Blake3Hash>,  // Per-entry granular hashes
}

// Boundary change detection
let current_spec = config.to_discovery_spec();
let current_boundary_hash = hash_discovery_spec_boundaries(&current_spec);

let stored_hash = repository.load_config_view()?.hash_state
    .entry_hashes.get("discovery_boundaries");

if stored_hash != Some(&current_boundary_hash) {
    // Extensions or exclusions changed → Full Vault Scan required
    requires_full_scan = FullScanScope::Vault;
}
```

**Granularity**: Context-specific boundary changes (e.g., `SchemaConfigSpec.directory` moved) trigger Full Context Scan only, not Full Vault Scan.

## Alternatives Considered

### Alternative 1: Always Full Scan

**Pros**:
- Simple implementation (no freshness tracking)
- Guaranteed consistency (always rebuilds from scratch)

**Cons**:
- Prohibitively slow for large vaults (10k files = 500ms+ scan time)
- Wastes I/O on unchanged files (99% of files unchanged on typical edit)

**Why rejected**: Performance is unacceptable. Scanning 10,000 files to find the 1-2 that changed is wasteful. Freshness checking reduces scan time by 100x.

### Alternative 2: File Watcher Daemon

**Pros**:
- Zero scan overhead (only process changed files from watcher events)
- Instant updates (no scan latency)

**Cons**:
- Requires daemon process (added complexity)
- File watcher reliability issues (missed events on high load)
- Cross-platform challenges (different APIs on macOS/Linux/Windows)
- Stale state on daemon restart (must reconcile watcher state with DB)

**Why rejected**: YAGNI. Freshness checking provides 100x speedup without daemon complexity. File watcher can be added later as an optimization (Event-Driven Scan is already in the API design).

### Alternative 3: Content-Based Hashing (Not Metadata)

**Pros**:
- Detects changes that don't affect metadata (e.g., `touch` without modification)
- More accurate staleness detection

**Cons**:
- Must read every file to compute hash (defeats incremental scan purpose)
- I/O overhead dominates (reading 10k files to hash them = slower than full scan)
- Metadata changes (timestamp + size) are sufficient for 99% of edit patterns

**Why rejected**: Reading file contents to compute hashes negates the performance benefit of incremental scanning. Metadata-based freshness (timestamp + size) catches all real edits (file edits change timestamp and/or size).

## Technical Validation

### Research Findings

- **Existing `DirScanner` analysis**: Current vault processor already performs metadata comparison (`is_timestamp_match()`, `is_size_match()`), confirming this pattern is proven in production.
- **File change patterns**: Normal editing workflows (save file in editor) always update mtime. Edge cases (manual `touch`, clock skew) are rare and can be handled by explicit `--force`.

### Benchmarks

From `.scratch/pipeline-restartability-research.md`:
- Full scan (1000 files): ~100ms
- Freshness checking (1000 files, 10 changed): ~5ms (95% reduction)

### Config Boundary Impact Analysis

Config changes trigger full scans to prevent index corruption:

**Example**: User adds `.org` to extensions. If we only scanned changed files, existing `.org` files would remain unindexed. Full Vault Scan ensures all `.org` files are discovered.

## Consequences

- **Positive**:
  - 100x speedup for typical edits (scan only changed files)
  - Automatic full scan on corruption/migration (no manual intervention)
  - User control via `--force` flag (escape hatch for edge cases)
  - Granular triggers (context-specific vs vault-wide)
  - Config boundary changes auto-detected (prevents silent misconfigurations)

- **Negative**:
  - Metadata false negatives: If file content changes without timestamp/size change (rare), freshness check misses it. User must use `--force`.
  - Full Vault Scan on config changes: Changing extensions = rescan entire vault (potentially expensive). Mitigated by context-specific scans when possible.

- **Risks**:
  - Clock skew: If system clock rewinds, freshness checks may incorrectly mark files as unchanged. Mitigated by size comparison (both timestamp AND size must match).
  - Config boundary hash collisions: Extremely unlikely with Blake3 (2^256 keyspace), but theoretically possible. No practical mitigation needed.

## References

- PRD: `.scratch/centralized-discovery-processor/PRD.md` (Section 8: Reindex Policy)
- Handoff: `/var/folders/9w/3qn47_qj3m9b27gkxwr5_k9m0000gn/T/opencode/handoff-centralized-discovery-continued.md` (Question 6)
- Current Vault Processor: `lithos-core/src/vault/processor.rs` (freshness checking implementation)
- FileMetadata Methods: `lithos-core/src/fs/metadata.rs` (`is_timestamp_match`, `is_size_match`)
