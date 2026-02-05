---
feature: Config Models (Global + Vault + Merged Config)
status: Draft # Options: Draft, In Review, Approved, Implemented, Archived
author: Jack Matanky (drafted with GitHub Copilot)
ticket: TBD
date_created: 2026-02-04
tags: [config, domain-models, cqrs, rkyv, invariants]
---

# Tech Spec: Config Models (Global + Vault + Merged Config)

> **Note**: See `docs/design/README.md` for usage instructions.

Related specs:

- [docs/design/002-config-cqrs.md](002-config-cqrs.md) (how config models are persisted and retrieved)
- [docs/design/006-task-management-system.md](006-task-management-system.md) (a major consumer of config-driven schemas/values; keep config model decisions compatible with planned task config)

## 1. Problem Space (The "Why")

### 1.1 Context & Background

The config bounded context defines how Lithos discovers and resolves configuration for a vault operation.

Current implementation lives in:

- `lithos-core/src/config/aggregate.rs` (`Config` aggregate root; merge + validation)
- `lithos-core/src/config/global.rs` (`Global` configuration)
- `lithos-core/src/config/vault.rs` (`Vault` configuration + `Metadata`)
- `lithos-core/src/config/types.rs` (shared config types: `Frontmatter`, `Logging`, `Schema`, `Template`, `SettingValue`)
- `lithos-core/src/config/error.rs` (`ConfigError`)
- `lithos-core/src/config/events.rs` (domain events)

High-level business rules:

- **Precedence**: vault-specific overrides beat global values.
- **Defaults**: system defaults apply when neither level provides a value.
- **Validation**: invalid configs should be rejected at build time.

Design tensions motivating this spec:

- Several configuration fields are represented as `String` with a post-hoc `validate()` (empty-string sentinel), which makes invalid states representable and complicates merging.
- Path-like fields are treated as generic strings; joining and validation are therefore brittle.
- Some “mutually exclusive” shapes are modeled as `Option` pairs that require runtime validation.

Terminology note:

- “Core substrate” (informal) means a foundational type/module that becomes widely reused across many contexts. The risk is making a type “too central” too early, which can lock in awkward constraints.

Clarification:

- `SettingValue` is **owned by the config context** (it is part of the config domain model and persisted config surface).
- Other contexts may *consume* `SettingValue` **only when interpreting config-defined dynamic values** (e.g., config-driven task metadata values), but should not treat it as a universal cross-context primitive.
- If a bounded context needs its own dynamic value model for non-config concerns, prefer defining a context-local type (e.g., note frontmatter’s `FieldValue`) and converting at the boundary.

Serialization boundary note (serde vs rkyv):

- The current code derives both `serde` and `rkyv` on many config types.
- Going forward, treat these as two distinct concerns:
  - `serde` is needed for **config file I/O** (TOML/YAML/JSON) and human-facing interchange.
  - `rkyv` is needed for **redb persistence** (fast, stable archived bytes).
- If the domain model becomes awkward due to serialization constraints, introduce explicit DTOs:
  - `*File` types (serde-only) for parsing,
  - validated domain types for in-memory logic,
  - persisted record types (rkyv, optionally serde if you want to debug-dump).

### 1.2 Goals & Non-Goals

**Goals**

- Define the **model contracts** for Global/Vault/Merged config:
  - responsibilities, invariants, and intended construction flows.
- Strengthen type-driven design where it reduces invalid states:
  - prefer `Option<T>` over empty-string sentinels,
  - use enums/newtypes for constrained values (log level, config source, etc.),
  - use path types for path operations.
- Keep persisted-format concerns explicit:
  - rkyv derives exist today; treat layout changes as on-disk format decisions.

**Non-Goals**

- Defining the persistence strategy and CQRS behavior in detail (covered by `docs/design/002-config-cqrs.md`).
- Introducing async APIs in core.

### 1.3 Constraints (The Hard Limits)

