---
labels: [ready-for-agent]
---

# 00: Pre-Work — File Structure, Renames, and Dead Code Deletion

## Problem Statement

The `crates/settings/` crate still uses the old file structure, naming conventions, and dead persistence modules. Attempting to implement issues 01–11 directly against the current layout will cause parallel-type confusion (two `Config` structs, two `Global` types, two `Builder` types), misdirected imports, and unnecessary merge conflicts. A pre-work pass clears the board so each subsequent issue operates on the correct structure.

## Goals

1. Create new file structure for the redesigned crate (stub types with TODOs, matching the PRD module tree)
2. Delete dead persistence code (`storage/`, `repository.rs`, `views.rs`, `merger.rs`, `events.rs`, `processor.rs`, `diagnostics.rs`)
3. Rename existing types in-place where old→new mapping is unambiguous
4. Update `lib.rs` and `config/mod.rs` to match the new module tree
5. Keep the crate **compilable** after every step (additive before destructive)
6. Avoid creating any new functionality — this is purely structural

## Operations

### A. Delete Dead Modules (safe to remove, test-only usage)

| Module               | Files                             | Risk                                                                 |
| -------------------- | --------------------------------- | -------------------------------------------------------------------- |
| `config/storage/`      | `mod.rs`, `read.rs`, `write.rs`, `tables.rs`, `testing.rs` | LOW — references only in `#[cfg(test)]` blocks in `builder.rs` and `bootstrap.rs` |
| `config/repository.rs` | `repository.rs`                   | LOW — `ReadRepository`/`WriteRepository`/`Repository` traits used only internally |
| `config/views.rs`      | `views.rs`                        | LOW — dead code, no imports outside settings crate                    |
| `config/merger.rs`     | `merger.rs`                       | LOW — dead code                                                       |
| `config/events.rs`     | `events.rs`                       | LOW — dead code                                                       |
| `config/processor.rs`  | `processor.rs`                    | LOW — dead code                                                       |
| `config/diagnostics.rs`| `diagnostics.rs`                  | LOW — already unused, `pub(crate)`                                    |
| `config/aggregate.rs`  | `aggregate.rs`                    | MEDIUM — `Config` aggregate is referenced in tests; move relevant logic to `config/app.rs` first |

**Mitigation**: Create new files (see §B) *before* deleting old ones. Test-code references to `InMemoryRepository`, `RedbRepository`, `ReadRepository`, `WriteRepository` in `builder.rs` (650, 702, 808) and `bootstrap.rs` (626, 1138) must be removed or rewritten to use the new types. Do this in the same deletion commit so CI never sees a broken state.

### B. Create New Files (empty stubs with type placeholders)

| File                 | Purpose                                                   |
| -------------------- | --------------------------------------------------------- |
| `src/candidate.rs`     | `CandidatePath { base: DirPath, path: FilePath }`           |
| `src/service.rs`       | `SettingsService` trait + `Service` impl (methods call into processor + builder) |
| `src/location.rs`      | Flat `&[&str]` path constants (marker filenames, tracking/trust subdirs, cache subdir) |
| `src/env.rs`           | `SettingsEnvVars` — env var names and reading               |
| `src/os_dirs.rs`       | Platform XDG/home/AppData directory resolution              |
| `src/config/tracker.rs` | `Tracker` module, `pub(crate)` — `track()`, `list_all()`, `clean()` |
| `src/config/trust.rs`   | `Trust` module, `pub` — `trust()`, `untrust()`, `is_trusted()`, `trust_check()`, `ignore()` |
| `src/config/options.rs` | `ConfigBuilderOptions { trust_mode, auto_confirm }`         |
| `src/config/local.rs`   | `LocalConfig` — newtype with `TryFrom<RawConfig>` (forbids `trusted_vaults`) |
| `src/config/app.rs`     | `AppConfig` — final merged config, exposes `create_cache_dir()` and to-spec methods |

**Existing files to modify** (not create):
- `src/config/raw.rs` — rewrite to single `RawConfig` DTO (all `Option` fields, replaces `RawGlobalConfig` + `RawVaultConfig`)
- `src/config/global.rs` — rename `Global` → `GlobalConfig`, remove `GlobalVersion`, add forbidden-field check for `cache`

### C. Rename In-Place (old → new)

