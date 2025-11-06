# Comprehensive Architectural Review - November 5, 2025

**Status**: IN PROGRESS - Change Checklist Section 2
**Trigger**: Story 3.2 implementation exposed systematic architectural issues
**Scope**: 15+ identified issues requiring resolution before Epic 3 completion

---

## Executive Summary

During implementation of Story 3.2 (Multi-Storage Cache Adapters), systematic architectural issues were discovered that require comprehensive review and refactoring before proceeding with Epic 3 completion. This document captures the course correction analysis using the BMad Change Navigation Checklist.

**Critical Architectural Principle Identified**:

- **Syntactic Validation** (structure/format checking) → **Adapter Layer**
- **Semantic Validation** (business rules checking) → **Domain Layer**

This principle fundamentally changes how we approach validation across the system.

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

## Summary Metrics

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

## Structured Analysis Plan

**Approach**: Option A - Full Sequential Analysis

- Complete Sections 1-2 for each issue group systematically
- Document findings in this file after each group
- Synthesize all findings into comprehensive story/epic plan after all groups complete

### Group 1: Validation Architecture (3 issues - FOUNDATIONAL)

**Issues**:

- **D1**: Anemic Domain Model + Validation Naming Ambiguity
- **B2**: IO in Domain Layer (FrontmatterService.Extract)
- **Hexagonal Principle**: Syntactic (adapter) vs Semantic (domain) validation

**Why Grouped**: All about where validation logic belongs in hexagonal architecture

**Analysis Status**: 🔄 NEXT - Starting with Section 1

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

**Key Questions to Answer**:

- Do we need separate CQRS read/write models or just separated operations?
- How should query routing work between BoltDB and SQLite?
- What coordination pattern for BoltDB+SQLite writes? (UoW? Saga? Dual-write?)
- How do DTOs map to each storage system (BoltDB, SQLite, JSON)?
- How do we leverage Go's fs.FileInfo instead of custom FileMetadata?
- What Obsidian patterns apply to our DTO design?
- How does QueryService interface with different storage backends?
- What does "cache as projection" mean for our architecture?

**Analysis Status**: ⏸️ PENDING - After Group 1

---

### Group 3: Orchestration & Coordination (2 issues - SYSTEM-WIDE)

**Issues**:

- **A1**: Component Orchestration (Event-driven vs Orchestrator pattern)
- **Related to A6**: Write coordination pattern (overlaps with storage)

**Why Grouped**: System-wide coordination patterns affecting component communication

**Analysis Status**: ⏸️ PENDING - After Group 2

---

### Group 4: Configuration Management (2 issues - INFRASTRUCTURE)

**Issues**:

- **A2**: Singleton Pattern for Config/PropertyBank
- **A3**: FileClassKey Configuration Impact

**Why Grouped**: Both about configuration architecture and lifecycle

**Analysis Status**: ⏸️ PENDING - After Group 3

---

### Group 5: Schema Domain System (1 issue - DOMAIN SPECIFIC)

**Issues**:

- **B3**: Schema Loading/Registration Coupling (SchemaLoaderPort vs SchemaRegistryPort)

**Why Grouped**: Schema-specific domain concern (A5 SQLite moved to Group 2 Storage)

**Analysis Status**: ⏸️ PENDING - After Group 4

---

### Group 6: Template System (1 issue - CRITICAL DEPENDENCY)

**Issues**:

- **Template Struct Analysis**:
  - Name conflict with text/template package?
  - Do we even need Template struct given stdlib?
  - If kept, should embed \*template.Template?
  - Is it fully utilizing text/template features?

**Why Standalone**: Epic 5 depends on this resolution; needs deep analysis of stdlib usage

**Analysis Status**: ⏸️ PENDING - After Group 5

---

### Group 7: Documentation & Patterns (1 issue - META)

**Issues**:

- **D3**: Missing Pattern Documentation

**Why Standalone**: Meta-issue about documenting patterns discovered in other groups

**Analysis Status**: ⏸️ PENDING - After Groups 1-6 (will synthesize patterns from all groups)

---

### Group 8: Implementation Blockers (3 issues - META)

**Issues**:

- **C1**: Multiple Questions Pending Implementation
- **C2**: Question 6 Unresolved
- **C3**: Documentation Misalignment

**Why Grouped**: Meta-issues about implementation state and process

**Analysis Status**: ⏸️ PENDING - After Groups 1-7 (will assess implementation roadmap)

---

## Research Strategy

### Phase 1: Go Native Capabilities (Priority)

- [ ] **io/fs package**: FileInfo, File, FS interfaces, WalkDir patterns
- [ ] **text/template**: Template composition, function maps, execution patterns
- [ ] **bbolt**: Bucket design, transaction patterns, cursor usage, best practices
- [ ] **sqlite (modernc.org/sqlite)**: Schema patterns, query optimization, Go idioms
- [ ] **goldmark**: Parser API, AST manipulation, extension patterns, frontmatter extraction

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

## Change Checklist Progress

### Group 1: Validation Architecture - Section 1 Analysis ✅ COMPLETE

**Issues Analyzed**:

- **Issue D1**: Anemic Domain Model + Validation Naming Ambiguity
- **Issue B2**: IO in Domain Layer (FrontmatterService.Extract)
- **Hexagonal Principle**: Syntactic vs Semantic validation layer separation

