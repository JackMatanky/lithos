# Sprint Change Proposal: Complete Architecture Alignment

**Date:** 2025-10-27
**Proposal ID:** SCP-2025-001
**Trigger:** Architecture review during Epic 3 planning revealed critical violations and coverage gaps
**Prepared By:** Sarah (Product Owner)
**Status:** Pending Approval
**Severity:** CRITICAL - Blocks Epic 3+ implementation

---

## EXECUTIVE SUMMARY

### Issue Summary

During Epic 3 (Vault Indexing Engine) planning, comprehensive architectural review identified **two critical problems**:

1. **Implementation Violations:** Epic 1-2 code contains 6 critical architectural violations
2. **Architecture Coverage Gaps:** Epics/stories don't cover ALL architecture components from v0.6.8

**Root Cause:**

- PRD/Epics written before architecture was finalized (v0.1-0.4)
- Epic/story creation not verified against complete architecture inventory
- Epic 1-2 implemented with violations baked into code

**Scope of Impact:**

- **Code:** 7 files with critical violations in production
- **Stories:** ALL ~30+ story files reference old architecture
- **Coverage:** Unknown number of architecture components not represented in epics

**Good News:**

- Epic 3+ not implemented yet - violations caught before major waste
- Architecture fully documented and correct (v0.6.8)

### Recommended Path Forward

**🎯 Archive ALL → Architecture Coverage Verification → Complete Story Regeneration → Re-Implementation**

**Timeline:** 17-25 days before Epic 3 can proceed

**Why This Approach:**

1. ✅ AI-agent workflow: Rigorous planning is SLOW but enables FAST implementation
2. ✅ Archive ALL stories (not selective) - Epic 1-2 stories also reference old architecture
3. ✅ Architecture coverage verification - Ensure NO component forgotten
4. ✅ Complete story regeneration - Consistent quality and perfect alignment
5. ✅ ~54-60 rigorous stories enable deterministic AI implementation

---

## SECTION 1: TRIGGER & CONTEXT

### Triggering Event

**When:** October 24-26, 2025, during Epic 3 story planning
**What:** User-initiated comprehensive architecture review
**Why:** Planning Epic 3 stories triggered critical reflection on architecture quality

**Timeline of Events:**

- Oct 8, 2025: Initial architecture v0.1-0.4 created
- Oct 11-21, 2025: Epic 1-2 implemented with v0.1-0.4 architecture
- Oct 24-26, 2025: Architecture review during Epic 3 planning
- Oct 24-26, 2025: Comprehensive architecture corrections (v0.5.0 - v0.6.8)
- Oct 27, 2025: Course correction initiated (this proposal)

### Core Problem Statement

**The initial architecture (v0.1-0.4) contained fundamental hexagonal architecture violations and pattern misapplications that led to:**

- Tight coupling between domain and infrastructure layers
- Non-idiomatic Go patterns (Result[T] instead of (T, error))
- Violation of SOLID principles (SRP, ISP, DIP)
- Missing abstractions (no NoteID, VaultPorts)

**These violations were implemented in Epic 1-2 code and referenced in ALL story files.**

### Root Cause Analysis

**Why This Happened:**

1. **PRD/Epics written before architecture** - Implementation-focused thinking vs domain-driven design
2. **Initial architecture had violations** - Baked into Epic 1-2 stories and implementation
3. **No architecture coverage verification** - Components added to architecture (v0.5.0-v0.6.8) without updating epics
4. **Incremental story creation** - Stories created without holistic architecture review

**Impact Scope:**

- **Technical Debt:** 6 critical violations in production code
- **Epic Debt:** ALL epics reference incomplete/incorrect architecture
- **Story Debt:** ALL ~30+ stories need regeneration
- **Missing Coverage:** Unknown architecture components not in epics
- **Timeline Impact:** ~3-5 weeks before Epic 3 can proceed

### Could This Have Been Prevented?

**Partially Yes:**

- ✅ Architecture-first approach (architecture → epics → stories)
- ✅ Architecture coverage checklist during epic planning
- ✅ Periodic epic-to-architecture alignment reviews

**Good Timing:**

- ✅ Caught before Epic 3 implementation
- ✅ Comprehensive architecture corrections completed (v0.6.8)
- ✅ All violations documented in architecture-review-and-refactoring-plan.md

---

## SECTION 2: EPIC IMPACT ASSESSMENT

### Current Epic Analysis

**Epic 1: Foundational CLI (IMPLEMENTED - Has Violations)**

| Story    | Status      | Violation                                         | Action                             |
| -------- | ----------- | ------------------------------------------------- | ---------------------------------- |
| 1.1-1.2  | ✅ Complete | ✅ None                                           | Archive story file, regenerate     |
| **1.3**  | ✅ Complete | 🔴 File, Note, Template models - layer violations | Archive code + story, re-implement |
| **1.4**  | ✅ Complete | 🔴 Result[T] pattern throughout                   | Archive code + story, re-implement |
| 1.5-1.6  | ✅ Complete | ✅ None (Logger, Registry correct)                | Archive story file, regenerate     |
| **1.7**  | ✅ Complete | 🔴 FileSystemPort (YAGNI violation)               | Archive code + story, re-implement |
| 1.8-1.13 | ✅ Complete | ✅ None                                           | Archive story files, regenerate    |

**Missing from Epic 1:**

- ⚠️ TemplateEngine service (referenced in architecture)
- ⚠️ CommandOrchestrator service (added v0.6.4)
- ⚠️ TemplatePort interface
- ⚠️ ConfigPort interface
- ⚠️ TemplateLoaderAdapter
- ⚠️ Explicit NoteID, FileMetadata, TemplateID model stories

**Epic 2: Configuration & Schema Loading (IMPLEMENTED - Needs Verification)**

| Story   | Status      | Issues                            | Action                          |
| ------- | ----------- | --------------------------------- | ------------------------------- |
| 2.1     | ✅ Complete | 🟡 Config in adapter layer        | Verify + move to domain         |
| 2.2-2.3 | ✅ Complete | ⚠️ Rich model verification needed | Verify Validate() methods exist |
| 2.6     | ✅ Complete | ⚠️ SchemaResolver service         | Verify service exists (v0.6.1)  |
| 2.7     | ✅ Complete | ⚠️ SchemaValidator service        | Verify service exists (v0.6.1)  |

**Missing from Epic 2:**

- ⚠️ SchemaEngine service story
- ⚠️ Explicit SchemaValidator service story (v0.6.1)
- ⚠️ Explicit SchemaResolver service story (v0.6.1)

**Epic 3: Vault Indexing Engine (DRAFTED - Completely Outdated)**

**All 9 Stories Outdated:**

- 3.1: References CacheCommandPort/QueryPort → should be CacheWriter/Reader
- 3.2: References single adapter → should be Write/Read split
- 3.3: References FileSystemPort → should be VaultReaderPort
- 3.4: Separate extraction → should be FrontmatterService (consolidated)
- 3.5: References VaultIndexer → needs focus verification
- 3.6: Orchestration → needs port/service alignment
- 3.7: QueryService → needs port/model alignment
- 3.8: CLI command → needs CommandOrchestrator callback pattern (v0.6.4)
- 3.9: Cache versioning → concepts valid, implementation details need update

**Missing from Epic 3:**

- 🔴 VaultReaderPort interface (added v0.6.8)
- 🔴 VaultReaderAdapter implementation (added v0.6.8)
- 🔴 VaultWriterPort interface (added v0.6.8)
- 🔴 VaultWriterAdapter implementation (added v0.6.8)
- 🔴 VaultFile DTO model (added v0.6.8)

**Epic 5 & 5: Future Epics (PLANNED - Need Review)**

**Potential Issues:**

