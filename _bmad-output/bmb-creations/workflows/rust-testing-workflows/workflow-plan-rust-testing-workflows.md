---
stepsCompleted: ['step-01-discovery', 'step-02-classification', 'step-03-requirements']
created: 2026-02-11
status: DISCOVERY
---

# Workflow Creation Plan

## Discovery Notes

**User's Vision:**
Create specialized Rust testing workflows for TEA agent that complement existing TEA workflows but emphasize Rust best practices. These workflows should be context-aware, high-quality, and efficient, avoiding vanity metrics and technical debt. They will be used frequently by developers working on Lithos project.

**Who It's For:**
Developers working on Lithos Rust project who need to write or improve tests. The workflows will be used by TEA agent to provide better testing guidance specific to Rust patterns and project's architecture.

**What It Produces:**
5 specialized workflows:
1. Rust TDD Workflow - Test-first development
2. Rust Unit Test Workflow - Comprehensive testing of existing code
3. Rust Test Review Workflow - Adversarial test quality improvement
4. Rust Integration & E2E Test Workflow - Component and full-stack testing
5. Rust Benchmark Workflow - Performance testing

**Key Insights:**
- Workflows must be context-aware to avoid boilerplate testing
- Quality over quantity - prevent vanity metrics and technical debt
- Dual capability for unit tests: TDD support + adversarial test review
- Integration + E2E combined due to shared patterns and mid-frequency usage
- Benchmarks kept separate due to different mindset and tooling
- Each workflow should have deep contextual analysis before execution
- Must complement, not replace, existing TEA workflows
- CRITICAL: Workflows MUST be scope-aware - understand what level of system they're operating at
- File-level scope: Unit tests for single functions/structs need only local context
- Module-level scope: Unit tests that aggregate across a module need module-wide context
- Multi-module scope: Integration tests need context across module boundaries
- Full-project scope: E2E tests need complete project context (CLI, all modules, external deps)
- Each workflow must determine its scope and gather appropriate context before proceeding
- Should ask clarifying questions when scope is ambiguous or context is insufficient
- Adaptive intelligence: When patterns suggest complexity beyond baseline, expand context automatically
- Example: Simple unit test that depends on external configuration may need project-level config context

## Classification Decisions

**Workflow Name:** tea-rust
**Target Path:** /Users/jack/Documents/41_personal/lithos/_bmad/custom/src/workflows/tea-rust/

**4 Key Decisions:**
1. **Document Output:** Hybrid approach
   - TDD: Document-producing (builds on story files)
   - Unit Test: Document-producing (test strategy/coverage plan)
   - Test Review: Document-producing (Rust-specific improvement reports)
   - Integration/E2E: Non-document (orchestrates testing)
   - Benchmark: Document-producing (performance reports, baseline comparison)
2. **Module Affiliation:** Standalone
3. **Session Type:** Continuable
4. **Lifecycle Support:** Tri-modal (Create + Edit + Validate)

**Structure Implications:**
- Needs steps-c/, steps-e/, steps-v/
- Needs step-01b-continue.md for continuation support
- Needs context discovery templates for scope awareness
- Needs hybrid document/non-document handling
- Custom location allows independent evolution from TEA module

## Requirements

**Flow Structure:**
- Pattern: Hybrid modular system (Router + 5 specialized workflows)
- Phases: Router dispatch → Specialized execution → Optional orchestration
- Estimated steps: 5-7 per workflow (25-35 total across system)
- Continuable: Yes - complex workflows may span multiple sessions

**User Interaction:**
- Style: Mixed (Router highly collaborative, specialized workflows mostly autonomous with strategic checkpoints)
- Decision points: Router workflow choices, workflow approval points, continuation checkpoints
- Checkpoint frequency: Each major phase or when user intervention needed

**Inputs Required:**
- Required: Router - user goal; Specialized - target scope (auto-detected when possible)
- Optional: Specific concerns, existing tests to review, performance focus areas
- Prerequisites: Project structure context (auto-discovered)
- Smart defaults: 80% auto-discovered, 20% user input when critical

**Output Specifications:**
- Type: Hybrid (TDD/Unit/Test Review/Benchmark = document-producing; Integration/E2E = orchestrating actions)
- Format: Free-form (TDD), Semi-structured (Unit Test), Structured (Test Review), Structured + data (Benchmark), Orchestration artifacts (Integration/E2E)
- Sections: Workflow-specific, comprehensive doc comment templates for Integration/E2E/Benchmark, NO doc comments for unit tests but GWT comment patterns inside tests
- Frequency: Single per workflow execution, with progressive content building

**Doc Comment Template Standards:**
- Unit Tests (TDD & Unit Test workflows): NO doc comments, GWT comments inside tests (prefer GWT format), clear behavioral documentation within test logic
- Integration/E2E & Benchmark workflows: Comprehensive module-level docs (`///`) + detailed function-level docs with full project context, performance characteristics, methodology, maintenance contracts
- Test Review workflow: Improved tests WITHOUT doc comments (maintains consistency), but provides template access for reference

**Knowledge Base Strategy:**
- Primary: `{tea-rust-workflow-folder}/knowledge/` with Rust-specific principles (unit, integration, E2E, fixtures, assertions, naming, tools, anti-patterns, best practices)
- Secondary: TEA test architecture knowledge base for universal testing principles
- Conflict resolution: Rust-specific takes precedence, document reasoning, hierarchical knowledge structure

**Success Criteria:**
- Router: Clear workflow guidance, proper orchestration
- TDD: Tests pass, no regressions, developer confidence, implementation follows test specification
- Unit Test: ALL public components tested, edge cases covered, GWT comments inside tests, no external doc comments
- Test Review: Senior dev ownership, quality improvements, ALL public components validated, flaky test elimination, concrete issue fixes with user approval
- Integration/E2E: Real interaction testing, reproducible setup, comprehensive documentation, public API coverage, failure diagnosability
- Benchmark: Regression detection, actionable insights, consistent results, public API benchmarking, noise reduction
- Quality validation: "Did this make code more robust?", "Are users less likely to encounter bugs?", "Did this improve maintainability?", "Are all public components protected?"

**Instruction Style:**
- Overall: Intent-based (Router guided, specialized workflows adaptive)
- Notes: Router collaborative, specialized autonomous with checkpoints, Test Review = senior dev ownership like dev agent code review
- Context awareness: Baseline scope heuristics (file/module/project) + adaptive expansion, Rust-specific knowledge privilege over TEA general knowledge
