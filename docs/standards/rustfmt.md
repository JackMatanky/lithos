# Rustfmt Standards

Lithos enforces strict formatting standards to ensure that both human and AI-generated code remains maintainable and readable. We use `rustfmt` with nightly features to enforce consistent import sorting and code layout.

## Configuration Details

The project uses a custom `rustfmt.toml` with the following key rules:

### Import Management
- **Granularity**: `imports_granularity = "Crate"`
  - All imports from the same crate are combined into a single `use` statement.
- **Grouping**: `group_imports = "StdExternalCrate"`
  - Imports are grouped as: standard library (`std`), followed by external crates, and finally internal modules.
- **Sorting**: `reorder_imports = true`
  - Imports are automatically sorted alphabetically within their groups.

### Code Layout
- **Line Width**: `max_width = 80`
  - Code and comments are limited to 80 characters to ensure readability in all editors and side-by-side diffs.
- **Brace Style**: `brace_style = "SameLineWhere"`
  - Braces are placed on the same line as `where` clauses for consistent vertical spacing.
- **Indentation**: `tab_spaces = 4`
  - Consistent 4-space indentation using spaces (no hard tabs).

### Forcing Multi-line Structures
To keep git diffs clean and improve readability, several structures are forced to multi-line:
- **Functions**: `fn_single_line = false`
- **Struct Literals**: `struct_lit_single_line = false`
- **Small Heuristics**: `use_small_heuristics = "Off"`

## Applying Formatting

Formatting is managed via `mise` and integrated into the `pre-commit` workflow.

### Automated Formatting
To automatically apply formatting to the entire workspace:
```bash
mise run fmt
```
This command uses the nightly toolchain and enables unstable features required by our configuration.

### CI/Verification
To check formatting without making changes (used in CI):
```bash
mise run fmt --check
```

## AI Agent Instructions
- Read and follow these standards exactly.
- Do not attempt to bypass formatting checks.
- If a specific file requires a formatting exception, use `#[rustfmt::skip]` with a clear justification.
