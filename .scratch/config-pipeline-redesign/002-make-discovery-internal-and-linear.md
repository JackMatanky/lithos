---
labels: [ready-for-agent]
---

# Make Discovery Internal And Linear

## Parent

.scratch/config-pipeline-redesign/PRD.md

## What to build

Move discovery behind the Settings boundary and implement the linear discovery flow: normalize options and environment into internal input, collect local candidates, collect global candidates, filter/dedupe, and return a discovery outcome. Discovery should no longer be modeled as an external port for new code.

Migrate discovery consumers to the new SettingsService API before marking old discovery service/port types unused. Do not delete old components here if any caller still needs them.

## Acceptance criteria

- [ ] `DiscoveryInput` is internal and constructed from `DiscoveryOptions` plus internally-read settings environment variables.
- [ ] `DiscoveryProcessor` uses explicit transition methods for local collection, global collection, and finish.
- [ ] Discovery flow is linear and has no old branch/cache-resolution phase model.
- [ ] Local collection returns candidates in outer-ancestor to nearest-ancestor order.
- [ ] `TRACES_DEFAULT_VAULT` is only used as fallback when normal local collection finds no local candidate.
- [ ] Global collection follows suppress, flag, env, platform-dir precedence.
- [ ] Exact filename slices in path constants replace marker-pattern/extension iteration for new discovery code.
- [ ] Dedupe/desymlink and ignored-path filtering happen before returning `DiscoveryOutcome`.

## Blocked by

- .scratch/config-pipeline-redesign/001-define-settings-service-boundary.md
