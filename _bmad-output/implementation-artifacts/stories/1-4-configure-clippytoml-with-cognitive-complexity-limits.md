# Story 1.4: configure-clippytoml-with-cognitive-complexity-limits

Status: ready-for-dev

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

- [ ] Research comprehensive clippy best practices and stringent configurations from enterprise Rust projects (axum, vector, rust-analyzer)
   - [ ] Analyze MSRV settings and version pinning
   - [ ] Review disallowed-methods and disallowed-types for anti-pattern prevention
   - [ ] Study per-lint level configurations for pedantic and restriction groups
   - [ ] Examine test-specific allowances and restrictions
- [ ] Create clippy.toml with all best practice settings
   - [ ] Set cognitive-complexity-threshold = 25 and too-many-lines-threshold = 100
   - [ ] Configure msrv = "1.70" for minimum supported Rust version
   - [ ] Add comprehensive disallowed-methods array preventing unwrap, expect, process::exit, mem::forget
   - [ ] Set allow-unwrap-in-tests = false and allow-expect-in-tests = false
   - [ ] Configure deny array with cognitive_complexity, too_many_lines, unwrap_used, expect_used, todo, unimplemented, dbg_macro, panic, unreachable, indexing_slicing, arithmetic_side_effects, missing_docs
- [ ] Configure per-lint levels in Cargo.toml [lints.clippy] section
   - [ ] Set pedantic group to warn level
   - [ ] Set restriction group to warn level with exceptions
   - [ ] Configure specific lints for deny level (correctness, suspicious)
- [ ] Test clippy configuration against existing codebase
   - [ ] Run cargo clippy with new configuration
   - [ ] Address any legitimate violations in existing code
   - [ ] Ensure no false positives from overly strict settings
- [ ] Integrate clippy checks into development workflow
   - [ ] Update mise tasks to include clippy with deny level
   - [ ] Verify pre-commit hooks run clippy successfully
   - [ ] Test full quality pipeline (fmt + lint + test + clippy)
- [ ] Establish lint disable policy and AI agent training
   - [ ] Create documentation for lint disable procedures requiring exhaustive alternatives and justification
   - [ ] Train AI agents on strict linting patterns to minimize violations
   - [ ] Add lint disable audit trail format: `// # LINT_DISABLE_REASON: [reason] | Options tried: [list] | Justification: [why last resort]`
- [ ] Document clippy standards and commit changes
   - [ ] Update README.md with code quality standards for AI-assisted development
   - [ ] Add comments to clippy.toml explaining each section and AI considerations
   - [ ] Stage and commit with conventional message: "feat(env): implement stringent clippy linting with cognitive complexity limits and AI safeguards"

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

{{agent_model_name_version}}

### Debug Log References

### Completion Notes List

### File List
