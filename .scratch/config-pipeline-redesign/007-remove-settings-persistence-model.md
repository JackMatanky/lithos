---
labels: [ready-for-agent]
---

# Remove Settings Persistence Model

## Parent

.scratch/config-pipeline-redesign/PRD.md

## What to build

Delete the old settings persistence model only after all production callers have migrated to the new discovery, config builder, tracker, and trust paths. This is a cleanup/removal slice, not a migration slice.

If any component still imports the old repository, storage, version, view, event, or processor APIs, migrate that caller first instead of deleting underneath it.

## Acceptance criteria

- [ ] No production caller imports settings repository/storage APIs before deletion.
- [ ] Old config repository traits and storage modules are removed from settings.
- [ ] Old config snapshot views, version counters, events, stale-analysis processor, and resolver plan are removed or made unreachable only in tests if temporarily necessary.
- [ ] Settings crate no longer depends on `traces-db`, `redb`, or `rkyv` for config persistence.
- [ ] Public exports no longer expose old `Config`, `Vault`, `Global`, repository, storage, or discovery-port names.
- [ ] Compile errors from removed components are fixed by using the new SettingsService and AppConfig path, not by adding compatibility wrappers.
- [ ] Architecture tests assert the persistence model is gone.

## Blocked by

- .scratch/config-pipeline-redesign/004-build-ephemeral-config-pipeline.md
- .scratch/config-pipeline-redesign/005-add-filesystem-tracker.md
- .scratch/config-pipeline-redesign/006-add-trust-and-ignore-system.md
