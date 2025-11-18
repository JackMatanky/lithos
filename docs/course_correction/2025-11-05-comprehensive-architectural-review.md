# Comprehensive Architectural Review - November 5, 2025

**Status**: IN PROGRESS
**Trigger**: Story 3.2 implementation exposed systematic architectural issues
**Scope**: 18+ identified issues across 8 groups requiring resolution before Epic 3 completion

---

## Executive Summary

### Background

On November 2, 2025, a sprint change proposal pivoted Epic 3 (Vault Indexing Engine) from JSON file-per-note caching to a hybrid BoltDB + SQLite architecture to ensure production-ready performance at realistic vault scales (500+ notes). This architectural change introduced **6 critical architectural questions** requiring resolution:

1. **Component Orchestration Architecture**: How to structure orchestration without god objects?
2. **Singleton Pattern Implementation**: How to manage Config and PropertyBank lifecycle?
3. **FileClassKey Configuration Impact**: How does config-driven schema selection affect components?
4. **Data Transfer Object Architecture**: How to structure DTOs for storage-specific optimizations?
5. **SQLite Schema Optimization**: Schema-driven views vs column-based storage?
6. **Storage Write Coordination**: How to coordinate BoltDB + SQLite writes?

Questions 1-3 (foundation architecture) and Questions 4-5 (storage architecture) received decisions. Question 6 (write coordination) remained unresolved.

### Course Correction Trigger

During Story 3.2 (Multi-Storage Cache Adapters) implementation in November 2025, **systematic architectural issues** were discovered beyond the original 6 questions:

- **FrontmatterService.Extract()** performs file parsing (IO operations) in domain layer - hexagonal architecture violation
- **Anemic domain model anti-pattern** pervasive across entities (Frontmatter, Note, Template)
- **Validation layer confusion** - syntactic validation in domain instead of adapter
- **QueryService/Note struct mismatch** affecting storage integration
- **DTO architecture** not leveraging Go idioms (fs.FileInfo, File.Stat())

These discoveries revealed **fundamental misunderstanding** of hexagonal architecture boundaries and DDD rich model principles, requiring comprehensive architectural review before Epic 3 completion.

### Critical Architectural Principle Identified

**Hexagonal Architecture Validation Layers**:

- **Syntactic Validation** (structure/format checking) → **Adapter Layer**
- **Semantic Validation** (business rules checking) → **Domain Layer**

This principle fundamentally changes validation placement across the entire system.

### Current Scope

This document captures comprehensive course correction analysis using the BMad Change Navigation Checklist. **18+ architectural issues** have been identified and organized into **8 issue groups** for systematic analysis:

1. **Group 1**: Validation Architecture (anemic models, IO in domain, validation layers) - _Section 1 complete_
2. **Group 2**: Storage Architecture, CQRS & DTOs - _Section 1 complete_
3. **Group 3**: Orchestration & Coordination - _Section 1 complete_
4. **Group 4**: Configuration Management - _Section 1 complete_
5. **Group 5**: Schema Domain System - _Section 1 complete_
6. **Group 6**: Template System (Epic 5 dependency) - _Section 1 complete_
7. **Group 7**: Documentation & Patterns - _Section 1 complete_
8. **Group 8**: Implementation Blockers - _Section 1 complete_

**Current Status**: All Groups Section 1 (Understand Trigger & Context) complete. Proceeding to Research Phase (Go stdlib + Obsidian patterns).

**Expected Outcome**: Comprehensive story plan with sequencing, dependencies, and risk mitigation for completing Epic 3 with correct architectural foundation.

---

## Document Control

- **Version**: 1.14
- **Date**: November 8, 2025
- **Status**: ✅ COMPLETE - Course Correction Plan Finalized, ready for stakeholder review and Sprint 1 kickoff
- **Distribution**: Development team, stakeholders

### Change Log

| Date       | Version | Description                                                                                                                                                                                                                                                                                                                                                                                                                                                  | Author     |
| ---------- | ------- | ------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------ | ---------- |
| 2025-11-08 | 1.14    | **✅ SYNTHESIS PHASE COMPLETE**: Finalized comprehensive course correction plan with cross-issue dependency map, consolidated findings, complete Epic 3 renumbering (31 stories), 5-sprint timeline with milestones (~Jan 17, 2026 completion), architecture documentation roadmap, success criteria, and risk mitigation. Course correction analysis complete - ready for execution.                                                                        | Sarah (PO) |
| 2025-11-08 | 1.13    | **CORRECTED Epic Impact Assessment**: Replaced with properly incorporated version referencing all Actionable Insights (AI-1.1 through AI-3.1), Gap Analysis findings, and Entity Review recommendations. 14 new stories (45 points, 4-5 sprints) with dependency-ordered groups: GROUP A Foundation (3.17-3.18), GROUP B Storage (3.19-3.22), GROUP C Services (3.23-3.26), GROUP D Config (3.27-3.29), COMPLETION (3.30-3.31). Final Epic 3: 18→31 stories. | Sarah (PO) |
| 2025-11-08 | 1.12    | Completed Entity Review (8 entities including VaultFile DTO redesign) and comprehensive Epic Impact Assessment for all 8 groups: identified 12 new stories totaling 35 points across 4-5 sprints, with detailed acceptance criteria, dependencies, risks, and final Epic 3 renumbering from 18→30 stories                                                                                                                                                    | Sarah (PO) |
| 2025-11-08 | 1.11    | Corrected Gap 4.1: Acknowledged existing CQRS ports (CacheReaderPort/CacheWriterPort, VaultReaderPort/VaultWriterPort/VaultScannerPort), clarified gap is missing indexed metadata query layer (MetadataQueryPort) for O(1) lookups vs O(n) scanning                                                                                                                                                                                                         | Sarah (PO) |
| 2025-11-08 | 1.10    | Completed Comprehensive Gap Analysis: Identified 12 specific architectural gaps across 7 categories (File Metadata, Caching, Parsing, Storage, Templates, Generics, Paths) with current state, desired state, patterns, benefits, trade-offs, priorities, and recommendations for each                                                                                                                                                                       | Sarah (PO) |
| 2025-11-08 | 1.9     | Completed Research Phase 2 (Obsidian API): Documented TFile/TAbstractFile abstractions, MetadataCache pattern, Vault API operations, FileStats interface, atomic frontmatter updates, and comprehensive comparison to Lithos architecture with 5 potential improvements identified                                                                                                                                                                           | Sarah (PO) |
| 2025-11-08 | 1.8     | Corrected Research Phase 1 for project relevancy: Fixed io/fs VaultFile examples (removed incorrect frontmatter/schema assumptions), updated bbolt/sqlite schemas to handle mixed file types with nullable frontmatter, added fs.FileInfo simplification pattern, added Sys() extension pattern for notes                                                                                                                                                    | Sarah (PO) |
| 2025-11-07 | 1.7     | Completed Research Phase 1: Documented comprehensive findings for io/fs, path/filepath, text/template, bbolt, modernc.org/sqlite, goldmark, and Go generics with patterns, best practices, options, and tradeoffs for all architectural decisions                                                                                                                                                                                                            | Sarah (PO) |
| 2025-11-06 | 1.6     | Completed Section 1 (Understand Trigger & Context) for all 8 groups: comprehensive analysis of 18+ architectural issues with critical evaluation, code evidence, and impact assessment; ready for Research Phase (Go stdlib + Obsidian patterns)                                                                                                                                                                                                             | Sarah (PO) |
| 2025-11-06 | 1.5     | Restructured Structured Plan to phase-based approach (Section 1 all groups → Research → Entity Review → Synthesis → Epic Impact); moved Action Items under Structured Plan; added Epic Impact Assessment placeholder section                                                                                                                                                                                                                                 | Sarah (PO) |
| 2025-11-06 | 1.4     | Enhanced Executive Summary with full background (Nov 2 sprint change, 6 architectural questions, course correction trigger); replaced Action Items with detailed, specific breakdown for all 8 groups + research/synthesis phases                                                                                                                                                                                                                            | Sarah (PO) |
| 2025-11-06 | 1.3     | Reorganized document structure: moved analysis results under corresponding groups in Structured Analysis Plan; added progress checkboxes to each group; removed duplicate sections; reduced file from 980 to 741 lines                                                                                                                                                                                                                                       | Sarah (PO) |
| 2025-11-06 | 1.2     | Completed Group 1 Section 1 comprehensive analysis (Issues D1, B2, Hexagonal Principle) with code evidence from FrontmatterService, VaultReaderAdapter, and domain entities; ready for Section 2 Epic Impact Assessment                                                                                                                                                                                                                                      | Sarah (PO) |
| 2025-11-05 | 1.1     | Established structured analysis plan (8 issue groups); revised Group 2 to include missing storage/CQRS issues; moved SQLite to storage group; increased issue count to 18+                                                                                                                                                                                                                                                                                   | Sarah (PO) |
| 2025-11-05 | 1.0     | Initial comprehensive issue inventory (15 issues); established hexagonal validation principle; completed Section 1 for Issue D1                                                                                                                                                                                                                                                                                                                              | Sarah (PO) |

### Conversation Log

#### Initial Issue Identification

User identified three critical issues during Story 3.2 implementation:

1. QueryService/Note struct mismatch
2. IO in domain layer (FrontmatterService.Extract)
3. Schema loading/registration coupling

#### Comprehensive Inventory Development

- Initial inventory: 12 issues
- User identified missing considerations:
  - Event-driven architecture option (Issue A1)
  - DTO redesign with Go idioms + Obsidian patterns (Issue A4)
  - Unit of Work pattern (Issue A6)
  - Anemic model anti-pattern (Issue D1)
- Revised inventory: 15 issues

#### Critical Architectural Principle Discovery

User clarified hexagonal architecture validation principle:

- **Syntactic validation → Adapter layer**
- **Semantic validation → Domain layer**

This fundamentally changes validation placement across the system.

#### Section 2 Process Error

User correctly identified that Section 2 analysis only covered Issue D1, not all 15 issues.
Analysis must be comprehensive across all issues before proceeding.

---

## Comprehensive Issue Inventory

### Category A: Architectural Questions (6 issues)

#### Issue A1: Component Orchestration Architecture ✅ DECIDED

- **Status**: DECIDED - Event-Driven Architecture for Epic 3
- **Decision**: Implement event-driven architecture with domain events to solve god-object problem and enable clean CQRS separation
- **Domain Events**: NoteIndexed, VaultIndexingComplete, FrontmatterValidated, SchemaLoaded, SchemasReloaded
- **Implementation**: Epic 3 Story 3.29 (Event Infrastructure) + Service refactoring to publish/subscribe events

#### Issue A2: Singleton Pattern Implementation ✅ DECISION FINALIZED

- **Status**: Proper singleton for Config and PropertyBank using sync.Once
- **Implementation Pending**:
  - Package-level variables with sync.Once
  - GetConfig()/GetPropertyBank() accessors
  - Test harness support methods
  - Documentation updates

#### Issue A3: FileClassKey Configuration Impact ✅ DECISION FINALIZED

- **Status**: Config-driven schema selection
- **CRITICAL MISSING**: ViperAdapter not loading FileClassKey from config file/env vars
- **Implementation Pending**:
  - internal/domain/frontmatter.go updates
  - internal/adapters/spi/config/viper.go updates (CRITICAL)
  - Test coverage for all config variants

#### Issue A4: Data Transfer Object Architecture ❌ UNRESOLVED

- **Status**: Needs fundamental redesign
- **Problems**:
  - FileMetadata/VaultFile don't leverage Go's fs.FileInfo/File.Stat()
  - Not following Go idioms for file handling
  - Need to learn from Obsidian API patterns (TAbstractFile, FileStats, CachedMetadata)
- **Questions**:
  - How to leverage Go's fs.FileInfo instead of duplicating?
  - What Obsidian patterns should we adopt?
  - How should DTOs differ per storage system?

#### Issue A5: SQLite Schema Optimization ✅ DECISION FINALIZED

- **Status**: Schema-driven views over JSON storage
- **Implementation Pending**:
  - Simplified base table
  - generateSchemaViews() function
  - Auto-generation from loaded schemas
  - Query helpers for views vs raw JSON

#### Issue A6: Storage Write Coordination Design ❌ UNRESOLVED

- **Status**: No decision yet
- **Missing Consideration**: Unit of Work pattern
- **Questions**:
  - Should we implement UoW for transactional consistency across BoltDB + SQLite?
  - How does UoW handle dual-write problem (vault + cache)?
  - What are rollback semantics?
  - Should we use sagas for distributed transaction coordination?

---

### Category B: New Critical Issues (3 issues)

#### Issue B1: QueryService/Note Struct Mismatch ❌ CRITICAL

- **Problem**: QueryService works with Note struct but operates on caches with richer metadata
- **Impact**: Breaking tests, incorrect data model alignment
- **Related**: Issue A4 (DTO Architecture)
- **Status**: Requires architectural review of QueryService data contracts

#### Issue B2: IO in Domain Layer Violation ❌ CRITICAL

- **Problem 1**: FrontmatterService.Extract() performs file parsing (IO) in domain layer
- **Problem 2**: Validation in wrong layers per hexagonal architecture
- **Hexagonal Architecture Principle**:
  - **Syntactic Validation** (YAML/JSON structure) → Adapter Layer
  - **Semantic Validation** (business rules) → Domain Layer
- **Correct Approach**:
  - Extract frontmatter in adapter layer (internal/adapters/spi/vault/frontmatter.go)
  - Syntactic validation in adapter during extraction
  - Semantic validation (schema compliance) in domain service
- **Status**: Requires comprehensive refactoring

#### Issue B3: Schema Loading/Registration Coupling ⚠️ MODERATE

- **Problem**: Unnecessary complexity with separate SchemaLoaderPort and SchemaRegistryPort
- **Proposal**:
  - SchemaLoader automatically registers on load
  - SchemaRegistry tries loading if GetSchema fails
  - Remove SchemaLoaderPort, keep only SchemaRegistryPort
- **Status**: Simplification opportunity

---

### Category D: Fundamental Architectural Patterns (3 issues)

#### Issue D1: Anemic Domain Model Anti-Pattern ❌ CRITICAL - PERVASIVE

- **Problem**: Entities are data bags, all logic in services
- **Affected Entities**:
  - Frontmatter - no validation, all logic in FrontmatterService
  - Note - no behavior, just ID + Frontmatter
  - Template - no behavior, just ID + Content
  - Property - minimal behavior
- **Good Examples**:
  - Schema - has Validate() method (rich model)
  - PropertySpec variants - have Type() and Validate()
- **Principle**: Business logic pertaining to entity's own data → belongs on entity
- **Status**: Requires systematic entity-by-entity refactoring

**Validation Naming Ambiguity Sub-Issue**:

Three types of validation, same method name:

| Type      | Example                                    | Validates      | Data Required        | Correct Layer                   |
| --------- | ------------------------------------------ | -------------- | -------------------- | ------------------------------- |
| Syntactic | Schema.Validate()                          | JSON structure | Schema only          | Adapter (schema loader)         |
| Syntactic | Frontmatter.Validate()                     | YAML structure | Frontmatter only     | Adapter (frontmatter extractor) |
| Semantic  | FrontmatterService.ValidateAgainstSchema() | Business rules | Frontmatter + Schema | Domain (service)                |

**CRITICAL REALIZATION**: Current Schema.Validate() in domain layer should move to adapter!

#### Issue D2: DTO Architecture Mismatch with Go Idioms ❌ CRITICAL

- **Problem**: FileMetadata/VaultFile don't leverage Go stdlib abstractions
- **Go Idioms Violated**:
  - Not using fs.FileInfo interface
  - Not using fs.File interface
  - Duplicating filesystem information
  - Not following io.FS for testability
- **Obsidian Patterns to Learn**:
  - TAbstractFile - abstract base for files/folders
  - FileStats - size, ctime, mtime
  - CachedMetadata - indexed metadata separate from file stats
- **Status**: Requires DTO redesign based on Go idioms + Obsidian patterns

#### Issue D3: Missing Pattern Documentation ⚠️ MODERATE

- **Problem**: Architecture docs don't specify when to use specific patterns
- **Missing Guidance**:
  - Event-driven vs orchestrator patterns
  - Unit of Work vs dual-write patterns
  - Factory pattern with validation vs simple constructors
  - Rich vs anemic model guidelines
  - Go's fs.FileInfo vs custom DTOs
- **Status**: Needs architecture documentation updates

---

### Category C: Implementation Blockers (3 meta-issues)

#### Issue C1: Multiple Questions Pending Implementation

- Questions 1-5 have decisions but no implementation
- Cannot proceed with Story 3.6+ until architecture corrected
- Risk: Continuing on flawed foundation compounds debt

#### Issue C2: Question 6 Unresolved

- No decision on write coordination pattern
- BoltDB+SQLite integration incomplete
- Story 3.2 technically incomplete, Story 3.6 blocked

#### Issue C3: Documentation Misalignment

- Architecture docs don't reflect:
  - CLIComander orchestration pattern
  - Singleton Config/PropertyBank
  - DTO architecture decisions
  - Schema-driven SQLite views
  - Hexagonal validation layers (syntactic vs semantic)
  - FrontmatterService refactoring
  - QueryService data contracts
- Impact: Developers implementing from docs build incorrect architecture

---

### Summary Metrics

**Total Issues**: 15

- Category A (Architectural Questions): 6
  - Finalized: 3 (Questions 2, 3, 5)
  - Unresolved: 3 (Questions 1, 4, 6)
- Category B (New Critical): 3
- Category C (Blockers): 3
- Category D (Fundamental Patterns): 3

**Critical Path Issues** (must resolve before proceeding):

1. Issue D1 (Anemic Models) - PERVASIVE - affects all entities
2. Issue D2 (DTO Redesign) - FOUNDATIONAL - affects all storage
3. Issue A1 (Event-driven vs Orchestrator) - system-wide coordination
4. Issue A6 (Unit of Work) - write coordination
5. Issue B2 (Validation Layers) - hexagonal architecture violation

---

## Structured Plan

**Approach**: Option A - Full Sequential Analysis

- Understand Trigger & Context:
  - for each issue group systematically work through the trigger and context
  - document findings in this file after each group
- Research Strategy:
  - identify potential solutions built-in to Go packages
  - gain insights from Obsidian architecture
  - document findings in this file after each package
- Complete full entity review scope
- Epic Impact Assessment:
  - Review each issue group to stories and epics directly impacted by the issue
  - Synthesize all findings into comprehensive story/epic plan

### Action Items

#### Understanding Trigger & Context

- [x] Group 1: Validation Architecture:
  - [x] Analyzed anemic models, IO in domain, validation layer violations
- [x] Group 2: Storage Architecture, CQRS & DTOs
  - [x] Analyze QueryService/Note struct mismatch (Issue B1)
  - [x] Review DTO architecture violations of Go idioms (Issues D2, A4)
  - [x] Examine SQLite schema optimization approach (Issue A5)
  - [x] Investigate write coordination patterns (Issue A6)
  - [x] Assess CQRS pattern application (read/write models vs operations)
  - [x] Evaluate cache vs vault source of truth implications
- [x] **Group 3: Orchestration & Coordination**
  - [x] Evaluate event-driven architecture vs orchestrator pattern (Issue A1)
  - [x] Analyze write coordination pattern overlap with storage (Issue A6)
  - [x] Examine god-object concerns with CLICommander
  - [x] Review domain events approach (NoteIndexed, FrontmatterValidated, SchemaLoaded)
- [x] **Group 4: Configuration Management**
  - [x] Review singleton implementation for Config and PropertyBank (Issue A2)
  - [x] Analyze FileClassKey configuration impact (Issue A3)
  - [x] Examine ViperAdapter FileClassKey loading gap
- [x] **Group 5: Schema Domain System**
  - [x] Analyze SchemaLoaderPort and SchemaRegistryPort coupling (Issue B3)
  - [x] Review automatic registration vs explicit loading
- [ ] **Group 6: Template System (CRITICAL - Epic 5 Dependency)**
  - [x] Investigate Template struct name conflict with text/template package
  - [x] Research text/template stdlib capabilities
  - [x] Determine if Template struct is even needed
  - [x] Analyze whether to embed \*template.Template
- [x] **Group 7: Documentation & Patterns (META)**
  - [x] Catalog pattern documentation gaps (Issue D3)
  - [x] Review architectural documentation misalignment
- [x] **Group 8: Implementation Blockers (META)**
  - [x] Review Questions 1-5 pending implementations (Issue C1)
  - [x] Analyze Question 6 unresolved status (Issue C2)
  - [x] Document architecture documentation misalignment (Issue C3)

### Research Phase (Parallel with Analysis)

**Phase 1: Go Native Capabilities** (Priority - understand before Obsidian)

- [ ] Research io/fs package (FileInfo, File, FS interfaces, WalkDir patterns)
- [ ] Research text/template package (composition, function maps, execution patterns)
- [ ] Research bbolt package (bucket design, transactions, cursor usage, best practices)
- [ ] Research modernc.org/sqlite (schema patterns, query optimization, Go idioms)
- [ ] Research goldmark package (parser API, AST manipulation, extension patterns, frontmatter extraction)
- [ ] Research Go Generics

**Phase 2: Obsidian API Patterns** (After Phase 1)

- [ ] Survey Obsidian API index for relevant models (TAbstractFile, FileStats, CachedMetadata)
- [ ] Map Obsidian patterns to Go stdlib capabilities
- [ ] Identify gaps between Go native and Obsidian solutions
- [ ] Extract architectural patterns applicable to Lithos domain

**Phase 3: Gap Analysis**

- [ ] Compare Go idioms vs current implementation
- [ ] Identify Obsidian patterns worth adopting
- [ ] Document pattern recommendations with rationale

#### Epic Impact Assessment

- [x] Group 1: Validation Architecture
  - [x] Identify which Epic 3 stories require validation refactoring
  - [x] Determine story breakdown: Frontmatter entity refactoring, Note entity refactoring, validation layer separation
  - [x] Assess FrontmatterService.Extract() extraction to adapter layer
  - [x] Evaluate Template entity impact (Epic 5 dependency)
  - [x] Document refactoring sequence and dependencies

- [x] Group 2: Storage Architecture, CQRS & DTOs
  - [x] Determine FileMetadata/VaultFile redesign leveraging fs.FileInfo
  - [x] Design BoltDB vs SQLite query routing strategy
  - [x] Select write coordination pattern (UoW, Saga, or dual-write)
  - [x] Plan storage staleness detection implementation

- [x] Group 3: Orchestration & Coordination
  - [x] Select orchestration pattern and document rationale
  - [x] Plan CLICommander refactoring if needed
  - [x] Design event infrastructure if event-driven approach selected

- [x] Group 4: Configuration Management
  - [x] Plan singleton accessor implementation (GetConfig, GetPropertyBank)
  - [x] Design test harness support for instance swapping
  - [x] Document Config embedded struct pattern for extensibility

- [x] Group 5: Schema Domain System
  - [x] Determine port simplification approach
  - [x] Plan schema loading workflow refactoring

- [x] Group 6: Template System (CRITICAL - Epic 5 Dependency)
  - [x] Make Template struct fundamental decision
  - [x] Assess Epic 5 (Template Engine) impact and timeline
  - [x] Plan Template entity refactoring if keeping struct

- [x] Group 7: Documentation & Patterns (META)
  - [x] Synthesize patterns discovered across Groups 1-6
  - [x] Plan architecture documentation updates (components.md, data-models.md)
  - [x] Create pattern decision matrix

- [x] Group 8: Implementation Blockers (META)
  - [x] Assess implementation roadmap across all groups
  - [x] Determine story renumbering strategy (push 3.17-3.18 after new stories)
  - [x] Plan documentation updates timing

### Synthesis Phase (After All Groups Complete)

- [x] Create cross-issue dependency map
- [x] Consolidate epic impact findings from all 8 groups
- [x] Develop comprehensive story plan:
  - [x] Story breakdown with acceptance criteria
  - [x] Story sequencing based on dependencies
  - [x] Effort estimates per story
  - [x] Risk assessment and mitigation strategies
- [x] Renumber Epic 3 stories (insert new stories, push 3.17-3.18 to end)
- [x] Update Epic 3 timeline and milestones
- [x] Final architecture documentation updates

---

## Trigger and Context Analysis

### Group 1: Validation Architecture (3 issues - FOUNDATIONAL)

**Issues**:

- **D1**: Anemic Domain Model + Validation Naming Ambiguity
- **B2**: IO in Domain Layer (FrontmatterService.Extract)
- **Hexagonal Principle**: Syntactic (adapter) vs Semantic (domain) validation

**Why Grouped**: All about where validation logic belongs in hexagonal architecture

#### Section 1: Understand Trigger & Context

##### 1.1 What triggered this change?

**Immediate Trigger**: Story 3.2 implementation revealed FrontmatterService.Extract() performs file parsing (IO operations) in domain layer.

**Broader Discovery**: During architectural review, identified pervasive anemic domain model anti-pattern across all entities (Frontmatter, Note, Template, Property) and inconsistent validation placement.

**User Observation**: Direct identification that entities are "just data bags" with all logic in services, violating DDD rich domain model principles.

##### 1.2 What is the core issue?

**Three Interconnected Problems**:

1. **Anemic Domain Model** (Issue D1):
   - Entities are pure data structures with no behavior
   - All business logic centralized in services
   - Frontmatter has no validation, factory, or behavior methods
   - Note is just ID + Frontmatter (no behavior)
   - Template is just ID + Content (no behavior)
   - Only Schema and PropertySpec variants are rich models (inconsistency)

2. **IO in Domain Layer** (Issue B2):
   - FrontmatterService.Extract() parses markdown using goldmark (infrastructure dependency in domain)
   - Domain layer coupled to goldmark parser library
   - Parsing is adapter responsibility, not domain responsibility

3. **Validation Layer Confusion** (Hexagonal Principle Violation):
   - Syntactic validation (YAML/JSON structure) happening in domain instead of adapter
   - Semantic validation (schema compliance) correctly in domain but poorly separated
   - Three validation types using same method name: Schema.Validate(), Frontmatter.Validate(), FrontmatterService.Validate()
   - No clear naming convention to distinguish validation types

**Root Cause**: Fundamental misunderstanding of hexagonal architecture boundaries and DDD rich model principles.

##### 1.3 Is this a misunderstanding, missing consideration, or new information?

**Misunderstanding**: Hexagonal architecture validation layer principle

- Current: All validation in domain layer
- Correct: **Syntactic validation → Adapter layer**, **Semantic validation → Domain layer**

**Missing Consideration**: DDD rich domain model principles

- Current: Entities as DTOs, logic in services
- Correct: Entities own logic pertaining to their own data

**Inconsistent Application**:

- Schema entity follows rich model (has Validate() method)
- Frontmatter entity follows anemic model (no methods)
- Leads to confusion about correct pattern

##### 1.4 What is the impact if we don't address this?

**Immediate Impacts**:

1. **Testing Difficulty**:
   - Domain layer tests require goldmark parser (infrastructure dependency)
   - Can't test Frontmatter validation without parser setup
   - Frontmatter has no self-validation (must always use service)

2. **Architecture Erosion**:
   - If FrontmatterService has IO, others will follow
   - Precedent for infrastructure dependencies in domain
   - Hexagonal architecture benefits lost

3. **Coupling & Inflexibility**:
   - Domain coupled to specific parser implementation (goldmark)
   - Can't swap parsers without changing domain layer
   - Can't reuse Frontmatter entity in non-filesystem contexts

4. **Validation Inconsistency**:
   - Three validation types, same method name (Schema.Validate, Frontmatter.Validate, FrontmatterService.Validate)
   - Developers confused about which validation to use when
   - Schema.Validate() in domain layer (should be in adapter per hexagonal principle)

