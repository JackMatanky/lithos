# Config Context Refactor Plan Audit (Start)

**Context:** `lithos-core/src/config/`
**Template:** `REFACTOR_PLAN_CHECKLIST_TEMPLATE.md`
**Primary Authority:** `_bmad-output/planning-artifacts/architecture/04-implementation-patterns-consistency-rules.md`

---

## 0) Inputs and Constraints

- [x] Read `_bmad-output/planning-artifacts/architecture/04-implementation-patterns-consistency-rules.md` (primary authority).
- [x] Read `_bmad-output/project-context.md` and confirm latest rules.
- [ ] Read ADR 002 (Repository) only for historical context.
- [ ] Read `docs/refs/rust/naming-taxonomy.md` and confirm method naming rules.
- [x] Confirm context isolation: config is cross-cutting and must not import business contexts.
- [x] Confirm file-based source-of-truth requirement for this refactor.
- [x] Reviewed archived audit for context: `docs/_archive/config-module-review.md` (pre-architecture change).

---

## 1) Full File and Component Audit (Context Inventory)

### 1.1 File Inventory

Directory: `lithos-core/src/config/`

- `adapter/` - I/O adapters (ingest + redb)
- `aggregate.rs` - Config aggregate root + Version
- `command.rs` - CQRS command wrapper
- `error.rs` - Config error types
- `events.rs` - Config events
- `frontmatter.rs` - Frontmatter config
- `global.rs` - Global config + GlobalVersion
- `logging.rs` - Logging config
- `mod.rs` - Module wiring + DB tables + aliases
- `paths.rs` - Path types + validation
- `ports.rs` - CQRS ports
- `query.rs` - CQRS query wrapper
- `raw.rs` - Raw config DTOs
- `task.rs` - Task config
- `value.rs` - Field spec/value validation
- `vault.rs` - Vault config + VaultId/VaultRoot + VaultVersion

Submodule: `lithos-core/src/config/adapter/`

- `ingest.rs` - Figment-based loading + file metadata
- `command.rs` - Redb command adapter
- `query.rs` - Redb query adapter
- `stored.rs` - ConfigMetadata (staleness metadata)

### 1.2 Component Inventory (Initial)

- Domain types: `Config`, `Global`, `Vault`, `Paths`, `Logging`, `Frontmatter`, `Task`
- Raw types: `RawConfig`, `RawPathsConfig`, `RawTaskConfig`, etc.
- Raw types currently represent unified schema (no explicit global/vault split).
- Stored/Projection types: `ConfigMetadata` (adapter/stored)
- Ports/Adapters: `ports::Command`, `ports::Query`, adapter command/query
- Loader/orchestration: ConfigService in application layer (not in config module)
- Errors: `ConfigError`, `ConfigCommandError`, `ConfigQueryError`, `ConfigIngestError`
- Events: `Events`, `ConfigUpdated`

### 1.3 Cross-File Coupling Audit (Initial)

- CQRS split present: `ports.rs` + `command.rs` + `query.rs` + adapter command/query
- Ingestor uses Figment + FsReader (I/O boundary in adapter)
- Config aggregate contains pending events (domain + event coupling)
- `mod.rs` defines db tables (cross-cutting infrastructure location)

---

## 2) Workflow and Pipeline Audit (Behavioral Inventory)

### 2.1 Pipeline Map (Current)

From archived audit + current module layout:

1) **File discovery + parsing (adapter/ingest)**
   - Global config path resolution (system + XDG + env)
   - Vault config at `.lithos/lithos.toml`
   - Figment merge: defaults → global → vault
   - Raw config produced with metadata

**Observation:** config schema defines global-only vs vault-only fields, but raw layer has a single `RawConfig` type.

2) **Staleness checks**
   - `Query::is_global_stale(created_at, modified_at)`
   - `Query::is_vault_stale(vault_id, created_at, modified_at)`

3) **Domain construction**
   - `Config::build(raw, vault_id, vault_root, version)`
   - Version allocation via command