#### 1.1 What triggered this change?

**Immediate Trigger**: Story 3.2 implementation revealed FrontmatterService.Extract() performs file parsing (IO operations) in domain layer.

**Broader Discovery**: During architectural review, identified pervasive anemic domain model anti-pattern across all entities (Frontmatter, Note, Template, Property) and inconsistent validation placement.

**User Observation**: Direct identification that entities are "just data bags" with all logic in services, violating DDD rich domain model principles.

#### 1.2 What is the core issue?

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

#### 1.3 Is this a misunderstanding, missing consideration, or new information?

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

#### 1.4 What is the impact if we don't address this?

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

#### 1.5 What evidence supports this change?

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
   FrontmatterService.ValidateAgainstSchema() (domain) → semantic validation only
   ```

**Validation Naming Ambiguity**:

| Current Method                | Type      | Validates         | Data Required    | Current Layer | Correct Layer                            |
| ----------------------------- | --------- | ----------------- | ---------------- | ------------- | ---------------------------------------- |
| Schema.Validate()             | Syntactic | JSON structure    | Schema only      | Domain        | **Adapter**                              |
| Frontmatter.Validate()        | Syntactic | YAML structure    | Frontmatter only | **Missing!**  | Adapter                                  |
| FrontmatterService.Validate() | Semantic  | Schema compliance | FM + Schema      | Domain        | Domain (rename to ValidateAgainstSchema) |

---

### Group 1: Validation Architecture - Section 2 Analysis ⏸️ NEXT

**Status**: Ready to begin Epic Impact Assessment for Issues D1, B2, Hexagonal Principle

**Questions to Answer**:

- Which Epic 3 stories are impacted?
- What new stories are needed?
- How does this affect Epic 5 (Template Engine)?
- What is the refactoring sequence?

---

### Overall Progress Tracking

**Section 1 (Understand Trigger & Context)**:

- ✅ Group 1 (Validation Architecture): Complete
- ⏸️ Groups 2-8: Pending

**Section 2 (Epic Impact Assessment)**:

- ⏸️ Group 1: Next task
- ⏸️ Groups 2-8: Pending

**Synthesis Phase**: Pending all group analyses

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

- `Validate()` - syntactic validation on entity (adapter layer)
- `ValidateAgainstSchema()` - semantic validation in service (domain layer)
- `IsValid()` - boolean syntactic check
- `IsWellFormed()` - alternative syntactic check

---

## Document Control

- **Version**: 1.1
- **Date**: November 5, 2025
- **Status**: IN PROGRESS - Preparing Group 1 Analysis (Validation Architecture)
- **Distribution**: Development team, stakeholders

### Change Log

| Date       | Version | Description                                                                                                                                                                                                             | Author     |
| ---------- | ------- | ----------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- | ---------- |
| 2025-11-06 | 1.2     | Completed Group 1 Section 1 comprehensive analysis (Issues D1, B2, Hexagonal Principle) with code evidence from FrontmatterService, VaultReaderAdapter, and domain entities; ready for Section 2 Epic Impact Assessment | Sarah (PO) |
| 2025-11-05 | 1.1     | Established structured analysis plan (8 issue groups); revised Group 2 to include missing storage/CQRS issues; moved SQLite to storage group; increased issue count to 18+                                              | Sarah (PO) |
| 2025-11-05 | 1.0     | Initial comprehensive issue inventory (15 issues); established hexagonal validation principle; completed Section 1 for Issue D1                                                                                         | Sarah (PO) |

---

## Conversation Log

### Initial Issue Identification

User identified three critical issues during Story 3.2 implementation:

1. QueryService/Note struct mismatch
2. IO in domain layer (FrontmatterService.Extract)
3. Schema loading/registration coupling

### Comprehensive Inventory Development

- Initial inventory: 12 issues
- User identified missing considerations:
  - Event-driven architecture option (Issue A1)
  - DTO redesign with Go idioms + Obsidian patterns (Issue A4)
  - Unit of Work pattern (Issue A6)
  - Anemic model anti-pattern (Issue D1)
- Revised inventory: 15 issues

### Critical Architectural Principle Discovery

User clarified hexagonal architecture validation principle:

- **Syntactic validation → Adapter layer**
- **Semantic validation → Domain layer**

This fundamentally changes validation placement across the system.

### Section 2 Process Error

User correctly identified that Section 2 analysis only covered Issue D1, not all 15 issues.
Analysis must be comprehensive across all issues before proceeding.

---

## Action Items

### Immediate

- [ ] Complete Section 2 epic impact analysis for ALL 15 issues
- [ ] Determine comprehensive story modifications for Epic 3
- [ ] Assess Epic 5 dependency on Template analysis

### Research Phase

- [ ] Phase 1: Go stdlib and primary packages research
- [ ] Phase 2: Obsidian API pattern extraction
- [ ] Gap analysis and pattern mapping

### Implementation Phase

- [ ] Story 3.19+: Rich domain model refactoring (split into multiple stories)
- [ ] Story 3.20+: Validation layer corrections (split into multiple stories)
- [ ] Update Stories 3.17-3.18 to test/document corrected architecture

---

_This document will be updated as the course correction process continues._
