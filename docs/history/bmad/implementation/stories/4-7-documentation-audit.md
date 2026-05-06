# Story 4.7: Documentation Audit

Status: pending

<!-- This story file contains COMPREHENSIVE context to prevent developer mistakes, omissions, and disasters -->

## Story

As a developer maintaining the long-term health of the codebase,
I want a comprehensive audit of all Epic 4 documentation and doc comments,
So that the codebase remains self-documenting, precise, and free of unnecessary clutter.

## Acceptance Criteria

1. **Given** Epic 4 implementation and testing are complete
   **When** I conduct the documentation audit
   **Then** every public API has accurate, high-fidelity doc comments (`///`)

2. **Given** the "Why" mandate in project-context.md
   **When** I review doc comments
   **Then** they focus on invariants and architectural context rather than just repeating the function name

3. **Given** doc-tests are used as "Living Documentation"
   **When** I audit the examples
   **Then** every doc-test is accurate, functional, and demonstrates idiomatic usage without boilerplate noise

4. **Given** the "Concise Documentation" goal
   **When** I review documentation
   **Then** unnecessary or redundant information is removed, leaving only high-signal content

5. **Given** the audit is complete
   **When** I run `mise run doc`
   **Then** the generated documentation is professional, clean, and perfectly reflects the system state

## Tasks / Subtasks

### Task 1: Public API Documentation Audit
- [ ] Review all `pub` and `pub(crate)` structs, enums, and traits in Epic 4 crates
- [ ] Ensure every public member has a `///` doc comment following the project style
- [ ] Verify that doc comments include an `# Errors` section for any `Result`-returning function
- [ ] Ensure all public traits include a runnable `/// # Example` block

### Task 2: Doc-Test Precision & Optimization
- [ ] Run `cargo test --doc` to verify all examples are functional
- [ ] Review each doc-test and hide setup boilerplate using the `#` prefix
- [ ] Ensure examples demonstrate the *idiomatic* way to use the API, including proper error handling
- [ ] Remove any doc-tests that are purely redundant with existing unit tests unless they provide unique documentation value

### Task 3: Signal-to-Noise Ratio Review
- [ ] Audit comments for "fluff" or obvious statements (e.g., `/// Sets the name` for `fn set_name`)
- [ ] Replace obvious descriptions with "The Why": invariants, thread-safety notes, or performance implications
- [ ] Ensure that technical terms are used consistently across all documentation
- [ ] Remove any outdated or "TODO" comments remaining from development

### Task 4: Architectural Alignment Check
- [ ] Cross-reference doc comments with `architecture.md` and `project-context.md` to ensure terminology alignment
- [ ] Verify that hexagonal boundaries are correctly described in the documentation
- [ ] Ensure that persistence-related invariants (rkyv, redb) are clearly documented in the infrastructure layer

### Task 5: Project Lifecycle & Documentation Updates
- [ ] **ROADMAP UPDATE:** Update `ROADMAP.md` to mark Epic 4 (File Loading Strategy Foundation) as complete in Milestone 2.
- [ ] **CHANGELOG UPDATE:** Update `CHANGELOG.md` with Epic 4 highlights (Unified loading interface, format detection, parsing strategies, loading mocks).
- [ ] Ensure all documentation accurately reflects the final state of the implementation.

### Task 6: Quality Assurance, Commit, and Remote Sync (MANDATORY FINAL TASK)
- [ ] Run `mise run doc` and browse the generated documentation to ensure clarity and professional presentation
- [ ] Verify that no linter warnings (e.g., `missing_docs`, `rustdoc::broken_intra_doc_links`) exist
- [ ] Run `mise run fmt` to format all code according to project standards
- [ ] Run `mise run lint` to check for all code quality issues and anti-patterns
- [ ] Run `mise run verify` for comprehensive verification (fmt + lint + tests + coverage)
- [ ] Run `pre-commit run --all-files` to execute all pre-commit hooks
- [ ] **CRITICAL:** Fix ALL linter warnings - NO EXCEPTIONS, NO BYPASSING
- [ ] **CRITICAL:** Ensure ALL pre-commit hooks pass - NO EXCEPTIONS, NO BYPASSING
- [ ] **MANDATORY:** If any warnings or hook failures exist, fix them immediately and re-run verification
- [ ] **MANDATORY:** Confirm all documentation meets quality standards and covers all requirements
- [ ] Stage all files created or modified during story development
- [ ] Commit with conventional commit message: `docs: complete epic 4 documentation audit and finalize project artifacts`
- [ ] **REMOTE SYNC:** Push all changes to the remote branch `rust-conversion` using `git push`.
- [ ] **CI VERIFICATION:** Monitor and verify that all GitHub Action CI checks pass for the pushed changes.

## Dev Notes

### The "Why" Mandate
- Refer to `project-context.md#Documentation as "Agent Glue"`: Doc comments MUST focus on Invariants and Architectural Context.

### Living Documentation
- Doc-tests are not just tests; they are the primary way new developers (and AI agents) learn the API. They must be perfect.
- Use the `lithos-test-utils` where appropriate to keep examples concise.
