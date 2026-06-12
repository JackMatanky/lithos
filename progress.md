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

## Verification

- No code changes have been made beyond planning files.
