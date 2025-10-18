# Goals and Background Context

## Goals

- **Primary Goal:** Empower individuals and OSS communities to create, manage, and validate structured Markdown templates and metadata outside the Obsidian app, via a portable Go-based CLI.
- Enable cross-environment template generation for personal or shareable packs, in both interactive and non-interactive modes.
- Provide robust support for file class metadata schemas, including required fields, types, and enums.
- Implement schema-based validation for both frontmatter and filenames.

**For detailed business objectives and KPIs**, see [Project Brief: Goals & Success Metrics](project_brief.md#goals--success-metrics).

## Background Context

Currently, powerful Obsidian templating tools like Templater are confined to the application's UI. This creates a clear problem for users who manage their knowledge vaults from the command line or through automated Git workflows, as notes created externally often lack the consistent structure and rich metadata applied by internal plugins. This friction effectively silos advanced templating from the broader plain-text ecosystem.

Lithos solves this problem by providing a standalone, Go-based CLI for generating structured Markdown files. It brings the core power of in-app templating and metadata validation to any terminal-based PKM workflow. By decoupling these features from a specific application, Lithos ensures that users can apply consistent, schema-driven templates and metadata to their notes programmatically, enhancing any PKM system that relies on well-formed, portable Markdown.

**For additional context:**

- **Problem Statement & Market Analysis**: See [Project Brief: Problem Statement](project_brief.md#problem-statement)
- **Target User Personas**: See [Project Brief: Target Users](project_brief.md#target-users) for detailed profiles of:
  - Primary: Individual Power Users & Enthusiasts (CLI-centric developers, technical writers)
  - Primary: OSS Communities & Template Authors (community leaders, starter vault creators)
  - Secondary: Technical Teams (collaborative documentation)
- **MVP Scope & Constraints**: See [Project Brief: MVP Scope](project_brief.md#mvp-scope) for detailed feature boundaries and non-goals

## Change Log

| Date       | Version | Description                                                 | Author   |
| ---------- | ------- | ----------------------------------------------------------- | -------- |
| 2025-09-29 | 0.1.0   | Initial PRD Draft. Refined context.                         | John, PM |
| 2025-09-30 | 0.2.0   | Refined requirements based on detailed feedback.            | John, PM |
| 2025-09-30 | 0.2.1   | Reorganized FRs and clarified Core Library scope.           | John, PM |
| 2025-09-30 | 0.2.2   | Refined FR3, FR7, and FR8 for technical clarity.            | John, PM |
| 2025-09-30 | 0.3.0   | Added User Interface Design Goals for CLI interaction.      | John, PM |
| 2025-09-30 | 0.3.1   | Refined UI Goals based on elicitation feedback.             | John, PM |
| 2025-09-30 | 0.4.0   | Added and refined Technical Assumptions section.            | John, PM |
| 2025-10-03 | 0.5.0   | Defined and refined high-level Epic structure for MVP.      | John, PM |
| 2025-10-03 | 0.6.0   | Added stories for Epic 1.                                   | John, PM |
| 2025-10-03 | 0.6.1   | Refined stories for Epic 1 based on elicitation.            | John, PM |
| 2025-10-03 | 0.6.2   | Added more specific Acceptance Criteria to Epic 1 stories.  | John, PM |
| 2025-10-03 | 0.7.0   | Added stories for Epic 2.                                   | John, PM |
| 2025-10-04 | 0.7.1   | Refined stories for Epic 2 based on elicitation.            | John, PM |
| 2025-10-04 | 0.7.2   | Broke down Story 2.4 into smaller, more granular stories.   | John, PM |
| 2025-10-04 | 0.8.0   | Added stories for Epic 3.                                   | John, PM |
| 2025-10-04 | 0.8.1   | Refined stories for Epic 3 to be more granular.             | John, PM |
| 2025-10-04 | 0.9.0   | Added stories for Epic 4.                                   | John, PM |
| 2025-10-04 | 0.9.1   | Refined stories for Epic 4 based on elicitation feedback.   | John, PM |
| 2025-10-04 | 0.10.0  | Added and refined stories for Epic 5.                       | John, PM |
| 2025-10-04 | 0.11.0  | Added stories for test infrastructure and renumbered epics. | John, PM |
| 2025-10-18 | 0.12.0  | Added Project Brief cross-references for strategic context. | John, PM |
