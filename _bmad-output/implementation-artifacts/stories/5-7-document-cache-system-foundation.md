# Story 5.7: Document Cache System Foundation

Status: ready-for-dev

<!-- Note: Validation is optional. Run validate-create-story for quality check before dev-story. -->

## Story

As a developer integrating caching in adapter implementations,
I want clear documentation for the Cache SPI with concrete examples and comprehensive doc comments,
So that I understand how to use the generic primitives in domain-specific contexts.

## Original Epic Acceptance Criteria

**Given** all Epic 5 code is implemented
**When** I review all doc comments
**Then** they are accurate, precise, and follow project standards from `project-context.md`
**And** every public component uses proper `///` documentation format

**Given** all Epic 5 public components are documented
**When** I verify doc comments
**Then** all public traits, structs, enums, functions, and methods have:

- Clear `///` doc comments explaining their purpose
- `# Examples` sections with runnable, well-written doc tests
- `# Errors` sections documenting error conditions where applicable
- `# Panics` sections documenting panic conditions where applicable

**And** doc tests demonstrate realistic usage patterns
**And** doc tests compile and pass via `cargo test --doc`

**Given** the Cache SPI is implemented
**When** I create `crates/adapters/src/spi/cache/README.md`
**Then** it includes:

- **Overview**: Purpose of the Cache SPI as generic infrastructure
- **Trait Contract**: Explanation of `Cache<K, V>` methods and semantics
- **Implementations**: MokaCache (memory), RedbCache (disk), Coordinator (memory+disk)
- **Example 1**: Using RedbCache with table isolation for configuration storage
- **Example 2**: Using Coordinator for schema caching with metadata tracking
- **rkyv Requirements**: Types cached in RedbCache must derive Archive + Serialize + Deserialize
- **Table Naming Conventions**: Suggested patterns (e.g., "schemas", "config", "query_results")

**Given** developers need architectural context
**When** I create `docs/spi/cache-foundation.md`
**Then** it explains:

- **Memory/Disk Architecture**: Why we use two-level caching (speed vs persistence)
- **When to Use What**:
  - MokaCache alone: Temporary session data, template execution caching
  - RedbCache alone: Persistent data without frequent access (cold storage)
  - Coordinator: High-performance persistent caching (schemas, config)
- **Metadata Storage**: How to use `CachedEntry` metadata for versioning, hash tracking, rollback
- **Integration Patterns**: How adapter implementations compose Cache primitives
- **Performance Characteristics**: Latency targets, memory bounds, concurrency behavior

**Given** examples must be runnable
**When** I include code examples in documentation
**Then** they compile and demonstrate:

- Creating a MokaCache with TTL configuration
- Creating a RedbCache with table isolation
- Composing a Coordinator with both layers
- Using metadata for hash-based invalidation
- Using metadata for versioned snapshots

## Acceptance Criteria (Quality Gates)

**Given** I am documenting the system
**When** I run `mise run test:unit:adapters --doc`
**Then** all examples in doc comments pass
**And** examples in `README.md` and `docs/` are verified against the current API

**Given** quality standards are paramount
**When** I run `mise run verify`
**Then** zero documentation-related warnings or errors are reported
**And** all module-level documentation is properly linked

## Tasks / Subtasks

### Task 1: Doc Comment Scaffolding & Audit
- [ ] Subtask 1.1: Audit all public components in `spi/cache/` for missing documentation or outdated comments
- [ ] Subtask 1.2: Identify missing `///` sections for `Cache` trait, `MokaCache`, `RedbCache`, `CacheCoordinator`, and `CacheError`
- [ ] Subtask 1.3: Run `mise run lint` and verify documentation coverage warnings
- [ ] Subtask 1.4: Run `mise run lint` and fix all clippy warnings/errors
    - **NOTE**: Review test-developer-guide.md Section 8 for comprehensive guidance on linting and code quality
    - **RULE**: Fix clippy issues properly rather than suppressing with `#[expect(...)]` attributes
    - **WORKFLOW**: `mise run lint` → Read diagnostic → Apply suggestions → Refactor for complexity → Verify with `mise run verify`
    - **ALLOWED USES**: `#[expect(...)]` only for intentional violations necessary for tests; `#[allow(...)]` primarily for generated code like `automock`
    - **COMMON FIXES**: Extract helper functions, use builder patterns, remove unnecessary collect(), avoid shadowing, document errors, use proper assertions

