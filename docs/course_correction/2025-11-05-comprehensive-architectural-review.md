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

1. **Group 1**: Validation Architecture (anemic models, IO in domain, validation layers) - *Section 1 complete*
2. **Group 2**: Storage Architecture, CQRS & DTOs - *Section 1 complete*
3. **Group 3**: Orchestration & Coordination - *Section 1 complete*
4. **Group 4**: Configuration Management - *Section 1 complete*
5. **Group 5**: Schema Domain System - *Section 1 complete*
6. **Group 6**: Template System (Epic 5 dependency) - *Section 1 complete*
7. **Group 7**: Documentation & Patterns - *Section 1 complete*
8. **Group 8**: Implementation Blockers - *Section 1 complete*

**Current Status**: All Groups Section 1 (Understand Trigger & Context) complete. Proceeding to Research Phase (Go stdlib + Obsidian patterns).

**Expected Outcome**: Comprehensive story plan with sequencing, dependencies, and risk mitigation for completing Epic 3 with correct architectural foundation.

---

## Document Control

- **Version**: 1.6
- **Date**: November 6, 2025
- **Status**: IN PROGRESS - Section 1 complete for all 8 groups, proceeding to Research Phase
- **Distribution**: Development team, stakeholders

### Change Log

| Date       | Version | Description                                                                                                                                                                                                                       | Author     |
| ---------- | ------- | --------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- | ---------- |
| 2025-11-06 | 1.6     | Completed Section 1 (Understand Trigger & Context) for all 8 groups: comprehensive analysis of 18+ architectural issues with critical evaluation, code evidence, and impact assessment; ready for Research Phase (Go stdlib + Obsidian patterns) | Sarah (PO) |
| 2025-11-06 | 1.5     | Restructured Structured Plan to phase-based approach (Section 1 all groups → Research → Entity Review → Synthesis → Epic Impact); moved Action Items under Structured Plan; added Epic Impact Assessment placeholder section      | Sarah (PO) |
| 2025-11-06 | 1.4     | Enhanced Executive Summary with full background (Nov 2 sprint change, 6 architectural questions, course correction trigger); replaced Action Items with detailed, specific breakdown for all 8 groups + research/synthesis phases | Sarah (PO) |
| 2025-11-06 | 1.3     | Reorganized document structure: moved analysis results under corresponding groups in Structured Analysis Plan; added progress checkboxes to each group; removed duplicate sections; reduced file from 980 to 741 lines            | Sarah (PO) |
| 2025-11-06 | 1.2     | Completed Group 1 Section 1 comprehensive analysis (Issues D1, B2, Hexagonal Principle) with code evidence from FrontmatterService, VaultReaderAdapter, and domain entities; ready for Section 2 Epic Impact Assessment           | Sarah (PO) |
| 2025-11-05 | 1.1     | Established structured analysis plan (8 issue groups); revised Group 2 to include missing storage/CQRS issues; moved SQLite to storage group; increased issue count to 18+                                                        | Sarah (PO) |
| 2025-11-05 | 1.0     | Initial comprehensive issue inventory (15 issues); established hexagonal validation principle; completed Section 1 for Issue D1                                                                                                   | Sarah (PO) |

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

#### Issue A1: Component Orchestration Architecture ❌ UNRESOLVED

- **Status**: Reconsidering - need to evaluate event-driven vs orchestrator patterns
- **Missing Consideration**: Event-driven architecture as solution to god-object problem
- **Questions**:
  - Should we use event-driven design for complex orchestration?
  - Would domain events (NoteIndexed, FrontmatterValidated, SchemaLoaded) reduce coupling?
  - How does event-driven approach compare to orchestrator pattern?
- **Implementation Pending**: All refactoring work from Question 1 decision

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

- [ ] Group 1: Validation Architecture

  - [ ] Identify which Epic 3 stories require validation refactoring
  - [ ] Determine story breakdown: Frontmatter entity refactoring, Note entity refactoring, validation layer separation
  - [ ] Assess FrontmatterService.Extract() extraction to adapter layer
  - [ ] Evaluate Template entity impact (Epic 5 dependency)
  - [ ] Document refactoring sequence and dependencies

- [ ] Group 2: Storage Architecture, CQRS & DTOs

  - [ ] Determine FileMetadata/VaultFile redesign leveraging fs.FileInfo
  - [ ] Design BoltDB vs SQLite query routing strategy
  - [ ] Select write coordination pattern (UoW, Saga, or dual-write)
  - [ ] Plan storage staleness detection implementation

- [ ] Group 3: Orchestration & Coordination

  - [ ] Select orchestration pattern and document rationale
  - [ ] Plan CLICommander refactoring if needed
  - [ ] Design event infrastructure if event-driven approach selected

- [ ] Group 4: Configuration Management

  - [ ] Plan singleton accessor implementation (GetConfig, GetPropertyBank)
  - [ ] Design test harness support for instance swapping
  - [ ] Document Config embedded struct pattern for extensibility

- [ ] Group 5: Schema Domain System

  - [ ] Determine port simplification approach
  - [ ] Plan schema loading workflow refactoring

- [ ] Group 6: Template System (CRITICAL - Epic 5 Dependency)

  - [ ] Make Template struct fundamental decision
  - [ ] Assess Epic 5 (Template Engine) impact and timeline
  - [ ] Plan Template entity refactoring if keeping struct

