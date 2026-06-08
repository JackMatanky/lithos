# Phase 2 Environment Discovery Architecture Refactor Plan

## Goal

Refactor Phase 2 discovery so `discovery/` owns mechanical filesystem path discovery and source precedence, while `config/` receives only selected file paths/metadata required to parse and merge config contents.

## Non-Negotiable Constraints

- All work happens in this worktree only: `.worktrees/root-config-discovery/07-phase-2-environment-config-discovery`.
- No implementation changes without explicit approval.
- `config/` must not receive vault/global source identity, discovery method, runtime source descriptions, or discovery warnings.
- `config/` should receive only selected file path data needed to parse config files: path/pathbuf, base path where needed for local relative path resolution, and format if parsing requires it.
- Discovery warnings are a Discovery concern. If Config needs Discovery warnings, the design is wrong.
- `GLOBAL_MARKER_FILES` keeps its name.
- `GLOBAL_MARKER_FILES` must support both `<base>/lithos.{toml,json,yaml,yml}` and `<base>/lithos/config.{toml,json,yaml,yml}`.
- `GlobalConfigProbe` must be renamed to `GlobalRootProbe` because it checks marker files in directory roots.
- Do not split `probe.rs`; keep vault/global probe types and marker patterns in that file.
- Rename `FoundRootMarker` to `DiscoveredMarker`.
- `GlobalRootProbe` should expose a plain `probe()` method like `VaultRootProbe`; warning handling belongs in `DiscoveryEngine`, similar to how `resolve_ascending()` handles vault warnings.
- `GlobalSourceType` keeps its name.
- The directory category enum must be named `GlobalSourceDirectory`.
- Do not introduce `SelectedConfigPath`; remove `location` from `DiscoveredConfigFile` so the existing handoff type contains only the selected file data Config actually needs.
- Avoid short-term workaround APIs that encode bad architecture for later cleanup.

## Current Problems To Resolve

1. `Builder::load()` passes `flag_path` to `find_vault()` and expects marker probing, but explicit vault source returns `marker: None`.
2. `ConfigDiscoveryResult::from_discovery()` currently accepts discovery source data and warnings; this couples Config to Discovery internals beyond path handoff.
3. `DiscoveryPolicy` only models vault precedence and not global precedence.
4. `GlobalSourceType` mixes source mechanism (`EnvVar`) with directory categories (`XdgConfig`, `UserConfig`, `SystemConfig`).
5. `GlobalDiscoveryInput` is a catch-all of env/XDG/user/system fields.
6. `GlobalConfigProbe` performs probing, mis-case scanning, warning creation, deduplication, and marker construction; it should become `GlobalRootProbe` with a plain `probe()` method, while `DiscoveryEngine` handles warnings.
7. `config/location.rs` and `config/candidates.rs` still generate config candidate paths and perform filesystem discovery work.
8. Tests miss architecture invariants and integration regressions.
9. Case-correction diagnostics are global-only even though mis-cased marker files can affect both vault and global discovery.
10. `resolve_override()` duplicates marker selection/result assembly from `resolve_ascending()` instead of using shared marker assembly.
11. `Builder::load()` accepts a vault root and calls `find_known_vault()`, which bypasses the intended runtime boundary: Discovery should find the vault root before Config loading.
12. `global_precedence` uses `Vec<GlobalSourceDirectory>` instead of `Vec<GlobalSourceType>`, making global precedence structurally different from vault precedence.
13. Vault sources model env/flag paths as directories while global env models a direct file; any direct file override must be explicit rather than hidden in global root discovery.

## Target Architecture

### Discovery Responsibilities

- Generate all mechanical candidate paths.
- Probe filesystem for vault/global marker files.
- Apply source precedence.
- Apply format precedence using `select_candidate()`.
- Emit Discovery-owned diagnostics for CLI/reporting layers, not Config.
- Return selected path records and alternatives.

### Config Responsibilities

- Receive selected global and local config file paths.
- Parse raw config files.
- Merge environment config and local vault config.
- Validate and build resolved config.
- Classify local/global config only if needed for config semantics, not based on source method.

### Builder Responsibilities

- Orchestrate Discovery -> Config pipeline.
- Pass selected file path handoff from Discovery to Config.
- Not hardcode partial runtime global discovery sources.
- Not pass runtime source identity into Config.

## Proposed Data Flow

1. Runtime/CLI/env layer constructs Discovery input.
2. `DiscoveryEngine` returns `VaultDiscoveryResult` and `GlobalDiscoveryResult`.
3. Builder extracts only selected file records from Discovery results.
4. Builder constructs Config input with selected global/local file paths only.
5. Config pipeline reads/parses/merges/validates.
6. Discovery warnings go to CLI/reporting path, not Config.

## Proposed Core Types

These are conceptual target shapes for implementation planning. Final names require approval before code changes.

```rust
pub(crate) struct DiscoveryPolicy {
    pub(crate) vault_precedence: Vec<VaultSourceType>,
    pub(crate) global_precedence: Vec<GlobalSourceType>,
    pub(crate) allow_marker_at_ceiling: bool,
    pub(crate) strict_overrides: bool,
}
```