- Template model changes (FilePath → TemplateID)
- CommandOrchestrator redesign (v0.6.4)
- Port/service name changes from Epic 3
- ALL story files reference potentially old architecture

### Architecture Components NOT Covered by Epics

**Domain Services (from components.md v0.6.8):**

- ❓ TemplateEngine - Which epic/story?
- ❓ FrontmatterService - Epic 3.4 (but not explicit)
- ❓ SchemaEngine - Which epic/story?
- ❓ SchemaValidator - Epic 2.7 (but needs verification)
- ❓ SchemaResolver - Epic 2.6 (but needs verification)
- ❓ VaultIndexer - Epic 3.5-3.6 (but needs focus verification)
- ❓ QueryService - Epic 3.7
- ❓ CommandOrchestrator - Which epic/story? (added v0.6.4)

**SPI Ports (13 total from components.md v0.6.8):**

- ❓ CacheWriterPort - Epic 3.1 (but wrong name in story)
- ❓ CacheReaderPort - Epic 3.1 (but wrong name in story)
- ❌ VaultReaderPort - NOT IN ANY EPIC (added v0.6.8)
- ❌ VaultWriterPort - NOT IN ANY EPIC (added v0.6.8)
- ❓ SchemaPort - Which epic/story?
- ❓ TemplatePort - Which epic/story?
- ❓ PromptPort - Epic 5 somewhere
- ❓ FinderPort - Epic 5 somewhere
- ❓ ConfigPort - Which epic/story?
- ❓ SchemaRegistryPort - Which epic/story?
- ✅ CLIPort - Epic 1 (but needs callback pattern v0.6.4)

**SPI Adapters (12 total from components.md v0.6.8):**

- Similar gaps and verification needed

**Domain Models (9 from data-models.md v0.6.8):**

- ❌ NoteID - NOT EXPLICIT in Epic 1
- ❓ Frontmatter - Epic 1.3 (but needs verification)
- ❓ Note - Epic 1.3 (but wrong composition)
- ❓ Schema - Epic 2.2
- ❓ Property - Epic 2.3
- ❓ PropertySpec - Epic 2.3
- ❓ PropertyBank - Epic 2.3
- ❌ TemplateID - NOT EXPLICIT in Epic 1
- ❓ Template - Epic 1.3 (but wrong structure)
- ❓ Config - Epic 2.1 (but wrong location)

**SPI Adapter Models (2 from data-models.md v0.6.8):**

- ❌ FileMetadata - NOT EXPLICIT (File exists in domain instead)
- ❌ VaultFile - NOT IN ANY EPIC (added v0.6.8)

### Summary: Complete Gap Analysis

**Critical Finding:** We cannot definitively say whether ALL architecture components are covered by epics because:

1. ❌ No architecture coverage checklist exists
2. ❌ Many components are "probably" covered but not explicitly
3. ❌ New components (v0.5.0-v0.6.8) may not be in epics
4. ❌ Component names in stories don't match architecture (File vs FileMetadata, CacheCommandPort vs CacheWriter)

**This is why we need Phase 2.0: Architecture Coverage Verification**

---

## SECTION 3: ARTIFACT CONFLICT & IMPACT ANALYSIS

### PRD Files (docs/prd/)

**epic-list.md:**

- Line 8: References "core Storage interface" → should be "CacheWriter, CacheReader, VaultReader, VaultWriter ports"
- **Action:** Update with correct port names

**epic-\*.md (5 files):**

- All reference incomplete/incorrect architecture
- Missing architecture version references
- Missing explicit component coverage
- **Action:** Rigorous update with architecture v0.6.8 + coverage verification

**technical-assumptions.md:**

- ✅ ALIGNED - No conflicts found

### Architecture Documents (docs/architecture/)

**Updated to v0.6.8 (7 files):**

- ✅ high-level-architecture.md (v0.5.0)
- ✅ data-models.md (v0.5.1-v0.5.8, v0.6.0)
- ✅ components.md (v0.5.11, v0.6.1-v0.6.8)
- ✅ error-handling-strategy.md (v0.5.9)
- ✅ coding-standards.md (v0.6.7)
- ✅ tech-stack.md (v0.5.10)
- ✅ change-log.md (v0.6.8)

**Need Updates (2 files):**

**source-tree.md:**

- Line 9: References "File" in domain → should list "NoteID, Frontmatter, Note, Schema, Property, Template, Config"
- Line 9: Missing "FileMetadata, VaultFile (SPI adapter models)"
- Line 18: References "FileSystemPort" → should be "CacheWriter, CacheReader, VaultReader, VaultWriter, etc."
- **Action:** Update model lists and port lists

**testing-strategy.md:**

- Line 18: "MockFileSystemPort" → should be "MockVaultReader, MockVaultWriter"
- Line 28: "Assert Result[T] states" → should be "Assert (T, error) return values"
- Line 68: "MockFileSystemPort" again
- **Action:** Update mock names and error handling references

**Need Verification (6 files):**

- core-workflows.md - Spot check for old patterns
- database-schema.md - Likely N/A for MVP
- external-apis.md - Likely N/A for MVP
- rest-api-spec.md - Likely N/A for MVP
- security.md - May reference old models
- infrastructure-and-deployment.md - Likely still valid

### Story Files (docs/stories/)

**Status:** ALL ~30+ story files need to be archived and regenerated

**Why ALL stories (not selective):**

- Epic 1-2 stories reference old architecture (File model, Result[T], FileSystemPort)
- Epic 3 stories completely outdated
- Epic 5-5 stories likely have conflicts
- Inconsistent format/detail across epics
- Easier to regenerate consistently than update piecemeal
- Archive preserves Dev Notes, QA Results for reference

**Estimated Count:**

- Epic 1: ~13 stories
- Epic 2: ~7-8 stories
- Epic 3: ~13 stories (9 + 4 new)
- Epic 5: ~13 stories
- Epic 4: ~8 stories
- **Total: ~54-60 stories**

### Implementation Code (internal/)

**Violated Code (Archive + Re-implement):**

```
internal/domain/file.go                    # File model - domain layer violation
internal/domain/note.go                    # Embeds File, should use NoteID
internal/domain/template.go                # Uses FilePath, should use TemplateID
internal/shared/errors/result.go           # Result[T] pattern - non-idiomatic Go
internal/ports/spi/filesystem.go           # FileSystemPort - YAGNI violation
internal/adapters/spi/filesystem/          # If exists - YAGNI
```

**Verified Correct Code (Keep):**

```
✅ internal/domain/frontmatter.go          # Anemic model, correct
✅ internal/domain/schema.go               # Verify rich model with Validate()
✅ internal/domain/property.go             # Verify rich model with Validate()
✅ internal/shared/logger/                 # Correct
✅ internal/shared/registry/               # Correct
```

**Needs Verification:**

```
⚠️ internal/adapters/spi/config/           # Should be in internal/domain/
⚠️ internal/adapters/spi/schema/           # Verify implementation
⚠️ internal/app/schema/                    # Verify SchemaValidator/Resolver exist
```

### Build Scripts

**justfile:**

- ✅ CLEAN - No conflicts found

### Summary Table

| Artifact Category           | Files            | Severity        | Action                                  |
| --------------------------- | ---------------- | --------------- | --------------------------------------- |
| PRD Epics                   | 5 files          | 🔴 CRITICAL     | Rigorous update + coverage verification |
| PRD Epic List               | 1 file           | 🟡 MINOR        | Update Storage reference                |
| Architecture (Updated)      | 7 files          | ✅ CURRENT      | No action                               |
| Architecture (Needs Update) | 2 files          | 🟡 MEDIUM       | Update source-tree, testing-strategy    |
| Architecture (Verify)       | 6 files          | ⚠️ UNKNOWN      | Spot checks recommended                 |
| **Story Files**             | **~54-60 files** | **🔴 CRITICAL** | **Archive ALL + regenerate**            |
| Implementation Code         | 7+ files         | 🔴 CRITICAL     | Archive violations + re-implement       |
| Implementation Tests        | ~10 files        | 🔴 HIGH         | Update/rewrite                          |
| Build Scripts               | 1 file           | ✅ CLEAN        | No action                               |

