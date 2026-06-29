---
labels: [ready-for-agent]
---

# 00: Pre-Work — File Structure Stubs and Renames

## Problem Statement

The `crates/settings/` crate still uses the old naming conventions (`Vault`, `Global`, `Config`, `Bootstrapper`). Attempting to implement issues 01–11 directly against the current layout will cause parallel-type confusion, misdirected imports, and unnecessary merge conflicts. A pre-work pass clears the board so each subsequent issue operates on the correct type names and file stubs.

## Goals

1. Create new file structure for the redesigned crate (stub types with TODOs, matching the PRD module tree) for additive components like tracking and trust.
2. Rename existing types in-place where old→new mapping is unambiguous (`Config` -> `AppConfig`, `Vault` -> `LocalConfig`, `Global` -> `GlobalConfig`).
3. Rename `Bootstrapper` to `BootstrapRunner` across all callers.
4. Keep the crate **compilable** after every step.
5. **CRITICAL**: Do NOT delete any existing logic, accessors, or pipeline components (`merger.rs`, `processor.rs`, `views.rs`, `storage/`, `repository.rs`, etc.). Deletions are strictly deferred to Issue 07 to ensure the existing config pipeline remains unbroken while the new one is built alongside it.

## Operations

### A. Create New Files (empty stubs with type placeholders)

These components are additive and can be introduced safely:

| File                 | Purpose                                                   |
| -------------------- | --------------------------------------------------------- |
| `src/candidate.rs`     | `CandidatePath { base: DirPath, path: FilePath }`           |
| `src/service.rs`       | `SettingsService` trait + `Service` impl (minimal stubs)    |
| `src/location.rs`      | Flat `&[&str]` path constants (marker filenames, etc.)      |
| `src/env.rs`           | `SettingsEnvVars` — env var names and reading               |
| `src/os_dirs.rs`       | Platform XDG/home/AppData directory resolution              |
| `src/config/tracker.rs` | `Tracker` module, `pub(crate)` — `track()`, `list_all()`, `clean()` stubs |
| `src/config/trust.rs`   | `Trust` module, `pub` — `trust()`, `untrust()`, `trust_check()` stubs |
| `src/config/options.rs` | `ConfigBuilderOptions { trust_mode, auto_confirm }`         |

### B. Rename In-Place (old → new)

Mechanically rename these structs and their usages. **Do not delete their existing methods, traits, or attributes.**

| Old Name (current)               | New Name (target)          | File(s)                                        |
| -------------------------------- | -------------------------- | ---------------------------------------------- |
| `Global`                         | `GlobalConfig`             | `src/config/global.rs`                           |
| `Vault`                          | `LocalConfig`              | `src/config/vault.rs`                            |
| `Config` (aggregate)             | `AppConfig`                | `src/config/aggregate.rs`                        |
| `RawGlobalConfig`                | `RawConfig` (unified)      | `src/config/raw.rs`                              |
| `RawVaultConfig`                 | remove/merge into `RawConfig` | `src/config/raw.rs`                           |
| `DiscoveryEnv`                   | `SettingsEnvVars`          | `src/discovery/env.rs`                           |

*Note: We keep the files `aggregate.rs` and `vault.rs`. We only rename the structs inside them.*

### C. External Caller Updates

Code outside `crates/settings/` that references renamed types:

| Caller                                      | Old Reference                | New Reference                              |
| ------------------------------------------- | ---------------------------- | ------------------------------------------ |
| `crates/cli/src/commands/config.rs`         | `Bootstrapper`               | `BootstrapRunner`                          |
| `crates/cli/src/commands/config_files.rs`   | `Bootstrapper`               | `BootstrapRunner`                          |
| `crates/cli/src/commands/doctor.rs`         | `Bootstrapper`               | `BootstrapRunner`                          |
| `crates/cli/src/commands/index.rs`          | `Bootstrapper`               | `BootstrapRunner`                          |
| `crates/cli/src/main.rs`                    | `Bootstrapper`               | `BootstrapRunner`                          |
| `crates/app/src/bootstrap.rs`               | `Bootstrapper`               | `BootstrapRunner`                          |

**Warning**: The external caller updates in CLI can only rename the import + type usage, not change the API. The API change (removing generic `D: DiscoveryPort`) belongs to issue 08. This pre-work is purely mechanical renames.

### D. Re-Export Adjustments in `lib.rs` and `config/mod.rs`

- `lib.rs` and `config/mod.rs` must re-export the renamed types (`AppConfig`, `LocalConfig`, `GlobalConfig`).
- Add the new additive modules (`tracker`, `trust`, `options`) to `config/mod.rs` declarations.
- DO NOT remove existing re-exports for `repository`, `storage`, `merger`, etc. They are still needed until Issue 07.

## Execution Order

Run these steps sequentially, verifying `cargo build` (not just `cargo check`) after each:

1. **Create new files** (§A) — all additive, cannot break build.
2. **Rename in-place** (§B) — update struct names in `global.rs`, `vault.rs`, `aggregate.rs`, `raw.rs`.
3. **Update external callers** (§C) — mechanical `Bootstrapper`→`BootstrapRunner` rename.
4. **Update Re-Exports** (§D) — expose the new names and additive stubs.
5. **Run `cargo build --workspace`** — verify full workspace compiles.

## Risk Mitigation

| Risk                              | Mitigation                                                                       |
| --------------------------------- | -------------------------------------------------------------------------------- |
| Parallel-type confusion           | Renaming `Config` to `AppConfig` immediately ensures no one accidentally uses the old name during the rewrite. |
| Broken existing pipeline          | Explicitly forbidding deletion of `merger.rs`, `processor.rs`, etc. ensures `Builder` still compiles. |
| Broken CLI compilation            | CLI references `Bootstrapper` in 5 files — update all in one commit.             |
| Figment dependency not yet added  | This pre-work does NOT add figment. New files use manual serde until issue 04.   |

## Definition of Done

- [ ] New additive file stubs (`trust.rs`, `tracker.rs`, etc.) are created.
- [ ] `Global` → `GlobalConfig`, `Vault` → `LocalConfig`, `Config` → `AppConfig`, `DiscoveryEnv` → `SettingsEnvVars` struct renames are complete.
- [ ] `Bootstrapper` renamed to `BootstrapRunner` everywhere.
- [ ] No pipeline functionality, `Config` methods, or database modules were deleted.
- [ ] `cargo build --workspace` and `cargo test --workspace` succeed.
