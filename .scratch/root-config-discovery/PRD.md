# PRD: Root Config Discovery

**Status**: ready-for-agent
**Created**: 2026-05-31
**Updated**: 2026-06-01
**Triage**: ready-for-agent

---

## Problem Statement

Lithos root and config discovery currently has three interlocking problems.

First, there is a circular dependency: the local config file stores the vault root path, but the vault root is required to locate the local config file. This is resolved today via hardcoded fallback paths, which breaks in non-standard directory layouts and makes the system brittle across platforms.

Second, the current `DiscoveryEngine` receives a `VaultRoot` as an input rather than producing it as an output. This means the caller is already required to know the vault root before discovery can run — which is the problem discovery is supposed to solve.

Third, the existing discovery logic is a flat, internal-only module with no explicit phase boundaries, no typed resolution results, no ceiling/boundary support, no symlink-safe traversal, and no correction diagnostics. It hardcodes only a single local config location and a single global config format, making it non-extensible and untestable at the phase level.

Root and config discovery must be rebuilt as an explicit, phased, typestate pipeline that owns the full responsibility of: (1) finding the vault root, and (2) locating the applicable config files — delivering typed, source-annotated results that downstream config processing can consume without re-running filesystem operations.

---

## Solution

Replace the current `config/discovery.rs` with a dedicated `config/discovery/` submodule that implements a two-phase root-config discovery pipeline:

**Phase 1 — Vault Root Resolution**: Determine the vault root from one of three sources: explicit CLI flag (`--vault`), environment variable (`LITHOS_VAULT_PATH`), or ascending directory walk from the canonicalized CWD. The phase terminates at the first match and returns a typed `VaultRootResolution` that records both the resolved path and the resolution source.

**Phase 2 — Config File Discovery**: Given the resolved vault root (or the absence of one), locate applicable config files from the applicable Config Sources in precedence order. The phase enumerates all candidate paths, checks existence, applies format precedence rules for multi-format ties, emits ambiguity warnings when multiple candidates coexist at the same priority tier, and returns a typed `ConfigDiscoveryResult` carrying the discovered path, format, location type, and base directory.

Both phases produce typed, source-annotated result values. Neither phase parses, validates, or processes config file contents. Discovery ends when file paths are returned. All downstream processing is out of scope.

The pipeline also introduces a minimal `lithos config` subcommand group to the CLI that exposes discovery results directly, enabling developers to validate and debug discovery behavior without running the full pipeline.

---

## User Stories