---

## SECTION 4: PATH FORWARD EVALUATION

### Options Considered

**Option 1: Incremental Refactoring**

- Refactor Epic 1-2 violations in place
- Update Epic 3 stories selectively
- **Rejected:** 14-21 days, higher complexity for AI agent, higher bug risk

**Option 2: MVP Re-scoping**

- Cut features to avoid refactoring
- **Rejected:** Violations block Epic 3 implementation anyway

**Option 3: Archive + Complete Regeneration (RECOMMENDED)**

- Archive ALL stories and violated code
- Rigorous epic-architecture alignment with coverage verification
- Complete story regeneration
- Fast AI re-implementation
- **Selected:** 17-25 days, perfect alignment, deterministic implementation

### Recommended Path Rationale

**Why Archive + Complete Regeneration:**

1. **AI-Agent Workflow Economics:**
   - Traditional: Planning fast, implementation slow → iterate during implementation
   - Your workflow: Planning slow/critical, implementation fast → maximize planning rigor
   - Investment in rigorous planning → deterministic AI execution

2. **Archive ALL Stories (Not Selective):**
   - Epic 1-2 stories reference old architecture too
   - Inconsistent format/detail across epics
   - Complete regeneration ensures consistency
   - Archive preserves Dev Notes, QA Results

3. **Architecture Coverage Verification:**
   - Prevent "forgotten components" problem
   - Systematic checklist of ALL architecture elements
   - Verify EVERY component has epic + story assignment
   - Critical for completeness

4. **Complete Story Regeneration:**
   - ~54-60 stories with perfect alignment
   - Consistent format and detail level
   - Zero ambiguity for AI agent
   - Every story references architecture docs with versions

5. **Timeline:**
   - 17-25 days is realistic for complete work
   - Faster than refactoring (14-21 days) once all factors considered
   - Much higher quality outcome

---

## SECTION 5: SPRINT CHANGE PROPOSAL - DETAILED PLAN

### Phase 1: Archive ALL (0.5 days)

**Objective:** Preserve existing work while creating clean baseline

**Actions:**

1. **Create Archive Branches:**

```bash
# Archive violated code
git checkout -b archive/epic-1-2-violations
git push origin archive/epic-1-2-violations

# Archive ALL story files
git checkout -b archive/stories-pre-epic-alignment
git push origin archive/stories-pre-epic-alignment
```

2. **Remove from Main Branch:**

```bash
git checkout main
git checkout -b refactor/complete-architecture-alignment

# Remove violated implementations
git rm internal/domain/file.go
git rm internal/domain/note.go
git rm internal/domain/template.go
git rm internal/shared/errors/result.go
git rm internal/ports/spi/filesystem.go
git rm -r internal/adapters/spi/filesystem/  # if exists

# Remove ALL story files (will regenerate)
git rm docs/stories/*.md

git commit -m "refactor: archive violations and all stories for complete regeneration

BREAKING CHANGE: Removes violated domain models (File, Note, Template),
Result[T] pattern, and FileSystemPort to enable clean re-implementation
with architecture v0.6.8 compliance.

Archives ALL story files for consistent regeneration from updated epics.

See docs/course_correction/sprint-change-proposal-2025-10-27-complete-architecture-alignment.md"
```

3. **Document Lessons Learned:**

Create `docs/lessons-learned-epic-1-2.md`:

```markdown
# Lessons Learned: Epic 1-2 Implementation

## What Worked Well

- Test patterns and TDD approach
- Logger and Registry implementations
- Schema system structure (if rich models verified)

## What to Preserve

- Test file structure and organization
- Table-driven test patterns
- Quality gate approach (golangci-lint, coverage)

## What to Avoid

- Implementing before architecture finalized
- Layer violations (File in domain)
- Non-idiomatic patterns (Result[T])
- Missing abstractions (NoteID, VaultPorts)

## Archive Branches

- code: archive/epic-1-2-violations
- stories: archive/stories-pre-epic-alignment
```

**Deliverables:**

- ✅ Two archive branches with all work preserved
- ✅ Clean baseline branch
- ✅ Lessons learned document
- ✅ Commit documenting BREAKING CHANGE

---

### Phase 2: Epic-Level Rigorous Alignment (5-7 days)

#### Phase 2.0: Architecture Coverage Verification (0.5 days) ⭐ CRITICAL

**Objective:** Create systematic inventory ensuring EVERY architecture component is represented

**Create:** `docs/prd/architecture-coverage-checklist.md`

