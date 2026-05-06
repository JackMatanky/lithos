---
stepsCompleted:
   - step-01-init
   - step-02-discovery
   - step-03-success
   - step-04-journeys
   - step-06-innovation
   - step-07-project-type
   - step-08-scoping
   - step-09-functional
   - step-10-nonfunctional
   - step-11-complete
lastStep: 11
---

# Product Requirements Document - lithos

**Author:** Jack
**Date:** 2026-01-05

## Executive Summary

Lithos is a command-line interface (CLI) tool built in Rust that provides powerful, scriptable template generation for Obsidian vaults. It empowers developers and knowledge workers to automate note creation and management directly from the terminal, solving the workflow friction of context-switching between Obsidian's plugin ecosystem and external environments.

### What Makes This Special

**Current Capabilities:** App-agnostic templating with interactive prompts and suggesters accessible from any terminal; portable metadata schemas enforcing required fields and enums outside Obsidian; automatic frontmatter generation and validation; vault-wide alias resolution for consistent links; shareable template packs via Git repositories.

**Technical Foundation:** Built in Rust for memory safety and high performance, with elegant, modular code following SRP/DRY principles. Prioritizes developer experience through clean architecture and user experience through parsimonious setup that even less technical users can master.

**Future Evolution:** Will expand into a comprehensive developer tooling ecosystem including LSP-like language support (inspired by markdown-oxide), linters/formatters (like obsidian-linter), and Neovim plugins, while exploring optimal indexing/caching/storage strategies leveraging Rust's unique capabilities.

## Project Classification

**Technical Type:** developer_tool (evolving toward productivity_tool platform)
**Domain:** general
**Complexity:** medium (phased ecosystem delivery)
**Project Context:** Greenfield project with future tooling ecosystem expansion

Signals from product brief and discussion: CLI tool, command-line interface, terminal operations, scripting support, LSP provider, IDE integration, linter, formatter, plugin development, language support for Markdown, Rust for safety/performance, elegant code (SRP/DRY/modular), parsimonious UX for setup, developer/user experience focus, indexing/caching exploration (evaluating RocksDB, SQLite, custom implementations).

This classification captures the MVP as a developer tool while accounting for the evolving vision toward a productivity platform. Cross-platform support is deprioritized for MVP in favor of personal project focus; indexing strategies will be thoroughly evaluated for optimal Rust performance.

## Functional Requirements

### Template Management
- FR1: Users can create modular templates composed of reusable sections with variables
- FR2: Users can execute templates interactively with prompts, suggesters, and multi-suggesters
- FR3: Users can compose complex templates from multiple sections with error prevention
- FR4: Users can apply date formatting and manipulation functions to template content
- FR5: Users can include dynamic commands and whitespace control in templates
- FR6: Users can define and use custom user functions within templates
- FR7: Users can execute advanced template operations with hooks and complex commands

### Schema Management
- FR8: Users can define metadata schemas with field types (string, number, date, file, boolean)
- FR9: Users can create schema-driven templates where field properties provide input parameters
- FR10: Users can validate notes against schemas with clear error feedback
- FR11: Users can use schema enums to populate suggester options in templates
- FR12: Users can filter file selections using schema-defined directory constraints
- FR13: Users can format dates using schema-defined format strings
- FR14: Users can inherit and extend schema definitions between related types

### Interactive Input
- FR15: Users can provide free-text input through template prompts
- FR16: Users can select from single-choice lists using suggesters
- FR17: Users can select multiple items from lists using multi-suggesters
- FR18: Users can receive contextual help and guidance during input
- FR19: Users can access progressive complexity modes for different expertise levels

### Vault Operations
- FR20: Users can index and search notes across entire vaults
- FR21: Users can perform lookups by filename, path, or schema-defined keys
- FR22: Users can resolve wiki-style links and aliases throughout vaults
- FR23: Users can query metadata fields from other notes for template use
- FR24: Users can maintain vault consistency across template operations
- FR25: Users can handle large vaults (1000+ files) without performance degradation

