---
labels: [ready-for-agent]
---

# Add Trust And Ignore System

## Parent

.scratch/config-pipeline-redesign/PRD.md

## What to build

Add the trust and ignore system used by SettingsService and future CLI trust commands. Trust state is stored as path-hash symlinks, ignored configs are filtered or skipped, global configs are trusted automatically, and paranoid/CI behavior is represented in the trust mode.

This slice creates the new trust path before old security/persistence assumptions are removed.

## Acceptance criteria

- [ ] Public `Trust` API supports trust, untrust, ignore, trusted check, ignored check, and status/listing needed by CLI.
- [ ] Trust and ignore entries use stable path-hash symlinks.
- [ ] Global config candidates are treated as trusted.
- [ ] CI mode trusts without prompting.
- [ ] Paranoid mode verifies trusted content hashes according to the PRD’s security model.
- [ ] Safe configs without trust-requiring directives can skip prompts.
- [ ] ConfigBuilder’s trust phase uses the trust system and skips ignored candidates.
- [ ] Tempdir tests cover trust, untrust, ignore, status, CI mode, and paranoid mismatch.

## Blocked by

- .scratch/config-pipeline-redesign/002-make-discovery-internal-and-linear.md
- .scratch/config-pipeline-redesign/005-add-filesystem-tracker.md