```markdown
# Architecture Coverage Checklist

**Purpose:** Ensure ALL components from docs/architecture/ (v0.6.8) are mapped to epic/story planning.

**Version:** Based on architecture v0.6.8 (Oct 26, 2025)

---

## Domain Services (from components.md v0.6.8)

- [ ] **TemplateEngine** → Epic: **_ Story: _**
  - Reference: components.md#templateengine
  - Render templates with custom function map

- [ ] **FrontmatterService** → Epic: **_ Story: _**
  - Reference: components.md#frontmatterservice (v0.5.9)
  - Extract() and Validate() methods

- [ ] **SchemaEngine** → Epic: **_ Story: _**
  - Reference: components.md#schemaengine
  - Schema loading and management

- [ ] **SchemaValidator** → Epic: **_ Story: _**
  - Reference: components.md#schemavalidator (v0.6.1)
  - Orchestrates schema validation

- [ ] **SchemaResolver** → Epic: **_ Story: _**
  - Reference: components.md#schemaresolver (v0.6.1)
  - Inheritance and $ref resolution

- [ ] **VaultIndexer** → Epic: **_ Story: _**
  - Reference: components.md#vaultindexer (v0.6.8)
  - Focused service (not God Service)

- [ ] **QueryService** → Epic: **_ Story: _**
  - Reference: components.md#queryservice
  - Read-side query operations

- [ ] **CommandOrchestrator** → Epic: **_ Story: _**
  - Reference: components.md#commandorchestrator (v0.6.4)
  - Use case orchestrator with callback pattern

---

## SPI Ports (from components.md v0.6.8)

- [ ] **CacheWriterPort** → Epic: **_ Story: _**
  - Reference: components.md#cachewriterport (v0.5.11)
  - Persist(Note) error, Delete(NoteID) error

- [ ] **CacheReaderPort** → Epic: **_ Story: _**
  - Reference: components.md#cachereaderport (v0.5.11)
  - Get, List, Filter methods

- [ ] **VaultReaderPort** → Epic: **_ Story: _**
  - Reference: components.md#vaultreaderport (v0.6.8)
  - ScanAll, ScanModified, Read methods

- [ ] **VaultWriterPort** → Epic: **_ Story: _**
  - Reference: components.md#vaultwriterport (v0.6.8)
  - Persist, Delete methods

- [ ] **SchemaPort** → Epic: **_ Story: _**
- [ ] **TemplatePort** → Epic: **_ Story: _**
- [ ] **PromptPort** → Epic: **_ Story: _**
- [ ] **FinderPort** → Epic: **_ Story: _**
- [ ] **ConfigPort** → Epic: **_ Story: _**
- [ ] **SchemaRegistryPort** → Epic: **_ Story: _**
- [ ] **CLIPort** → Epic: **_ Story: _**
  - Reference: components.md#cliport (v0.6.4)
  - Start(ctx, handler) with CommandPort callback

---

## SPI Adapters (from components.md v0.6.8)

- [ ] **JSONCacheWriteAdapter** → Epic: **_ Story: _**
- [ ] **JSONCacheReadAdapter** → Epic: **_ Story: _**
- [ ] **VaultReaderAdapter** → Epic: **_ Story: _**
- [ ] **VaultWriterAdapter** → Epic: **_ Story: _**
- [ ] **SchemaLoaderAdapter** → Epic: **_ Story: _**
- [ ] **TemplateLoaderAdapter** → Epic: **_ Story: _**
- [ ] **PromptUIAdapter** → Epic: **_ Story: _**
- [ ] **FuzzyFinderAdapter** → Epic: **_ Story: _**
- [ ] **ViperAdapter** → Epic: **_ Story: _**
- [ ] **SchemaRegistryAdapter** → Epic: **_ Story: _**

---

## API Ports (from components.md v0.6.8)

- [ ] **CLIPort** → Epic: **_ Story: _**

---

## API Adapters (from components.md v0.6.8)

- [ ] **CobraCLIAdapter** → Epic: **_ Story: _**

---

## Domain Models (from data-models.md v0.6.8)

- [ ] **NoteID** → Epic: **_ Story: _**
  - Reference: data-models.md#noteid (v0.5.2)
  - Opaque domain identifier

- [ ] **Frontmatter** → Epic: **_ Story: _**
  - Reference: data-models.md#frontmatter
  - Anemic model: FileClass + Fields

- [ ] **Note** → Epic: **_ Story: _**
  - Reference: data-models.md#note (v0.5.2)
  - Composition: NoteID + Frontmatter

- [ ] **Schema** → Epic: **_ Story: _**
  - Reference: data-models.md#schema (v0.6.0)
  - Rich model with Validate() method

- [ ] **Property** → Epic: **_ Story: _**
  - Reference: data-models.md#property (v0.6.0)
  - Rich model with Validate() method

- [ ] **PropertySpec** → Epic: **_ Story: _**
  - Reference: data-models.md#propertyspec (v0.6.0)
  - Interface + 5 variants (String, Number, Date, File, Bool)
  - Rich models with Validate() methods

- [ ] **PropertyBank** → Epic: **_ Story: _**
  - Reference: data-models.md#propertybank

- [ ] **TemplateID** → Epic: **_ Story: _**
  - Reference: data-models.md#templateid (v0.5.6)
  - Template name identifier

- [ ] **Template** → Epic: **_ Story: _**
  - Reference: data-models.md#template (v0.5.6)
  - ID + Content (no FilePath)

- [ ] **Config** → Epic: **_ Story: _**
  - Reference: data-models.md#config (v0.5.7)
  - Domain value object

---

## SPI Adapter Models (from data-models.md v0.6.8)

- [ ] **FileMetadata** → Epic: **_ Story: _**
  - Reference: data-models.md#filemetadata (v0.5.1)
  - SPI adapter model, not domain

- [ ] **VaultFile** → Epic: **_ Story: _**
  - Reference: data-models.md#vaultfile (v0.6.8)
  - DTO: embeds FileMetadata + Content

---

## Verification Checklist

- [ ] Every architecture component has epic assignment
- [ ] Every architecture component has story assignment (specific story number)
- [ ] No components marked "???" or blank
- [ ] Dependencies properly sequenced (Epic 1 → 2 → 3 → 4 → 5)
- [ ] No duplicate coverage (same component in multiple stories)
- [ ] All architecture version references documented
```

**Process:**

1. Work through checklist systematically
2. For each component, identify which epic/story should implement it
3. If component has no epic/story → flag for addition
4. Document all findings

**Deliverable:**

- ✅ architecture-coverage-checklist.md 100% filled out
- ✅ List of missing components/stories to add
- ✅ Complete inventory of architecture → epic mapping

---

#### Phase 2.1: Update Epic List (0.5 days)

**File:** `docs/prd/epic-list.md`

**Change:**

```diff
- This includes defining the core `Storage` interface.
+ This includes defining the cache and vault port interfaces (CacheWriter,
+ CacheReader, VaultReader, VaultWriter) following CQRS pattern per architecture
+ v0.6.8.
```

**Add to header:**

```markdown
**Architecture Version:** Based on docs/architecture/ v0.6.8 (Oct 26, 2025)
```

---

#### Phase 2.2: Update Epic 1 Detail (1.5 days)

**File:** `docs/prd/epic-1-foundational-cli-static_template-engine.md`

**Add to Epic Header:**

```markdown
**Architecture References:**

- Components: docs/architecture/components.md v0.6.4-v0.6.8
- Models: docs/architecture/data-models.md v0.5.1-v0.5.7
- Errors: docs/architecture/error-handling-strategy.md v0.5.9
- Standards: docs/architecture/coding-standards.md v0.6.7
```

**Update Story 1.3: Core Domain Models (COMPLETE REWRITE)**

```markdown
## Story 1.3: Implement Core Domain Models

As a developer, I want to implement the core domain models following hexagonal
architecture principles, so that the domain layer has no infrastructure dependencies.

### Key Requirements

**Domain Models:**

- NoteID - Opaque identifier (Reference: data-models.md#noteid v0.5.2)
- Note - Composition of NoteID + Frontmatter (Reference: data-models.md#note v0.5.2)
- Frontmatter - Anemic model with FileClass + Fields (Reference: data-models.md#frontmatter)
- TemplateID - Template name identifier (Reference: data-models.md#templateid v0.5.6)
- Template - ID + Content, no FilePath (Reference: data-models.md#template v0.5.6)
- Config - Domain value object (Reference: data-models.md#config v0.5.7)

**SPI Adapter Models:**

- FileMetadata - Infrastructure metadata in SPI layer (Reference: data-models.md#filemetadata v0.5.1)

### Acceptance Criteria

- 1.3.1: Create NoteID domain model in `internal/domain/note_id.go`:
  - Type: Opaque string identifier
  - Purpose: Decouple domain from infrastructure
  - No filesystem path knowledge

- 1.3.2: Create FileMetadata SPI adapter model in `internal/adapters/spi/file_dto.go`:
  - Fields: Path, Basename, Folder, Ext, ModTime, Size, MimeType
  - Purpose: Used by adapters to map NoteID ↔ filesystem paths
  - Not accessible to domain layer

- 1.3.3: Create Note domain model in `internal/domain/note.go`:
  - Fields: ID (NoteID), Frontmatter (Frontmatter)
  - Pure data structure, no embedded File
  - Method: SchemaName() string (delegates to Frontmatter)

- 1.3.4: Verify Frontmatter domain model in `internal/domain/frontmatter.go`:
  - Fields: FileClass (string), Fields (map[string]any)
  - Anemic model (pure data)
  - Method: SchemaName() string

- 1.3.5: Create TemplateID domain model in `internal/domain/template_id.go`:
  - Type: String representing template name
  - Purpose: Template identification for Go text/template composition

- 1.3.6: Create Template domain model in `internal/domain/template.go`:
  - Fields: ID (TemplateID), Content (string)
  - No FilePath field
  - Pure data structure

- 1.3.7: Verify Config domain model location:
  - Should be in `internal/domain/config.go` (not adapters/spi/config/)
  - Value object with VaultPath, TemplatesDir, SchemasDir, etc.

### Architecture References

- data-models.md#noteid (v0.5.2)
- data-models.md#filemetadata (v0.5.1)
- data-models.md#note (v0.5.2)
- data-models.md#frontmatter
- data-models.md#templateid (v0.5.6)
- data-models.md#template (v0.5.6)
- data-models.md#config (v0.5.7)
```

**Update Story 1.4: Shared Errors Package (COMPLETE REWRITE)**