4) **Persistence**
   - `Command::record_global` + `record_vault` + `record_config`
   - Metadata stored in `config_metadata`

### 2.2 Bloat and Inefficiency Checks (Initial)

- CQRS split likely unnecessary for file-based config (target is unified Repository).
- `ConfigMetadata` duplicates staleness info (potential replace with raw views).
- Orchestration lives in application layer; config module lacks loader.
- Hybrid staleness logic exists but needs alignment with new architecture.

### 2.3 Modularity and Isolation Checks (Initial)

- Figment confined to adapter/ingest (good boundary).
- Raw types are serde DTOs (good).
- Config aggregate includes event queue (may be unnecessary in new architecture).

---

## 3) Architecture Alignment Audit (Initial)

- [ ] Raw types serde-only? **Yes** (raw.rs DTOs; no behavior).
- [ ] Raw parsing helpers to avoid persisting invalid raw views? **Partial** (Figment parsing in adapter; no explicit Raw parsing helpers).
- [ ] RawGlobalConfig / RawVaultConfig split? **No (currently single RawConfig).**
- [ ] Domain types validated and used as storage shape? **Yes** (Config/Global/Vault are persisted).
- [ ] Unified Repository trait? **No** (CQRS ports present).
- [ ] File ingestion uses FsReader? **Yes** (adapter/ingest uses FsReader).
- [x] Context isolation? **Checked: no note/schema/template imports.**
- [x] Naming taxonomy? **Initial scan: no `are_many_*`; CQRS split still conflicts.**

---

## 4) Refactor Targets and Removal Candidates (Initial)

- [ ] Replace CQRS ports + wrappers with unified `Repository` in `storage.rs`.
- [ ] Add `loader.rs` inside config module to own orchestration (hybrid loading).
- [ ] Decide whether to remove `ConfigMetadata` in favor of raw views.
- [ ] Introduce `RawGlobalConfig` and `RawVaultConfig` for per-file parsing and validation; merge into `RawConfig` for Figment layering.
- [ ] Align naming to taxonomy (`all_*`/`any_*`, no `get_` getters).
- [ ] Reduce domain event coupling if unused.
- [ ] Move `adapter/ingest.rs` to `ingestor.rs` (standardize naming across contexts).
- [ ] Update `schema/config.schema.json` to align with RawGlobal/RawVault split + task updates.

---

## 11) Gap Analysis (Initial)

### Architecture Conflicts

- CQRS ports and wrappers present; must move to unified Repository.
- Orchestration outside context (application layer); should move to `config::loader`.
- `ConfigMetadata` duplicates staleness info; evaluate replacement with raw views.

### Pipeline Gaps

- No raw views for global/vault config files (needed for hybrid staleness checks).
- No in-context loader to coordinate global/vault freshness with Figment merge.
- JSON schema currently models unified config; needs Global/Vault split (oneOf).

### Module Structure Gaps

- `storage.rs` does not exist; Repository trait + concrete repository needed.
- `views/` module does not exist; raw views should live there.

---

## 5) Proposed Module Structure (Target State) (Draft)

```
config/
├── mod.rs              # Public API + re-exports
├── aggregate.rs        # Config aggregate + Version
├── global.rs           # Global config + GlobalVersion
├── vault.rs            # Vault config + VaultVersion + VaultRoot
├── paths.rs            # Path types
├── logging.rs          # Logging config
├── frontmatter.rs      # Frontmatter config
├── task.rs             # Task config
├── value.rs            # Field specs
├── raw.rs              # Raw config DTOs
├── views.rs            # Raw views (global/vault raw versions)
├── storage.rs          # Repository trait + concrete repository
├── loader.rs           # Hybrid config loading pipeline
├── error.rs            # Error types
└── adapter/            # Figment ingestion adapter (kept)
```

---

## 6) Target Pipeline Design (Target State) (Draft)

