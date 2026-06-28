---
labels: [ready-for-agent]
---

# Build Ephemeral Config Pipeline

## Parent

.scratch/config-pipeline-redesign/PRD.md

## What to build

Implement the new ConfigBuilder as an ephemeral typestate pipeline that consumes discovery results, reads config files, validates raw configs into domain configs, merges global and local layers, and returns AppConfig. The builder should not read or write persisted config snapshots.

Migrate SettingsService build behavior to this builder, but leave old repository-backed builder code in place until all callers have moved off it.

## Acceptance criteria

- [ ] `ConfigBuilder` consumes `DiscoveryOutcome` and `ConfigBuilderOptions`.
- [ ] Builder states follow the approved sequence: tracked, trusted, loaded, validated, ready.
- [ ] Loaded state reads TOML/JSON/YAML files into `RawConfig`.
- [ ] Validated state constructs optional `GlobalConfig` and ordered `LocalConfig` stack.
- [ ] Merge applies global first, then local configs outer-ancestor to nearest-ancestor, with nearest local values winning.
- [ ] `build_config` errors when no local candidate can provide the concrete app config base required by command semantics.
- [ ] `AppConfig` finalization does not persist snapshots, versions, or cached views.
- [ ] Tests cover merge precedence and repository-free app config construction.

## Blocked by

- .scratch/config-pipeline-redesign/002-make-discovery-internal-and-linear.md
- .scratch/config-pipeline-redesign/003-simplify-raw-and-domain-config-types.md
