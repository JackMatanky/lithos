---
labels: [ready-for-agent]
---

# PRD: Config Pipeline Redesign

## Problem Statement

The `crates/settings/` crate has become over-engineered for what it needs to do. It persists resolved config snapshots in a database (redb), tracks config staleness through a typestate processor (`ConfigFileProcessor`), manages vault identity via UUID v7 persisted in the repository, and maintains monotonically incrementing version counters — none of which other CLI tools (mise, chezmoi, starship, helix, ripgrep, bat) do. The repository, processor, views, merger, events, diagnostics, and version tracking all exist to support this persistence model, adding ~2000 lines of complexity.

The codebase needs to be radically simplified: config is ephemeral, re-parsed each invocation, tracked via path-hash symlinks on the filesystem, with a trust/ignore system for security.

## Solution

Replace the Repository-backed persistence model with a mise-inspired ephemeral config pipeline:

1. **Path-hash symlink tracking** replaces the redb Repository for config file tracking
2. **`TryFrom<RawConfig>`** replaces `ConfigFileProcessor` + `ConfigType` for domain conversion
3. **Unified `RawConfig`** (all fields optional) replaces `RawGlobalConfig`/`RawVaultConfig`
4. **Symlink-based trust/ignore system** for security (mise-inspired)
5. **Simplified domain types** — no `VaultId`, `VaultVersion`, `GlobalVersion`, `Version`, `Metadata`, `AppVersion`
6. **`AppConfig`** (was `Config`) drops `vault_metadata`, keeps merged domain fields plus `base: DirPath` + `path: Option<FilePath>`
7. **`GlobalConfig`** (was `Global`) drops `GlobalVersion`, adds `path: FilePath`
8. **`LocalConfig`** (was `Vault`) drops `VaultVersion` + `VaultId` + `Metadata`, adds `base: DirPath` + `path: FilePath`
9. **Cache root** flows from `DiscoveryResult::cache_root()` (already implemented)
10. **Forbidden-field errors** on GlobalConfig/LocalConfig construction — warn later, error now

## User Stories

1. As a traces CLI user, I want `traces trust <path>` to mark a config file as trusted, so that I can use config files with templates/env directives without repeated prompts.
2. As a traces CLI user, I want `traces untrust <path>` to remove trust from a config file, so that I can revoke trust when a config is no longer safe.
3. As a traces CLI user, I want `traces trust --all` to trust all currently untrusted configs in one command, so that I can set up a new vault quickly.
4. As a traces CLI user, I want `traces trust --show` to see which configs are trusted/ignored/untrusted, so that I can audit my security posture.
5. As a traces CLI user, I want `traces trust --ignore` to permanently ignore a config without trusting it, so that I can skip untrusted configs silently.
6. As a developer using `trace-settings` as a library, I want `AppConfig` to be constructable without a database, so that I can use it in tests and non-CLI contexts.
7. As a developer using `trace-settings` as a library, I want `GlobalConfig` and `LocalConfig` to report errors when constructed from a `RawConfig` that contains forbidden fields, so that I catch config file mistakes early.
8. As a developer maintaining `trace-settings`, I want config file discovery reused from the Discovery context (not re-implemented), so that there is one source of truth for where config files live.
9. As a developer contributing to `traces`, I want the config pipeline to have fewer files and types, so that I can understand the full flow without bouncing between 10+ modules.
10. As a developer testing `trace-settings`, I want to construct `AppConfig` from inline data without filesystem I/O or a database, so that tests are fast and deterministic.
11. As a user running `traces sync` across many vaults, I want config files that don't change between runs to parse quickly, so that repeated operations don't stall.
12. As a security-conscious user, I want paranoid mode that verifies content hashes of trusted configs, so that I can detect tampering.
13. As a user, I want global config files (system-level) to be automatically trusted, so that I don't get prompted for OS-managed config.

## Implementation Decisions

### Architecture: Ephemeral Config Pipeline

```
DiscoveryResult
  → [for each candidate: read + parse + trust_check]
    → [TryFrom<RawConfig> for GlobalConfig/LocalConfig]
      → [merge by precedence → AppConfig]
```

No persistence. No staleness detection beyond re-reading the file. No version counters. No repository.

### Types

**`RawConfig`** (unified, replaces `RawGlobalConfig` + `RawVaultConfig`):
- All fields `Option<T>`
- Pure serde Deserialize, no domain logic
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
- `base: DirPath` (the vault root, always present)
- `path: Option<FilePath>` (the local config file, None when only global exists)
- `logging: Logging`
- `cache: CacheConfig`
- `template: TemplateConfig`
- `schema: SchemaConfig`
- `frontmatter: Frontmatter`
- `task: Task`
- `to_template_spec()`, `to_schema_spec()` etc. use `self.base` for path resolution
- Built via merge function from `(Option<GlobalConfig>, Option<LocalConfig>)`

### Removed Types

