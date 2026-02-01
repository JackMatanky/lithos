---
title: "Requirements Traceability Matrix"
description: "Mapping of functional requirements to architectural components and design decisions"
author: "Jack"
date: "2026-01-23"
last_updated: "2026-01-23"
section: "Requirements Tracking"
---

# Requirements Traceability Matrix

| ID       | Requirement              | Primary Module/Path               | Architectural Strategy                       |
| :------- | :----------------------- | :-------------------------------- | :------------------------------------------- |
| **FR1**  | Modular templates        | `lithos-core/template/aggregate.rs` | Recursive composition model.                 |
| **FR2**  | Interactive prompts      | `lithos-cli/src/commands/`        | Abstracted UI traits.                        |
| **FR3**  | Complex composition      | `lithos-core/template/composition.rs` | Orchestrates section-by-section flow.        |
| **FR4**  | Date functions           | `lithos-core/template/functions.rs` | MiniJinja custom functions.                  |
| **FR5**  | Dynamic commands         | `lithos-core/template/functions.rs` | Whitespace control & shell hooks.            |
| **FR6**  | User functions           | `lithos-core/config/user.rs`      | Discovered scripts registered to engine.     |
| **FR7**  | Advanced hooks           | `lithos-core/template/events.rs`  | Lifecycle events.                            |
| **FR8**  | Metadata schemas         | `lithos-core/schema/aggregate.rs` | Unified aggregate for property specs.        |
| **FR9**  | **Schema-Driven Design** | `lithos-core/template/designer.rs` | Schema properties dictate UI prompts.        |
| **FR10** | Note validation          | `lithos-core/schema/compliance.rs` | Semantic check between Note and Schema.      |
| **FR11** | Enum-driven suggesters   | `lithos-core/template/designer.rs` | Schema enums passed to UI Port.              |
| **FR12** | Directory filters        | `lithos-core/schema/resolver.rs`  | Constraints applied to file pickers.         |
| **FR13** | Date formatting          | `lithos-core/schema/property.rs`  | Format logic in PropertySpec.                |
| **FR14** | Schema inheritance       | `lithos-core/schema/resolver.rs`  | Dereferences `$ref` and processes `extends`. |
| **FR15** | Free-text prompts        | `lithos-cli/src/prompts.rs`       | Implements UI Port via standard input.       |
| **FR16** | Single-choice lists      | `lithos-cli/src/prompts.rs`       | Implements UI Port via fuzzy-select.         |
| **FR17** | Multi-suggesters         | `lithos-cli/src/prompts.rs`       | Implements UI Port via multi-select.         |
| **FR18** | Contextual help          | `lithos-core/error.rs`            | miette-rich diagnostic labels.               |
| **FR19** | Progressive complexity   | `lithos-core/config/aggregate.rs` | User mode toggle in Figment config.          |
| **FR20** | Index & Search           | `lithos-core/db.rs`               | Snapshots from Redb tables.                  |
| **FR21** | Multi-key lookups        | `lithos-core/db.rs`               | B-tree indexed path/uuid/alias keys.         |
| **FR22** | Link resolution          | `lithos-core/note/resolver.rs`    | Logical resolution via aliases.              |
| **FR23** | Metadata queries         | `lithos-core/db.rs`               | Snapshots from RedbSnapshot.                 |
| **FR24** | Vault consistency        | `lithos-core/db.rs`               | Atomic batch writes.                         |
| **FR25** | Large vault scale        | `lithos-core/note/aggregate.rs`   | Zero-copy rkyv::Archive.                     |
| **FR26** | Template packs           | `lithos-core/fs/packs.rs`         | Discovery logic for Git-cloned packs.        |
| **FR27** | Manage schemas           | `lithos-cli/src/commands/schema.rs` | CLI subcommands for schema registry.         |
| **FR28** | App preferences          | `lithos-core/config/aggregate.rs` | Figment provider hierarchy.                  |
| **FR29** | Custom lint rules        | `lithos-core/schema/compliance.rs` | Compliance engine ruleset.                   |
| **FR30** | OS Consistency           | `lithos-cli/`                     | Static binary + .gitattributes.              |
| **FR31** | Terminal access          | `lithos-cli/src/main.rs`          | Primary driver (Clap).                       |
| **FR32** | IDE integration          | `lithos-lsp/` (Future)            | Secondary driver (LSP).                      |
| **FR33** | CI/CD automation         | `lithos-cli/`                     | CLI-first design support.                    |
| **FR34** | Share Git packs          | `mise.toml`                       | Tasks for pack orchestration.                |
| **FR35** | Discover packs           | `README.md`                       | Community documentation.                     |
| **FR36** | Validate 3rd party       | `lithos-core/schema/compliance.rs` | Reuses core compliance engine.               |
| **FR37** | Contribute to packs      | `mise.toml`                       | Pre-commit quality gates.                    |
| **FR38** | Access control           | `lithos-core/fs/permissions.rs`   | OS filesystem permissions.                   |
| **FR39** | Encrypt sensitive files  | `lithos-core/config/crypto.rs`    | age/gpg support via Encryption Port.         |
| **FR40** | Audit logging            | `lithos-core/events/audit.rs`     | Dedicated Audit subscriber.                  |
| **FR41** | CLI subcommands          | `lithos-cli/src/commands/`        | Nested clap subcommands.                     |
| **FR42** | Comprehensive help       | `lithos-cli/src/main.rs`          | Auto-generated help via Clap.                |
| **FR43** | Status & Config view     | `lithos-cli/src/commands/status.rs` | Maps status to Config snapshot.              |
| **FR44** | CLI Vault Ops            | `lithos-cli/src/commands/vault.rs` | Maps CLI intent to Database.                 |
| **FR45** | Format destinations      | `lithos-cli/src/output.rs`        | Config-driven output routing.                |
| **FR46** | Configure CLI behavior   | `lithos-core/config/ui.rs`        | UI preference models.                        |
| **FR47** | Single-word commands     | `lithos-cli/src/main.rs`          | Default fuzzy-pickers for shortcuts.         |
| **FR48** | Actionable errors        | `lithos-core/error.rs`            | High-fidelity miette diagnostics.            |
| **FR49** | Rollback failure         | `lithos-core/db.rs`               | Atomic storage transactions.                 |
| **FR50** | Troubleshooting          | `lithos-cli/src/commands/debug.rs` | Graphical config validation.                 |