### Task 2: Trait & Error Documentation
- [ ] Subtask 2.1: Add comprehensive `///` comments to `trait Cache` including `# Examples`, `# Errors`, and `# Panics`
- [ ] Subtask 2.2: Ensure all trait methods have executable doc tests
- [ ] Subtask 2.3: Document `CacheError` variants with detailed, context-aware descriptions
- [ ] Subtask 2.4: Run `mise run test:unit:adapters --doc` and verify trait documentation passes
- [ ] Subtask 2.5: Run `mise run lint` and fix all warnings/errors
    - **NOTE**: Review test-developer-guide.md Section 8 for comprehensive guidance on linting and code quality
    - **RULE**: Fix clippy issues properly rather than suppressing with `#[expect(...)]` attributes
    - **WORKFLOW**: `mise run lint` → Read diagnostic → Apply suggestions → Refactor for complexity → Verify with `mise run verify`
    - **ALLOWED USES**: `#[expect(...)]` only for intentional violations necessary for tests; `#[allow(...)]` primarily for generated code like `automock`
    - **COMMON FIXES**: Extract helper functions, use builder patterns, remove unnecessary collect(), avoid shadowing, document errors, use proper assertions

### Task 3: Adapter Implementation Documentation
- [ ] Subtask 3.1: Implement `MokaCache` doc comments with async examples and explanation of TinyLFU protection
- [ ] Subtask 3.2: Implement `RedbCache` doc comments with persistence examples, table isolation usage, and `rkyv` setup requirements
- [ ] Subtask 3.3: Verify all adapter-specific examples compile and run via `cargo test --doc`
- [ ] Subtask 3.4: Run `mise run lint` and fix all warnings/errors
    - **NOTE**: Review test-developer-guide.md Section 8 for comprehensive guidance on linting and code quality
    - **RULE**: Fix clippy issues properly rather than suppressing with `#[expect(...)]` attributes
    - **WORKFLOW**: `mise run lint` → Read diagnostic → Apply suggestions → Refactor for complexity → Verify with `mise run verify`
    - **ALLOWED USES**: `#[expect(...)]` only for intentional violations necessary for tests; `#[allow(...)]` primarily for generated code like `automock`
    - **COMMON FIXES**: Extract helper functions, use builder patterns, remove unnecessary collect(), avoid shadowing, document errors, use proper assertions

### Task 4: Coordinator & Orchestration Documentation
- [ ] Subtask 4.1: Implement `CacheCoordinator` doc comments explaining Read-Through/Write-Through strategies
- [ ] Subtask 4.2: Add runnable examples for composing the coordinator with Moka and Redb backends
- [ ] Subtask 4.3: Document metadata-based invalidation patterns with concrete code snippets
- [ ] Subtask 4.4: Run `mise run test:unit:adapters --doc` and verify coordinator examples pass
- [ ] Subtask 4.5: Run `mise run lint` and fix all warnings/errors
    - **NOTE**: Review test-developer-guide.md Section 8 for comprehensive guidance on linting and code quality
    - **RULE**: Fix clippy issues properly rather than suppressing with `#[expect(...)]` attributes
    - **WORKFLOW**: `mise run lint` → Read diagnostic → Apply suggestions → Refactor for complexity → Verify with `mise run verify`
    - **ALLOWED USES**: `#[expect(...)]` only for intentional violations necessary for tests; `#[allow(...)]` primarily for generated code like `automock`
    - **COMMON FIXES**: Extract helper functions, use builder patterns, remove unnecessary collect(), avoid shadowing, document errors, use proper assertions

