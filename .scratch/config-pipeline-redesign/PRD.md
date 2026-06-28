---
labels: [ready-for-agent]
---

# PRD: Settings Pipeline Redesign

## Problem Statement

The `crates/settings/` crate has become over-engineered for what it needs to do. It persists resolved config snapshots in a database (redb), tracks config staleness through a typestate processor (`ConfigFileProcessor`), manages vault identity via UUID v7 persisted in the repository, and maintains monotonically incrementing version counters — none of which other CLI tools (mise, chezmoi, starship, helix, ripgrep, bat) do. The repository, processor, views, merger, events, diagnostics, version tracking, and separate DiscoveryPort all exist to support this persistence model and over-abstracted domain boundary, adding ~2500 lines of complexity.

The codebase needs radical simplification: config is ephemeral, re-parsed each invocation, tracked via path-hash symlinks on the filesystem, with a trust/ignore system for security. Discovery is internal to the settings domain, not a separate port. The single entry point is a `SettingsService` following hexagonal architecture.

## Solution

Replace the Repository-backed persistence model with a mise-inspired ephemeral config pipeline:

1. **`SettingsService`** — the sole inbound port for the settings domain. Owns discovery, trust checking, tracking, and config building internally.
2. **Path-hash symlink tracking** replaces the redb Repository for config file tracking
3. **`ConfigBuilder` (typestate)** replaces the generic `Builder<R>`
4. **`TryFrom<RawConfig>`** replaces `ConfigFileProcessor` + `ConfigType` for domain conversion
5. **Unified `RawConfig`** (all `Option` fields, serde Deserialize for TOML/JSON/YAML)
6. **Symlink-based trust/ignore system** for security (mise-inspired)
7. **Discovery is internal** — no `DiscoveryPort`, no `DiscoveryService`. Internal `DiscoveryProcessor` typestate pipeline is kept.
8. **`DiscoveryEnv` → `SettingsEnvVars`** — internal, read by `SettingsService`
9. **`DiscoveryOptions` + `ConfigBuilderOptions`** — split input DTOs from CLI (flags + overrides, NOT env vars)
10. **`BootstrapRunner`** — composition root between CLI and SettingsService
11. **Simplified domain types** — no `VaultId`, `VaultVersion`, `GlobalVersion`, `Version`, `Metadata`, `AppVersion`
12. **Forbidden-field errors** on GlobalConfig/LocalConfig construction — error now, warn later
13. **Cache dir** `<base>/.traces/cache/` is created by `SettingsService::setup_cache_dir(&AppConfig)` after config construction, not by `DiscoveryProcessor` or `DiscoveryOutcome`.
14. **Config file format** — TOML (default), JSON, YAML — all via serde

## User Stories

1. As a traces CLI user, I want `traces trust <path>` to mark a config file as trusted, so that I can use config files with templates/env directives without repeated prompts.
2. As a traces CLI user, I want `traces untrust <path>` to remove trust from a config file, so that I can revoke trust when a config is no longer safe.
3. As a traces CLI user, I want `traces trust --all` to trust all currently untrusted configs in one command, so that I can set up a new vault quickly.
4. As a traces CLI user, I want `traces trust --show` to see which configs are trusted/ignored/untrusted, so that I can audit my security posture.
5. As a traces CLI user, I want `traces trust --ignore` to permanently ignore a config without trusting it, so that I can skip untrusted configs silently.
6. As a developer using `trace-settings` as a library, I want `AppConfig` to be constructable without a database, so that I can use it in tests and non-CLI contexts.
7. As a developer using `trace-settings` as a library, I want `GlobalConfig` and `LocalConfig` to report errors when constructed from a `RawConfig` that contains forbidden fields, so that I catch config file mistakes early.
8. As a developer using `trace-settings` as a library, I want a single `SettingsService` entry point that handles discovery, parsing, and building, so that I don't orchestrate three different components.
9. As a developer contributing to `traces`, I want the settings pipeline to have fewer files and types, so that I can understand the full flow without bouncing between 10+ modules.
10. As a developer testing `trace-settings`, I want to construct `AppConfig` from inline data without filesystem I/O, so that tests are fast and deterministic.
11. As a user running `traces sync` across many vaults, I want config file tracking to avoid database overhead and support cleanup/diagnostics, while config itself remains ephemeral and re-parsed each invocation.
12. As a security-conscious user, I want paranoid mode that verifies content hashes of trusted configs, so that I can detect tampering.
13. As a user, I want global config files (system-level) to be automatically trusted, so that I don't get prompted for OS-managed config.
14. As a user, I want config files to be writable in TOML, JSON, or YAML, so that I can use whatever format I prefer.
15. As a CLI command that only needs the vault root (e.g. `traces sync`), I want to call discovery without building full config, so that I avoid unnecessary I/O.

