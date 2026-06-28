---
labels: [ready-for-agent]
---

# Add Trust CLI Commands

## Parent

.scratch/config-pipeline-redesign/PRD.md

## What to build

Add user-facing trust commands over the new Trust module. Keep CLI as a thin orchestration layer: parse command intent, call Trust/SettingsService discovery as needed, and render status or errors.

## Acceptance criteria

- [ ] `traces trust <path>` marks a config path trusted.
- [ ] `traces untrust <path>` removes trust for a config path.
- [ ] `traces trust --ignore <path>` marks a config path ignored.
- [ ] `traces trust --show` displays trusted, ignored, and untrusted discovered configs.
- [ ] `traces trust --all` trusts all currently untrusted discovered configs.
- [ ] Human and JSON output are deterministic and tested where the CLI supports both.
- [ ] Command handlers do not own trust storage rules.

## Blocked by

- .scratch/config-pipeline-redesign/006-add-trust-and-ignore-system.md
- .scratch/config-pipeline-redesign/009-rewire-existing-cli-commands.md