```markdown
## Story 1.4: Implement Error Handling with Idiomatic Go

As a developer, I want domain-specific error types using idiomatic Go patterns,
so that error handling is clear and follows Go best practices.

### Key Requirements

**NO Result[T] Pattern** - Use idiomatic `(T, error)` throughout
**Domain Error Types** - Implement standard `error` interface
**Error Wrapping** - Use `fmt.Errorf("context: %w", err)` for stack traces

### Acceptance Criteria

- 1.4.1: Remove Result[T] pattern entirely:
  - Delete `internal/shared/errors/result.go` if exists
  - All function signatures use `(T, error)` return type

- 1.4.2: Create domain error types in `internal/shared/errors/`:
  - FrontmatterError (not ValidationError)
  - SchemaError
  - CacheReadError
  - CacheWriteError
  - NotFoundError
  - Each implements standard `error` interface

- 1.4.3: Implement error wrapping helpers:
  - Use `fmt.Errorf("context: %w", err)` for wrapping
  - Provide Unwrap() methods for error chain inspection
  - Support errors.Is() and errors.As() for type checking

- 1.4.4: Update all services to use `(T, error)`:
  - No Result[T].IsOk(), Result[T].IsErr() patterns
  - Direct error checking: `if err != nil`

### Architecture References

- error-handling-strategy.md (v0.5.9)
- coding-standards.md#error-handling (v0.6.7)
```

**Update Story 1.7: Remove FileSystemPort (COMPLETE REWRITE)**

```markdown
## Story 1.7: Use Go Stdlib Directly (YAGNI Principle)

As a developer, I want adapters to use Go stdlib directly, so that we avoid
unnecessary abstraction with single implementation.

### Key Requirements

**NO FileSystemPort** - YAGNI principle (architecture v0.5.11 decision)
**Use Go Stdlib** - os.ReadFile, filepath.Walk, atomicwriter.WriteFile
**Business-Level Ports** - VaultReader/Writer defined in Epic 3 (v0.6.8)

### Acceptance Criteria

- 1.7.1: Ensure NO FileSystemPort interface exists:
  - No `internal/ports/spi/filesystem.go`
  - No generic filesystem abstraction

- 1.7.2: Adapters use Go stdlib directly:
  - os.ReadFile for file reading
  - filepath.Walk for directory traversal
  - atomicwriter.WriteFile for atomic writes

- 1.7.3: Document decision in code comments:
  - Reference architecture v0.5.11 YAGNI decision
  - Note: VaultReader/Writer provide business-level abstractions (Epic 3)

### Architecture References

- components.md (v0.5.11 YAGNI decision)
- components.md#vault-ports (v0.6.8 business-level abstractions)
```

**Add Missing Stories Based on Architecture Coverage Checklist:**

_After completing checklist in Phase 2.0, add stories for any missing components_

Examples of potentially missing:

- Story 1.X: Implement TemplateEngine Service
- Story 1.X: Implement CommandOrchestrator
- Story 1.X: Implement TemplatePort Interface
- Story 1.X: Implement TemplateLoaderAdapter
- Story 1.X: Implement ConfigPort Interface
- Story 1.X: Update CLIPort for Callback Pattern

---

#### Phase 2.3: Update Epic 2 Detail (1 day)

**File:** `docs/prd/epic-2-configuration-schema-loading.md`

**Add Epic Header with Architecture References**

**Update Story 2.1: Config Model**

- Add note: Config should be in `internal/domain/config.go` (v0.5.7)
- Reference: data-models.md#config

**Verify Stories 2.2-2.3: Schema/Property Models**

- Add rich model requirements (v0.6.0)
- Require Validate() methods on Schema, Property, PropertySpec
- Reference: data-models.md#schema, #property, #propertyspec

**Update Story 2.6: Inheritance Resolution**

- Explicitly require SchemaResolver service (v0.6.1)
- Reference: components.md#schemaresolver

**Update Story 2.7: Schema Validation**

- Explicitly require SchemaValidator service (v0.6.1)
- Separate from model validation
- Orchestrates schema validation and cross-schema checks
- Reference: components.md#schemavalidator

**Add Missing Stories:**

- Story 2.X: SchemaEngine service (if not explicit)
- Any other components from coverage checklist

---

#### Phase 2.4: Update Epic 3 Detail (2-3 days)

**File:** `docs/prd/epic-3-vault-indexing-engine.md`

**Complete Epic Description Rewrite:**

```markdown
# Epic 3: Vault Indexing Engine

This epic builds the core vault indexing system. It scans the vault via
VaultReaderPort, parses frontmatter via FrontmatterService, validates against
schemas, and persists to cache via CacheWriterPort. QueryService provides
read-side access to indexed data. Implements CQRS pattern at both vault level
(VaultReader/Writer) and cache level (CacheWriter/Reader).

**Dependencies:** Epic 2 (Configuration & Schema Loading)

**Architecture References:**

- Ports: components.md#vault-ports (v0.6.8), #cache-ports (v0.5.11)
- Services: components.md#vaultindexer (v0.6.8), #frontmatterservice (v0.5.9), #queryservice
- Models: data-models.md#noteid, #filemetadata, #vaultfile (v0.6.8)
- Patterns: high-level-architecture.md#cqrs (v0.5.0)
```

**Rewrite ALL 9 Stories + Add 4 New (13 total):**

**Story 3.1: Implement Cache Port Interfaces** (REWRITE)

```markdown
## Story 3.1: Implement Cache Port Interfaces

As a developer, I want to define CacheWriter and CacheReader port interfaces,
so that cache operations follow CQRS principles with proper naming.

### Acceptance Criteria

- 3.1.1: Create `internal/ports/spi/cache.go` with CacheWriterPort:
  - Persist(ctx context.Context, note Note) error
  - Delete(ctx context.Context, id NoteID) error
  - Uses Note domain model (NoteID + Frontmatter)
  - Returns (T, error) NOT Result[T]

- 3.1.2: Define CacheReaderPort in same file:
  - Get(ctx context.Context, id NoteID) (Note, error)
  - List(ctx context.Context) ([]Note, error)
  - Filter(ctx context.Context, query Query) ([]Note, error)

- 3.1.3: Port naming uses "Writer/Reader" (not "Command/Query")
- 3.1.4: Method naming uses "Persist" (not "Write") for consistency

### Architecture References

- components.md#cachewriterport (v0.5.11)
- components.md#cachereaderport (v0.5.11)
- data-models.md#note (v0.5.2)
```

**Story 3.2: Implement JSON Cache Adapters (CQRS Split)** (REWRITE)

**Story 3.3: Implement VaultReaderPort** (NEW)

```markdown
## Story 3.3: Implement VaultReaderPort Interface

As a developer, I want VaultReaderPort with business-level vault scanning
operations, so that indexing uses proper abstraction (not generic FileSystemPort).

### Acceptance Criteria

- 3.3.1: Create `internal/ports/spi/vault.go` with VaultReaderPort:
  - ScanAll(ctx context.Context) ([]VaultFile, error)
  - ScanModified(ctx context.Context, since time.Time) ([]VaultFile, error)
  - Read(ctx context.Context, path string) ([]byte, error)

- 3.3.2: Define VaultFile DTO in same file (SPI adapter layer):
  - Embeds FileMetadata (Path, Basename, Folder, Ext, ModTime, Size, MimeType)
  - Adds Content []byte
  - NOT a domain model - infrastructure DTO only

### Architecture References

- components.md#vaultreaderport (v0.6.8)
- data-models.md#vaultfile (v0.6.8)
- data-models.md#filemetadata (v0.5.1)
```

**Story 3.4: Implement VaultReaderAdapter** (NEW)

