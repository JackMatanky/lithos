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
| **FR1**  | Modular templates        | `traces-core/template/aggregate.rs` | Recursive composition model.                 |
| **FR2**  | Interactive prompts      | `traces-cli/src/commands/`        | Abstracted UI traits.                        |
| **FR3**  | Complex composition      | `traces-core/template/composition.rs` | Orchestrates section-by-section flow.        |
| **FR4**  | Date functions           | `traces-core/template/functions.rs` | MiniJinja custom functions.                  |
| **FR5**  | Dynamic commands         | `traces-core/template/functions.rs` | Whitespace control & shell hooks.            |
| **FR6**  | User functions           | `traces-core/config/user.rs`      | Discovered scripts registered to engine.     |
| **FR7**  | Advanced hooks           | `traces-core/template/events.rs`  | Lifecycle events.                            |
| **FR8**  | Metadata schemas         | `traces-core/schema/aggregate.rs` | Unified aggregate for property specs.        |
| **FR9**  | **Schema-Driven Design** | `traces-core/template/designer.rs` | Schema properties dictate UI prompts.        |
| **FR10** | Note validation          | `traces-core/schema/compliance.rs` | Semantic check between Note and Schema.      |
| **FR11** | Enum-driven suggesters   | `traces-core/template/designer.rs` | Schema enums passed to UI Port.              |
| **FR12** | Directory filters        | `traces-core/schema/resolver.rs`  | Constraints applied to file pickers.         |
| **FR13** | Date formatting          | `traces-core/schema/property.rs`  | Format logic in PropertySpec.                |
| **FR14** | Schema inheritance       | `traces-core/schema/resolver.rs`  | Dereferences `$ref` and processes `extends`. |
| **FR15** | Free-text prompts        | `traces-cli/src/prompts.rs`       | Implements UI Port via standard input.       |
| **FR16** | Single-choice lists      | `traces-cli/src/prompts.rs`       | Implements UI Port via fuzzy-select.         |
| **FR17** | Multi-suggesters         | `traces-cli/src/prompts.rs`       | Implements UI Port via multi-select.         |
| **FR18** | Contextual help          | `traces-core/error.rs`            | miette-rich diagnostic labels.               |
| **FR19** | Progressive complexity   | `traces-core/config/aggregate.rs` | User mode toggle in Figment config.          |
| **FR20** | Index & Search           | `traces-core/db.rs`               | Snapshots from Redb tables.                  |
| **FR21** | Multi-key lookups        | `traces-core/db.rs`               | B-tree indexed path/uuid/alias keys.         |
| **FR22** | Link resolution          | `traces-core/note/resolver.rs`    | Logical resolution via aliases.              |
| **FR23** | Metadata queries         | `traces-core/db.rs`               | Snapshots from RedbSnapshot.                 |
| **FR24** | Vault consistency        | `traces-core/db.rs`               | Atomic batch writes.                         |
| **FR25** | Large vault scale        | `traces-core/note/aggregate.rs`   | Zero-copy rkyv::Archive.                     |
| **FR26** | Template packs           | `traces-core/fs/packs.rs`         | Discovery logic for Git-cloned packs.        |
| **FR27** | Manage schemas           | `traces-cli/src/commands/schema.rs` | CLI subcommands for schema registry.         |
| **FR28** | App preferences          | `traces-core/config/aggregate.rs` | Figment provider hierarchy.                  |
| **FR29** | Custom lint rules        | `traces-core/schema/compliance.rs` | Compliance engine ruleset.                   |
| **FR30** | OS Consistency           | `traces-cli/`                     | Static binary + .gitattributes.              |
| **FR31** | Terminal access          | `traces-cli/src/main.rs`          | Primary driver (Clap).                       |
| **FR32** | IDE integration          | `traces-lsp/` (Future)            | Secondary driver (LSP).                      |
| **FR33** | CI/CD automation         | `traces-cli/`                     | CLI-first design support.                    |
| **FR34** | Share Git packs          | `mise.toml`                       | Tasks for pack orchestration.                |
| **FR35** | Discover packs           | `README.md`                       | Community documentation.                     |
| **FR36** | Validate 3rd party       | `traces-core/schema/compliance.rs` | Reuses core compliance engine.               |
| **FR37** | Contribute to packs      | `mise.toml`                       | Pre-commit quality gates.                    |
| **FR38** | Access control           | `traces-core/fs/permissions.rs`   | OS filesystem permissions.                   |
| **FR39** | Encrypt sensitive files  | `traces-core/config/crypto.rs`    | age/gpg support via Encryption Port.         |
| **FR40** | Audit logging            | `traces-core/events/audit.rs`     | Dedicated Audit subscriber.                  |
| **FR41** | CLI subcommands          | `traces-cli/src/commands/`        | Nested clap subcommands.                     |
| **FR42** | Comprehensive help       | `traces-cli/src/main.rs`          | Auto-generated help via Clap.                |
| **FR43** | Status & Config view     | `traces-cli/src/commands/status.rs` | Maps status to Config snapshot.              |
| **FR44** | CLI Vault Ops            | `traces-cli/src/commands/vault.rs` | Maps CLI intent to Database.                 |
| **FR45** | Format destinations      | `traces-cli/src/output.rs`        | Config-driven output routing.                |
| **FR46** | Configure CLI behavior   | `traces-core/config/ui.rs`        | UI preference models.                        |
| **FR47** | Single-word commands     | `traces-cli/src/main.rs`          | Default fuzzy-pickers for shortcuts.         |
| **FR48** | Actionable errors        | `traces-core/error.rs`            | High-fidelity miette diagnostics.            |
| **FR49** | Rollback failure         | `traces-core/db.rs`               | Atomic storage transactions.                 |
| **FR50** | Troubleshooting          | `traces-cli/src/commands/debug.rs` | Graphical config validation.                 |
