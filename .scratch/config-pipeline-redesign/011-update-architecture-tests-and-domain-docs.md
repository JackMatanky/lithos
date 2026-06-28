---
labels: [ready-for-agent]
---

# Update Architecture Tests And Domain Docs

## Parent

.scratch/config-pipeline-redesign/PRD.md

## What to build

Bring architecture tests and domain documentation in line with the completed settings pipeline redesign. This slice should document the final state after migration and deletion, not paper over temporary compatibility shims from earlier slices.

## Acceptance criteria

- [ ] Settings context docs use the final domain language: SettingsService, AppConfig, GlobalConfig, LocalConfig, RawConfig, DiscoveryOutcome, Tracker, Trust.
- [ ] Stale terms from the removed persistence model are deleted or explicitly marked historical where needed.
- [ ] Architecture tests assert discovery is internal and settings has one inbound service boundary.
- [ ] Architecture tests assert settings no longer exposes repository/storage/snapshot/version APIs.
- [ ] ADR supersession notes from the PRD are reflected in the relevant docs.
- [ ] Documentation matches public exports and tested behavior.

## Blocked by

- .scratch/config-pipeline-redesign/007-remove-settings-persistence-model.md
- .scratch/config-pipeline-redesign/009-rewire-existing-cli-commands.md
- .scratch/config-pipeline-redesign/010-add-trust-cli-commands.md
