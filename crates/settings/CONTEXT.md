# Settings

The Settings context defines how application settings are discovered from the environment and config files, merged through a precedence chain, validated, and exposed to other contexts. It is the bounded domain behind the `SettingsService` inbound port and owns all discovery, tracking, trust, and building logic internally.

## Language: Config Types

**Raw Config**:
A deserializable DTO (TOML/JSON/YAML) with all-optional fields that is the first domain representation after file parsing. Not validated, not merged.
_Avoid_: config object, validated config, domain config

**Global Config**:
A validated domain type representing system-wide configuration settings applied as the base layer before vault-local overrides. Rejects forbidden fields (`cache`) at construction.
_Avoid_: environment config, system config, raw global

**Local Config**:
A validated domain type representing vault-specific configuration settings that override Global Config values. Rejects forbidden fields (`trusted_vaults`) at construction. Includes `base: DirPath` (vault root), `path: FilePath` (config file), and `name: Box<str>` (defaults to vault root directory basename).
_Avoid_: vault config, vault settings, project config

**App Config**:
The final, validated, merged settings object produced by the ConfigBuilder. Downstream contexts consume this via Config Specs.
_Avoid_: resolved settings, full config, merged config

**Config Spec**:
A narrowed view of App Config exposing only the values needed by a specific downstream context (e.g., `TemplateSpec`, `SchemaSpec`).
_Avoid_: raw settings map, generic config blob

**Precedence Chain**:
The rule that Global Config is the base layer and Local Config overrides it when the same key appears in both sources.
_Avoid_: priority guess, merge magic

**Forbidden Field**:
A config key that is valid in Raw Config but invalid for a particular domain type. `cache` is forbidden in Global Config; `trusted_vaults` is forbidden in Local Config. Construction errors when present, to catch config file mistakes early.
_Avoid_: illegal setting, invalid key

**Declarative Path**:
A configuration value that records an intended file or directory location without asserting that the location exists on disk at configuration time.
_Avoid_: FS-validated path, resolved path, storage key in config

## Language: Discovery (internal)

**Vault Root**:
The directory that bounds a Traces vault and serves as the base for local config path resolution.
_Avoid_: project root, workspace root

**Root Marker**:
A conventional config filename (e.g., `traces.toml`) whose presence in a directory establishes that directory as the Vault Root.
_Avoid_: config location, index marker, marker file

**Candidate Marker**:
A config file found during traversal or global resolution, ordered by source precedence but not yet confirmed. Each Candidate Marker has a type (global or local) determined by its directory.
_Avoid_: discovered config, selected config, result file

**Ascending Walk**:
The traversal strategy that searches parent directories upward from a starting anchor until a Root Marker is found or a Ceiling stops the walk.
_Avoid_: upward scan, directory crawl, recursive search

**Ceiling**:
A directory boundary that stops the Ascending Walk, preventing discovery from searching above a declared limit.
_Avoid_: stop path, upper bound, limit dir

**Override**:
An explicit Vault Root path supplied via CLI flag (`--vault`) or environment variable (`TRACES_VAULT`) that preempts the Ascending Walk entirely.
_Avoid_: forced path, hardcoded path, explicit config

**Discovery Result**:
The output of an internal discovery step: the located Vault Root, ranked global and local Candidate Markers, and the cache root directory. Not a public boundary — consumed internally by the ConfigBuilder.
_Avoid_: discovery output, found configs, resolved paths

**Discovery Report**:
Process metadata produced alongside the Discovery Result, capturing skipped Overrides, skipped Ceilings, and why the Ascending Walk stopped. Consumed by BootstrapRunner for diagnostics only.
_Avoid_: discovery log, diagnostics, discovery warnings

**Settings Env Vars**:
Environment variables that influence discovery behavior: `TRACES_VAULT`, `TRACES_CEILING`, `TRACES_CACHE_DIR`, etc. Read internally by SettingsService during `build_config()`. Not exposed through the public API.
_Avoid_: env config, discovery env, system env overrides

## Language: Pipeline (internal)

**SettingsService**:
The sole inbound port for the Settings domain. Owns the full pipeline: reading Settings Env Vars → Discovery → Trust check → ConfigBuilder → Tracker. Two methods: `build_config()` (full pipeline) and `discover()` (discovery only).
_Avoid_: config service, settings handler, settings orchestrator

**BuildContext**:
The input DTO constructed by BootstrapRunner from CLI flags. Contains anchor directory, optional vault override, trust mode, auto-confirm, and global suppression flag. Does NOT contain environment variables — those are read by SettingsService internally.
_Avoid_: cli input, settings args, pipeline config

**ConfigBuilder**:
An internal typestate builder that sequences config construction through linear states: Init → Tracked → Trusted → Loaded → Validated → Ready. Each state transition consumes the previous state and produces the next.
_Avoid_: config assembler, config factory, settings merger

**BootstrapRunner**:
The composition root (in `crates/app/`) that maps CLI flags to a BuildContext and calls SettingsService. Formerly Bootstrapper.
_Avoid_: main helper, startup orchestrator, app initializer

## Language: Tracking & Trust

**Tracker**:
An internal module that creates path-hash symlinks in `TRACKED_CONFIGS/` for every config file consumed. Provides `track()`, `list_all()`, and `clean()`.
_Avoid_: config registry, file index, symlink manager

**Trust**:
A module that manages config file trust via symlinks in `TRUSTED_CONFIGS/` and `IGNORED_CONFIGS/`. Publicly exposed for `traces trust` CLI commands, but also called internally during ConfigBuilder's trust check phase.
_Avoid_: security module, config verification, authorization

**Trust Check**:
The step in ConfigBuilder (Trusted state) that verifies each Candidate Marker is trusted before loading. Prompts interactively on first encounter. Global configs are auto-trusted. CI mode trusts everything. Configs without templates/env directives skip the check (safe config optimization).
_Avoid_: trust verification, config validation

**Paranoid Mode**:
An operating mode that additionally verifies content hashes of trusted config files to detect tampering. Enabled via `BuildContext.trust_mode`.
_Avoid_: security mode, hash verification, strict mode

## Example Dialogue

> **Dev**: How does a CLI command get its config?
>
> **Domain expert**: BootstrapRunner maps CLI flags to a BuildContext and calls SettingsService::build_config(). The service handles everything internally — reads env vars, runs discovery, checks trust, parses files, and builds AppConfig.
>
> **Dev**: Discovery is no longer a separate context?
>
> **Domain expert**: Correct. Discovery is an internal implementation detail of SettingsService. The Ascending Walk, Ceilings, and Candidate Markers all live within the Settings context. Downstream contexts never see them.
>
> **Dev**: What happens if a config file isn't trusted?
>
> **Domain expert**: ConfigBuilder's Trusted state calls trust_check before loading. The user gets a prompt. If they accept, a trust symlink is created. If they decline, the file is permanently ignored. If they're running in CI mode, everything is trusted automatically.
>
> **Dev**: And config files can be in different formats?
>
> **Domain expert**: TOML, JSON, or YAML — serde picks the format from the file extension. All three produce the same RawConfig DTO.
>
> **Dev**: How does the Tracked state work?
>
> **Domain expert**: After Discovery finds all Candidate Markers, ConfigBuilder records each one in TRACKED_CONFIGS via a path-hash symlink. This lets us list and clean tracked files without a database. Next run, the symlink already exists — no duplicate tracking. The tracked files can be used for diagnostics and cleanup (`traces config clean`).