1. As a CLI user, I want explicit `--vault` and `--config` flags to win immediately and unconditionally, so that my command intent is always respected.
2. As a CLI user, I want a descriptive error when `--vault` or `--config` points to a path that does not exist, so that I know exactly what went wrong and what to fix.
3. As a CLI user, I want vault root discovery to work from any subdirectory of my vault, so that I don't need to be at the vault root to run commands.
4. As a CLI user, I want a `lithos config where` command that shows me exactly what vault root and config files were found and why, so that I can debug config issues without reading source code.
5. As a CLI user, I want `lithos config list-sources` to enumerate every candidate config path that was checked, so that I can understand the full search space.
6. As a CLI user, I want `lithos config check` to validate that discovery is clean, with a `--strict` flag that promotes warnings to errors, so that I can use it in CI.
7. As a CLI user, I want clear, actionable error messages that suggest corrective actions, so that I am never left without a next step.
8. As a config maintainer, I want vault root resolved as a pipeline output, not an input, so that the circular dependency between root and config is eliminated.
9. As a config maintainer, I want the discovery result to carry a `base_path` alongside the config file path, so that downstream relative path resolution is unambiguous.
10. As a config maintainer, I want multiple local config location patterns checked in documented precedence order, so that migration between layout conventions is safe.
11. As a config maintainer, I want multiple structured file formats checked per location with documented precedence, so that format selection is deterministic.
12. As a config maintainer, I want format stability behavior — the previously persisted format winning ties — to be part of the discovery contract, so that format selection does not thrash between runs.
13. As a config maintainer, I want `vault_path` removed from local raw config schema and DTOs, so that local config no longer encodes what should be runtime-discovered context.
14. As a cross-platform user, I want the ascending walk to canonicalize the starting path before traversal, so that symlinks in the CWD are handled correctly on all platforms.
15. As a cross-platform user, I want global config locations to use XDG conventions with OS-appropriate fallbacks, so that the tool integrates naturally on Linux, macOS, and Windows.
16. As a cross-platform user, I want case-sensitivity diagnostics, so that if a mis-cased config filename is detected I receive a corrective warning pointing to the expected filename.
17. As a reliability-focused engineer, I want the ascending walk to maintain a visited-path set, so that symlink loops are detected and the walk terminates cleanly without error.
18. As a reliability-focused engineer, I want a `LITHOS_CEILING` environment variable that terminates the ascending walk at a specified path, so that automated tooling can prevent discovery from escaping a project boundary.
19. As a reliability-focused engineer, I want missing Environment Config to be a non-event, so that discovery succeeds in containerized environments where no global config exists.
20. As a reliability-focused engineer, I want explicit override paths that do not exist to produce an immediate fatal error, so that misconfigured invocations never silently fall back to discovered config.
21. As a reliability-focused engineer, I want ambiguity warnings emitted when multiple candidate config files exist at the same priority tier, so that users are informed about potential confusion without failing the build.
22. As a test author, I want discovery phases to be explicit, typed, and independently testable, so that each edge case can be verified at the correct seam.
23. As a test author, I want the discovery result types to carry resolution source annotations, so that tests can assert not just what was found but how it was found.
24. As an operator, I want trace-level logs at each phase transition and each filesystem check, so that discovery decisions are fully observable.
25. As an operator, I want `--verbose` output on `lithos config where` to show the directories traversed during the ascending walk, so that I can diagnose unexpected root resolution without reading debug logs.
26. As an architecture reviewer, I want root discovery located under `config/discovery/`, so that module ownership is clear and the context boundary is enforced.
27. As an architecture reviewer, I want the discovery pipeline to be strictly separated from config parsing, validation, and hashing, so that each concern can evolve independently.

---

## Implementation Decisions

### Module Structure

- Replace `config/discovery.rs` (flat file) with a `config/discovery/` submodule.
- The new submodule owns: vault root resolution algorithm, ascending walk logic, global config location enumeration, local config candidate generation, format precedence and stability selection, and all discovery result types.
- The existing `config/builder.rs` is a downstream consumer of discovery results; it does not run discovery itself.
- The `lithos-cli` crate gains a `config` subcommand group with three subcommands (`where`, `list-sources`, `check`) that expose discovery results directly without invoking the config processor.

### Phase 1: Vault Root Resolution

The phase evaluates sources in strict precedence order and short-circuits at the first valid match:

1. **CLI flag** (`--vault <path>`): The provided path must exist and be a directory. If it does not exist, emit a fatal error immediately. Do not fall back to discovery.
2. **Environment variable** (`LITHOS_VAULT_PATH`): Same validation rules as the CLI flag. If set to a non-existent path, emit a fatal error.
3. **Ascending walk**: Canonicalize the CWD via `std::fs::canonicalize()` before beginning traversal. Walk upward through ancestors using `Path::ancestors()`. At each directory, check for any recognized config filename marker. Maintain a `seen: HashSet<PathBuf>` of canonical paths to detect and terminate on symlink loops. Stop the walk when the `LITHOS_CEILING` environment variable path is reached, if set.

The phase returns a typed `VaultRootResolution` carrying the resolved path and the resolution source (explicit flag, environment variable, ascending walk, or not found).

**Not found behavior**: When the ascending walk reaches the filesystem root without a match, and no explicit override was given, `VaultRootResolution::NotFound` is returned. This is not immediately fatal — it is propagated to the CLI or orchestrator for contextual handling. The `lithos config where` command exits with code 1 and emits an actionable error; other commands may propagate differently.