## Implementation Decisions

### Architecture: Settings Domain (Hexagonal)

```
┌─ CLI layer ────────────────────────────┐
│  BootstrapRunner                        │
│    maps CLI flags → option DTOs          │
│    calls SettingsService                 │
└────────────────────────────────────────┘
         │
         ▼
┌─ settings crate ──────────────────────┐
│  SettingsService (inbound port)        │
│    ┌──────────────────────────────┐    │
│    │  SettingsEnvVars (internal)  │    │
│    │  DiscoveryProcessor          │    │
│    │    + collectors              │    │
│    │  ConfigBuilder (typestate)   │    │
│    │    - Init→Tracked→Trusted    │    │
│    │      →Loaded→Validated→Ready │    │
│    │  Trust (internal)            │    │
│    │  Tracker (internal)          │    │
│    └──────────────────────────────┘    │
│  AppConfig, GlobalConfig, LocalConfig  │
└────────────────────────────────────────┘
```

**Methods on `SettingsService`:**
- `build_config(&self, outcome: DiscoveryOutcome, builder: ConfigBuilderOptions) → Result<AppConfig, SettingsError>` — build config from a discovery outcome
- `discover(&self, options: DiscoveryOptions) → Result<DiscoveryOutcome, SettingsError>` — discovery subset (for commands that only need vault root/config paths)
- `setup_cache_dir(&self, config: &AppConfig) → Result<DirPath, SettingsError>` — creates `<base>/.traces/cache/` after `AppConfig` has been constructed

**No `DiscoveryPort`.** Discovery is an internal module. `DiscoveryService` and `DiscoveryPort` are removed, but the internal `DiscoveryProcessor` typestate pipeline is kept as the core discovery engine. Sub-modules: `input`, `walk`, `probe`, `global`, `filter`, `processor`, `outcome`, `error`. Path definitions consolidate: `location.rs` holds all path constants (marker filenames, boundary markers, tracking/trust subdir names, cache subdirs); `os_dirs.rs` holds XDG + per-platform directory resolution. No more `policy.rs` or scattered path literals.

**`DiscoveryProcessor` is a linear typestate orchestrator.** It uses `transition()` methods (PropertyBankProcessor pattern), not `impl From`, but does not need branch enums. Input normalization happens before the processor starts; filesystem work lives in collector components; the processor sequences local collection → global collection → done.

**`DiscoveryOptions`** (input DTO, constructed by BootstrapRunner for discovery only):
```rust
struct DiscoveryOptions {
    anchor: DirPath,
    flag_vault: Option<DirPath>,
    flag_global: Option<FilePath>,
    suppress_global: bool,      // --no-global
}
```

**`ConfigBuilderOptions`** (input DTO, constructed by BootstrapRunner for config building only):
```rust
struct ConfigBuilderOptions {
    trust_mode: TrustMode,      // normal | paranoid | ci
    auto_confirm: bool,         // --yes
}
```

**`SettingsEnvVars`** (internal, read by SettingsService):
- Formerly `DiscoveryEnv`
- Read from the environment inside `SettingsService::build_config()` and `discover()`
- Includes: `TRACES_DEFAULT_VAULT` fallback directory, `TRACES_GLOBAL_CONFIG` explicit global config file, ceiling paths, etc.

**`BootstrapRunner`** (renamed from `Bootstrapper`):
- Lives in `crates/app/`
- Composition root: maps CLI flags to `DiscoveryOptions` and `ConfigBuilderOptions`, calls `SettingsService`
- No longer takes a `Repository` or `DiscoveryPort`

### Pipeline Flow

