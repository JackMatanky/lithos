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
2. **Group 2**: Storage Architecture, CQRS & DTOs - *Pending*
3. **Group 3**: Orchestration & Coordination - *Pending*
4. **Group 4**: Configuration Management - *Pending*
5. **Group 5**: Schema Domain System - *Pending*
6. **Group 6**: Template System (Epic 5 dependency) - *Pending*
7. **Group 7**: Documentation & Patterns - *Pending*
8. **Group 8**: Implementation Blockers - *Pending*

**Current Status**: Group 1 Section 1 complete. Groups 2-8 Section 1 (Understand Trigger & Context) in progress.

**Expected Outcome**: Comprehensive story plan with sequencing, dependencies, and risk mitigation for completing Epic 3 with correct architectural foundation.

---

## Document Control

- **Version**: 1.5
- **Date**: November 6, 2025
- **Status**: IN PROGRESS - Groups 2-8 Section 1 (Understand Trigger & Context)
- **Distribution**: Development team, stakeholders

### Change Log

| Date       | Version | Description                                                                                                                                                                                                                       | Author     |
| ---------- | ------- | --------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- | ---------- |
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
- [ ] **Group 4: Configuration Management**
  - [ ] Review singleton implementation for Config and PropertyBank (Issue A2)
  - [ ] Analyze FileClassKey configuration impact (Issue A3)
  - [ ] Examine ViperAdapter FileClassKey loading gap
- [ ] **Group 5: Schema Domain System**
  - [ ] Analyze SchemaLoaderPort and SchemaRegistryPort coupling (Issue B3)
  - [ ] Review automatic registration vs explicit loading
- [ ] **Group 6: Template System (CRITICAL - Epic 5 Dependency)**
  - [ ] Investigate Template struct name conflict with text/template package
  - [ ] Research text/template stdlib capabilities
  - [ ] Determine if Template struct is even needed
  - [ ] Analyze whether to embed \*template.Template
- [ ] **Group 7: Documentation & Patterns (META)**
  - [ ] Catalog pattern documentation gaps (Issue D3)
  - [ ] Review architectural documentation misalignment
- [ ] **Group 8: Implementation Blockers (META)**
  - [ ] Review Questions 1-5 pending implementations (Issue C1)
  - [ ] Analyze Question 6 unresolved status (Issue C2)
  - [ ] Document architecture documentation misalignment (Issue C3)

### Research Phase (Parallel with Analysis)

**Phase 1: Go Native Capabilities** (Priority - understand before Obsidian)

- [ ] Research io/fs package (FileInfo, File, FS interfaces, WalkDir patterns)
- [ ] Research text/template package (composition, function maps, execution patterns)
- [ ] Research bbolt package (bucket design, transactions, cursor usage, best practices)
- [ ] Research modernc.org/sqlite (schema patterns, query optimization, Go idioms)
- [ ] Research goldmark package (parser API, AST manipulation, extension patterns, frontmatter extraction)

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

##### 1.2 What is the core issue?

##### 1.3 Is this a misunderstanding, missing consideration, or new information?

##### 1.4 What is the impact if we don't address this?

##### 1.5 What evidence supports this?

---

### Group 5: Schema Domain System (1 issue - DOMAIN SPECIFIC)

**Issues**:

- **B3**: Schema Loading/Registration Coupling (SchemaLoaderPort vs SchemaRegistryPort)

**Why Grouped**: Schema-specific domain concern (A5 SQLite moved to Group 2 Storage)

##### 1.1 What triggered this change?

##### 1.2 What is the core issue?

##### 1.3 Is this a misunderstanding, missing consideration, or new information?

##### 1.4 What is the impact if we don't address this?

##### 1.5 What evidence supports this?

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

##### 1.2 What is the core issue?

##### 1.3 Is this a misunderstanding, missing consideration, or new information?

##### 1.4 What is the impact if we don't address this?

##### 1.5 What evidence supports this?

---

### Group 7: Documentation & Patterns (1 issue - META)

**Issues**:

- **D3**: Missing Pattern Documentation

**Why Standalone**: Meta-issue about documenting patterns discovered in other groups

##### 1.1 What triggered this change?

##### 1.2 What is the core issue?

##### 1.3 Is this a misunderstanding, missing consideration, or new information?

##### 1.4 What is the impact if we don't address this?

##### 1.5 What evidence supports this?

---

### Group 8: Implementation Blockers (3 issues - META)

**Issues**:

- **C1**: Multiple Questions Pending Implementation
- **C2**: Question 6 Unresolved
- **C3**: Documentation Misalignment

**Why Grouped**: Meta-issues about implementation state and process

##### 1.1 What triggered this change?

##### 1.2 What is the core issue?

##### 1.3 Is this a misunderstanding, missing consideration, or new information?

##### 1.4 What is the impact if we don't address this?

##### 1.5 What evidence supports this?

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
