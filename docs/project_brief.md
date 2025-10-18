# Project Brief: Lithos

## Executive Summary

**Project:** Lithos

**Summary:** Lithos is a command-line interface (CLI) tool built in Go that provides powerful, scriptable template generation for Obsidian vaults. Inspired by the functionality of popular Obsidian plugins like Templater and Metadata Menu, Lithos allows users to automate note creation and management directly from the terminal, leveraging Go's native templating engine for a fast and dependency-free experience.

## Problem Statement

While Obsidian's plugin ecosystem offers powerful templating engines like Templater and sophisticated metadata management with Metadata Menu, their functionality is exclusively confined to the Obsidian desktop application. This creates a significant workflow bottleneck for a growing number of power users, developers, and writers who manage their vaults from external environments.

The key issues are:

- **Lack of Portability:** Powerful templating and metadata-driven features cannot be applied consistently when creating or managing notes through CLI workflows, Git repositories, or CI/CD automation pipelines.
- **Workflow Friction:** Users who primarily work in IDEs or terminals are forced into an inefficient context switch, leaving their preferred environment simply to apply a template within the Obsidian UI.
- **Inconsistent Note Structure:** Without access to these tools externally, notes created outside of Obsidian often lack the consistent structure and rich metadata that are crucial for a well-organized vault, undermining features like file classes and automated queries.

This siloed functionality prevents Obsidian from being a truly app-agnostic, plain-text knowledge management system, limiting the potential for advanced automation and integration.

## Proposed Solution

To bridge this gap, Lithos will be a standalone Go CLI tool that operates directly on an Obsidian vault's markdown files. It will bring the core power of in-app templating to any terminal or scripted environment.

Key features include:

- **App-Agnostic Templating:** Implements robust template generation with prompts, suggesters, variables, and other key functionalities, accessible from any terminal or IDE.
- **Portable Metadata Schemas:** Supports file class metadata schemas, allowing for the enforcement of required fields, enums, and value sources outside of Obsidian.
- **Consistent Frontmatter:** Automatically generates and validates frontmatter against defined schemas, ensuring metadata integrity across all notes, regardless of where they are created.
- **Vault-Wide Alias Resolution:** Provides alias resolution to ensure that links and references remain intact throughout the vault during template processing.
- **Shareable Template Packs:** Enables the distribution and reuse of templates and schemas as "template packs" via Git repositories, promoting community sharing and standardization.

By decoupling templating from the Obsidian application, Lithos empowers users to integrate their knowledge base seamlessly into broader development and automation workflows.

## Target Users

### Primary User Segment: Individual Power Users & Enthusiasts

- **Profile:** Developers, technical writers, and researchers who are comfortable with the command line and prefer to manage their notes from IDEs (like VS Code), terminals (like iTerm2 or Windows Terminal), or through automated scripts.
- **Behaviors:** They version their vaults with Git, use scripts (Python, Go, shell) to automate note creation, and often work with their notes as plain text files in their editor of choice. They value efficiency, automation, and keyboard-driven workflows.
- **Pain Points:** The need to open the Obsidian app breaks their CLI-centric flow, slowing them down and preventing full automation of their knowledge management process.

### Primary User Segment: OSS Communities & Template Authors

- **Profile:** Maintainers of open-source projects, community leaders, and individuals who create and share "starter vaults" or template packs for specific use cases (e.g., Zettelkasten, PARA, project management).
- **Behaviors:** They use Git to manage and distribute their vault structures and templates. They need a reliable way to ensure that users of their packs can apply templates and schemas consistently, without being locked into the Obsidian UI.
- **Pain Points:** It is difficult to bundle and distribute complex templating logic and metadata schemas that work reliably outside of the Obsidian application, limiting the portability and reach of their creations.

### Secondary User Segment: Teams

- **Profile:** Small technical teams using a shared Obsidian vault for documentation, project management, or a collaborative knowledge base.
- **Behaviors:** They manage their shared vault via a central Git repository and may use CI/CD pipelines to validate or process notes.
- **Pain Points:** Ensuring every team member applies the correct templates and metadata schemas is challenging. An external, scriptable tool would allow them to enforce consistency programmatically. While supported, this is not the primary focus for the initial MVP.

## Goals & Success Metrics

### Business Objectives

- **Primary Goal:** Empower individuals and OSS communities to create, manage, and validate Obsidian-compatible templates and metadata outside the Obsidian app, via a portable Go-based CLI.
- **Objective 1:** Enable cross-environment template generation for personal or shareable packs, in both interactive and non-interactive modes.
- **Objective 2:** Provide robust support for file class metadata schemas, including required fields, types, and enums.
- **Objective 3:** Implement schema-based validation for both frontmatter and filenames.
- **Objective 4:** Ensure vault-wide alias resolution to maintain link consistency.
- **Objective 5:** Deliver a portable, easy-to-install single binary for each major OS.
- **Objective 6:** Foster community adoption through the creation and sharing of OSS template packs.

### Key Performance Indicators (KPIs)

- **Capability Coverage %:** Achieve ≥ 60% coverage of required Templater behaviors (must include prompts & suggesters).
- **Schema Compliance Rate:** Ensure ≥ 98% of files generated via Lithos pass metadata validation.
- **Cross-Platform Consistency:** Achieve 100% parity on golden path tests across macOS, Linux, and Windows.
- **Time to First Note:** A new user should be able to go from installation to their first generated note in ≤ 2 minutes.
- **Template Render Latency:** Maintain a typical generation time of ≤ 300 ms for a single file.
- **Community Adoption:** See at least 3 distinct, reusable OSS template packs published by the author or the community.
- **Documentation Clarity:** A user should be able to get from the README to their first successful run in ≤ 5 minutes.