```
CLI invocation
  → BootstrapRunner::run()
    → constructs DiscoveryOptions + ConfigBuilderOptions from flags
    → SettingsService::discover(discovery_options)
      → reads SettingsEnvVars (env vars)
      → DiscoveryInput::from_options(discovery_options, env)
      → DiscoveryProcessor::run(input)         // internal orchestration
        → LocalCollect phase                   // mise-style ancestor stack
        → GlobalCollect phase                  // explicit/global config
        → Done → .finish()
        → DiscoveryOutcome { local, global, report }
    → SettingsService::build_config(outcome, builder_options)
      → ConfigBuilder::new(outcome, builder_options)  // Init state
        → .track()                  // Tracked state — checks tracking symlinks
        → .trust()                  // Trusted state — trust_check per candidate
        → .load_files(reader)       // Loaded state — reads + parses RawConfig
        → .validate()              // Validated state — TryFrom<RawConfig>
        → .merge()                 // Ready state — precedence merge
        → .finalize()              // AppConfig
    → SettingsService::setup_cache_dir(&app_config)
```

### DiscoveryProcessor Transition Pattern

`DiscoveryProcessor` follows the `PropertyBankProcessor` transition-method pattern, but the approved flow is linear and branch-free:

```rust
let input = DiscoveryInput::from_options(discovery_options, env)?;

DiscoveryProcessor::<Init>::new(input)
    .collect_local()?    // local stack, outer ancestor → nearest ancestor
    .collect_global()?   // explicit/global config unless suppressed
    .finish()            // DiscoveryOutcome

pub(crate) struct DiscoveryInput {
    anchor: DirPath,
    flag_global: Option<FilePath>,
    flag_vault: Option<DirPath>,
    env_global: Option<FilePath>,        // TRACES_GLOBAL_CONFIG
    env_default_vault: Option<DirPath>,  // TRACES_DEFAULT_VAULT fallback dir
    ceiling_dirs: Box<[DirPath]>,
    suppress_global: bool,
}

pub struct DiscoveryOutcome {
    local: Box<[CandidatePath]>,
    global: Box<[CandidatePath]>,
    report: DiscoveryReport,
}
```

Key points:
- **`transition()`** method (private) constructs the next processor, same as `PropertyBankProcessor::transition()`.
- **No branch enums** unless a real second path appears; env values are normalized inputs, not control-flow states.
- **`SettingsService::discover()`** calls the linear processor; it does not drive individual transitions.
- `Init` owns `DiscoveryInput`; option/env normalization happens before the typestate processor starts.
- Phase marker types remain zero-sized (data lives in the processor struct fields).

### Discovery Collection Rules

Local collection is mise-style layered discovery with Traces constraints:
- Starting anchor is `flag_vault` when set; otherwise `anchor`.
- Enumerate ancestors from the starting anchor to configured ceilings.
- Probe each directory for exact local marker filenames from `location.rs`.
- If ancestor collection finds no local config, probe `env_default_vault` as a fallback.
- Return local config candidates in outer-ancestor → nearest-ancestor order.
- Dedupe canonical/desymlinked paths and filter ignored paths before returning the outcome.

Global collection:
- If `suppress_global`, return empty.
- Else include `flag_global` when set.
- Else include `env_global` when set.
- Else probe platform global config directories from `os_dirs.rs`.
- Dedupe canonical/desymlinked paths and filter ignored paths before returning the outcome.

Component boundaries:
- `processor.rs`: linear typestate orchestration only.
- `input.rs`: `DiscoveryInput` construction and env var names.
- `walk.rs`: ancestor enumeration + local collection.
- `probe.rs`: one-directory exact filename probing.
- `global.rs`: global config collection.
- `filter.rs`: dedupe + ignored filtering.
- `outcome.rs`: `DiscoveryOutcome`, `CandidatePath`, `DiscoveryReport`.

### ConfigBuilder Typestate

| State      | Holds                                    | Transition                              |
| ---------- | ---------------------------------------- | --------------------------------------- |
| `Init`       | `DiscoveryOutcome`                         | → `track()` → `Tracked`                   |
| `Tracked`    | + tracking status per path                 | → `trust_check()` → `Trusted`              |
| `Trusted`    | + trust/ignore status per path             | → `load_files()` → `Loaded`                |
| `Loaded`     | + `RawConfig` values for global + local stack | → `TryFrom` → `Validated`               |
| `Validated`  | + optional `GlobalConfig` + ordered `LocalConfig` stack | → `merge()` → `Ready`       |
| `Ready`      | `AppConfig`                                 | `.finalize() -> AppConfig`                |

