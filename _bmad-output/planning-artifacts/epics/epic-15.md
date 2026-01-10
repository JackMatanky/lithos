## Epic 15: User Documentation & Onboarding

Users have comprehensive documentation, starter templates, sample schemas, and migration guides that enable successful adoption.
**FRs covered:** NFR13 (clear help), NFR20 (migration paths), NFR28 (installation success)
**Implementation Notes:**
- Consolidates documentation from Epics 4-12
- Starter kit from converted docs/refs/obsidian/ samples (sanitized)
- Installation guide, quickstart, migration guides
- API documentation for power users
- Progressive complexity documentation (basic → advanced)
- Note: Documentation created at story-level in epics; this consolidates and polishes

### Story 15.1: [Docs] Installation and Setup Guide

As a new user, I want clear installation instructions and setup guidance, so that I can get lithos running quickly on my system.
**Acceptance Criteria:**
- **Given** the completed system
- **When** I create the installation guide
- **Then** it includes step-by-step instructions for macOS and Linux.
- **And** it covers prerequisites, binary installation, and initial configuration.
- **And** it achieves 95% successful installations based on user feedback.
**References:** NFR28

### Story 15.2: [Docs] Quick Start Tutorial

As a new user, I want a hands-on tutorial to create my first note with lithos, so that I can experience the core functionality immediately.
**Acceptance Criteria:**
- **Given** the completed system
- **When** I create the quick start guide
- **Then** it walks through creating a simple note template and executing it.
- **And** it introduces basic concepts (vaults, schemas, templates) through examples.
- **And** it takes <15 minutes to complete for first-time users.
**References:** NFR13

### Story 15.3: [Docs] Starter Template and Schema Library

As a new user, I want ready-to-use templates and schemas for common use cases, so that I can start productive work immediately.
**Acceptance Criteria:**
- **Given** the converted Obsidian templates
- **When** I create the starter kit
- **Then** it includes sanitized templates for daily notes, projects, contacts, and knowledge notes.
- **And** it provides sample schemas for common metadata patterns.
- **And** all starter content is documented with usage examples.
**References:** NFR20

### Story 15.4: [Docs] Migration Guide from Obsidian

As an existing Obsidian user, I want guidance on migrating my workflow to lithos, so that I can transition smoothly with minimal disruption.
**Acceptance Criteria:**
- **Given** the template conversion examples
- **When** I create the migration guide
- **Then** it maps Obsidian concepts to lithos equivalents (Templater → lithos templates).
- **And** it provides conversion examples for common template patterns.
- **And** it addresses compatibility considerations and limitations.
**References:** NFR20

### Story 15.5: [Docs] User Manual and Feature Reference

As a power user, I want comprehensive documentation of all features and configuration options, so that I can master advanced functionality.
**Acceptance Criteria:**
- **Given** all epic-level documentation
- **When** I consolidate the user manual
- **Then** it includes detailed sections for templates, schemas, vaults, and CLI.
- **And** it documents all configuration options and environment variables.
- **And** it provides troubleshooting guides for common issues.
**References:** NFR13

### Story 15.6: [Docs] API Documentation for Developers

As a developer extending lithos, I want API documentation for the plugin system and extension points, so that I can build custom integrations.
**Acceptance Criteria:**
- **Given** the system architecture
- **When** I create the API documentation
- **Then** it documents the hexagonal architecture ports and adapters.
- **And** it provides examples for creating custom template functions and suggesters.
- **And** it includes the Rust API reference for power users.
**References:** NFR13

### Story 15.7: [Docs] Progressive Complexity Documentation Structure

As a user at any skill level, I want documentation organized by complexity, so that I can learn at my own pace without being overwhelmed.
**Acceptance Criteria:**
- **Given** all documentation content
- **When** I organize it by complexity levels
- **Then** it provides clear learning paths: Beginner → Intermediate → Advanced.
- **And** each level builds on the previous without redundant explanations.
- **And** cross-references guide users to more detailed information when needed.
**References:** NFR15

### Story 15.8: [Test] Epic 15 Documentation Validation

As a documentation maintainer, I want validation that all documentation is accurate and complete, so that users receive reliable information.
**Acceptance Criteria:**
- **Given** the completed documentation
- **When** I validate it against the implementation
- **Then** all code examples are tested and functional.
- **And** all CLI commands in documentation work as described.
- **And** all configuration options are accurately documented.
**References:** NFR13