### Phase 2: Config File Discovery

Given the outcome of Phase 1, the phase locates applicable config files from the applicable Config Sources.

**Environment Config discovery** (lowest precedence in the Precedence Chain):
Candidates checked in order:
1. `$LITHOS_CONFIG_FILE` (env var, if set and non-empty)
2. `$XDG_CONFIG_HOME/lithos/lithos.{toml,json,yaml,yml}` (if `XDG_CONFIG_HOME` is set)
3. `~/.config/lithos/lithos.{toml,json,yaml,yml}` (user home fallback)
4. `/etc/lithos/lithos.{toml,json,yaml,yml}` (system fallback)

Missing Environment Config at any tier is not an error; the phase continues to the next tier.

**Local (Vault) Config discovery** (highest precedence in the Precedence Chain):
Given the resolved vault root, local config candidates are checked at three location patterns, each with all supported formats, in documented precedence order:
- `LocalConfigLocation::VaultRootFile`: `<root>/lithos.{toml,json,yaml,yml}`
- `LocalConfigLocation::HiddenRootFile`: `<root>/.lithos.{toml,json,yaml,yml}`
- `LocalConfigLocation::ConfigDirectoryFile`: `<root>/.lithos/config.{toml,json,yaml,yml}`

Missing local config is not an error.

**Format precedence** (within a single location): `toml > json > yaml > yml`. This uses `StructuredFileFormat::PRECEDENCE` from `fs/format.rs`, which is already implemented.

**Format stability**: When multiple formats are found at the same location, the previously persisted format (if provided by the orchestrator from the cached view) wins the tie. If no previously persisted format exists, strict precedence applies. When multiple formats are found, always emit a `MULTIPLE_FORMATS` warning regardless of which wins.

**Ambiguity warnings**: When multiple config candidates are found at the same priority tier (e.g., both `.lithos/lithos.toml` and `lithos.toml`), emit a `MULTIPLE_LOCAL_CONFIGS` warning. The highest-priority candidate wins; no error is raised.

**Case-sensitivity diagnostic**: If a mis-cased variant of a recognized config filename is found (e.g. `Lithos.toml`) but the canonical form is not, emit a corrective warning identifying the expected canonical filename.

**`--config <path>` flag**: Bypasses all local config discovery. The provided path must exist. If it does not exist, emit a fatal error immediately.

**`--no-global-config` flag**: Suppresses Environment Config discovery entirely. Only local (vault) config is considered. Useful for isolation testing.

The phase returns a `ConfigDiscoveryResult` carrying:
- `global: Option<DiscoveredConfigFile>` — the selected Environment Config file (if any)
- `local: Option<DiscoveredConfigFile>` — the selected local (vault) config file (if any)
- `warnings: Vec<DiscoveryWarning>` — any ambiguity or diagnostic warnings

Each `DiscoveredConfigFile` carries:
- `path: PathBuf` — absolute path to the file
- `base_path: PathBuf` — the directory from which the config was resolved (required for downstream relative path resolution)
- `format: StructuredFileFormat` — the file format
- `location: ConfigLocation` — the location type (which tier and variant matched)
- `source: ConfigSource` — the resolution source (flag, env var, discovery)

### Removal of `vault_path` from Local Config

`vault_path` must be removed from `RawVaultConfig` and any raw local config DTOs. The vault root is now a runtime-discovered value produced by Phase 1; it must not be encoded in the config file itself.

### Typed Discovery Modes

The pipeline supports two root resolution modes, which are mutually exclusive entry points:

- **Explicit mode**: Entered when `--vault` or `LITHOS_VAULT_PATH` is set. The ascending walk does not run.
- **Ascending mode**: Entered when no explicit override is provided. The walk runs from canonicalized CWD.

This distinction mirrors the `Fixed` vs `Hierarchical` strategy split used by Ruff and Biome, and must be preserved in the result type so that callers can reason about which algorithm was used.