### Configuration Management
- FR26: Users can configure template packs using TOML files
- FR27: Users can manage schema definitions through configuration files
- FR28: Users can set application preferences via configuration
- FR29: Users can define custom validation rules and linting settings

### Cross-Environment Compatibility
- FR30: Users can execute templates consistently across operating systems
- FR31: Users can access templates through terminal interfaces
- FR32: Users can integrate with external editors and IDEs
- FR33: Users can run templates in automated scripts and CI/CD pipelines

### Community Features
- FR34: Users can share and distribute template packs via Git repositories
- FR35: Users can discover and adopt community-created template packs
- FR36: Users can validate third-party templates against schemas
- FR37: Users can contribute improvements to shared template ecosystems

### Security & Privacy
- FR38: Users can control access to sensitive vault data and templates
- FR39: Users can encrypt sensitive configuration and schema files
- FR40: Users can audit template execution and data access patterns

### Command Line Interface
- FR41: Users can execute lithos commands with subcommands for templates, schemas, and vaults
- FR42: Users can access comprehensive help and documentation from the CLI
- FR43: Users can view status and configuration of templates and schemas
- FR44: Users can manage vault operations (index, search, validate) from command line
- FR45: Users can run templates with various output formats and destinations
- FR46: Users can configure CLI behavior and preferences
- FR47: Users can execute most important commands with single words (e.g., `lithos new` opens fuzzy picker for template selection)

### Error Handling & Recovery
- FR48: Users can receive clear, actionable error messages when operations fail
- FR49: Users can recover from failed template executions with rollback capabilities
- FR50: Users can diagnose and troubleshoot configuration and schema issues

## Non-Functional Requirements

### Performance
- Template execution completes in under 500ms for individual operations
- Vault indexing completes in under 2 seconds for 1000+ files
- File I/O operations maintain efficient read/write performance for large vault scalability
- CLI commands provide instant feedback and help

### Security
- Sensitive configuration and schema files are encrypted at rest
- Users control access permissions for vault data and templates
- Template execution and data access are logged for auditing

### Scalability
- System handles vaults with thousands of files without performance degradation
- Memory usage remains bounded under 500MB for typical operations
- Multiple template executions run concurrently without interference

### Integration
- MVP supports macOS, with Linux added if implementation complexity is minimal
- CLI integrates reliably with terminal environments
- Future platform support (Windows, editors) added gradually

### Usability
- CLI provides clear help, auto-completion, and command discoverability
- Error messages are actionable and help users troubleshoot issues
- Progressive complexity modes accommodate different user expertise levels

### Maintainability
- Code maintains comprehensive test coverage and contributor documentation
- Binary distribution provides self-contained executables without external dependencies
- Safe rollback and version management support system updates

### Compatibility
- System gracefully handles Obsidian vault structure changes
- Migration paths support transition from existing template workflows

### Observability
- Comprehensive logging enables debugging of template execution and vault operations
- Performance metrics track system behavior for optimization
- Diagnostic tools help users identify and resolve issues

### Reliability
- System achieves 99.9% uptime for CLI operations
- Zero crashes during normal vault operations
- Failed operations provide clear recovery paths and state preservation

### Deployment
- Binary updates complete successfully in under 30 seconds with automatic rollback on failure
- Installation process succeeds for 95% of users without manual intervention
- Version compatibility maintained across patch releases

## User Journeys

**Journey 1: Alex Chen - Power User Regains Flow**
*Problem:* Alex is a senior software engineer who maintains a massive personal knowledge base in Obsidian, tracking project decisions, API designs, and debugging sessions. His workflow constantly breaks when creating structured notes from the terminal—either he loses momentum switching to the Obsidian UI, or externally created notes lack the rich metadata that makes his vault searchable. "I spend 20 minutes just to create one proper note," he grumbles, watching his focus evaporate.

*Discovery:* Frustrated, Alex discovers Lithos while searching for CLI knowledge tools. The installation takes 2 minutes, and the guided setup asks about his vault structure.

