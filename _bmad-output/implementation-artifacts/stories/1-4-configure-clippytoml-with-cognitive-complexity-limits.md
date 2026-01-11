# Story 1.4: configure-clippytoml-with-cognitive-complexity-limits

Status: done

<!-- Note: Validation is optional. Run validate-create-story for quality check before dev-story. -->

## Story

As a developer writing code,
I want clippy to enforce cognitive complexity limits as a quality safeguard,
So that functions remain maintainable and complex logic is broken down appropriately.

## Acceptance Criteria

**Given** I have researched clippy best practices for cognitive complexity and stringent linting
**When** I review clippy.toml configuration
**Then** cognitive complexity limits are set to:
- `cognitive-complexity-threshold = 25` (deny level)
- `too-many-lines-threshold = 100` (function length limit)

**Given** I have researched anti-pattern prevention in Rust
**When** I check the clippy configuration
**Then** these anti-patterns are denied:
- `cognitive_complexity` - Functions exceeding complexity threshold
- `too_many_lines` - Functions exceeding length threshold
- `unwrap_used` - No unwrap in production code
- `expect_used` - No expect in production code
- `todo` - No TODO comments in production code
- `unimplemented` - No unimplemented!() in production code
- `dbg_macro` - No debug prints in production code
- `panic` - No explicit panics in production code
- `unreachable` - No unreachable code in production code
- `indexing_slicing` - No direct indexing that can panic
- `arithmetic_side_effects` - No arithmetic that can overflow
- `missing_docs` - All public items must be documented

**Given** I have researched additional stringent clippy settings
**When** I check the configuration
**Then** additional quality gates are configured:
- `msrv = "1.70"` - Minimum supported Rust version
- `allow-unwrap-in-tests = false` - No unwrap in tests either
- `allow-expect-in-tests = false` - No expect in tests
- `disallowed-methods = ["std::env::current_dir", "std::fs::canonicalize"]` - Prevent common pitfalls

**Given** clippy.toml is configured with stringent rules
**When** I run `cargo clippy`
**Then** code violating any of these rules generates deny-level errors

**Given** code violates the stringent rules
**When** I run clippy
**Then** specific line numbers, suggestions, and explanations are provided for remediation

**Given** code will be primarily AI-generated
**When** lint violations occur
**Then** all alternatives must be exhausted before disabling lints, with comprehensive documentation of attempts, alternatives tried, and justification for why disabling is the only viable option

**Given** I have researched clippy ecosystem standards for AI-assisted development
**When** I check the configuration
**Then** settings align with enterprise Rust projects, prevent code smells and AI-generated slop, encourage best practices, and maintain strict quality gates for automated code generation

## Tasks / Subtasks

- [x] Research comprehensive clippy best practices and stringent configurations from enterprise Rust projects (axum, vector, rust-analyzer)
   - [x] Analyze MSRV settings and version pinning
   - [x] Review disallowed-methods and disallowed-types for anti-pattern prevention
   - [x] Study per-lint level configurations for pedantic and restriction groups
   - [x] Examine test-specific allowances and restrictions
- [x] Create clippy.toml with all best practice settings
   - [x] Set cognitive-complexity-threshold = 25 and too-many-lines-threshold = 100
   - [x] Configure msrv = "1.70" for minimum supported Rust version
   - [x] Add comprehensive disallowed-methods array preventing unwrap, expect, process::exit, mem::forget
   - [x] Set allow-unwrap-in-tests = false and allow-expect-in-tests = false
   - [x] Configure deny array with cognitive_complexity, too_many_lines, unwrap_used, expect_used, todo, unimplemented, dbg_macro, panic, unreachable, indexing_slicing, arithmetic_side_effects, missing_docs
- [x] Configure per-lint levels in Cargo.toml [lints.clippy] section
   - [x] Set pedantic group to warn level
   - [x] Set restriction group to warn level with exceptions
   - [x] Configure specific lints for deny level (correctness, suspicious)
- [x] Test clippy configuration against existing codebase
   - [x] Run cargo clippy with new configuration
   - [x] Address any legitimate violations in existing code
   - [x] Ensure no false positives from overly strict settings