### Discovery Type Contracts

The following types and functions form the testable interface of the discovery submodule. They are named explicitly here so implementation and tests operate at the correct seams.

**`GlobalConfigLocation`** — enum encoding the Environment Config search space. Variants that carry a concrete path (`ExplicitOverride(PathBuf)`, `EnvironmentOverride(PathBuf)`) are semantically distinct from location variants that generate candidates at resolution time (`XdgConfig`, `UserConfig`, `SystemConfig`). This distinction encodes the invariant that an explicit override always has a known path, while fallback locations do not.

**`LocalConfigLocation`** — enum encoding the local (vault) config search space with variants `RootConfigFile`, `HiddenRootConfigFile`, and `ConfigDirectoryFile`. Exposes a `candidate_path(root, format) -> PathBuf` method that generates the concrete candidate path for a given location/format pair. This is the inner generation step and is independently testable.

**`ConfigLocation`** — unified wrapper enum over `GlobalConfigLocation` and `LocalConfigLocation`. This is the type carried in `DiscoveredConfigFile::location`.

**`DiscoveredConfigFile`** — the concrete result of a successful config file lookup. Fields:
- `location: ConfigLocation` — which tier and variant produced this result
- `base: PathBuf` — the directory from which the file was resolved (note: for `ConfigDirectoryFile`, `base` is the vault root, not the `.lithos/` subdirectory)
- `path: PathBuf` — absolute path to the file
- `format: StructuredFileFormat` — the file format

**`DiscoveryWarning`** — typed enum of warning conditions emitted during discovery and carried in `ConfigDiscoveryResult::warnings`. Variants include at minimum:
- `MultipleLocalConfigs` — more than one local config location pattern matched
- `MultipleFormats { location: LocalConfigLocation }` — more than one format variant found at the same location
- `CaseWrongFilename { found: PathBuf, expected: String }` — a mis-cased config filename was detected

**`ConfigDiscoveryResult`** — the final output of Phase 2. Carries:
- `global: Option<DiscoveredConfigFile>`
- `local: Option<DiscoveredConfigFile>`
- `warnings: Vec<DiscoveryWarning>`

**`find_local_config_candidates(root, location) -> Vec<DiscoveredConfigFile>`** — independently testable function: given a vault root and a `LocalConfigLocation` variant, iterate `StructuredFileFormat::PRECEDENCE`, generate each candidate path via `candidate_path()`, check existence, and return all that exist on disk.

**`select_config_candidate(candidates, persisted_format) -> Option<DiscoveredConfigFile>`** — independently testable function: given a list of existing candidates and an optional previously persisted format, apply format precedence and stability rules and return the winner. When `persisted_format` matches an existing candidate, that candidate wins regardless of precedence rank. Otherwise, the highest-precedence candidate wins. Always returns `None` for an empty candidate list.

### Tracing

Emit `tracing` spans at:
- The start of each phase
- Each filesystem existence check during Environment Config discovery
- Each directory visited during the ascending walk
- The selection decision when multiple candidates exist
- Each warning condition

All tracing output uses `tracing` spans and events; no `println!` or direct stderr writes except via the CLI diagnostic layer.

### CLI Surface

The `lithos config` subcommand group provides three subcommands. These commands invoke the discovery pipeline only; they do not parse or validate config file contents.

**`lithos config where`**: Resolves vault root and config files; reports paths, location types, and resolution sources. Exits 0 on success, 1 if vault root not found, 2 if an explicit path argument is invalid, 3 on permission error.

**`lithos config list-sources`**: Enumerates all candidate config paths checked (global and local), with found/not-found status for each. Always exits 0. Never invokes the full discovery engine — pure path enumeration.

**`lithos config check`**: Validates discovery is clean. Reports pass/warn/fail for each condition. With `--strict`, promotes warnings to errors (exit 2).