- Hybrid pipeline (global + vault are separate files):
  1) **Discover global config** (system paths, env override)
  2) **Load vault config** from `.lithos/lithos.toml`
  3) **Parse to RawGlobalConfig / RawVaultConfig** (per-file DTOs)
  4) **Merge to RawConfig** (Figment merge or explicit merge step)
  4) **Staleness check** per file (global and vault)
  5) **Build domain** (`Config::build`) with merged raw
  6) **Persist** global/vault/config snapshot + raw views

- Raw views should track file identity and timestamps for staleness.
- Repository owns storage; loader orchestrates file → raw → domain → storage.

### 6.1 Target Raw View Shapes (Draft)

```rust
// config/views/raw.rs
#[derive(Debug, Clone, PartialEq, Archive, Serialize, Deserialize)]
#[non_exhaustive]
pub struct RawGlobalConfigView {
    file_path: Box<str>,
    versions: RingBuffer<RawFileVersion, 5>,
}

#[derive(Debug, Clone, PartialEq, Archive, Serialize, Deserialize)]
#[non_exhaustive]
pub struct RawVaultConfigView {
    vault_id: VaultId,
    file_path: Box<str>,
    versions: RingBuffer<RawFileVersion, 5>,
}

#[derive(Debug, Clone, PartialEq, Archive, Serialize, Deserialize)]
#[non_exhaustive]
pub struct RawFileVersion {
    compressed_content: Vec<u8>,
    content_hash: [u8; 32],
    created_at: Option<SystemTime>,
    modified_at: Option<SystemTime>,
    recorded_at: SystemTime,
}
```

### 6.2 Target Hybrid Pipeline (Draft)

1) **Discover global config** (system paths + env override)
2) **Discover vault config** (`.lithos/lithos.toml`)
3) **Parse to RawConfig** via Figment merge (defaults → global → vault)
4) **Staleness checks**
   - Global: compare raw view latest version (timestamps + hash)
   - Vault: compare raw view latest version (timestamps + hash)
5) **Rebuild conditions**
   - Rebuild merged config if global or vault is stale
6) **Persist**
   - Save RawGlobalConfigView / RawVaultConfigView
   - Save Global/Vault/Config snapshots via Repository

---

## 7) Migration Plan (Ordered Steps) (Pending)

### 7.1 Sequence (Draft)

1) **Define raw views + storage API**
   - Add `views/raw.rs` with `RawGlobalConfigView` + `RawVaultConfigView`
   - Add `RawFileVersion` for config files (timestamps + hash)

2) **Replace CQRS with Repository**
   - Remove `ports.rs`, `command.rs`, `query.rs`
   - Create `storage.rs` with `Repository` trait + Redb implementation

3) **Move orchestration into config context**
   - Create `loader.rs` to own hybrid pipeline
   - Loader accepts `FsReader` + Repository (no application-level orchestration)
   - Move `adapter/ingest.rs` to `ingestor.rs` at root
   - Standardize on `ingestor.rs` naming across contexts

4) **Integrate Figment and staleness checks**
   - Loader uses adapter/ingest for parsing
   - Use raw views to determine global/vault freshness independently
   - Rebuild merged config only when needed

5) **Update callers and tests**
   - Replace ConfigService usage with config::Loader
   - Update tests to new Repository/Loader API

### 7.2 Risks (Draft)

- **Behavioral regressions**: incorrect staleness checks could skip needed rebuilds.
- **Path resolution differences**: global config discovery order must remain identical.
- **Figment merge semantics**: changing entry points must preserve merge behavior.
- **API breakage**: removing CQRS wrappers breaks existing imports.

---

## 8) Test and Verification Plan (Pending)

### 8.1 Parsing Boundary Tests

- [ ] Raw config parsing via Figment produces expected RawConfig

### 8.2 Loader Pipeline Tests

- [ ] Global-only changes rebuild merged config
- [ ] Vault-only changes rebuild merged config
- [ ] Touch-only changes do not rebuild
- [ ] Missing global config still loads vault config

### 8.3 Repository Tests