- `Tracked` state checks existing tracking symlinks for each candidate path.
- `Trusted` state calls `trust_check()` per candidate — prompts for untrusted, skips ignored.
- `Loaded` state reads and deserializes each TOML/JSON/YAML file into `RawConfig`.
- `Validated` state runs `TryFrom<RawConfig>` → one optional `GlobalConfig` plus an ordered stack of `LocalConfig` values.
- `Ready` merges global first, then local configs in discovery order (outer ancestor → nearest ancestor), so nearest local config wins.
- `merge()` errors when no local candidate exists and a command requires an `AppConfig` with a concrete base. Commands that only need global config should use `discover()`/global-specific handling rather than `build_config()`.
- `Finalize` tracks discovered paths via `Tracker::track()`.

### Types

**`RawConfig`** (unified, replaces `RawGlobalConfig` + `RawVaultConfig`):
- All fields `Option<T>`
- Pure serde Deserialize, no domain logic
- Deserialized from TOML, JSON, or YAML based on file extension
- Fields: logging, cache, template, schema, frontmatter, task, trusted_vaults

**`GlobalConfig`** (was `Global`):
- `path: FilePath`
- `logging: Logging` (required, with default)
- `frontmatter: Frontmatter` (required, with default)
- `template: Option<TemplateConfig>`
- `schema: Option<SchemaConfig>`
- `trusted_vaults: Option<TrustedVaults>`
- `task: Option<Task>`
- `TryFrom<RawConfig>` errors if `cache` is Some (forbidden field)

**`LocalConfig`** (was `Vault`):
- `base: DirPath` (the vault root)
- `path: FilePath` (the config file path)
- `name: Box<str>` (defaults to vault root directory basename)
- `logging: Option<Logging>`
- `cache: Option<CacheConfig>`
- `template: Option<TemplateConfig>`
- `schema: Option<SchemaConfig>`
- `frontmatter: Option<Frontmatter>`
- `task: Option<Task>`
- `TryFrom<RawConfig>` errors if `trusted_vaults` is Some (forbidden field)

**`AppConfig`** (was `Config`):
- `base: DirPath` (from the nearest local `CandidatePath::base`, used for relative path resolution)
- `path: FilePath` (the nearest local config file)
- `logging: Logging`
- `cache: CacheConfig`
- `template: TemplateConfig`
- `schema: SchemaConfig`
- `frontmatter: Frontmatter`
- `task: Task`
- `to_template_spec()`, `to_schema_spec()` etc. use `self.base` for path resolution
- Built via merge function from `(Option<GlobalConfig>, Box<[LocalConfig]>)`

### Config File Format

Serde handles format polymorphism:
```rust
fn deserialize_config(path: &Path, content: &str) -> Result<RawConfig> {
    match path.extension().and_then(|e| e.to_str()) {
        Some("json") => Ok(serde_json::from_str(content)?),
        Some("yaml" | "yml") => Ok(serde_yaml::from_str(content)?),
        _ => Ok(toml::from_str(content)?),
    }
}
```

`serde_json` and `serde_yaml` are optional features (default: TOML only).

### Path Definitions

Path constants split across two files:

**`discovery/location.rs`** — canonical path constants (flat `&[&str]` slices):
- Config marker filenames: exact names like `"traces.toml"`, `"traces.json"`, `".traces/config.toml"` — no `MarkerPattern` struct or format-extension iteration. Probe just checks `path.exists()` for each name.
- Boundary markers: `".git"`, `".workspace"`
- Tracking/trust subdirectory names: `"TRACKED_CONFIGS"`, `"TRUSTED_CONFIGS"`, `"IGNORED_CONFIGS"`
- Cache subdirectory: `".traces/cache"` (relative to vault root)

**`discovery/os_dirs.rs`** — per-platform directory resolution:
- `HOME` — user home, test- redirected to fixtures
- `XDG_CONFIG_HOME`, `XDG_CACHE_HOME`, `XDG_DATA_HOME`, `XDG_STATE_HOME` — per-OS `LazyLock` with platform fallbacks (macOS `~/Library/*`, Windows `%APPDATA%`, Linux `~/.config` etc.)
- What was in `env.rs`'s static section, extracted out

No other module defines filesystem path patterns.

### Removed Types