### Task 5: README and Architectural Guide Creation
- [ ] Subtask 5.1: Create `crates/adapters/src/spi/cache/README.md` containing the technical overview and table naming conventions
- [ ] Subtask 5.2: Create `docs/spi/cache-foundation.md` with detailed architectural context and L1/L2 flow descriptions
- [ ] Subtask 5.3: Ensure both documents include links to the relevant source code and ADRs
- [ ] Subtask 5.4: Run `mise run verify` and confirm all documentation is properly formatted and passes quality gates
- [ ] Subtask 5.5: Run `mise run lint` and fix all warnings/errors
    - **NOTE**: Review test-developer-guide.md Section 8 for comprehensive guidance on linting and code quality
    - **RULE**: Fix clippy issues properly rather than suppressing with `#[expect(...)]` attributes
    - **WORKFLOW**: `mise run lint` → Read diagnostic → Apply suggestions → Refactor for complexity → Verify with `mise run verify`
    - **ALLOWED USES**: `#[expect(...)]` only for intentional violations necessary for tests; `#[allow(...)]` primarily for generated code like `automock`
    - **COMMON FIXES**: Extract helper functions, use builder patterns, remove unnecessary collect(), avoid shadowing, document errors, use proper assertions

### Task 6: Final Verification & Integration
- [ ] Subtask 6.1: Perform a final pass of all public APIs to ensure 100% documentation coverage
- [ ] Subtask 6.2: Verify that all links in the generated markdown files are valid
- [ ] Subtask 6.3: Run `mise run verify` to confirm all Lithos quality gates are satisfied
- [ ] Subtask 6.4: Run `pre-commit run --all-files` and verify all hooks pass (NEVER use `--no-verify`)
- [ ] Subtask 6.5: Stage and commit all files created, deleted, or modified during the story implementation with a fully descriptive conventional commit style message (NEVER use `--no-verify`)
    - **NOTE**: Review test-developer-guide.md Section 8 for comprehensive guidance on linting and code quality
    - **RULE**: Fix clippy issues properly rather than suppressing with `#[expect(...)]` attributes
    - **WORKFLOW**: `mise run lint` → Read diagnostic → Apply suggestions → Refactor for complexity → Verify with `mise run verify`
    - **ALLOWED USES**: `#[expect(...)]` only for intentional violations necessary for tests; `#[allow(...)]` primarily for generated code like `automock`
    - **COMMON FIXES**: Extract helper functions, use builder patterns, remove unnecessary collect(), avoid shadowing, document errors, use proper assertions

## Dev Notes

### Architecture Compliance
- **Living Documentation**: Documentation is treated as executable code via doc tests.
- **Port/Adapter Clarity**: Ensure the distinction between the Port (`trait Cache`) and Adapters is crystal clear for future implementers.
- **Transparency**: Architectural decisions regarding persistence and consistency must be justified in the foundation guide.

### Technical Requirements
- **Runnable Examples**: Every example in doc comments must be a working Rust program snippet.
- **Error Mapping**: Documentation must explain the semantic meaning of every `CacheError` variant.
- **Async Focus**: Documentation must explicitly state that all methods are async and require a tokio executor.

### Library Dependencies
- **async-trait**: Required for trait examples.
- **tokio**: Required for async doc test execution.
- **tracing**: For observability documentation.

### File Structure Requirements
- **Location**: `crates/adapters/src/spi/cache/README.md`
- **Location**: `docs/spi/cache-foundation.md`
- **In-code**: `crates/adapters/src/spi/cache/*.rs` and `spi/errors.rs`.

### Project Structure Notes
- **Alignment**: Standardizes documentation across the adapters crate.
- **Discovery**: Enhances discoverability of the caching system for new developers.

### References
- [Source: project-context.md#Documentation-as-Agent-Glue]
- [Source: project-context.md#Doc-Tests]
- [Source: Story 5.1 - 5.6]

## Dev Agent Record

### Agent Model Used
Claude-3.5-Sonnet (2024-10-22)

### Debug Log References
None - Story created through systematic analysis of artifacts and project context.

### Completion Notes List
- Refactored to remove TDD framework language while maintaining granularity and clarity.
- Preserved original Epic ACs for documentation depth.
- Integrated mandatory linting workflows and mise orchestration.
- Provided specific tasks for README and Architectural Guide creation.
- Enforced Section 8 compliance for all doc snippets.

### File List
- `crates/adapters/src/spi/cache/README.md` - Technical overview.
- `docs/spi/cache-foundation.md` - Architectural guide.
- `crates/adapters/src/spi/cache/*.rs` - Doc comment targets.
