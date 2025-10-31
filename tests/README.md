# Test Suite Overview

This directory hosts integration and end-to-end tests for Lithos. Fixtures live
under `testdata/` and must follow the canonical layout introduced in Story 1.15.

## Canonical Fixture Layout

- `testdata/schemas/` – JSON schema fixtures organised into subdirectories:
  - `valid/` – valid schema examples (e.g., `note.json`).
  - `invalid/` – malformed schema files.
  - `duplicate/` – duplicate-name scenarios for negative tests.
  - `properties/` – property bank fragments (e.g., `bank.json`).
  - Root files (e.g., `property_bank.json`) use snake_case naming.
- `testdata/templates/` – Template fixtures (`static_template.md`,
  `basic_note.md`, etc.). Text fixtures such as
  `integration_test_template.txt` are allowed for negative tests.
- `testdata/golden/` – Expected outputs for regression assertions. We retain
  this top-level directory to avoid conflating input templates with rendered
  artifacts.
- `testdata/vault/` – Configuration samples (e.g., `lithos.json`).
- Top-level fixtures (e.g., `basic_note.md`) exist for legacy compatibility and
  must remain in snake_case form.

All fixture names **must** use snake_case (lowercase letters, digits, optional
underscores) with file extensions in lowercase.

## Helper Utilities

- `tests/utils/testdata.go` exposes `Root`, `Path`, and `Open` helpers. These
  enforce the canonical layout and snake_case naming. Use `Path(t, segments...)`
  instead of hand-built `filepath.Join` calls.
- `tempfs` utilities (Story 1.14) provide workspace management for tests that
  need scratch directories. Prefer `tempfs.NewWorkspace` once available and
  avoid manual `os.MkdirTemp` usage.

## Required Validations

- Run `go test ./...` after modifying fixtures or utilities to ensure the new
  layout is honoured.
- `golangci-lint run ./...` must pass; helpers are subject to the same linting
  rules as production code.