```rust
pub(crate) enum VaultSourceType {
    ExplicitFlag,
    EnvVar,
    AscendingWalk,
}
```

```rust
pub(crate) enum GlobalSourceType {
    EnvVar,
    Directory(GlobalSourceDirectory),
}
```

```rust
pub(crate) enum GlobalSourceDirectory {
    XdgConfig,
    UserConfig,
    SystemConfig,
}
```

```rust
pub(crate) struct GlobalDirectoryCandidate<'a> {
    pub(crate) directory: GlobalSourceDirectory,
    pub(crate) base: &'a Path,
}
```

```rust
pub(crate) struct GlobalDiscoveryInput<'a> {
    pub(crate) env_path: Option<&'a Path>,
    pub(crate) directories: &'a [GlobalDirectoryCandidate<'a>],
    pub(crate) suppress: bool,
}
```

```rust
pub(crate) struct DiscoveredConfigFile {
    pub(crate) base: PathBuf,
    pub(crate) path: PathBuf,
    pub(crate) format: StructuredFileFormat,
}
```

`DiscoveredConfigFile` should be the Config handoff shape after removing `location`. It already has the fields Config needs and avoids adding a second type with the same meaning.

## File Responsibilities After Refactor

### `lithos-core/src/discovery/policy.rs`

- Owns vault and global discovery precedence.
- Defines source/method enums used only inside Discovery.
- Does not mention Config locations.

### `lithos-core/src/discovery/probe.rs`

- Owns marker pattern constants and directory probing.
- Keeps `GLOBAL_MARKER_FILES` with both supported global marker path forms.
- Renames `GlobalConfigProbe` to `GlobalRootProbe`.
- Keeps both `VaultRootProbe` and `GlobalRootProbe` in this file; do not split `probe.rs`.
- Both probes expose a plain `probe()` method through the same shape.
- Does not emit warnings directly from `GlobalRootProbe`.
- Owns a shared `case_correction_markers(patterns, base)` helper used by both vault and global discovery flows.

### `lithos-core/src/discovery/engine.rs`

- Orchestrates Discovery policy and probes.
- Returns Discovery results with selected marker, alternatives, source, and warnings.
- Uses shared marker selection/alternative assembly instead of duplicating it across vault/global branches.
- Does not expose a runtime `find_known_vault()` shortcut for Builder; runtime callers should use Discovery to find the vault root.
- Orchestrates warning collection around exact probing and shared case-correction marker discovery for both vault and global flows.

### `lithos-core/src/discovery/diagnostics.rs`

- Owns Discovery warnings.
- Not imported by Config.

### `lithos-core/src/config/root.rs`

- Owns Config path handoff input only.
- Does not accept Discovery source enums or Discovery warnings.
- Owns `DiscoveredConfigFile` as a path-only handoff shape, with `location` removed.
- Does not map Discovery source identity into Config location taxonomy.

### `lithos-core/src/config/location.rs`

- Taxonomy only, if still needed.
- No candidate path generation.

### `lithos-core/src/config/candidates.rs`

- Remove or shrink to pure config-level selection if still needed.
- No filesystem candidate generation.

### `lithos-core/src/config/builder.rs`

- Calls Discovery from runtime inputs rather than accepting a pre-discovered vault root.
- Extracts selected paths.
- Passes selected paths only to Config pipeline.

### `lithos-core/src/discovery/selector.rs`

- Owns pure marker selection and selected/alternative splitting.
- Avoid adding a trait until vault/global need different selection policies.

## Task Breakdown

### Task 1: Lock Architecture With Failing Tests

- [x] Add a test showing `Builder::load()` applies local vault config from `<vault>/lithos.toml`.
- [x] Add a compile-level or module-level test proving Config does not import `DiscoveryWarning`.
- [x] Add a test proving Config path handoff does not include vault/global source identity.
- [x] Add a global marker test for `<base>/lithos.toml`.
- [x] Add a global marker test for `<base>/lithos/config.toml`.
- [x] Add a policy test showing global precedence can be customized.

### Task 2: Redesign Discovery Policy

- [x] Replace vault-only `precedence` with `vault_precedence`.
- [x] Add `global_precedence`.
- [x] Split global source method from global directory kind.
- [x] Keep the enum name `GlobalSourceType`.
- [x] Name the directory enum `GlobalSourceDirectory`.
- [x] Update all policy tests.

### Task 3: Redesign Global Discovery Input

- [x] Replace XDG/user/system fields with directory candidates list.
- [x] Keep env file as a direct file source.
- [x] Keep suppression as Discovery input, not Config input.
- [x] Ensure Config never sees these runtime/source structures.

### Task 4: Rename And Simplify Global Probe

- [x] Rename `GlobalConfigProbe` to `GlobalRootProbe`.
- [x] Rename type and tests.
- [x] Keep `GLOBAL_MARKER_FILES` name.
- [x] Add marker patterns for `lithos` and `lithos/config`.
- [x] Make probe responsible only for marker existence/canonicalization.
- [x] Keep the implementation in `probe.rs`; do not split the file.
- [x] Ensure `GlobalRootProbe` has a plain `probe()` method like `VaultRootProbe`.

