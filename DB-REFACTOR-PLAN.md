# DB Refactor Plan: Per-Table Redb Tables (No Tuple Keys)

## Goal
Remove key-prefix string allocations and move to per-table Redb tables, keeping
single-key lookups zero-allocation and table definitions close to their context
modules.

## Decisions
- Do not use tuple keys (no perf win for single-key, adds encoding overhead).
- Remove key-prefix types: `NamespacedKey`, `MultimapKey`, `TablePrefix`.
- Use `redb::TableDefinition` and `redb::MultimapTableDefinition` constants
  defined in each context `mod.rs`.

## Target Design
- Each context defines its own table constants in `mod.rs`.
- `Database` APIs accept table definitions instead of `&str` table names.
- Multimap operations accept table definitions and raw `&str` keys/values.
- Listing uses full table iteration (no prefix scanning).

## Scope (Contexts)
- config: config, merged_config_versions, merged_config_active,
  vault_id_by_path, vault_path_by_id
- note: notes, path_to_id, tags_to_notes, alias_to_id, file_class_to_id,
  folder_to_id, tasks_by_*, frontmatter_kv
- schema: schemas
- template: templates, template_name_to_id

## Implementation Steps
1. Add table definitions in context modules
   - Example in `lithos-core/src/note/mod.rs`:
     - `pub(crate) const NOTES_TABLE: TableDefinition<&str, &[u8]> = ...`
     - `pub(crate) const PATH_TO_ID: MultimapTableDefinition<&str, &str> = ...`
   - Repeat for config, schema, template.

2. Update DB API signatures
   - `put/get/get_owned/delete/list_owned` to accept `&TableDefinition<...>`.
   - `multimap_insert/multimap_remove/multimap_get` to accept
     `&MultimapTableDefinition<...>`.
   - Update `WriteBatch` methods to match.

3. Remove key-prefix helpers
   - Delete or retire `NamespacedKey`, `MultimapKey`, `TablePrefix` in
     `lithos-core/src/db/keys.rs`.
   - Remove `DATA_TABLE` usage from `lithos-core/src/db/mod.rs`.

4. Rework DB internals
   - Replace `open_table(DATA_TABLE)` with `open_table(*table_def)`.
   - Replace prefix scanning in `list_owned` with full table iteration.

5. Update call sites
   - Replace string table names with context table constants.
   - Example: `db.put("notes", key, value)` -> `db.put(&NOTES_TABLE, key, value)`.
   - Update tests and fixtures accordingly.

6. Verify behavior
   - Run existing tests for db, note, schema, template, config.
   - Ensure no `format!("{table}:{key}")`-style construction remains.

## Execution Order (Commit-Friendly)
- [ ] Phase 1: DB layer refactor with parallel APIs (no removals)
  - [ ] Add new DB APIs that accept `TableDefinition` / `MultimapTableDefinition`.
  - [ ] Keep existing string-based APIs untouched.
  - [ ] Add tests for new APIs in `lithos-core/src/db/reader.rs` and
        `lithos-core/src/db/writer.rs`.
  - [ ] Ensure existing db tests still pass.

- [ ] Phase 2: Migrate schema + template
  - [ ] Define table constants in `lithos-core/src/schema/mod.rs`.
  - [ ] Define table constants in `lithos-core/src/template/mod.rs`.
  - [ ] Update schema command/query to use table constants.
  - [ ] Update template command/query to use table constants.
  - [ ] Update schema/template tests.

- [ ] Phase 3: Migrate config
  - [ ] Define table constants in `lithos-core/src/config/mod.rs`.
  - [ ] Update `lithos-core/src/db/config_adapter.rs` to use constants.
  - [ ] Update config tests if needed.

- [ ] Phase 4: Migrate note (largest surface area)
  - [ ] Define table constants in `lithos-core/src/note/mod.rs`.
  - [ ] Update note command/query to use constants.
  - [ ] Update note tests and fixtures.

- [ ] Phase 5: Remove legacy APIs and key types
  - [ ] Remove `NamespacedKey`, `MultimapKey`, `TablePrefix`.
  - [ ] Remove `DATA_TABLE` and prefix-scan logic.
  - [ ] Remove string-based db methods and update callers.
  - [ ] Re-run full test suite.

## Risk Notes
- Changes are internal and should not affect external APIs beyond the db layer.
- No migration needed if database contents are not persisted yet.

## Follow-ups (Optional)
- Consider a typed table registry only if cross-context usage grows.
- For composite indexes, keep separate multimap tables (no tuple keys).