**Global flags**:
- `--vault <path>` / `-V`: Explicit vault root; bypasses ascending walk.
- `--config <path>` / `-C`: Explicit local config file; bypasses local config discovery.
- `--no-global-config`: Suppress Environment Config discovery.
- `--format <fmt>` / `-f`: Output format for discovery commands (`human`, `json`, `toml`).
- `--verbose` / `-v` (stackable): Step-by-step trace of discovery to stderr.
- `--strict`: (`check` only) Promote warnings to errors.

All trace output goes to stderr. Structured `--format json` output goes to stdout, enabling scripting.

**Exit codes**:
- `0`: Discovery succeeded.
- `1`: Vault root could not be resolved.
- `2`: Explicit path argument (`--vault`, `--config`) does not exist or is the wrong type.
- `3`: Permission error during discovery.

`lithos config list-sources` always returns 0.

---

## Testing Decisions

Good tests assert phase-level outcomes via typed results, not internal helper structure. Tests should assert what was returned and why (source annotation), not how the implementation traversed the filesystem.

**Test scenarios required:**
- Explicit vault flag resolves correctly; missing path produces fatal error.
- Explicit config flag resolves correctly; missing path produces fatal error.
- Env var vault path resolves correctly; missing path produces fatal error.
- Ascending walk finds vault from CWD, from one level up, from multiple levels up.
- Ascending walk terminates at filesystem root without finding a vault (returns `NotFound`).
- Ceiling environment variable terminates walk before reaching a valid vault.
- CWD is a symlink; canonicalization ensures walk proceeds correctly.
- Symlink loop in ancestor chain; walk terminates without error.
- All three local config location patterns found; correct priority winner selected.
- All four format variants found at same location; `toml` wins on fresh run.
- Format stability: `json` wins on second run when previously persisted format was `json`.
- Multiple local config candidates at same tier emit `MULTIPLE_LOCAL_CONFIGS` warning.
- Missing Environment Config at every tier; no error, global returns `None`.
- `--no-global-config` flag suppresses Environment Config entirely.
- Case-wrong config filename detected; corrective warning emitted.
- `ConfigDiscoveryResult` carries correct `base_path` for both global and local results.
- `vault_path` is absent from `RawVaultConfig` deserialization.

**Prior art**: existing config builder tests, `fs/` path validation tests, `StructuredFileFormat` unit tests (`fs/format.rs`).

---

## Out of Scope

- Config file parsing, deserialization, or schema validation.
- Semantic config validation or constraint checking.
- `ConfigHashView` construction or boundary-change detection (see `config-pipeline-refactor` PRD).
- `DiscoveryConfigSpec` construction and handoff to the filesystem discovery processor.
- `extends` chain resolution or config inheritance.
- Settings materialization or `EffectiveSettings` construction.
- Filesystem discovery processor behavior (`FsDiscoveryProcessor`).
- Context routing (schema, note, template orchestration).
- Context-level event sourcing.
- Full pipeline restartability infrastructure.
- `lithos config show` or `--show-config` settings provenance reporting (future).
- Interactive vault initialization (`vault init` prompt behavior).

---

## Further Notes

- This PRD is a prerequisite for the Config Pipeline Refactor PRD, which consumes the typed `ConfigDiscoveryResult` produced here.
- This PRD is a prerequisite for centralized filesystem discovery, which requires a resolved `VaultRoot` and `DiscoveryConfigSpec` as inputs.
- The `StructuredFileFormat` type and `PRECEDENCE` constant are already implemented in `fs/format.rs` (commit `1d6ddc74`). Discovery must use them directly.
- ADR 0002 (`config-as-prerequisite-lens`) documents the architectural rationale for resolving config before discovery. This PRD implements the discovery portion of that decision.
- The research grounding for phase design, algorithm details, and comparative analysis from Cargo, Git, Ruff, Biome, and dprint is preserved in `/var/folders/9w/3qn47_qj3m9b27gkxwr5_k9m0000gn/T/opencode/root-config-discovery-research.md`.