- **Sync-first core**: config resolution must remain synchronous.
- **Persisted bytes contract**: changes to rkyv-archived model layouts are treated as migrations.
- **No hidden allocations** in getters: prefer borrowed accessors or clear "owned" naming.

## 2. Guide-Level Explanation (The "What")

### 2.1 User/Dev Experience

Most callers should not manually merge config fields. The normal workflow is:

1) Load global and vault configs.
2) Build a merged, validated `Config` aggregate.
3) Use the merged config as immutable runtime input.

Sketch (current shape):

```rust
use lithos_core::config::{aggregate::Config, global::Global, vault::Vault};

let global = Global::default();
let vault = Vault::default();

// The merged aggregate is validated during build.
let merged = Config::build(Some(&global), "/vault", vault)?;

// Use merged values (frontmatter/logging/filesystem paths).
let _log_level = merged.logging.log_level.as_str();
```

### 2.2 Mental Model

Think of config as a two-layer overlay:

- **Global**: lowest precedence. System defaults and user-wide defaults.
- **Vault**: highest precedence. Overrides for the specific vault.
- **Merged Config**: a computed, immutable aggregate used for the current operation.

The key property is that consumers hold **one** `Config` value with all invariants satisfied.

Additional runtime/persistence mental model (for caching + rollback):

- A vault has a **stable identity** (`VaultId`) and a **current location** (vault root path).
- For each vault, we can persist **versioned merged configs** as a read model.
  - There is exactly one **active** merged config per vault at a time.
  - Older versions are retained to support rollback.
  - This “singleton” notion exists at the storage/read-model level (per vault), not as a global singleton in the domain.

## 3. Detailed Design (The "How")

### 3.1 System Architecture

```mermaid
flowchart LR
  Caller[App/CLI] --> Qry[config::query::Query]
  Caller --> Cmd[config::command::Command]
  Qry --> Agg[Config::build]
  Agg --> Merged[Merged Config]
  Cmd --> Store[(Database)]
  Qry --> Store

  Store -->|active version| Merged
  Store -->|version history| Merged
```

### 3.2 Component & Interface Specifications

#### Component: `Global`

- **Responsibility**: provide the global-level configuration layer.
- **Invariants**:
  - its sub-structures are valid (`Paths`, `Frontmatter`, `Logging`).
  - optional `trusted_vaults` is either absent or valid.

#### Component: `Vault`

- **Responsibility**: provide the vault-level configuration layer (overrides).
- **Invariants**:
  - required vault identity is provided by `Metadata` during merge/build (not necessarily stored in `Vault` itself).
  - override fields should be represented as `Option<T>` (preferred) rather than empty strings.

Vault identity:

- The unique, primary key for a vault is `VaultId` (see 3.4.2).
- The vault root path is the vault’s **current location**; persistence may also keep a canonical `VaultPathKey` mapping to support lookups by path.

#### Component: `Config` (aggregate)

- **Responsibility**: represent the merged, validated runtime configuration.
- **Public Interface**:
  - `Config::build(global: Option<&Global>, vault_path: &str, vault: Vault) -> Result<Config, ConfigError>`
  - `validate(&self) -> Result<(), ConfigError>` (redundant if `build` always validates; keep if useful as a defensive check)
  - `pending_events() -> &[Events]` and `take_events()` for event staging.

- **Invariants**:
  - all required fields are non-empty and internally consistent.
  - constrained values (e.g., log level) are valid.

### 3.3 Integration & Data Flow

Sequence: resolve config for a vault operation.

```mermaid
sequenceDiagram
  participant Caller
  participant Q as config::query::Query
  participant Agg as Config::build
  participant DB as Database

  Caller->>Q: load_global()
  Q->>DB: get_owned("config","global")
  DB-->>Q: Option<Global>

  Caller->>Q: load_vault()
  Q->>DB: get_owned("config","vault")
  DB-->>Q: Option<Vault>

  Caller->>Agg: build(global, vault_path, vault)
  Agg-->>Caller: Config (merged + validated)

If a versioned merged read model is used (recommended for per-vault cache/rollback), the merged `Config` produced above becomes the value that is persisted as a new version.
```