- [ ] Unified Repository saves and reads Config/Raw*View
- [ ] `with_archived` access works for hot paths

---

## 9) Output Deliverables (Pending)

- [x] Context audit report (this document)
- [x] Target module tree diagram
- [x] Gap analysis
- [x] Ordered refactor steps + risks
- [x] Test plan

---

## Target Module Tree Diagram

```
config/
├── mod.rs
├── aggregate.rs
├── global.rs
├── vault.rs
├── paths.rs
├── logging.rs
├── frontmatter.rs
├── task.rs
├── value.rs
├── raw.rs
├── views/
│   ├── mod.rs
│   └── raw.rs
├── storage.rs
├── loader.rs
├── ingestor.rs
└── error.rs
```

---

## 10) Context-Specific Notes (Initial)

- Global and vault config are different files with different resolution paths.
- Figment is required for layered merge (defaults → global → vault).
- Config currently uses CQRS wrappers and metadata table for staleness.

---

## Task Config Review (Dataview + Tasks Compatibility)

### Current Task Config (from `config/task.rs` + `config/raw.rs`)

- Tags: `task_tags` with `#task` default
- Status mapping: `status` map (name → symbol)
- Dates: `due`, `created`, `completed`, `reminder`
- Fields: custom `FieldSpec` (enum/date/string/integer/float)
- Indexing: `indexed_fields`
- Dependencies: `enabled` only

### Dataview / Tasks References (Gaps)

**Dataview (Tasks fields)**:
- Implicit fields include `status`, `checked`, `completed`, `fullyCompleted`, `line`, `path`, `section`, `tags`, `outlinks`, `children`, `parent`, `blockId`
- Emoji date shorthands: `due`, `completion`, `created`, `start`, `scheduled`

**Tasks Plugin**:
- Status types: TODO/IN_PROGRESS/ON_HOLD/DONE/CANCELLED/NON_TASK
- Priority signifiers: `🔺`, `⏫`, `🔼`, `🔽`, `⏬️`
- Recurrence: `🔁` rule + `when done`
- On completion: `🏁 keep|delete`
- Dependencies: `🆔 id`, `⛔ depends on` with blocking logic
- Global filter + global query layering for tasks

### Alignment Notes (Decisions Pending)

- **Missing date fields**: `start`, `scheduled` not present in config.
- **Status typing**: no status type enum (Tasks has TODO/IN_PROGRESS/ON_HOLD/DONE/CANCELLED/NON_TASK).
- **Emoji handling**: Tasks uses emoji by default; config has no format toggle.
- **Recurrence + onCompletion**: no config fields present.
- **Dependencies**: config only toggles dependency indexing; no format config.
- **Global query/filter**: not represented in config.

### Proposed Actions

- [x] Add `StatusType` enum for task statuses.
- [x] Add `use_emoji: bool` to select emoji vs dataview formats.
- [x] Add date fields for `start` and `scheduled` with defaults.
- [ ] Decide if priority/recurrence/onCompletion should be configurable or parser-only.
- [ ] Decide if global filter/query belong in config schema.

### Draft Schema/Type Updates (Task Config)