- [ ] Group 7: Documentation & Patterns (META)

  - [ ] Synthesize patterns discovered across Groups 1-6
  - [ ] Plan architecture documentation updates (components.md, data-models.md)
  - [ ] Create pattern decision matrix

- [ ] Group 8: Implementation Blockers (META)
  - [ ] Assess implementation roadmap across all groups
  - [ ] Determine story renumbering strategy (push 3.17-3.18 after new stories)
  - [ ] Plan documentation updates timing

### Synthesis Phase (After All Groups Complete)

- [ ] Create cross-issue dependency map
- [ ] Consolidate epic impact findings from all 8 groups
- [ ] Develop comprehensive story plan:
  - [ ] Story breakdown with acceptance criteria
  - [ ] Story sequencing based on dependencies
  - [ ] Effort estimates per story
  - [ ] Risk assessment and mitigation strategies
- [ ] Renumber Epic 3 stories (insert new stories, push 3.17-3.18 to end)
- [ ] Update Epic 3 timeline and milestones
- [ ] Final architecture documentation updates

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

**Root Cause**: Orchestrator pattern attempted but resulted in god-object, no comprehensive evaluation of alternative patterns (Event-Driven, Saga, Mediator, etc.) and their interaction with DI pattern.

##### 1.3 Is this a misunderstanding, missing consideration, or new information?

**Missing Consideration (Primary)**: Alternative orchestration patterns not evaluated

- **What was missed**: Comprehensive evaluation of orchestration patterns and their trade-offs
- **Patterns not considered**: Event-Driven, Saga, Mediator, Command, Unit of Work
- **Question not asked**: Which pattern best fits our orchestration needs?
- **Consequence**: Defaulted to orchestrator pattern without comparing alternatives

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
3. **Embedding**: If kept, should Template embed *template.Template for stdlib access?
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
3. If keeping domain.Template, should it embed *template.Template for rich behavior?
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

- [ ] FileInfo, File, FS interfaces, WalkDir, GlobFS, patterns

##### path/filepath

- [ ] <https://pkg.go.dev/path/filepath>

##### text/template

- [ ] Template composition, function maps, execution patterns

#### Go Third-Party Packages

##### bbolt

- [ ] Bucket design, transaction patterns, cursor usage, best practices

##### sqlite (modernc.org/sqlite)

- [ ] Schema patterns, query optimization, Go idioms

##### goldmark

Local References:

- `docs/refs/yuin-goldmark-digest.txt`
- `docs/refs/abhinav-goldmark-frontmatter-digest.txt`

- [ ] Parser API, AST manipulation, extension patterns, frontmatter extraction

#### Go Generics

- [ ] Generic types, interfaces, constraints, patterns

### Phase 2: Obsidian Patterns (After Phase 1)

- [ ] Survey Obsidian API index for all relevant models
- [ ] Map Obsidian patterns to Go capabilities
- [ ] Identify gaps between Go native and Obsidian solutions
- [ ] Extract architectural patterns applicable to our domain

---

## Entity Review Scope

### System 1: Schema System

- [ ] Schema - currently has Validate() (should move to adapter per hexagonal principle)
- [ ] PropertyBank - singleton pattern, needs method review
- [ ] Property - has Validate() (delegates to Spec), needs review
- [ ] PropertySpec - interface with variants, needs review

### System 2: Note System

- [ ] Note - anemic (just ID + Frontmatter), needs behavior methods
- [ ] NoteID - simple identifier, likely fine
- [ ] Frontmatter - CRITICAL needs refactoring (validation in adapter, factory in domain)

### System 3: Config System

- [ ] Config - needs embedded struct analysis for extensibility
- [ ] Should break into: VaultConfig, SchemaConfig, TemplateConfig, LoggingConfig
- [ ] Needs method review: Validate(), Resolve(), computed paths

### System 4: Template System

- [ ] Template - CRITICAL QUESTIONS:
  - Name conflict with text/template package?
  - Do we even need Template struct given stdlib?
  - If kept, should embed \*template.Template?
  - Is it fully utilizing text/template features?

### System 5: File/Storage DTOs

- [ ] FileMetadata - needs redesign with fs.FileInfo
- [ ] VaultFile - needs redesign review

---

## Epic Impact Assessment

---

## Key Architectural Principles Established

### Hexagonal Architecture Validation Layers

- **Adapter Layer**: Syntactic validation (structure/format checking)
  - YAML parsing validation
  - JSON schema structure validation
  - File format validation
- **Domain Layer**: Semantic validation (business rules checking)
  - Schema compliance validation
  - Business invariant enforcement
  - Cross-entity constraint validation

### Rich vs Anemic Models

- **Rich Models**: Entities with behavior methods for logic pertaining to their own data
- **Anemic Models**: Just data bags (anti-pattern)
- **Guideline**: If logic uses only entity's own data → method belongs on entity

### Validation Naming Convention (Proposed)

- `Validate()` - semantic validation in domain entity
- `IsSchemaCompliant()` - semantic validation in application service
- `IsValidSyntax()` - syntactic validation on input data in adapter layer
- `ValidateSyntax()` - boolean syntactic check
- `IsWellFormed()` - alternative syntactic check

---

*This document will be updated as the course correction process continues.*