### 3.4 Data Models

This section records the important model decisions.

#### 3.4.1 Type-driven upgrades (recommended)

These changes are recommended because they reduce invalid states, simplify merging, and make the model easier to evolve.

- **Remove empty-string sentinels from the domain**
  - Today, merge uses empty-string checks (`choose_value`) and many structs allow empty strings.
  - Prefer representing “unset/override not provided” as `Option<T>` in the *override* layers.

- **Introduce validated newtypes for repeated invariants**
  - `NonEmptyString` (or more specific types like `FrontmatterKey`, `DirName`, `FileName`).
  - `VaultRoot(PathBuf)` with validation rules.
  - `CacheDir(RelPath)` vs a generic string.

- **Constrain enum-like strings**
  - `Logging.log_level: String` should be `LogLevel` enum.
  - `ConfigUpdated.source: String` should be `ConfigSource` enum.

- **Fix path composition**
  - `Schema::property_bank_path() -> String` should return `PathBuf` (or `VaultRelativePath`) and use join semantics, not string formatting.

- **Model mutual exclusivity as an enum**
  - `TrustedVaults { list: Option<_>, map: Option<_> }` should be `enum TrustedVaults { List(Vec<VaultRoot>), Map(HashMap<Box<str>, VaultRoot>) }` with `#[serde(untagged)]`.

- **Avoid `Deref` for semantic newtypes**
  - `SchemaVersion(pub String)` currently implements `Deref<Target = str>`.
  - Prefer an explicit `as_str()` (and `Display`) so the type boundary stays visible.

#### 3.4.2 Vault identity (required)

- **Decision (project convention)**: introduce a stable `VaultId(Uuid)` and use it as the primary vault identifier.

Rationale:

- Note and Schema use stable identifiers; config should follow the same convention.
- A stable ID makes “vault moved/renamed” a *representable* case (even if not always automatically discoverable).

Required types (recommended):

- `VaultId(Uuid)` (stable identity)
- `VaultRoot(PathBuf)` (current location, validated)
- `VaultPathKey(Box<str>)` (canonical encoding used for lookup/mapping)

Discovery/persistence rule:

- A stable ID only helps with vault moves if it can be re-discovered at the new path.
- Therefore, the vault must persist its ID somewhere stable (e.g., `.lithos/vault-id`), or the application must otherwise be able to prove the new path corresponds to the same vault.

Storage invariants (recommended):

- `vault_id_by_path: VaultPathKey -> VaultId`
- `vault_path_by_id: VaultId -> VaultPathKey`

Canonicalization rules (recommendation):

- Use an absolute path.
- Normalize separators (platform default is fine as long as the encoding is stable).
- Consider resolving symlinks only if you intentionally want “symlinked paths” to collapse to the same vault identity.

#### 3.4.3 Versioned merged config read model (recommended)

To support a single-vault cache and rollback, persist merged configs as immutable versions.

- `ConfigVersion(u64)` (monotonic, per vault)
- `MergedConfigRecord { vault_id: VaultId, version: ConfigVersion, created_at: i64, config: Config }`
- `ActiveMergedConfig { vault_id: VaultId, version: ConfigVersion }`

Design rule: keep “exactly one active merged config per vault” as a storage invariant, not a global domain singleton.

### 3.5 Core Logic & Algorithms

Merge rule: vault overrides global; defaults apply when neither provides a value.

Design rule:

- Merging should be expressed as `Option` selection (`vault.or(global).unwrap_or(default)`) rather than checking for empty strings.

Rollback rule (if versioned merged configs are persisted):