- `VaultId` — ephemeral UUID, no value without persistence
- `VaultVersion` — monotonically incrementing counter, no value without persistence
- `GlobalVersion` — same
- `Version` (Config aggregate version) — same
- `AppVersion` — just use `env!("CARGO_PKG_VERSION")` where needed
- `Metadata` — merged into LocalConfig as base + path
- `VaultName` — derived from path basename where needed
- `VaultRoot` — replaced by `DirPath` directly
- `DiscoveryEnv` → renamed to `SettingsEnvVars`
- `Vault` → renamed to `LocalConfig`
- `Global` → renamed to `GlobalConfig`
- `Config` → renamed to `AppConfig`

### Removed Modules/Features

- `config/repository.rs` — `ReadRepository`, `WriteRepository`, `Repository` traits
- `config/storage/` — `RedbRepository`, `InMemoryRepository`, tables
- `config/processor.rs` — `ConfigFileProcessor`, `ConfigType`, `ConfigFieldHashes`, `ConfigField`, branch/outcome enums
- `config/views.rs` — `RawGlobalConfigView`, `RawVaultConfigView`, `RawFileVersion`
- `config/merger.rs` — `ConfigResolver`, `ResolutionPlan`
- `config/events.rs` — `Events`, `ConfigUpdated` (no persistence means no event-driven updates)
- `config/diagnostics.rs` — already unused
- `discovery/service.rs` — `DiscoveryService` (orchestration logic moves into `DiscoveryProcessor::run()`)
- `discovery/port.rs` — `DiscoveryPort` (removed, no port needed for internal function)
- `discovery/policy.rs` — `MarkerPattern` struct and constants (folded into `location.rs` as flat `&[&str]`)
- `aggregate.rs` — `Version` removed, `Config` → `AppConfig`, `pending_events` removed
- `vault.rs` — `VaultId`, `VaultVersion`, `VaultRoot`, `VaultName`, `AppVersion`, `Metadata` removed; `Vault` → `LocalConfig`
- `config/global.rs` — `GlobalVersion` removed; `Global` → `GlobalConfig`
- `raw.rs` — `RawGlobalConfig` + `RawVaultConfig` unified into `RawConfig`
- `rkyv` dependency — no persistence means no archive types needed
- `trace-db` dependency — removed from settings crate
- `redb` dependency — removed from settings crate

### Config File Tracking (mise-inspired)

A `Tracker` module (`pub(crate)`) with static methods:
- `track(path)` — creates symlink at `TRACKED_CONFIGS/<path-hash>` → canonicalized path
- `list_all()` — reads all symlinks, resolves targets
- `clean()` — removes dangling symlinks

Path hash uses BLAKE3 or SHA-256 of the canonicalized path string.

### Trust System (mise-inspired)

A `Trust` module with:
- `trust(path)` — creates symlink at `TRUSTED_CONFIGS/<path-hash>` → canonicalized path
- `untrust(path)` — removes trust symlink
- `is_trusted(path)` — checks trust symlink existence (with paranoid mode content hash)
- `trust_check(path)` — interactive prompt on first encounter (guarded by static mutex)
- `ignore(path)` — creates symlink in `IGNORED_CONFIGS/<path-hash>`
- `is_ignored(path)` — checks if a path has been flagged as ignored
- Global config files and `trusted_config_paths` are automatically trusted.
- CI mode trusts everything.
- "Safe config" optimization: configs without templates/env directives skip trust check.

Trust module is `pub` (re-exported for `traces trust` CLI commands), but called internally by SettingsService during `build_config()`.

### Visibility Hardening

The crate defaults to `pub(crate)` and exports only:

| Component              | Visibility | Notes                         |
| ---------------------- | ---------- | ----------------------------- |
| `SettingsService` trait | `pub`      | Inbound port                  |
| `Service` (impl)        | `pub`      | Concrete implementation       |
| `DiscoveryOptions`      | `pub`      | Discovery input DTO           |
| `ConfigBuilderOptions`  | `pub`      | Config-building input DTO     |
| `AppConfig`             | `pub`      | Domain output + spec types    |
| `GlobalConfig`          | `pub`      | Domain type                   |
| `LocalConfig`           | `pub`      | Domain type                   |
| `Trust`                 | `pub`      | For CLI trust commands        |
| `DiscoveryOutcome`      | `pub`      | Returned by `discover()`        |
| `ConfigError`           | `pub`      | Error type                    |
| `ConfigBuilder`         | `pub(crate)` | Typestate builder           |
| `Tracker`               | `pub(crate)` | Internal tracking           |
| `SettingsEnvVars`       | `pub(crate)` | Internal env var reading    |
| `DiscoveryInput`        | `pub(crate)` | Normalized discovery input   |
| `RawConfig`             | `pub(crate)` | Pure DTO                     |