## MVP Scope

### Core Features (Must Have)

The MVP must enable individuals and OSS communities to generate, manage, and validate Obsidian-compatible notes outside the app, using schemas and templates as the source of truth.

- **Template Composition:**
  - Users must be able to create templates composed of multiple, reusable sections.
  - Templates can be assembled like building blocks, with support for variables.
  - Composition must prevent errors from missing sections or circular references.
- **Frontmatter Handling & Validation:**
  - Read, merge, and write frontmatter, preserving unknown fields.
  - Validate notes against metadata class definitions, enforcing required fields, allowed values, and correct types.
  - Validation errors must produce clear, actionable feedback.
- **Metadata Class Integration:**
  - Load metadata class definitions (e.g., Contact, Organization).
  - Support inheritance (extending) between metadata classes.
  - Define how fields are validated, displayed, and sourced (free text, fixed list, or from other notes).
- **Vault-Wide Lookup & Indexing:**
  - Maintain an up-to-date index of notes across the vault.
  - Retrieve notes by path, title, or alias, with deterministic alias resolution.
  - Support lookups of metadata fields from other notes for use in templates.
- **Interactive Input:**
  - Support interactive prompts for free-text and selections from lists.
  - Allow input to be passed non-interactively for automation.
- **Core Utilities:**
  - Provide essential date/time operations, string transformations, path utilities, and link generation.
- **Cross-Platform Consistency:**
  - Produce identical output on macOS, Linux, and Windows.

### Out of Scope for V1 (Non-Goals)

- **No GUI or Plugin Integration:** Lithos is a CLI-only tool and will not integrate directly with the Obsidian application's UI.
- **No Hosted Template Registry:** Template packs will be distributed and managed via Git repositories only.
- **Limited Scripting Runtime:** Lithos is not a full JavaScript execution environment. It will support a targeted subset of Templater's most-used features (e.g., prompts, suggesters, date operations, includes) rather than arbitrary script execution.
- **Complement, Don't Compete:** The tool is designed to work alongside Obsidian, not replace it.
- **Advanced graph queries:** Backlinks, block IDs, and heading references are not part of the MVP.

## Post-MVP Vision

- **Phase 2 - Templater Parity:** Expand the built-in templating functions to cover the majority of Templater's capabilities, providing a near feature-complete experience for common use cases.
- **Phase 3 - Custom Functions:** Introduce support for user-defined functions and the execution of safe, sandboxed system commands, allowing for greater customization and power.
- **Phase 4 - Advanced IDE Integration:** Implement a Language Server Protocol (LSP) provider to offer rich, in-editor support for Lithos templates and schemas directly within IDEs like VS Code.
- **Long-term Vision:** Evolve Lithos into a standalone Terminal User Interface (TUI) or a NeoVim plugin, incorporating advanced features like bidirectional linking and Dataview-like querying capabilities, transforming it from a templating tool into a comprehensive CLI-based knowledge management environment.
- **Expansion Opportunities:** Explore support for other plain-text knowledge management tools (e.g., Logseq).

## Technical Considerations

- **Platform Requirements:** The CLI must be delivered as a single, self-contained binary for macOS (x86_64, ARM64), Linux (x86_64, ARM64), and Windows (x86_64).
- **Technology Preferences:**
  - **Core Logic:** Go. Utilize standard libraries like `text/template` for templating and a robust CLI library such as `Cobra` or `Viper` for command parsing and configuration.
  - **Vault Indexing:** For the MVP, a simple file-based index (e.g., JSON) is preferred for portability. For future performance, an embedded key-value store like BoltDB could be considered.
- **Architecture Considerations:** A modular CLI architecture is preferred, with clear separation between parsing, indexing, template execution, and validation logic. Template packs will be managed via Git, so integration with standard Git workflows is essential.

## Constraints & Assumptions

- **Constraints:**
  - **Timeline:** Undefined, but focused on a rapid MVP release to gather feedback.
  - **Resources:** This is scoped as a solo developer or small OSS community project.
  - **Technical:** Must be a standalone Go binary with no external runtime dependencies (e.g., no Node.js, Python).
- **Key Assumptions:**
  - The target audience is technically proficient and comfortable with both the command line and Git.
  - The feature set of Templater and Metadata Menu is a valid and desired model to replicate for external use.
  - Go's native templating capabilities are sufficient for the MVP's logic requirements.

## Risks & Open Questions

- **Key Risks:**
  - **Personal Workflow Fit Risk:** The final tool might not integrate as seamlessly into the intended personal workflow as envisioned, requiring significant rework.
  - **Feature Prioritization Risk:** Defining the _most valuable 60%_ of Templater's features for the MVP could lead to development delays if low-impact but complex features are chosen over core, high-value ones.
  - **Performance Risk:** Vault indexing could be slow on exceptionally large vaults, negatively impacting the user experience.
- **Open Questions:**
  - What are the precise implementation details and limitations for each of the core capabilities defined in the MVP scope?
  - What is the definitive priority order for note lookups (e.g., file path > YAML `id` key > filename)? How should conflicts be handled if multiple notes match a lookup query?
  - What is the final schema for the configuration file that defines a template pack?

## Next Steps

- **Immediate Actions:**
  1. Finalize and approve this Project Brief.
  2. Begin creation of a detailed Product Requirements Document (PRD) that breaks down MVP features into epics and user stories.
  3. Initiate the high-level architecture and system design based on the technical considerations outlined above.
- **PM Handoff:** This Project Brief provides the full context for Lithos. The next step is to transition to a Product Manager role to create the PRD. The brief should be used as the source of truth to define detailed requirements, epics, and stories, ensuring they align with the stated MVP scope and goals.