- Rollback is implemented by selecting an older `ConfigVersion` as active for the given vault.
- The merge algorithm must be deterministic so that “rebuild” produces predictable versions.

### 3.6 Recommended Code Modularization

The current config context is functional, but `types.rs` is doing too much and the validation/merge story is spread across several structs.

Constraints respected:

- Keep `command.rs` and `query.rs` at the root of the context.

Recommended module layout (target):

Design rule (idiomatic Rust): prefer a **flat module tree**. Only introduce nested directories when there is sustained pressure (many closely related modules) and the extra hierarchy meaningfully improves navigation.

Target layout (flat, file-per-module):

- `lithos-core/src/config/mod.rs`
  - public re-exports for the main types (`Config`, `Global`, `Vault`, errors)

- `lithos-core/src/config/aggregate.rs`
  - `Config` (merged runtime aggregate)
  - merge/build logic lives here (this is the primary implementation of `Config`)
  - private helper fns for overlay selection (no empty-string sentinels)

- `lithos-core/src/config/global.rs`
  - `Global` + global-only shapes (e.g., `TrustedVaults`)

- `lithos-core/src/config/vault.rs`
  - `Vault` + `Metadata`
  - vault identity/value types that are only meaningful in the vault model:
    - `VaultId`, `VaultRoot`, `VaultPathKey`
    - `SchemaVersion` (if it remains a vault-local concept)

- `lithos-core/src/config/frontmatter.rs`
  - `Frontmatter` + validated newtypes like `FrontmatterKey`

- `lithos-core/src/config/logging.rs`
  - `Logging` + `LogLevel`

- `lithos-core/src/config/schema.rs`
  - `Schema` + schema-related validated value types

- `lithos-core/src/config/template.rs`
  - `Template` + template-related validated value types

- `lithos-core/src/config/setting_value.rs`
  - `SettingValue` (config-specific) and any focused value newtypes/enums it decomposes into

Notes:

- If you want to keep the public surface stable while splitting, `types.rs` can temporarily become a thin re-export module (or be deleted once call sites migrate).
- If `ConfigVersion` and versioned merged-record structs are introduced, place them near their primary owner:
  - if they are aggregate-level read models, keep them in `aggregate.rs` (or `query.rs` if they are strictly query-facing persisted records).

This approach keeps the context readable:

- merge stays with the aggregate (where it is easiest to understand and test),
- vault-only identity types live with `Vault` (so the meaning is obvious), and
- “big types.rs” is decomposed without creating an over-nested module tree.

### 3.7 Validation-in-Types (keep public APIs clean)

Goal: callers should not have to remember to call `.validate()` everywhere.

Recommended pattern:

- Make “always-valid” domain types constructible only through fallible constructors (`try_new`, `new_checked`) or `TryFrom`.
- Use serde integration to validate at deserialization time.
- Keep `validate()` as `pub(crate)` (or omit it) once construction is guaranteed to produce valid instances.

Serde patterns:

- `#[serde(try_from = "String")]` on newtypes like `FrontmatterKey`, `DirName`, `FileName`.
- `#[serde(untagged)]` enums for mutually exclusive shapes (`TrustedVaults`).
- `#[serde(default)]` + `Option<T>` overlays for vault overrides.

rkyv patterns:

- Only require `rkyv` on types that actually cross the DB boundary.
- If serde-friendly shapes diverge from archive-friendly shapes, introduce explicit DTOs at the boundary rather than contorting the core model.

## 4. Alternatives & Decisions (The "Divergence")

### 4.1 Tactical Decisions

#### Decision: Use `Option` overlays rather than empty strings

- **Context**: merging currently uses empty-string sentinels in several places.
- **Choice**: represent “unset” as `None` instead.
- **Alternatives Considered**:
  - _Empty-string sentinel_: easy to deserialize but makes invalid states representable; complicates merge logic. Rejected.

## 5. Operational Readiness (The "Reality Check")

