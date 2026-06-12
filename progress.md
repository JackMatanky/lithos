# Paths Refactor Assessment Progress

## Session Log

- Initialized planning files in repo root.
- Reviewed provided source snapshots for `paths.rs`, `raw.rs`, and `processor.rs`.
- Ran GitNexus query for config path flows and context/impact for `Paths` and `SchemaConfigSpec`.
- GitNexus struct-node impact for `Paths` and `SchemaConfigSpec` reported LOW risk, but additional exact UID and text reference checks are still needed.
- Ran exact UID impact for path value objects and text searched config path imports.
- Read `config/aggregate.rs`, `config/global.rs`, `config/vault.rs`, `config/mod.rs`, and `schema/discovery.rs` for concrete seams.
- Read `config/builder.rs` and `config/merger.rs`; confirmed the raw merge boundary and that processor/merger are insulated from resolved path type shape unless raw DTOs change.
- User preference recorded: move away from `Paths` structs where possible, with `Paths` only as a possible temporary bridge.
- User selected split `Config` fields as the preferred resolved storage shape.
- User confirmed no compatibility is required for old rkyv-serialized `Config` records.
- User selected an intentional public API break now instead of staging `Paths` compatibility shims.
- User clarified desired file split: separate `cache.rs`, `template.rs`, and `schema.rs` files.
- User selected direct `config/` file locations for the split modules.
- User selected moving `SchemaConfigSpec` into `config::schema`.
- Confirmed existing `TemplateConfigSpec` in current workspace and recorded move to `config::template`.
- User requested creating `CacheConfigSpec`.
- User selected `CacheConfigSpec` shape matching `TemplateConfigSpec`.
- User selected removing `config::global::Paths` and `config::vault::Paths` in the same refactor.
- User selected placing `PropertyBankFile` inside `SchemaConfig`.
- Main assessment decisions are complete.
- User clarified that leaving `RawPathsConfig`, `RawVaultPaths`, `RawGlobalPaths`, and `schema/config.schema.json` unchanged is only to keep this refactor small; the next refactor will update those raw/schema inputs.

## Verification

- No code changes have been made beyond planning files.
- 2026-06-12: Created dedicated worktree `/Users/jack/Documents/41_personal/lithos/.worktrees/feat/config-path-modules` on branch `feat/config-path-modules`; baseline `cargo test -p lithos-core --lib` passed with 1697 tests.
- 2026-06-12: Completed Tasks 1-4 and committed changes through `facab80f`; commit hooks passed after fixing stale `config/mod.rs` doctest.
- 2026-06-12: Task 5 completed and committed as `db7ac831`; `cargo test -p lithos-core config::aggregate::tests::config_specs --lib` passed 3/3, `cargo test -p lithos-core --lib` passed 1739/1739, GitNexus change detection reported low risk and no affected processes.
