# Documentation Index

## Root Documents

### [Elicitation Summary](./elicitation_summary.md)

This document provides a comprehensive summary of the key findings, discussions, and decisions made during the advanced elicitation sessions for the Lithos project.

### [Project Brief](./project_brief.md)

This document outlines the project vision, problem statement, proposed solution, target users, goals, and scope for the Lithos CLI tool.

## Architecture

Documents related to the technical architecture and design of the Lithos project.

### [Change Log](./architecture/change-log.md)

A log of changes and versions for the architecture document.

### [Checklist Results Report](./architecture/checklist-results-report.md)

A report of the results from running the architecture checklist.

### [Coding Standards](./architecture/coding-standards.md)

Mandatory coding standards for Lithos contributors and AI agents.

### [Components](./architecture/components.md)

Identifies the major logical components and services that implement the system's functionality.

### [Core Workflows](./architecture/core-workflows.md)

Diagrams and notes on the core user workflows for template generation, discovery, and vault indexing.

### [Data Model Relationships Diagram](./architecture/data-model-relationships-diagram.md)

A diagram illustrating the relationships between the various data models in the Lithos project.

### [Data Models](./architecture/data-models.md)

Definitions of the data models used throughout the system, organized by architectural layer.

### [Database Schema](./architecture/database-schema.md)

Details on how Lithos persists structured metadata as JSON documents.

### [Error Handling Strategy](./architecture/error-handling-strategy.md)

The strategy for error handling, including explicit results, error types, and logging standards.

### [External APIs](./architecture/external-apis.md)

Information on external APIs that Lithos interacts with (none for MVP).

### [High Level Architecture](./architecture/high-level-architecture.md)

An overview of the hexagonal architecture, design patterns, and high-level project diagram.

### [Index](./architecture/index.md)

The main index file for the architecture documentation.

### [Infrastructure and Deployment](./architecture/infrastructure-and-deployment.md)

Details on the distribution strategy, infrastructure, environments, and deployment.

### [Next Steps](./architecture/next-steps.md)

Next steps for the architecture and project.

### [REST API Spec](./architecture/rest-api-spec.md)

Placeholder for a REST API specification for post-MVP.

### [Security](./architecture/security.md)

Security considerations for input validation, authentication, secrets management, and data protection.

### [Source Tree](./architecture/source-tree.md)

The layout of the source code repository.

### [Starter Template or Existing Project](./architecture/starter-template-or-existing-project.md)

Information on the starter template used for the project structure.

### [Tech Stack](./architecture/tech-stack.md)

The definitive technology stack for the Lithos project.

### [Testing Strategy](./architecture/testing-strategy.md)

The strategy for unit, integration, and end-to-end testing.

## PRD

Product Requirements Document for the Lithos project.

### [Epic 1: Foundational CLI & Static Template Engine](./prd/epic-1-foundational-cli-static-template-engine.md)

This epic establishes the project's backbone, delivering a runnable Go application with a basic command structure and core, non-interactive template rendering.

### [Epic 2: Hexagonal Architecture Realignment](./prd/epic-2-hexagonal-architecture-realignment.md)

This epic refactors the foundational CLI implementation from Epic 1 to follow hexagonal architecture patterns.

### [Epic 3: Configuration & Schema Loading](./prd/epic-3-configuration-schema-loading.md)

This epic introduces the ability for the CLI to read configuration files and understand the structure of the user's data through schema definitions.

### [Epic 4: Vault Indexing Engine](./prd/epic-4-vault-indexing-engine.md)

This epic focuses on building the core data layer of Lithos, which will scan the user's vault, parse frontmatter, and build a persistent cache.

### [Epic 5: Interactive Input Engine](./prd/epic-5-interactive-input-engine.md)

This epic brings the interactive user experience to life by implementing the fuzzy finder for template discovery and adding prompt and suggester functions.

### [Epic 6: Schema-Driven Lookups & Validation](./prd/epic-6-schema-driven-lookups-validation.md)

This epic connects the schema and indexing engines to the template engine, unlocking dynamic, data-driven note creation and validation.

### [Epic List](./prd/epic-list.md)

A list of all the epics for the Lithos project.

### [Goals and Background Context](./prd/goals-and-background-context.md)

The goals and background context for the Lithos project.

### [Index](./prd/index.md)

The main index file for the Product Requirements Document.

### [Next Steps](./prd/next-steps.md)

Next steps for the product.

### [Requirements](./prd/requirements.md)

Functional and non-functional requirements for the Lithos project.

### [Technical Assumptions](./prd/technical-assumptions.md)

Initial technical direction and library choices that will guide the architecture and implementation.

### [User Interface Design Goals](./prd/user-interface-design-goals.md)

The user interface design goals for the Lithos CLI.

## Stories

User stories for the Lithos project.

### [Story 1.1: Establish Test Vault and Golden Files](./stories/1.1.establish-test-vault-and-golden-files.md)

As a QA specialist, I want to create a foundational test data set, so that all unit and integration tests can use a consistent and predictable set of "golden" files.

### [Story 1.2: Foundational Project Setup](./stories/1.2.foundational-project-setup.md)

As a developer, I want to initialize a Go module with a .gitignore file, so that I have a clean, version-controlled foundation for the project.

### [Story 1.3: Implement a Runnable main.go and Basic CI Build](./stories/1.3.implement-runnable-main-and-ci.md)

As a developer, I want a main.go that prints a version number and a GitHub Action that builds it, so that I can verify the project compiles and have an automated build process.

### [Story 1.4: Integrate Cobra for Basic Command Structure](./stories/1.4.integrate-cobra-cli-framework.md)

As a developer, I want to integrate Cobra to create a root lithos command and a version subcommand, so that I have a robust CLI framework.

### [Story 1.5: Add Placeholder new Command](./stories/1.5.add-placeholder-new-command.md)

As a developer, I want to add a new command that accepts an optional <template-path> argument, so that the command's API is defined.

### [Story 1.6: Implement Core Template File Reading](./stories/1.6.implement-template-file-reading.md)

As a developer, I want the new command to read a template file into memory, so that the content is available for parsing.

### [Story 1.7: Implement Static Template Parsing](./stories/1.7.implement-static-template-parsing.md)

As a developer, I want to parse template content using Go's text/template engine, so that I can process a template without dynamic functions.

### [Story 1.8: Add Basic Date & String Functions to Template Engine](./stories/1.8.add-template-functions.md)

As a developer, I want to add a function map with now, toLower, and toUpper, so that templates can perform basic dynamic operations.

### [Story 1.9: Write Rendered Template to a New File](./stories/1.9.write-rendered-template-to-file.md)

As a user, I want the lithos new <template-path> command to create a new Markdown file, so that I can generate a complete note.

## References

This directory contains a large collection of reference materials, primarily related to Obsidian. Due to the large number of files, this directory is not indexed on a file-by-file basis.