### 5.1 Observability

- Emit a config-loaded/config-merged log at the application boundary (CLI/app), not inside core models.
- Consider structured logging of:
  - which layer provided each value (global/vault/default),
  - config version, and
  - config source.

### 5.2 Migration Strategy

- Any rkyv layout change for persisted config models should be treated as a migration event.
- Prefer introducing persisted DTOs if the model churn becomes frequent.

### 5.3 Security & Privacy

- `SettingValue::Encrypted` must never reveal raw bytes in debug output (current behavior masks bytes).
- Encryption/decryption belongs at the adapter boundary; core stores opaque bytes.

## 6. Pre-Mortem (The "Inversion")

- **Risk**: configuration merge silently accepts invalid data because “unset” is represented as empty strings.
  - _Mitigation_: represent absence as `Option` and make invalid states unrepresentable.

- **Risk**: path concatenation via string formatting produces invalid paths on Windows or with trailing slashes.
  - _Mitigation_: use `PathBuf` joins at the model boundary.

- **Risk**: persisted rkyv layout changes break existing on-disk configs.
  - _Mitigation_: treat layout changes as migrations; consider persisted DTO boundaries.

## 7. Critique & Refinement Log

| Date       | Critique / Issue                              | Resolution                                      |
| :--------- | :-------------------------------------------- | :---------------------------------------------- |
| 2026-02-04 | "Merge uses empty-string sentinels."         | "Recommend Option overlays; see Decision 4.1." |
| 2026-02-04 | "Paths are generic Strings."                 | "Recommend PathBuf/newtypes; see 3.4.1."      |

## Appendix A: External Patterns (Figment + Layered Config) and What Lithos Should Steal

This appendix summarizes real-world patterns from Rust projects using layered configuration (often via Figment), and translates them into actionable guidance for Lithos’ two-tier config model (Global + Vault → Merged Config).

The point is not to copy any one project’s approach verbatim; it is to extract the repeatable, “battle-tested” ideas that reduce surprises:

- deterministic precedence (“last write wins” with a documented order)
- optional profile/context selection implemented consistently
- clear “where did this value come from?” diagnostics
- a clean separation between unvalidated input trees and validated runtime configs

### A.1 Figment Capabilities Worth Designing Around

Even if Lithos does not expose Figment directly as a public API, Figment’s model is a strong mental model:

- **Providers**: each configuration source is a provider (defaults, files, env vars, CLI overrides, etc.).
- **Merge**: configuration is composed by merging providers into a single tree.
  - Rule of thumb: *later merges override earlier merges* (for scalars).
  - Complex shapes (tables) are generally merged key-by-key.
- **Profiles**: a single provider can contain multiple profiles (e.g., `debug`/`release`, `dev`/`prod`). The selected profile determines which values are active.
- **`nested()` vs non-nested file providers**:
  - `nested()` treats the file as containing profiles at the top-level.
  - This matters when you want context-specific config from the same file.
- **Error metadata**: Figment’s extraction errors can carry rich metadata (selected profile, key path, and sometimes source info).

Design implication for Lithos:

- Config should be representable as **multiple explicit sources** merged into an intermediate representation.
- The “final product” for business logic should remain a **validated merged aggregate** (the `Config` aggregate), regardless of how many sources fed it.

### A.2 Case Studies

#### A.2.1 Rocket: canonical example of layered defaults + file + env

Rocket’s configuration guide is one of the clearest, widely-used demonstrations of Figment layering.

Key patterns:

- **Document a strict order**: Rocket effectively composes `defaults → Rocket.toml → env vars`.
- **Profiles**: Rocket supports named profiles (`debug`, `release`, etc.) and also uses two “meta-profiles”:
  - `default`: inherited by all other profiles (base values)
  - `global`: overrides all profiles

What Lithos should steal:

