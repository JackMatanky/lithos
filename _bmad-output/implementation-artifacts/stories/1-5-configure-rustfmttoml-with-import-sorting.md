# Story 1.5: configure-rustfmttoml-with-import-sorting

Status: done

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

- [x] Research comprehensive rustfmt best practices and import sorting standards from Rust community
   - [x] Analyze imports_granularity and group_imports options for optimal grouping
   - [x] Review version_sorting and max_width settings for readability
   - [x] Study additional formatting options for maintainable code (brace style, indentation, comments)
   - [x] Examine enterprise Rust projects' rustfmt configurations
- [x] Create rustfmt.toml with all best practice settings for readable and maintainable code
   - [x] Set imports_granularity = "Crate" and group_imports = "StdExternalCrate"
   - [x] Enable version_sorting = true for improved import ordering
   - [x] Configure max_width = 80 and other readability settings
   - [x] Add settings for consistent brace style, indentation, and comment formatting
- [x] Test rustfmt configuration against existing codebase
   - [x] Run cargo fmt to apply formatting
   - [x] Verify import sorting follows the configured groups
   - [x] Check for any formatting conflicts or issues
   - [x] Ensure formatted code remains readable and maintainable
- [x] Integrate rustfmt checks into development workflow
   - [x] Update mise tasks to include rustfmt with check mode
   - [x] Verify pre-commit hooks run rustfmt successfully
   - [x] Test integration with clippy and other quality tools
- [x] Document rustfmt standards and commit changes
   - [x] Update README.md with formatting standards
   - [x] Add comments to rustfmt.toml explaining each setting's purpose
   - [x] Stage and commit with conventional message: "feat(env): implement rustfmt import sorting and formatting standards"

## Dev Notes

- Used Nightly Rust for advanced formatting features.
- Configured `unstable_features = true` in `rustfmt.toml`.
- Created `rust-toolchain.toml` to pin the nightly channel and ensure deterministic builds.
- Hardened `.mise/tasks/fmt.sh` to work reliably across environments and bypass rustup shim issues.
- Created `docs/standards/rustfmt.md` for detailed formatting documentation.
- Integrated `mise run fmt --check` into `pre-commit` hooks.
- Enabled `experimental = true` in `mise.toml` to support the nightly rust backend.

## Dev Agent Record

### Agent Model Used

Claude 3.5 Sonnet

### Debug Log References

- Verified import sorting by adding unsorted imports to `crates/domain/src/lib.rs` and running `mise run fmt`.

### Completion Notes List

- Implemented comprehensive `rustfmt.toml` with `imports_granularity`, `group_imports`, and `max_width = 80`.
- Enabled `nightly` toolchain via `mise` and `rust-toolchain.toml`.
- Documented standards in `docs/standards/rustfmt.md`.
- Hardened `.mise/tasks/fmt.sh` with robust path handling and toolchain parity.
- [AI Review Fix] Corrected `Cargo.toml` MSRV to 1.92 to align with project context.
- [AI Review Fix] Added missing `imports_granularity` and `group_imports` (Crate/StdExternalCrate) to `rustfmt.toml`.
- [AI Review Fix] Refactored pre-commit to use `mise run fmt` entry point.
- [AI Review Fix] Created `rust-toolchain.toml` to eliminate manual toolchain switching.
- All pre-commit hooks pass.

### File List

- `rustfmt.toml`
- `rust-toolchain.toml`
- `mise.toml`
- `.pre-commit-config.yaml`
- `README.md`
- `.mise/tasks/fmt.sh`
- `docs/standards/rustfmt.md`
- `_bmad-output/implementation-artifacts/sprint-status.yaml`
- `_bmad-output/implementation-artifacts/stories/1-5-configure-rustfmttoml-with-import-sorting.md`