### Cache Dir

`<base>/.traces/cache/` is created by `SettingsService::setup_cache_dir(&AppConfig)` after `AppConfig` is constructed. It uses `AppConfig.base`, not `DiscoveryOutcome`, because the final config owns the path-resolution base. This is not a discovery concern; `DiscoveryOutcome` has no `cache_root` field and `DiscoveryProcessor` has no cache phase. The `Tracker` module separately creates its `TRACKED_CONFIGS/` subdirectory on first use.

### Trust CLI Commands

New CLI commands (in `crates/cli/`):
- `traces trust <path>` — mark config as trusted
- `traces trust --all` — trust all untrusted in hierarchy
- `traces trust --show` — show trust status
- `traces trust --ignore <path>` — add to ignored
- `traces untrust <path>` — remove trust

## Testing Decisions

Good tests:
- **Unit**: Construct `RawConfig` from inline TOML/JSON/YAML, call `TryFrom`, assert correct domain type or error
- **Unit**: Merge `(Some(GlobalConfig), Some(LocalConfig))` → assert vault fields win, global defaults fill gaps
- **Unit**: Merge `(Some(GlobalConfig), None)` → assert global values used
- **Unit**: Merge `(None, None)` → assert all defaults
- **Unit**: `GlobalConfig::try_from` with `RawConfig { cache: Some(...), .. }` → assert forbidden-field error
- **Unit**: `LocalConfig::try_from` with `RawConfig { trusted_vaults: Some(...), .. }` → assert forbidden-field error
- **Unit**: `deserialize_config` with `.toml`, `.json`, `.yaml` → all produce same `RawConfig`
- **Unit**: Trust module tests with tempdir (create/check/untrust/ignore symlinks)
- **Unit**: Tracker module tests with tempdir (track/list/clean symlinks)
- **Unit**: `DiscoveryOutcome` preserves `local` and `global` order with boxed slices
- **Unit**: Local collection returns outer-ancestor → nearest-ancestor ordering
- **Unit**: `TRACES_DEFAULT_VAULT` is used only when normal local collection finds nothing
- **Unit**: Global collection precedence is `flag_global` → `env_global` → platform global dirs, unless suppressed
- **Unit**: `SettingsService::setup_cache_dir()` creates cache from `AppConfig.base`
- **Unit**: `SettingsService` with mock reader → assert `discover()` returns `DiscoveryOutcome` and `build_config()` consumes it
- **Integration**: Full pipeline from tempdir (write TOML files → BootstrapRunner → SettingsService → AppConfig → assert merged values)

Not tested:
- Filesystem I/O paths that are impossible to reach (e.g., read failure after discovery succeeded)
- Version type initial/next tests (removed with types)
- Format polymorphism edge-case deserialization (serde owns this)

Prior art: Existing `build_from_layers_regression` tests in `crates/settings/src/config/builder.rs` show the pattern for merge testing.

## Out of Scope

- FIGMENT integration (ADR 009) — not currently used, no plan to add
- Multi-vault workflows (deferred)
- Config file schema versioning (no project does this)
- WASM target support for settings crate
- Config file watching / hot-reload
- Monorepo-specific trust propagation (mise has this, defer for traces)
- `conf.d/*.toml` drop-in directory support (defer)

## Further Notes

- ADR 009 (Figment) is effectively superseded — the Figment provider pattern was never implemented.
- ADR 016 (Segregated Repository Traits) — the Repository traits themselves go away, but the Read/Write separation principle still applies if any persistence is added later.
- ADR 021 (`app` as composition root) remains valid — BootstrapRunner is the composition root, it just calls `SettingsService::build_config()` instead of `Builder::new().build()`.
- ADR 024 (Bootstrapper as orchestration point) — Bootstrapper → BootstrapRunner rename. Principle unchanged, but no longer takes a Repository or DiscoveryPort.
- ADR config/0001 (Config Builder Decoupling) — superseded. ConfigBuilder typestate replaces the generic `Builder<R>` pattern.
- ADR discovery/0001 (Discovery Service Redesign) — superseded. Discovery is an internal module, not a port/service.
- Settings and Discovery are merged into the same context. CONTEXT.md should reflect that Discovery is internal to Config/Settings, not a peer context.
