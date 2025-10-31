# Elicitation Summary for Lithos Project

This document provides a comprehensive summary of the key findings, discussions, and decisions made during the advanced elicitation sessions for the Lithos project. It serves as a record of the project's evolution through the planning phase.

# Part 1: Project Brief Elicitations

## 1.1. Initial Vision & Scope Refinement

**Context**: Refining the initial "Post-MVP Vision" and clarifying the project's focus.

**User Feedback**:

- Corrected the phased roadmap to focus on expanding Templater-like capabilities, then custom functions, then LSP support, and finally a TUI/NeoVim plugin.
- Clarified that CI/CD hooks were not part of the project's purpose.

**Outcome**: The "Post-MVP Vision" in the Project Brief was completely rewritten to reflect a clear, phased evolution.

## 1.2. Risk Assessment & Project Focus

**Context**: Clarifying the primary motivation for the project.

**User Feedback**:

- Emphasized that while OSS community adoption is a long-term goal, the primary driver for the MVP is to create a personal tool to facilitate a terminal-based workflow.
- Corrected an over-zealous edit that had removed all mentions of the OSS community, confirming that this vision should remain but that the primary risk was personal workflow fit, not community adoption.

**Outcome**: The Project Brief was adjusted to center the MVP on being a personal productivity tool, while retaining the long-term vision for community engagement. The "Risks" section was updated to reflect this.

## 1.3. Finalizing Project Brief Risks & Open Questions

**Context**: A final review of the risks and open questions before moving to the PRD.

**User Feedback**:

- Pointed out that the "Scope Creep" risk was inconsistent with the clearly defined phased roadmap.
- Clarified that the goal is not to "recreate Templater in Go," but to use it as a reference, with the MVP scope being the definitive feature list.
- Provided crucial technical details on lookups: they should use stable identifiers (paths, specific frontmatter keys) as primary keys, not aliases.

**Outcome**: The "Risks" section was refined to focus on the challenge of selecting the _right subset_ of features. The "Open Questions" were rewritten to be more specific, focusing on implementation details of the defined scope rather than feature selection.

# Part 2: Product Requirements Document (PRD) Elicitations

## 2.1. Elicitation On: PRD Goals & Background Context

**Context**: Stress-testing the foundational goals of the PRD.

**Key Findings & Decisions**:

- **Templater Parity**: It was clarified that 100% parity with Templater is a long-term, post-MVP goal, not a requirement for the initial release. This addresses the risk of user expectation mismatch.
- **Target Audience Confirmation**: Reaffirmed that the project targets terminal-savvy users for the MVP.
- **Primary Success Criterion**: Confirmed the MVP's success is tied to solving the author's personal workflow needs, not broad adoption metrics.
- **Schema Complexity**: Acknowledged as an accepted risk for the MVP. A potential mitigation strategy for the future is to make the schema system an optional feature.
- **Go Library Deferral**: Agreed that creating a separate Go library would add too much overhead to the MVP and should be a post-MVP enhancement.

## 2.2. Elicitation On: PRD Requirements

**Context**: A deep dive to refine, clarify, and validate the list of functional and non-functional requirements.

**Key Findings & Decisions**:

- **Clarified Ambiguities**:
  - **Validation (FR4)**: "Correct types" was clarified to mean standard types (string, boolean, integer, float), custom date types (using Go time layouts), and custom string types (using regex).
  - **Lookups (FR6)**: The role of aliases was clarified; lookups will use stable identifiers like file paths or frontmatter keys as primary keys.
  - **Index Freshness (NFR5)**: A manual `lithos index` command is sufficient for the MVP.
- **Identified Risks**: Acknowledged performance risks with indexing, complexity risks with the schema system, and usability risks with designing non-interactive flags for interactive features.
- **Architectural Decisions**:
  - **Decoupling**: The Indexing and Template engines must be decoupled via a clean internal API.
  - **Componentization**: The fuzzy finder should be built as a core, reusable component.
  - **Index Architecture**: Decided on a hybrid system for the MVP: a persistent on-disk cache stored within the vault (`.lithos/`) and a smaller in-memory index for performance.
- **Scope Refinements**:
  - **Platform Support (NFR1)**: The MVP will target macOS, with Linux support included only if it doesn't add significant complexity.
  - **Core Utilities (FR9)**: The requirement was refined to focus on building PKM-specific functions on top of Go's standard library, not reinventing them.
- **"Hindsight" Insight**: The reflection on template validation led to the idea of a `lithos template lint` command as a valuable post-MVP feature.

## 2.3. Elicitation On: PRD User Interface Design Goals

**Context**: Reviewing the proposed CLI command structure (new, find, index).

**Key Findings & Decisions**:

