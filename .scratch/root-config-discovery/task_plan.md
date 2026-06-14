# DiscoveryProcessor Typestate — Design Plan (Final)

## Constraint: coexist with legacy

We are in the middle of redesigning the discovery module. **Old code stays** — `engine.rs`, `diagnostics.rs`, `DiscoveryPolicy` struct, `VaultRootProbe`/`GlobalRootProbe`, flat error enum — no deletions, no breaking changes to existing call sites. The new processor lives alongside it. Legacy retirement is a separate future issue.

## Processor struct

```rust
struct DiscoveryProcessor<'ctx, P> {
    config: &'ctx DiscoveryServiceConfig,
    ctx: &'ctx DiscoveryContext<'ctx>,
    vault: Vec<CandidatePath>,
    global: Vec<CandidatePath>,
    report: DiscoveryReport,
    phase: P,
}
```

`ctx` carries anchor/flags/env — never extracted. `vault`/`global` are the accumulators.

## Phases

| Phase | Phase data | Transition does |
|---|---|---|
| `Init` | (empty) | Constructor stores refs |
| `FlagOverride` | `{ flag_vault: Option<DirPath>, flag_config: Option<FilePath>, suppress_global: bool, valid_ceilings: Vec<DirPath> }` | Reads `ctx.flags()`. Probes flag vault dir with `FolderProbe{VAULT_MARKER_PATTERNS}` → `vault`. Parses ceilings from `ctx.env().ceiling_dirs_raw()` → `report.skipped_ceilings`. |
| `EnvOverride` | `{ traversal_anchor: DirPath, suppress_global: bool }` | Resolves flag>env for vault anchor (`traversal_anchor`). Resolves config precedence. Sets `report.local_traversal_stop_reason = ExplicitConfigFile` when config present. |
| `AscendingTraversal` | (empty) | Walks from `traversal_anchor` with `FolderProbe{VAULT_MARKER_PATTERNS}`. Fills `vault`. Sets `report.local_traversal_stop_reason`. |
| `GlobalResolution` | (empty) | Probes `config.global_directories` with `FolderProbe{GLOBAL_MARKER_PATTERNS}` → `global`. Sets `report.global_resolution_skip_reason` if suppressed. |
| `Finalized` | (empty) | `finalize() -> (DiscoveryResult, DiscoveryReport)` |

## Port signature

```rust
// port.rs
trait DiscoveryPort {
    fn discover(&self, ctx: &DiscoveryContext<'_>)
        -> Result<(DiscoveryResult, DiscoveryReport), DiscoveryError>;
}
```

No `DiscoveryOutput` struct — return the tuple directly. `bootstrap.rs` pass-through returns the same.

## Branching (in DiscoveryService)

After `EnvOverride`:

```rust
let p: DiscoveryProcessor<EnvOverride> = /* ... */;
match p.branch_strategy() {
    Branch::VaultProbedSkipGlobal => p.into_finalized().finalize(),
    Branch::VaultProbedRunGlobal => p.into_global_resolution()?.finalize(),
    Branch::AscendSkipGlobal => p.into_ascending_traversal()?.finalize(),
    Branch::AscendThenGlobal => p.into_ascending_traversal()?
                                 .into_global_resolution()?
                                 .finalize(),
}
```

`branch_strategy()` returns an enum based on `(has_vault_override, has_config_override, suppress_global)` — all query methods on `EnvOverride` phase data.

## Transitions

| From | To | Error? | Logic |
|---|---|---|---|
| `Init → FlagOverride` | Infallible | Read `ctx.flags()`. Probe flag vault_dir with FolderProbe. Parse ceilings. |
| `FlagOverride → EnvOverride` | Infallible | Resolve vault anchor (flag > env > ctx.anchor()). Resolve config override. Set stop reason if config present. |
| `EnvOverride → AscendingTraversal` | `Result` | Walk from `traversal_anchor`. Stop at marker/ceiling/boundary/root. |
| `EnvOverride → GlobalResolution` | Infallible | Vault already probed, skip local, go direct to global. |
| `EnvOverride → Finalized` | Infallible | Vault already probed + config present → skip everything. |
| `AscendingTraversal → GlobalResolution` | `Result` | Probe global dirs with FolderProbe. |
| `AscendingTraversal → Finalized` | `Result` | Config present → skip global. |
| `GlobalResolution → Finalized` | Infallible | Build output tuple. |

## FolderProbe (probe.rs)

```rust
struct FolderProbe { patterns: &'static [MarkerPattern] }
impl FolderProbe {
    fn probe(&self, dir: &DirPath) -> Vec<CandidatePath> {
        // for each pattern × StructuredFileFormat::PRECEDENCE:
        //   build path, is_file → CandidatePath { base: dir, path }
    }
}
```

Infallible — paths pre-validated. Replaces `VaultRootProbe`/`GlobalRootProbe` logic but old probes stay untouched.

## What changes (new only, no deletions)

| File | Change |
|---|---|
| `discovery/processor.rs` | **New** — typestate processor (private) |
| `discovery/probe.rs` | **Add** `FolderProbe` alongside existing probes |
| `discovery/port.rs` | Update trait sig to `Result<(DiscoveryResult, DiscoveryReport), DiscoveryError>` |
| `discovery/service.rs` | Implement `DiscoveryPort` for `DiscoveryService` |
| `app/bootstrap.rs` | Update `discover()` return type |

## What stays untouched

- `engine.rs` — still there, not called by new code
- `diagnostics.rs` — still there, not called by new code
- `DiscoveryPolicy` in `policy.rs` — still there
- `VaultRootProbe`/`GlobalRootProbe` in `probe.rs` — still there
- All existing `#[allow(dead_code)]` — still there

## Tests

Added to `processor.rs` tests module (uses `DiscoveryContext` directly, no `DiscoveryPort`). Key scenarios:

- Flag vault dir → vault populated, local skipped
- Flag config file → local stop reason set, global runs
- No overrides → ascending walk + global
- Ceiling parsing → skipped ceilings reported
- Boundary marker → probe-then-stop, reason recorded
- Global suppression → skip reason set
- Dedup + ordering in finalization

Bootstrapper tests via `MockDiscoveryPort` in `bootstrap.rs` verify the tuple shape is preserved.