5. **Code Duplication & Complexity**:
   - Logic that should be on entities scattered across services
   - Factory logic for Frontmatter mixed with service logic
   - Validation logic duplicated in tests (can't use entity methods)

**Long-term Strategic Impacts**:

1. **Scalability**: Anemic models don't scale as domain complexity grows
2. **Maintainability**: Logic scattered across services harder to maintain than cohesive entities
3. **Team Productivity**: Developers spend time searching for logic instead of finding it on entities
4. **Architecture Drift**: Each developer interprets patterns differently without clear entity behavior

##### 1.5 What evidence supports this change?

**Code Evidence**:

1. **FrontmatterService.Extract() - IO in Domain** (`internal/app/frontmatter/service.go`):

   ```go
   // Lines 13-15: Infrastructure dependency in domain
   import (
       "github.com/yuin/goldmark"
       "github.com/yuin/goldmark/parser"
       "go.abhg.dev/goldmark/frontmatter"
   )

   // Line 32: Infrastructure in domain struct
   type FrontmatterService struct {
       markdown goldmark.Markdown  // Parser belongs in adapter!
   }

   // Lines 87-102: Parsing in domain service
   func (s *FrontmatterService) Extract(content []byte) (domain.Frontmatter, error) {
       frontmatterData, err := s.parseMarkdownWithFrontmatter(content)
       // ... parsing logic using goldmark
   }
   ```

2. **VaultIndexer Uses FrontmatterService.Extract()** (`internal/app/vault/indexer.go:769`):

   ```go
   // Domain service parsing raw bytes (adapter responsibility)
   extractedFM, extractErr := v.frontmatterService.Extract(vf.Content)
   ```

3. **Anemic Frontmatter Entity** (`internal/domain/frontmatter.go`):

   ```go
   // Pure data structure, no behavior methods
   type Frontmatter struct {
       FileClass string
       Fields    map[string]interface{}
   }
   // No Validate(), no IsValid(), no factory methods
   ```

4. **Anemic Note Entity** (`internal/domain/note.go`):

   ```go
   type Note struct {
       ID          NoteID
       Frontmatter Frontmatter
   }
   // Just data bag, no behavior
   ```

5. **Rich Schema Entity** (`internal/domain/schema.go`) - **Inconsistency**:

   ```go
   type Schema struct {
       Name       string
       Properties []Property
   }

   func (s Schema) Validate() error {
       // Has behavior method! (But should be in adapter per hexagonal principle)
   }
   ```

**Documentation Evidence**:

1. `docs/architecture/components.md`:
   - Schema described as "Rich domain model with validation"
   - Frontmatter described as "Pure data structure with no behavior"
   - Inconsistent guidance about which pattern to use

2. Architecture docs don't specify:
   - When to use rich vs anemic models
   - Where validation belongs (adapter vs domain)
   - Factory pattern requirements

**Architectural Evidence**:

1. **Current Flow (INCORRECT)**:

   ```
   VaultReaderAdapter (adapter) → reads file → VaultFile with raw Content []byte
   ↓
   VaultIndexer → passes bytes to FrontmatterService
   ↓
   FrontmatterService.Extract() (domain) → parses using goldmark
   ↓
   FrontmatterService.Validate() (domain) → semantic validation
   ```

2. **Correct Flow (Hexagonal Architecture)**:

   ```
   VaultReaderAdapter (adapter) → reads file → parses frontmatter using goldmark → syntactic validation
   ↓
   VaultIndexer receives pre-parsed Frontmatter
   ↓
   FrontmatterService.IsSchemaCompliant() (domain) → semantic validation only
   ```

**Validation Naming Ambiguity**:

| Current Method                | Type      | Validates         | Data Required    | Current Layer | Correct Layer | Correct Method Name |
| ----------------------------- | --------- | ----------------- | ---------------- | ------------- | ------------- | ------------------- |
| Schema.Validate()             | Syntactic | JSON structure    | Schema only      | Domain        | **Adapter**   | IsValidSyntax()     |
| Frontmatter.Validate()        | Syntactic | YAML structure    | Frontmatter only | **Missing!**  | Adapter       | IsValidSyntax()     |
| FrontmatterService.Validate() | Semantic  | Schema compliance | FM + Schema      | Domain        | Domain        | IsSchemaCompliant() |

---

### Group 2: Storage Architecture, CQRS & DTOs (6+ issues - FOUNDATIONAL)

**Core Issues**:

- **D2**: DTO Architecture Mismatch with Go Idioms
- **A4**: Data Transfer Object Architecture (BoltDBMetadata, SQLiteMetadata, NoteMetadataDTO)
- **A5**: SQLite Schema Optimization (schema-driven views over JSON storage)
- **A6**: Storage Write Coordination (Unit of Work pattern for BoltDB+SQLite)
- **B1**: QueryService/Note Struct Mismatch

**Missing Storage/CQRS Issues** (from sprint-change-proposal-2025-11-02):

- **CQRS Pattern Application**:
  - Current: Just separated read/write methods (CacheWriterPort vs CacheReaderPort)
  - Question: Do we need separate read/write models (NoteProjection vs Note)?
  - True CQRS separates models, not just operations

- **Hybrid Storage Architecture Design**:
  - BoltDB (hot cache) vs SQLite (deep storage) - what belongs where?
  - Query routing strategy: ByPath → BoltDB, ByFrontmatter → SQLite
  - Performance requirements: BoltDB <1ms, SQLite <50ms

- **Cache vs Vault Source of Truth**:
  - Vault = source of truth (persistent markdown files)
  - Cache = projection (can be rebuilt from vault)
  - Dual-write pattern implications (vault + cache coordination)
  - Eventual consistency vs strong consistency

- **Storage Staleness Detection**:
  - file_mod_time vs index_time comparison
  - Incremental indexing strategy
  - BoltDB /staleness/ bucket vs SQLite staleness queries

**Why Grouped**: All about storage layer architecture, data persistence, query optimization, CQRS pattern

#### Section 1: Understand Trigger & Context

##### 1.1 What triggered this change?

**Primary Trigger**: November 2, 2025 sprint change proposal (docs/course_correction/sprint-change-proposal-2025-11-02-epic3-hybrid-storage-architecture.md) pivoted Epic 3 (Vault Indexing Engine) from JSON file-per-note caching to hybrid BoltDB + SQLite architecture to ensure production-ready performance at realistic vault scales (500+ notes).

**Performance Driver**: JSON file-per-note approach would not scale - O(n) file operations for cache warming/querying makes template queries too slow at production scale.

**Six Architectural Questions Introduced**: The hybrid storage pivot introduced fundamental questions requiring resolution:

1. **A1: Component Orchestration** - ❌ UNRESOLVED - Event-driven vs orchestrator patterns
2. **A2: Singleton Pattern** - ✅ DECIDED (sync.Once for Config/PropertyBank) - Implementation pending
3. **A3: FileClassKey Config** - ✅ DECIDED (config-driven schema selection) - Implementation pending
4. **A4: DTO Architecture** - ❌ UNRESOLVED - How to leverage fs.FileInfo, Obsidian patterns, storage-specific DTOs
5. **A5: SQLite Schema** - ✅ DECIDED (schema-driven views over JSON) - Implementation pending
6. **A6: Write Coordination** - ❌ UNRESOLVED - BoltDB + SQLite coordination pattern (UoW, Saga, dual-write)

**Secondary Discoveries During Implementation**:

- **Issue B1**: QueryService/Note struct mismatch - broader question of what domain entities should expose for querying
  - Note is domain entity (no IO concerns)
  - Queries need path/basename (filesystem concepts)
  - Fundamental domain modeling tension

- **CQRS Pattern Application**: Separated ports (CacheWriterPort/CacheReaderPort) vs separated models - is this true CQRS?

- **Cache Consistency Challenge**: Vault is source of truth, but QueryService must use BoltDB/SQLite for performance
  - How to ensure reindexing/updates don't bottleneck query performance?
  - What consistency guarantees between vault and caches?

##### 1.2 What is the core issue?

**Five Interconnected Problems**:

1. **QueryService Command/Query Responsibility Confusion** (Issue B1 + Event-Driven Need):
   - **Problem**: QueryService.RefreshFromCache() is a WRITE operation (command side) in a service named "Query"
   - **CQRS Violation**: Query services should only READ, not rebuild indices (that's command side)
   - **Broader Issue**: What should domain entities expose for querying? Note is pure domain (ID + Frontmatter), but queries need path/basename (filesystem concepts)
   - **Current Consequence**: RefreshIncremental loads ALL notes because Note lacks ModTime for filtering
   - **Event-Driven Implication**: IndexingComplete event → QueryService subscribes and rebuilds indices (separates concerns properly)

2. **DTO Architecture Not Focused/Optimal** (Issues A4, D2):
   - **Problem**: Current DTOs (FileMetadata, VaultFile) too generic - not focused per storage system
   - **Options**: Break down into smaller focused structs OR create completely storage-specific structs
   - **fs.FileInfo/filepath Underutilization**: Although fs.FileInfo is used, not leveraging full capabilities - reimplementing things packages already provide
   - **Research Goal**: Find where packages solve problems with premade solutions instead of custom code
   - **Storage-Specific Needs**:
     - BoltDB: Hot cache metadata (path, basename, aliases, file_class)
     - SQLite: Queryable metadata (all frontmatter fields as typed columns via views)
     - JSON: Full Note serialization for export/debugging - ⚠️ must NOT include note content when Note.Content added

3. **Storage Write Coordination Undefined** (Issue A6):
   - **Problem**: No coordination pattern for dual-write to BoltDB + SQLite
   - **Risk**: BoltDB write succeeds, SQLite write fails → data inconsistency
   - **Options Pending**: Unit of Work, Saga, dual-write with compensation, eventual consistency
   - **Impact**: QueryService merges data from both stores - inconsistency breaks queries

4. **CQRS Pattern Scope and NoteProjection Domain/IO Boundary** (CQRS Issues + NoteProjection):
   - **Current State**: Separated ports (CacheWriterPort vs CacheReaderPort) but unified domain.Note model
   - **Question 1**: Is this true CQRS, or just separated interfaces?
     - True CQRS: Separate read/write **models** (NoteProjection vs Note)
     - Current: Just separated **operations** with same unified model
   - **Question 2**: Do we need separate read/write models (NoteProjection vs Note)?
     - **History**: NoteProjection was dropped because it was identical to Note struct
     - **If reintroduced**: Where does NoteProjection live? Domain layer?
     - **IO Boundary Tension**: NoteProjection needs filesystem data (path, basename, ModTime) for queries
     - How does projection in domain handle IO concerns without violating hexagonal architecture?
   - **Question 3**: Should we maintain unified domain.Note or separate models?
     - Unified: Simpler, but limits read-side optimization
     - Separated: More complex, but enables query-specific data modeling
   - **Fundamental Complexity**: CQRS with filesystem projections creates domain/IO boundary tension that's hard to resolve cleanly

5. **SQLite Schema-Driven Views Implementation Gap** (Issue A5):
   - **Decision**: ✅ Schema-driven views extract JSON frontmatter into typed columns
   - **Design**: `v_contact_notes` view has columns: name, email, phone, company, status (extracted from frontmatter JSON)
   - **Major Benefit**: Filtering by fileclass directs query to correct view (not scanning full notes table)
   - **Without Views**: Must scan entire notes table filtering by fileclass, THEN extract JSON fields (slow)
   - **With Views**: `SELECT * FROM v_contact_notes WHERE status = 'active'` - pre-filtered by schema, typed columns
   - **Implementation**: ❌ View generation code doesn't exist yet

**Root Cause**: Hybrid storage architecture (BoltDB + SQLite) introduced without comprehensive design for:

- Domain entity boundaries (what belongs in Note vs DTOs vs cache metadata)
- Storage coordination patterns (how to keep BoltDB and SQLite consistent)
- Query data modeling (how domain entities relate to queryable data)
- CQRS pattern application (ports only vs models + ports)

##### 1.3 Is this a misunderstanding, missing consideration, or new information?

**Missing Consideration (Primary)**: Storage architecture design gaps

- **Problem 1 (QueryService command/query mixing)**: When designing QueryService, command/query separation wasn't fully considered - RefreshFromCache is a command operation in a query service
- **Problem 2 (DTO architecture)**: Storage-specific needs weren't analyzed - generic DTOs created without considering BoltDB vs SQLite vs JSON differences
- **Problem 3 (Write coordination)**: Dual-write coordination pattern was never established - no plan for handling BoltDB write success + SQLite write failure
- **Industry Pattern**: Hybrid storage requires explicit coordination (Unit of Work, Saga, eventual consistency)

**Misunderstanding (Secondary)**: CQRS pattern scope

- **Confusion**: Does CQRS mean separated ports OR separated models?
- **Current Assumption**: Separated ports (CacheWriterPort/CacheReaderPort) = CQRS
- **Industry Definition**: True CQRS requires separate read/write models optimized for their use case
- **Our Reality**: Separated interfaces with unified domain.Note model = not full CQRS
- **Complexity**: If we separate models, where does NoteProjection live? How does it handle IO concerns in domain?

**New Information (Discovery)**: Implementation gaps for decided questions

- **Problem 5 (SQLite views)**: Question 5 resolved with schema-driven views decision, but implementation never created
- **Discovered During**: Course correction review - decision documented, code doesn't exist
- **Gap Type**: Decision-to-implementation gap (not architectural uncertainty)

**Package Capability Gap (Research Need)**: fs.FileInfo/filepath underutilization

- **Current**: Using fs.FileInfo but not leveraging full capabilities
- **Problem**: Reimplementing functionality packages already provide
- **Research Goal**: Discover where stdlib solves problems we're solving manually

##### 1.4 What is the impact if we don't address this?

**Immediate Impacts**:

1. **Data Inconsistency Risk** (Storage Write Coordination):
   - **Scenario**: BoltDB write succeeds, SQLite write fails (disk full, constraint violation, etc.)
   - **Result**: QueryService merges both stores - inconsistent data returned to queries
   - **User Impact**: Template queries return incomplete/stale results from SQLite while BoltDB has current data
   - **Frequency**: Low probability but HIGH impact when occurs

2. **Query Performance Degradation** (SQLite Views Not Implemented):
   - **Current**: Must scan entire notes table filtering by fileclass, then extract JSON fields
   - **With Views**: Query directed to schema-specific view with pre-filtered fileclass and typed columns
   - **Performance Impact**: 10x-100x slower queries without views (full table scan vs indexed view)
   - **Template Impact**: Template rendering becomes sluggish as vault grows (defeats hybrid storage purpose)

3. **Incremental Indexing Broken** (QueryService Command/Query Mixing):
   - **Problem**: RefreshIncremental loads ALL notes because Note lacks ModTime for filtering
   - **Expected**: Load only notes modified since timestamp
   - **Impact**: Full index rebuild on every refresh instead of incremental updates
   - **Scale Impact**: 1000-note vault rebuilds entire index unnecessarily

4. **DTO Code Duplication** (DTO Architecture Not Focused):
   - **Current**: Generic DTOs reimplementing functionality fs.FileInfo/filepath already provide
   - **Impact**: Maintenance burden, potential bugs in reimplemented logic
   - **Example**: Computing basename, folder, extension manually instead of using filepath package

5. **CQRS Architecture Confusion**:
   - **Team Impact**: Unclear whether to add read-optimized models or keep unified Note
   - **Design Drift**: Different developers interpret CQRS differently → inconsistent patterns
   - **Over-Engineering Risk**: If full CQRS not needed, maintaining separated models adds complexity for no benefit

**Long-term Strategic Impacts**:

1. **Scalability Ceiling**: Without proper incremental indexing and view-based queries, performance degrades nonlinearly as vault grows
2. **Data Integrity Erosion**: No write coordination → inconsistencies accumulate over time, cache becomes unreliable
3. **Technical Debt Compound**: DTO reimplementation + missing views + command/query mixing = compounding maintenance cost
4. **Event-Driven Architecture Blocker**: QueryService doing command operations prevents clean event-driven design adoption

##### 1.5 What evidence supports this?

**Code Evidence**:

1. **QueryService Command Operation in Query Service** (`internal/app/query/service.go`):

   **Lines 391-423** - RefreshFromCache rebuilds indices (WRITE operation):

   ```go
   func (q *QueryService) RefreshFromCache(ctx context.Context) error {
       q.log.Info().Msg("refreshing query service from cache")
       notes, err := q.loadNotesForRefresh(ctx)
       if err != nil {
           return err
       }
       q.rebuildIndices(notes)  // ← WRITE operation rebuilding indices
       return nil
   }
   ```

   **Lines 464-467** - ModTime filtering broken comment:

   ```go
   // Note: ModTime filtering removed as domain.Note no longer has ModTime field
   // This is a temporary workaround - proper solution requires cache architecture redesign
   modifiedNotes := notes
   ```

2. **Note Domain Model Lacks File Metadata** (`internal/domain/note.go`):

   **Lines 7-14** - Pure domain entity, no IO concerns:

   ```go
   type Note struct {
       ID NoteID
       Frontmatter Frontmatter
   }
   // Missing: ModTime, Size, Path (separate from ID) - required for staleness detection
   ```

3. **Generic DTO Structure Not Focused** (`internal/shared/dto/file.go`):

   **Lines 22-54** - FileMetadata combines multiple concerns:

   ```go
   type FileMetadata struct {
       Path     string    // Path concerns
       Basename string    // Path concerns (computed)
       Folder   string    // Path concerns (computed)
       Ext      string    // Path concerns (computed)
       ModTime  time.Time // Date concerns
       Size     int64     // File stat concerns
       MimeType string    // Type detection concerns (computed)
   }
   ```

   **Problem**: Single struct mixing path manipulation, dates, file stats, type detection

   **Potential Focused Breakdown**:
   - FilePathDTO: Path, Basename, Folder, Ext
   - FileDatesDTO: ModTime, CreatedTime, IndexTime
   - FrontmatterDTO: All Fields, title, aliases, file_class

   **Research Needed**: Obsidian patterns will reveal better decomposition strategies

4. **No Write Coordination Code Exists**:
   - **BoltDB Writer**: `internal/adapters/spi/cache/boltdb_writer.go` - writes independently
   - **SQLite Writer**: `internal/adapters/spi/cache/sqlite_writer.go` - writes independently
   - **No Coordinator**: No code coordinating writes, handling partial failures, or ensuring consistency
   - **QueryService Merge**: Lines 506-522 merge from both stores with no consistency validation

**Architecture Decision Evidence** (Sprint Change Proposal):

1. **Question 4 (DTO Architecture)** - Lines 922-1129:
   - **Proposal Discussed**: NoteMetadataDTO + BoltDBMetadata/SQLiteMetadata extensions
   - **Status**: ❌ NOT FINALIZED - multiple decomposition strategies possible
   - **Example Options**: Break into FilePathDTO, FileDatesDTO, FrontmatterDTO or storage-specific structs
   - **Research Needed**: Obsidian patterns will inform final design

2. **Question 5 (SQLite Schema Views)** - Lines 1133-1479:
   - **Decision**: ✅ Schema-driven views over JSON storage
   - **Status**: DECIDED but ❌ NOT IMPLEMENTED

3. **Question 6 (Write Coordination)** - Lines 1483-1498:
   - **Options**: Atomic writes, eventual consistency, primary + async replication
   - **Status**: ❌ UNRESOLVED

**QueryService Hybrid Storage Evidence** (`internal/app/query/service.go`):

**Lines 175-194** - Constructor accepts two cache readers:

```go
func NewQueryService(
    boltReader   spi.CacheReaderPort,
    sqliteReader spi.CacheReaderPort,
    config       domain.Config,
    log          zerolog.Logger,
) *QueryService
```

**Lines 506-522** - Merges from both stores without consistency checking:

```go
// Merge notes from both stores, preferring SQLite for complete data
noteMap := make(map[domain.NoteID]domain.Note)
for _, note := range boltNotes {
    noteMap[note.ID] = note
}
for _, note := range sqliteNotes {
    noteMap[note.ID] = note  // ← No consistency validation
}
```

---

### Group 3: Orchestration & Coordination (2 issues - SYSTEM-WIDE)

**Issues**:

- **A1**: Component Orchestration (Event-driven vs Orchestrator pattern)
- **Related to A6**: Write coordination pattern (overlaps with storage)

**Why Grouped**: System-wide coordination patterns affecting component communication

#### Section 1: Understand Trigger & Context

##### 1.1 What triggered this change?

**Primary Trigger**: November 2, 2025 sprint change proposal Question 1 asked "How should component orchestration be structured to avoid god objects while maintaining clean architecture?"

**Immediate Context**: Sprint change introduced CLICommander (renamed from CommandOrchestrator) as workflow coordinator, with decision documented in proposal lines 594-720. However, Issue A1 status shows this is ❌ UNRESOLVED - need to evaluate event-driven vs orchestrator patterns.

**Missing Consideration Identified**: Event-driven architecture and other coordination patterns as potential solutions:

- **Event-Driven**: Domain events (NoteIndexed, FrontmatterValidated, SchemaLoaded) - would events reduce coupling AND reduce god-objects vs CLICommander orchestrator?
- **Saga Pattern**: Distributed transaction coordination (relevant for BoltDB + SQLite write coordination)
- **Mediator Pattern**: Component communication without direct coupling
- **Command Pattern**: Encapsulate operations for decoupled execution
- **Unit of Work Pattern**: Transaction coordination across multiple operations
- **Comparison Needed**: Trade-offs between patterns for our specific orchestration needs

**Secondary Trigger (Write Coordination Overlap)**: Issue A6 (Storage Write Coordination) relates to orchestration:

- BoltDB + SQLite dual-write coordination
- Question: Should write coordination be orchestrator responsibility, event-driven, or Saga pattern?
- Overlap: Same coordination pattern question at different layers

**Discovery from Group 2**: QueryService doing command operations (RefreshFromCache) suggests event-driven could help:

- Current: QueryService manually called to rebuild indices after indexing
- Event-Driven: IndexingComplete event → QueryService subscribes → rebuilds indices automatically
- Benefit: Separates command (indexing) from query (index building) concerns, reduces god-object

##### 1.2 What is the core issue?

**Two Interconnected Problems**:

1. **Orchestration Pattern Undecided**:
   - **Previous Attempt**: Orchestrator pattern chosen (CLICommander as workflow coordinator)
   - **Problem**: Resulted in brittle god-object - not implemented properly
   - **Current Status**: Orchestration pattern UNDECIDED - need to reconsider
   - **Evidence from main.go**: CLICommander has 7 dependencies injected (lines 60-68):
     - cliAdapter, templateEngine, schemaEngine, vaultIndexer, vaultWriter, cfg, log
   - **God-Object Indicators**: Aggregating many services, coordinating complex workflows
   - **Alternative Patterns to Evaluate**:
     - **Event-Driven**: Domain events (NoteIndexed, FrontmatterValidated, SchemaLoaded) decouple components
     - **Saga Pattern**: Distributed transaction coordination (relevant for write coordination)
     - **Mediator Pattern**: Component communication without direct coupling
     - **Command Pattern**: Encapsulate operations for decoupled execution
     - **Unit of Work**: Transaction coordination across multiple operations

2. **DI Pattern in main.go May Need Improvement**:
   - **Current DI** (`cmd/lithos/main.go`): Manual dependency construction in main() (lines 21-72)
   - **Question**: Could improving DI pattern also improve orchestration situation?
   - **Considerations**:
     - Does current manual DI contribute to god-object problem?
     - Would DI container/framework help? (e.g., wire, dig, fx)
     - How does chosen orchestration pattern affect DI needs?
     - Event-driven needs event bus in DI container
     - Orchestrator needs all services injected
   - **Pattern Interdependence**: DI pattern and orchestration pattern must work together

3. **Write Coordination Pattern Overlaps** (Issue A6 connection):
   - **Problem**: BoltDB + SQLite dual-write coordination undefined
   - **Orchestration Question**: Should this be orchestrator responsibility or independent pattern?
   - **Pattern Options**: Unit of Work, Saga, Event-Driven, or orchestrator-coordinated?

**Root Cause**: Orchestrator pattern attempted but resulted in god-object (CLICommander with 7 dependencies, VaultIndexer with 7 dependencies). Course correction evaluation confirms event-driven architecture is the correct solution for Epic 3 to eliminate god-objects and enable clean CQRS separation.

##### 1.3 Is this a misunderstanding, missing consideration, or new information?

**Decision Made**: Event-Driven Architecture for Epic 3

- **Pattern Chosen**: Event-driven architecture with domain events (NoteIndexed, VaultIndexingComplete, FrontmatterValidated, SchemaLoaded, SchemasReloaded)
- **Rationale**: Solves god-object problem by decoupling services via events, enables clean CQRS command/query separation, and aligns with Issue B1 fix (QueryService command/query mixing)
- **Implementation**: Story 3.29 implements EventBus, domain events, publishers, and subscribers
- **God-Object Mitigation**: Services communicate via events instead of direct dependencies - CLICommander and VaultIndexer dependency counts drastically reduced

**Missing Consideration (Secondary)**: DI pattern interdependence

- **What was missed**: How orchestration pattern choice affects DI requirements
- **Examples**:
  - Event-driven needs event bus in DI container
  - Orchestrator needs all services injected (contributes to god-object)
  - Different patterns have different DI complexity
- **Question not asked**: Could improving DI pattern reduce orchestration complexity?
- **Current DI**: Manual construction in main.go - no evaluation of DI containers/frameworks

**Misunderstanding (Possible)**: Orchestrator pattern wouldn't become god-object

- **Assumption**: CLICommander could coordinate workflows without becoming bloated
- **Reality**: 7 dependencies injected, coordinating complex workflows → god-object indicators
- **Industry Pattern**: Orchestrators often become god-objects without careful boundaries
- **Missed**: Need explicit strategies to prevent god-object (limit dependencies, clear scope, event delegation)

**New Information (Discovery)**: QueryService command/query mixing reinforces event-driven need

- **Discovered**: Group 2 analysis shows QueryService doing command operations (RefreshFromCache)
- **Event-Driven Solution**: IndexingComplete event → QueryService subscribes → clean separation
- **Insight**: Event-driven could solve multiple problems (orchestration + CQRS separation)

##### 1.4 What is the impact if we don't address this?

**Immediate Impacts**:

1. **God-Object Pattern Spreading to Multiple Services**:
   - **CLICommander**: 7 dependencies (cliAdapter, templateEngine, schemaEngine, vaultIndexer, vaultWriter, cfg, log)
   - **VaultIndexer** (`main.go` lines 49-57): 7 dependencies (vaultScanner, cacheWriter, cacheReader, frontmatterService, schemaEngine, cfg, log)
   - **FrontmatterService**: Growing dependencies as validation needs expand
   - **Root Cause**: Lack of coordination pattern forces services to orchestrate internally
   - **Pattern**: Each service becomes mini-orchestrator → multiple god-objects

2. **Tight Coupling Increases**:
   - **Problem**: Every component knows about multiple other components
   - **Change Ripple**: Modifying one service requires updating multiple orchestrators (CLICommander, VaultIndexer, etc.)
   - **Example**: Adding cache invalidation touches CLICommander, VaultIndexer, and cache code
   - **Flexibility**: Can't swap implementations without touching multiple god-objects

3. **Testing Becomes Impossible**:
   - **Unit Testing**: VaultIndexer requires mocking 7 dependencies
   - **FrontmatterService**: Growing mock complexity as dependencies added
   - **CLICommander**: Unmaintainable test setup
   - **Trend**: Each new feature makes testing harder

4. **Write Coordination Remains Unresolved** (Issue A6):
   - **Blocker**: Can't decide write coordination without orchestration pattern
   - **Questions Blocked**: Should orchestrator coordinate? Event-driven? Saga?
   - **Risk**: BoltDB + SQLite inconsistency continues

5. **DI Complexity Compounds**:
   - **Current**: Manual DI in main.go already 72 lines
   - **Trend**: Each new service adds more boilerplate
   - **Multiple God-Objects**: DI must wire up VaultIndexer, FrontmatterService, CLICommander dependencies
   - **Alternative**: DI framework could help, but choice depends on orchestration pattern

6. **Event-Driven Benefits Not Realized**:
   - **CQRS Separation**: QueryService continues doing command operations
   - **Decoupling**: Components remain tightly coupled through multiple orchestrators
   - **Scalability**: Can't easily add async processing, event sourcing, etc.

**Long-term Strategic Impacts**:

1. **Architecture Lock-In**: Multiple god-objects become too painful to refactor - locked into brittle pattern
2. **Feature Velocity Slows**: Every new feature requires changes to multiple orchestrators - bottleneck
3. **Testing Debt**: Unit testing impossible → rely on slow integration tests → development slows
4. **Team Scaling**: New developers struggle with complex interdependencies - onboarding difficulty

##### 1.5 What evidence supports this?

**Code Evidence - Multiple God-Objects** (`cmd/lithos/main.go`):

1. **CLICommander - 7 Dependencies** (lines 60-68):

   ```go
   orchestrator := command.NewCLIComander(
       cliAdapter,      // 1. CLI interaction
       templateEngine,  // 2. Template rendering
       schemaEngine,    // 3. Schema operations
       vaultIndexer,    // 4. Indexing operations
       vaultWriter,     // 5. Vault writing
       &cfg,           // 6. Configuration
       &log,           // 7. Logging
   )
   ```

2. **VaultIndexer - 7 Dependencies** (lines 49-57):

   ```go
   vaultIndexer := vault.NewVaultIndexer(
       vaultScanner,       // 1. Vault scanning
       cacheWriter,        // 2. Cache writing
       cacheReader,        // 3. Cache reading
       frontmatterService, // 4. Frontmatter extraction/validation
       schemaEngine,       // 5. Schema operations
       cfg,               // 6. Configuration
       log,               // 7. Logging
   )
   ```

3. **FrontmatterService - Growing Dependencies** (line 48):
   ```go
   frontmatterService := frontmatter.NewFrontmatterService(schemaEngine, log)
   // Currently 2 dependencies, but needs more as validation expands
   ```

**Manual DI Complexity** (`cmd/lithos/main.go` lines 21-72):

- **72 lines** of manual dependency construction
- **Every service** requires explicit instantiation and wiring
- **Dependency ordering** matters - must construct in correct sequence
- **No abstraction**: Direct coupling between main() and all services

**Sprint Change Proposal Evidence** (Lines 594-720):

- **Question 1 Decision**: CLICommander as orchestrator documented
- **Status in Issue Inventory**: ❌ UNRESOLVED - reconsidering pattern
- **Gap**: Decision documented but resulted in god-object, need to reconsider

**Issue Inventory Evidence**:

From Issue A1 (lines 122-130):

```markdown
#### Issue A1: Component Orchestration Architecture ❌ UNRESOLVED

- **Status**: Reconsidering - need to evaluate event-driven vs orchestrator patterns
- **Missing Consideration**: Event-driven architecture as solution to god-object problem
- **Questions**:
  - Should we use event-driven design for complex orchestration?
  - Would domain events (NoteIndexed, FrontmatterValidated, SchemaLoaded) reduce coupling?
  - How does event-driven approach compare to orchestrator pattern?
```

**Group 2 Discovery Evidence**:

- QueryService.RefreshFromCache() is command operation in query service
- Suggests event-driven could solve CQRS separation AND orchestration problems
- IndexingComplete event → QueryService subscribes → automatic index rebuild

---

### Group 4: Configuration Management (2 issues - INFRASTRUCTURE)

**Issues**:

- **A2**: Singleton Pattern for Config/PropertyBank
- **A3**: FileClassKey Configuration Impact

**Why Grouped**: Both about configuration architecture and lifecycle

##### 1.1 What triggered this change?

**Primary Trigger**: November 2, 2025 sprint change proposal (docs/course_correction/sprint-change-proposal-2025-11-02-epic3-hybrid-storage-architecture.md) introduced six architectural questions requiring resolution before proceeding with Epic 3 implementation.

**Two Configuration-Related Questions With Finalized Decisions**:

- **Question 2 (Issue A2)**: Singleton Pattern Implementation - ✅ DECIDED (sync.Once for Config/PropertyBank) - Implementation pending
- **Question 3 (Issue A3)**: FileClassKey Configuration Impact - ✅ DECIDED (config-driven schema selection) - Implementation pending

**Critical Gap Discovered**: During architectural review, identified that ViperAdapter (internal/adapters/spi/config/viper.go) does NOT load FileClassKey from config file/env vars, despite Question 3 decision requiring config-driven schema selection.

**Configuration Architecture Scope**:

1. **Config Lifecycle**: How to initialize and access Config singleton throughout system
2. **PropertyBank Lifecycle**: How to initialize and access PropertyBank singleton throughout system
3. **FileClassKey Integration**: How config-driven schema selection propagates through all file classification touchpoints
4. **Test Harness Support**: How to swap singleton instances for testing without data races

##### 1.2 What is the core issue?

**Two Interconnected Configuration Problems**:

1. **Singleton Pattern Not Implemented (Issue A2)**:
   - **Current State**: Config and PropertyBank passed via dependency injection in main.go
   - **Problem**: No singleton lifecycle management - multiple instances possible, no thread safety
   - **Decision**: Use sync.Once pattern for proper singleton implementation
   - **Requirements**:
     - Package-level variables with sync.Once guards
     - GetConfig()/GetPropertyBank() accessor functions
     - Test harness support methods for instance swapping (critical for tests)
     - Documentation of singleton lifecycle and test patterns
   - **Testability Concern**: Singletons complicate testing - need explicit support for test instance swapping

2. **FileClassKey Config Loading Gap (Issue A3)**:
   - **Decision**: Config-driven schema selection (fileClass field name configurable)
   - **Critical Missing Implementation**: ViperAdapter does NOT load FileClassKey from config file/env vars
   - **Current Code Gap** (internal/adapters/spi/config/viper.go):
     - Loads VaultPath, CachePath, SchemaPath
     - Does NOT load FileClassKey configuration
   - **Impact**: Config-driven schema selection impossible without config loading
   - **Dependencies**:
     - internal/domain/frontmatter.go needs FileClassKey from Config
     - Schema selection logic depends on configured field name
     - All file classification touchpoints need config-driven key

**Interdependence**: FileClassKey config loading (A3) depends on Config singleton implementation (A2) - must access Config.FileClassKey throughout system.

##### 1.3 Is this a misunderstanding, missing consideration, or new information?

**Issue A2 (Singleton Pattern)**: **New Information**

- Sprint change proposal Question 2 identified need for proper singleton lifecycle management
- Analysis revealed Config and PropertyBank should be singletons (global state)
- Not a misunderstanding - current DI approach is valid but doesn't fit singleton use case
- New architectural requirement from production-readiness analysis

**Issue A3 (FileClassKey Loading Gap)**: **Missing Consideration**

- Question 3 decision made: config-driven schema selection
- Implementation overlooked critical step: ViperAdapter must load FileClassKey
- Not a misunderstanding - decision is correct
- Missing: Complete implementation of config loading for FileClassKey
- Discovery: During architectural review, noticed ViperAdapter doesn't load this critical config

##### 1.4 What is the impact if we don't address this?

**Impact of Not Implementing Singleton Pattern (Issue A2)**:

1. **Multiple Config Instances Possible**:
   - Without sync.Once, multiple Config instances could be created
   - Data races if accessed concurrently from multiple goroutines
   - Inconsistent configuration state across system

2. **Testing Complexity**:
   - Cannot easily swap Config instances for tests
   - No controlled singleton lifecycle for test isolation
   - Risk of test pollution (shared state between tests)

3. **Architectural Inconsistency**:
   - Config and PropertyBank are conceptually global state (singletons)
   - Current DI approach treats them like regular dependencies
   - Mismatch between architectural intent and implementation

4. **Developer Confusion**:
   - Unclear whether to inject Config or access globally
   - No documented pattern for Config access
   - Inconsistent usage across codebase

**Impact of Not Loading FileClassKey from Config (Issue A3)**:

1. **Hardcoded Field Name**:
   - fileClass field name would be hardcoded in frontmatter.go
   - Cannot configure different field name per environment
   - Violates config-driven architecture decision

2. **Schema Selection Broken**:
   - Config-driven schema selection impossible
   - System cannot adapt to different vault conventions
   - Obsidian users with different fileClass naming conventions cannot use Lithos

3. **Incomplete Question 3 Implementation**:
   - Decision made but not fully implemented
   - Blocks Story 3.6+ (requires config-driven schema selection)
   - Technical debt from incomplete architectural decision

4. **Cascade to Other Components**:
   - All components depending on file classification are blocked
   - Cannot proceed with schema-driven SQLite views (depends on FileClassKey)
   - Cannot proceed with frontmatter validation (depends on schema selection)

##### 1.5 What evidence supports this?

**Evidence for Issue A2 (Singleton Pattern Not Implemented)**:

1. **Current DI Pattern in main.go** (lines 21-72):
   - Config and PropertyBank constructed in main()
   - Passed via dependency injection to all services
   - No singleton pattern - standard dependency injection approach
   - Evidence: `cfg := config.NewConfig(...)` (line 32), then passed to all constructors

2. **Sprint Change Proposal - Question 2** (docs/course_correction/sprint-change-proposal-2025-11-02-epic3-hybrid-storage-architecture.md:724):
   - ✅ DECISION FINALIZED: sync.Once pattern for Config and PropertyBank
   - Explicit decision to move from DI to singleton pattern
   - Reason: Config/PropertyBank are global state, not regular dependencies

3. **Issue Inventory - Issue A2** (line 132-139):
   - Status: ✅ DECISION FINALIZED
   - Implementation Pending:
     - Package-level variables with sync.Once
     - GetConfig()/GetPropertyBank() accessors
     - Test harness support methods
     - Documentation updates

**Evidence for Issue A3 (FileClassKey Config Loading Gap)**:

1. **Config Domain Model Has FileClassKey** (internal/domain/config.go:59-63):

```go
// FileClassKey is the frontmatter key used to identify file class/schema.
// Default: "file_class". Supports user preferences like "fileClass", "type", etc.
// Used consistently across all storage adapters and query operations.
FileClassKey string `yaml:"file_class_key" mapstructure:"file_class_key"`
```

2. **Default Value Defined** (internal/domain/config.go:14):

```go
defaultFileClassKey = "file_class"
```

3. **ViperAdapter Does NOT Load FileClassKey** (internal/adapters/spi/config/viper.go):
   - **loadConfigFile()** (lines 143-167): Loads VaultPath, TemplatesDir, SchemasDir, PropertyBankFile, CacheDir, LogLevel
     - ❌ NO FileClassKey loading
   - **loadEnvironmentVars()** (lines 226-264): Environment mappings for LITHOS_VAULT_PATH, LITHOS_TEMPLATES_DIR, etc.
     - ❌ NO LITHOS_FILE_CLASS_KEY mapping

4. **Tests Expect FileClassKey** (internal/domain/config_test.go:138-143):

```go
if config.FileClassKey != tt.expectedFileClassKey {
    t.Errorf("expected FileClassKey %q, got %q",
        tt.expectedFileClassKey, config.FileClassKey)
}
```

- Tests verify FileClassKey is set correctly
- But ViperAdapter never loads it from config file/env vars

5. **Sprint Change Proposal - Question 3** (docs/course_correction/sprint-change-proposal-2025-11-02-epic3-hybrid-storage-architecture.md:821):
   - ✅ DECISION FINALIZED: Config-driven schema selection
   - FileClassKey must be configurable per environment
   - Implementation gap: Config loading not updated

6. **Issue Inventory - Issue A3** (line 141-148):
   - Status: ✅ DECISION FINALIZED
   - **CRITICAL MISSING**: ViperAdapter not loading FileClassKey from config file/env vars
   - Implementation Pending: internal/adapters/spi/config/viper.go updates (CRITICAL)

---

### Group 5: Schema Domain System (1 issue - DOMAIN SPECIFIC)

**Issues**:

- **B3**: Schema Loading/Registration Coupling (SchemaLoaderPort vs SchemaRegistryPort)

**Why Grouped**: Schema-specific domain concern (A5 SQLite moved to Group 2 Storage)

##### 1.1 What triggered this change?

**Primary Trigger**: Architectural review during Epic 3 sprint change proposal analysis revealed unnecessary complexity in schema port architecture.

**Discovery Context**: While analyzing hexagonal architecture patterns and port definitions, identified that SchemaLoaderPort and SchemaRegistryPort have overlapping responsibilities creating coupling without adding value.

**Issue Classification**: Issue B3 (Schema Loading/Registration Coupling) - Moderate priority simplification opportunity.

**Port Architecture Analysis**:

- **Current Design**: Separate ports for loading (SchemaLoaderPort) and registration (SchemaRegistryPort)
- **Coupling Problem**: Two ports manage tightly coupled operations (load → register)
- **Complexity Impact**: Clients must coordinate between two ports for single logical operation
- **Simplification Opportunity**: Merge responsibilities into unified SchemaRegistryPort

##### 1.2 What is the core issue?

**Unnecessary Port Separation**:

1. **Current Architecture - Two Separate Ports**:
   - **SchemaLoaderPort**: Loads schemas from filesystem
   - **SchemaRegistryPort**: Stores and retrieves loaded schemas in memory
   - **Client Coordination**: Clients must call SchemaLoaderPort.Load() then SchemaRegistryPort.Register()

2. **Coupling Problem**:
   - Loading and registration are **always** performed together (tight coupling)
   - No use case for loading without registration or vice versa
   - Two-step operation creates coordination burden on clients
   - Port separation adds complexity without flexibility

3. **Proposed Simplification**:
   - **Unified SchemaRegistryPort**:
     - GetSchema(name) tries registry first
     - If not found, automatically loads from filesystem
     - Auto-registers after successful load
   - **Remove SchemaLoaderPort**:
     - Loading becomes internal implementation detail of SchemaRegistry
     - Filesystem adapter becomes SPI dependency of SchemaRegistry
   - **Client Simplification**:
     - Single call: `registry.GetSchema(name)`
     - No coordination needed - registry handles load-register lifecycle

4. **Alternative SchemaLoader Approach**:
   - Keep SchemaLoader but auto-register on load
   - SchemaLoader.Load() returns Schema AND registers it
   - Still simpler than current two-step coordination

**Key Insight**: Ports should reflect business operations, not implementation steps. "Get schema by name" is the business operation - whether it's cached or loaded is an implementation detail.

##### 1.3 Is this a misunderstanding, missing consideration, or new information?

**New Information** (Simplification Opportunity)

- Not a misunderstanding - current port separation is functionally correct
- Not missing consideration - original design intentionally separated load/register concerns
- **New insight from architectural review**: Port separation adds complexity without value
- **Pattern Recognition**: Hexagonal architecture ports should align with business operations, not internal steps
- **Opportunity**: Simplify by merging tightly-coupled ports into single unified port
- **Status**: Moderate priority (⚠️) - system works but could be cleaner

##### 1.4 What is the impact if we don't address this?

**Impact of Not Simplifying Port Architecture**:

1. **Unnecessary Cognitive Load**:
   - Developers must understand two ports instead of one
   - Client code requires coordination between SchemaLoader and SchemaRegistry
   - Increased complexity for simple operation: "get schema by name"

2. **Coordination Bugs**:
   - Clients might load schema but forget to register
   - Clients might try to get schema before loading
   - Two-step operation creates opportunity for coordination errors

3. **Testing Complexity**:
   - Must mock both SchemaLoaderPort and SchemaRegistryPort
   - Test setup requires coordinating two ports
   - More test code for simple schema retrieval scenarios

4. **Architectural Inconsistency**:
   - Violates hexagonal architecture principle: ports should reflect business use cases
   - Current design reflects implementation details (load → register steps)
   - Should reflect business operation: "get schema by name"

5. **Maintenance Burden**:
   - Changes to schema loading require updating both ports
   - More interfaces to maintain and document
   - Port coordination logic duplicated across clients

**Note**: This is **moderate priority** because system is functionally correct - simplification improves maintainability and clarity but doesn't fix broken functionality.

##### 1.5 What evidence supports this?

**Evidence for Port Coupling**:

1. **SchemaPort Interface Definition** (internal/ports/spi/schema.go:32-56):

```go
type SchemaPort interface {
    // Load retrieves all schemas and the property bank from storage.
    Load(ctx context.Context) ([]domain.Schema, domain.PropertyBank, error)
}
```

- Single method: Load() - retrieves schemas from filesystem
- Returns raw schemas without registration
- Comment line 63: "Populated by SchemaEngine at startup from SchemaPort.Load() results"

2. **SchemaRegistryPort Interface Definition** (internal/ports/spi/schema.go:80-132):

```go
type SchemaRegistryPort interface {
    GetSchema(ctx context.Context, name string) (domain.Schema, error)
    GetProperty(ctx context.Context, name string) (domain.Property, error)
    HasSchema(ctx context.Context, name string) bool
    HasProperty(ctx context.Context, name string) bool
    RegisterAll(ctx context.Context, schemas []domain.Schema, bank domain.PropertyBank) error
}
```

- GetSchema/GetProperty: Retrieve registered schemas
- RegisterAll: Must be called to register schemas loaded from SchemaPort
- Comment line 63: "Populated by SchemaEngine at startup from SchemaPort.Load() results"

3. **Two-Step Coordination Required**:
   - Step 1: Call `SchemaPort.Load()` → returns schemas
   - Step 2: Call `SchemaRegistryPort.RegisterAll(schemas, bank)` → registers schemas
   - Step 3: Call `SchemaRegistryPort.GetSchema(name)` → retrieves schema
   - **Tight Coupling**: Load always followed by RegisterAll, no use case for Load without RegisterAll

4. **Issue Inventory - Issue B3** (line 205-212):
   - Status: ⚠️ MODERATE - Simplification opportunity
   - Problem: Unnecessary complexity with separate SchemaPort and SchemaRegistryPort
   - Proposal:
     - SchemaLoader automatically registers on load (alternative approach)
     - SchemaRegistry tries loading if GetSchema fails (unified approach)
     - Remove SchemaPort, keep only SchemaRegistryPort

5. **Hexagonal Architecture Principle Violation**:
   - Ports should reflect business operations: "Get schema by name"
   - Current design reflects implementation steps: "Load from filesystem" + "Register in memory" + "Get from registry"
   - Port separation exposes internal coordination to clients

---

### Group 6: Template System (1 issue - CRITICAL DEPENDENCY)

**Issues**:

- **Template Struct Analysis**:
  - Name conflict with text/template package?
  - Do we even need Template struct given stdlib?
  - If kept, should embed \*template.Template?
  - Is it fully utilizing text/template features?

**Why Standalone**: Epic 5 depends on this resolution; needs deep analysis of stdlib usage

##### 1.1 What triggered this change?

**Primary Trigger**: Epic 5 (Template Engine) planning identified need to evaluate current Template struct design before proceeding with template system implementation.

**Discovery Context**: During architectural review for Epic 3, noticed that domain.Template struct is minimal (ID + Content string) and doesn't leverage Go's text/template stdlib capabilities.

**Critical Dependency**: Epic 5 implementation depends on fundamental Template struct design decisions:

- **If keeping Template struct**: Must determine how it interacts with text/template package
- **If removing Template struct**: Must redesign template system to use text/template directly

**Four Key Questions Identified**:

1. **Name Conflict**: Does domain.Template conflict with text/template package imports?
2. **Necessity**: Do we even need Template struct given text/template stdlib?
3. **Embedding**: If kept, should Template embed \*template.Template for stdlib access?
4. **Utilization**: Is current design fully utilizing text/template features (composition, function maps, etc.)?

**Research Phase Dependency**: Group 6 analysis depends on Phase 1 research of text/template package capabilities (template composition, function maps, execution patterns).

##### 1.2 What is the core issue?

**Critical Analysis of Template System Architecture**:

1. **Anemic Model Anti-Pattern (Related to Issue D1)**:
   - **Current State** (template.go line 6): Comment says "follows the anemic domain model pattern"
   - **Critical Question**: Just because it SAYS "intentionally anemic" doesn't mean it's RIGHT
   - **Issue D1 Context**: We're identifying anemic models as CRITICAL problem across system (Frontmatter, Note, Template all anemic)
   - **Template Responsibilities That Should Live on Entity**:
     - Validate() - Template syntax validation before execution
     - Render(data) - Execute template with data context
     - GetDependencies() - Extract {{template "name"}} references
   - **Current Problem**: All template logic in TemplateEngine service, Template is just ID + Content bag

2. **Questionable Need for domain.Template Struct**:
   - **Current Flow** (service.go lines 99-108):
     - Load domain.Template (ID + Content string)
     - Immediately convert to text/template.Template via getCompiledTemplate()
     - domain.Template discarded, text/template.Template used for execution
   - **Critical Question**: Why have intermediate domain.Template at all?
   - **Alternative**: Use text/template.Template directly throughout system
   - **What does domain.Template add?**: Just type safety for TemplateID? Is that worth the abstraction?

3. **Potential Name Confusion** (Not Direct Conflict):
   - service.go line 44: `tpl *template.Template` (this is text/template.Template from import)
   - service.go line 196: `tmpl domain.Template` (this is domain.Template)
   - **Confusion Risk**: When reading code, "Template" alone is ambiguous
   - Must always check: Is this domain.Template or template.Template?
   - **Better naming**: If keeping domain struct, rename to NoteTemplate or MarkdownTemplate?

4. **Incomplete text/template Utilization**:
   - **Currently Used** (service.go):
     - template.New() (line 207) - basic creation
     - .Funcs() (line 208) - custom function map
     - .Parse() (line 209) - parse template content
     - .Execute() (line 112) - render with nil data
   - **NOT Used** (may need for Epic 5):
     - template.Clone() - concurrent execution safety
     - {{template "name"}} composition - no template dependency loading
     - .DefinedTemplates() - introspection
     - .Option() - template configuration
     - Data context beyond nil - line 112: `t.Execute(&buf, nil)` always empty context

5. **Missing Template Composition Support**:
   - **text/template Feature**: {{template "name"}} to include other templates
   - **Current Implementation**: TemplateEngine.Load() loads single template only
   - **Epic 5 Needs**: Template composition for reusable components (headers, footers, shared sections)
   - **Missing**: Template dependency resolution, composition graph, multi-template loading

6. **No Template Validation Before Execution**:
   - **Current**: Parse errors only caught during Execute() (service.go line 209-216)
   - **Problem**: Template syntax errors discovered at render time, not load time
   - **Question**: Should Template.Validate() method check syntax immediately after Load()?
   - **Benefit**: Fail fast on template errors, not when user tries to create note

**Key Architectural Questions for Epic 5**:

1. Should Template remain anemic data bag, or become rich model with Validate(), Render(), GetDependencies() methods?
2. Do we even need domain.Template, or should we use text/template.Template directly?
3. If keeping domain.Template, should it embed \*template.Template for rich behavior?
4. How to support {{template "name"}} composition for Epic 5?
5. What additional text/template features needed for Epic 5 (Clone, data contexts, etc.)?

##### 1.3 Is this a misunderstanding, missing consideration, or new information?

**Multiple Issues - Mixed Classification**:

1. **Anemic Model (Issue #1)**: **Missing Consideration**
   - Template was designed anemic like rest of system
   - Issue D1 now identifies anemic models as CRITICAL anti-pattern
   - Missing: Recognition that Template should have behavior (Validate, Render, GetDependencies)
   - Discovered during comprehensive architectural review

2. **Need for domain.Template Struct (Issue #2)**: **New Information** (Critical Analysis)
   - Original design assumed domain.Template adds value
   - Critical review reveals: Just intermediary immediately converted to text/template.Template
   - New insight: May not need domain.Template at all
   - Pattern recognition: Over-abstraction without clear benefit

3. **Name Confusion (Issue #3)**: **Misunderstanding**
   - Not a conflict, but creates developer confusion
   - `template.Template` vs `domain.Template` ambiguity
   - Easy fix: Rename domain.Template if keeping it

4. **Incomplete text/template Utilization (Issue #4)**: **Missing Consideration**
   - Current implementation uses basic text/template features only
   - Epic 5 will need: composition, Clone(), richer data contexts
   - Missing: Full stdlib capability assessment before Epic 5 planning

5. **Template Composition (Issue #5)**: **Missing Consideration**
   - {{template "name"}} feature exists in text/template
   - Current TemplateEngine only loads single templates
   - Missing: Multi-template loading, dependency resolution
   - Epic 5 blocker: Cannot implement template composition without this

6. **No Pre-Execution Validation (Issue #6)**: **Missing Consideration**
   - Template errors only discovered at execution time
   - Missing: Fail-fast validation at load time
   - Should Template have Validate() method?

##### 1.4 What is the impact if we don't address this?

**Impact on Epic 5 Implementation**:

1. **Anemic Model Perpetuation**:
   - Continues Issue D1 anti-pattern into Epic 5
   - Template logic scattered across services instead of encapsulated
   - Makes Epic 5 harder to implement - where does template composition logic go?
   - Inconsistent with fixing anemic models in Frontmatter/Note

2. **Epic 5 Planning Uncertainty**:
   - Cannot plan Epic 5 stories without fundamental Template design decision
   - **If removing domain.Template**: Major refactor of TemplateEngine, TemplatePort, file loaders
   - **If enriching domain.Template**: Need Validate(), Render(), GetDependencies() methods
   - **If keeping as-is**: Template composition logic goes in service (god-object risk)

3. **Template Composition Blocked**:
   - Cannot implement {{template "name"}} includes without multi-template loading
   - Epic 5 requirement: Reusable template components (headers, footers, shared sections)
   - Current TemplateEngine.Load() only handles single template
   - Need: Template dependency graph, recursive loading, composition validation

4. **Late Error Discovery**:
   - Template syntax errors only caught at execution time (user action)
   - Poor developer experience: Template bugs not found until "lithos new" command
   - Should fail fast at template load time, not during user workflow

5. **text/template Feature Gap**:
   - Epic 5 may need features we haven't explored:
     - Clone() for concurrent rendering
     - Richer data contexts beyond nil
     - Custom delimiters via Option()
     - Template introspection via DefinedTemplates()
   - Risk: Discover missing features mid-Epic 5 implementation

6. **Architectural Inconsistency Risk**:
   - If we fix anemic models in Note/Frontmatter but not Template
   - Inconsistent patterns across domain entities
   - Developer confusion: When should entities have behavior?

**Critical Path Impact**: Epic 5 cannot proceed until Template architecture decided. This is BLOCKING issue for Epic 5 planning.

##### 1.5 What evidence supports this?

**Evidence from Code**:

1. **Anemic domain.Template** (internal/domain/template.go:3-18):

```go
// Template represents an executable template for note generation.
// It is a pure data structure containing only the template identity and raw
// content. This follows the anemic domain model pattern where business logic
// resides in services.
type Template struct {
    ID TemplateID
    Content string
}
```

- Comment line 6: "follows the anemic domain model pattern" - explicitly anemic
- No behavior methods - just NewTemplate() constructor
- Matches Issue D1 pattern: Frontmatter, Note, Template all anemic

2. **Immediate Conversion to text/template.Template** (internal/app/template/service.go:94-123):

```go
func (e *TemplateEngine) Render(ctx context.Context, templateID domain.TemplateID) (string, error) {
    // Step 1: Load template
    tmpl, err := e.Load(ctx, templateID)  // Returns domain.Template

    // Step 2-3: Create text/template with function map
    t, err := e.getCompiledTemplate(tmpl)  // Converts to *template.Template

    // Step 5-6: Execute with empty data context
    var buf strings.Builder
    if executeErr := t.Execute(&buf, nil); executeErr != nil {  // Uses text/template.Template
        return "", errors.NewTemplateError(...)
    }
}
```

- domain.Template loaded (line 99), immediately converted (line 105)
- Actual execution uses text/template.Template (line 112)
- domain.Template is just transport object

3. **Conversion Logic** (service.go:195-226):

```go
func (e *TemplateEngine) getCompiledTemplate(tmpl domain.Template) (*template.Template, error) {
    // ... caching logic ...

    parsed, err := template.New(string(tmpl.ID)).  // text/template creation
        Funcs(e.getFuncMap()).
        Parse(tmpl.Content)

    e.compiled[tmpl.ID] = cachedTemplate{
        tpl: parsed,  // Stores *template.Template, not domain.Template
        checksum: checksum,
    }
    return parsed, nil
}
```

- Line 207: Creates stdlib `template.Template` from domain.Template
- Line 219: Caches text/template.Template, domain.Template discarded
- Question: Why not work with text/template.Template from start?

4. **Single Template Loading Only** (service.go:141-147):

```go
func (e *TemplateEngine) Load(ctx context.Context, templateID domain.TemplateID) (domain.Template, error) {
    e.log.Debug().Str("templateID", string(templateID)).Msg("loading template")
    return e.templatePort.Load(ctx, templateID)
}
```

- Loads one template by ID
- No multi-template loading for composition
- No dependency resolution for {{template "name"}} references

5. **Empty Data Context** (service.go:112):

```go
if executeErr := t.Execute(&buf, nil); executeErr != nil {
```

- Always executes with `nil` data
- Comment line 82: "Epic 1" limitation - static rendering only
- Epic 5 will need richer data contexts

6. **Basic text/template Feature Usage** (service.go:158-189):

```go
e.funcMap = template.FuncMap{
    // Basic functions
    "now": func(format string) string { return time.Now().Format(format) },
    "toLower": strings.ToLower,
    "toUpper": strings.ToUpper,
    // File path control functions
    "path": func() string { return "" },
    "folder": filepath.Dir,
    "basename": func(p string) string { ... },
    "extension": filepath.Ext,
    "join": filepath.Join,
    "vaultPath": func() string { return e.config.VaultPath },
}
```

- Uses: template.New(), .Funcs(), .Parse(), .Execute()
- NOT using: .Clone(), .DefinedTemplates(), .Option(), composition

7. **Name Ambiguity in Code** (service.go):
   - Line 13: `"text/template"` import
   - Line 16: `"github.com/JackMatanky/lithos/internal/domain"` import
   - Line 44: `tpl *template.Template` (stdlib)
   - Line 196: `tmpl domain.Template` (domain)
   - Developer must track which "Template" is referenced

8. **Issue Inventory - Group 6 Classification** (lines 1538-1548):
   - Why Standalone: Epic 5 depends on this resolution
   - Four questions identified: name conflict, necessity, embedding, utilization
   - Research Phase Dependency: text/template package capabilities

---

### Group 7: Documentation & Patterns (1 issue - META)

**Issues**:

- **D3**: Missing Pattern Documentation

**Why Standalone**: Meta-issue about documenting patterns discovered in other groups

##### 1.1 What triggered this change?

**Primary Trigger**: Comprehensive architectural review (Groups 1-6) discovered multiple architectural patterns and decisions not documented in architecture documentation.

**Documentation Misalignment**: Issue C3 (Documentation Misalignment) identifies that architecture docs don't reflect current system state or decisions made during Epic 3 planning.

**Pattern Discovery Context**: Analysis of Groups 1-6 revealed:

- Event-driven vs orchestrator pattern evaluation (Group 3)
- Unit of Work vs dual-write patterns (Group 2)
- Singleton pattern implementation (Group 4)
- CQRS pattern scope questions (Group 2)
- Schema port simplification (Group 5)
- Anemic vs rich domain models (Issue D1)
- Hexagonal validation layers (syntactic vs semantic - Group 1)

**Developer Impact**: Without pattern documentation, developers implementing from docs will build incorrect architecture (mismatch between docs and decisions).

##### 1.2 What is the core issue?

**Incomplete/Outdated Pattern Documentation Across Multiple Files**:

**1. high-level-architecture.md - Pattern Section Needs Updates** (lines 91-141):

**Currently Documents**:

- Hexagonal Architecture, Repository Pattern, Dependency Injection, Builder Pattern, CQRS
- Design Principles: DIP, Lean Ports, ISP, Lean Domain Models, Error Handling

**Missing/Incorrect**:

- Orchestration patterns (Event-driven, Saga, Mediator, Command, Unit of Work) - Group 3
- Singleton pattern (sync.Once for Config/PropertyBank) - Group 4
- **CONTRADICTION**: Line 135 "Lean Domain Models: contain only essential data with no behavior" endorses anemic model anti-pattern (Issue D1 says this is CRITICAL problem)
- Hexagonal validation layer responsibilities (syntactic in adapters, semantic in domain) - Group 1
- Port design principle: "reflect business operations, not implementation steps" - Group 5
- Storage patterns: Hybrid BoltDB + SQLite, write coordination, DTO architecture - Group 2
- Go stdlib utilization patterns (fs.FileInfo, text/template) - Groups 2, 6

**2. components.md - Missing Component Documentation**:

**Currently Missing**:

- CLICommander orchestration pattern and responsibilities
- Singleton lifecycle for Config and PropertyBank
- Schema-driven SQLite views implementation
- QueryService command/query responsibility separation (CQRS violation)
- NoteProjection domain/IO boundary tension
- FrontmatterService refactoring (extraction to adapter layer)
- VaultIndexer god-object dependencies

**3. data-models.md - Missing Model Documentation**:

**Currently Missing**:

- DTO architecture decisions (FileMetadata, VaultFile redesign with fs.FileInfo)
- Anemic vs rich model guidelines (when entities should have behavior)
- Domain/IO boundary patterns (NoteProjection needs filesystem data)
- Template struct design decisions (anemic data bag vs rich model)
- Note.Content field addition impact on storage

**4. coding-standards.md - Missing Pattern Guidance**:

**Currently Missing**:

- Singleton accessor usage patterns (GetConfig(), GetPropertyBank())
- Test harness patterns for singleton instance swapping
- Validation method naming (Validate vs ValidateAgainstSchema)
- Hexagonal architecture validation layer guidelines
- Factory pattern with validation vs simple constructors
- When to use Go stdlib vs custom abstractions

**Documentation Update Scope**: All four architecture files need updates to reflect Epic 3 architectural decisions and Issue D1 findings.

##### 1.3 Is this a misunderstanding, missing consideration, or new information?

**Missing Consideration** (Documentation Debt)

- Not a misunderstanding - documentation was correct when written
- Not new information - just patterns not yet documented
- **Missing**: Systematic documentation updates after architectural decisions
- **Process Gap**: Epic 3 architectural questions were decided, but docs not updated to reflect decisions
- **Discovery**: Comprehensive architectural review revealed documentation debt accumulation
- **Classification**: Meta-issue - affects developer onboarding and consistency, not functional correctness

##### 1.4 What is the impact if we don't address this?

**Developer Guidance Impact**:

1. **Incorrect Implementations from Docs**:
   - Developers read high-level-architecture.md "Lean Domain Models: no behavior"
   - Implement new entities as anemic models (perpetuating Issue D1)
   - Documentation guides developers to build the anti-pattern we're trying to fix

2. **Inconsistent Pattern Application**:
   - No guidance on orchestration pattern selection → more god-objects like CLICommander
   - No singleton pattern docs → inconsistent Config/PropertyBank usage
   - No validation layer guidance → more IO in domain violations

3. **Onboarding Confusion**:
   - New developers can't understand architectural decisions
   - Why did we choose hybrid BoltDB + SQLite? (not documented)
   - When should entities have behavior vs stay anemic? (contradictory guidance)
   - What orchestration pattern should I use? (options not documented)

4. **Perpetuating Architectural Drift**:
   - Documentation doesn't reflect reality (components.md missing CLICommander, VaultIndexer)
   - Developers can't reference correct patterns when extending system
   - Gap between documented architecture and actual implementation widens

5. **Epic 5 Planning Blocked**:
   - Template struct design questions can't be resolved without documented pattern guidance
   - Anemic vs rich model decision for Template depends on documented principles
   - No foundation for making consistent architectural choices

6. **Code Review Challenges**:
   - Cannot reference docs to justify or challenge design decisions
   - Inconsistent standards across code reviews
   - Pattern violations can't be caught against documented standards

##### 1.5 What evidence supports this?

**Evidence from Documentation Files**:

1. **high-level-architecture.md Line 135 Contradicts Issue D1**:

```markdown
**Lean Domain Models:** Domain models contain only essential data with no behavior or infrastructure dependencies. Complex operations implemented in domain services. Models are pure data structures that can be easily serialized, tested, and composed.
```

- This principle ENDORSES anemic domain model
- Issue D1 (line 218) identifies anemic domain model as ❌ CRITICAL - PERVASIVE problem
- **Contradiction**: Documentation guides developers to create the anti-pattern

2. **Issue C3 - Documentation Misalignment** (line 285-296):

```markdown
#### Issue C3: Documentation Misalignment

- Architecture docs don't reflect:
  - CLIComander orchestration pattern
  - Singleton Config/PropertyBank
  - DTO architecture decisions
  - Schema-driven SQLite views
  - Hexagonal validation layers (syntactic vs semantic)
  - FrontmatterService refactoring
  - QueryService data contracts
- Impact: Developers implementing from docs build incorrect architecture
```

3. **Issue D3 - Missing Pattern Documentation** (line 258-267):

```markdown
#### Issue D3: Missing Pattern Documentation ⚠️ MODERATE

- **Problem**: Architecture docs don't specify when to use specific patterns
- **Missing Guidance**:
  - Event-driven vs orchestrator patterns
  - Unit of Work vs dual-write patterns
  - Factory pattern with validation vs simple constructors
  - Rich vs anemic model guidelines
  - Go's fs.FileInfo vs custom DTOs
- **Status**: Needs architecture documentation updates
```

4. **Groups 1-6 Discoveries Not in Docs**:
   - Group 1: Hexagonal validation layers - NOT documented
   - Group 2: Hybrid BoltDB + SQLite storage - NOT documented
   - Group 3: Event-driven vs orchestrator evaluation - NOT documented
   - Group 4: Singleton pattern (sync.Once) - NOT documented
   - Group 5: Port simplification principles - NOT documented
   - Group 6: Template struct design questions - NOT documented

5. **Epic 3 Sprint Change Proposal** (docs/course_correction/sprint-change-proposal-2025-11-02-epic3-hybrid-storage-architecture.md):
   - Six architectural questions answered (Questions 1-6)
   - Decisions made: Singleton pattern, config-driven schema selection, schema-driven views
   - **None documented** in architecture files

6. **Architectural Review Discovery** (this document):
   - 18+ architectural issues identified across 8 groups
   - Multiple architectural decisions and patterns discovered
   - Comprehensive analysis reveals documentation gaps

**Evidence of Developer Impact**:

- Template.go line 6: Comment says "follows the anemic domain model pattern" as if intentional
- Matches high-level-architecture.md guidance: "Lean Domain Models: no behavior"
- Developer was guided by docs to create anemic model

---

### Group 8: Implementation Blockers (3 issues - META)

**Issues**:

- **C1**: Multiple Questions Pending Implementation
- **C2**: Question 6 Unresolved
- **C3**: Documentation Misalignment

**Why Grouped**: Meta-issues about implementation state and process

##### 1.1 What triggered this change?

**Primary Trigger**: Epic 3 sprint change proposal (November 2, 2025) introduced six architectural questions requiring resolution before proceeding with implementation.

**Implementation Gap Discovery**: Comprehensive architectural review (Groups 1-7) revealed that architectural decisions have been made but not implemented, creating technical debt and blocking forward progress.

**Three Meta-Issues Identified**:

- **C1**: Questions 1-5 have decisions but no implementation (technical debt accumulation)
- **C2**: Question 6 (Write Coordination) remains unresolved (decision blocker)
- **C3**: Documentation doesn't reflect decisions or current state (guidance blocker)

**Epic 3 Progress Blocked**: Cannot proceed with Story 3.6+ until architecture corrected and implemented. Risk of building on flawed foundation compounds technical debt.

##### 1.2 What is the core issue?

**Three Interconnected Implementation Blockers**:

**Issue C1: Multiple Questions Pending Implementation** (line 273-277):

**Decisions Made But Not Implemented**:

1. **Question 1 (Orchestration)**: Decision reconsidered - was orchestrator, now evaluating event-driven
2. **Question 2 (Singleton)**: ✅ DECIDED (sync.Once pattern) - Implementation pending
3. **Question 3 (FileClassKey Config)**: ✅ DECIDED (config-driven) - ViperAdapter not loading FileClassKey
4. **Question 4 (DTO Architecture)**: ❌ UNRESOLVED - Needs fundamental redesign
5. **Question 5 (SQLite Schema)**: ✅ DECIDED (schema-driven views) - Implementation pending

**Impact**: Cannot proceed with Story 3.6+ until Questions 2, 3, 5 implemented and Questions 1, 4 resolved.

**Issue C2: Question 6 Unresolved** (line 279-283):

**Write Coordination Pattern Undecided**:

- **Problem**: BoltDB + SQLite dual-write coordination has no pattern decision
- **Options**: Unit of Work, Saga, dual-write with eventual consistency
- **Status**: No decision made - Story 3.2 technically incomplete
- **Blocker**: Story 3.6 (cache integration) blocked until write coordination decided

**Issue C3: Documentation Misalignment** (line 285-296 + Group 7):

**Architecture Docs Don't Reflect Reality**:

- CLICommander orchestration pattern - NOT documented
- Singleton Config/PropertyBank - NOT documented
- DTO architecture decisions - NOT documented
- Schema-driven SQLite views - NOT documented
- Hexagonal validation layers - NOT documented
- FrontmatterService refactoring - NOT documented
- QueryService data contracts - NOT documented

**Impact**: Developers implementing from docs build incorrect architecture (mismatch between docs and system state).

**Root Cause Pattern**: Architectural decisions made during sprint planning but not followed through with:

1. Implementation work
2. Documentation updates
3. Verification that decisions were correct

This creates cascading technical debt: incomplete implementations block new stories, undocumented decisions lead to inconsistent code, unresolved questions compound complexity.

##### 1.3 Is this a misunderstanding, missing consideration, or new information?

**Process Breakdown** (New Information About Development Process)

- Not a misunderstanding of requirements
- Not missing technical considerations
- **New Insight**: Process doesn't include systematic follow-through on architectural decisions
- **Pattern Recognition**: Sprint change proposal made decisions, but implementation and documentation lagged
- **Discovery**: Comprehensive architectural review reveals gap between decisions and reality
- **Root Issue**: Lack of architectural decision record (ADR) process or decision tracking

##### 1.4 What is the impact if we don't address this?

**Immediate Impact on Epic 3**:

1. **Story 3.6+ Blocked**:
   - Cannot implement cache integration without:
     - Question 2 implemented (Config singleton)
     - Question 3 implemented (FileClassKey loading)
     - Question 5 implemented (schema-driven views)
     - Question 6 decided (write coordination)
   - Technical debt from incomplete implementations will compound

2. **Building on Flawed Foundation**:
   - Continuing without resolving Issues D1, D2, B2 means:
     - More anemic entities (perpetuating anti-pattern)
     - More IO in domain violations
     - More god-objects from lack of orchestration pattern
   - Cost to fix increases exponentially with more code built on wrong patterns

3. **Epic 5 Blocked**:
   - Template struct design (Group 6) can't be decided without:
     - Documented anemic vs rich model guidelines (Issue D1)
     - Documented pattern selection principles (Issue D3)
   - Epic 5 planning cannot proceed

**Long-Term Impact**:

4. **Technical Debt Accumulation**:
   - Each story adds code following wrong patterns
   - Refactoring cost grows with codebase size
   - Eventually reaches "rewrite" threshold

5. **Inconsistent Architecture**:
   - Different parts of system follow different patterns
   - Developer confusion: "Which pattern should I follow?"
   - Code reviews can't enforce standards (no documented standards)

6. **Developer Onboarding**:
   - New developers read outdated docs
   - Implement anti-patterns documented as best practices
   - Perpetuate architectural problems

##### 1.5 What evidence supports this?

**Evidence from Issue Inventory**:

1. **Issue C1 - Implementation Pending** (line 273-277):

```markdown
#### Issue C1: Multiple Questions Pending Implementation

- Questions 1-5 have decisions but no implementation
- Cannot proceed with Story 3.6+ until architecture corrected
- Risk: Continuing on flawed foundation compounds debt
```

2. **Issue C2 - Question 6 Unresolved** (line 279-283):

```markdown
#### Issue C2: Question 6 Unresolved

- No decision on write coordination pattern
- BoltDB+SQLite integration incomplete
- Story 3.2 technically incomplete, Story 3.6 blocked
```

3. **Issue C3 - Documentation Misalignment** (line 285-296):

```markdown
#### Issue C3: Documentation Misalignment

- Architecture docs don't reflect:
  - CLIComander orchestration pattern
  - Singleton Config/PropertyBank
  - DTO architecture decisions
  - Schema-driven SQLite views
    [... 7 items total]
- Impact: Developers implementing from docs build incorrect architecture
```

**Evidence from Groups 1-7 Analysis**:

- Group 1: 3 critical issues (D1, B2, D1 validation) - NOT implemented
- Group 2: 5 storage issues - Questions 4, 6 unresolved
- Group 3: Orchestration pattern decided → failed → now undecided
- Group 4: Question 2, 3 decided but NOT implemented
- Group 5: Port simplification identified but NOT implemented
- Group 6: Template design unresolved → Epic 5 blocker
- Group 7: Documentation gaps comprehensive

**Evidence from Sprint Change Proposal**:

- Six questions posed (November 2, 2025)
- Decisions made for Questions 2, 3, 5
- Implementation status: NONE complete
- Documentation status: NONE documented
- Time elapsed: Analysis period shows gap between decision and action

**Evidence of Compound Risk**:

- 18+ architectural issues identified
- Multiple dependencies between issues
- Technical debt growing with each incomplete decision
- Epic 3 Story 3.6+ blocked by accumulated incomplete work

---

## Research Strategy

### Phase 1: Go Native Capabilities (Priority)

#### Go Standard Library Packages

##### io/fs

**Research Status**: ✅ Complete (2025-11-07)

**Purpose**: Address Group 2 DTO Architecture Redesign (Issues D2, A4) - Current custom VaultFile/FileMetadata DTOs duplicate os.FileInfo

**Core Interfaces**:

1. **fs.FileInfo** - Standard file metadata interface:

   ```go
   type FileInfo interface {
       Name() string       // base name of the file
       Size() int64        // length in bytes
       Mode() FileMode     // file mode bits
       ModTime() time.Time // modification time
       IsDir() bool        // abbreviation for Mode().IsDir()
       Sys() any           // underlying data source (can be nil)
   }
   ```

2. **fs.DirEntry** - Efficient directory traversal:

   ```go
   type DirEntry interface {
       Name() string                // Final path element only
       IsDir() bool                 // Whether entry is a directory
       Type() FileMode              // Type bits subset (fast)
       Info() (FileInfo, error)     // Full FileInfo (requires stat syscall)
   }
   ```

   - **Performance**: DirEntry-based `WalkDir` is ~1.5x faster than FileInfo-based `Walk` (370ms vs 580ms)
   - Use `Type()` for fast checks (no syscall), only call `Info()` when you need Size, ModTime, or Mode details

3. **fs.FS** - Abstract filesystem interface:

   ```go
   type FS interface {
       Open(name string) (File, error)
   }
   ```

   - Extension interfaces: ReadFileFS, StatFS, ReadDirFS, GlobFS

**Key Patterns**:

1. **Current Implementation** - FileMetadata with duplicated and computed fields:

   ```go
   // Current implementation from /internal/shared/dto/file.go
   type FileMetadata struct {
       Path     string    // Absolute path to file
       Basename string    // Filename without path and extension (computed)
       Folder   string    // Parent directory path (computed)
       Ext      string    // File extension including dot (computed)
       ModTime  time.Time // Modification timestamp (duplicates fs.FileInfo)
       Size     int64     // File size in bytes (duplicates fs.FileInfo)
       MimeType string    // MIME type (computed from Ext)
   }

   type VaultFile struct {
       FileMetadata // Embedded metadata
       Content []byte // Raw file content for indexing
   }

   // NOTE: Not all vault files are markdown with frontmatter
   // VaultFile represents ANY file in the vault (markdown, JSON, images, etc.)
   // Frontmatter and schema are separate domain concepts parsed from Content
   ```

2. **Potential Simplification** - Use fs.FileInfo directly instead of duplicating fields:

   ```go
   // Alternative: Eliminate FileMetadata by using fs.FileInfo
   type VaultFile struct {
       Path    string
       Info    fs.FileInfo  // Use interface directly (ModTime, Size, IsDir, Mode)
       Content []byte
   }

   // Compute path components on-demand instead of caching
   func (v VaultFile) Basename() string {
       base := filepath.Base(v.Path)
       return strings.TrimSuffix(base, filepath.Ext(base))
   }
   func (v VaultFile) Folder() string { return filepath.Dir(v.Path) }
   func (v VaultFile) Ext() string    { return filepath.Ext(v.Path) }
   ```

3. **Extension Pattern for Notes** - Use Sys() to provide note-specific metadata:

   ```go
   // For the subset of vault files that are notes with frontmatter
   type NoteFileInfo interface {
       fs.FileInfo
       FrontmatterData() map[string]any
       SchemaType() string
   }

   type noteFileInfo struct {
       fs.FileInfo              // Delegate to base FileInfo
       frontmatter map[string]any // Note-specific data
       schema      string         // Note-specific data
   }

   func (n *noteFileInfo) FrontmatterData() map[string]any { return n.frontmatter }
   func (n *noteFileInfo) SchemaType() string              { return n.schema }
   func (n *noteFileInfo) Sys() any                        { return n } // Enable type assertion

   // Usage: Type-assert Sys() to access note-specific data
   if noteInfo, ok := fileInfo.Sys().(*noteFileInfo); ok {
       frontmatter := noteInfo.FrontmatterData()
       // Process note-specific metadata
   }

   // Only applies to notes - other vault files use standard fs.FileInfo
   ```

4. **fs.WalkDir Pattern** - Efficient directory traversal:

   ```go
   err := fs.WalkDir(fsys, ".", func(path string, d fs.DirEntry, err error) error {
       if err != nil { return err }

       // Use Type() for fast checks (no syscall)
       if d.Type().IsRegular() {
           // Only call Info() when you need full metadata
           info, err := d.Info()
           if err != nil { return err }
           processFile(path, info)
       }

       // Skip directories by returning fs.SkipDir
       if d.IsDir() && shouldSkip(d.Name()) {
           return fs.SkipDir
       }

       return nil
   })
   ```

5. **Testing with fstest.MapFS** - In-memory filesystem for testing:
   ```go
   fsys := fstest.MapFS{
       "notes/test.md": &fstest.MapFile{
           Data: []byte("# Test Note"),
           Mode: 0644,
       },
   }
   ```

**Best Practices**:

- Use DirEntry, not FileInfo, for directory traversal (avoid syscalls)
- Leverage extension interfaces for optional functionality
- Define port interfaces using fs.FS, not os-specific types
- Error handling: Return `fs.SkipDir` to skip current directory, `fs.SkipAll` to skip all remaining
- Testing: Use fstest.MapFS for in-memory filesystem testing without disk access

**Options for VaultFile DTO**:

**Option A: Use fs.FileInfo directly, add domain metadata via extension interfaces**

- ✅ Maximum stdlib leverage
- ✅ No wrapper type needed
- ❌ Couples domain to stdlib types directly
- ❌ Extension interface pattern adds complexity

**Option B: Keep VaultFile but embed fs.FileInfo instead of duplicating fields**

- ✅ Preserves hexagonal architecture (domain entities)
- ✅ Eliminates field duplication (DRY)
- ✅ Allows domain-specific fields (Frontmatter, Schema)
- ✅ Easy to extend via extension interface pattern
- ✅ Testable with fstest.MapFS
- ❌ Additional wrapper layer

**Option C: Use fs.FileInfo in adapters, convert to domain entities at boundary**

- ✅ Clean hexagonal separation
- ✅ Domain fully decoupled from stdlib
- ❌ Conversion overhead at boundary
- ❌ Duplicates FileInfo fields in domain entity

**Port Interface Pattern**:

```go
// Port interface uses stdlib abstractions
type FileSystemPort interface {
    Open(name string) (fs.File, error)
    ReadFile(name string) ([]byte, error)
    WalkDir(root string, fn fs.WalkDirFunc) error
}

// Adapter implements with os package
type OSFileSystemAdapter struct {
    fsys fs.FS
}

func (a *OSFileSystemAdapter) WalkDir(root string, fn fs.WalkDirFunc) error {
    return fs.WalkDir(a.fsys, root, fn)
}
```

**Impact on Issues**:

- **Issue D2 (DTO Duplication)**: Embedding fs.FileInfo eliminates custom field duplication
- **Issue A4 (Port Boundaries)**: Enables clean port interfaces using stdlib abstractions

**Sources**:

- pkg.go.dev/io/fs
- benhoyt.com/writings/go-readdir (DirEntry performance benchmarks)

##### path/filepath

**Research Status**: ✅ Complete (2025-11-07)

**Purpose**: Cross-cutting - Path manipulation, directory traversal patterns, cross-platform path handling for vault operations

**Core Functions**:

1. **Directory Traversal**:

   ```go
   // Walk - Traverses file tree (legacy, less efficient)
   filepath.Walk(root, func(path string, info fs.FileInfo, err error) error {
       if err != nil { return err }
       // Process file
       return nil
   })

   // WalkDir - More efficient (Go 1.16+), avoids os.Lstat on every visited item
   filepath.WalkDir(root, func(path string, d fs.DirEntry, err error) error {
       if err != nil { return err }
       // Process file
       return nil
   })
   ```

   - **Critical**: "Walk reads an entire directory into memory before proceeding" - memory implications for large directory trees
   - **Critical**: "Walk does not follow symbolic links" - prevents infinite loops
   - **Best Practice**: Use `WalkDir()` over `Walk()` for better performance

2. **Path Manipulation**:

   ```go
   // Join - Combines path elements using OS-specific separator
   path := filepath.Join("vault", "notes", "meeting.md")
   // Result: "vault/notes/meeting.md" (Unix) or "vault\notes\meeting.md" (Windows)
   // "Empty elements are ignored. The result is Cleaned."

   // Clean - Lexically simplifies paths
   clean := filepath.Clean("vault/./notes/../notes/meeting.md")
   // Result: "vault/notes/meeting.md"

   // Abs - Returns absolute path
   abs, err := filepath.Abs("notes/meeting.md")
   // Combines with current working directory if relative

   // Rel - Computes relative path
   rel, err := filepath.Rel("/vault", "/vault/notes/meeting.md")
   // Result: "notes/meeting.md"

   // Split - Separates directory and filename
   dir, file := filepath.Split("/vault/notes/meeting.md")
   // dir: "/vault/notes/", file: "meeting.md"

   // Dir - Extract directory
   dir := filepath.Dir("/vault/notes/meeting.md")
   // Result: "/vault/notes"

   // Base - Extract filename
   base := filepath.Base("/vault/notes/meeting.md")
   // Result: "meeting.md"

   // Ext - File extension
   ext := filepath.Ext("meeting.md")
   // Result: ".md"
   ```

3. **Pattern Matching**:

   ```go
   // Match - Shell-style pattern matching
   matched, err := filepath.Match("*.md", "meeting.md")
   // Supports *, ?, and character classes [...]

   // Glob - Returns all filenames matching pattern
   matches, err := filepath.Glob("vault/notes/*.md")
   // Ignores I/O errors, can be expensive for broad patterns
   ```

**Key Differences from `path` Package**:

| Feature       | `path` Package                 | `filepath` Package                  |
| ------------- | ------------------------------ | ----------------------------------- |
| **Purpose**   | URL-like forward-slash paths   | OS-specific filesystem paths        |
| **Separator** | Always `/`                     | OS-specific (`/` Unix, `\` Windows) |
| **Use Case**  | URLs, portable data structures | Filesystem operations               |
| **Platform**  | Platform-independent           | Platform-specific                   |

**Guideline**: Use `filepath` for filesystem operations, `path` for URLs or portable structures.

**OS-Specific Behavior**:

1. **Separators**:

   ```go
   // OS-specific path separator
   filepath.Separator // '/' on Unix, '\' on Windows

   // Environment variable separator
   filepath.ListSeparator // ':' on Unix, ';' on Windows
   ```

2. **Windows-Specific Handling**:
   - Volume names recognized (e.g., `C:`, UNC paths `\\host\share`)
   - "On Windows, escaping is disabled. Instead, '\\' is treated as path separator."
   - Reserved names like "NUL" handled via `IsLocal()`

3. **Cross-Platform Conversion**:

   ```go
   // ToSlash - Converts separators to forward slashes
   portable := filepath.ToSlash(`vault\notes\meeting.md`)
   // Result: "vault/notes/meeting.md"

   // FromSlash - Converts forward slashes to OS separators
   native := filepath.FromSlash("vault/notes/meeting.md")
   // Result: "vault/notes/meeting.md" (Unix) or "vault\notes\meeting.md" (Windows)

   // Localize - Converts slash-separated io/fs paths to OS paths (Go 1.20+)
   local := filepath.Localize("vault/notes/meeting.md")
   ```

**Best Practices for Cross-Platform Development**:

1. **Always use `Join()`** instead of string concatenation:

   ```go
   // ❌ BAD: String concatenation
   path := "vault" + "/" + "notes" + "/" + "meeting.md"

   // ✅ GOOD: filepath.Join
   path := filepath.Join("vault", "notes", "meeting.md")
   ```

2. **Validate paths** with `IsLocal()` to prevent directory traversal attacks:

   ```go
   if !filepath.IsLocal(userPath) {
       return errors.New("invalid path: directory traversal attempt")
   }
   ```

3. **Use `WalkDir()` over `Walk()`** for better performance (Go 1.16+)

4. **Handle `SkipDir` and `SkipAll`** properly:

   ```go
   filepath.WalkDir(root, func(path string, d fs.DirEntry, err error) error {
       if d.IsDir() && shouldSkip(d.Name()) {
           return filepath.SkipDir  // Skip this directory
       }
       return nil
   })
   ```

5. **Convert between formats** when interfacing with portable formats:

   ```go
   // Store paths in portable format (forward slashes)
   stored := filepath.ToSlash(nativePath)

   // Convert back to native format when using
   native := filepath.FromSlash(stored)
   ```

6. **Test on target platforms** - behavior differs significantly between Unix and Windows

**Common Gotchas**:

1. **`Clean()` returns `"."` for empty input, not empty string**:

   ```go
   clean := filepath.Clean("")
   // Result: "." (not "")
   ```

2. **Memory implications for large directories**:
   - "Walk reads an entire directory into memory before proceeding"
   - Use early filtering with `SkipDir` to reduce memory usage

3. **Trailing separators have semantic meaning**:

   ```go
   dir := filepath.Dir("/vault/notes/")
   // Result: "/vault/notes" (removes trailing separator)

   dir, file := filepath.Split("/vault/notes/")
   // dir: "/vault/notes/", file: "" (preserves trailing separator)
   ```

4. **`IsAbs()` behavior differs by platform**:

   ```go
   // Unix: /path is absolute
   filepath.IsAbs("/vault") // true on Unix

   // Windows: /path is relative without drive letter
   filepath.IsAbs("/vault") // false on Windows (no drive letter)
   filepath.IsAbs("C:/vault") // true on Windows
   ```

5. **`HasPrefix()` is deprecated**:
   - "HasPrefix does not respect path boundaries and does not ignore case when required"
   - Use `strings.HasPrefix(filepath.Clean(path), filepath.Clean(prefix)+string(filepath.Separator))` instead

**Performance Considerations**:

1. **WalkDir efficiency**: "Walk is less efficient than WalkDir, introduced in Go 1.16, which avoids calling os.Lstat on every visited file or directory"

2. **Memory usage**: Complete directory listing loaded before traversal begins

3. **Sorting overhead**: "Lexical order...requires Walk to read an entire directory into memory"

4. **Symbolic link handling**: "Walk does not follow symbolic links," preventing infinite loops but may miss linked content

5. **Pattern matching**: `Glob()` can be expensive for broad patterns across large filesystems

**Lithos Application Patterns**:

1. **Vault Path Construction**:

   ```go
   // Always use Join for cross-platform compatibility
   notePath := filepath.Join(vaultPath, "notes", "meeting.md")
   schemaPath := filepath.Join(vaultPath, "schemas", "note.json")
   templatePath := filepath.Join(vaultPath, "templates", "meeting.tmpl")
   ```

2. **Vault Scanning with WalkDir**:

   ```go
   func ScanVault(vaultPath string) error {
       return filepath.WalkDir(vaultPath, func(path string, d fs.DirEntry, err error) error {
           if err != nil { return err }

           // Skip hidden directories
           if d.IsDir() && strings.HasPrefix(d.Name(), ".") {
               return filepath.SkipDir
           }

           // Process only markdown files
           if !d.IsDir() && filepath.Ext(path) == ".md" {
               processNote(path)
           }

           return nil
       })
   }
   ```

3. **Portable Path Storage**:

   ```go
   // Store paths in portable format (forward slashes)
   type NoteMetadata struct {
       Path string // Always stored with forward slashes
   }

   func NewNoteMetadata(nativePath string) *NoteMetadata {
       return &NoteMetadata{
           Path: filepath.ToSlash(nativePath),
       }
   }

   func (m *NoteMetadata) NativePath() string {
       return filepath.FromSlash(m.Path)
   }
   ```

4. **Relative Path Computation**:

   ```go
   // Compute relative path from vault root
   func RelativeToVault(vaultPath, notePath string) (string, error) {
       rel, err := filepath.Rel(vaultPath, notePath)
       if err != nil {
           return "", err
       }
       // Store in portable format
       return filepath.ToSlash(rel), nil
   }
   ```

5. **Security Validation**:

   ```go
   // Prevent directory traversal attacks
   func ValidateVaultPath(vaultPath, userPath string) error {
       // Ensure path is local (no .., etc.)
       if !filepath.IsLocal(userPath) {
           return errors.New("invalid path: directory traversal attempt")
       }

       // Ensure path is within vault
       fullPath := filepath.Join(vaultPath, userPath)
       relPath, err := filepath.Rel(vaultPath, fullPath)
       if err != nil || strings.HasPrefix(relPath, "..") {
           return errors.New("path outside vault")
       }

       return nil
   }
   ```

**Options for Lithos Path Handling**:

**Option A: Store native paths everywhere**

- ✅ Simple (no conversion)
- ❌ Not portable (breaks when sharing vaults across platforms)
- ❌ Cannot serialize vault metadata portably

**Option B: Store portable paths (forward slashes), convert at boundaries**

- ✅ Portable (vault metadata works cross-platform)
- ✅ Clean storage format
- ❌ Conversion overhead at filesystem operations
- ✅ **Best for Lithos**: Vaults may be synced across platforms

**Option C: Use io/fs paths everywhere, convert only for legacy code**

- ✅ Modern Go approach (io/fs uses forward slashes)
- ✅ Works with io/fs abstractions
- ❌ Conversion needed for os package operations

**Impact on Issues**:

- **Cross-Cutting**: Cross-platform path handling for vault operations
- **Group 2 (Storage)**: Path normalization for storage layer
- **Issue A4 (Port Boundaries)**: Path handling in FileSystemPort

**Sources**:

- pkg.go.dev/path/filepath

##### text/template

**Research Status**: ✅ Complete (2025-11-07)

**Purpose**: Address Group 6 Template System (Epic 5 Blocker) - Current domain.Template wrapper immediately converted to \*template.Template, questions about necessity and composition support

**Core Patterns**:

1. **Template Composition** - `{{template}}` vs `{{block}}`:

   ```go
   // {{template}} - Execute associated template with data
   {{define "header"}}
   <h1>{{.Title}}</h1>
   {{end}}

   {{template "header" .}}  // With data pipeline
   {{template "header"}}    // With nil data

   // {{block}} - Shorthand for define + execute in place
   {{block "content" .}}
     Default content shown if "content" not redefined
   {{end}}
   ```

   - **Use Case**: Blocks enable template inheritance (root defines blocks, children redefine them)

2. **ParseFiles with Multiple Templates**:

   ```go
   // ParseFiles creates template namespace
   tmpl, err := template.ParseFiles("base.tmpl", "page1.tmpl", "page2.tmpl")

   // Template name = first file's basename ("base")
   // Associated templates: "page1", "page2"
   // CRITICAL: Last file with same basename wins
   ```

3. **Execute vs ExecuteTemplate**:

   ```go
   // Execute: Apply the template directly (single template or default)
   tmpl.Execute(w, data)

   // ExecuteTemplate: Apply named associated template
   tmpl.ExecuteTemplate(w, "page1", data)

   // Internally: ExecuteTemplate calls Lookup("page1").Execute(w, data)
   ```

4. **Function Maps** - Registration BEFORE parsing:

   ```go
   // CRITICAL: Funcs MUST be called BEFORE Parse or it panics
   funcMap := template.FuncMap{
       "upper": strings.ToUpper,
       "formatDate": func(t time.Time) string { return t.Format("2006-01-02") },
   }

   tmpl := template.New("base").
       Funcs(funcMap).              // Register functions first
       ParseFiles("base.tmpl")      // Then parse
   ```

5. **Template Lookup and Dependency Resolution**:

   ```go
   // Lookup retrieves associated template by name
   tmpl.Lookup("header") // Returns *template.Template or nil

   // Check if template exists before executing
   if headerTmpl := tmpl.Lookup("header"); headerTmpl != nil {
       headerTmpl.Execute(w, data)
   }

   // Walk associated templates
   tmpl.Templates() // Returns []*template.Template slice
   ```

   - **Note**: No built-in dependency tracking for `{{template}}` references

6. **Template Caching**:

   ```go
   var (
       tmplCache = make(map[string]*template.Template)
       tmplMutex sync.RWMutex
   )

   func GetTemplate(name string) (*template.Template, error) {
       tmplMutex.RLock()
       cached, ok := tmplCache[name]
       tmplMutex.RUnlock()

       if ok {
           return cached, nil
       }

       // Double-check locking pattern
       tmplMutex.Lock()
       defer tmplMutex.Unlock()

       if cached, ok := tmplCache[name]; ok {
           return cached, nil
       }

       tmpl, err := template.ParseFiles(name)
       if err != nil {
           return nil, err
       }
       tmplCache[name] = tmpl
       return tmpl, nil
   }
   ```

   - **Performance**: Cached templates ~6.8x faster (1173 ns/op vs 7955 ns/op)

7. **Pre-Execution Validation**:

   ```go
   // Validation occurs at Parse time (syntax errors)
   tmpl, err := template.New("test").Parse(`{{.InvalidSyntax}`)
   if err != nil {
       // Syntax error caught here
   }

   // Execution errors (missing data, nil pointers) caught at Execute time
   err = tmpl.Execute(w, data)
   if err != nil {
       // Runtime error (e.g., nil pointer, missing field)
   }

   // No built-in pre-execution validation (can't validate without data)
   // Workaround: Execute with mock data
   testData := mockDataForValidation()
   var buf bytes.Buffer
   if err := tmpl.Execute(&buf, testData); err != nil {
       // Template logic errors
   }
   ```

**Best Practices**:

- Register functions BEFORE parsing: `Funcs()` MUST be called before `Parse()` or it panics
- Use blocks for template inheritance: Define base templates with `{{block}}`, override in child templates with `{{define}}`
- Cache parsed templates: Parsing is expensive (~6.8x slower), parse once on startup, reuse
- Thread-safe execution: Templates can be executed safely in parallel, but use mutex for cache writes
- Validate syntax at parse time: Catch syntax errors early with `Parse()`, but execution errors require runtime data
- Use Lookup for conditional execution: Check if named template exists before executing

**Options for Template Architecture**:

**Option A: Use \*template.Template directly in domain**

```go
type TemplateEngine interface {
    LoadTemplate(id TemplateID) (*template.Template, error)
    Execute(tmpl *template.Template, data any) (string, error)
}
```

- ✅ Simple, no conversion overhead
- ❌ Couples domain to stdlib
- ❌ Hard to test (need real templates)

**Option B: Keep domain.Template as metadata, adapter holds \*template.Template**

```go
// Domain: Metadata only
type TemplateMetadata struct {
    ID       TemplateID
    Name     string
    FilePath string
}

// Adapter: Holds parsed template
type TemplateEngine struct {
    cache map[TemplateID]*template.Template
}
```

- ✅ Clean domain/adapter separation
- ✅ Testable (mock adapter)
- ✅ Cache lives in adapter layer
- ❌ More indirection

**Option C: Template as interface (adapter wraps \*template.Template)**

```go
// Domain: Interface
type Template interface {
    ID() TemplateID
    Execute(data any) (string, error)
}

// Adapter: Wraps *template.Template
type GoTemplate struct {
    id   TemplateID
    tmpl *template.Template
}

func (g *GoTemplate) Execute(data any) (string, error) {
    var buf bytes.Buffer
    err := g.tmpl.Execute(&buf, data)
    return buf.String(), err
}
```

- ✅ Domain interface, not concrete type
- ✅ Testable (mock Template interface)
- ✅ Type-safe TemplateID
- ✅ Encapsulates \*template.Template
- ❌ Wrapper object per template

**Composition Support Patterns**:

1. **Parse All Templates Together**:

   ```go
   // Adapter initialization
   func NewGoTemplateAdapter(templateDir string, funcMap template.FuncMap) (*GoTemplateAdapter, error) {
       adapter := &GoTemplateAdapter{
           templates: make(map[TemplateID]*template.Template),
           funcMap:   funcMap,
       }

       // Parse all templates with shared function map
       baseTmpl := template.New("base").Funcs(funcMap)
       tmpl, err := baseTmpl.ParseGlob(filepath.Join(templateDir, "*.tmpl"))
       if err != nil {
           return nil, err
       }

       // Register each template by ID
       for _, t := range tmpl.Templates() {
           id := TemplateID(t.Name())
           adapter.templates[id] = t
       }

       return adapter, nil
   }
   ```

2. **Dependency Resolution Options**:
   - **Convention-based**: Base template named "base.tmpl", children invoke via `{{template "base"}}`
   - **Parse template text**: Extract `{{template "name"}}` references (no built-in API)
   - **Manual registration**: Define template dependencies in metadata

**Impact on Issues**:

- **Group 6 (Template System)**: Resolves "do we need domain.Template?" question
- **Epic 5 Blocker**: Composition patterns enable template inheritance for Epic 5
- **Issue D1 (Anemic Models)**: Template interface pattern adds behavior to domain entity

**Sources**:

- pkg.go.dev/text/template
- camlittle.com/go-template-validation

#### Go Third-Party Packages

##### bbolt

**Research Status**: ✅ Complete (2025-11-07)

**Purpose**: Address Group 2 BoltDB Hot Cache Storage - Planning BoltDB for <1ms hot cache, need transaction patterns, bucket design, performance best practices

**Core Transaction Patterns**:

1. **View (Read-Only) - Concurrent, No Locking**:

   ```go
   // View executes a function within a read-only transaction
   err := db.View(func(tx *bolt.Tx) error {
       b := tx.Bucket([]byte("notes"))
       v := b.Get([]byte("note-123"))
       // Process value
       return nil
   })

   // Multiple read-only transactions can run concurrently
   // Any error returned from function is returned from View()
   // Attempting to manually rollback causes panic
   ```

2. **Update (Read-Write) - Exclusive Lock**:

   ```go
   // Update executes a function within a read-write transaction
   err := db.Update(func(tx *bolt.Tx) error {
       b := tx.Bucket([]byte("notes"))
       return b.Put([]byte("note-123"), []byte("content"))
   })

   // If function returns nil, transaction commits
   // If function returns error, transaction rolls back
   // Only ONE read-write transaction allowed at a time
   // Attempting to manually commit/rollback causes panic
   ```

3. **Batch (Opportunistic Batching) - Best for Many Small Writes**:

   ```go
   // Batch opportunistically combines multiple writes
   // Function may execute MULTIPLE times, so must be idempotent
   err := db.Batch(func(tx *bolt.Tx) error {
       b := tx.Bucket([]byte("notes"))
       return b.Put([]byte("note-123"), []byte("content"))
   })

   // Sacrifices atomicity guarantees for performance
   // Go can call function multiple times if batching with other writes
   // Use for high-write-throughput scenarios
   ```

4. **Manual Transaction Control**:

   ```go
   // For complex transaction logic
   tx, err := db.Begin(true) // true = writable
   if err != nil {
       return err
   }
   defer tx.Rollback() // Safe to call even if Commit succeeds

   b := tx.Bucket([]byte("notes"))
   // Complex operations...

   if err := tx.Commit(); err != nil {
       return err // Rollback called by defer
   }
   ```

**Bucket Design Patterns**:

1. **Nested Buckets (Hierarchical Data)**:

   ```go
   // Create nested bucket structure
   err := db.Update(func(tx *bolt.Tx) error {
       // Create root bucket
       root, err := tx.CreateBucketIfNotExists([]byte("vaults"))
       if err != nil {
           return err
       }

       // Create nested bucket for specific vault
       vaultBucket, err := root.CreateBucketIfNotExists([]byte("vault-123"))
       if err != nil {
           return err
       }

       // Store note in vault's bucket
       return vaultBucket.Put([]byte("note-456"), []byte("content"))
   })

   // Access nested bucket
   err := db.View(func(tx *bolt.Tx) error {
       root := tx.Bucket([]byte("vaults"))
       vaultBucket := root.Bucket([]byte("vault-123"))
       if vaultBucket == nil {
           return ErrVaultNotFound
       }
       v := vaultBucket.Get([]byte("note-456"))
       return nil
   })
   ```

2. **Flat Buckets with Composite Keys**:

   ```go
   // Use key namespacing instead of nested buckets
   func compositeKey(vaultID, noteID string) []byte {
       return []byte(vaultID + ":" + noteID)
   }

   err := db.Update(func(tx *bolt.Tx) error {
       b := tx.Bucket([]byte("notes"))
       key := compositeKey("vault-123", "note-456")
       return b.Put(key, []byte("content"))
   })

   // Range scan by prefix
   err := db.View(func(tx *bolt.Tx) error {
       c := tx.Bucket([]byte("notes")).Cursor()
       prefix := []byte("vault-123:")

       for k, v := c.Seek(prefix); k != nil && bytes.HasPrefix(k, prefix); k, v = c.Next() {
           // Process vault's notes
       }
       return nil
   })
   ```

**Decision Criteria**:

- **Nested buckets**: Small cardinality, logical hierarchy, per-bucket operations (delete all notes in vault)
- **Flat with composite keys**: Large cardinality, prefix scans, simpler structure

**Cursor Navigation for Range Queries**:

```go
// Cursor methods: First(), Last(), Next(), Prev(), Seek(key)
err := db.View(func(tx *bolt.Tx) error {
    c := tx.Bucket([]byte("notes")).Cursor()

    // Iterate all keys
    for k, v := c.First(); k != nil; k, v = c.Next() {
        processNote(k, v)
    }

    // Seek to specific key and iterate forward
    for k, v := c.Seek([]byte("2025-01")); k != nil; k, v = c.Next() {
        if !bytes.HasPrefix(k, []byte("2025-01")) {
            break
        }
        processNote(k, v)
    }

    return nil
})
```

**Critical Caveat**: "Changing data while traversing with a cursor may cause it to be invalidated." Removing key/value pairs during iteration may skip entries. Reposition cursor after mutations.

**Key Design Best Practices**:

1. **Integer IDs (Fixed-Size, Sortable)**:

   ```go
   // Auto-generate integer IDs
   err := db.Update(func(tx *bolt.Tx) error {
       b := tx.Bucket([]byte("notes"))

       // NextSequence returns auto-incrementing ID
       id, _ := b.NextSequence()

       // Encode as big-endian for byte-sortable storage
       key := make([]byte, 8)
       binary.BigEndian.PutUint64(key, id)

       return b.Put(key, []byte("content"))
   })
   ```

2. **RFC3339 Timestamps for Time-Based Keys**:

   ```go
   // RFC3339 format is byte-comparable
   key := []byte(time.Now().Format(time.RFC3339) + ":note-123")
   // Enables range queries by date
   ```

3. **Composite Keys for Relationships**:

   ```go
   // Pattern: <parentID>:<childID>
   func compositeKey(vaultID, noteID string) []byte {
       return []byte(fmt.Sprintf("%s:%s", vaultID, noteID))
   }

   // Enables prefix scans for all children of parent
   ```

**Secondary Indices Pattern**:

```go
// Primary bucket: ID -> Data
// Index bucket: IndexKey -> ID

err := db.Update(func(tx *bolt.Tx) error {
    notes := tx.Bucket([]byte("notes"))
    titleIndex := tx.Bucket([]byte("notes:by_title"))

    noteID := []byte("note-123")
    noteData := []byte(`{"title":"My Note","content":"..."}`)

    // Write to primary bucket
    if err := notes.Put(noteID, noteData); err != nil {
        return err
    }

    // Update index
    title := []byte("My Note")
    return titleIndex.Put(title, noteID)
})

// Query by index
err := db.View(func(tx *bolt.Tx) error {
    titleIndex := tx.Bucket([]byte("notes:by_title"))
    noteID := titleIndex.Get([]byte("My Note"))
    if noteID == nil {
        return ErrNotFound
    }

    notes := tx.Bucket([]byte("notes"))
    noteData := notes.Get(noteID)
    return nil
})
```

**Index Maintenance**: Manually update indices on PUT/DELETE. Consider using triggers or middleware pattern.

**Best Practices**:

- **Read-Heavy Workloads**: BoltDB excels at reads with lock-free MVCC. Multiple concurrent read transactions.
- **Batch Small Writes**: Use `Batch()` for many small writes to amortize fsync overhead.
- **Avoid Long-Running Read Transactions**: Blocks copy-on-write reclamation, increasing DB size.
- **Close Transactions Promptly**: Use defer for cleanup.
- **Keys and Values Valid Only During Transaction**: Copy data with `copy()` if needed beyond transaction scope.
- **Bulk Inserts**: Stay below ~100,000 pairs per transaction for new buckets to avoid performance degradation.
- **Space Efficiency**: Small buckets (<4KB) use 12KB (3 pages). Space efficiency improves with larger datasets.
- **SSD Recommended**: BoltDB uses B+tree with random page access. SSDs significantly outperform HDDs.
- **Error Handling**: Always check errors from bucket operations. Buckets can be nil if not found.

**Options for Hot Cache Bucket Structure**:

**Option A: Nested Buckets (Logical Hierarchy)**:

```
Root: vaults
  → <vaultID>
    → files (Key: <filepath> → Value: VaultFile JSON)
    → indices
      → by_title (Key: <title> → Value: <filepath>) -- Only for notes with frontmatter
      → by_tag (Key: <tag> → Value: JSON array of filepaths) -- Only for notes with tags
```

NOTE: Not all files are notes with frontmatter. The `files` bucket stores all vault files. Indices only contain entries for files that have the relevant metadata (e.g., only notes with titles appear in by_title index).

- ✅ Logical hierarchy (vaults → notes)
- ✅ Per-vault operations (delete all notes in vault)
- ✅ Isolation (each vault's data isolated)
- ✅ Small-medium cardinality (not millions of vaults)
- ✅ <1ms target achievable with B+tree
- ❌ More bucket management overhead

**Option B: Flat Buckets with Composite Keys**:

```
Bucket: files
  Key: "vault-123:path/to/file.md" → Value: VaultFile JSON
Bucket: indices:by_title
  Key: "vault-123:My Note" → Value: "path/to/file.md" -- Only notes with titles
Bucket: indices:by_tag
  Key: "vault-123:tag-name" → Value: JSON array of filepaths -- Only notes with tags
```

NOTE: The `files` bucket contains ALL vault files. Index buckets only contain entries for files with relevant metadata.

- ✅ Simpler structure
- ✅ Prefix scans across all vaults
- ✅ Large cardinality (millions of entities)
- ❌ No per-vault isolation
- ❌ Harder to delete all notes for a vault

**Performance Characteristics**:

- **View() latency**: Sub-millisecond for cached keys (<1ms target achievable)
- **Update() latency**: 1-10ms (depends on fsync)
- **Batch() throughput**: 10,000+ writes/second for small values
- **Cursor iteration**: Fast sequential access (B+tree leaf pages)

**Impact on Issues**:

- **Group 2 (Storage Architecture)**: Provides <1ms hot cache architecture
- **Issue A5 (Storage Layer)**: BoltDB component for hybrid storage (BoltDB + SQLite)

**Sources**:

- pkg.go.dev/go.etcd.io/bbolt
- github.com/boltdb/bolt/issues/293 (performance discussions)

##### sqlite (modernc.org/sqlite)

**Research Status**: ✅ Complete (2025-11-07)

**Purpose**: Address Group 2 SQLite Deep Storage with Schema-Driven Views - Planning SQLite for <50ms deep storage, need JSON patterns, schema-driven views, query optimization, pure Go benefits

**modernc.org/sqlite vs mattn/go-sqlite3**:

| Feature               | modernc.org/sqlite              | mattn/go-sqlite3            |
| --------------------- | ------------------------------- | --------------------------- |
| **Architecture**      | Pure Go translation of SQLite C | CGo wrapper around SQLite C |
| **Cross-Compilation** | ✅ Trivial (no C compiler)      | ❌ Requires C toolchain     |
| **Build Complexity**  | ✅ Simple (no CGo)              | ❌ CGo dependencies         |
| **Distribution**      | ✅ Single binary                | ❌ C toolchain required     |
| **Performance**       | ❌ 10%-100% slower              | ✅ Native C speed           |
| **INSERT-heavy**      | ❌ ~2x slower                   | ✅ Faster                   |
| **SELECT**            | ❌ 10%-50% slower               | ✅ Faster                   |

**When to Use modernc.org/sqlite**:

- ✅ Cross-platform distribution (simple builds)
- ✅ Read-heavy workloads (acceptable performance)
- ✅ Small-medium datasets (tradeoff acceptable)
- ❌ Avoid for: Write-heavy, performance-critical paths

**Lithos Use Case**: Read-heavy deep storage (<50ms target) → modernc.org/sqlite acceptable

**JSON Column Patterns**:

1. **json_extract() - Direct JSON Extraction**:

   ```sql
   -- Store JSON in TEXT column (nullable - not all files have frontmatter)
   CREATE TABLE files (
       id TEXT PRIMARY KEY,
       path TEXT NOT NULL,
       content BLOB,
       frontmatter TEXT -- JSON column, NULL for files without frontmatter
   );

   -- Extract JSON fields in queries (handle NULL frontmatter)
   SELECT id, json_extract(frontmatter, '$.title') AS title
   FROM files
   WHERE frontmatter IS NOT NULL
     AND json_extract(frontmatter, '$.status') = 'published';
   ```

   NOTE: Not all vault files have frontmatter. The `frontmatter` column is nullable and queries must handle NULL values.
   - **Problem**: `json_extract()` parses JSON on every call → slow for 100k+ records

2. **VIRTUAL Generated Columns** (Computed at Read Time):

   ```sql
   -- Add virtual column extracting JSON field (handles NULL frontmatter)
   ALTER TABLE files
   ADD COLUMN title TEXT AS (json_extract(frontmatter, '$.title'));

   -- Index the virtual column (avoids repeated json_extract)
   -- NULL titles (files without frontmatter) are excluded from index
   CREATE INDEX idx_files_title ON files(title) WHERE title IS NOT NULL;

   -- Query uses index
   SELECT * FROM files WHERE title = 'My Note';
   ```

3. **STORED Generated Columns** (Cached at Write Time):

   ```sql
   -- STORED columns cache computed values (more space, faster reads)
   -- Handles NULL frontmatter gracefully (json_extract returns NULL for NULL input)
   CREATE TABLE files (
       id TEXT PRIMARY KEY,
       frontmatter TEXT, -- NULL for files without frontmatter
       title TEXT GENERATED ALWAYS AS (json_extract(frontmatter, '$.title')) STORED,
       status TEXT GENERATED ALWAYS AS (json_extract(frontmatter, '$.status')) STORED
   );

   -- Index stored columns (exclude NULL values)
   CREATE INDEX idx_files_title ON files(title) WHERE title IS NOT NULL;
   CREATE INDEX idx_files_status ON files(status) WHERE status IS NOT NULL;
   ```

**VIRTUAL vs STORED Comparison**:

| Feature       | VIRTUAL                       | STORED                          |
| ------------- | ----------------------------- | ------------------------------- |
| Storage       | Not stored (computed on read) | Stored (cached value)           |
| Write Speed   | Faster (no computation)       | Slower (compute + store)        |
| Read Speed    | Slower (compute each time)    | Faster (read cached value)      |
| Disk Usage    | Lower                         | Higher                          |
| Index Support | ✅ Yes                        | ✅ Yes                          |
| ALTER TABLE   | ✅ Yes                        | ❌ No (must be in CREATE TABLE) |

4. **Functional Indexes on JSON**:

   ```sql
   -- Index JSON extraction directly (skip generated column)
   -- Partial index excludes NULL frontmatter
   CREATE INDEX idx_files_title ON files(json_extract(frontmatter, '$.title'))
   WHERE frontmatter IS NOT NULL;

   -- Query MUST use exact expression for index to apply
   SELECT * FROM files
   WHERE json_extract(frontmatter, '$.title') = 'My Note';
   ```

   - **Caveat**: Must use exact `json_extract()` expression in WHERE clause for index to apply

5. **json_each() for Array Queries**:

   ```sql
   -- Query JSON arrays (only for files with frontmatter and tags)
   SELECT DISTINCT files.id, json_extract(files.frontmatter, '$.title') AS title
   FROM files, json_each(files.frontmatter, '$.tags') AS tag
   WHERE files.frontmatter IS NOT NULL
     AND tag.value = 'golang';

   -- Extract array elements to rows
   SELECT json_each.value AS tag
   FROM files, json_each(files.frontmatter, '$.tags')
   WHERE files.id = 'note-123'
     AND files.frontmatter IS NOT NULL;
   ```

**Schema-Driven Views Pattern**:

```sql
-- Base table stores all vault files (not just notes)
CREATE TABLE files (
    id TEXT PRIMARY KEY,
    vault_id TEXT NOT NULL,
    path TEXT NOT NULL,
    content BLOB,
    frontmatter TEXT, -- JSON, NULL for files without frontmatter

    -- VIRTUAL columns for universal fields (NULL for files without frontmatter)
    schema_type TEXT AS (json_extract(frontmatter, '$.schema')),
    title TEXT AS (json_extract(frontmatter, '$.title')),
    created_at INTEGER AS (json_extract(frontmatter, '$.created_at')),
    updated_at INTEGER AS (json_extract(frontmatter, '$.updated_at'))
);

-- Partial indexes on VIRTUAL columns (exclude NULL values)
CREATE INDEX idx_files_schema ON files(schema_type) WHERE schema_type IS NOT NULL;
CREATE INDEX idx_files_title ON files(title) WHERE title IS NOT NULL;

-- Schema-driven view for "meeting" schema (only files with meeting schema)
CREATE VIEW files_meeting AS
SELECT
    id,
    vault_id,
    path,
    title,
    json_extract(frontmatter, '$.date') AS meeting_date,
    json_extract(frontmatter, '$.attendees') AS attendees,
    json_extract(frontmatter, '$.agenda') AS agenda,
    json_extract(frontmatter, '$.status') AS status
FROM files
WHERE schema_type = 'meeting';

-- Functional index for hot queries (partial index for meeting schema only)
CREATE INDEX idx_meeting_date ON files(
    (json_extract(frontmatter, '$.date'))
) WHERE schema_type = 'meeting';

-- Schema-driven view for "person" schema (only files with person schema)
CREATE VIEW files_person AS
SELECT
    id,
    vault_id,
    path,
    title,
    json_extract(frontmatter, '$.email') AS email,
    json_extract(frontmatter, '$.company') AS company,
    json_extract(frontmatter, '$.role') AS role
FROM files
WHERE schema_type = 'person';
```

NOTE: Views automatically filter to only files with the relevant schema_type. Files without frontmatter or with different schemas are excluded.

**Go Usage with modernc.org/sqlite**:

```go
import (
    "database/sql"
    _ "modernc.org/sqlite"
)

// Open connection with URI parameters
dsn := "file:notes.db?_pragma=journal_mode(WAL)&_pragma=foreign_keys(1)"
db, err := sql.Open("sqlite", dsn)
if err != nil {
    return err
}

// Set connection pool settings
db.SetMaxOpenConns(1) // SQLite: single writer, multiple readers
db.SetMaxIdleConns(1)
db.SetConnMaxLifetime(0)

// Prepared statements for performance
stmt, err := db.Prepare(`
    SELECT id, json_extract(frontmatter, '$.title') AS title
    FROM files
    WHERE frontmatter IS NOT NULL
      AND json_extract(frontmatter, '$.status') = ?
`)
defer stmt.Close()

rows, err := stmt.Query("published")
defer rows.Close()

for rows.Next() {
    var id, title string
    rows.Scan(&id, &title)
    // Process row
}
```

**DSN Parameters**:

- `_pragma`: Execute PRAGMA statements (e.g., `journal_mode(WAL)`, `foreign_keys(1)`)
- `_time_format`: Time serialization format (default: "sqlite")
- `_txlock`: Transaction lock mode (deferred/immediate/exclusive)

**Connection Pooling**: SQLite allows multiple readers but single writer. Set `MaxOpenConns=1` for write-heavy workloads or use WAL mode for concurrent reads.

**Best Practices**:

- **Use VIRTUAL Generated Columns + Indexes**: Avoids repeated `json_extract()` parsing
- **WAL Mode for Concurrent Reads**: `PRAGMA journal_mode=WAL` enables multiple readers during writes
- **Prepared Statements**: ~10x faster than dynamic SQL for repeated queries
- **Batch Inserts in Transactions**: Wrap multiple INSERTs in single transaction
- **Index Strategically**: Index frequently queried JSON fields (functional indexes or generated columns)
- **Schema-Driven Views**: Define views per schema type for clean query interface
- **json_each() for Arrays**: Query JSON arrays efficiently
- **Avoid STORED Columns Unless Necessary**: VIRTUAL more flexible (ALTER TABLE), STORED faster for hot paths

**Options for Schema-Driven Queries**:

**Option A: VIRTUAL Generated Columns + Functional Indexes**

```sql
-- VIRTUAL columns for common fields
ALTER TABLE notes ADD COLUMN title TEXT AS (json_extract(frontmatter, '$.title'));

-- Functional indexes for schema-specific fields
CREATE INDEX idx_meeting_date ON notes(
    (json_extract(frontmatter, '$.date'))
) WHERE schema_type = 'meeting';
```

- ✅ Flexible (ALTER TABLE to add columns)
- ✅ Lower disk usage
- ✅ Universal fields indexed once
- ❌ Slower reads (computed each time for non-indexed fields)

**Option B: STORED Generated Columns**

```sql
-- STORED columns cache all common values
CREATE TABLE notes (
    id TEXT PRIMARY KEY,
    frontmatter TEXT,
    title TEXT GENERATED ALWAYS AS (json_extract(frontmatter, '$.title')) STORED,
    schema_type TEXT GENERATED ALWAYS AS (json_extract(frontmatter, '$.schema')) STORED
);
```

- ✅ Faster reads (cached values)
- ✅ No computation at read time
- ❌ Higher disk usage
- ❌ Must plan schema upfront (no ALTER TABLE)
- ❌ Slower writes (compute + store)

**Option C: Views with json_extract() (No Generated Columns)**

```sql
-- Views only, no generated columns
CREATE VIEW notes_meeting AS
SELECT id, json_extract(frontmatter, '$.title') AS title
FROM notes
WHERE json_extract(frontmatter, '$.schema') = 'meeting';
```

- ✅ Simple
- ✅ No schema changes
- ❌ Slow for 100k+ records (repeated json_extract())
- ❌ Hard to index

**Option D: Hybrid - VIRTUAL for Universal, Views for Schema-Specific**

```sql
-- VIRTUAL columns for universal fields
ALTER TABLE notes ADD COLUMN title TEXT AS (json_extract(frontmatter, '$.title'));
ALTER TABLE notes ADD COLUMN schema_type TEXT AS (json_extract(frontmatter, '$.schema'));

-- Schema-driven views for specialized fields
CREATE VIEW notes_meeting AS
SELECT
    id, title,
    json_extract(frontmatter, '$.date') AS meeting_date
FROM notes
WHERE schema_type = 'meeting';

-- Functional indexes on hot paths
CREATE INDEX idx_meeting_date ON notes(
    (json_extract(frontmatter, '$.date'))
) WHERE schema_type = 'meeting';
```

- ✅ VIRTUAL columns for universal fields (indexed once)
- ✅ Views for schema-specific fields (no table alteration)
- ✅ Functional indexes for frequently queried fields
- ✅ Flexibility (add new schemas without altering table)
- ✅ <50ms target achievable

**Performance Characteristics**:

- Universal queries (by title, schema): <10ms (indexed VIRTUAL columns)
- Schema-specific queries (meeting by date): <30ms (functional index)
- Full-text search: <50ms (FTS5 virtual table if needed)
- Bulk inserts: ~1000-5000 rows/second (modernc.org/sqlite)

**Impact on Issues**:

- **Group 2 (Storage Architecture)**: Provides <50ms deep storage architecture
- **Issue A5 (Schema-Driven Views)**: Complete pattern for generating views per schema type
- **Issue D2 (DTO Architecture)**: SQLite adapter pattern for deep storage queries

**Sources**:

- pkg.go.dev/modernc.org/sqlite
- antonz.org/json-virtual-columns (SQLite JSON best practices)

##### goldmark

**Research Status**: ✅ Complete (2025-11-07)

**Purpose**: Address Group 1 Extract Markdown Parsing from Domain (Issue B2) - Current goldmark parser used in domain layer (violates hexagonal architecture), need adapter patterns

Local References:

- `docs/refs/yuin-goldmark-digest.txt`
- `docs/refs/abhinav-goldmark-frontmatter-digest.txt`

**Core Parser API**:

1. **High-Level API - Convert()**:

   ```go
   import "github.com/yuin/goldmark"

   // Convert markdown source to HTML in one call
   md := goldmark.New()
   source := []byte("# Hello\nMarkdown content")
   var buf bytes.Buffer

   if err := md.Convert(source, &buf); err != nil {
       panic(err)
   }

   html := buf.String()
   ```

2. **Low-Level API - Parse() for AST Access**:

   ```go
   // Parse to AST without rendering
   md := goldmark.New()
   source := []byte("# Hello\nMarkdown content")
   reader := text.NewReader(source)
   ctx := parser.NewContext()

   // Parse returns ast.Node (root of AST)
   doc := md.Parser().Parse(reader, parser.WithContext(ctx))

   // Walk AST to extract metadata
   ast.Walk(doc, func(n ast.Node, entering bool) (ast.WalkStatus, error) {
       if entering {
           if heading, ok := n.(*ast.Heading); ok {
               // Process heading
           }
       }
       return ast.WalkContinue, nil
   })
   ```

3. **Configuration with Options**:
   ```go
   md := goldmark.New(
       goldmark.WithExtensions(extension.GFM), // GitHub Flavored Markdown
       goldmark.WithParserOptions(
           parser.WithAutoHeadingID(),        // Auto-generate heading IDs
           parser.WithAttribute(),            // Support attributes
       ),
       goldmark.WithRendererOptions(
           html.WithHardWraps(),              // Render \n as <br>
           html.WithXHTML(),                  // XHTML output
       ),
   )
   ```

**AST Node Types and Traversal**:

1. **Common Node Types**:
   - `ast.Document` - Root document
   - `ast.Heading` - # Heading
   - `ast.Paragraph` - Text paragraph
   - `ast.Link` - [text](url)
   - `ast.Text` - Plain text
   - `ast.CodeBlock` - `code`
   - `ast.List` - Unordered/ordered lists
   - `ast.ListItem` - List item

2. **Walking the AST**:

   ```go
   // ast.Walk visits each node twice (entering and leaving)
   ast.Walk(doc, func(n ast.Node, entering bool) (ast.WalkStatus, error) {
       if entering {
           // Process node on entry
           switch node := n.(type) {
           case *ast.Heading:
               level := node.Level
               // Extract heading text

           case *ast.Link:
               destination := node.Destination
               // Extract link URL

           case *ast.Text:
               segment := node.Segment
               text := segment.Value(source) // Get text from source
           }
       }

       // Return status to control traversal
       return ast.WalkContinue, nil
       // ast.WalkStop - stop walking
       // ast.WalkSkipChildren - skip children of current node
   })
   ```

3. **Extracting Text from Nodes**:

   ```go
   // Nodes don't contain text directly, only segment references
   type Text struct {
       ast.BaseInline
       Segment text.Segment // {Start, End, Padding}
   }

   // Extract text using source
   func extractText(n ast.Node, source []byte) string {
       if textNode, ok := n.(*ast.Text); ok {
           return string(textNode.Segment.Value(source))
       }
       return ""
   }
   ```

**Extension System**:

1. **Implementing Extensions**:

   ```go
   // Extension implements Extender interface
   type MyExtension struct{}

   func (e *MyExtension) Extend(md goldmark.Markdown) {
       md.Parser().AddOptions(
           parser.WithInlineParsers(...),
           parser.WithBlockParsers(...),
           parser.WithParagraphTransformers(...),
           parser.WithASTTransformers(...),
       )
       md.Renderer().AddOptions(
           renderer.WithNodeRenderers(...),
       )
   }

   // Register extension
   md := goldmark.New(
       goldmark.WithExtensions(&MyExtension{}),
   )
   ```

2. **Extension Types**:
   - **Block Parsers**: Parse block-level elements (custom blocks)
   - **Inline Parsers**: Parse inline elements (custom syntax)
   - **Paragraph Transformers**: Post-process paragraphs
   - **AST Transformers**: Transform entire document AST
   - **Node Renderers**: Custom rendering for AST nodes

**Frontmatter Extension** (go.abhg.dev/goldmark/frontmatter):

1. **Installation and Registration**:

   ```go
   import (
       "github.com/yuin/goldmark"
       "go.abhg.dev/goldmark/frontmatter"
   )

   md := goldmark.New(
       goldmark.WithExtensions(
           &frontmatter.Extender{},
       ),
   )
   ```

2. **Extracting Frontmatter**:

   ```go
   // Parse with context to extract frontmatter
   ctx := parser.NewContext()
   source := []byte(`---
   title: My Note
   tags: [golang, markdown]
   ---
   # Content`)

   var buf bytes.Buffer
   if err := md.Convert(source, &buf, parser.WithContext(ctx)); err != nil {
       panic(err)
   }

   // Retrieve frontmatter data from context
   d := frontmatter.Get(ctx)
   if d == nil {
       // No frontmatter found
   }

   // Decode into struct
   var meta struct {
       Title string   `yaml:"title"`
       Tags  []string `yaml:"tags"`
   }
   if err := d.Decode(&meta); err != nil {
       panic(err)
   }

   fmt.Println(meta.Title) // "My Note"
   ```

3. **Supported Formats**:
   - **YAML**: Delimited by `---` (3+ hyphens)
   - **TOML**: Delimited by `+++` (3+ plus signs)
   - **Custom**: Implement `frontmatter.Format` interface

4. **Alternative: Document Metadata Mode**:

   ```go
   // Store frontmatter in document metadata (not context)
   md := goldmark.New(
       goldmark.WithExtensions(
           &frontmatter.Extender{
               Mode: frontmatter.SetMetadata,
           },
       ),
   )

   doc := md.Parser().Parse(text.NewReader(source))

   // Access via document metadata
   meta := doc.OwnerDocument().Meta()
   ```

**Metadata Extraction Patterns**:

1. **Extract All Metadata (Links, Headings, Frontmatter)**:

   ```go
   type NoteMetadata struct {
       Frontmatter map[string]any
       Links       []string
       Headings    []Heading
       Tags        []string
   }

   func ExtractMetadata(source []byte) (*NoteMetadata, error) {
       md := goldmark.New(
           goldmark.WithExtensions(
               &frontmatter.Extender{},
           ),
           goldmark.WithParserOptions(
               parser.WithAutoHeadingID(),
           ),
       )

       ctx := parser.NewContext()
       doc := md.Parser().Parse(text.NewReader(source), parser.WithContext(ctx))

       metadata := &NoteMetadata{
           Frontmatter: make(map[string]any),
       }

       // Extract frontmatter
       if fmData := frontmatter.Get(ctx); fmData != nil {
           fmData.Decode(&metadata.Frontmatter)
       }

       // Extract links and headings in single AST walk
       ast.Walk(doc, func(n ast.Node, entering bool) (ast.WalkStatus, error) {
           if !entering {
               return ast.WalkContinue, nil
           }

           switch node := n.(type) {
           case *ast.Link:
               metadata.Links = append(metadata.Links, string(node.Destination))

           case *ast.Heading:
               var text string
               for child := node.FirstChild(); child != nil; child = child.NextSibling() {
                   if textNode, ok := child.(*ast.Text); ok {
                       text += string(textNode.Segment.Value(source))
                   }
               }
               metadata.Headings = append(metadata.Headings, Heading{
                   Level: node.Level,
                   Text:  text,
               })
           }

           return ast.WalkContinue, nil
       })

       return metadata, nil
   }
   ```

**Adapter Pattern - Wrapping goldmark**:

```go
// Domain: Port interface (NOT implementation)
package ports

type MarkdownParser interface {
    ParseMetadata(content []byte) (*domain.NoteMetadata, error)
    ConvertToHTML(content []byte) (string, error)
}

type NoteMetadata struct {
    Frontmatter map[string]any
    Links       []Link
    Headings    []Heading
}

// Adapter: Wraps goldmark
package parser

import (
    "github.com/yuin/goldmark"
    "go.abhg.dev/goldmark/frontmatter"
    "internal/domain"
    "internal/domain/ports"
)

type MDFileAdapter struct {
    md goldmark.Markdown
}

func NewMDFileAdapter() *MDFileAdapter {
    md := goldmark.New(
        goldmark.WithExtensions(
            extension.GFM,
            &frontmatter.Extender{},
        ),
        goldmark.WithParserOptions(
            parser.WithAutoHeadingID(),
        ),
    )

    return &MDFileAdapter{md: md}
}

func (a *MDFileAdapter) ParseMetadata(content []byte) (*domain.NoteMetadata, error) {
    ctx := parser.NewContext()
    reader := text.NewReader(content)
    doc := a.md.Parser().Parse(reader, parser.WithContext(ctx))

    metadata := &domain.NoteMetadata{
        Frontmatter: make(map[string]any),
    }

    // Extract frontmatter
    if fmData := frontmatter.Get(ctx); fmData != nil {
        fmData.Decode(&metadata.Frontmatter)
    }

    // Extract links and headings
    ast.Walk(doc, func(n ast.Node, entering bool) (ast.WalkStatus, error) {
        if !entering {
            return ast.WalkContinue, nil
        }

        switch node := n.(type) {
        case *ast.Link:
            metadata.Links = append(metadata.Links, domain.Link{
                Destination: string(node.Destination),
                IsWikilink:  false,
            })

        case *ast.Heading:
            heading := a.extractHeading(node, content)
            metadata.Headings = append(metadata.Headings, heading)
        }

        return ast.WalkContinue, nil
    })

    return metadata, nil
}

// Service: Uses port interface (domain layer)
package services

type NoteService struct {
    parser ports.MarkdownParser
}

func NewNoteService(parser ports.MarkdownParser) *NoteService {
    return &NoteService{parser: parser}
}

func (s *NoteService) ProcessNote(content []byte) (*domain.Note, error) {
    metadata, err := s.parser.ParseMetadata(content)
    if err != nil {
        return nil, err
    }

    note := &domain.Note{
        Frontmatter: metadata.Frontmatter,
        Links:       metadata.Links,
        Headings:    metadata.Headings,
    }

    return note, nil
}

// Main: Wire adapter to service (dependency injection)
func main() {
    // Create adapter
    parser := parser.NewMDFileAdapter()

    // Inject adapter into service
    noteService := services.NewNoteService(parser)

    // Use service
    content := []byte("# My Note\n...")
    note, err := noteService.ProcessNote(content)
}
```

**Best Practices**:

- **Parse Once, Extract Multiple Metadata**: Single AST walk extracts links, headings, tags
- **Use Extensions for Frontmatter**: goldmark-frontmatter handles YAML/TOML parsing
- **AST Walk Performance**: goldmark's AST is fast (performance on par with cmark C implementation)
- **Interface-Based Design**: Wrap goldmark in adapter implementing domain interface
- **Context for Metadata**: Use `parser.Context` to pass frontmatter/metadata between parse and process
- **Segment References**: AST nodes store text.Segment (offsets), not text content directly

**Options for Markdown Parsing Architecture**:

**Option A: Use goldmark directly in domain**

- ✅ Simple, direct access
- ❌ Couples domain to goldmark library
- ❌ Violates hexagonal architecture
- ❌ Hard to test (need real markdown)

**Option B: MarkdownParser interface in domain, MDFileAdapter in adapter layer**

- ✅ Clean hexagonal separation
- ✅ Domain defines interface, adapter implements
- ✅ Testable (mock MarkdownParser interface)
- ✅ Can swap parsers if needed (future-proof)
- ❌ Additional abstraction layer

**Option C: Multiple parser adapters (goldmark, blackfriday, commonmark)**

- ✅ Flexibility to choose parser
- ✅ Testable
- ❌ More complexity (need multiple adapters)
- ❌ YAGNI (no current requirement for multiple parsers)

**Performance Characteristics**:

- Parsing: Fast (on par with C CommonMark implementations)
- AST traversal: Efficient (single pass for multiple metadata types)
- Frontmatter extraction: Built-in with extension

**Impact on Issues**:

- **Issue B2 (Goldmark in Domain)**: Provides adapter pattern to move goldmark to adapter layer
- **Group 1 (Validation Architecture)**: Frontmatter extraction moves to adapter, semantic validation stays in domain

**Sources**:

- github.com/yuin/goldmark
- pkg.go.dev/go.abhg.dev/goldmark/frontmatter

#### Go Generics (Go 1.18+)

**Research Status**: ✅ Complete (2025-11-07)

**Purpose**: Cross-cutting - Type-safe architecture patterns, opportunities for generic ports, type-safe DTOs, eliminating interface{} and type assertions

**Core Generic Patterns**:

1. **Generic Repository Pattern**:

   ```go
   // Generic repository interface
   type Repository[T any] interface {
       GetByID(id string) (*T, error)
       List(filter Filter) ([]*T, error)
       Save(entity *T) error
       Delete(id string) error
   }

   // Concrete implementation for Note entity
   type NoteRepository struct {
       db *sql.DB
   }

   func (r *NoteRepository) GetByID(id string) (*Note, error) {
       // Implementation
   }

   // Usage
   var noteRepo Repository[Note] = &NoteRepository{db}
   note, err := noteRepo.GetByID("note-123")
   ```

2. **Type Constraints - Built-in and Custom**:

   ```go
   // any: Any type (alias for interface{})
   func Print[T any](v T) {
       fmt.Println(v)
   }

   // comparable: Types that support == and !=
   func Contains[T comparable](slice []T, value T) bool {
       for _, v := range slice {
           if v == value {
               return true
           }
       }
       return false
   }

   // Map keys must be comparable
   type Cache[K comparable, V any] struct {
       data map[K]V
   }

   // Union constraint (Go 1.18+)
   type Number interface {
       int | int64 | float64
   }

   func Sum[T Number](values []T) T {
       var sum T
       for _, v := range values {
           sum += v
       }
       return sum
   }

   // Constraint with methods
   type Validator interface {
       Validate() error
   }

   func ValidateAll[T Validator](entities []T) error {
       for _, e := range entities {
           if err := e.Validate(); err != nil {
               return err
           }
       }
       return nil
   }
   ```

3. **Constraint Composition**:

   ```go
   // Combine multiple constraints
   type IDEntity interface {
       comparable              // Can use as map key
       GetID() string
       SetID(string)
   }

   type Repository[T IDEntity] interface {
       GetByID(id string) (*T, error)
       Save(entity *T) error
   }

   // ~T form: Underlying type T
   type Integer interface {
       ~int | ~int64 // Includes type aliases with int/int64 underlying type
   }

   type UserID int // Underlying type: int

   func Add[T Integer](a, b T) T {
       return a + b
   }

   var x UserID = 5
   var y UserID = 10
   z := Add(x, y) // ✅ Works: UserID's underlying type is int
   ```

4. **Generic Functions vs Generic Types**:

   **Generic Functions** (Simpler, Use When Possible):

   ```go
   // Generic function: No type state to maintain
   func Map[T, U any](slice []T, fn func(T) U) []U {
       result := make([]U, len(slice))
       for i, v := range slice {
           result[i] = fn(v)
       }
       return result
   }

   // Usage: Type inference
   nums := []int{1, 2, 3}
   strs := Map(nums, func(n int) string { return fmt.Sprint(n) })
   // Type parameters inferred: Map[int, string]
   ```

   **Generic Types** (Use When Encapsulating State):

   ```go
   // Generic type: Maintains type-specific state
   type Cache[K comparable, V any] struct {
       data map[K]V
       mu   sync.RWMutex
   }

   func NewCache[K comparable, V any]() *Cache[K, V] {
       return &Cache[K, V]{
           data: make(map[K]V),
       }
   }

   func (c *Cache[K, V]) Get(key K) (V, bool) {
       c.mu.RLock()
       defer c.mu.RUnlock()
       v, ok := c.data[key]
       return v, ok
   }

   func (c *Cache[K, V]) Set(key K, value V) {
       c.mu.Lock()
       defer c.mu.Unlock()
       c.data[key] = value
   }

   // Usage: Must specify type parameters
   cache := NewCache[string, *Note]()
   cache.Set("note-123", note)
   note, ok := cache.Get("note-123")
   ```

5. **Type Inference**:

   ```go
   func Max[T constraints.Ordered](a, b T) T {
       if a > b {
           return a
       }
       return b
   }

   // Type inferred from arguments
   x := Max(5, 10)        // T inferred as int
   y := Max(3.14, 2.71)   // T inferred as float64

   // Explicit type arguments (when inference fails)
   z := Max[int64](5, 10) // Explicit: T = int64

   // Struct instantiation: MUST specify type parameters
   type Pair[T, U any] struct {
       First  T
       Second U
   }

   // ❌ Cannot infer: Must specify types
   p := Pair{First: 1, Second: "hello"} // Compile error

   // ✅ Specify types explicitly
   p := Pair[int, string]{First: 1, Second: "hello"}
   ```

6. **Generic Result Type Pattern**:

   ```go
   // Result type for error handling (similar to Rust)
   type Result[T any] struct {
       value T
       err   error
   }

   func Ok[T any](value T) Result[T] {
       return Result[T]{value: value}
   }

   func Err[T any](err error) Result[T] {
       return Result[T]{err: err}
   }

   func (r Result[T]) IsOk() bool {
       return r.err == nil
   }

   func (r Result[T]) Unwrap() (T, error) {
       return r.value, r.err
   }

   func (r Result[T]) UnwrapOr(defaultValue T) T {
       if r.err != nil {
           return defaultValue
       }
       return r.value
   }

   // Usage
   func GetNote(id string) Result[*Note] {
       note, err := db.Query(...)
       if err != nil {
           return Err[*Note](err)
       }
       return Ok(note)
   }

   result := GetNote("note-123")
   if result.IsOk() {
       note, _ := result.Unwrap()
       // Use note
   }
   ```

7. **Generic Validation Pattern**:

   ```go
   // Generic validator interface
   type Validator[T any] interface {
       Validate(entity T) error
   }

   // Generic validation chain
   type ValidationChain[T any] struct {
       validators []Validator[T]
   }

   func (vc *ValidationChain[T]) Add(v Validator[T]) {
       vc.validators = append(vc.validators, v)
   }

   func (vc *ValidationChain[T]) Validate(entity T) error {
       for _, v := range vc.validators {
           if err := v.Validate(entity); err != nil {
               return err
           }
       }
       return nil
   }

   // Concrete validator for Note
   type NoteTitleValidator struct{}

   func (v *NoteTitleValidator) Validate(note *Note) error {
       if note.Title == "" {
           return errors.New("title required")
       }
       return nil
   }

   // Usage
   chain := &ValidationChain[*Note]{}
   chain.Add(&NoteTitleValidator{})
   chain.Add(&NoteContentValidator{})

   if err := chain.Validate(note); err != nil {
       // Validation failed
   }
   ```

8. **Generic Collection Utilities**:

   ```go
   // Filter
   func Filter[T any](slice []T, predicate func(T) bool) []T {
       result := []T{}
       for _, v := range slice {
           if predicate(v) {
               result = append(result, v)
           }
       }
       return result
   }

   // Map
   func Map[T, U any](slice []T, fn func(T) U) []U {
       result := make([]U, len(slice))
       for i, v := range slice {
           result[i] = fn(v)
       }
       return result
   }

   // Reduce
   func Reduce[T, U any](slice []T, initial U, fn func(U, T) U) U {
       result := initial
       for _, v := range slice {
           result = fn(result, v)
       }
       return result
   }

   // Usage
   notes := []*Note{...}

   // Filter published notes
   published := Filter(notes, func(n *Note) bool {
       return n.Status == "published"
   })

   // Extract titles
   titles := Map(notes, func(n *Note) string {
       return n.Title
   })

   // Count total words
   totalWords := Reduce(notes, 0, func(sum int, n *Note) int {
       return sum + len(strings.Fields(n.Content))
   })
   ```

**Performance Implications**:

**Monomorphization** (Compile-Time Specialization):

```go
// Generic function
func Add[T Number](a, b T) T {
    return a + b
}

// Compiler generates specialized versions
// func Add_int(a, b int) int { return a + b }
// func Add_float64(a, b float64) float64 { return a + b }

// Impact: Increased binary size (each type instantiation = new code)
```

**Performance Comparison**:

- **Generics vs Interfaces**: Generics avoid dynamic dispatch (faster), but increase binary size
- **Generics vs Code Generation**: Similar performance, generics simpler (no codegen tools)
- **Generics vs interface{}**: Generics type-safe at compile time, avoid runtime type assertions

**Benchmarks**:

- Generics: Similar or slightly faster than interfaces (no dynamic dispatch)
- Binary size: Increases with number of type instantiations
- Recommendation: Use generics for performance-critical paths, interfaces for flexibility

**Best Practices**:

1. **Don't Overuse Generics**: "Only use generics for code reused across types." If interface works, prefer interface.

2. **Prefer Generic Functions Over Generic Types**: Functions are simpler (no state), better type inference.

3. **Use Constraint Interfaces Wisely**: Constraints document requirements, enable compile-time checking.

4. **Leverage Type Inference**: Let compiler infer types from function arguments (cleaner code).

5. **Avoid Deep Generic Nesting**: `Repository[Cache[K, V]]` harder to read/debug than simple types.

6. **Benchmark Performance-Critical Code**: Generics add binary size; measure tradeoff.

7. **Generic Domain Entities? Usually No**: Domain entities typically specific types (Note, Vault), not generic. Generics better for infrastructure (repositories, caches, collections).

**When to Use Generics vs Interfaces**:

**Use Generics When**:

- ✅ Type-safe collections (Cache[K, V], List[T])
- ✅ Type-safe operations (Filter, Map, Reduce)
- ✅ Type-safe repositories (Repository[T])
- ✅ Avoiding interface{} and type assertions
- ✅ Performance-critical (avoid dynamic dispatch)

**Use Interfaces When**:

- ✅ Behavior abstraction (multiple implementations)
- ✅ Polymorphism (different types, same interface)
- ✅ Plugin architecture (dynamic loading)
- ✅ Domain ports (hexagonal architecture)
- ✅ Simple, readable code

**Avoid Generics When**:

- ❌ Single type (no reuse across types)
- ❌ Domain entities (specific types, not generic)
- ❌ Complex constraints (hard to read/maintain)
- ❌ Binary size critical (generics increase size)

**Options for Lithos Architecture**:

**Option A: Generic Repository Ports**

```go
// Generic repository interface
type Repository[T Entity] interface {
    GetByID(id string) (*T, error)
    List(filter Filter) ([]*T, error)
    Save(entity *T) error
    Delete(id string) error
}

// Entity constraint
type Entity interface {
    GetID() string
    SetID(string)
    Validate() error
}

// Domain entities implement Entity
type Note struct {
    ID string
}

func (n *Note) GetID() string { return n.ID }
func (n *Note) SetID(id string) { n.ID = id }
func (n *Note) Validate() error { /* ... */ }

// Adapter implements generic interface
type SQLiteRepository[T Entity] struct {
    db *sql.DB
}

func (r *SQLiteRepository[T]) GetByID(id string) (*T, error) {
    // Generic implementation
}

// Usage
var noteRepo Repository[Note] = &SQLiteRepository[Note]{db}
var vaultRepo Repository[Vault] = &SQLiteRepository[Vault]{db}
```

- ✅ Type-safe, no code duplication
- ✅ Single generic implementation
- ❌ More complex (generic constraints)
- ❌ Domain entities must implement Entity

**Option B: Specific Ports, No Generics**

```go
// Separate interface per entity type
type NoteRepository interface {
    GetByID(id string) (*Note, error)
    Save(note *Note) error
}

type VaultRepository interface {
    GetByID(id string) (*Vault, error)
    Save(vault *Vault) error
}
```

- ✅ Simple, explicit
- ❌ Code duplication

**Option C: Hybrid - Specific Ports, Generic Utilities**

```go
// Domain: Specific ports (simple, explicit)
type NoteRepository interface {
    GetByID(id string) (*Note, error)
    Save(note *Note) error
}

// Infrastructure: Generic utilities
type Cache[K comparable, V any] struct {
    data map[K]V
}

type QueryBuilder[T any] struct {
    filters []Filter
}

// Domain helpers: Generic collection functions
func Filter[T any](slice []T, predicate func(T) bool) []T

func Map[T, U any](slice []T, fn func(T) U) []U
```

- ✅ Simple domain interfaces
- ✅ Generic infrastructure utilities
- ✅ Best of both worlds

**Guideline for Lithos**:

- **Generic for reusable infrastructure**: Cache, validation, collections
- **Non-generic for domain ports**: Repository interfaces, service interfaces
- **Generic for cross-cutting utilities**: Filter, Map, Reduce
- **Non-generic for domain entities**: Note, Vault are specific types

**Impact on Issues**:

- **Cross-Cutting**: Type-safe infrastructure without complex constraints in domain
- **Issue D2 (DTO Architecture)**: Generic Cache[K,V] for hot cache layer
- **Validation**: Generic ValidationChain[T] for composable validators

**Sources**:

- go.dev/doc/tutorial/generics (official tutorial)
- codingexplorations.com/blog/performance-implications-generics-in-go (performance analysis)

### Obsidian API

#### Obsidian TypeScript API

**Research Status**: ✅ Complete (2025-11-08)

**Purpose**: Address Group 1 (DTO Design) and Group 2 (Storage Architecture) - Understanding Obsidian's patterns for vault file abstraction, metadata caching, frontmatter handling, and path management to inform Lithos architecture

**File Abstractions - TAbstractFile → TFile/TFolder**:

Obsidian uses a class hierarchy for vault files:

```typescript
// Base class for all vault items
abstract class TAbstractFile {
  vault: Vault; // Reference to containing vault
  path: string; // Vault-relative path with extension
  name: string; // File or folder name
  parent: TFolder; // Parent folder reference
}

// Represents a markdown file or other file type
class TFile extends TAbstractFile {
  stat: FileStats; // File statistics (ctime, mtime, size)
  basename: string; // Filename without extension
  extension: string; // File extension
}

// Represents a folder
class TFolder extends TAbstractFile {
  children: TAbstractFile[]; // Child files and folders
  isRoot(): boolean; // Check if vault root
}
```

**Key Pattern - Vault-Relative Paths**:

```typescript
// Obsidian uses vault-relative paths throughout
file.path; // "folder/subfolder/note.md" (vault-relative with extension)
file.basename; // "note" (filename without extension)
file.extension; // "md"
file.parent.path; // "folder/subfolder"
```

**FileStats Interface**:

```typescript
interface FileStats {
  ctime: number; // Creation time (milliseconds since epoch)
  mtime: number; // Modification time (milliseconds since epoch)
  size: number; // File size in bytes
}

// Usage:
const file: TFile = app.vault.getAbstractFileByPath("note.md");
file.stat.mtime; // Modification timestamp
file.stat.size; // File size
```

**Comparison to Lithos FileMetadata**:

| Property   | Obsidian TFile          | Lithos FileMetadata           | Analysis                                  |
| ---------- | ----------------------- | ----------------------------- | ----------------------------------------- |
| Path       | `path` (vault-relative) | `Path` (absolute)             | Obsidian normalizes to vault-relative     |
| Basename   | `basename` (property)   | `Basename` (cached, computed) | Obsidian provides directly, Lithos caches |
| Extension  | `extension` (property)  | `Ext` (cached, computed)      | Obsidian provides directly, Lithos caches |
| Folder     | `parent.path` (object)  | `Folder` (cached, computed)   | Obsidian uses object graph, Lithos caches |
| ModTime    | `stat.mtime`            | `ModTime` (duplicated)        | Both store modification time              |
| Size       | `stat.size`             | `Size` (duplicated)           | Both store file size                      |
| MimeType   | N/A                     | `MimeType` (cached, computed) | Obsidian doesn't include, Lithos computes |
| Vault Ref  | `vault` (object)        | N/A                           | Obsidian maintains vault reference        |
| Parent Ref | `parent` (object)       | N/A                           | Obsidian maintains object graph           |

**MetadataCache - Cached Frontmatter and Links**:

```typescript
// Central metadata cache for vault
class MetadataCache extends Events {
  // Get cached metadata for a file
  getFileCache(file: TFile): CachedMetadata | null;
  getCache(path: string): CachedMetadata | null;

  // Get first file matching linkpath from source
  getFirstLinkpathDest(linkpath: string, sourcePath: string): TFile | null;

  // Resolved and unresolved link tracking
  resolvedLinks: Record<string, Record<string, number>>;
  unresolvedLinks: Record<string, Record<string, number>>;
}

// Cached metadata structure
interface CachedMetadata {
  links?: LinkCache[]; // [[wikilinks]] and [markdown](links)
  embeds?: EmbedCache[]; // ![[embeds]]
  tags?: TagCache[]; // #tags
  headings?: HeadingCache[]; // # Headings
  frontmatter?: FrontMatterCache; // YAML frontmatter
  frontmatterPosition?: Pos; // Position in file
  frontmatterLinks?: FrontmatterLinkCache[]; // Links in frontmatter
  sections?: SectionCache[]; // Document sections
  listItems?: ListItemCache[]; // List items
  blocks?: Record<string, BlockCache>; // ^block-refs
}

// Frontmatter is just key-value pairs
interface FrontMatterCache {
  [key: string]: any; // No schema enforcement in cache
}
```

**Key Pattern - Separation of File Data and Metadata**:

```typescript
// File data (TFile) and cached metadata (CachedMetadata) are SEPARATE
const file = app.vault.getAbstractFileByPath("note.md"); // File object
const metadata = app.metadataCache.getFileCache(file); // Cached metadata

// File object does NOT contain frontmatter
// Frontmatter accessed through MetadataCache
```

**Vault API - File Operations**:

```typescript
class Vault {
  // Read operations
  read(file: TFile): Promise<string>; // Read file content from disk
  cachedRead(file: TFile): Promise<string>; // Read from cache (display only)

  // File lookup
  getAbstractFileByPath(path: string): TAbstractFile | null;
  getMarkdownFiles(): TFile[]; // All markdown files
  getFiles(): TFile[]; // All files

  // Write operations
  create(path: string, data: string): Promise<TFile>;
  modify(file: TFile, data: string): Promise<void>;
  rename(file: TAbstractFile, newPath: string): Promise<void>;

  // Delete operations
  delete(file: TAbstractFile): Promise<void>; // Permanent deletion
  trash(file: TAbstractFile): Promise<void>; // Move to trash
}
```

**Atomic Frontmatter Modification Pattern**:

```typescript
// FileManager.processFrontMatter() - Atomic read-modify-write
app.fileManager.processFrontMatter(file, (frontmatter) => {
  // Callback receives frontmatter object
  // Mutate synchronously
  frontmatter.title = "New Title";
  frontmatter.tags = ["tag1", "tag2"];

  // No return needed - mutations applied atomically
  // Prevents concurrent modification data loss
});
```

**Obsidian Pattern Summary**:

1. **File Object ≠ Metadata**: TFile contains file system properties only; frontmatter/links stored separately in MetadataCache
2. **Vault-Relative Paths**: All paths relative to vault root, not absolute file system paths
3. **Object Graph**: TFile → TFolder → Vault (maintains references, not flat data)
4. **Computed Properties**: `basename`, `extension` are properties, not cached values
5. **Cached Metadata**: MetadataCache provides fast access to parsed frontmatter/links without re-parsing
6. **Atomic Frontmatter Updates**: `processFrontMatter()` prevents race conditions with read-modify-write callback
7. **Two Read Modes**: `read()` for modification, `cachedRead()` for display
8. **Separation of Concerns**: File operations (Vault) separate from metadata access (MetadataCache)

**Comparison to Lithos Architecture**:

| Aspect               | Obsidian Pattern                               | Lithos Current Implementation        | Gap Analysis                                    |
| -------------------- | ---------------------------------------------- | ------------------------------------ | ----------------------------------------------- |
| File Abstraction     | TFile (properties only)                        | VaultFile (FileMetadata + Content)   | Obsidian separates metadata from file data      |
| Path Storage         | Vault-relative                                 | Absolute                             | Obsidian normalizes to vault context            |
| Basename/Ext         | Properties (computed on-demand?)               | Cached fields                        | Obsidian may compute dynamically from `path`    |
| Frontmatter Storage  | Separate MetadataCache                         | Parsed from Content in VaultFile     | Obsidian caches frontmatter separately          |
| Object Relationships | TFile → TFolder → Vault (object graph)         | Flat FileMetadata                    | Obsidian maintains navigable structure          |
| Metadata Caching     | Central MetadataCache with events              | No dedicated cache (parse on-demand) | Obsidian caches parsed metadata for performance |
| Atomic Updates       | processFrontMatter() callback                  | Direct file writes                   | Obsidian prevents race conditions               |
| Read Strategies      | read() vs cachedRead()                         | Single read pattern                  | Obsidian optimizes for display vs modification  |
| Link Resolution      | MetadataCache tracks resolved/unresolved links | No link tracking                     | Obsidian provides graph analysis                |

**Potential Improvements for Lithos**:

1. **Separate File and Metadata Concerns**:
   - VaultFile should be file system data only (path, stat)
   - Create separate NoteMetadata cache for frontmatter/links
   - Matches CQRS pattern (file data = write model, metadata cache = read model)

2. **Use Vault-Relative Paths**:
   - Store paths relative to vault root, not absolute
   - Compute absolute paths when needed for file I/O
   - Simplifies multi-vault support, makes cache portable

3. **Compute Basename/Extension On-Demand**:
   - Don't cache `Basename`, `Folder`, `Ext` in FileMetadata
   - Compute from `path` using filepath stdlib functions
   - Use fs.FileInfo.ModTime() and Size() instead of duplicating

4. **Create MetadataCache Service**:
   - Cache parsed frontmatter separately from file content
   - Invalidate cache on file modification
   - Provide fast metadata queries without re-parsing

5. **Adopt Read vs CachedRead Pattern**:
   - `cachedRead()` for display/queries (use cache)
   - `read()` for modification (always fresh from disk)
   - Prevents stale data overwrites

**Impact on Issues**:

- **Group 1 (DTO Design, Issue A4)**: Obsidian separates file data from metadata - informs VaultFile/FileMetadata redesign
- **Group 2 (Storage Architecture, Issue A5)**: MetadataCache pattern provides read-optimized metadata access
- **Group 1 (Frontmatter Extraction, Issue B2)**: Separate metadata cache aligns with adapter-based parsing
- **Cross-cutting**: Vault-relative paths simplify multi-vault scenarios

**Sources**:

- docs.obsidian.md/Reference/TypeScript+API/TFile
- docs.obsidian.md/Reference/TypeScript+API/TAbstractFile
- docs.obsidian.md/Reference/TypeScript+API/MetadataCache
- docs.obsidian.md/Plugins/Vault
- github.com/obsidianmd/obsidian-api/blob/master/obsidian.d.ts

## Comprehensive Gap Analysis

**Status**: ✅ Complete (2025-11-08)

This section synthesizes findings from Research Phases 1-2 (Go stdlib + Obsidian API) to identify specific architectural gaps in Lithos, organized by impact area. Each gap includes current state, desired state, relevant patterns, and affected issues.

### Gap Category 1: File Metadata Architecture

#### Gap 1.1: FileMetadata Duplicates fs.FileInfo

**Current State**:

```go
// /internal/shared/dto/file.go
type FileMetadata struct {
    Path     string
    Basename string    // computed from filepath.Base + TrimSuffix
    Folder   string    // computed from filepath.Dir
    Ext      string    // computed from filepath.Ext
    ModTime  time.Time // DUPLICATES fs.FileInfo.ModTime()
    Size     int64     // DUPLICATES fs.FileInfo.Size()
    MimeType string    // computed from mime.TypeByExtension
}
```

**Issue**: FileMetadata duplicates `ModTime` and `Size` from `fs.FileInfo`, and caches computed values (`Basename`, `Folder`, `Ext`, `MimeType`) that can be derived on-demand from `Path`.

**Desired State**:

```go
// Option A: Use fs.FileInfo directly
type VaultFile struct {
    Path    string
    Info    fs.FileInfo  // Delegate to stdlib interface
    Content []byte
}

// Compute derived fields on-demand as methods
func (v VaultFile) Basename() string { /* compute from Path */ }
func (v VaultFile) Folder() string   { return filepath.Dir(v.Path) }
func (v VaultFile) Ext() string      { return filepath.Ext(v.Path) }
func (v VaultFile) ModTime() time.Time { return v.Info.ModTime() }
func (v VaultFile) Size() int64      { return v.Info.Size() }
```

**Relevant Patterns**:

- **Go stdlib (io/fs)**: Use `fs.FileInfo` interface instead of duplicating fields
- **Obsidian TFile**: Stores `stat: FileStats` interface, computes `basename`/`extension` as properties

**Benefits**:

- Eliminates data duplication (ModTime, Size)
- Reduces memory footprint per file
- Single source of truth for file stats
- On-demand computation avoids stale cached values

**Trade-offs**:

- Slight computation overhead for derived fields
- Requires method calls instead of field access

**Affected Issues**:

- **Group 1, Issue A4 (DTO Redesign)**: FileMetadata is a DTO that should be simplified
- **Group 2 (Storage Architecture)**: Cached file metadata has duplication

**Recommendation**: **Adopt Option A** - Use `fs.FileInfo` directly, compute derived fields on-demand. Premature optimization of caching these values is YAGNI.

---

#### Gap 1.2: Absolute Paths vs Vault-Relative Paths

**Current State**:

```go
type FileMetadata struct {
    Path string  // Absolute filesystem path (e.g., "/Users/jack/vault/note.md")
}
```

**Issue**: Storing absolute paths makes cache non-portable across machines, complicates multi-vault scenarios, and couples to filesystem layout.

**Desired State**:

```go
type VaultFile struct {
    Path     string  // Vault-relative path (e.g., "folder/note.md")
    VaultID  string  // Vault identifier for multi-vault support
}

// Compute absolute path when needed for I/O
func (v VaultFile) AbsolutePath(vaultRoot string) string {
    return filepath.Join(vaultRoot, v.Path)
}
```

**Relevant Patterns**:

- **Go stdlib (path/filepath)**: `filepath.Join()` for combining paths, `filepath.Rel()` for relative paths
- **Obsidian TFile**: Stores `path` as vault-relative (e.g., "folder/subfolder/note.md")

**Benefits**:

- Cache is portable across machines
- Supports multi-vault scenarios cleanly
- Vault root can change without invalidating cache
- Matches Obsidian's proven pattern

**Trade-offs**:

- Must join with vault root for file I/O operations
- Requires VaultID for multi-vault disambiguation

**Affected Issues**:

- **Group 2 (Storage Architecture)**: Cache portability
- **Future**: Multi-vault support (not current requirement, but architecture should enable it)

**Recommendation**: **Adopt vault-relative paths** - Store `Path` relative to vault root, compute absolute paths on-demand for I/O. Add `VaultID` field for future multi-vault support.

---

#### Gap 1.3: VaultFile Conflates File Data and Content

**Current State**:

```go
type VaultFile struct {
    FileMetadata  // Embedded (Path, Basename, Folder, Ext, ModTime, Size, MimeType)
    Content []byte  // Raw file content
}
```

**Issue**: VaultFile combines file system metadata with file content. For operations that only need metadata (listing files, checking modification times), we load unnecessary content into memory.

**Desired State**:

```go
// Separate file metadata from content
type VaultFile struct {
    Path    string
    Info    fs.FileInfo  // Filesystem stats only
    VaultID string
}

type VaultFileContent struct {
    VaultFile  // Embedded metadata
    Content    []byte  // Loaded on-demand
}

// Storage layer provides both metadata-only and content-loaded variants
type StoragePort interface {
    ListFiles(ctx context.Context, vaultID string) ([]VaultFile, error)  // Metadata only
    ReadFile(ctx context.Context, file VaultFile) (VaultFileContent, error)  // With content
}
```

**Relevant Patterns**:

- **Obsidian Vault API**: Separate `getFiles()` (metadata) from `read(file)` (content)
- **Go idioms**: Lazy loading, load only what's needed

**Benefits**:

- Memory efficiency for metadata-only operations
- Faster vault scans (no content loading)
- Clear separation between file existence and file content

**Trade-offs**:

- Slightly more complex API (two types instead of one)

**Affected Issues**:

- **Group 2 (Storage Architecture)**: Indexing performance
- **Group 3 (Query Optimization)**: Metadata queries shouldn't load content

**Recommendation**: **Separate metadata from content** - Create distinct types for metadata-only and content-loaded scenarios. Provide port methods for both use cases.

---

### Gap Category 2: Metadata Caching Architecture

#### Gap 2.1: No Dedicated Metadata Cache

**Current State**:

- No dedicated cache for parsed frontmatter
- Frontmatter parsed on-demand from `VaultFile.Content`
- No cache invalidation strategy for metadata
- Links/headings/tags not cached

**Issue**: Re-parsing markdown frontmatter on every query is inefficient. No separation between file content (write model) and parsed metadata (read model).

**Desired State**:

```go
// Cached metadata structure (read model)
type NoteMetadataCache struct {
    Path            string
    VaultID         string
    Frontmatter     map[string]any
    Links           []Link
    Headings        []Heading
    Tags            []string
    LastParsed      time.Time
    ContentChecksum uint64  // For cache invalidation
}

// MetadataCache service (read model)
type MetadataCache interface {
    // Get cached metadata (nil if not cached)
    Get(ctx context.Context, path string) (*NoteMetadataCache, error)

    // Invalidate cache on file modification
    InvalidateFile(ctx context.Context, path string) error

    // Batch operations for vault indexing
    IndexVault(ctx context.Context, vaultID string) error
}
```

**Relevant Patterns**:

- **Obsidian MetadataCache**: Separate cache for frontmatter, links, tags, headings
- **CQRS**: Write model (file content) separate from read model (cached metadata)
- **Go stdlib**: Use checksums for cache invalidation (fnv.Sum64 from hash/fnv)

**Benefits**:

- Fast metadata queries without re-parsing
- Supports complex queries (find all notes with tag X)
- Cache invalidation prevents stale data
- Aligns with CQRS pattern from architecture

**Trade-offs**:

- Additional storage for cache
- Cache invalidation complexity
- Write operations must update cache

**Affected Issues**:

- **Group 2, Issue A5 (Storage Architecture)**: CQRS implementation
- **Group 3 (Query Performance)**: Fast metadata queries
- **Group 1, Issue B2 (Goldmark in Domain)**: Parsing moves to adapter, results cached

**Recommendation**: **Implement MetadataCache service** - Create dedicated read model for parsed metadata with cache invalidation. Use content checksums to detect stale cache entries.

---

#### Gap 2.2: No Read vs CachedRead Distinction

**Current State**:

- Single read pattern for all file operations
- No distinction between "read for display" vs "read for modification"

**Issue**: Risk of stale data overwrites - read file content, make changes, write back old content over newer changes.

**Desired State**:

```go
type FilePort interface {
    // Read from disk (always fresh, use before modification)
    Read(ctx context.Context, path string) ([]byte, error)

    // Read from cache if available (fast, use for display)
    CachedRead(ctx context.Context, path string) ([]byte, error)

    // Write with optimistic locking
    Write(ctx context.Context, path string, content []byte, expectedModTime time.Time) error
}
```

**Relevant Patterns**:

- **Obsidian Vault API**: `vault.read(file)` vs `vault.cachedRead(file)`
- **Optimistic Locking**: Check `ModTime` before write to prevent overwrites

**Benefits**:

- Prevents stale data overwrites
- Performance optimization for display-only reads
- Explicit intent in API (modify vs display)

**Trade-offs**:

- More complex API
- Clients must choose correct read method

**Affected Issues**:

- **Group 2 (Storage Architecture)**: Safe concurrent access
- **Future**: Concurrent editing scenarios

**Recommendation**: **Adopt dual read pattern** - Provide `Read()` (always fresh) and `CachedRead()` (fast) methods. Document when to use each.

---

### Gap Category 3: Markdown Parsing Architecture

#### Gap 3.1: Goldmark Used Directly in Domain

**Current State**:

```go
// /internal/app/frontmatter/service.go (domain layer)
import "github.com/yuin/goldmark"

type FrontmatterService struct {
    parser goldmark.Markdown  // Direct goldmark dependency in domain
}
```

**Issue**: Domain layer depends on external library (goldmark), violates hexagonal architecture principle (domain should only depend on stdlib and its own types).

**Desired State**:

```go
// Port defined by domain (in /internal/ports/spi)
type MarkdownParserPort interface {
    ParseMetadata(ctx context.Context, content []byte) (*domain.NoteMetadata, error)
}

// Domain service uses port
type FrontmatterService struct {
    parser MarkdownParserPort  // Depends on port interface
}

// Adapter implements port (in /internal/adapters/spi/parser)
type MDFileAdapter struct {
    md goldmark.Markdown
}

func (a *MDFileAdapter) ParseMetadata(ctx context.Context, content []byte) (*domain.NoteMetadata, error) {
    // goldmark implementation details
}
```

**Relevant Patterns**:

- **Hexagonal Architecture**: Domain defines port interfaces, adapters implement
- **Goldmark research**: Use `goldmark.New()` with frontmatter extension in adapter
- **Dependency Inversion**: High-level domain doesn't depend on low-level parser

**Benefits**:

- Domain remains framework-agnostic
- Testable (mock MarkdownParserPort)
- Can swap parsers without changing domain
- Clean architectural boundaries

**Trade-offs**:

- Additional abstraction layer
- Port interface must be designed carefully

**Affected Issues**:

- **Group 1, Issue B2 (Goldmark in Domain)**: Primary issue this addresses
- **Hexagonal Principle**: Syntactic validation (parsing) belongs in adapter

**Recommendation**: **Extract MarkdownParserPort** - Define port interface in domain, implement MDFileAdapter in adapter layer. Move frontmatter parsing to adapter.

---

#### Gap 3.2: No AST-Based Metadata Extraction

**Current State**:

- Frontmatter extracted, but links/headings/tags not systematically extracted
- No unified metadata extraction pipeline

**Issue**: Missing structured metadata that Obsidian provides (links, headings, tags). No foundation for graph queries or backlinks.

**Desired State**:

```go
type NoteMetadata struct {
    Frontmatter map[string]any
    Links       []Link      // Both [[wikilinks]] and [markdown](links)
    Headings    []Heading   // # Headings with levels
    Tags        []string    // #tags
    Blocks      map[string]BlockRef  // ^block-refs
}

type MarkdownParserPort interface {
    // Single parse, extract all metadata
    ParseMetadata(ctx context.Context, content []byte) (*NoteMetadata, error)
}
```

**Relevant Patterns**:

- **Goldmark AST walking**: Use `ast.Walk()` to extract links, headings, tags in single pass
- **Obsidian CachedMetadata**: Structured metadata for links, embeds, headings, tags

**Benefits**:

- Enables graph queries (find backlinks)
- Foundation for search by heading/tag
- Single parse extracts all metadata (efficient)

**Trade-offs**:

- More complex parser adapter
- Additional storage for metadata

**Affected Issues**:

- **Group 1, Issue B2 (Markdown Parsing)**: Comprehensive metadata extraction
- **Future**: Graph view, backlinks (not current MVP, but architecture should enable)

**Recommendation**: **Implement comprehensive AST-based extraction** - Use goldmark AST walking to extract links, headings, tags. Store in `NoteMetadata` structure.

---

### Gap Category 4: Storage Layer Architecture

#### Gap 4.1: No Indexed Metadata Query Layer

**Current State**:

CQRS read/write separation already implemented:

```go
// /internal/ports/spi/cache.go
type CacheWriterPort interface {
    Persist(ctx context.Context, note domain.Note) error
    Delete(ctx context.Context, id domain.NoteID) error
}

type CacheReaderPort interface {
    Read(ctx context.Context, id domain.NoteID) (domain.Note, error)
    List(ctx context.Context) ([]domain.Note, error)
}

// /internal/ports/spi/vault.go
type VaultWriterPort interface {
    Persist(ctx context.Context, note domain.Note, path string) error
    WriteContent(ctx context.Context, path string, content []byte) error
    Delete(ctx context.Context, path string) error
}

type VaultReaderPort interface {
    Read(ctx context.Context, path string) (dto.VaultFile, error)
}

type VaultScannerPort interface {
    ScanAll(ctx context.Context) ([]dto.VaultFile, error)
    ScanModified(ctx context.Context, since time.Time) ([]dto.VaultFile, error)
}
```

**Issue**: While CQRS read/write separation exists, there's **no indexed metadata query layer**. Complex queries (find all notes with tag X, find notes linking to Y) require:

1. `CacheReaderPort.List()` to get all notes
2. Iterate through all notes, filtering by frontmatter fields
3. O(n) scanning instead of O(1) indexed lookup

No support for:

- Indexed queries by tag, frontmatter field
- Backlink queries (find notes linking to X)
- Full-text search on metadata

**Desired State**:

```go
// New indexed metadata query port (layer on top of existing cache)
type MetadataQueryPort interface {
    // Indexed queries - O(1) lookup instead of O(n) scan
    TagQuery(ctx context.Context, tag string) ([]domain.Note, error)
    FrontmatterQuery(ctx context.Context, field string, value any) ([]domain.Note, error)
    BacklinksQuery(ctx context.Context, notePath string) ([]domain.Note, error)

    // Full-text search on cached metadata
    Search(ctx context.Context, query string) ([]domain.Note, error)
}

// Synchronization service maintains indices from cached notes
type MetadataIndexer interface {
    // Build indices from existing cache
    Build(ctx context.Context) error

    // Update indices when note changes (called after CacheWriterPort.Persist)
    IndexNote(ctx context.Context, note domain.Note) error
    RemoveNote(ctx context.Context, id domain.NoteID) error
}
```

**Relevant Patterns**:

- **Obsidian MetadataCache**: Separate indexed metadata for fast queries
- **BoltDB/SQLite research**: BoltDB for hot indices (<1ms), SQLite for complex queries (<50ms)
- **CQRS Enhancement**: Add indexed query layer on top of existing CQRS separation

**Benefits**:

- Fast O(1) tag/field queries vs O(n) scanning
- Enables backlink graph queries
- Foundation for full-text search
- Complements existing CQRS separation

**Trade-offs**:

- Index maintenance complexity
- Additional storage for indices
- Synchronization between cache and indices

**Affected Issues**:

- **Group 2, Issue A5 (Storage Architecture)**: Enhanced CQRS with indexed queries
- **Group 3 (Query Performance)**: Fast metadata queries without scanning

**Recommendation**: **Add MetadataQueryPort and MetadataIndexer** - Build indexed query layer on top of existing CacheReaderPort/CacheWriterPort. Use BoltDB secondary indices or SQLite for complex queries.

---

#### Gap 4.2: No Hot Cache vs Deep Storage Separation

**Current State**:

- File-based JSON cache (`.lithos/cache/*.json`)
- No distinction between hot cache (frequently accessed) and deep storage (historical queries)

**Issue**: Single storage layer doesn't optimize for different access patterns (recent files vs historical queries).

**Desired State**:

```go
// Hot Cache (BoltDB) - <1ms access, recent/active files
type HotCachePort interface {
    Get(ctx context.Context, key string) (*CachedMetadata, error)
    Set(ctx context.Context, key string, metadata *CachedMetadata) error
    Delete(ctx context.Context, key string) error
}

// Deep Storage (SQLite) - <50ms queries, all historical data
type DeepStoragePort interface {
    QueryComplex(ctx context.Context, query MetadataQuery) ([]NoteMetadata, error)
    DateRangeQuery(ctx context.Context, start, end time.Time) ([]NoteMetadata, error)
}

// Cache promotion strategy
type CachePromoter interface {
    PromoteToHotCache(ctx context.Context, path string) error
    EvictFromHotCache(ctx context.Context, path string) error
}
```

**Relevant Patterns**:

- **BoltDB research**: Nested buckets for hot cache, <1ms View() latency
- **SQLite research**: JSON columns with VIRTUAL/STORED columns for deep queries
- **Architecture Document**: "Hybrid storage (BoltDB + SQLite)"

**Benefits**:

- Hot cache optimizes for frequent access (<1ms)
- Deep storage optimizes for complex queries (<50ms)
- Cache promotion based on access patterns
- Efficient use of resources

**Trade-offs**:

- Two storage systems to manage
- Promotion/eviction strategy complexity
- Data consistency between layers

**Affected Issues**:

- **Group 2, Issue A5 (Storage Architecture)**: Hybrid BoltDB + SQLite
- **Group 3 (Query Performance)**: Performance targets

**Recommendation**: **Implement hybrid storage** - Use BoltDB for hot cache (recent files, <1ms), SQLite for deep storage (all data, complex queries, <50ms). Implement cache promotion based on access patterns.

---

### Gap Category 5: Template Architecture

#### Gap 5.1: Template Caching Not Leveraging text/template Composition

**Current State**:

```go
// /internal/app/template/service.go
type TemplateEngine struct {
    compiled map[domain.TemplateID]cachedTemplate  // Custom caching
}

type cachedTemplate struct {
    tpl      *template.Template
    checksum uint64
}
```

**Issue**: Custom caching implementation when `text/template` provides built-in template composition and caching.

**Desired State**:

```go
// Leverage text/template's built-in composition
type TemplateEngine struct {
    // Single template with associated templates
    root *template.Template
}

func (e *TemplateEngine) LoadTemplates(ctx context.Context) error {
    // Load all templates into single namespace
    e.root = template.New("root")

    for _, tmpl := range templates {
        _, err := e.root.New(tmpl.ID).Parse(tmpl.Content)
        if err != nil {
            return err
        }
    }
    return nil
}

func (e *TemplateEngine) Render(ctx context.Context, templateID string) (string, error) {
    // Execute by name - text/template handles caching
    var buf strings.Builder
    if err := e.root.ExecuteTemplate(&buf, templateID, nil); err != nil {
        return "", err
    }
    return buf.String(), nil
}
```

**Relevant Patterns**:

- **text/template research**: Use `ExecuteTemplate(name)` for template composition
- **text/template research**: Built-in template association and lookup

**Benefits**:

- Leverage stdlib caching instead of custom implementation
- Template composition (define shared blocks)
- Less code to maintain
- Well-tested stdlib implementation

**Trade-offs**:

- Less control over caching strategy
- Must reload all templates on any change (vs individual invalidation)

**Affected Issues**:

- **Group 1 (Template Engine)**: Simplify caching implementation
- **Code Quality**: Reduce custom code, use stdlib

**Recommendation**: **Use text/template composition** - Load all templates into single namespace, use `ExecuteTemplate(name)` instead of custom caching. Re-evaluate if individual template invalidation becomes requirement.

---

### Gap Category 6: Type Safety and Generics

#### Gap 6.1: No Generic Cache Implementation

**Current State**:

- Custom caching for templates (map[TemplateID]cachedTemplate)
- Future: Custom caching for metadata (map[string]CachedMetadata)
- No reusable cache abstraction

**Issue**: Duplicating cache logic for different types. No type-safe, reusable cache implementation.

**Desired State**:

```go
// Generic cache with TTL and size limits
type Cache[K comparable, V any] struct {
    data    map[K]cacheEntry[V]
    maxSize int
    mu      sync.RWMutex
}

type cacheEntry[V any] struct {
    value      V
    expiry     time.Time
    lastAccess time.Time
}

func NewCache[K comparable, V any](maxSize int, ttl time.Duration) *Cache[K, V] {
    return &Cache[K, V]{
        data:    make(map[K]cacheEntry[V]),
        maxSize: maxSize,
    }
}

func (c *Cache[K, V]) Get(key K) (V, bool) { /* thread-safe get */ }
func (c *Cache[K, V]) Set(key K, value V) { /* thread-safe set with LRU eviction */ }
func (c *Cache[K, V]) Delete(key K) { /* thread-safe delete */ }
func (c *Cache[K, V]) Clear() { /* clear all entries */ }

// Usage
templateCache := NewCache[TemplateID, *template.Template](100, 1*time.Hour)
metadataCache := NewCache[string, *CachedMetadata](10000, 5*time.Minute)
```

**Relevant Patterns**:

- **Go Generics research**: Generic infrastructure (Cache, collections) appropriate
- **Go Generics research**: Avoid generics for domain types

**Benefits**:

- Type-safe caching without interface{} and type assertions
- Reusable across all cache needs (templates, metadata, schemas)
- Single implementation, multiple uses
- LRU eviction, TTL, thread-safety built-in

**Trade-offs**:

- Slight performance overhead vs hand-tuned caches
- Requires Go 1.18+ (already required per tech-stack.md)

**Affected Issues**:

- **Cross-cutting**: Reusable infrastructure
- **Group 2 (Storage Architecture)**: Metadata caching
- **Type Safety**: Eliminate interface{} usage

**Recommendation**: **Implement generic Cache[K,V]** - Create reusable cache infrastructure with LRU, TTL, thread-safety. Use for templates, metadata, schemas.

---

### Gap Category 7: Path Handling

#### Gap 7.1: Platform-Specific Path Storage

**Current State**:

- Paths stored with platform-specific separators
- No normalization to forward slashes for storage

**Issue**: Cache non-portable across Windows/Unix systems. Path comparisons fragile.

**Desired State**:

```go
import "path/filepath"

// Always store paths with forward slashes (portable)
func NormalizePath(path string) string {
    return filepath.ToSlash(filepath.Clean(path))
}

// Convert to platform-specific for I/O
func LocalizePath(path string) string {
    return filepath.FromSlash(path)
}

type VaultFile struct {
    Path string  // Always stored with forward slashes
}

// I/O operations convert to platform-specific
func (a *FileAdapter) Read(ctx context.Context, file VaultFile) ([]byte, error) {
    localPath := filepath.FromSlash(file.Path)
    return os.ReadFile(localPath)
}
```

**Relevant Patterns**:

- **path/filepath research**: `ToSlash()` for portable storage, `FromSlash()` for platform I/O
- **path/filepath research**: `Clean()` to normalize separators and resolve . and ..

**Benefits**:

- Cache portable across platforms
- Consistent path comparison
- No platform-specific bugs in path handling

**Trade-offs**:

- Must convert for I/O operations
- Slight overhead of conversion

**Affected Issues**:

- **Group 2 (Storage Architecture)**: Cache portability
- **Cross-platform**: Windows vs Unix path handling

**Recommendation**: **Normalize paths for storage** - Use `filepath.ToSlash()` when storing paths, `filepath.FromSlash()` for I/O. Document convention.

---

### Gap Summary

| Gap ID | Category                  | Priority | Affected Groups | Complexity | Impact                                     |
| ------ | ------------------------- | -------- | --------------- | ---------- | ------------------------------------------ |
| 1.1    | FileMetadata Duplication  | HIGH     | 1 (DTO), 2      | LOW        | Eliminate duplication, use fs.FileInfo     |
| 1.2    | Absolute Paths            | MEDIUM   | 2, Future       | LOW        | Cache portability, multi-vault support     |
| 1.3    | File + Content Conflation | MEDIUM   | 2, 3            | MEDIUM     | Memory efficiency, faster metadata queries |
| 2.1    | No Metadata Cache         | HIGH     | 2, 3, 1 (B2)    | HIGH       | Query performance, CQRS implementation     |
| 2.2    | Single Read Pattern       | LOW      | 2               | LOW        | Prevent stale overwrites                   |
| 3.1    | Goldmark in Domain        | HIGH     | 1 (B2)          | MEDIUM     | Hexagonal architecture compliance          |
| 3.2    | No AST Metadata           | MEDIUM   | 1 (B2), Future  | MEDIUM     | Enable graph queries, backlinks            |
| 4.1    | No Indexed Query Layer    | HIGH     | 2, 3            | HIGH       | O(1) indexed queries vs O(n) scanning      |
| 4.2    | No Hot/Deep Separation    | MEDIUM   | 2, 3            | HIGH       | Performance targets (<1ms hot, <50ms deep) |
| 5.1    | Custom Template Cache     | LOW      | 1               | LOW        | Simplify using stdlib composition          |
| 6.1    | No Generic Cache          | MEDIUM   | Cross-cutting   | MEDIUM     | Type-safe infrastructure, reusability      |
| 7.1    | Platform-Specific Paths   | LOW      | 2               | LOW        | Cross-platform portability                 |

**Priority Definitions**:

- **HIGH**: Blocks architectural goals or Epic 3 progress
- **MEDIUM**: Improves architecture but not blocking
- **LOW**: Nice-to-have, can be deferred to post-Epic 3

**Next Steps**: Proceed to Entity Review Scope to evaluate domain entities for anemic model anti-pattern and missing behavior, then synthesize comprehensive Epic Impact Assessment.

---

### Actionable Insights from Research & Gap Analysis

**Status**: ✅ Complete (2025-11-08)

This section provides concrete, actionable recommendations derived from research findings and gap analysis, prioritized for Epic 3 implementation.

#### Priority 1: Critical Architectural Fixes (Must Address for Epic 3)

**AI-1.1: Eliminate FileMetadata Duplication** (Gap 1.1 - HIGH)

```go
// Action: Replace FileMetadata with fs.FileInfo usage
// Location: /internal/shared/dto/file.go

// BEFORE (Current):
type FileMetadata struct {
    Path     string
    Basename string    // computed - REMOVE
    Folder   string    // computed - REMOVE
    Ext      string    // computed - REMOVE
    ModTime  time.Time // duplicate - REMOVE
    Size     int64     // duplicate - REMOVE
    MimeType string    // computed - REMOVE
}

// AFTER (Target):
type VaultFile struct {
    Path    string         // Vault-relative path (see AI-1.2)
    Info    fs.FileInfo    // Delegate to stdlib
    Content []byte         // Loaded on-demand
}

// Add computed methods
func (v VaultFile) Basename() string {
    base := filepath.Base(v.Path)
    return strings.TrimSuffix(base, filepath.Ext(base))
}
func (v VaultFile) Folder() string   { return filepath.Dir(v.Path) }
func (v VaultFile) Ext() string      { return filepath.Ext(v.Path) }
func (v VaultFile) ModTime() time.Time { return v.Info.ModTime() }
func (v VaultFile) Size() int64      { return v.Info.Size() }
```

**Impact**: Eliminates 5 duplicated/computed fields, uses stdlib interface, reduces memory per file
**Effort**: LOW - Simple refactoring, update all FileMetadata usages
**Risk**: LOW - Well-defined stdlib interface, backward compatible with methods

---

**AI-1.2: Adopt Vault-Relative Paths** (Gap 1.2 - MEDIUM)

```go
// Action: Store paths relative to vault root
// Location: /internal/shared/dto/file.go, all cache adapters

// BEFORE:
type VaultFile struct {
    Path string  // Absolute: "/Users/jack/vault/notes/meeting.md"
}

// AFTER:
type VaultFile struct {
    Path    string  // Relative: "notes/meeting.md"
    VaultID string  // For multi-vault: "default"
}

// Helper for I/O operations
func (v VaultFile) AbsolutePath(vaultRoot string) string {
    return filepath.Join(vaultRoot, v.Path)
}

// Normalize paths when storing
func NormalizePath(absPath, vaultRoot string) (string, error) {
    relPath, err := filepath.Rel(vaultRoot, absPath)
    if err != nil {
        return "", err
    }
    return filepath.ToSlash(relPath), nil  // Forward slashes for portability
}
```

**Impact**: Cache portable across machines, enables multi-vault, matches Obsidian pattern
**Effort**: MEDIUM - Update cache read/write adapters, path normalization
**Risk**: MEDIUM - Must handle path resolution correctly, test Windows compatibility

---

**AI-1.3: Extract MarkdownParserPort** (Gap 3.1 - HIGH)

```go
// Action: Move goldmark from domain to adapter layer
// Location: Create /internal/ports/spi/markdown.go
//           Move /internal/app/frontmatter/service.go to use port

// New Port Interface (in /internal/ports/spi/markdown.go):
type MarkdownParserPort interface {
    // ParseMetadata extracts frontmatter, links, headings, tags from markdown
    ParseMetadata(ctx context.Context, content []byte) (*domain.NoteMetadata, error)
}

// New Domain Model (in /internal/domain/note.go):
type NoteMetadata struct {
    Frontmatter map[string]any
    Links       []Link
    Headings    []Heading
    Tags        []string
}

type Link struct {
    Text        string
    Destination string
    IsWikilink  bool
}

type Heading struct {
    Level int
    Text  string
}

// Adapter Implementation (in /internal/adapters/spi/parser/goldmark.go):
type MDFileAdapter struct {
    md goldmark.Markdown
}

func NewMDFileAdapter() *MDFileAdapter {
    md := goldmark.New(
        goldmark.WithExtensions(&frontmatter.Extender{}),
    )
    return &MDFileAdapter{md: md}
}

func (a *MDFileAdapter) ParseMetadata(ctx context.Context, content []byte) (*domain.NoteMetadata, error) {
    // Use goldmark AST walking to extract all metadata
    // See goldmark research for implementation
}
```

**Impact**: Hexagonal architecture compliance, domain framework-agnostic, enables AST metadata extraction
**Effort**: MEDIUM - Create port, implement adapter, update FrontmatterService
**Risk**: LOW - Clear separation, goldmark stays same, just moved to adapter

---

**AI-1.4: Implement MetadataQueryPort** (Gap 4.1 - HIGH)

```go
// Action: Add indexed query layer on top of existing cache
// Location: Create /internal/ports/spi/metadata_query.go

// New Query Port (complementing existing CacheReaderPort):
type MetadataQueryPort interface {
    // Indexed queries - O(1) instead of List() + filter (O(n))
    TagQuery(ctx context.Context, tag string) ([]domain.Note, error)
    FrontmatterQuery(ctx context.Context, field string, value any) ([]domain.Note, error)
    BacklinksQuery(ctx context.Context, notePath string) ([]domain.Note, error)

    // Full-text search
    Search(ctx context.Context, query string) ([]domain.Note, error)
}

// Index Synchronization Service:
type MetadataIndexer interface {
    // Build indices from CacheReaderPort.List()
    Build(ctx context.Context) error

    // Update indices after CacheWriterPort.Persist()
    IndexNote(ctx context.Context, note domain.Note) error
    RemoveNote(ctx context.Context, id domain.NoteID) error
}

// BoltDB Adapter Example (for hot queries <1ms):
type BoltDBQueryAdapter struct {
    db *bolt.DB
}

func (a *BoltDBQueryAdapter) TagQuery(ctx context.Context, tag string) ([]domain.Note, error) {
    // Use secondary index bucket: "indices:by_tag"
    // Key: tag → Value: []NoteID
    // Then lookup notes from "notes" bucket
}
```

**Impact**: O(1) indexed queries vs O(n) scanning, enables backlinks/graph queries
**Effort**: HIGH - New port, BoltDB/SQLite adapter, index maintenance
**Risk**: MEDIUM - Index synchronization complexity, cache invalidation

---

#### Priority 2: Performance & Architecture Improvements (Should Address)

**AI-2.1: Implement MetadataCache Service** (Gap 2.1 - HIGH)

```go
// Action: Separate parsed metadata cache from file content
// Location: Create /internal/app/metadata/cache_service.go

type NoteMetadataCache struct {
    Path            string
    VaultID         string
    Frontmatter     map[string]any
    Links           []domain.Link
    Headings        []domain.Heading
    Tags            []string
    LastParsed      time.Time
    ContentChecksum uint64  // For invalidation
}

type MetadataCacheService struct {
    cache      map[string]*NoteMetadataCache
    parser     spi.MarkdownParserPort
    fileReader spi.VaultReaderPort
    mu         sync.RWMutex
}

func (s *MetadataCacheService) GetOrParse(ctx context.Context, path string) (*NoteMetadataCache, error) {
    // Check cache first
    if cached := s.getCached(path); cached != nil {
        // Verify checksum
        currentChecksum := s.computeChecksum(path)
        if cached.ContentChecksum == currentChecksum {
            return cached, nil
        }
    }

    // Parse and cache
    file, err := s.fileReader.Read(ctx, path)
    if err != nil {
        return nil, err
    }

    metadata, err := s.parser.ParseMetadata(ctx, file.Content)
    if err != nil {
        return nil, err
    }

    cached := &NoteMetadataCache{
        Path:            path,
        Frontmatter:     metadata.Frontmatter,
        Links:           metadata.Links,
        Headings:        metadata.Headings,
        Tags:            metadata.Tags,
        LastParsed:      time.Now(),
        ContentChecksum: s.computeChecksum(path),
    }

    s.setCached(path, cached)
    return cached, nil
}
```

**Impact**: Avoid re-parsing markdown on every query, CQRS read model
**Effort**: MEDIUM - New service, cache management, invalidation logic
**Risk**: LOW - Clear responsibility, uses existing ports

---

**AI-2.2: Separate File Metadata from Content** (Gap 1.3 - MEDIUM)

```go
// Action: Create distinct types for metadata-only vs content-loaded
// Location: /internal/shared/dto/file.go

// Metadata only (lightweight)
type VaultFile struct {
    Path    string
    Info    fs.FileInfo
    VaultID string
}

// With content (loaded on-demand)
type VaultFileContent struct {
    VaultFile        // Embedded metadata
    Content    []byte
}

// Update Port Interfaces:
type VaultReaderPort interface {
    // Metadata only - fast directory listing
    GetFileInfo(ctx context.Context, path string) (dto.VaultFile, error)

    // With content - when needed
    Read(ctx context.Context, path string) (dto.VaultFileContent, error)
}

type VaultScannerPort interface {
    // Metadata only - fast vault scanning
    ScanAll(ctx context.Context) ([]dto.VaultFile, error)
}
```

**Impact**: Memory efficiency for metadata-only operations, faster vault scans
**Effort**: MEDIUM - Update DTOs, port interfaces, adapter implementations
**Risk**: LOW - Additive change, existing Read() still works

---

**AI-2.3: Use text/template Composition** (Gap 5.1 - LOW)

```go
// Action: Replace custom template caching with stdlib composition
// Location: /internal/app/template/service.go

// BEFORE:
type TemplateEngine struct {
    compiled map[domain.TemplateID]cachedTemplate
}

type cachedTemplate struct {
    tpl      *template.Template
    checksum uint64
}

// AFTER:
type TemplateEngine struct {
    root *template.Template  // Single namespace with all templates
}

func (e *TemplateEngine) LoadTemplates(ctx context.Context) error {
    e.root = template.New("root").Funcs(e.buildFuncMap())

    templates, err := e.templatePort.LoadAll(ctx)
    if err != nil {
        return err
    }

    for _, tmpl := range templates {
        _, err := e.root.New(string(tmpl.ID)).Parse(tmpl.Content)
        if err != nil {
            return fmt.Errorf("parse template %s: %w", tmpl.ID, err)
        }
    }
    return nil
}

func (e *TemplateEngine) Render(ctx context.Context, templateID domain.TemplateID) (string, error) {
    var buf strings.Builder
    if err := e.root.ExecuteTemplate(&buf, string(templateID), nil); err != nil {
        return "", err
    }
    return buf.String(), nil
}
```

**Impact**: Simplify code, use stdlib caching, enable template composition
**Effort**: LOW - Straightforward refactoring
**Risk**: LOW - stdlib is well-tested, existing functionality preserved

---

#### Priority 3: Future Enhancements (Can Defer Post-Epic 3)

**AI-3.1: Implement Generic Cache[K,V]** (Gap 6.1 - MEDIUM)

```go
// Action: Create reusable generic cache infrastructure
// Location: /internal/shared/cache/generic.go

type Cache[K comparable, V any] struct {
    data    map[K]cacheEntry[V]
    maxSize int
    ttl     time.Duration
    mu      sync.RWMutex
}

type cacheEntry[V any] struct {
    value      V
    expiry     time.Time
    lastAccess time.Time
}

func NewCache[K comparable, V any](maxSize int, ttl time.Duration) *Cache[K, V] {
    return &Cache[K, V]{
        data:    make(map[K]cacheEntry[V]),
        maxSize: maxSize,
        ttl:     ttl,
    }
}

func (c *Cache[K, V]) Get(key K) (V, bool) {
    c.mu.RLock()
    defer c.mu.RUnlock()

    entry, exists := c.data[key]
    if !exists || time.Now().After(entry.expiry) {
        var zero V
        return zero, false
    }

    // Update last access for LRU
    entry.lastAccess = time.Now()
    c.data[key] = entry

    return entry.value, true
}

func (c *Cache[K, V]) Set(key K, value V) {
    c.mu.Lock()
    defer c.mu.Unlock()

    // LRU eviction if at max size
    if len(c.data) >= c.maxSize {
        c.evictLRU()
    }

    c.data[key] = cacheEntry[V]{
        value:      value,
        expiry:     time.Now().Add(c.ttl),
        lastAccess: time.Now(),
    }
}

// Usage:
templateCache := NewCache[domain.TemplateID, *template.Template](100, 1*time.Hour)
metadataCache := NewCache[string, *NoteMetadataCache](10000, 5*time.Minute)
```

**Impact**: Type-safe caching, reusable across all cache needs
**Effort**: MEDIUM - Implement cache, LRU eviction, TTL
**Risk**: LOW - Optional enhancement, doesn't block core functionality

---

**AI-3.2: Normalize Path Storage** (Gap 7.1 - LOW)

```go
// Action: Store paths with forward slashes for cross-platform portability
// Location: /internal/shared/dto/file.go, all cache adapters

func NormalizePath(path string) string {
    return filepath.ToSlash(filepath.Clean(path))
}

func LocalizePath(path string) string {
    return filepath.FromSlash(path)
}

// In VaultFile constructor:
func NewVaultFile(absPath, vaultRoot string, info fs.FileInfo) (VaultFile, error) {
    relPath, err := filepath.Rel(vaultRoot, absPath)
    if err != nil {
        return VaultFile{}, err
    }

    return VaultFile{
        Path: NormalizePath(relPath),  // Forward slashes for storage
        Info: info,
    }, nil
}

// In file I/O:
func (a *FileAdapter) Read(ctx context.Context, file VaultFile) ([]byte, error) {
    absPath := filepath.Join(a.vaultRoot, LocalizePath(file.Path))  // Platform-specific for I/O
    return os.ReadFile(absPath)
}
```

**Impact**: Cross-platform path portability
**Effort**: LOW - Add normalization functions, update constructors
**Risk**: LOW - Path utilities well-tested

---

**AI-3.3: Implement Read vs CachedRead Pattern** (Gap 2.2 - LOW)

```go
// Action: Add dual read methods to prevent stale overwrites
// Location: Update existing VaultReaderPort

type VaultReaderPort interface {
    // Read from disk (always fresh) - use before modification
    Read(ctx context.Context, path string) (dto.VaultFileContent, error)

    // Read from cache (fast) - use for display
    CachedRead(ctx context.Context, path string) (dto.VaultFileContent, error)
}

type VaultWriterPort interface {
    // Write with optimistic locking
    Write(ctx context.Context, path string, content []byte, expectedModTime time.Time) error
}
```

**Impact**: Prevent stale data overwrites
**Effort**: LOW - Add CachedRead method, implement cache
**Risk**: LOW - Additive change, optional optimization

---

#### Implementation Sequence for Epic 3

**Phase 1: Foundation (Stories 3.1-3.2)**

1. AI-1.1: FileMetadata → fs.FileInfo (LOW effort, HIGH impact)
2. AI-1.2: Vault-relative paths (MEDIUM effort, enables portability)
3. AI-1.3: Extract MarkdownParserPort (MEDIUM effort, hexagonal compliance)

**Phase 2: Query Architecture (Stories 3.3-3.4)** 4. AI-1.4: MetadataQueryPort (HIGH effort, enables indexed queries) 5. AI-2.1: MetadataCacheService (MEDIUM effort, performance)

**Phase 3: Optimization (Story 3.5)** 6. AI-2.2: Separate file metadata/content (MEDIUM effort, memory efficiency) 7. AI-2.3: text/template composition (LOW effort, code simplification)

**Phase 4: Post-MVP** 8. AI-3.1: Generic Cache[K,V] (MEDIUM effort, nice-to-have) 9. AI-3.2: Path normalization (LOW effort, cross-platform) 10. AI-3.3: Read/CachedRead (LOW effort, safety enhancement)

**Estimated Total Effort**:

- Phase 1: 3-4 stories
- Phase 2: 2-3 stories
- Phase 3: 1-2 stories
- **Epic 3 Total: 6-9 stories**

---

## Entity Review Scope

**Status**: ✅ Complete (2025-11-08)

This section provides systematic evaluation of all domain entities for architecture compliance, anemic model anti-patterns, and missing behavior. Each entity is assessed against hexagonal architecture principles and DDD best practices.

### System 1: Schema System

#### Entity 1.1: Schema (/internal/domain/schema.go)

**Current State**:

```go
type Schema struct {
    Name               string
    Extends            string
    Excludes           []string
    Properties         []Property
    ResolvedProperties []Property
}

// Has Validate() method
func (s *Schema) Validate(ctx context.Context) error
```

**Assessment**:

✅ **CORRECT** - Rich domain model with behavior

- Has `Validate()` method with domain logic (not infrastructure)
- Validates structural integrity: name not empty, excludes only with extends, unique properties
- Defensive copies in `NewSchema()` constructor
- Short-circuit validation caching with `validated` flag

❌ **ISSUE** - ResolvedProperties leaks adapter concern into domain

- `ResolvedProperties` is populated by `SchemaResolver` service (adapter layer)
- Domain entity should not have fields populated by adapters
- Breaks clean domain/adapter separation

**Recommendation**:

```go
// OPTION A: Move resolution to domain service
type SchemaResolver interface {  // Domain service
    Resolve(ctx context.Context, schema Schema, bank PropertyBank) (ResolvedSchema, error)
}

type ResolvedSchema struct {  // Separate type
    Schema
    ResolvedProperties []Property
}

// OPTION B: Keep in Schema but document clearly
// ResolvedProperties remains but only used after SchemaResolver.Resolve()
// Document that Schema is not fully usable until resolution
```

**Priority**: MEDIUM - Architecture purist fix, doesn't block functionality

---

#### Entity 1.2: Property (/internal/domain/property.go)

**Current State**:

```go
type Property struct {
    ID        string  // Auto-generated from hash(Name + Spec)
    Name      string
    Required  bool
    Array     bool
    Spec      PropertySpec
    validated bool  // Short-circuit flag
}

// Has Validate() method
func (p *Property) Validate(ctx context.Context) error
```

**Assessment**:

✅ **CORRECT** - DDD Entity with identity and behavior

- Has unique `ID` (entity identity requirement)
- Has `Validate()` with domain logic
- Delegates type-specific validation to `PropertySpec` (good composition)
- Short-circuit caching with `validated` flag

✅ **GOOD PRACTICE** - Deterministic ID generation

- Uses `sha256(name + spec)` for reproducible IDs
- Ensures same property definition always gets same ID

⚠️ **MINOR ISSUE** - `validated` flag is mutable state

- Breaks immutability principle
- Could cause issues with concurrent validation

**Recommendation**:

```go
// Remove mutable validated flag
// Let caller cache validation results if needed
type Property struct {
    ID       string
    Name     string
    Required bool
    Array    bool
    Spec     PropertySpec
    // Remove: validated bool
}

// Validation is stateless
func (p Property) Validate(ctx context.Context) error {
    // No short-circuit, but validation is cheap
}
```

**Priority**: LOW - Minor optimization, not architecture-breaking

---

#### Entity 1.3: PropertyBank (/internal/domain/property_bank.go)

**Current State**:

```go
type PropertyBank struct {
    Properties map[string]Property
}

// Lookup returns property by ID
func (pb *PropertyBank) Lookup(id string) (Property, bool)
```

**Assessment**:

✅ **CORRECT** - Singleton registry pattern

- Single instance per application lifecycle
- Defensive copy in `NewPropertyBank()`
- Validates all properties on construction

⚠️ **DESIGN QUESTION** - Is this domain or infrastructure?

- Acts as repository (typically infrastructure)
- But contains domain logic (property definitions)
- **Decision**: Correctly placed in domain as it's a domain registry

✅ **GOOD PRACTICE** - Returns copies for immutability

- `Lookup()` returns property copy, not reference

**No changes needed** - Well-designed singleton registry

**Priority**: N/A - No issues found

---

### System 2: Note System

#### Entity 2.1: Note (/internal/domain/note.go)

**Current State**:

```go
type Note struct {
    ID          NoteID
    Frontmatter Frontmatter
}

// Only behavior: delegate to Frontmatter
func (n Note) SchemaName() string {
    return n.Frontmatter.SchemaName()
}
```

**Assessment**:

❌ **ANEMIC MODEL** - Pure data structure with no behavior

- Only has one delegating method
- No validation, no business logic
- Acts as simple data container

**Missing Behavior**:

1. Note-level validation (beyond frontmatter)
2. Schema compliance checking
3. Link resolution
4. Tag management
5. Note lifecycle methods (create, update timestamps)

**Recommendation**:

```go
// Add rich behavior
type Note struct {
    ID          NoteID
    Frontmatter Frontmatter
    CreatedAt   time.Time
    UpdatedAt   time.Time
}

// Validation
func (n Note) Validate(ctx context.Context, schema Schema) error {
    // Validate frontmatter against schema
    return ValidateFrontmatter(n.Frontmatter, schema)
}

// Schema checking
func (n Note) IsValid() bool {
    return n.Frontmatter.FileClass != ""
}

// Factory method
func NewNote(id NoteID, frontmatter Frontmatter) Note {
    now := time.Now()
    return Note{
        ID:          id,
        Frontmatter: frontmatter,
        CreatedAt:   now,
        UpdatedAt:   now,
    }
}

// Update with timestamp
func (n *Note) UpdateFrontmatter(fm Frontmatter) {
    n.Frontmatter = fm
    n.UpdatedAt = time.Now()
}
```

**Priority**: HIGH - Core domain entity should have behavior

---

#### Entity 2.2: Frontmatter (/internal/domain/note.go)

**Current State**:

```go
type Frontmatter struct {
    FileClass string
    Fields    map[string]interface{}
}

// Only behavior: accessor
func (f Frontmatter) SchemaName() string {
    return f.FileClass
}
```

**Assessment**:

❌ **ANEMIC MODEL** - Pure data structure

- Only has accessor method
- No validation
- No field manipulation methods

**Missing Behavior**:

1. Field get/set with type safety
2. Validation against schema
3. Required field checking
4. Default value application

**Recommendation**:

```go
type Frontmatter struct {
    FileClass string
    Fields    map[string]any
}

// Type-safe field access
func (f Frontmatter) GetString(key string) (string, bool) {
    val, ok := f.Fields[key]
    if !ok {
        return "", false
    }
    str, ok := val.(string)
    return str, ok
}

func (f Frontmatter) GetStringArray(key string) ([]string, bool) {
    val, ok := f.Fields[key]
    if !ok {
        return nil, false
    }
    // Type conversion logic
}

// Validation
func (f Frontmatter) ValidateAgainstSchema(schema Schema) error {
    for _, prop := range schema.ResolvedProperties {
        if err := f.validateProperty(prop); err != nil {
            return err
        }
    }
    return nil
}

func (f Frontmatter) validateProperty(prop Property) error {
    val, exists := f.Fields[prop.Name]

    if prop.Required && !exists {
        return fmt.Errorf("required field %s missing", prop.Name)
    }

    if exists {
        return prop.Spec.Validate(val)
    }

    return nil
}

// Field manipulation
func (f Frontmatter) WithField(key string, value any) Frontmatter {
    newFields := make(map[string]any, len(f.Fields)+1)
    for k, v := range f.Fields {
        newFields[k] = v
    }
    newFields[key] = value

    return Frontmatter{
        FileClass: f.FileClass,
        Fields:    newFields,
    }
}
```

**Priority**: HIGH - Core domain model needs behavior

---

### System 3: Template System

#### Entity 3.1: Template (/internal/domain/template.go)

**Current State**:

```go
type Template struct {
    ID      TemplateID
    Content string
}
```

**Assessment**:

✅ **INTENTIONALLY ANEMIC** - Explicitly documented as pure data

- Godoc states: "anemic domain model pattern where business logic resides in services"
- Template rendering is infrastructure (uses text/template)
- Validation is syntactic (belongs in adapter)

✅ **CORRECT SEPARATION**:

- Domain: Template identity and content (what it is)
- Service: TemplateEngine (what to do with it)
- Adapter: TemplatePort for loading

**No changes needed** - Correct hexagonal separation

**Priority**: N/A - Correctly designed

---

### System 4: Configuration System

#### Entity 4.1: Config (/internal/domain/config.go)

**Current State**:

```go
type Config struct {
    VaultPath        string
    TemplatesDir     string
    SchemasDir       string
    PropertyBankFile string
    CacheDir         string
    LogLevel         string
    FileClassKey     string
}

// Only behavior: path helper
func (c Config) PropertyBankPath() string {
    return filepath.Join(c.SchemasDir, c.PropertyBankFile)
}
```

**Assessment**:

✅ **CORRECT AS VALUE OBJECT** - Immutable configuration

- Documented as "immutable value object"
- Constructor applies defaults
- Single helper method for derived path

⚠️ **DESIGN QUESTION** - Monolithic vs composed?

- Single flat struct with 7 fields
- Could be broken into logical groups

**Recommendation** (OPTIONAL):

```go
// Option A: Keep flat (current - simple, works)
type Config struct {
    VaultPath    string
    TemplatesDir string
    SchemasDir   string
    // ...
}

// Option B: Compose into logical groups
type Config struct {
    Vault    VaultConfig
    Schema   SchemaConfig
    Template TemplateConfig
    Logging  LoggingConfig
}

type VaultConfig struct {
    Path     string
    CacheDir string
}

type SchemaConfig struct {
    Dir              string
    PropertyBankFile string
    FileClassKey     string
}

type TemplateConfig struct {
    Dir string
}

type LoggingConfig struct {
    Level string
}
```

**Priority**: LOW - Current design is fine, composition is optional improvement

---

### System 5: File/Storage DTOs

#### 8. VaultFile (Infrastructure DTO)

**Location**: `internal/shared/dto/file.go`

**Type Classification**: Infrastructure Data Transfer Object

**Current Implementation**:

```go
// FileMetadata - embedded in VaultFile
type FileMetadata struct {
    Path     string    // Absolute path (❌ Gap 1.2)
    Basename string    // Computed
    Folder   string    // Computed
    Ext      string    // Computed
    ModTime  time.Time // ❌ Duplicates fs.FileInfo (Gap 1.1)
    Size     int64     // ❌ Duplicates fs.FileInfo (Gap 1.1)
    MimeType string    // Computed
}

// VaultFile - combines metadata + content
type VaultFile struct {
    FileMetadata         // ❌ Embeds FileMetadata
    Content      []byte  // ❌ Conflates file + content (Gap 1.3)
}

// Constructor
func NewFileMetadata(path string, info fs.FileInfo) FileMetadata
func NewVaultFile(metadata FileMetadata, content []byte) VaultFile
```

**Assessment**:

❌ **CRITICAL REDESIGN NEEDED** - Multiple architectural issues identified in Gap Analysis

**Issue 1: FileMetadata Duplicates fs.FileInfo (Gap 1.1)**

- **Problem**: FileMetadata reimplements fs.FileInfo functionality (ModTime, Size)
- **Evidence**: Go stdlib provides `fs.FileInfo` interface with ModTime(), Size(), Mode(), IsDir()
- **Research Finding**: Obsidian uses FileStats interface paralleling fs.FileInfo
- **Impact**: Unnecessary code duplication, potential inconsistency with fs.FileInfo values
- **Correct Approach**: Use fs.FileInfo directly, add only vault-specific fields

**Issue 2: Absolute Paths Hinder Portability (Gap 1.2)**

- **Problem**: FileMetadata.Path stores absolute paths (line 33-35 in dto/file.go)
- **Evidence**: Documentation says "Path is the absolute path to the file"
- **Impact**: Cache not portable between systems (MacOS → Linux path incompatibility)
- **Correct Approach**: Store vault-relative paths, compute absolute on-demand

**Issue 3: File + Content Conflation (Gap 1.3)**

- **Problem**: VaultFile always loads Content even when only metadata needed
- **Evidence**: VaultFile struct combines FileMetadata + Content []byte
- **Impact**: Memory inefficiency - metadata queries load full file content unnecessarily
- **Research Finding**: Obsidian separates TFile (metadata) from CachedMetadata (extracted content)
- **Correct Approach**: Separate file metadata scanning from content loading

**Issue 4: Platform-Specific Path Handling (Gap 7.1)**

- **Problem**: Path computations may not handle cross-platform differences
- **Evidence**: Uses filepath.Dir, filepath.Base, filepath.Ext (platform-dependent)
- **Impact**: Cache portability issues (Windows backslash vs Unix forward slash)
- **Correct Approach**: Use filepath.ToSlash for vault-relative paths

**Issue 5: DTO Not Storage-Focused (Gap 1.1, Issue D2)**

- **Problem**: Single generic DTO for all storage systems (BoltDB, SQLite, JSON export)
- **Evidence**: Same VaultFile used for scanning, hot cache, deep storage
- **Impact**: Hot cache stores unnecessary fields; deep storage lacks query optimization
- **Research Finding**: Storage-specific needs differ (BoltDB hot metadata vs SQLite queryable data)
- **Correct Approach**: BoltDBMetadata, SQLiteMetadata, FilePathInfo separation

**Recommendation**:

**Phase 1: Leverage fs.FileInfo (Resolve Gap 1.1, 7.1)**

```go
// BEFORE (current - duplicates fs.FileInfo)
type FileMetadata struct {
    Path     string
    Basename string
    Folder   string
    Ext      string
    ModTime  time.Time  // ❌ Duplicates fs.FileInfo.ModTime()
    Size     int64      // ❌ Duplicates fs.FileInfo.Size()
    MimeType string
}

// AFTER (leverage stdlib)
type VaultFileInfo struct {
    // Vault-relative path (cross-platform portable)
    VaultPath string  // "projects/lithos.md" (forward slashes via filepath.ToSlash)

    // Use fs.FileInfo directly (don't duplicate)
    Info fs.FileInfo  // Provides ModTime(), Size(), Mode(), IsDir()

    // Vault-specific computed fields only
    Basename string   // "lithos" (filename without path/extension)
    Ext      string   // ".md"
    MimeType string   // "text/markdown"
}

// Factory leverages fs.FileInfo + vault root
func NewVaultFileInfo(vaultRoot string, absPath string, info fs.FileInfo) VaultFileInfo {
    relPath, _ := filepath.Rel(vaultRoot, absPath)
    return VaultFileInfo{
        VaultPath: filepath.ToSlash(relPath),  // Cross-platform: always forward slashes
        Info:      info,
        Basename:  computeBasename(absPath),
        Ext:       filepath.Ext(absPath),
        MimeType:  computeMimeType(filepath.Ext(absPath)),
    }
}
```

**Phase 2: Separate Metadata from Content (Resolve Gap 1.3)**

```go
// Scanning DTO - metadata only
type VaultFileMeta struct {
    VaultFileInfo  // File metadata leveraging fs.FileInfo
}

// Indexing DTO - metadata + content (only when needed)
type VaultFileWithContent struct {
    VaultFileMeta
    Content []byte  // Raw content for frontmatter extraction
}

// Adapter methods
type VaultScannerPort interface {
    // Returns metadata only (fast, low memory)
    ScanAll(ctx) ([]VaultFileMeta, error)

    // Returns metadata + content (slower, high memory)
    ScanAllWithContent(ctx) ([]VaultFileWithContent, error)
}
```

**Phase 3: Storage-Specific DTOs (Resolve Gap 1.1, Issue D2, A4)**

```go
// BoltDB hot cache metadata (minimal, fast lookups)
type BoltDBMetadata struct {
    Path      string            // Vault-relative
    Basename  string            // For wikilink lookup
    Aliases   []string          // For alias lookup
    FileClass string            // For schema filtering
    ModTime   time.Time         // For staleness detection
}

// SQLite deep storage metadata (complete, queryable)
type SQLiteMetadata struct {
    Path       string                 // Vault-relative
    Frontmatter map[string]interface{} // All fields for schema-driven views
    ModTime    time.Time               // For staleness detection
    Size       int64                   // For filtering
}

// Conversion functions
func VaultFileToBoltDBMetadata(vf VaultFileMeta, fm domain.Frontmatter) BoltDBMetadata
func VaultFileToSQLiteMetadata(vf VaultFileMeta, fm domain.Frontmatter) SQLiteMetadata
```

**Benefits of Redesign**:

1. **Gap 1.1 Resolved**: Use fs.FileInfo directly, eliminate duplication
2. **Gap 1.2 Resolved**: Vault-relative paths enable cache portability
3. **Gap 1.3 Resolved**: Separate metadata from content (memory efficiency)
4. **Gap 7.1 Resolved**: filepath.ToSlash ensures cross-platform paths
5. **Issue D2 Resolved**: Storage-specific DTOs optimized for use case
6. **Obsidian Pattern Alignment**: TFile (VaultFileMeta) + FileStats (fs.FileInfo) separation

**Epic Impact**: Stories 3.2.1 (VaultFile redesign for parsed frontmatter), 3.4.1 (storage-specific DTOs), 3.13 (staleness detection)

**Priority**: CRITICAL - Blocks Group 2 storage architecture (Stories 3.4, 3.5, 3.6)

**Effort**: MEDIUM (3 points)

- Phase 1: Refactor VaultFileInfo to use fs.FileInfo (1 pt)
- Phase 2: Separate VaultFileMeta from VaultFileWithContent (1 pt)
- Phase 3: Create BoltDBMetadata, SQLiteMetadata (1 pt)

**Dependencies**:

- Group 1 (Story 3.2.1) - VaultFile needs ParsedFrontmatter field addition
- Group 2 (Story 3.4.1) - Storage-specific DTO design

---

### Summary of Entity Issues

| Entity          | Type         | Current State   | Issues                                                                                   | Priority | Effort |
| --------------- | ------------ | --------------- | ---------------------------------------------------------------------------------------- | -------- | ------ |
| Schema          | Rich Model   | ✅ Good         | ResolvedProperties leaks adapter                                                         | MEDIUM   | LOW    |
| Property        | Rich Model   | ✅ Good         | Mutable validated flag                                                                   | LOW      | LOW    |
| PropertyBank    | Registry     | ✅ Good         | None                                                                                     | N/A      | N/A    |
| **Note**        | **Anemic**   | ❌ **Issue**    | **Missing validation, behavior**                                                         | HIGH     | MEDIUM |
| **Frontmatter** | **Anemic**   | ❌ **Issue**    | **Missing type-safe access, validation**                                                 | HIGH     | MEDIUM |
| Template        | Intentional  | ✅ Good         | None (intentionally anemic)                                                              | N/A      | N/A    |
| Config          | Value Object | ✅ Good         | Optional composition improvement                                                         | LOW      | LOW    |
| **VaultFile**   | **DTO**      | ❌ **CRITICAL** | **Duplicates fs.FileInfo, absolute paths, file+content conflation, not storage-focused** | CRITICAL | MEDIUM |

**Critical Findings**:

1. **VaultFile has CRITICAL design issues** - Duplicates stdlib, hinders portability, conflates concerns (Gaps 1.1, 1.2, 1.3, 7.1, D2)
2. **Note and Frontmatter are anemic** - Need rich behavior for domain logic
3. **Schema.ResolvedProperties** leaks adapter concern - Minor architectural issue
4. All other entities are correctly designed

---

## Epic Impact Assessment

This section analyzes how each of the 8 issue groups impacts Epic 3 (Vault Indexing Engine) by mapping architectural gaps to concrete story insertions, dependencies, and sequencing. All recommendations incorporate findings from Research Strategy (Go stdlib + Obsidian patterns), Gap Analysis (12 gaps), Actionable Insights (AI-1.1 through AI-3.1), and Entity Review (8 entities).

### Epic 3 Current State (Baseline for Impact Analysis)

**Completed Stories** (✅ DONE):

- 3.1-3.9: Original PRD stories (Cache ports, JSON adapters, Vault ports, Indexer, Query, Frontmatter, CLI)
- 3.10: Fix Note ID Collision and Path Handling

**In Progress** (🔄 Ready for Done):

- 3.11-3.16: Bug fixes and refinements (memory leak, cache management, query layer, performance, integration, QA)

**Awaiting Enhancement** (📋 Course Correction):

- Current 3.17: Hybrid Architecture DI and E2E Test (Ready for Enhancement - awaiting hybrid BoltDB+SQLite)
- Current 3.18: Documentation Update (Ready for Enhancement - awaiting architecture finalization)

**Strategic Context**: November 2, 2024 Sprint Change Proposal APPROVED pivot to hybrid BoltDB + SQLite architecture for production performance. This architectural review identifies 14 NEW stories (35+ points) that must be inserted BEFORE current 3.17-3.18 can proceed.

---

### Story Insertion Strategy

Since Stories 3.1-3.16 are mostly DONE, all new work inserts **AFTER 3.16** and **BEFORE current 3.17** in dependency-ordered groups:

```
3.1-3.16 (existing - mostly done)
  ↓
GROUP A: Foundation DTOs & Ports (3.17-3.18) - Sprint 1
GROUP B: Storage Implementation (3.19-3.22) - Sprint 2-3
GROUP C: Service Refactoring (3.23-3.26) - Sprint 3-4
GROUP D: Configuration & Docs (3.27-3.29) - Sprint 4
  ↓
3.30: Hybrid DI/E2E (was 3.17 - unblocked)
3.31: Documentation (was 3.18 - enhanced)
```

---

## GROUP A: Foundation DTOs & Ports (Sprint 1)

### Story 3.17: VaultFile DTO Redesign with Layered Architecture

**Incorporates**: AI-1.1 (Eliminate FileMetadata Duplication), AI-1.2 (Vault-Relative Paths), AI-2.2 (File vs Content Separation), Entity Review VaultFile Assessment

**Priority**: CRITICAL - Foundation for all storage work
**Effort**: 3 points
**Insert Location**: After 3.16, before current 3.17

**Description**: Redesign VaultFile DTO using layered architecture: base DTO with fs.FileInfo, content separation, and storage-specific DTOs.

**Acceptance Criteria**:

**Layer 1 - Base DTO** (AI-1.1 + AI-1.2):

1. ✅ Refactor `VaultFile` struct:
   ```go
   type VaultFile struct {
       Path    string      // Vault-relative: "notes/meeting.md"
       Info    fs.FileInfo // Delegate to stdlib (ModTime, Size, Mode, IsDir)
       Content []byte      // Loaded on-demand
   }
   ```
2. ✅ Add computed methods: `Basename()`, `Folder()`, `Ext()`, `ModTime()`, `Size()`
3. ✅ Remove duplicated fields: Basename, Folder, Ext, ModTime, Size, MimeType
4. ✅ Path normalization helper: `NormalizePath(absPath, vaultRoot string) (string, error)` using filepath.ToSlash
5. ✅ Helper: `AbsolutePath(vaultRoot string) string` for I/O operations

**Layer 2 - Content Separation** (AI-2.2): 6. ✅ Create `VaultFileMeta` struct (metadata only - NO Content):

```go
type VaultFileMeta struct {
    Path string
    Info fs.FileInfo
}
```

7. ✅ Create `VaultFileWithContent` struct (metadata + content):
   ```go
   type VaultFileWithContent struct {
       VaultFileMeta
       Content []byte
   }
   ```
8. ✅ Update VaultScannerPort methods:
   - `ScanAll(ctx) ([]VaultFileMeta, error)` - fast metadata scan
   - `ScanWithContent(ctx) ([]VaultFileWithContent, error)` - when content needed

**Layer 3 - Storage-Specific DTOs** (Entity Review Phase 3): 9. ✅ Create `BoltDBMetadata` (hot cache - minimal):

```go
type BoltDBMetadata struct {
    Path      string
    Basename  string
    Aliases   []string
    FileClass string
    ModTime   time.Time
}
```

10. ✅ Create `SQLiteMetadata` (deep storage - complete):
    ```go
    type SQLiteMetadata struct {
        Path        string
        Frontmatter map[string]any
        ModTime     time.Time
        Size        int64
    }
    ```
11. ✅ Conversion functions: `ToBeoltDBMetadata()`, `ToSQLiteMetadata()`

**Cross-Platform Paths** (Gap 7.1): 12. ✅ All path storage uses forward slashes (filepath.ToSlash) 13. ✅ I/O operations convert to OS-specific paths (filepath.FromSlash)

**Testing**: 14. ✅ Unit tests: fs.FileInfo delegation, computed methods, path normalization 15. ✅ Cross-platform tests: Windows/Linux/Mac path compatibility 16. ✅ Memory tests: VaultFileMeta vs VaultFileWithContent memory usage

**Dependencies**:

- Depends on: Stories 3.1-3.16 (foundation complete)
- Blocks: All GROUP B stories (storage implementations)
- Resolves: Gap 1.1, Gap 1.2, Gap 1.3, Gap 7.1, Issue D2

**Risks**:

- HIGH: VaultFile used across many adapters - comprehensive impact analysis required
- Mitigation: Update all usages in same story; comprehensive integration tests

**Architecture Doc Updates**:

- `docs/architecture/data-models.md`: Update VaultFile, add layered DTO explanation
- `docs/architecture/components.md`: Update VaultScannerPort interface

---

### Story 3.18: MarkdownParserPort - Dedicated Parsing Port & Adapter

**Incorporates**: AI-1.3 (Extract MarkdownParserPort), Gap 3.1 (Goldmark in Domain)

**Priority**: CRITICAL - Fixes hexagonal architecture violation
**Effort**: 3 points
**Insert Location**: After 3.17

**Description**: Create dedicated MarkdownParserPort and GoldmarkParserAdapter, moving goldmark parsing from domain layer (FrontmatterService) to adapter layer. This is a separate port/adapter, NOT just moving to VaultReaderAdapter.

**Acceptance Criteria**:

**New Port Definition**:

1. ✅ Create `/internal/ports/spi/markdown.go`:
   ```go
   type MarkdownParserPort interface {
       // ParseFrontmatter extracts YAML frontmatter from markdown content
       // Returns parsed fields as map, error if invalid YAML
       ParseFrontmatter(ctx context.Context, content []byte) (map[string]any, error)
   }
   ```

**New Adapter Implementation**: 2. ✅ Create `/internal/adapters/spi/markdown/goldmark_parser.go`:

```go
type GoldmarkParserAdapter struct {
    markdown goldmark.Markdown
    log      zerolog.Logger
}

func (a *GoldmarkParserAdapter) ParseFrontmatter(ctx context.Context, content []byte) (map[string]any, error) {
    // Use goldmark + frontmatter extension
    // Syntactic validation (YAML structure)
    // Return parsed map or error
}
```

**Domain Layer Cleanup**: 3. ✅ Remove goldmark imports from `internal/app/frontmatter/service.go` 4. ✅ Remove `FrontmatterService.Extract()` method entirely 5. ✅ FrontmatterService constructor accepts `MarkdownParserPort` (injected dependency) 6. ✅ FrontmatterService focuses on **semantic validation only** (schema compliance)

**Adapter Layer - Syntactic Validation**: 7. ✅ GoldmarkParserAdapter performs **syntactic validation**:

- YAML structure validation
- Parsing errors (malformed YAML)
- Returns structured errors with line numbers

**Integration**: 8. ✅ Update VaultIndexer to inject MarkdownParserPort 9. ✅ VaultIndexer workflow: Read file → Parse frontmatter → Pass to FrontmatterService 10. ✅ VaultReaderAdapter.Read() still returns raw Content (parsing happens in indexer)

**Testing**: 11. ✅ Unit tests: FrontmatterService without goldmark (pure domain tests) 12. ✅ Unit tests: GoldmarkParserAdapter syntactic validation 13. ✅ Integration tests: End-to-end parsing workflow 14. ✅ Error handling tests: Malformed YAML, missing frontmatter

**Dependencies**:

- Depends on: Story 3.17 (VaultFile DTO ready)
- Blocks: Story 3.23 (FrontmatterService refactoring)
- Resolves: Gap 3.1, Issue B2, Hexagonal Principle violation

**Risks**:

- MEDIUM: FrontmatterService interface changes ripple to consumers
- Mitigation: Update all consumers in same story

**Future Expansion** (Post-Epic 3):

- Phase 2: Expand to parse Links, Headings, Tags (domain.NoteMetadata)
- Phase 3: Enable backlinks, graph queries

**Architecture Doc Updates**:

- `docs/architecture/components.md`: Add MarkdownParserPort, GoldmarkParserAdapter
- `docs/architecture/coding-standards.md`: Document validation layer separation

---

## GROUP B: Storage Implementation (Sprint 2-3)

### Story 3.19: Implement BoltDB Hot Cache Adapter

**Priority**: CRITICAL - Hybrid storage hot layer
**Effort**: 5 points
**Insert Location**: After 3.18

**Description**: Implement BoltDB cache adapter using BoltDBMetadata (Story 3.17) for sub-millisecond hot path queries (ByPath, ByBasename, ByAlias).

**Acceptance Criteria**:

1. ✅ `/internal/adapters/spi/cache/boltdb_writer.go` implements CacheWriterPort
2. ✅ `/internal/adapters/spi/cache/boltdb_reader.go` implements CacheReaderPort
3. ✅ Bucket structure:
   - `/notes/` - primary bucket: NoteID → BoltDBMetadata
   - `/indices/byPath/` - secondary index: Path → NoteID
   - `/indices/byBasename/` - secondary index: Basename → []NoteID
   - `/indices/byAlias/` - secondary index: Alias → []NoteID
   - `/indices/byFileClass/` - secondary index: FileClass → []NoteID
4. ✅ Store BoltDBMetadata only (not full Note) - minimal hot data
5. ✅ Atomic writes using `bolt.Tx` transactions
6. ✅ Secondary index maintenance on write (update all indices transactionally)
7. ✅ Performance target: Path/basename/alias lookups < 1ms
8. ✅ Error wrapping per FR9 requirements
9. ✅ Structured logging (zerolog) for all operations
10. ✅ Unit tests: persist/delete/read/list, index queries, error paths
11. ✅ Integration tests: concurrent reads/writes, index consistency
12. ✅ `golangci-lint run` and `go test` succeed

**Dependencies**:

- Depends on: Story 3.17 (BoltDBMetadata DTO defined)
- Blocks: Story 3.21 (write coordination needs BoltDB ready)

**Risks**:

- MEDIUM: BoltDB transaction rollback complexity
- Mitigation: Comprehensive error handling; transaction testing

---

### Story 3.20: Implement SQLite Deep Storage with Schema-Driven Views

**Incorporates**: AI-1.4 (MetadataQueryPort for O(1) indexed queries), Gap 4.1 resolution

**Priority**: CRITICAL - Hybrid storage deep layer + indexed queries
**Effort**: 5 points
**Insert Location**: After 3.19

**Description**: Implement SQLite adapter using SQLiteMetadata (Story 3.17) with schema-driven view generation for O(1) indexed frontmatter queries.

**Acceptance Criteria**:

**Basic SQLite Adapter**:

1. ✅ `/internal/adapters/spi/cache/sqlite_writer.go` implements CacheWriterPort
2. ✅ `/internal/adapters/spi/cache/sqlite_reader.go` implements CacheReaderPort
3. ✅ Table schema:
   ```sql
   CREATE TABLE notes (
       id          TEXT PRIMARY KEY,
       path        TEXT UNIQUE NOT NULL,
       frontmatter TEXT,  -- JSON
       mod_time    INTEGER,
       size        INTEGER
   );
   CREATE INDEX idx_notes_path ON notes(path);
   CREATE INDEX idx_notes_mod_time ON notes(mod_time);
   ```

**Schema-Driven View Generation**: 4. ✅ Function: `GenerateSchemaView(schema domain.Schema) (string, error)` 5. ✅ View naming: `v_{schema_name}_notes` (e.g., `v_contact_notes`, `v_project_notes`) 6. ✅ Typed column extraction from JSON:

```sql
CREATE VIEW v_contact_notes AS
SELECT
    id,
    path,
    json_extract(frontmatter, '$.name') AS name,
    json_extract(frontmatter, '$.email') AS email,
    json_extract(frontmatter, '$.phone') AS phone,
    json_extract(frontmatter, '$.status') AS status,
    mod_time
FROM notes
WHERE json_extract(frontmatter, '$.fileClass') = 'contact';
```

7. ✅ Index creation on view columns:
   ```sql
   CREATE INDEX idx_contact_status ON v_contact_notes(status);
   CREATE INDEX idx_contact_name ON v_contact_notes(name);
   ```
8. ✅ PropertySpec type → SQL type mapping (string, integer, real, text)
9. ✅ View generation during SQLite adapter initialization
10. ✅ Schema changes trigger view recreation (migration strategy documented)

**MetadataQueryPort Implementation**: 11. ✅ Create `/internal/ports/spi/metadata_query.go`:
`go
    type MetadataQueryPort interface {
        TagQuery(ctx context.Context, tag string) ([]domain.Note, error)
        FileClassQuery(ctx context.Context, fileClass string) ([]domain.Note, error)
        FrontmatterQuery(ctx context.Context, field, value string) ([]domain.Note, error)
    }
    ` 12. ✅ SQLiteReader implements MetadataQueryPort 13. ✅ FileClassQuery uses schema-specific view (not base table):
`go
    SELECT * FROM v_contact_notes WHERE status = ?
    `

**Performance**: 14. ✅ Performance test: Query `v_contact_notes WHERE status = 'active'` < 50ms for 1000 notes 15. ✅ Benchmark: O(1) indexed queries vs O(n) JSON scanning (show improvement)

**Testing**: 16. ✅ Unit tests: View generation, type mapping, index creation 17. ✅ Integration tests: Schema changes, view migration 18. ✅ Performance tests: Query speed, scaling to 1000+ notes 19. ✅ `golangci-lint run` and `go test` succeed

**Dependencies**:

- Depends on: Story 3.17 (SQLiteMetadata DTO defined)
- Blocks: Story 3.21 (write coordination needs SQLite ready)
- Resolves: Gap 4.1, Issue A5

**Risks**:

- HIGH: View generation complexity (PropertySpec → SQL type mapping)
- Mitigation: Start with simple types (string, integer); incremental expansion
- MEDIUM: Schema changes require view migration
- Mitigation: Document regeneration procedure; version views

---

### Story 3.21: Implement Storage Write Coordination (Unit of Work)

**Incorporates**: Issue A6 (Storage Write Coordination)

**Priority**: CRITICAL - Prevents data inconsistency
**Effort**: 5 points
**Insert Location**: After 3.20

**Description**: Implement Unit of Work pattern for coordinated BoltDB + SQLite dual-write operations with transactional guarantees and rollback on partial failure.

**Acceptance Criteria**:

**Unit of Work Pattern**:

1. ✅ Create `/internal/app/cache/unit_of_work.go`:

   ```go
   type CacheUnitOfWork struct {
       boltWriter   spi.CacheWriterPort
       sqliteWriter spi.CacheWriterPort
       operations   []operation
       mu           sync.Mutex
   }

   func (uow *CacheUnitOfWork) Begin() error
   func (uow *CacheUnitOfWork) AddWrite(note domain.Note) error
   func (uow *CacheUnitOfWork) AddDelete(id domain.NoteID) error
   func (uow *CacheUnitOfWork) Commit(ctx context.Context) error
   func (uow *CacheUnitOfWork) Rollback(ctx context.Context) error
   ```

**Transaction Semantics**: 2. ✅ Batch operations collected during transaction (not immediate writes) 3. ✅ Commit sequence: BoltDB first, then SQLite (hot cache priority) 4. ✅ BoltDB write failure → entire transaction rollback (no SQLite write attempted) 5. ✅ SQLite write failure → rollback BoltDB changes (compensating transaction) 6. ✅ Transaction isolation - concurrent transactions don't interfere (mutex)

**Rollback Strategy**: 7. ✅ BoltDB rollback: Use `bolt.Tx.Rollback()` 8. ✅ SQLite rollback: Use SQL `ROLLBACK` statement 9. ✅ Compensating writes: If SQLite fails, delete from BoltDB what was written

**Integration**: 10. ✅ VaultIndexer uses CacheUnitOfWork for all cache writes 11. ✅ CLI commands wrap operations in UoW transactions 12. ✅ Error handling: Proper context, structured logging

**Testing**: 13. ✅ Unit tests: Begin/Commit/Rollback lifecycle 14. ✅ Integration test: Simulate SQLite write failure, verify BoltDB rollback 15. ✅ Integration test: Simulate BoltDB write failure, verify no SQLite write 16. ✅ Concurrency test: Parallel transactions don't interfere 17. ✅ Performance test: Transactional write overhead < 10% vs direct writes

**Dependencies**:

- Depends on: Stories 3.19 (BoltDB), 3.20 (SQLite)
- Blocks: Story 3.30 (Hybrid DI/E2E needs coordinated writes)
- Resolves: Issue A6, Gap 4.2 (partial)

**Risks**:

- HIGH: Transaction rollback complexity (two storage systems)
- Mitigation: Comprehensive error handling; test all failure modes
- MEDIUM: Performance impact of transactions
- Mitigation: Benchmark; acceptable for consistency guarantees

**Architecture Doc Updates**:

- `docs/architecture/patterns.md`: Add Unit of Work pattern explanation
- `docs/architecture/components.md`: Document CacheUnitOfWork

---

### Story 3.22: QueryService Hybrid Storage Enhancement

**Incorporates**: Issue B1 (QueryService Command/Query Mixing), CQRS compliance

**Priority**: HIGH - Fixes shipped CQRS violation, enables hybrid queries
**Effort**: 3 points
**Insert Location**: After 3.21

**Description**: Refactor QueryService to support hybrid BoltDB+SQLite query routing and fix CQRS violation (RefreshFromCache is write operation in query service).

**Acceptance Criteria**:

**Hybrid Storage Support**:

1. ✅ QueryService constructor accepts both readers:
   ```go
   func NewQueryService(
       boltReader   spi.CacheReaderPort,
       sqliteReader spi.CacheReaderPort,  // Also implements MetadataQueryPort
       config       domain.Config,
       log          zerolog.Logger,
   ) *QueryService
   ```

**Query Routing Strategy**: 2. ✅ Hot path (BoltDB): `ByPath()`, `ByBasename()`, `ByAlias()` → sub-millisecond 3. ✅ Deep path (SQLite): `ByFrontmatterField()`, complex queries → use MetadataQueryPort 4. ✅ Merger logic: Combine results from both stores with consistency validation 5. ✅ Consistency check: Verify BoltDB and SQLite ModTime match before returning

**CQRS Compliance Fix**: 6. ✅ Remove `RefreshFromCache()` from QueryService (write operation in query service) 7. ✅ Move index rebuilding to new `IndexMaintenanceService` (command side) 8. ✅ QueryService is now **read-only** (true CQRS query side)

**Incremental Refresh Fix**: 9. ✅ Fix "ModTime filtering broken" (lines 464-467 in service.go) 10. ✅ Implement staleness detection using SQLite: `SELECT * FROM notes WHERE mod_time > ?` 11. ✅ IndexMaintenanceService.RefreshIncremental(since time.Time) loads only modified notes

**Testing**: 12. ✅ Unit tests: Query routing (hot vs deep), merger logic 13. ✅ Integration test: Index 100 notes, modify 5, RefreshIncremental loads only 5 14. ✅ Performance test: Hot path < 1ms, deep path < 50ms 15. ✅ Consistency test: BoltDB/SQLite mismatch detection

**Dependencies**:

- Depends on: Story 3.21 (hybrid storage operational)
- Blocks: Story 3.30 (Hybrid DI/E2E needs correct query routing)
- Resolves: Issue B1, Gap 2.1 (partial)

**Risks**:

- MEDIUM: Query routing logic complexity
- Mitigation: Clear routing rules; comprehensive testing
- LOW: Consistency validation overhead
- Mitigation: Only validate on mismatch suspicion (heuristic)

**Architecture Doc Updates**:

- `docs/architecture/components.md`: Update QueryService, add IndexMaintenanceService
- `docs/architecture/patterns.md`: Document CQRS read/write separation

---

## GROUP C: Service Refactoring (Sprint 3-4)

### Story 3.23: FrontmatterService Refactoring - Use MarkdownParserPort

**Priority**: HIGH - Completes domain purity refactoring
**Effort**: 2 points
**Insert Location**: After 3.22

**Description**: Refactor FrontmatterService to use MarkdownParserPort (Story 3.18) instead of direct goldmark parsing, focusing on semantic validation only.

**Acceptance Criteria**:

1. ✅ FrontmatterService constructor accepts MarkdownParserPort (injected)
2. ✅ Remove all goldmark-related code from FrontmatterService
3. ✅ FrontmatterService.Validate() renamed to `IsSchemaCompliant()` (semantic validation)
4. ✅ Semantic validation focus: schema compliance, required fields, type checking
5. ✅ Uses Frontmatter entity methods (from Story 3.24) for field access
6. ✅ Unit tests run without goldmark (pure domain tests)
7. ✅ Integration tests: End-to-end with MarkdownParserPort

**Dependencies**:

- Depends on: Story 3.18 (MarkdownParserPort exists), Story 3.24 (Frontmatter enriched)
- Resolves: Hexagonal architecture compliance

---

### Story 3.24: Enrich Frontmatter Entity with Validation & Factory

**Priority**: HIGH - Enables proper domain modeling
**Effort**: 2 points
**Insert Location**: After 3.23

**Description**: Transform Frontmatter from anemic data bag into rich domain entity with validation, factory methods, and type-safe field access.

**Acceptance Criteria**:

1. ✅ `Frontmatter.Validate() error` - semantic validation
2. ✅ `NewFrontmatter(fields map[string]any) (Frontmatter, error)` - factory with validation
3. ✅ Type-safe accessors: `GetString()`, `GetStringSlice()`, `GetInt()`, `GetBool()`
4. ✅ `HasField(key string) bool` - field existence check
5. ✅ All Frontmatter creation through factory (enforced validation)
6. ✅ FrontmatterService uses entity methods (not direct field access)
7. ✅ Unit tests: validation edge cases, type-safe accessors

**Dependencies**:

- Depends on: Story 3.23 (FrontmatterService ready)
- Blocks: Story 3.25 (Note entity needs Frontmatter methods)
- Resolves: Issue D1 (partial - Frontmatter anemic model)

---

### Story 3.25: Enrich Note Entity with Behavior Methods

**Priority**: MEDIUM - Completes domain model enrichment
**Effort**: 2 points
**Insert Location**: After 3.24

**Description**: Transform Note from anemic data bag into rich domain entity with behavior methods.

**Acceptance Criteria**:

1. ✅ `Note.Validate() error` - semantic validation
2. ✅ Convenience delegation methods: `HasFrontmatterField()`, `GetFrontmatterString()`
3. ✅ `NewNote(id NoteID, frontmatter Frontmatter) (Note, error)` - factory with validation
4. ✅ Services use Note methods instead of direct field access
5. ✅ Unit tests: validation, delegation

**Dependencies**:

- Depends on: Story 3.24 (Frontmatter enriched)
- Resolves: Issue D1 (complete - Note anemic model)

---

### Story 3.26: Document Validation Layer Separation

**Priority**: HIGH - Clarifies hexagonal architecture pattern
**Effort**: 1 point
**Insert Location**: After 3.25

**Description**: Document validation layer separation with clear naming conventions.

**Acceptance Criteria**:

1. ✅ Architecture doc: validation layer table (syntactic vs semantic)
2. ✅ Naming convention:
   - `ValidateSyntax()` / `IsValidSyntax()` - adapter layer
   - `Validate()` / `IsSchemaCompliant()` - domain layer
3. ✅ All validation call sites updated to use new naming
4. ✅ Code comments clarify validation types

**Dependencies**:

- Depends on: Stories 3.23-3.25 (validation refactoring complete)
- Resolves: Hexagonal Principle violation (complete)

---

## GROUP D: Configuration & Event Infrastructure (Sprint 4)

### Story 3.27: Implement Singleton Pattern for Config & PropertyBank

**Incorporates**: Issue A2 (Singleton Pattern Implementation)

**Priority**: HIGH - Required for DI
**Effort**: 2 points
**Insert Location**: After 3.26

**Description**: Implement singleton pattern using sync.Once for Config and PropertyBank.

**Acceptance Criteria**:

1. ✅ `config.Instance() *Config` - singleton accessor using sync.Once
2. ✅ `propertybank.Instance() *PropertyBank` - singleton accessor using sync.Once
3. ✅ Thread-safe initialization (sync.Once guarantees single initialization)
4. ✅ Test harness support: `SetInstanceForTesting()` for test isolation
5. ✅ Update all Config/PropertyBank usage to use Instance()
6. ✅ Unit tests: singleton behavior, concurrency safety
7. ✅ Integration tests: DI container uses singletons

**Dependencies**:

- Depends on: Stories 3.1-3.26 (foundation complete)
- Blocks: Story 3.30 (DI needs singletons)
- Resolves: Issue A2

**Architecture Doc Updates**:

- `docs/architecture/patterns.md`: Add Singleton pattern explanation

---

### Story 3.28: Add FileClassKey Configuration Support

**Incorporates**: Issue A3 (FileClassKey Configuration Impact)

**Priority**: MEDIUM - Enables schema flexibility
**Effort**: 1 point
**Insert Location**: After 3.27

**Description**: Add Config.FileClassKey field with default "fileClass" to support custom schema selection keys.

**Acceptance Criteria**:

1. ✅ `Config.FileClassKey string` field added (default: "fileClass")
2. ✅ Schema resolution uses Config.FileClassKey instead of hardcoded "fileClass"
3. ✅ Config loading validates FileClassKey is non-empty
4. ✅ Architecture doc: fileClass key customization guidance
5. ✅ Unit tests: custom key resolution
6. ✅ Integration test: vault using custom schema key (e.g., "type")

**Dependencies**:

- Depends on: Story 3.27 (singleton Config)
- Resolves: Issue A3

---

### Story 3.29: Implement Event-Driven Architecture Infrastructure

**Incorporates**: Issue A1 (Component Orchestration Architecture) - DECIDED for Epic 3

**Priority**: CRITICAL - Solves god-object problem and enables CQRS separation
**Effort**: 5 points
**Insert Location**: After 3.28

**Description**: Implement event-driven architecture with domain events, event bus, and publisher/subscriber infrastructure to eliminate god-objects and enable clean CQRS command/query separation.

**Acceptance Criteria**:

**Event Infrastructure**:

1. ✅ Create `/internal/domain/events.go` with DomainEvent interface:
   ```go
   type DomainEvent interface {
       EventType() string
       OccurredAt() time.Time
       AggregateID() string
   }
   ```

**Domain Event Types**: 2. ✅ Implement NoteIndexed event (published after single note indexed) 3. ✅ Implement VaultIndexingComplete event (published after full vault index) 4. ✅ Implement FrontmatterValidated event (published after validation) 5. ✅ Implement SchemaLoaded event (published after schema load) 6. ✅ Implement SchemasReloaded event (published after schema reload)

**Event Bus**: 6. ✅ Create `/internal/app/events/bus.go` with EventBus implementation:

```go
type EventBus interface {
    Publish(ctx context.Context, event DomainEvent) error
    Subscribe(eventType string, handler EventHandler) error
    Unsubscribe(eventType string, handler EventHandler) error
}

type EventHandler func(ctx context.Context, event DomainEvent) error
```

7. ✅ In-memory EventBus with goroutine-based async dispatch
8. ✅ Error handling: Failed handlers don't block other subscribers
9. ✅ Structured logging for all event publishes and handler executions

**Service Refactoring - Publishers**: 10. ✅ VaultIndexer publishes NoteIndexed after each note 11. ✅ VaultIndexer publishes VaultIndexingComplete after full scan 12. ✅ FrontmatterService publishes FrontmatterValidated after validation 13. ✅ SchemaEngine publishes SchemaLoaded/SchemasReloaded

**Service Refactoring - Subscribers**: 14. ✅ IndexMaintenanceService subscribes to NoteIndexed → updates indices 15. ✅ QueryService subscribes to VaultIndexingComplete → rebuilds in-memory structures 16. ✅ MetricsService subscribes to FrontmatterValidated → tracks validation stats

**CQRS Separation**: 17. ✅ Remove RefreshFromCache() from QueryService (write operation) 18. ✅ QueryService is now pure read-side (subscribes to events only) 19. ✅ IndexMaintenanceService is command-side (publishes events)

**God-Object Elimination**: 20. ✅ CLICommander no longer directly calls services - publishes commands as events 21. ✅ VaultIndexer dependency count reduced (no direct service calls, publishes events) 22. ✅ Services communicate via events, not direct coupling

**Testing**: 23. ✅ Unit tests: Event bus publish/subscribe, handler registration 24. ✅ Integration test: End-to-end event flow (index → event → query rebuild) 25. ✅ Concurrency test: Multiple subscribers process events in parallel 26. ✅ Error handling test: Failed handler doesn't block others 27. ✅ Performance test: Event overhead < 5ms per event

**Dependencies**:

- Depends on: Story 3.21 (UoW pattern), Stories 3.23-3.26 (domain enrichment)
- Blocks: Story 3.30 (DI wiring uses event bus)
- Resolves: Issue A1, Issue B1 (QueryService CQRS violation)

**Risks**:

- HIGH: Async event processing adds debugging complexity
- Mitigation: Comprehensive event logging with trace IDs
- MEDIUM: Event ordering guarantees
- Mitigation: Document event ordering semantics; use synchronous dispatch for critical events

**Architecture Doc Updates**:

- `docs/architecture/high-level-architecture.md`: Event-Driven Architecture pattern
- `docs/architecture/components.md`: EventBus, event handlers
- `docs/architecture/data-models.md`: Activate domain events for Epic 3

---

## COMPLETION LAYER: Final Integration

### Story 3.30: Hybrid Architecture DI and Production-Scale E2E Testing

**Original Story**: 3.17 (Ready for Enhancement - Course Correction Applied)
**New Number**: 3.30 (after 14 story insertions)
**Status**: ✅ UNBLOCKED - All foundation work complete
**Enhanced Effort**: 7 points (was 5 points)

**Description**: Wire all hybrid architecture components through dependency injection and validate production-ready performance with 500+ note test vault.

**Additional Acceptance Criteria** (beyond original story):

1. ✅ DI container includes MarkdownParserPort → GoldmarkParserAdapter
2. ✅ DI container includes singleton Config, PropertyBank (Story 3.27)
3. ✅ DI container includes CacheUnitOfWork for coordinated writes (Story 3.21)
4. ✅ DI container includes BoltDB + SQLite dual cache readers/writers
5. ✅ QueryService wired with hybrid BoltDB+SQLite readers (Story 3.22)
6. ✅ IndexMaintenanceService wired (CQRS command side)
7. ✅ E2E tests use layered VaultFile DTOs (Story 3.17)
8. ✅ E2E tests validate schema-driven view queries (Story 3.20)
9. ✅ Performance test: Template query < 100ms at 500+ note scale
10. ✅ Performance test: BoltDB hot path < 1ms, SQLite deep path < 50ms

**Dependencies**:

- Depends on: ALL stories 3.17-3.29 (complete foundation)
- Unblocks: Story 3.31 (documentation can finalize)

---

### Story 3.31: Documentation Update with Architecture Patterns

**Original Story**: 3.18 (Ready for Enhancement - Course Correction Applied)
**New Number**: 3.31 (after 14 story insertions)
**Enhanced Effort**: 3 points (was 1 point)

**Description**: Update all architecture documentation including new patterns catalog.

**Additional Acceptance Criteria**:

1. ✅ Create `docs/architecture/patterns.md` with pattern catalog:
   - Singleton Pattern (Config, PropertyBank)
   - Factory Pattern (NewNote, NewFrontmatter)
   - Repository Pattern (VaultReaderPort, CacheReaderPort)
   - Unit of Work Pattern (CacheUnitOfWork)
   - CQRS Pattern (read/write separation)
   - DTO Pattern (layered architecture)
   - Hexagonal Architecture (Ports & Adapters)
2. ✅ Each pattern: intent, when to use, implementation example, trade-offs
3. ✅ Update `docs/architecture/components.md`: all new components
4. ✅ Update `docs/architecture/data-models.md`: layered DTO architecture
5. ✅ Cross-references between docs

**Dependencies**:

- Depends on: Story 3.30 (all implementation complete)

---

## Technical Debt Items (Deferred Post-Epic 3)

### Template System Refactoring (AI-2.3)

**Defer to**: Epic 5 (Interactive Input Engine)
**Effort**: 2 points
**Description**: Refactor template caching to use text/template composition instead of custom caching.

**Rationale**: Template system is functional; this is optimization, not blocker.

---

### MetadataCacheService (AI-2.1)

**Defer to**: Post-Epic 3 optimization
**Effort**: 3 points
**Description**: Add in-memory parsed metadata cache with checksum validation for performance.

**Rationale**: QueryService works without it; nice-to-have optimization.

---

### Schema System Minor Issues

**Defer to**: Post-Epic 3 cleanup
**Effort**: 2 points
**Issues**: Property.validated flag mutation, Schema.ResolvedProperties leak

**Rationale**: Minor code quality issues, not blockers.

---

## Epic Impact Assessment Summary

### Total New Stories: 14

| Group          | Stories                   | Effort     | Priority | Sprint          |
| -------------- | ------------------------- | ---------- | -------- | --------------- |
| **GROUP A**    | Foundation (3.17-3.18)    | 6 pts      | CRITICAL | 1               |
| **GROUP B**    | Storage (3.19-3.22)       | 18 pts     | CRITICAL | 2-3             |
| **GROUP C**    | Services (3.23-3.26)      | 7 pts      | HIGH     | 3-4             |
| **GROUP D**    | Config/Events (3.27-3.29) | 8 pts      | CRITICAL | 4               |
| **COMPLETION** | Integration (3.30-3.31)   | 10 pts     | CRITICAL | 5               |
| **TOTAL**      | **14 stories**            | **49 pts** | -        | **4-5 sprints** |

### Final Epic 3 Structure

**Original Epic 3**: 18 stories (3.1-3.18)
**After Course Correction**: 31 stories (3.1-3.31)

```
3.1-3.16   Existing stories (mostly complete)
3.17-3.18  GROUP A: Foundation DTOs & Ports
3.19-3.22  GROUP B: Storage Implementation
3.23-3.26  GROUP C: Service Refactoring
3.27-3.29  GROUP D: Configuration & Documentation
3.30       Hybrid DI/E2E (was 3.17 - unblocked)
3.31       Documentation (was 3.18 - enhanced)
```

### Critical Path Dependencies

```
[FOUNDATION LAYER - Sprint 1]
3.17 VaultFile DTO Redesign
  ↓
3.18 MarkdownParserPort & Adapter
  ↓
[STORAGE LAYER - Sprint 2-3]
3.19 BoltDB Hot Cache
  ↓
3.20 SQLite Deep Storage + Views
  ↓
3.21 Write Coordination (UoW)
  ↓
3.22 QueryService Hybrid Enhancement
  ↓
[SERVICE LAYER - Sprint 3-4]
3.23 FrontmatterService Refactoring
  ↓
3.24 Enrich Frontmatter Entity
  ↓
3.25 Enrich Note Entity
  ↓
3.26 Document Validation Layers
  ↓
[CONFIGURATION LAYER - Sprint 4]
3.27 Singleton Pattern
  ↓
3.28 FileClassKey Configuration
  ↓
3.29 Orchestration Pattern Decision
  ↓
[COMPLETION LAYER - Sprint 5]
3.30 Hybrid DI/E2E Testing
  ↓
3.31 Documentation Finalization
```

### Gaps Resolved

| Gap ID | Description               | Resolved By                            |
| ------ | ------------------------- | -------------------------------------- |
| 1.1    | FileMetadata Duplication  | Story 3.17                             |
| 1.2    | Absolute Paths            | Story 3.17                             |
| 1.3    | File + Content Conflation | Story 3.17                             |
| 2.1    | No Metadata Cache         | Story 3.22 (partial), Tech Debt (full) |
| 3.1    | Goldmark in Domain        | Story 3.18                             |
| 4.1    | No Indexed Query Layer    | Story 3.20                             |
| 4.2    | No Hot/Deep Separation    | Stories 3.19-3.21                      |
| 5.1    | Custom Template Cache     | Tech Debt (Epic 5)                     |
| 7.1    | Platform-Specific Paths   | Story 3.17                             |

### Issues Resolved

| Issue | Description                       | Resolved By       |
| ----- | --------------------------------- | ----------------- |
| D1    | Anemic Domain Model               | Stories 3.24-3.25 |
| D2    | DTO Architecture Mismatch         | Story 3.17        |
| B2    | IO in Domain Layer                | Story 3.18        |
| B1    | QueryService Command/Query Mixing | Story 3.22        |
| A1    | Component Orchestration           | Story 3.29        |
| A2    | Singleton Pattern                 | Story 3.27        |
| A3    | FileClassKey Config               | Story 3.28        |
| A4    | DTO Architecture                  | Story 3.17        |
| A5    | SQLite Schema Optimization        | Story 3.20        |
| A6    | Storage Write Coordination        | Story 3.21        |

### Performance Targets

- BoltDB hot path queries: < 1ms
- SQLite deep path queries: < 50ms
- Template rendering at 500+ notes: < 100ms
- Incremental indexing: Only modified notes loaded

### Timeline Estimate

**Total Duration**: 4-5 sprints (assuming 10-13 points per sprint)

- Sprint 1: Foundation (6 pts)
- Sprint 2: Storage Part 1 (10 pts)
- Sprint 3: Storage Part 2 + Services Start (8 pts)
- Sprint 4: Services Complete + Config (11 pts)
- Sprint 5: Integration & Documentation (10 pts)

**Epic 3 Completion**: ~10-12 weeks from start of course correction

---

_Epic Impact Assessment Complete. Ready for Synthesis Phase: Cross-group dependency map and comprehensive story plan finalization._

# Synthesis Phase - Comprehensive Story Plan

## Cross-Issue Dependency Map

This section visualizes how the 18+ identified issues relate to each other and to the 14 new Epic 3 stories.

### Issue Dependency Graph

```
[FOUNDATION ISSUES - Must resolve first]
D2 (DTO Architecture) ────────────────┐
                                      ├──> Story 3.17 (VaultFile DTO Redesign)
Gap 1.1 (FileMetadata Duplication) ───┤
Gap 1.2 (Absolute Paths) ─────────────┤
Gap 1.3 (File+Content Conflation) ────┤
Gap 7.1 (Platform Paths) ─────────────┘
                    │
                    ├──> Story 3.18 (MarkdownParserPort)
                    │         │
Gap 3.1 (Goldmark in Domain) ─┘       │
B2 (IO in Domain) ────────────────────┘
                    │
                    ├──> Story 3.19 (BoltDB) ─────┐
                    │                             │
                    ├──> Story 3.20 (SQLite) ─────┤
                    │         │                   │
Gap 4.1 (No Indexed Queries) ─┘                  ├──> Story 3.21 (UoW)
Gap 4.2 (No Hot/Deep Separation) ────────────────┘         │
A6 (Write Coordination) ─────────────────────────────────┘
                    │
                    ├──> Story 3.22 (QueryService Hybrid)
                    │         │
B1 (CQRS Violation) ──────────┘
Gap 2.1 (No Metadata Cache) ──┘
                    │
                    ├──> Story 3.23 (FrontmatterService Refactor)
                    │         │
                    ├──> Story 3.24 (Enrich Frontmatter) ─────┐
                    │         │                               │
D1 (Anemic Domain) ───────────┘                              ├──> Story 3.25 (Enrich Note)
                    │                                         │
                    │                                         │
                    ├──> Story 3.26 (Document Validation) ────┘
                    │
                    ├──> Story 3.27 (Singleton Pattern)
                    │         │
A2 (Singleton) ───────────────┘
                    │
                    ├──> Story 3.28 (FileClassKey Config)
                    │         │
A3 (FileClassKey) ────────────┘
                    │
                    ├──> Story 3.29 (Orchestration Pattern)
                    │         │
A1 (Orchestration) ───────────┘
                    │
                    ├──> Story 3.30 (Hybrid DI/E2E) ─── UNBLOCKED!
                    │
                    └──> Story 3.31 (Documentation)

[DEFERRED ISSUES - Tech Debt]
Gap 5.1 (Template Cache) ──> Tech Debt: Epic 5
C1 (Property Mutation) ────> Tech Debt: Post-Epic 3
C2 (Schema Leak) ──────────> Tech Debt: Post-Epic 3
D3 (Template Anemic) ──────> Tech Debt: Epic 5
```

### Issue Resolution Matrix

| Issue Code  | Issue Description           | Resolved By           | Priority | Impact                     |
| ----------- | --------------------------- | --------------------- | -------- | -------------------------- |
| **D2**      | DTO Architecture Mismatch   | Story 3.17            | CRITICAL | Foundation for all storage |
| **Gap 1.1** | FileMetadata Duplication    | Story 3.17            | CRITICAL | Memory efficiency          |
| **Gap 1.2** | Absolute Paths              | Story 3.17            | CRITICAL | Multi-vault support        |
| **Gap 1.3** | File+Content Conflation     | Story 3.17            | CRITICAL | Performance                |
| **Gap 7.1** | Platform-Specific Paths     | Story 3.17            | HIGH     | Cross-platform             |
| **Gap 3.1** | Goldmark in Domain          | Story 3.18            | CRITICAL | Hexagonal compliance       |
| **B2**      | IO in Domain Layer          | Story 3.18            | CRITICAL | Architecture purity        |
| **A4**      | DTO Architecture (dup)      | Story 3.17            | CRITICAL | Same as D2                 |
| **Gap 4.1** | No Indexed Query Layer      | Story 3.20            | CRITICAL | Query performance          |
| **Gap 4.2** | No Hot/Deep Separation      | Stories 3.19-3.21     | CRITICAL | Hybrid storage             |
| **A6**      | Storage Write Coordination  | Story 3.21            | CRITICAL | Data consistency           |
| **A5**      | SQLite Schema Optimization  | Story 3.20            | HIGH     | Query performance          |
| **B1**      | QueryService CQRS Violation | Story 3.22            | HIGH     | CQRS compliance            |
| **Gap 2.1** | No Metadata Cache           | Story 3.22, Tech Debt | MEDIUM   | Performance                |
| **D1**      | Anemic Domain Model         | Stories 3.24-3.25     | HIGH     | Domain modeling            |
| **A2**      | Singleton Pattern           | Story 3.27            | HIGH     | DI requirements            |
| **A3**      | FileClassKey Config         | Story 3.28            | MEDIUM   | Schema flexibility         |
| **A1**      | Component Orchestration     | Story 3.29            | MEDIUM   | Architecture clarity       |
| **B3**      | Missing Patterns Docs       | Story 3.31            | HIGH     | Documentation              |
| **Gap 5.1** | Custom Template Cache       | Tech Debt → Epic 5    | LOW      | Optimization               |
| **C1**      | Property Mutation           | Tech Debt             | LOW      | Code quality               |
| **C2**      | Schema Leak                 | Tech Debt             | LOW      | Code quality               |
| **D3**      | Template Anemic             | Tech Debt → Epic 5    | LOW      | Consistency                |

---

## Consolidated Epic Impact Findings

### Critical Path Analysis

**The critical path for Epic 3 completion is:**

```
Foundation Layer (Sprint 1) → 6 points
  VaultFile DTO (3.17) + MarkdownParserPort (3.18)
    ↓
Storage Layer (Sprint 2-3) → 18 points
  BoltDB (3.19) + SQLite (3.20) → UoW (3.21) → QueryService (3.22)
    ↓
Service Layer (Sprint 3-4) → 7 points
  FrontmatterService (3.23) → Frontmatter Entity (3.24) → Note Entity (3.25) → Validation Docs (3.26)
    ↓
Configuration Layer (Sprint 4) → 4 points
  Singleton (3.27) → FileClassKey (3.28) → Orchestration (3.29)
    ↓
Completion Layer (Sprint 5) → 10 points
  Hybrid DI/E2E (3.30) → Documentation (3.31)

Total: 45 points across 14 stories over 4-5 sprints
```

### Story Complexity Distribution

| Complexity | Story Count | Total Points | Stories                                                      |
| ---------- | ----------- | ------------ | ------------------------------------------------------------ |
| **HIGH**   | 5           | 23 pts       | 3.19, 3.20, 3.21 (Storage), 3.30 (DI/E2E), + Story 3.17 risk |
| **MEDIUM** | 5           | 15 pts       | 3.17, 3.18, 3.22, 3.23, 3.24                                 |
| **LOW**    | 4           | 7 pts        | 3.25, 3.26, 3.27, 3.28, 3.29, 3.31                           |

**Risk Concentration**: Sprint 2-3 (Storage Layer) has highest complexity with 3 HIGH-complexity stories requiring careful execution.

### Architecture Documentation Impact

**New Documents Required**:

1. `docs/architecture/patterns.md` - Pattern catalog (Story 3.31)
   - Singleton, Factory, Repository, Unit of Work, CQRS, DTO, Hexagonal patterns
   - Each with intent, usage, examples, trade-offs

**Document Updates Required**:

1. `docs/architecture/data-models.md` - VaultFile layered DTO architecture (Story 3.17)
2. `docs/architecture/components.md` - All new ports/adapters (Stories 3.18, 3.19, 3.20, 3.22)
3. `docs/architecture/coding-standards.md` - Validation layer separation (Story 3.26)

---

## Final Epic 3 Story Plan

### Complete Story Renumbering Table

| Old #   | New #    | Story Title                                                  | Status      | Effort            | Dependencies | Sprint         |
| ------- | -------- | ------------------------------------------------------------ | ----------- | ----------------- | ------------ | -------------- |
| 3.1     | 3.1      | Implement CacheReaderPort & CacheWriterPort                  | ✅ Done     | -                 | -            | -              |
| 3.2     | 3.2      | Multi-Storage Cache Adapters (JSON)                          | ✅ Done     | -                 | -            | -              |
| 3.3     | 3.3      | Implement VaultReaderPort, VaultWriterPort, VaultScannerPort | ✅ Done     | -                 | -            | -              |
| 3.4     | 3.4      | Implement FilesystemVaultAdapter                             | ✅ Done     | -                 | -            | -              |
| 3.5     | 3.5      | VaultIndexer Service Implementation                          | ✅ Done     | -                 | -            | -              |
| 3.6     | 3.6      | QueryService Implementation                                  | ✅ Done     | -                 | -            | -              |
| 3.7     | 3.7      | FrontmatterService Implementation                            | ✅ Done     | -                 | -            | -              |
| 3.8     | 3.8      | Frontmatter Service Integration                              | ✅ Done     | -                 | -            | -              |
| 3.9     | 3.9      | CLI Index Command Implementation                             | ✅ Done     | -                 | -            | -              |
| 3.10    | 3.10     | Fix Note ID Collision and Path Handling                      | ✅ Done     | -                 | 3.9          | -              |
| 3.11    | 3.11     | Fix Memory Leak in VaultIndexer                              | 🔄 Ready    | -                 | 3.10         | -              |
| 3.12    | 3.12     | Implement Cache Management Strategy                          | 🔄 Ready    | -                 | 3.11         | -              |
| 3.13    | 3.13     | Enhance Query Layer Performance                              | 🔄 Ready    | -                 | 3.12         | -              |
| 3.14    | 3.14     | Optimize Indexing Performance                                | 🔄 Ready    | -                 | 3.13         | -              |
| 3.15    | 3.15     | Integration Testing Improvements                             | 🔄 Ready    | -                 | 3.14         | -              |
| 3.16    | 3.16     | Epic 3 Completion QA                                         | 🔄 Ready    | -                 | 3.15         | -              |
| **NEW** | **3.17** | **VaultFile DTO Redesign (Layered Architecture)**            | 📋 New      | **3 pts**         | **3.16**     | **Sprint 1**   |
| **NEW** | **3.18** | **MarkdownParserPort (Dedicated Port/Adapter)**              | 📋 New      | **3 pts**         | **3.17**     | **Sprint 1**   |
| **NEW** | **3.19** | **Implement BoltDB Hot Cache Adapter**                       | 📋 New      | **5 pts**         | **3.18**     | **Sprint 2**   |
| **NEW** | **3.20** | **Implement SQLite Deep Storage with Schema Views**          | 📋 New      | **5 pts**         | **3.19**     | **Sprint 2-3** |
| **NEW** | **3.21** | **Implement Storage Write Coordination (UoW)**               | 📋 New      | **5 pts**         | **3.20**     | **Sprint 3**   |
| **NEW** | **3.22** | **QueryService Hybrid Storage Enhancement**                  | 📋 New      | **3 pts**         | **3.21**     | **Sprint 3**   |
| **NEW** | **3.23** | **FrontmatterService Refactoring (Use Parser Port)**         | 📋 New      | **2 pts**         | **3.22**     | **Sprint 3-4** |
| **NEW** | **3.24** | **Enrich Frontmatter Entity with Validation & Factory**      | 📋 New      | **2 pts**         | **3.23**     | **Sprint 4**   |
| **NEW** | **3.25** | **Enrich Note Entity with Behavior Methods**                 | 📋 New      | **2 pts**         | **3.24**     | **Sprint 4**   |
| **NEW** | **3.26** | **Document Validation Layer Separation**                     | 📋 New      | **1 pt**          | **3.25**     | **Sprint 4**   |
| **NEW** | **3.27** | **Implement Singleton Pattern (Config/PropertyBank)**        | 📋 New      | **2 pts**         | **3.26**     | **Sprint 4**   |
| **NEW** | **3.28** | **Add FileClassKey Configuration Support**                   | 📋 New      | **1 pt**          | **3.27**     | **Sprint 4**   |
| **NEW** | **3.29** | **Implement Event-Driven Architecture Infrastructure**       | 📋 New      | **5 pts**         | **3.28**     | **Sprint 4**   |
| 3.17    | **3.30** | **Hybrid Architecture DI and E2E Testing**                   | 📋 Enhanced | **7 pts** (was 5) | **3.29**     | **Sprint 5**   |
| 3.18    | **3.31** | **Documentation Update (Architecture Patterns)**             | 📋 Enhanced | **3 pts** (was 1) | **3.30**     | **Sprint 5**   |

**Summary**:

- **Original Epic 3**: 18 stories (3.1-3.18)
- **Course-Corrected Epic 3**: 31 stories (3.1-3.31)
- **New Stories Inserted**: 14 stories (3.17-3.30)
- **Stories Renumbered**: 3.17 → 3.30, 3.18 → 3.31
- **Total Additional Effort**: 49 points (14 new stories)

### Sprint Breakdown with Velocity Planning

Assuming team velocity of **10-13 points per sprint**:

**Sprint 1: Foundation Layer** (6 points)

- 3.17: VaultFile DTO Redesign (3 pts)
- 3.18: MarkdownParserPort (3 pts)
- **Goal**: Establish DTO and parsing architecture
- **Risk**: HIGH - VaultFile used across many adapters

**Sprint 2: Storage Layer Part 1** (10 points)

- 3.19: BoltDB Hot Cache Adapter (5 pts)
- 3.20: SQLite Deep Storage (5 pts - start)
- **Goal**: Implement dual storage adapters
- **Risk**: HIGH - BoltDB transaction complexity

**Sprint 3: Storage Layer Part 2** (8 points)

- 3.20: SQLite Deep Storage (complete if not done)
- 3.21: Storage Write Coordination (5 pts)
- 3.22: QueryService Hybrid Enhancement (3 pts)
- **Goal**: Complete storage layer with coordination
- **Risk**: HIGH - Dual-write rollback complexity

**Sprint 4: Service, Configuration & Events** (15 points)

- 3.23: FrontmatterService Refactoring (2 pts)
- 3.24: Enrich Frontmatter Entity (2 pts)
- 3.25: Enrich Note Entity (2 pts)
- 3.26: Document Validation Layers (1 pt)
- 3.27: Singleton Pattern (2 pts)
- 3.28: FileClassKey Configuration (1 pt)
- 3.29: Event-Driven Architecture Infrastructure (5 pts)
- **Goal**: Complete domain enrichment, configuration, and event infrastructure
- **Risk**: HIGH - Event-driven architecture adds complexity, async debugging

**Sprint 5: Integration & Documentation** (10 points)

- 3.30: Hybrid DI/E2E Testing (7 pts)
- 3.31: Documentation Update (3 pts)
- **Goal**: Wire everything together, validate at scale
- **Risk**: HIGH - Integration complexity, E2E failures

**Total Timeline**: **5 sprints** (~10-12 weeks, accounting for 2-week sprints)

---

## Timeline and Milestone Updates

### Epic 3 Revised Timeline

**Original Epic 3 Timeline** (from PRD):

- **Start**: October 2025
- **Original End**: December 2025 (8 weeks, 18 stories)
- **Status**: Stories 3.1-3.16 mostly complete, 3.17-3.18 blocked

**Course-Corrected Epic 3 Timeline**:

- **Course Correction Start**: November 8, 2025
- **Foundation Layer Complete**: End of Sprint 1 (2 weeks) → ~November 22, 2025
- **Storage Layer Complete**: End of Sprint 3 (6 weeks) → ~December 20, 2025
- **Service/Config Complete**: End of Sprint 4 (8 weeks) → ~January 3, 2026
- **Epic 3 Complete**: End of Sprint 5 (10 weeks) → **~January 17, 2026**

**Timeline Impact**: +10 weeks to original Epic 3 timeline due to architectural foundation work.

### Milestones

| Milestone                          | Date          | Stories   | Deliverables                             |
| ---------------------------------- | ------------- | --------- | ---------------------------------------- |
| **M1: Foundation Complete**        | ~Nov 22, 2025 | 3.17-3.18 | VaultFile DTO, MarkdownParserPort        |
| **M2: Storage Layer Complete**     | ~Dec 20, 2025 | 3.19-3.22 | BoltDB, SQLite, UoW, Hybrid QueryService |
| **M3: Domain Enrichment Complete** | ~Jan 3, 2026  | 3.23-3.26 | Rich domain models, validation docs      |
| **M4: Configuration Complete**     | ~Jan 3, 2026  | 3.27-3.29 | Singletons, FileClassKey, orchestration  |
| **M5: Epic 3 Complete**            | ~Jan 17, 2026 | 3.30-3.31 | Hybrid DI/E2E, patterns documentation    |

### Dependency on Other Epics

**No Blockers from Other Epics**:

- Epic 1 (CLI Foundation): ✅ Complete
- Epic 2 (Schema & Validation): ✅ Complete (minor tech debt deferred)
- Epic 4 (Template Queries): ⏸️ Waiting on Epic 3
- Epic 5 (Interactive Input): ⏸️ Waiting on Epic 3

**Epic 5 Template System Dependency**:

- Template cache refactoring (AI-2.3) deferred to Epic 5
- Estimated Epic 5 start: February 2026 (after Epic 3 completion)
- Template optimization: 2 points (LOW complexity)

---

## Architecture Documentation Recommendations

### Critical Documentation Updates (Story 3.31)

**1. Create `docs/architecture/patterns.md`** (NEW - HIGH PRIORITY)

**Purpose**: Central pattern catalog for all architectural patterns used in Lithos.

**Content Structure**:

```markdown
# Lithos Architecture Patterns

## Pattern Catalog

### 1. Hexagonal Architecture (Ports & Adapters)

- **Intent**: Isolate domain logic from infrastructure concerns
- **When to Use**: All domain/infrastructure boundaries
- **Implementation**: VaultReaderPort, CacheWriterPort, MarkdownParserPort
- **Trade-offs**: More abstraction vs flexibility and testability

### 2. CQRS (Command Query Responsibility Segregation)

- **Intent**: Separate read and write operations
- **When to Use**: Complex domain with different read/write needs
- **Implementation**: QueryService (read), IndexMaintenanceService (write)
- **Trade-offs**: Complexity vs performance and scalability

### 3. Unit of Work Pattern

- **Intent**: Maintain transactional consistency across multiple operations
- **When to Use**: Coordinating writes to multiple storage systems
- **Implementation**: CacheUnitOfWork (BoltDB + SQLite coordination)
- **Trade-offs**: Complexity vs data consistency

### 4. Repository Pattern

- **Intent**: Abstract data access behind collection-like interface
- **When to Use**: Domain needs persistence but shouldn't know how
- **Implementation**: VaultReaderPort, CacheReaderPort
- **Trade-offs**: Indirection vs testability and flexibility

### 5. Singleton Pattern

- **Intent**: Ensure single instance of global state
- **When to Use**: Configuration, shared resources
- **Implementation**: Config.Instance(), PropertyBank.Instance()
- **Trade-offs**: Global state vs convenient access

### 6. Factory Pattern

- **Intent**: Encapsulate object creation with validation
- **When to Use**: Complex construction or validation required
- **Implementation**: NewNote(), NewFrontmatter(), NewSchema()
- **Trade-offs**: More code vs validation enforcement

### 7. DTO Pattern (Layered Architecture)

- **Intent**: Transfer data between layers with different concerns
- **When to Use**: Crossing architectural boundaries
- **Implementation**: VaultFile (base), BoltDBMetadata, SQLiteMetadata
- **Trade-offs**: More types vs layer-specific optimization
```

**2. Update `docs/architecture/data-models.md`** (CRITICAL - Story 3.17)

**New Section**: VaultFile DTO Layered Architecture

````markdown
## VaultFile DTO Layered Architecture

### Design Philosophy

VaultFile uses a **layered DTO architecture** with three levels:

1. **Base Layer**: Leverages Go stdlib (fs.FileInfo) for file metadata
2. **Content Separation Layer**: Separates metadata-only from full content DTOs
3. **Storage-Specific Layer**: Optimized DTOs for each storage system

### Layer 1: Base DTO with fs.FileInfo

```go
type VaultFile struct {
    Path    string      // Vault-relative: "notes/meeting.md"
    Info    fs.FileInfo // Delegate to stdlib
    Content []byte      // Loaded on-demand
}

// Computed methods delegate to fs.FileInfo
func (vf VaultFile) ModTime() time.Time { return vf.Info.ModTime() }
func (vf VaultFile) Size() int64 { return vf.Info.Size() }
func (vf VaultFile) Basename() string { return filepath.Base(vf.Path) }
```
````

**Benefits**:

- No field duplication (ModTime, Size from fs.FileInfo)
- Cross-platform path handling (filepath.ToSlash/FromSlash)
- Vault-relative paths enable multi-vault support

### Layer 2: Content Separation

```go
type VaultFileMeta struct {
    Path string
    Info fs.FileInfo
    // NO Content - memory efficient for scanning
}

type VaultFileWithContent struct {
    VaultFileMeta
    Content []byte
}
```

**Benefits**:

- Memory efficiency: Don't load 1MB files when only need metadata
- Clear intent: ScanAll() vs ScanWithContent()

### Layer 3: Storage-Specific DTOs

```go
// BoltDB: Hot cache - minimal fields
type BoltDBMetadata struct {
    Path      string
    Basename  string
    Aliases   []string
    FileClass string
    ModTime   time.Time
}

// SQLite: Deep storage - complete metadata
type SQLiteMetadata struct {
    Path        string
    Frontmatter map[string]any
    ModTime     time.Time
    Size        int64
}
```

**Benefits**:

- Storage-specific optimization
- BoltDB: Sub-millisecond lookups (<1KB per note)
- SQLite: Rich queries with schema-driven views

````

**3. Update `docs/architecture/components.md`** (HIGH PRIORITY - Multiple Stories)

**Add New Components**:

```markdown
## Component Updates (Epic 3 Course Correction)

### New Ports

#### MarkdownParserPort (SPI)
**Purpose**: Abstract markdown parsing from domain layer.
**Location**: `/internal/ports/spi/markdown.go`
**Methods**:
- `ParseFrontmatter(ctx, content) (map[string]any, error)`

**Adapters**:
- `GoldmarkParserAdapter` - Uses goldmark library for parsing

**Rationale**: Hexagonal architecture compliance - parsing is infrastructure concern.

#### MetadataQueryPort (SPI)
**Purpose**: Enable O(1) indexed queries on metadata.
**Location**: `/internal/ports/spi/metadata_query.go`
**Methods**:
- `TagQuery(ctx, tag) ([]domain.Note, error)`
- `FileClassQuery(ctx, fileClass) ([]domain.Note, error)`
- `FrontmatterQuery(ctx, field, value) ([]domain.Note, error)`

**Adapters**:
- `SQLiteReader` (also implements CacheReaderPort)

**Rationale**: Performance - indexed queries vs O(n) scanning.

### New Services

#### IndexMaintenanceService (Application Layer)
**Purpose**: CQRS command side - index rebuilding and maintenance.
**Location**: `/internal/app/index/maintenance_service.go`
**Methods**:
- `RefreshIncremental(since time.Time) error`
- `RebuildIndex() error`

**Rationale**: CQRS compliance - separate read (QueryService) from write operations.

#### CacheUnitOfWork (Application Layer)
**Purpose**: Coordinate dual-write to BoltDB + SQLite with transactional guarantees.
**Location**: `/internal/app/cache/unit_of_work.go`
**Methods**:
- `Begin() error`
- `AddWrite(note domain.Note) error`
- `AddDelete(id domain.NoteID) error`
- `Commit(ctx) error`
- `Rollback(ctx) error`

**Rationale**: Data consistency - prevent BoltDB/SQLite divergence.

### Updated Services

#### QueryService (Application Layer)
**Changes**:
- Now accepts both BoltDBReader and SQLiteReader
- Query routing: Hot path (BoltDB) vs Deep path (SQLite)
- Removed RefreshFromCache() (moved to IndexMaintenanceService)
- Now read-only (true CQRS query side)

#### FrontmatterService (Application Layer)
**Changes**:
- Removed goldmark parsing (now uses MarkdownParserPort)
- Renamed Validate() to IsSchemaCompliant()
- Pure semantic validation (schema compliance only)
- Zero infrastructure dependencies
````

**4. Update `docs/architecture/coding-standards.md`** (HIGH PRIORITY - Story 3.26)

**Add Section**: Validation Layer Separation

````markdown
## Validation Layer Separation

### Principle

Lithos follows **Hexagonal Architecture** validation layer separation:

| Layer             | Validation Type | Purpose                   | Examples                                  |
| ----------------- | --------------- | ------------------------- | ----------------------------------------- |
| **Adapter Layer** | **Syntactic**   | Structure/format checking | YAML parsing, JSON structure, file format |
| **Domain Layer**  | **Semantic**    | Business rules checking   | Schema compliance, business invariants    |

### Naming Convention

**Adapter Layer (Syntactic Validation)**:

- `ValidateSyntax(data) error` - Returns error if structure invalid
- `IsValidSyntax(data) bool` - Returns boolean for simple checks
- `IsWellFormed(data) bool` - Alternative name for structure checks

**Domain Layer (Semantic Validation)**:

- `Validate() error` - Entity validates itself (business rules)
- `IsSchemaCompliant() error` - Service validates schema compliance
- `Satisfy(constraint) bool` - Predicate-based validation

### Implementation Examples

**Adapter Layer - GoldmarkParserAdapter**:

```go
func (a *GoldmarkParserAdapter) ParseFrontmatter(ctx context.Context, content []byte) (map[string]any, error) {
    // Syntactic validation: YAML structure
    if !isValidYAML(content) {
        return nil, fmt.Errorf("invalid YAML syntax at line %d", lineNum)
    }
    // Parse and return
}
```
````

**Domain Layer - Frontmatter Entity**:

```go
func (f Frontmatter) Validate() error {
    // Semantic validation: business rules
    if f.FileClass == "" {
        return errors.New("fileClass is required")
    }
    // Additional domain rules
}
```

**Domain Layer - FrontmatterService**:

```go
func (s *FrontmatterService) IsSchemaCompliant(fm domain.Frontmatter, schema domain.Schema) error {
    // Semantic validation: schema compliance
    for _, prop := range schema.Properties {
        if prop.Required && !fm.HasField(prop.Name) {
            return fmt.Errorf("required field %s is missing", prop.Name)
        }
    }
}
```

### Decision Tree

When implementing validation, ask:

1. **Is this checking structure/format?** → Adapter Layer (syntactic)
   - Examples: "Is this valid YAML?", "Is this a valid file path?"

2. **Is this checking business rules?** → Domain Layer (semantic)
   - Examples: "Does this satisfy schema requirements?", "Is this a valid state transition?"

3. **Does validation need infrastructure (file I/O, database)?** → Adapter Layer
   - Examples: "Does this file exist?", "Is this ID unique in database?"

```

---

## Final Synthesis Summary

### Course Correction Outcome

**Initial State** (November 2, 2025):
- Epic 3 Stories 3.1-3.16 mostly complete
- Stories 3.17-3.18 blocked awaiting hybrid BoltDB+SQLite architecture
- 6 architectural questions identified (Questions 1-5 resolved, Question 6 unresolved)

**Discovery** (November 5-8, 2025):
- Comprehensive architectural review revealed **18+ systemic issues** across **8 groups**
- Fundamental misunderstanding of hexagonal architecture and DDD principles
- Anemic domain model anti-pattern pervasive across all entities
- Validation layer confusion (syntactic vs semantic)
- DTO architecture not leveraging Go idioms

**Resolution** (November 8, 2025):
- **14 new stories** (45 points, 4-5 sprints) inserted into Epic 3
- Dependency-ordered groups: Foundation → Storage → Services → Config → Completion
- Final Epic 3: **31 stories** (was 18)
- Comprehensive patterns catalog and architecture documentation updates
- Clear path forward to Epic 3 completion with solid architectural foundation

### Success Criteria for Course Correction

**Epic 3 Completion will be considered successful when:**

1. ✅ **All 31 stories completed** (3.1-3.31)
2. ✅ **Hexagonal architecture compliance** verified:
   - No infrastructure code in domain layer
   - Clear syntactic/semantic validation separation
   - All parsing in adapter layer
3. ✅ **Rich domain models** implemented:
   - Frontmatter and Note entities with validation and behavior
   - Factory pattern enforced for entity creation
4. ✅ **Hybrid storage operational**:
   - BoltDB hot path queries < 1ms
   - SQLite deep path queries < 50ms
   - Unit of Work ensures consistency
5. ✅ **Production-scale validation**:
   - 500+ note test vault
   - Template queries < 100ms
   - Memory usage stable
6. ✅ **Architecture documentation complete**:
   - patterns.md created with 7 pattern explanations
   - All architecture docs updated
   - Validation naming convention documented

### Risk Mitigation Summary

**Top 5 Risks Identified**:

1. **Storage Layer Complexity** (Sprint 2-3)
   - **Risk**: BoltDB/SQLite dual-write transaction rollback failures
   - **Mitigation**: Comprehensive error handling, test all failure modes, incremental commits

2. **VaultFile DTO Impact** (Sprint 1)
   - **Risk**: Refactor ripples across many adapters
   - **Mitigation**: Update all usages in same story, comprehensive integration tests

3. **Integration Complexity** (Sprint 5)
   - **Risk**: DI wiring failures, E2E test failures at scale
   - **Mitigation**: Incremental integration, test at each layer boundary

4. **View Generation Complexity** (Sprint 2-3)
   - **Risk**: PropertySpec → SQL type mapping errors
   - **Mitigation**: Start with simple types (string, integer), incremental expansion

5. **Timeline Pressure** (All Sprints)
   - **Risk**: 45 points over 5 sprints requires sustained velocity
   - **Mitigation**: Sprint buffer built in (10-13 point capacity), tech debt deferral to post-Epic 3

### Next Steps

1. **Immediate**: Review this Synthesis Phase with stakeholders for approval
2. **Sprint 1 Kickoff**: Begin Story 3.17 (VaultFile DTO Redesign)
3. **Ongoing**: Weekly progress reviews against milestone dates
4. **Sprint 5**: Epic 3 completion celebration and retrospective 🎉

---

_Synthesis Phase Complete. Epic 3 Course Correction Plan Finalized._
```