### Task 5: Move Case-Correction Diagnostics Into Engine-Orchestrated Flow

- [x] Remove `probe_with_warnings()` from the global probe shape.
- [x] Add a `DiscoveryEngine` helper that collects global warnings around the `GlobalRootProbe` call.
- [x] Keep case-correction detection Discovery-owned, but not part of `GlobalRootProbe`'s primary interface.
- [x] Keep warnings in Discovery result.
- [x] Do not pass warnings into Config.
- [x] Add direct Discovery tests for warning emission.

### Task 6: Fix Known Vault Root Discovery

- [x] Add a Discovery API for known vault roots that probes marker files.
- [x] Update `Builder::load()` to use that API.
- [x] Avoid using explicit source behavior as a shortcut if it does not mean marker probing.

### Task 7: Redesign Config Handoff

- [x] Introduce or adapt Config input to accept selected global/local file paths only.
- [x] Remove `location` from `DiscoveredConfigFile` rather than introducing `SelectedConfigPath`.
- [x] Rename Discovery's `FoundRootMarker` to `DiscoveredMarker` and update Discovery internals to use it.
- [x] Remove Discovery source enum imports from `config/root.rs`.
- [x] Remove Discovery warning imports from `config/root.rs`.
- [x] Keep base path only where needed for local relative path resolution.
- [x] Keep format only if parser needs it; otherwise parser can infer from path.

### Task 8: Remove Config-Owned Candidate Discovery

- [x] Remove filesystem path generation from `config/location.rs`.
- [x] Remove or shrink `config/candidates.rs` so it does not scan filesystem.
- [x] Move any remaining candidate-generation tests to Discovery.

### Task 9: Builder Runtime Boundary

- [x] Decide where env/XDG/user/system path construction belongs.
- [x] Ensure that layer passes only Discovery input to Discovery.
- [x] Ensure Config receives only selected paths.
- [x] Remove hardcoded `/etc/lithos`-only partial behavior unless explicitly approved as an out-of-scope placeholder.

### Task 10: Verification

- [x] Run `mise run test:unit:core` in the dedicated worktree.
- [x] Run `mise run lint` in the dedicated worktree.
- [x] Run `mise run fmt` in the dedicated worktree.
- [x] Run GitNexus impact before edits and detect changes before commit.

### Task 11: Latest Review Corrections

- [x] Add shared case-correction marker helper and use it for both vault and global flows.
- [x] Add tests proving mis-cased vault markers emit warnings consistently with global markers.
- [x] Replace duplicated selected-marker/alternatives assembly with a shared selector helper.
- [x] Remove `find_known_vault()` from the runtime Builder path.
- [x] Change `Builder::load()` so it obtains vault root through Discovery rather than accepting an already-known root.
- [x] Change `global_precedence` to `Vec<GlobalSourceType>`.
- [x] Treat global env path as a root directory source for marker discovery, not an implicit direct file override.
- [x] If direct file override remains needed, model it as a distinct explicit source in a later task.

## Open Decisions Required Before Implementation

1. Final values for `GlobalSourceDirectory`: `XdgConfig/UserConfig/SystemConfig` versus shorter `Xdg/User/System`.
2. Exact placement for case-correction helper if it is not a `GlobalRootProbe` method.
3. Whether Config needs `format` in `DiscoveredConfigFile` or can infer format while parsing.
4. Where runtime env/XDG/home/system path construction belongs if CLI wiring is out of scope for this issue.
5. Whether to preserve any direct global config file override in this issue; default is no, because symmetry calls for root-directory discovery.

## Rejected Approaches

- Passing Discovery warnings into Config.
- Adding `RuntimeGlobalConfigPaths` to Config.
- Keeping global source identity in Config handoff.
- Renaming `GlobalSourceType`.
- Introducing `SelectedConfigPath` when `DiscoveredConfigFile` can become the path-only handoff by removing `location`.
- Renaming `GLOBAL_MARKER_FILES`.
- Splitting `probe.rs` as part of this refactor.
- Keeping `FoundRootMarker` as the Discovery marker type name.
- Keeping warning collection in `GlobalRootProbe`'s primary interface.
- Short-term fixes that preserve config-owned path discovery.
- Keeping `find_known_vault()` as a Builder-facing runtime workaround.
- Adding a `DiscoverySelector` trait before vault/global need divergent selection policies.
- Adding extra methods to `DiscoveryProbe` for diagnostics; exact probing and case-correction diagnostics stay separate.

## Errors Encountered

| Error | Attempt | Resolution |
| --- | --- | --- |
| Initial plan passed source/warning data into Config | Review feedback | Revised architecture so Config receives selected path data only |
| Initial plan renamed `GLOBAL_MARKER_FILES` | Review feedback | Keep name and expand patterns |
| Initial plan proposed short-term runtime config shape | Review feedback | Remove Config-facing runtime/source shape |
| `cargo test` with two test-name filters failed because Cargo accepts one filter before `--` | Focused verification | Rerun focused tests one filter at a time |