*Solution:* Alex creates his first template pack. Instead of the usual 20-minute context switch, he generates a complete project decision note in under 2 minutes: `lithos new project-decision --interactive`. The CLI shows real-time validation feedback, catching formatting issues before he commits. The built-in linting ensures his notes integrate perfectly with his existing vault.

*Impact:* Six months later, Alex has integrated Lithos into his daily workflow, saving 5+ hours weekly. "This is the seamless CLI-first knowledge management I've always wanted," he says. His vault now scales without friction, and he's shared his template packs with his development team.

**Journey 2: Sarah Martinez - Knowledge Enthusiast Builds Her System**
*Problem:* Sarah is a PhD researcher who uses Obsidian as her central hub for literature reviews, research notes, and project planning. As her vault grows to 2000+ interconnected notes, she struggles with consistency—some notes have perfect metadata, others are hastily created without templates. The separate Templater and Metadata Menu plugins create constant context switches, and large vaults crash 30% of the time during complex template processing. "I can't focus on my research when I'm worried about tool reliability," she admits.

*Discovery:* Sarah finds Lithos through academic communities. The "zero crashes in large vaults" claim intrigues her.

*Solution:* Lithos lets her create modular, schema-validated notes directly from research scripts. She builds custom template packs for literature reviews, with automatic metadata inheritance and vault-wide linking. The CLI handles her 2000+ note vault without crashes: `lithos index --fast && lithos new literature-review --template academic`.

*Impact:* Sarah's knowledge system now scales with her research. She shares template packs with fellow academics, turning her personal tool into a community resource. "My research productivity has increased 40% since eliminating crashes and context switches," she reports.

**Journey 3: Jordan Rivera - OSS Community Builder Shares Innovation**
*Problem:* Jordan leads an open-source community building starter vaults for project management methodologies. They spend hours creating template packs that work perfectly in Obsidian, but struggle with portability—users complain that complex schemas and multi-section templates don't work reliably outside the app. Git distribution feels incomplete without external validation. "Our community deserves reliable tools, not just pretty demos," Jordan says.

*Discovery:* Jordan discovers Lithos and sees its potential for community distribution. The free tier and GitHub integration appeal to their open-source ethos.

*Solution:* Lithos becomes their distribution platform, enabling template packs with guaranteed cross-environment compatibility. They build a comprehensive project management vault that works identically everywhere. Schema validation and modular templating make packs robust, and community contributions integrate easily. When a user reports an issue, Jordan can reproduce and fixes it entirely from the command line.

*Impact:* Their template ecosystem grows from personal project to widely adopted community standard. "Lithos increased our community engagement by 200% through reliable external tooling," Jordan shares. Next step: monetize premium template packs.

**Journey 4: Maya Patel - Template Pack Consumer Discovers Simplicity**
*Problem:* Maya is a content strategist who discovered Obsidian through online communities and wants to standardize her team's documentation process. She's not deeply technical but loves organized knowledge systems. When she finds community template packs, complex ones require Obsidian UI expertise she lacks, while simpler ones lack automation for consistent team outputs. "I want professional documentation without becoming a developer," she explains.

*Discovery:* Maya finds Lithos through community recommendations. The "guided prompts for non-technical users" and free trial appeal to her.

*Solution:* Lithos makes powerful template packs accessible through guided terminal prompts. She adopts Jordan's project management templates without learning advanced Obsidian features—just clear questions about project details, and perfect documentation emerges. Built-in validation ensures her team's notes always meet standards.

*Impact:* Maya becomes an advocate in her community, showing how sophisticated knowledge management can be approachable. "Our team documentation quality improved 60% with zero training required," she reports. Next step: expand to company-wide adoption.

**Journey 5: Carlos Mendoza - Enterprise IT Admin Ensures Compliance**
*Problem:* Carlos is an IT administrator at a mid-sized consulting firm implementing Obsidian for team knowledge management. He needs to ensure consistent, compliant documentation across 50+ users, but struggles with enforcing templates and metadata standards programmatically. Manual reviews are time-consuming, and plugin dependencies create support overhead. "We need enterprise-grade control without breaking user workflows," he states.

