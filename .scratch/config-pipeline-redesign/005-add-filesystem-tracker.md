---
labels: [ready-for-agent]
---

# Add Filesystem Tracker

## Parent

.scratch/config-pipeline-redesign/PRD.md

## What to build

Add the minimal filesystem tracking layer for consumed config files. Tracking is path-hash symlinks under the configured tracking directory and is used for diagnostics/cleanup, not for config snapshot persistence.

This slice adds the replacement mechanism before the old repository storage is removed.

## Acceptance criteria

- [ ] Internal `Tracker` provides `track`, `list_all`, and `clean` operations.
- [ ] Tracking uses a stable hash of canonicalized config file paths.
- [ ] Tracking creates symlinks from `TRACKED_CONFIGS/<path-hash>` to the canonical file path.
- [ ] Tracking is idempotent when called repeatedly for the same path.
- [ ] `clean` removes dangling tracking symlinks.
- [ ] Tempdir tests cover track, list, idempotency, and clean.

## Blocked by

- .scratch/config-pipeline-redesign/002-make-discovery-internal-and-linear.md
