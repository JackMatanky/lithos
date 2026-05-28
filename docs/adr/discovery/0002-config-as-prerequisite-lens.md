---
name: config-as-prerequisite-lens
status: accepted
date_proposed: 2026-05-28
date_decided: 2026-05-28
stakeholders: [Core Team]
---

# ADR 0002: Config as Prerequisite Lens for Discovery

## Context

Lithos previously had a circular dependency: the local configuration file defined the vault root path, but the vault root was needed to locate the configuration file. This created a bootstrapping problem that was hacked around with hardcoded search paths.

The initial centralized discovery design proposed treating config as a discovery consumer: "Discovery scans the vault → finds config files → Config processor parses them." This is architecturally backwards. Configuration defines HOW discovery should run (which file formats to index, which paths to exclude), so config must be resolved BEFORE discovery executes.

The technical forces at play:
- **Bootstrapping problem**: How to find the vault root without config, and config without vault root?
- **Discovery lens**: Config defines extensions (`.md`, `.markdown`) and exclusions (`node_modules/**`, `.git/**`) that discovery must respect
- **Fail-fast vs graceful degradation**: Should corrupted config halt the pipeline or fall back to defaults?
- **Static vs dynamic scope**: Should scan scope (full vault vs targeted context) be stored in config or passed at runtime?

## Decision

**We will resolve config BEFORE discovery runs, using Ascending Discovery to break the circular dependency. Config acts as a static "lens" that discovery consumes.**

### Ascending Discovery Algorithm

Starting from CWD, traverse upward to `/` (or boundary like `.git`), stopping at the first directory containing:
- `lithos.{toml|json|yaml|yml}`
- `.lithos.{toml|json|yaml|yml}`
- `.lithos/config.{toml|json|yaml|yml}`

If no vault found, fall back to global "trusted paths" (e.g., `~/Documents/`). CLI overrides (`--vault <path>`) take precedence over ascending discovery.

### Five-Phase Pipeline

```
1. Context Resolution (stateless I/O)
   ↓
2. Config Hydration (stateless I/O, FAIL-FAST)
   ↓
3. State Rehydration (database)
   ↓
4. Filesystem Discovery (uses frozen config)
   ↓
5. Context Processing (parallel)
```

### Config-to-Discovery Handoff

Config produces a static `DiscoveryConfigSpec`:
```rust
pub struct DiscoveryConfigSpec {
    pub root: VaultRoot,           // From Ascending Discovery (NOT config file)
    pub extensions: Extensions,     // Active file formats
    pub exclusions: Vec<PathPattern>, // User config + implicit (.git, cache_dir)
}
```

Scan scope is a **runtime parameter** (NOT stored in config):
```rust
pub enum DiscoveryScope {
    FullVault { bypass_freshness: bool },
    Contexts { contexts: Vec<ContextScope>, bypass_freshness: bool },
    Targeted { path: PathKey, bypass_freshness: bool },
}
```

### Error Propagation

Config errors are **fatal**. No fallback to defaults, no silent degradation:
```rust
let config = ConfigBuilder::load(vault_root, repository)
    .map_err(PipelineError::ConfigLoadFailed)?;  // ❌ HALT HERE
```

## Alternatives Considered

### Alternative 1: Config as Discovery Consumer

**Pros**:
- Conceptually simple (discovery finds everything, including config)
- Consistent discovery interface (all files discovered the same way)

**Cons**:
- Circular dependency: config defines discovery behavior, but discovery finds config
- Requires hardcoded search paths (violates Ascending Discovery principle)
- Cannot enforce "config-first" execution order

**Why rejected**: The circular dependency is unresolvable. Discovery cannot run without knowing which file formats to index and which paths to exclude—this information lives in config.

### Alternative 2: Default Config with Merge

**Pros**:
- Graceful degradation (pipeline continues even if user config is corrupted)
- User config errors don't halt the system

**Cons**:
- Silent misconfigurations can corrupt the index (e.g., wrong exclusions → indexing `node_modules/`)
- Default config may not match user's vault structure
- Harder to debug (user doesn't know their config was ignored)

**Why rejected**: Silent failures are worse than loud failures. If user config is corrupted, the system should halt and report the error clearly. Falling back to defaults creates subtle bugs that are hard to diagnose.

### Alternative 3: Store Scan Scope in Config

**Pros**:
- Persistent preference (user doesn't need to specify `--force` every time)
- Single source of truth for discovery behavior

**Cons**:
- Couples static config with dynamic runtime behavior
- Cannot override scope per invocation (must edit config file)
- Violates separation of concerns (config = what to index, CLI = when/how to index)

**Why rejected**: Scan scope is a runtime decision (e.g., "force full scan this time" vs "incremental next time"). Storing it in config conflates static preferences with dynamic execution parameters.

## Technical Validation

### Research Findings

- **Cross-platform path analysis** (`.scratch/CROSS_PLATFORM_PATH_FINDINGS.md`): Forward slashes work universally on Windows/macOS/Linux, confirming that `PathKey` can use forward slashes without OS-specific logic.
- **Existing config builder analysis** (`lithos-core/src/config/builder.rs`): Current implementation already performs discovery inside `ConfigBuilder::load()`, confirming the circular dependency exists in production.

### Ascending Discovery Precedent

Git uses a similar algorithm: traverse upward from CWD to find `.git/` directory. This is a proven pattern for locating repository roots without hardcoded paths.

## Consequences

- **Positive**:
  - Breaks circular dependency (config → discovery is now unidirectional)
  - Fail-fast on config errors (prevents silent misconfigurations)
  - Static lens (config) + dynamic scope (CLI) = clear separation of concerns
  - No hardcoded search paths (Ascending Discovery is deterministic)
  - CLI flexibility (can override vault root, scan scope per invocation)

- **Negative**:
  - Phase 2 must complete before Phase 4 (sequential dependency)
  - Config errors are unrecoverable (no graceful degradation)
  - CLI crate not fully set up yet (pipeline design must remain open to prepending Phase 0)

- **Risks**:
  - Ascending Discovery may find unexpected vault (e.g., nested vaults). Mitigated by explicit `--vault` override.
  - Config errors halt pipeline (user cannot proceed). This is intentional—better to fail loudly than silently use wrong config.

## References

- PRD: `.scratch/centralized-discovery-processor/PRD.md` (Section 7: Orchestration Policy)
- Handoff: `/var/folders/9w/3qn47_qj3m9b27gkxwr5_k9m0000gn/T/opencode/handoff-centralized-discovery-continued.md` (Question 5)
- Existing Config Builder: `lithos-core/src/config/builder.rs` (lines 201-272)