*Discovery:* Carlos evaluates Lithos for enterprise deployment. The audit logging and centralized configuration management catch his attention.

*Solution:* Lithos provides enterprise controls: centralized template packs, audit trails, and automated compliance validation. Carlos deploys it via their IT infrastructure, with single-sign-on integration. Teams use guided workflows while Carlos monitors adoption through detailed analytics.

*Impact:* The firm achieves 95% template compliance with 80% reduction in IT support tickets. "Lithos gave us the control we needed while maintaining user autonomy," Carlos says. Next step: integrate with their existing documentation platform.

### Journey Requirements Summary
These journeys reveal key capabilities: modular CLI templating with real-time validation, crash-resistant large vault processing (under 500ms for indexing 1000+ files, under 100ms for individual operations), guided non-technical onboarding, enterprise compliance controls, and community pack portability with Git-based distribution. Performance must scale efficiently as vaults balloon to thousands of files, with all actions remaining fast to maintain user flow.

## Project Scoping & Phased Development

### MVP Strategy & Philosophy

**MVP Approach:** Problem-Solving MVP with Platform MVP secondary focus - solve core template pain points (70% reduction in template coding through schema-driven inputs, zero crashes in large vaults) while building Rust foundation for ecosystem expansion.

**Resource Requirements:** Solo developer (you) with focus on maintainable, well-tested Rust codebase.

### MVP Feature Set (Phase 1)

**Core User Journeys Supported:**
- Alex (Power User): Modular template creation, CLI-first workflow, large vault performance
- Sarah (Knowledge Enthusiast): Schema-driven templates with function inputs, zero-crash operation, script integration
- Jordan (OSS Community): Cross-environment compatibility, schema validation
- Maya (Template Consumer): Guided setup, template pack access

**Must-Have Capabilities:**
- Schema-driven templates where schemas provide function inputs (enums for suggesters, directories for file filtering, date formats) to eliminate repetitive template coding
- Modular template composition and debugging
- CLI-first workflow with terminal/Neovim priority
- Large vault performance (under 500ms operations, zero crashes)
- Cross-environment compatibility and schema validation
- TOML-based configuration with Rust-native defaults
- Essential interactive functions: prompts, suggesters, multi-suggesters for template interactivity

### Post-MVP Features

**Phase 1.5 (Core Templater Parity + Basic UX):**
- Essential Templater functions (file, frontmatter basics - date functions use native Rust chrono)
- Dynamic commands and whitespace control
- User-defined functions foundation
- Basic task management schema support
- Beginner mode with guided template creation and progressive complexity disclosure
- Predefined regex identifiers for common patterns (basic email/link validation)

**Phase 2a (Advanced Templater Features + UX):**
- Full Templater module system (app, config, system, web)
- Complex execution commands and hooks
- Advanced user functions and scripting
- Selective Metadata-Menu integration (core field types, basic inheritance - not full plugin copy)
- Date tokens (moment.js style) for enhanced date handling
- Advanced beginner mode with contextual help and template suggestions

**Phase 2b (LSP & Tree-Sitter Foundation):**
- LSP implementation for comprehensive markdown intelligence (beyond templating: intelligent linking, go-to-definition, backlinks, rename refactoring, daily notes, tags, callouts, footnotes, block references, semantic tokens, code lens, unresolved link support - inspired by markdown-oxide capabilities)
- Tree-sitter grammar for advanced templating and markdown syntax
- Foundation for editor integrations and linter/formatter

**Phase 2c (Linter/Formatter Integration + UX):**
- Built-in markdown linter using LSP/tree-sitter foundation
- Automatic formatting on template output
- Custom linting rules for schemas
- Enhanced UX with smart error messages and auto-corrections

**Phase 3a (Neovim Integration):**
- Neovim plugin leveraging LSP/tree-sitter
- Native Neovim commands and completion
- Seamless integration with existing workflow

