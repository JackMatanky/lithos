---
labels: [ready-for-agent]
---

# Define SettingsService Boundary

## Parent

.scratch/config-pipeline-redesign/PRD.md

## What to build

Define the new public Settings boundary without changing the full implementation yet. Add the public DTOs and service API shape that downstream migration slices will target: discovery options, builder options, discovery outcome, and the SettingsService entry point. Existing code may delegate through adapters during this slice, but new callers should have a clear API to migrate toward.

Do not remove old discovery/config components in this slice. They stay until later slices migrate consumers away from them.

## Acceptance criteria

- [ ] `SettingsService` is the documented public inbound boundary for settings.
- [ ] Public DTOs exist for `DiscoveryOptions`, `ConfigBuilderOptions`, and `DiscoveryOutcome`.
- [ ] `DiscoveryOptions` contains CLI/runtime discovery inputs, not environment variables.
- [ ] `ConfigBuilderOptions` contains config-build inputs such as trust mode and auto-confirm.
- [ ] `SettingsService` exposes `discover`, `build_config`, and `setup_cache_dir` with the PRD semantics, even if implementation delegates internally for now.
- [ ] Existing public callers continue to compile.
- [ ] Tests or compile-time checks cover the new API shape.

## Blocked by

None - can start immediately