- `VaultId` — ephemeral UUID, no value without persistence
- `VaultVersion` — monotonically incrementing counter, no value without persistence
- `GlobalVersion` — same
- `Version` (Config aggregate version) — same
- `AppVersion` — just use `env!("CARGO_PKG_VERSION")` where needed
- `Metadata` — merged into LocalConfig as base + path
- `VaultName` — derived from path basename where needed
- `VaultRoot` — replaced by `DirPath` directly
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
- `aggregate.rs` — `Version` removed, `Config` → `AppConfig`, `pending_events` removed
- `vault.rs` — `VaultId`, `VaultVersion`, `VaultRoot`, `VaultName`, `AppVersion`, `Metadata` removed; `Vault` → `LocalConfig`
- `global.rs` — `GlobalVersion` removed; `Global` → `GlobalConfig`
- `raw.rs` — `RawGlobalConfig` + `RawVaultConfig` unified into `RawConfig`
- `rkyv` dependency — no persistence means no archive types needed
- `trace-db` dependency — removed from settings crate
- `redb` dependency — removed from settings crate

### Config File Tracking (mise-inspired)

A `Tracker` module (new) with static methods:
- `track(path)` — creates symlink at `TRACKED_CONFIGS/<path-hash>` → canonicalized path
- `list_all()` — reads all symlinks, resolves targets
- `clean()` — removes dangling symlinks

Path hash uses BLAKE3 or SHA-256 of the canonicalized path string.

### Trust System (mise-inspired)

A `Trust` module (new) with:
- `trust(path)` — creates symlink at `TRUSTED_CONFIGS/<path-hash>` → canonicalized path
- `untrust(path)` — removes trust symlink
- `is_trusted(path)` — checks trust symlink existence (with paranoid mode content hash)
- `trust_check(path)` — interactive prompt on first encounter (guarded by static mutex)
- `ignore(path)` — creates symlink in `IGNORED_CONFIGS/<path-hash>`
- Global config files and `trusted_config_paths` are automatically trusted.
- CI mode trusts everything.
- "Safe config" optimization: configs without templates/env directives skip trust check.

### Simplified Builder

`Builder` (replaces `Builder<R>`):
- No generic `R: Repository` parameter
- `new(vault_candidates, global_candidates)` — no repository
- `build()`:
  1. For each vault candidate: read file → parse RawConfig → trust_check → TryFrom<LocalConfig>
  2. For each global candidate: read file → parse RawConfig → trust_check → TryFrom<GlobalConfig>
  3. Merge GlobalConfig + LocalConfig into AppConfig via precedence layer
  4. Track discovered paths via `Tracker::track()`
- Returns `AppConfig`

### Cache Root

No cache directory from Config. Cache root already available via `DiscoveryResult::cache_root()`. The `to_cache_spec()` deprecation on Config becomes removal.

### Trust CLI Commands

New CLI commands (in `crates/cli/`):
- `traces trust <path>` — mark config as trusted
- `traces trust --all` — trust all untrusted in hierarchy
- `traces trust --show` — show trust status
- `traces trust --ignore <path>` — add to ignored
- `traces untrust <path>` — remove trust

### Visibility Hardening

The current crate has too many `pub` items. The redesign should default to `pub(crate)` and export only:
- `AppConfig` and its spec types
- `GlobalConfig`, `LocalConfig`
- `Builder`
- `Trust` module (trust/untrust/is_trusted)
- Discovery types (already reasonably scoped)
- Error types

Internal types (`RawConfig`, `Tracker`, etc.) should be `pub(crate)`.

### ConfigService Pattern (deferred)

Whether to introduce a `ConfigService` (hexagonal-architecture-style) or use a simpler `AppState` struct is unresolved. The hexagonal architecture guide recommends a Service trait for multi-step orchestration, but the simplified pipeline may not need it. Decision deferred until CLI integration is designed.

### Point 4: Global Cache Directory (unresolved)

The global cache directory (`CacheRoot`) is already resolved via `DiscoveryResult::cache_root()`. The original concern was that "a global cache directory is no longer needed for discovery, but we still need to ensure the local cache directory is created." This needs clarification: is the local cache directory creation the responsibility of Discovery, Config, or the CLI layer?

## Testing Decisions

Good tests:
- **Unit**: Construct `RawConfig` from inline data, call `TryFrom`, assert correct domain type or error
- **Unit**: Merge `(Some(GlobalConfig), Some(LocalConfig))` → assert vault fields win, global defaults fill gaps
- **Unit**: Merge `(Some(GlobalConfig), None)` → assert global values used
- **Unit**: Merge `(None, None)` → assert all defaults
- **Unit**: `GlobalConfig::try_from` with `RawConfig { cache: Some(...), .. }` → assert forbidden-field error
- **Unit**: `LocalConfig::try_from` with `RawConfig { trusted_vaults: Some(...), .. }` → assert forbidden-field error
- **Unit**: Trust module tests with tempdir (create/check/untrust symlinks)
- **Unit**: Tracker module tests with tempdir (track/list/clean symlinks)
- **Integration**: Full pipeline from tempdir (write TOML files → discover → build AppConfig → assert merged values)

Not tested:
- Filesystem I/O paths that are impossible to reach (e.g., read failure after discovery succeeded)
- Version type initial/next tests (removed with types)

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
- ADR 021 (app as composition root) and ADR 024 (Bootstrapper as orchestration point) remain valid — the Bootstrapper still owns the sequence, it just no longer takes a Repository parameter.
- ADR config/0001 (Config Builder Decoupling) — the `from_discovery` → `build` pattern was the right idea but is now simplified since there's no persistence phase.
- ADR discovery/0001 (Discovery Service Redesign) remains valid and unchanged.