**Phase 3b (Extended Ecosystem):**
- VS Code extension
- Zed editor support (lower priority)
- Community plugin framework

**Phase 4 (Enterprise & Scale):**
- Multi-vault support
- Advanced audit logging
- Commercial features

### Risk Mitigation Strategy

**Technical Risks:** PKM domain complexity - mitigate through incremental Rust implementation, technical spikes for schema performance/validation, and automated testing. Use Serde for schema serialization, benchmark against similar CLI tools for performance baselines.

**Market Risks:** Developer adoption within PKM ecosystem - mitigate through clear Rust-first positioning, user interviews validating pain points, and market sizing (TAM: $XXM developer productivity tools, SAM: $XM CLI-first PKM solutions). Consider network effects of Obsidian ecosystem compatibility.

**Resource Risks:** Personal project scope - mitigate with granular phases, technical debt tracking, and community contribution potential for ecosystem phases.

**Phase Importance Order:** Core pain points first, essential interactive functions in MVP, then phased Templater parity using native Rust capabilities, LSP/tree-sitter foundation before editor plugins, Neovim prioritized over other editors, with UX simplifications dispersed by value/difficulty (basic in 1.5, advanced in 2a+).

## Innovation & Novel Patterns

### Detected Innovation Areas
**Systemic Integration:** Unifying disjointed structures from Templater, Metadata Menu, and Linter into a single CLI-first system, removing friction and enabling enhanced bidirectional linking through schema-structured data emergence.

**CLI-First PKM Challenge:** Challenging the assumption that PKM programs require GUI interfaces, proving they can work equally or better in IDEs/terminals with maintained performance and template usability.

### Market Context & Competitive Landscape
Developer-focused PKM market shows fragmentation between GUI tools (Obsidian, Notion) and CLI utilities. Most solutions offer single-aspect functionality with plugin communities creating integration overhead. Lithos differentiates through native integration, targeting developer productivity with expansion potential to broader knowledge workers.

### Validation Approach
- **Performance Metrics:** Under 500ms operations in 1000+ file vaults, zero crashes, maintained IDE responsiveness
- **Template Usability:** Pre/post metrics on template maintenance time, development complexity, and usage errors
- **MVP Testing:** Ship minimal integration to measure adoption and productivity uplift

### Risk Mitigation
**Risk Assessment Matrix:**
- Integration failure (High probability, High impact): Mitigate through modular architecture and proven tool foundations
- User switching costs (Medium probability, Medium impact): Validate through MVP testing and easy reversion
- Scalability limitations (Low probability, High impact): Address with performance benchmarks and architectural safeguards

If integration fails to deliver enhanced experience, revert to existing separate tool solutions with minimal switching friction.

## Success Criteria

### User Success
Users overcome Obsidian's template limitations by:
- Creating modular, debuggable templates in under 30 minutes vs. hours with monolithic approaches (95% user satisfaction rating)
- Experiencing zero crashes in large vaults (1000+ notes) during template processing
- Automatically validating and formatting output against metadata schemas at creation time (98%+ compliance)
- Setting up template packs without technical expertise, guided by clear terminal prompts (successful setup in <5 minutes for non-technical users)

### Business Success
- Personal productivity gains: Measurable time savings (e.g., 2+ hours/week in note creation and vault management)
- Project sustainability: Maintainable Rust codebase with projected maintenance costs under 20% of development time
- Long-term viability: Solves core workflow friction while remaining accessible; community adoption risk assessed (low initial risk, high potential upside)
- User validation: 95% of users report significant improvement in template creation experience

### Technical Success
- Rust performance delivers reliable operation in large vaults without crashes
- Elegant architecture enables easy feature additions (LSP, linter) without technical debt
- Cross-platform consistency as core requirement for broad compatibility
- Scalability benchmarks: Template processing under 2 seconds for 1000+ notes

