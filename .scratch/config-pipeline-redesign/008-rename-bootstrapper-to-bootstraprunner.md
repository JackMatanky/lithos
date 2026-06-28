---
labels: [ready-for-agent]
---

# Rename Bootstrapper To BootstrapRunner

## Parent

.scratch/config-pipeline-redesign/PRD.md

## What to build

Migrate the app composition root from the old generic discovery-port/repository bootstrapper to `BootstrapRunner` backed by SettingsService. The runner maps CLI flags into settings DTOs, runs discovery/config build through SettingsService, and creates the cache directory from the final AppConfig.

Do this migration before any old Bootstrapper-facing settings components are removed.

## Acceptance criteria

- [ ] App composition root exposes `BootstrapRunner` instead of the generic `Bootstrapper<D: DiscoveryPort>` for new code.
- [ ] BootstrapRunner maps CLI discovery flags into `DiscoveryOptions`.
- [ ] BootstrapRunner maps build flags into `ConfigBuilderOptions`.
- [ ] BootstrapRunner calls `SettingsService::discover` for discovery-only flows.
- [ ] BootstrapRunner calls `SettingsService::build_config` for full config flows.
- [ ] BootstrapRunner calls `SettingsService::setup_cache_dir` after AppConfig construction.
- [ ] No repository argument is required to run bootstrap.
- [ ] Existing app-level bootstrap tests are migrated to the new runner.

## Blocked by

- .scratch/config-pipeline-redesign/004-build-ephemeral-config-pipeline.md