```rust
// config/raw.rs
#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
#[non_exhaustive]
pub struct RawTaskConfig {
    pub enabled: Option<bool>,
    pub task_tags: Option<Vec<String>>,
    pub status: Option<HashMap<String, RawStatusSpec>>,
    pub dates: Option<RawTaskDates>,
    pub fields: Option<HashMap<String, RawFieldSpec>>,
    pub indexing: Option<RawIndexingConfig>,
    pub dependencies: Option<RawTaskDependencies>,
    pub use_emoji: Option<bool>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[non_exhaustive]
pub struct RawStatusSpec {
    pub symbol: char,
    pub status_type: StatusType,
    pub next_symbol: Option<char>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[non_exhaustive]
pub struct RawTaskDates {
    pub created: Option<RawDateFieldSpec>,
    pub due: Option<RawDateFieldSpec>,
    pub completed: Option<RawDateFieldSpec>,
    pub reminder: Option<RawDateFieldSpec>,
    pub start: Option<RawDateFieldSpec>,
    pub scheduled: Option<RawDateFieldSpec>,
}

// config/task.rs
#[derive(Debug, Clone, Copy, PartialEq, Eq, Archive, Serialize, Deserialize)]
#[non_exhaustive]
pub enum StatusType {
    Todo,
    InProgress,
    OnHold,
    Done,
    Cancelled,
    NonTask,
}

#[derive(Debug, Clone, PartialEq, Archive, Serialize, Deserialize)]
#[non_exhaustive]
pub struct CheckboxStatus {
    by_name: HashMap<StatusName, StatusSpec>,
    by_symbol: HashMap<StatusSymbol, StatusSpec>,
}

#[derive(Debug, Clone, PartialEq, Archive, Serialize, Deserialize)]
#[non_exhaustive]
pub struct StatusSpec {
    name: StatusName,
    symbol: StatusSymbol,
    status_type: StatusType,
    next_symbol: StatusSymbol,
}
```

---

## Raw Types Review (Needed)

- Config schema distinguishes **global-only** (`trusted_vaults`) vs **vault-only** (`vault_path`, `name`, `version`, `cache_dir`) fields.
- Raw layer currently uses a single `RawConfig` DTO.
- Proposed: add `RawGlobalConfig` + `RawVaultConfig` for per-file parsing and basic validation.
- Merge path: `RawGlobalConfig` + `RawVaultConfig` → `RawConfig` (for Figment layering and `Config::build`).

### Draft RawGlobal/RawVault Shapes

```rust
// config/raw.rs
#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
#[non_exhaustive]
pub struct RawGlobalConfig {
    pub logging: Option<RawLogging>,
    #[serde(default)]
    pub paths: RawGlobalPaths,
    pub trusted_vaults: Option<RawTrustedVaults>,
    pub frontmatter: Option<RawFrontmatter>,
    pub task: Option<RawTaskConfig>,
}

#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
#[non_exhaustive]
pub struct RawVaultConfig {
    pub vault_path: String,
    pub name: Option<String>,
    pub version: Option<String>,
    pub logging: Option<RawLogging>,
    #[serde(default)]
    pub paths: RawVaultPaths,
    pub frontmatter: Option<RawFrontmatter>,
    pub task: Option<RawTaskConfig>,
}

impl From<(RawGlobalConfig, RawVaultConfig)> for RawConfig {
    fn from((global, vault): (RawGlobalConfig, RawVaultConfig)) -> Self {
        Self {
            logging: vault.logging.or(global.logging),
            paths: RawPathsConfig::merge(global.paths.into(), vault.paths.into()),
            trusted_vaults: global.trusted_vaults,
            frontmatter: vault.frontmatter.or(global.frontmatter),
            task: vault.task.or(global.task),
        }
    }
}

#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
#[non_exhaustive]
pub struct RawGlobalPaths {
    pub templates_dir: Option<String>,
    pub schemas_dir: Option<String>,
    pub property_bank_file: Option<String>,
}

#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
#[non_exhaustive]
pub struct RawVaultPaths {
    pub cache_dir: Option<String>,
    pub templates_dir: Option<String>,
    pub schemas_dir: Option<String>,
    pub property_bank_file: Option<String>,
}

impl From<RawGlobalPaths> for RawPathsConfig {
    fn from(paths: RawGlobalPaths) -> Self {
        Self {
            cache_dir: None,
            templates_dir: paths.templates_dir,
            schemas_dir: paths.schemas_dir,
            property_bank_file: paths.property_bank_file,
        }
    }
}

impl From<RawVaultPaths> for RawPathsConfig {
    fn from(paths: RawVaultPaths) -> Self {
        Self {
            cache_dir: paths.cache_dir,
            templates_dir: paths.templates_dir,
            schemas_dir: paths.schemas_dir,
            property_bank_file: paths.property_bank_file,
        }
    }
}
```
