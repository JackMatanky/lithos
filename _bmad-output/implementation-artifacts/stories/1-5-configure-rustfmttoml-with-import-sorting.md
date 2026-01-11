# Story 1.5: configure-rustfmttoml-with-import-sorting

Status: ready-for-dev

<!-- Note: Validation is optional. Run validate-create-story for quality check before dev-story. -->

## Story

As a developer formatting code,
I want consistent import sorting and formatting standards,
So that code style is uniform and readable across the codebase.

## Acceptance Criteria

**Given** I have researched rustfmt best practices for import sorting and code formatting
**When** I review rustfmt.toml configuration
**Then** import sorting is configured with:
- `imports_granularity = "Crate"` - Group imports by crate
- `group_imports = "StdExternalCrate"` - Standard library, external crates, then internal
- `version_sorting = true` - Use version-sort algorithm for better sorting

**Given** I have researched code formatting standards for readable and maintainable Rust code
**When** I check the rustfmt configuration
**Then** settings enhance readability and maintainability:
- `max_width = 80` - Optimal line length for readability and maintainability
- `tab_spaces = 4` - Consistent indentation for better code structure
- `brace_style = "SameLineWhere"` - Consistent brace placement for clarity
- `comment_width = 80` - Ensures comments are readable without wrapping
- `fn_single_line = false` - Forces functions to be multi-line for better readability
- `struct_lit_single_line = false` - Prevents single-line struct literals
- `array_lit_single_line = false` - Prevents single-line array literals
- `use_small_heuristics = "Off"` - Avoids condensing code into single lines

**Given** rustfmt.toml is configured with import sorting and formatting standards
**When** I run `cargo fmt`
**Then** all imports are sorted consistently across the codebase following std → external → internal grouping

**Given** code with unsorted or poorly formatted imports
**When** I run `cargo fmt --check`
**Then** the command fails with specific file locations needing formatting

**Given** I have researched formatting standards for Rust ecosystems
**When** I check the configuration
**Then** settings align with Rust community standards for:
- Maximum line width (80 characters)
- Brace style consistency (SameLineWhere preferred)
- Comment formatting (80 width limit)
- Macro formatting and attribute handling

**Given** code is formatted with rustfmt
**When** I review the codebase
**Then** the code is more readable and maintainable due to consistent formatting and logical import grouping

## Tasks / Subtasks

- [ ] Research comprehensive rustfmt best practices and import sorting standards from Rust community
   - [ ] Analyze imports_granularity and group_imports options for optimal grouping
   - [ ] Review version_sorting and max_width settings for readability
   - [ ] Study additional formatting options for maintainable code (brace style, indentation, comments)
   - [ ] Examine enterprise Rust projects' rustfmt configurations
- [ ] Create rustfmt.toml with all best practice settings for readable and maintainable code
   - [ ] Set imports_granularity = "Crate" and group_imports = "StdExternalCrate"
   - [ ] Enable version_sorting = true for improved import ordering
   - [ ] Configure max_width = 80 and other readability settings
   - [ ] Add settings for consistent brace style, indentation, and comment formatting
- [ ] Test rustfmt configuration against existing codebase
   - [ ] Run cargo fmt to apply formatting
   - [ ] Verify import sorting follows the configured groups
   - [ ] Check for any formatting conflicts or issues
   - [ ] Ensure formatted code remains readable and maintainable
- [ ] Integrate rustfmt checks into development workflow
   - [ ] Update mise tasks to include rustfmt with check mode
   - [ ] Verify pre-commit hooks run rustfmt successfully
   - [ ] Test integration with clippy and other quality tools
- [ ] Document rustfmt standards and commit changes
   - [ ] Update README.md with formatting standards
   - [ ] Add comments to rustfmt.toml explaining each setting's purpose
   - [ ] Stage and commit with conventional message: "feat(env): implement rustfmt import sorting and formatting standards"

## Dev Notes

- Relevant architecture patterns and constraints
- Source tree components to touch
- Testing standards summary

### Project Structure Notes

- Alignment with unified project structure (paths, modules, naming)
- Detected conflicts or variances (with rationale)

### References

- [Source: _bmad-output/planning-artifacts/architecture.md#Implementation Patterns & Consistency Rules]
- [Source: _bmad-output/planning-artifacts/epics/epic-1-development-environment-tooling-mvp-core.md#Story 1.5]
- [Source: Rustfmt Documentation (https://rust-lang.github.io/rustfmt/)]
- [Source: Rust 2024 Edition Guide - Version Sorting (https://doc.rust-lang.org/edition-guide/rust-2024/rustfmt-version-sorting.html)]

## Dev Agent Record

### Agent Model Used

{{agent_model_name_version}}

### Debug Log References

### Completion Notes List

### File List
