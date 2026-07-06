---
labels: ["ready-for-agent"]
---

# Migrate Settings to traces-core

## Parent

PRD: `.scratch/core-crate/PRD.md`

## What to build

Migrate the settings context into `traces-core` and update the workspace.

1. Move the contents of the `traces-settings` crate into `traces-core::settings`.
2. Delete the old `traces-settings` crate.
3. Update all downstream contexts in the workspace (e.g., `traces-note`, `traces-schema`, `traces-template`, etc.) to pull their `SettingsService`, `AppConfig`, and other config types from `traces-core` instead of `traces-settings`.
4. Ensure the workspace compiles and all tests pass with the new dependency structure.

## Acceptance criteria

- [ ] `traces-settings` crate is completely removed from the workspace.
- [ ] `SettingsService` and related types live in `traces-core::settings`.
- [ ] All downstream workspace members correctly depend on `traces-core` for settings.
- [ ] Workspace compiles and tests pass.

## Blocked by

- 02-move-refactor-indexer.md
