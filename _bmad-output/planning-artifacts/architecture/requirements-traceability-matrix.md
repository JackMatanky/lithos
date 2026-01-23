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
| **FR1**  | Modular templates        | `domain/template/aggregate.rs`       | Recursive composition model.                 |
| **FR2**  | Interactive prompts      | `domain/ports/api/ui.rs`          | Abstracted UI traits.                        |
| **FR3**  | Complex composition      | `app/template/composer.rs`        | Orchestrates section-by-section flow.        |
| **FR4**  | Date functions           | `adapters/spi/template/`          | MiniJinja custom functions.                  |
| **FR5**  | Dynamic commands         | `adapters/spi/template/`          | Whitespace control & shell hooks.            |
| **FR6**  | User functions           | `adapters/spi/config/`            | Discovered scripts registered to engine.     |
| **FR7**  | Advanced hooks           | `app/template/composer.rs`        | Lifecycle events on Hybrid Bus.              |
| **FR8**  | Metadata schemas         | `domain/schema/aggregate.rs`         | Unified aggregate for property specs.        |
| **FR9**  | **Schema-Driven Design** | `app/template/designer.rs`        | Schema properties dictate UI prompts.        |
| **FR10** | Note validation          | `app/compliance/engine.rs`        | Semantic check between Note and Schema.      |
| **FR11** | Enum-driven suggesters   | `app/template/designer.rs`        | Schema enums passed to UI Port.              |
| **FR12** | Directory filters        | `adapters/spi/schema/`            | Constraints applied to file pickers.         |
| **FR13** | Date formatting          | `domain/schema/aggregate.rs`         | Format logic in PropertySpec.                |
| **FR14** | Schema inheritance       | `adapters/spi/schema/resolver.rs` | Dereferences `$ref` and processes `extends`. |
| **FR15** | Free-text prompts        | `adapters/api/cli/`               | Implements UI Port via standard input.       |
| **FR16** | Single-choice lists      | `adapters/api/cli/`               | Implements UI Port via fuzzy-select.         |
| **FR17** | Multi-suggesters         | `adapters/api/cli/`               | Implements UI Port via multi-select.         |
| **FR18** | Contextual help          | `domain/errors.rs`                | miette-rich diagnostic labels.               |
| **FR19** | Progressive complexity   | `adapters/spi/config/`            | User mode toggle in Figment config.          |
| **FR20** | Index & Search           | `app/queries/`                    | Snapshots from Redb tables.                  |
| **FR21** | Multi-key lookups        | `adapters/spi/storage/`           | B-tree indexed path/uuid/alias keys.         |
| **FR22** | Link resolution          | `app/services/resolver.rs`        | Logical resolution via aliases.              |
| **FR23** | Metadata queries         | `app/queries/`                    | Snapshots from RedbSnapshot.                 |
| **FR24** | Vault consistency        | `app/indexer/`                    | Single-writer transactions.                  |
| **FR25** | Large vault scale        | `domain/note/aggregate.rs`           | Zero-copy rkyv::Archive.                     |
| **FR26** | Template packs           | `adapters/spi/fs/`                | Discovery logic for Git-cloned packs.        |
| **FR27** | Manage schemas           | `adapters/api/cli/`               | CLI subcommands for schema registry.         |
| **FR28** | App preferences          | `adapters/spi/config/`            | Figment provider hierarchy.                  |
| **FR29** | Custom lint rules        | `app/compliance/`                 | Compliance engine ruleset.                   |
| **FR30** | OS Consistency           | `lithos/`                         | Static binary + .gitattributes.              |
| **FR31** | Terminal access          | `adapters/api/cli/`               | Primary driver (Clap).                       |
| **FR32** | IDE integration          | `adapters/api/lsp/`               | Secondary driver (LSP).                      |
| **FR33** | CI/CD automation         | `lithos/`                         | CLI-first design support.                    |
| **FR34** | Share Git packs          | `mise.toml`                       | Tasks for pack orchestration.                |
| **FR35** | Discover packs           | `README.md`                       | Community documentation.                     |
| **FR36** | Validate 3rd party       | `app/compliance/`                 | Reuses core compliance engine.               |
| **FR37** | Contribute to packs      | `mise.toml`                       | Pre-commit quality gates.                    |
| **FR38** | Access control           | `adapters/spi/fs/`                | OS filesystem permissions.                   |
| **FR39** | Encrypt sensitive files  | `adapters/spi/config/`            | age/gpg support via Encryption Port.         |
| **FR40** | Audit logging            | `adapters/spi/events/`            | Dedicated Audit subscriber.                  |
| **FR41** | CLI subcommands          | `adapters/api/cli/`               | Nested clap subcommands.                     |
| **FR42** | Comprehensive help       | `adapters/api/cli/`               | Auto-generated help via Clap.                |
| **FR43** | Status & Config view     | `adapters/api/cli/`               | Maps status to Config snapshot.              |
| **FR44** | CLI Vault Ops            | `app/commands/`                   | Maps CLI intent to Indexer mailbox.          |
| **FR45** | Format destinations      | `app/commands/`                   | Config-driven output routing.                |
| **FR46** | Configure CLI behavior   | `domain/`                  | UI preference models.                        |
| **FR47** | Single-word commands     | `adapters/api/cli/`               | Default fuzzy-pickers for shortcuts.         |
| **FR48** | Actionable errors        | `domain/errors.rs`                | High-fidelity miette diagnostics.            |
| **FR49** | Rollback failure         | `app/indexer/`                    | Atomic storage transactions.                 |
| **FR50** | Troubleshooting          | `adapters/api/cli/`               | Graphical config validation.                 |