- **Error Handling**: Confirmed that robust, clear, and actionable error handling is a critical requirement to avoid silent failures.
- **Command Renaming**: The `lithos fuzzy` command was renamed to `lithos find` for better clarity.
- **Testing Strategy**: Acknowledged the need to find a Go library for testing interactive terminal applications.
- **Post-MVP Expansions**: The idea of subcommands (`lithos template list`, `lithos schema validate`) was proposed and added to the PRD as a post-MVP feature to keep the initial scope clean.
- **"Hindsight" Insight**: The idea of "favorite" or "pinned" templates for the fuzzy finder was identified as a valuable potential MVP feature if complexity allows.

## 2.4. Elicitation On: PRD Technical Assumptions

**Context**: Validating the chosen stack of Go libraries.

**Key Findings & Decisions**:

- **Storage Interface**: Confirmed the critical decision to define a `Storage` interface from the start to decouple indexing logic from the cache implementation.
- **CLI & Configuration Framework**: Affirmed that Cobra/Viper is a pragmatic choice, even if slightly heavy for the MVP, as it provides long-term benefits.
- **Fuzzy Finder Library**: The library ktr0731/go-fuzzyfinder was selected for the project.
- **Rationale Documentation**: Agreed to add a "Rationale" subsection to document the "why" behind each library choice.

## 2.5. Elicitation On: Epic Structure

**Context**: A multi-stage discussion to define the high-level development roadmap.

**Key Findings & Decisions**:

- **Initial Critique**: The first proposal of three large epics was rejected as being too broad, delaying value delivery and hiding integration risks. A subsequent proposal of eight epics was deemed too granular.
- **Guiding Principle**: The key principle was established: **An epic is optimally sized when it delivers a significant, coherent, and independently testable piece of user value.**
- **Final Structure Adopted**: A balanced, **5-epic "Vertical Slice First" structure was adopted**. This was a key strategic decision that ensures a tangible, testable artifact is produced in the first epic (mitigating integration risk) and prioritizes early value delivery.

## 2.6. Elicitation On: Story Definition & Granularity

**Context**: This was a series of detailed discussions to refine the stories within each epic. Several key principles were established, including a strict story sizing guideline ("2-4 hours for a junior developer"), a focus on testability as a first-class concern, and the creation of explicit architectural stories to define key interfaces.

### 2.6.1. Elicitation on Stories for Epic 1 (Foundational CLI)

**Key Findings & Decisions**:

- **Story Granularity**: A decision was made to keep stories highly granular and not combine related implementation steps, favoring clarity and parallelizability.
- **CI/CD Scope**: The initial story for CI was de-scoped. The `goreleaser` setup was deferred to a post-MVP phase to accelerate the delivery of core features.
- **Architectural Foresight**: The "Hindsight" reflection led to adding an AC to ensure the template function engine was designed to be extensible from the start.
- **Testability**: The QA perspective led to the decision to add a foundational story to create the "golden files" test vault.
- **AC Refinement**: All ACs were reviewed and made more specific (e.g., clarifying output file locations, filenames, and error messages).

### 2.6.2. Elicitation on Stories for Epic 2 (Configuration & Schemas)

**Key Findings & Decisions**:

- **Story Breakdown**: A key decision was made to break down the complex "Implement Schema Inheritance" feature into three smaller, more manageable stories: **Single-Level Inheritance**, **Multi-Level Inheritance**, and **Circular Dependency Detection**.
- **Architectural Foresight**: The "Hindsight" reflection led to adding an AC to Story 2.1, requiring the creation of a `Config` struct to make future configuration changes easier.
- **AC Refinement**: ACs were updated to specify the config file search path (`lithos.yaml` traversal) and to require clearer error logging for invalid schema files.

### 2.6.3. Elicitation on Stories for Epic 3 (Vault Indexing)

**Key Findings & Decisions**:

- **Story Granularity**: The initial, larger stories were broken down into five distinct, single-responsibility stories: **1. Define Interface**, **2. Implement Cache**, **3. Implement Scanner**, **4. Implement Parser**, **5. Implement Command Logic**. This was a crucial step to de-risk the development of the most complex component by building it from testable units.

### 2.6.4. Elicitation on Stories for Epic 5 (Interactive Engine)

**Key Findings & Decisions**:

- **Architectural Foresight**: The elicitation process identified the need for an abstract `Interactive` interface to decouple the template engine from the specific prompt/fuzzy-finder libraries. This was added as a new architectural story.
- **AC Refinement**: ACs were added to handle edge cases, such as what happens when the fuzzy finder finds no templates.

### 2.6.5. Elicitation on Stories for Epic 4 (Lookups Epic 5 (Lookups & Validation) Validation)

**Key Findings & Decisions**:

- **Pivotal Architectural Shift**: The "Hindsight" reflection led to a major decision: instead of building a simple, hardcoded lookup function, the stories were restructured to build a **flexible, generic query engine for the MVP**. The lookup function will now be a thin, convenience wrapper around this more powerful core.
- **Risk Mitigation**: An AC was added to the new query story to handle cases where the index is stale or empty, ensuring the user gets a helpful warning.
- **Validation Scope**: The scope of "Simple Type" validation was clarified to explicitly handle integer vs. float and to limit date validation to Go's standard library layouts.