**Story 3.5: Implement FrontmatterService** (UPDATE - consolidate Extract + Validate)

**Story 3.6: Implement VaultIndexer Service** (UPDATE - focused, not God Service)

**Story 3.7: Implement QueryService** (UPDATE)

**Story 3.8: Implement Index Command** (REWRITE - CommandOrchestrator callback pattern v0.6.4)

**Story 3.9: Implement Cache Versioning** (MINOR UPDATE)

**Story 3.X: Implement VaultWriterPort** (NEW)

**Story 3.X: Implement VaultWriterAdapter** (NEW)

---

#### Phase 2.5: Update Epic 5-5 Details (1 day)

**Review all stories for:**

- Template model changes (FilePath → TemplateID)
- CommandOrchestrator redesign (v0.6.4)
- Port/service naming updates
- Architecture references

**Add stories for any missing components from coverage checklist**

---

#### Phase 2.6: Final Architecture Coverage Audit (0.5 days)

**Process:**

1. Review completed `docs/prd/architecture-coverage-checklist.md`
2. Verify EVERY component has epic + story assignment
3. Verify NO components marked "???" or blank
4. Verify dependencies properly sequenced
5. Verify no duplicate coverage

**If ANY component unchecked → GO BACK and add to appropriate epic**

**Deliverables:**

- ✅ All 5 epic files rigorously updated
- ✅ architecture-coverage-checklist.md 100% complete
- ✅ Every architecture component mapped to story
- ✅ All architecture references documented with versions
- ✅ Zero gaps, zero ambiguity

---

### Phase 3: Documentation Cleanup (0.5 days)

**Update source-tree.md:**

```diff
- ├── domain/              # Core models (File, Frontmatter, Note, Schema, Property)
+ ├── domain/              # Core models (NoteID, Frontmatter, Note, Schema, Property, Template, Config)
+ #                        # SPI adapter models: FileMetadata, VaultFile (in adapters/spi/)

- │   └── spi/             # FileSystemPort, Cache ports, SchemaLoaderPort, etc.
+ │   └── spi/             # Cache ports (CacheWriter, CacheReader)
+ #                        # Vault ports (VaultReader, VaultWriter)
+ #                        # Schema ports, Template ports, Interactive ports, Config ports
```

**Update testing-strategy.md:**

```diff
- Mock Implementations: MockFileSystemPort
+ Mock Implementations: MockVaultReader, MockVaultWriter, MockCacheWriter, MockCacheReader

- Assert both `Result[T]` states and domain side effects
+ Assert return values and errors using idiomatic (T, error) pattern
```

**Spot check remaining docs:**

- core-workflows.md
- security.md
- Others as needed

---

### Phase 4: Complete Story Regeneration (3-5 days)

**Objective:** Generate ALL ~54-60 story files from scratch with perfect quality

#### Story Generation Standards

**EVERY story must include:**

1. **Clear Story Statement**
   - User/developer perspective
   - Business value

2. **Detailed Acceptance Criteria**
   - Specific file paths
   - Interface/method signatures
   - Architecture doc references with section + version

3. **TDD Task Breakdown**
   - Red phase: Write failing tests
   - Green phase: Implement to pass tests
   - Refactor phase: Improve code quality
   - Quality gates: lint, test, coverage

4. **Dev Notes Section**
   - Previous story insights
   - Data models with architecture references
   - API specifications
   - Component specifications
   - File locations
   - Testing requirements
   - Technical constraints
   - Edge cases