### Measurable Outcomes
- Template creation time: <30 minutes for new templates via modular components
- Crash rate: 0% in large vault operations
- Schema compliance: 98%+ automatic validation and formatting
- Setup accessibility: Non-technical users successful in <5 minutes with guided prompts
- User satisfaction: 95% report easier template creation vs. Obsidian alternatives

## Product Scope

### MVP - Minimum Viable Product
Core CLI tool with modular templating, schema integration, built-in linting/formatting, and large vault stability—solving your personal Obsidian pain points with cross-platform consistency.

### Growth Features (Post-MVP)
LSP-like language support, advanced linter/formatter, Neovim plugin integration—driven by user feedback loops and journey validation.

### Vision (Future)
Comprehensive developer tooling platform for Markdown/knowledge management, with community adoption and co-maintainers if user validation proves demand.

## Developer Tool Specific Requirements

### Project-Type Overview
Lithos is a developer tool focused on CLI-first knowledge management, prioritizing Rust as the core language while supporting any file types found in Obsidian vaults. The tool emphasizes developer experience with comprehensive documentation and examples.

### User Personas
- **Power User Developer:** Wants deep customization, extensible APIs, and programmatic access to all features
- **New User:** Needs guided onboarding, clear defaults, and progressive complexity disclosure
- **Programmatic Consumer:** Requires clean APIs for automation, AI integration, and tool composition

### Technical Architecture Considerations
- **File Type Agnostic:** Universal file support as content objects with metadata, specialized processing (templating, formatting) for Markdown and well-defined formats
- **Primary Language:** Rust-first design with comprehensive language support
- **IDE Integration Strategy:** Terminal-first with Neovim priority, expanding to other editors
- **Markdown Focus:** Special attention to Markdown files as primary vault content and the only file type relevant to templates, schemas, and formatter/linter functionality

### Language Matrix
- **Primary:** Rust (core implementation, CLI, templating engine)
- **Supported Inputs:** Any file type in Obsidian vaults (universal handling with specialized processing for text formats)
- **Configuration/Schema:** TOML (primary for Rust-native config), JSON (secondary for web compatibility), YAML (tertiary for human readability)
- **Template Language:** Rust-based (primary, most powerful), with openness to alternatives for non-technical user accessibility
- **File Processing:** Universal file type support as content objects, specialized parsing for Markdown and structured formats

### Installation Methods
- **Primary:** Cargo (native Rust ecosystem integration for seamless developer workflow)
- **Future:** Homebrew and OS-specific package managers for broader accessibility
- **Distribution:** Single binary distribution for cross-platform compatibility

### API Surface
- **CLI Interface:** Comprehensive command structure with progressive help levels and auto-completion
- **Configuration API:** Programmatic access to TOML/JSON/YAML settings and schemas with runtime validation
- **Template API:** Extensible templating system with Rust-based engine and potential alternative syntaxes for accessibility
- **Vault API:** File system abstraction for vault operations with universal file support and specialized Markdown processing

### Code Examples
- **Starter Templates:** Basic to advanced template examples with runnable CLI commands
- **Schema Examples:** Metadata schema configurations with validation examples
- **Vault Examples:** Sample vault structures with real data and test scenarios
- **Integration Examples:** Terminal and Neovim usage patterns with concrete command sequences
- **Test Data:** Real Obsidian vault samples (as provided in docs/refs/obsidian/) for immediate testing

### Migration Guide
- **From Manual Obsidian:** Step-by-step migration with success checklists and measurable outcomes
- **Tool Integration:** Migrating from separate Templater/Metadata Menu workflows with rollback capabilities
- **Version Upgrades:** Clear upgrade paths for future Lithos versions with compatibility guarantees
- **Backup Strategies:** Safe migration with automated validation and easy reversion

### Implementation Considerations
Multi-audience design with progressive disclosure: power users get deep customization, new users get guided wizards, programmatic consumers get clean APIs. IDE integration prioritizes terminals and Neovim with extensible plugin architecture. Error messages are contextual (technical for developers, actionable for new users). Markdown receives special attention as the primary file type for templates, schemas, and formatting/linting operations.
