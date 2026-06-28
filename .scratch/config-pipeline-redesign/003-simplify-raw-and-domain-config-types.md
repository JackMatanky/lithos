---
labels: [ready-for-agent]
---

# Simplify Raw And Domain Config Types

## Parent

.scratch/config-pipeline-redesign/PRD.md

## What to build

Replace the persisted config-domain shape with ephemeral domain types. Config files deserialize into one all-optional raw DTO, then domain construction parses that raw shape into global, local, or app config values with explicit forbidden-field errors.

Keep compatibility shims only where required for current callers; remove those shims later after callers migrate.

## Acceptance criteria

- [ ] A unified `RawConfig` can deserialize config fields used by both global and local files.
- [ ] TOML config parsing is supported by default.
- [ ] JSON/YAML parsing is available according to the project dependency/feature decision in the PRD.
- [ ] `GlobalConfig::try_from(RawConfig)` rejects `cache` as a forbidden field.
- [ ] `LocalConfig::try_from(RawConfig)` rejects `trusted_vaults` as a forbidden field.
- [ ] `LocalConfig` carries `base`, `path`, and a derived/defaultable name.
- [ ] `AppConfig` is constructable without a repository or database.
- [ ] Inline-data tests cover raw parsing, forbidden fields, and default construction.

## Blocked by

- .scratch/config-pipeline-redesign/001-define-settings-service-boundary.md