- [x] Integrate clippy checks into development workflow
   - [x] Update mise tasks to include clippy with deny level
   - [x] Verify pre-commit hooks run clippy successfully
   - [x] Test full quality pipeline (fmt + lint + test + clippy)
- [x] Establish lint disable policy and AI agent training
   - [x] Create documentation for lint disable procedures requiring exhaustive alternatives and justification
   - [x] Train AI agents on strict linting patterns to minimize violations
   - [x] Add lint disable audit trail format: `// # LINT_DISABLE_REASON: [reason] | Options tried: [list] | Justification: [why last resort]`
- [x] Document clippy standards and commit changes
   - [x] Update README.md with code quality standards for AI-assisted development
   - [x] Add comments to clippy.toml explaining each section and AI considerations
   - [x] Stage and commit with conventional message: "feat(env): implement stringent clippy linting with cognitive complexity limits and AI safeguards"

## Dev Notes

- Relevant architecture patterns and constraints
- Source tree components to touch
- Testing standards summary

### Project Structure Notes

- Alignment with unified project structure (paths, modules, naming)
- Detected conflicts or variances (with rationale)

### Technical Requirements

- Configure clippy.toml with cognitive-complexity-threshold = 25, too-many-lines-threshold = 100, msrv = "1.70"
- Define disallowed-methods: ["std::option::Option::unwrap", "std::result::Result::unwrap", "std::result::Result::expect", "std::process::exit", "std::mem::forget"]
- Set allow-unwrap-in-tests = false, allow-expect-in-tests = false
- Configure deny array: ["cognitive_complexity", "too_many_lines", "unwrap_used", "expect_used", "todo", "unimplemented", "dbg_macro", "panic", "unreachable", "indexing_slicing", "arithmetic_side_effects", "missing_docs", "mutable_static", "unsafe_code"]
- Update Cargo.toml [lints.clippy]: pedantic = "warn", restriction = "warn", correctness = "deny", suspicious = "deny"
- Implement mandatory lint disable policy: exhaustive alternatives required with full documentation

### Party Mode Insights (AI-Assisted Development Considerations)

- **AI Code Generation Risks**: AI agents can introduce subtle bugs in error handling, boundary conditions, and architectural patterns - linting serves as primary defense
- **Zero Tolerance for Convenience Disables**: Only disable lints as absolute last resort after exhausting all refactoring options
- **Documentation Requirements**: All lint disables must include audit trail: reason, alternatives tried, why disabling is necessary
- **AI Agent Training**: Train agents on linting patterns to minimize violations and understand when refactoring vs disabling is appropriate
- **Quality Metrics**: Track lint violation rates and exception documentation completeness as AI development quality indicators

### References

- [Source: _bmad-output/planning-artifacts/architecture.md#Clippy Complexity Limits]
- [Source: _bmad-output/planning-artifacts/architecture.md#Implementation Patterns & Consistency Rules]
- [Source: _bmad-output/planning-artifacts/epics/epic-1-development-environment-tooling-mvp-core.md#Story 1.4]
- [Source: Clippy Documentation (https://doc.rust-lang.org/clippy/configuration.html)]

## Dev Agent Record

### Agent Model Used

Claude 3.5 Sonnet

### Debug Log References

- Configured clippy.toml and Cargo.toml per requirements.
- Resolved contradictory lints in restriction group.
- Fixed existing violations in domain and cli crates.
- Verified with mise run verify.

### Completion Notes List

- Stringent clippy linting implemented.
- Cognitive complexity threshold set to 25.
  - AI safeguards documented in docs/standards/clippy.md.
- Fixed audit trail violations in CLI and Test crates.
- Replaced println! with tracing in CLI entry point.
- Updated mise lint task to enforce deny-level warnings.

### File List

- clippy.toml
- Cargo.toml
- mise.toml
- crates/domain/src/lib.rs
- crates/cli/src/main.rs
- crates/app/tests/dummy_integration.rs
- docs/standards/clippy.md
- README.md

## Change Log

- 2026-01-11: Implement stringent clippy linting with cognitive complexity limits and AI safeguards.
