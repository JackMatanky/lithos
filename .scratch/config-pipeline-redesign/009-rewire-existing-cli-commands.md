---
labels: [ready-for-agent]
---

# Rewire Existing CLI Config Doctor Index Commands

## Parent

.scratch/config-pipeline-redesign/PRD.md

## What to build

Migrate existing CLI commands to the new BootstrapRunner/SettingsService path while preserving command behavior. `config`, `config files`, `doctor`, and `index` should no longer depend on discovery ports, old discovery flags, or in-memory settings repositories.

This slice removes CLI usage of old components before the components are deleted.

## Acceptance criteria

- [ ] `traces config` uses BootstrapRunner/SettingsService and preserves human and JSON output semantics.
- [ ] `traces config files` uses discovery-only SettingsService flow and still exits successfully when discovery fails.
- [ ] `traces doctor` uses the new full config and discovery-only flows.
- [ ] `traces index` obtains AppConfig and cache setup through BootstrapRunner without pre-creating settings repository state.
- [ ] CLI tests are updated away from `DiscoveryService`, `DiscoveryFlags`, and `InMemoryRepository`.
- [ ] User-facing diagnostics remain actionable and avoid leaking internal pipeline names unless already part of CLI contract.

## Blocked by

- .scratch/config-pipeline-redesign/008-rename-bootstrapper-to-bootstraprunner.md