5. **Architecture References**
   - Specific sections (e.g., data-models.md#noteid)
   - Version numbers (e.g., v0.5.2)

6. **Zero Ambiguity**
   - AI agent can implement without clarification questions
   - All decisions pre-made
   - All dependencies specified

#### Story Generation Process

**Epic 1: ~13 Stories**

Using updated `epic-1-foundational-cli-static_template-engine.md`:

- Regenerate stories 1.1-1.13
- Add new stories for missing components (from coverage checklist)
- Examples:
  - 1.3: Domain Models (NoteID, FileMetadata, Note, TemplateID, Template)
  - 1.4: Error Handling ((T, error), domain errors)
  - 1.7: Remove FileSystemPort (YAGNI)
  - 1.X: TemplateEngine Service (NEW if missing)
  - 1.X: CommandOrchestrator (NEW if missing)
  - 1.X: TemplatePort Interface (NEW if missing)
  - 1.X: TemplateLoaderAdapter (NEW if missing)

**Epic 2: ~7-8 Stories**

Using updated `epic-2-configuration-schema-loading.md`:

- Regenerate all stories
- Add explicit stories for SchemaEngine, SchemaValidator, SchemaResolver if needed
- Verify rich model requirements in stories 2.2-2.3

**Epic 3: ~13 Stories**

Using updated `epic-3-vault-indexing-engine.md`:

- Generate ALL NEW stories (9 rewrites + 4 new)
- Perfect CQRS alignment
- All new port names (CacheWriter/Reader, VaultReader/Writer)
- All new models (NoteID, VaultFile, FileMetadata)

**Epic 5: ~13 Stories**

- Regenerate all stories
- Update for Template changes
- Update for CommandOrchestrator changes

**Epic 4: ~8 Stories**

- Regenerate all stories
- Update for Epic 2-4 changes

#### Quality Verification

**For each generated story:**

- [ ] Story statement clear and valuable
- [ ] Acceptance criteria detailed and testable
- [ ] Architecture references with versions
- [ ] TDD task breakdown complete
- [ ] Dev Notes comprehensive
- [ ] File locations specified
- [ ] Testing requirements defined
- [ ] Zero ambiguity - AI executable

**Cross-story verification:**

- [ ] architecture-coverage-checklist.md matches stories
- [ ] No duplicate component coverage
- [ ] No missing component coverage
- [ ] Story sequence matches dependencies

**Deliverables:**

- ✅ ~54-60 rigorous story files in docs/stories/
- ✅ Perfect architecture v0.6.8 alignment
- ✅ 100% architecture component coverage verified
- ✅ Consistent format and quality
- ✅ Ready for deterministic AI implementation

---

### Phase 5: Complete Re-Implementation (8-12 days)

**Objective:** Clean re-implementation of Epic 1-2 with perfect architecture alignment

#### Sub-Phase 5.1: Epic 1 Re-Implementation (4-5 days)

**Implement ALL ~13 Epic 1 stories:**

**Day 1-2: Core Models**

- Story 1.3: NoteID, FileMetadata, Note, TemplateID, Template, Config
- Comprehensive unit tests
- Quality gates (lint, test, coverage)

**Day 3: Error Handling**

- Story 1.4: Domain error types, (T, error) pattern
- Remove any Result[T] remnants
- Update all error handling

**Day 4: Services and Ports**

- Stories for TemplateEngine, CommandOrchestrator
- Port interfaces (TemplatePort, ConfigPort, CLIPort)
- Adapter implementations

**Day 5: Integration and Quality**

- Integration testing
- Quality gates
- Documentation

#### Sub-Phase 5.2: Epic 2 Verification + Gaps (2-3 days)

**Verify existing Epic 2 code:**

- Check Schema/Property Validate() methods exist
- Check SchemaValidator service exists
- Check SchemaResolver service exists
- Move Config to internal/domain/

**Implement missing components:**

- Any gaps identified in coverage checklist
- Update tests
- Quality gates

#### Sub-Phase 5.3: Quality Gates (1-2 days)

**Run comprehensive quality checks:**

- golangci-lint run: 0 warnings required
- go test ./...: 100% pass required
- Test coverage: ≥95% internal/app, ≥85% overall
- Manual smoke testing
- Architecture alignment verification using coverage checklist

#### Sub-Phase 5.4: Documentation (1 day)

**Update documentation:**

- Implementation notes
- Any architecture insights
- Update lessons-learned if needed

**Deliverables:**

- ✅ Epic 1-2 completely re-implemented
- ✅ Zero architectural violations
- ✅ All quality gates passing
- ✅ architecture-coverage-checklist.md verified against implementation
- ✅ Ready for Epic 3 implementation

---

## TIMELINE & RESOURCE ALLOCATION

| Phase                      | Duration       | Owner    | Effort   | Key Deliverable                          |
| -------------------------- | -------------- | -------- | -------- | ---------------------------------------- |
| **1. Archive ALL**         | 0.5 days       | You (PO) | Low      | Archive branches, lessons learned        |
| **2.0 Coverage Checklist** | 0.5 days       | You (PO) | **High** | **Architecture coverage 100% mapped**    |
| **2.1-2.6 Epic Alignment** | 5-7 days       | You (PO) | **High** | **Epics cover 100% of architecture**     |
| **3. Doc Cleanup**         | 0.5 days       | You (PO) | Low      | Updated source-tree, testing-strategy    |
| **4. Story Regeneration**  | 3-5 days       | You (PO) | **High** | **~54-60 rigorous stories**              |
| **5. Re-Implementation**   | 8-12 days      | AI Agent | Medium   | **Clean code, all violations fixed**     |
| **TOTAL**                  | **17-25 days** |          |          | **Epic 3 ready with Epic 1-2 perfected** |

**Critical Path:** Coverage checklist → Epic alignment → Story generation → Re-implementation

**Parallelization:**

- Phase 2 (epic updates) can partially overlap across epics
- Phase 3 (doc cleanup) can overlap with Phase 2
- Multiple stories can be implemented in parallel in Phase 5 (if multiple AI agents)

**PO Time Investment:** ~9-13 days (Phases 1-4)
**AI Agent Time:** ~8-12 days (Phase 5)

---

## RISKS & MITIGATION

| Risk                                         | Probability | Impact | Mitigation                                                                 |
| -------------------------------------------- | ----------- | ------ | -------------------------------------------------------------------------- |
| **Coverage checklist reveals major gaps**    | Medium      | High   | Time-boxed to 2 weeks max for epic alignment; prioritize Epic 1-3          |
| **Epic alignment takes longer than 7 days**  | Medium      | Medium | Can compress Phase 4 (story generation) slightly if needed                 |
| **Story generation reveals conflicts**       | Low         | Medium | Epic alignment phase should catch all major issues                         |
| **Re-implementation hits unexpected issues** | Low         | Medium | Stories detailed enough to minimize ambiguity; can reference archived code |
| **Work feels "thrown away" (morale)**        | Low         | Low    | Frame as "leveling up"; archive preserves all work; lessons learned doc    |
| **Timeline estimate too optimistic**         | Low         | Medium | Built in buffer (17-25 days); can extend to 30 days max if needed          |

---

## SUCCESS CRITERIA

### Phase 2.0 Complete: ✅

- [ ] architecture-coverage-checklist.md created
- [ ] ALL architecture components inventoried (services, ports, adapters, models)
- [ ] EVERY component assigned to epic + story
- [ ] ZERO components unchecked or marked "???"

### Phase 2.6 Complete: ✅

- [ ] All 5 epic files updated with architecture v0.6.8 references
- [ ] architecture-coverage-checklist.md shows 100% coverage
- [ ] Every story description has architecture section references
- [ ] No orphaned components
- [ ] Dependencies properly sequenced

### Phase 4 Complete: ✅

- [ ] ~54-60 story files generated (ALL epics)
- [ ] Every story has detailed acceptance criteria
- [ ] Every story references architecture docs with versions
- [ ] architecture-coverage-checklist.md confirms all components have stories
- [ ] Consistent story format and quality
- [ ] AI agent can implement with zero ambiguity

### Phase 5 Complete: ✅

- [ ] Epic 1-2 completely re-implemented
- [ ] golangci-lint: 0 warnings
- [ ] go test ./...: 100% pass
- [ ] Test coverage: ≥95% internal/app, ≥85% overall
- [ ] No Result[T] pattern anywhere
- [ ] No File model in domain layer
- [ ] No FileSystemPort exists
- [ ] Config in internal/domain/
- [ ] architecture-coverage-checklist.md verified against implementation
- [ ] Epic 3 can proceed immediately with confidence

---

## PRD MVP IMPACT

### MVP Scope: UNCHANGED ✅

**All 5 epics remain in scope:**

- Epic 1: Foundational CLI ✅
- Epic 2: Configuration & Schema Loading ✅
- Epic 3: Vault Indexing Engine ✅
- Epic 5: Interactive Input Engine ✅
- Epic 4: Schema-Driven Lookups & Validation ✅

**No features cut** - Only implementation quality improvement

### MVP Timeline Impact

**Delay:** +17-25 days before Epic 3 can start
**Impact:** ~3-5 weeks delay to MVP delivery

**Rationale:**

- Fixing violations now prevents technical debt compounding
- Investment in rigorous planning pays dividends in fast implementation
- Ensures NO missing components (coverage checklist)
- Perfect foundation for Epic 3-5

### MVP Value Proposition: ENHANCED ✅

**Same features, BETTER implementation:**

- ✅ Clean hexagonal architecture (enables future TUI/LSP)
- ✅ Proper CQRS pattern (enables performance optimizations)
- ✅ Idiomatic Go (easier for contributors)
- ✅ Zero technical debt from Epic 1-2
- ✅ 100% architecture coverage (no forgotten components)
- ✅ Perfect foundation for Epic 3-5

---

## AGENT HANDOFF PLAN

### Immediate Next Steps (If Approved)

**Day 1: Archive Phase**

- **PO (You)** creates archive branches
- **PO** removes violated code
- **PO** documents lessons learned

**Week 1: Architecture Coverage**

- **PO** creates architecture-coverage-checklist.md
- **PO** systematically fills out checklist
- **Architect (Winston)** available for questions
- **Deliverable:** 100% coverage checklist

**Week 2-3: Epic Alignment**

- **PO** updates all 5 epic files
- **PO** adds missing stories for uncovered components
- **PO** verifies 100% coverage
- **Deliverable:** Perfect epic-architecture alignment

**Week 3: Documentation + Story Prep**

- **PO** updates source-tree.md, testing-strategy.md
- **PO** prepares for story generation
- **Deliverable:** Clean documentation

**Week 4-5: Story Generation**

- **SM** generates ~54-60 rigorous stories (`.bmad-core/tasks/create-next-story.md`)
- **QA** ensures all stories implement TDD framework and predefine well-designed tests, if applicable (`.bmad-core/tasks/test-design.md`)
- **SM** quality checks all stories (`.bmad-core/tasks/execute-checklist.md`)
- **PO** reviews and validates each story clarity (`.bmad-core/tasks/validate-next-story.md`)
- **Deliverable:** Perfect, consistent stories

**Week 5-7: Re-Implementation**

- **Dev Agent** implements stories
- **QA** validates quality gates
- **PO** performs acceptance
- **Deliverable:** Clean code, Epic 3 ready

### Roles Required

| Role                              | Responsibilities                             | Time Commitment         |
| --------------------------------- | -------------------------------------------- | ----------------------- |
| **Product Owner (You/Sarah)**     | Epic alignment, story generation, acceptance | ~9-13 days focused work |
| **Architect (Winston/Reference)** | Architecture clarification questions         | Ad-hoc availability     |
| **Dev Agent (AI)**                | Re-implementation execution                  | ~8-12 days execution    |
| **QA Agent**                      | Quality gate validation                      | Ad-hoc verification     |

---

## APPROVAL REQUEST

### Decision Points

Please confirm approval for:

1. **✅ Archive ALL approach?**
   - Archive all violated code
   - Archive ALL story files (not selective)
   - Alternative: Incremental refactoring (rejected - slower, riskier)

2. **✅ Architecture coverage verification?**
   - Create systematic checklist
   - Verify 100% component coverage
   - Add missing stories for uncovered components
   - Alternative: Skip verification (rejected - risk of forgotten components)

3. **✅ Complete story regeneration?**
   - Regenerate ALL ~54-60 stories
   - Perfect consistency and quality
   - Alternative: Selective updates (rejected - inconsistent quality)

4. **✅ Timeline: 17-25 days?**
   - Realistic for complete work
   - Can extend to 30 days if needed
   - Alternative: Rush (rejected - quality risk)

5. **✅ Your time commitment: ~9-13 days?**
   - Focused planning work
   - Highest value activity for quality
   - Alternative: Less rigorous planning (rejected - defeats AI-agent workflow advantage)

### Constraints or Concerns?

Please share any:

- Budget constraints
- Timeline pressures
- Team availability issues
- Other concerns

---

## FINAL RECOMMENDATION

**🎯 APPROVE: Archive ALL → Architecture Coverage Verification → Complete Story Regeneration**

**Why This is the RIGHT Path:**

1. **Matches Your Workflow** ✅
   - AI-agent development: Planning slow/critical, implementation fast
   - Rigorous planning → deterministic execution
   - Investment in quality saves time overall

2. **Ensures Completeness** ✅
   - Architecture coverage checklist prevents forgotten components
   - Systematic verification catches all gaps
   - 100% coverage guarantee

3. **Maximizes Quality** ✅
   - Perfect architecture alignment
   - Consistent story format
   - Zero technical debt
   - Best long-term foundation

4. **Realistic Timeline** ✅
   - 17-25 days is honest estimate
   - Accounts for complete regeneration
   - Buffer built in

5. **Best ROI** ✅
   - 3-5 weeks now saves months of refactoring later
   - Perfect foundation for Epic 3-5
   - Enables fast Epic 3+ implementation

**Investment:** ~3-5 weeks of rigorous planning and clean re-implementation
**Return:** Zero technical debt + 100% architecture coverage + perfect Epic 3+ foundation

---

## APPENDICES

### Appendix A: Key Architectural Violations Detail

**1. File Model in Domain (CRITICAL)**

- **Location:** `internal/domain/file.go`
- **Violation:** Domain contains filesystem paths (Path, Basename, Folder, ModTime)
- **Correct:** NoteID (domain) + FileMetadata (SPI adapter)
- **Reference:** architecture-review-and-refactoring-plan.md Section "Layer Violation: File Model"

**2. Note Model Embeds File (CRITICAL)**

- **Location:** `internal/domain/note.go`
- **Violation:** Embeds File (infrastructure)
- **Correct:** Note{ID NoteID, Frontmatter Frontmatter}
- **Reference:** data-models.md#note (v0.5.2)

**3. Template Uses FilePath (CRITICAL)**

- **Location:** `internal/domain/template.go`
- **Violation:** Uses FilePath string
- **Correct:** Template{ID TemplateID, Content string}
- **Reference:** data-models.md#template (v0.5.6)

**4. Result[T] Pattern (CRITICAL)**

- **Location:** `internal/shared/errors/result.go`
- **Violation:** Non-idiomatic Go pattern
- **Correct:** Use (T, error) throughout
- **Reference:** coding-standards.md (v0.6.7)

**5. FileSystemPort Exists (MEDIUM)**

- **Location:** `internal/ports/spi/filesystem.go`
- **Violation:** YAGNI - single implementation
- **Correct:** Remove, use Go stdlib directly
- **Reference:** components.md (v0.5.11 YAGNI decision)

**6. Config in Adapter Layer (MEDIUM)**

- **Location:** `internal/adapters/spi/config/config.go`
- **Violation:** Should be domain value object
- **Correct:** Move to `internal/domain/config.go`
- **Reference:** data-models.md#config (v0.5.7)

### Appendix B: Architecture Version Timeline

| Version       | Date           | Key Changes                                                        |
| ------------- | -------------- | ------------------------------------------------------------------ |
| v0.1.0-0.4.3  | Oct 8-11, 2025 | Initial architecture (had violations)                              |
| v0.5.0        | Oct 24, 2025   | Clean hexagonal design principles                                  |
| v0.5.1        | Oct 24, 2025   | FileMetadata, CQRS adapter references                              |
| v0.5.2        | Oct 24, 2025   | NoteID, Note composition                                           |
| v0.5.3-v0.5.8 | Oct 24, 2025   | Schema/Property/Template/Config corrections                        |
| v0.5.9        | Oct 24, 2025   | Error handling: removed Result[T]                                  |
| v0.5.10       | Oct 24, 2025   | Added atomicwriter                                                 |
| v0.5.11       | Oct 24, 2025   | Components: removed FileSystemPort (YAGNI), updated ports/adapters |
| v0.6.0        | Oct 26, 2025   | Rich domain models (Validate methods)                              |
| v0.6.1        | Oct 26, 2025   | SchemaValidator, SchemaResolver services                           |
| v0.6.2        | Oct 26, 2025   | FrontmatterService validation details                              |
| v0.6.3        | Oct 26, 2025   | Template file path functions                                       |
| v0.6.4        | Oct 26, 2025   | CommandOrchestrator callback pattern                               |
| v0.6.5        | Oct 26, 2025   | Dependency injection pattern                                       |
| v0.6.6        | Oct 26, 2025   | Validation architecture overview                                   |
| v0.6.7        | Oct 26, 2025   | Coding standards: (T, error)                                       |
| v0.6.8        | Oct 26, 2025   | **VaultReaderPort, VaultWriterPort, VaultFile**                    |

### Appendix C: Related Documents

**Course Correction:**

- This document: `sprint-change-proposal-2025-10-27-complete-architecture-alignment.md`
- Background: `architecture-review-and-refactoring-plan.md`

**Architecture (v0.6.8):**

- `docs/architecture/high-level-architecture.md`
- `docs/architecture/data-models.md`
- `docs/architecture/components.md`
- `docs/architecture/error-handling-strategy.md`
- `docs/architecture/coding-standards.md`
- `docs/architecture/change-log.md`

**PRD:**

- `docs/prd/epic-list.md`
- `docs/prd/epic-1-foundational-cli-static_template-engine.md`
- `docs/prd/epic-2-configuration-schema-loading.md`
- `docs/prd/epic-3-vault-indexing-engine.md`
- `docs/prd/epic-4-interactive-input-engine.md`
- `docs/prd/epic-5-schema-driven-lookups-validation.md`

**To Be Created:**

- `docs/prd/architecture-coverage-checklist.md` (Phase 2.0)
- `docs/lessons-learned-epic-1-2.md` (Phase 1)
- `docs/stories/*.md` (~54-60 files, Phase 4)

---

**END OF SPRINT CHANGE PROPOSAL**

**Status:** Awaiting approval to proceed with Phase 1 (Archive)

**Prepared by:** Sarah (Product Owner)
**Date:** 2025-10-27
**Proposal ID:** SCP-2025-001

---

## APPROVAL RECORD

**Approved By:** Jack (Project Owner)
**Date:** 2025-10-27
**Status:** ✅ APPROVED - Proceed with Phase 1

**Approved Scope:**

- ✅ Archive ALL code violations and story files
- ✅ Architecture coverage verification (Phase 2.0)
- ✅ Complete story regeneration (~54-60 stories)
- ✅ Timeline: 17-25 days
- ✅ PO time commitment: ~9-13 days for rigorous planning

**Next Immediate Action:** Begin Phase 1 (Archive) - 0.5 days

---