- **The concept of “default vs global” as semantics** is extremely useful.
  - Lithos already has “system defaults” plus “global config”; Rocket shows a proven way to explain these.
- **Be explicit about precedence**.
  - Avoid “it depends” layering. Users interpret config systems as a predictable override chain.

What Lithos should *not* steal:

- Rocket’s profiles are “runtime context” (debug/release), not “per-vault identity.” Don’t model one profile per vault.

#### A.2.2 apple-codesign (`rcodesign`): multiple config files + env + CLI, with profiles

The `rcodesign` CLI builds config by merging multiple sources:

- **Implicit config file search** (user config dir + current directory) or **explicit config file list**.
- **Profile selection** (defaults to `default`, but supports selecting a named profile).
- **Environment variables override everything**.
- **CLI args can be merged** by serializing a config struct into the selected profile.

Two details are particularly valuable:

1) **Multiple config files are merged in order**.
   - This pattern makes it easy to support: “corporate base config” + “user config” + “project config.”

2) **User-facing error reporting prints**:
   - selected profile
   - problem key path
   - source file path (when available)

What Lithos should steal:

- Supporting multiple config files (optional) is an easy “escape hatch” that plays well with your two-tier model.
  - Example shape: global can itself be composed from multiple global sources.
- When config extraction fails, printing **(vault id, profile/context, key path, source)** is a huge UX win.

What Lithos should be careful about:

- If you adopt “profiles inside TOML,” you must implement the “nested + select + profile everywhere” discipline described in A.3.3.

#### A.2.3 cargo-lambda: a clean template for context/profile support

cargo-lambda uses Figment to merge a small constellation of sources:

- environment variables (prefixed)
- a config file
- defaults/metadata-derived settings
- optional overrides

It also supports an explicit **context** concept, and implements it consistently:

- select the context/profile for the Figment instance
- ensure the file provider is treated as nested when contexts are present
- apply the same profile selection to env vars and serialized overlays

What Lithos should steal:

- If Lithos ever introduces “contexts” (e.g., per-user or per-environment), cargo-lambda provides the blueprint for correctness:
  - `select(context)`
  - `Toml::file(...).nested()`
  - apply `.profile(context)` to every provider that needs it (env providers, serialized overlays)

This avoids a common failure mode:

- You “select a profile,” but only some sources honor it, so values silently come from a different profile than you think.

#### A.2.4 Taplo LSP: incremental “defaults + update blob” without custom merge code

Taplo’s LSP config update demonstrates a simple, robust pattern:

- Start from the current config as defaults.
- Merge in an incoming JSON blob.
- Extract into the typed config.

What Lithos should steal:

- Use Figment-style composition for incremental updates (CLI overrides, “temporary session overrides,” etc.) rather than writing custom merge logic.
- This pattern is especially attractive for the Command side: “apply override config, validate, persist new merged version.”

#### A.2.5 Arti (`tor-config`): separate unvalidated input tree from validated runtime config

Arti’s config system is not a Figment tutorial, but it is a great *domain-level* model for configuration correctness.

Core flow:

1) Load multiple sources into an **unvalidated, dynamically-typed configuration tree**.
2) Resolve into **typed builders** via deserialization.
3) Build/validate once for all consumers so unrecognized keys can be reported in one pass.

What Lithos should steal:

- Keep a clear separation between:
  - unvalidated input configuration (best-effort parse tree)
  - typed domain structures (Global/Vault)
  - validated runtime aggregate (`Config`)
- When reporting errors, do it from a unified resolution step so “unknown keys” and “invalid values” are coherent.

### A.3 Recommendations for Lithos (Grounded in the Case Studies)

This section converts the case studies into concrete, Lithos-specific guidance.

#### A.3.1 Make precedence a first-class, versioned contract

Observed pattern across Rocket + apple-codesign + cargo-lambda:

- Users only trust layered config if the precedence is deterministic and documented.

Recommended Lithos precedence order (suggested default):