| Old Name (current)               | New Name (target)          | File(s)                                        |
| -------------------------------- | -------------------------- | ---------------------------------------------- |
| `Global`                         | `GlobalConfig`             | `src/config/global.rs`                           |
| `Vault`                          | `LocalConfig`              | `src/config/vault.rs` → `src/config/local.rs`    |
| `Config` (aggregate)             | `AppConfig`                | `src/config/aggregate.rs` → `src/config/app.rs`  |
| `RawGlobalConfig`                | `RawConfig` (unified)      | `src/config/raw.rs`                              |
| `RawVaultConfig`                 | removed (folded into `RawConfig`) | `src/config/raw.rs`                       |
| `Metadata`                       | folded into `LocalConfig` fields | removed                                    |
| `VaultId`                        | removed                    | removed                                          |
| `VaultVersion`/`GlobalVersion`/`Version` | removed           | removed                                          |
| `VaultRoot`                      | `DirPath` (existing)       | removed                                          |
| `DiscoveryEnv`                   | `SettingsEnvVars`              | `src/discovery/env.rs` → `src/env.rs`            |
| `DiscoveryPort`                  | removed (internal)         | `src/discovery/port.rs` — delete module           |
| `DiscoveryService`               | removed (internal)         | `src/discovery/service.rs` — delete module        |

### D. Re-Export Adjustments in `lib.rs` and `config/mod.rs`

After deletion + creation:
- `lib.rs` must no longer re-export `repository`, `storage`, `merger`, `events`, `processor`, `views`, `diagnostics`
- `lib.rs` must re-export new public types: `SettingsService`, `CandidatePath`, `DiscoveryOptions`, `ConfigBuilderOptions`, `AppConfig`, `GlobalConfig`, `LocalConfig`, `Trust`, `DiscoveryOutcome`, `ConfigError`
- `config/mod.rs` must remove `storage`, `repository`, `views`, `merger`, `events`, `processor`, `diagnostics` module declarations
- `config/mod.rs` must add `tracker`, `trust`, `options`, `local`, `app` module declarations

### E. External Caller Updates

Code outside `crates/settings/` that references renamed types:

| Caller                                      | Old Reference                | New Reference                              |
| ------------------------------------------- | ---------------------------- | ------------------------------------------ |
| `crates/app/src/bootstrap.rs` (tests)       | `InMemoryRepository`, `Builder` | Remove test (covered by new pipeline)     |
| `crates/cli/src/commands/config.rs`         | `Bootstrapper`                | `BootstrapRunner`                          |
| `crates/cli/src/commands/config_files.rs`   | `Bootstrapper`                | `BootstrapRunner`                          |
| `crates/cli/src/commands/doctor.rs`         | `Bootstrapper`                | `BootstrapRunner`                          |
| `crates/cli/src/commands/index.rs`          | `Bootstrapper`                | `BootstrapRunner`                          |
| `crates/cli/src/main.rs`                    | `Bootstrapper`                | `BootstrapRunner`                          |

**Warning**: The external caller updates in CLI can only rename the import + type usage, not change the API. The API change (removing generic `D: DiscoveryPort`) belongs to issue 08. This pre-work is purely mechanical renames.

## Execution Order

Run these steps sequentially, verifying `cargo build` (not just `cargo check`) after each:

1. **Create new files** (§B) — all additive, cannot break build
2. **Rename in-place** (§C) — update struct names + file paths
3. **Delete dead modules** (§A) — update `lib.rs`/`config/mod.rs` re-exports AND remove test code referencing deleted types in same commit
4. **Update external callers** (§E) — mechanical `Bootstrapper`→`BootstrapRunner` rename in CLI files
5. **Run `cargo build --workspace`** — verify full workspace compiles

## Risk Mitigation

| Risk                              | Mitigation                                                                       |
| --------------------------------- | -------------------------------------------------------------------------------- |
| Parallel-type confusion           | Do not create new types alongside old types (additive then remove in same atomic step via §B→§A order) |
| Broken CLI compilation            | CLI references `Bootstrapper` in 5 files — update all in one commit              |
| Broken architecture tests         | `tests/architecture.rs` may reference deleted modules — update exclusion patterns in this step |
| CI fails mid-step                 | Sequence ensures build compiles after each of the 4 steps; revert any single failing step |
| Figment dependency not yet added  | This pre-work does NOT add figment. New files use manual serde until issue 04    |
| Lost git history                  | Use `git mv` for renames (preserves history), `git rm` for deletions             |

## Definition of Done

- [ ] New file structure matches PRD module tree
- [ ] `storage/`, `repository.rs`, `views.rs`, `merger.rs`, `events.rs`, `processor.rs`, `diagnostics.rs` deleted
- [ ] `Global` → `GlobalConfig`, `Vault` → `LocalConfig`, `Config` → `AppConfig`, `DiscoveryEnv` → `SettingsEnvVars` renamed
- [ ] `lib.rs` and `config/mod.rs` re-export only new public types
- [ ] All CLI files use `BootstrapRunner` name (still generic until issue 08)
- [ ] `cargo build --workspace` succeeds
- [ ] `cargo test --workspace` succeeds (tests referencing deleted types removed or rewritten)
- [ ] No parallel old/new types exist for the same concept