1) system defaults
2) global config file(s)
3) vault config file(s)
4) environment variables (optional, if Lithos chooses to support them)
5) command/CLI overrides (explicit user intent)

Keep this order stable over time. If it changes, treat it as a breaking change.

#### A.3.2 Use profiles/contexts only for “operational context,” not vault identity

Profiles work well when:

- you want to run the *same application* under different environments (`dev`, `test`, `prod`)

Profiles do not work well when:

- you want N independent configs for N vaults (vaults are not “environments”; they are domain instances)

Guidance for Lithos:

- Keep **Global vs Vault** as distinct layers.
- If you add “context,” add it orthogonally:
  - select one context at a time
  - apply it consistently to all providers

#### A.3.3 If contexts exist, adopt a strict rule: `nested + select + profile` everywhere

If Lithos introduces a “context” concept, follow this discipline:

- if a file is expected to contain contexts/profiles, treat it as nested
- select the context in the Figment instance early
- apply the same profile/context to env vars and serialized overlays

This is a correctness rule, not a style preference.

#### A.3.4 Provide an “explain config” query (UX win)

Observed in apple-codesign and cargo-lambda:

- Being able to show “what config is active” and “where it came from” is invaluable.

Recommended query capability (read-model / debug view):

- `explain_merged_config(vault_id, context?) -> { active_version, layers_present, merged_config }`

Optionally include a “provenance mode”:

- For each major section (`frontmatter`, `schema`, `template`, `logging`), record whether it came from `vault`, `global`, or `default`.

This dovetails with the versioned merged-config read model and rollback story.

#### A.3.5 Be explicit about list semantics (`merge` vs additive)

Observed in cargo-lambda:

- merging lists is contentious; some users want replacement, others want additive behavior.

Guidance for Lithos:

- Default to **replace** for arrays/lists in config overrides.
- Only adopt additive merging for carefully chosen fields where it is obviously correct (and document it field-by-field).

If Lithos needs both semantics, represent it in the model:

- e.g., `TrustedVaultsPolicy::Replace(Vec<_>)` vs `TrustedVaultsPolicy::Extend(Vec<_>)`

#### A.3.6 Separate “input config shapes” from “persisted config bytes”

Tension present in Lithos today:

- domain types derive both `serde` and `rkyv`, which can make the domain model awkward to evolve.

Guidance (aligned with the patterns above):

- Treat config as a pipeline:
  1) parse/load (serde-focused, best-effort)
  2) build domain objects (type-driven invariants)
  3) build merged aggregate (validated)
  4) persist read-model bytes (rkyv-focused)

If a domain model change would force a persisted-format migration, prefer introducing explicit DTO boundaries rather than contorting core domain types.

### A.4 Practical Design Additions (Compatible with the Rest of This Spec)

These additions are recommended because they are high leverage and don’t require changing the fundamental “Global + Vault → merged aggregate” model.

#### A.4.1 Add a domain-friendly “config resolution report” type

Even without per-key provenance, a simple report object helps debugging and observability:

- which sources were loaded successfully
- which sources were absent
- which vault identity was used
- which merged version was activated (if using versioned merged configs)

This can be emitted at the application boundary and/or persisted as part of the merged-config version metadata.

#### A.4.2 Normalize unknown-key reporting behavior

If Lithos uses Figment extraction for file/env inputs, decide and document:

- whether unknown keys are hard errors or warnings
- where the error is raised (Global/Vault build vs final merged `Config::build`)

Arti’s guidance is strong here:

- resolve once for all consumers so unrecognized keys are reported coherently

#### A.4.3 Keep “VaultId is primary identity” consistent in user-facing messages

Given the decision in 3.4.2, all user-facing config messages should prefer:

- `VaultId` as the stable identifier
- `VaultRoot` as a current location hint

This matches the model’s intent (stable identity) and avoids confusing path-only errors after vault moves.
