# Test Design: Story 3.2 - Create Note Bounded Context

**Date:** Wed Jan 14 2026
**Author:** Jack (via Murat - Master Test Architect)
**Status:** Approved
**Story Link:** [\_bmad-output/implementation-artifacts/stories/3-2-create-note-bounded-context.md](_bmad-output/implementation-artifacts/stories/3-2-create-note-bounded-context.md)

---

## Executive Summary

**Scope:** Targeted test design for the Note aggregate and its 8 subentities (Frontmatter, Links, Embeds, Tags, Headings, Tasks, Sections).

**Risk Summary:**

- **R-002 (Identity):** High Risk. Failure to ensure UUID v7 monotonicity breaks time-ordered indexing.
- **R-004 (Security):** High Risk. Path traversal in Note/Embed paths could expose sensitive system files.
- **R-003 (Perf):** Medium Risk. Complex subentity parsing could exceed 100μs budget.

**Coverage Summary:**

- P0 (Critical): 12 scenarios
- P1 (High): 18 scenarios
- P2 (Medium): 15 scenarios
- **Total Effort:** 45 scenarios (~24 hours)

---

## Risk Assessment

| Risk ID   | Category | Description                                                                          | Probability | Impact | Score | Mitigation                                                                        | Owner |
| --------- | -------- | ------------------------------------------------------------------------------------ | ----------- | ------ | ----- | --------------------------------------------------------------------------------- | ----- |
| **R-002** | DATA     | UUID v7 generation fails monotonicity or uses non-v7 format, breaking vault sorting. | 1           | 3      | 3     | Property-based testing (proptest) for 10k sequential IDs.                         | DEV   |
| **R-004** | SEC      | Path traversal via `../` or absolute paths in Note/Embed paths.                      | 2           | 3      | 6     | Strict regex and `Path` component validation; rejection of non-relative segments. | DEV   |
| **R-003** | PERF     | Note construction > 100μs due to excessive allocations in subentity parsing.         | 2           | 3      | 6     | Use `Criterion` benchmarks; prefer `Box<str>`/`SmolStr` for immutable paths/tags. | DEV   |
| **R-009** | DATA     | Incorrect Wiki-link alias resolution leads to broken graph integrity.                | 2           | 2      | 4     | Unit tests for all Wiki-link permutations (`[[T]]`, `[[T\|A]]`, `[[T#H]]`).       | DEV   |

---

## Test Coverage Plan

### P0 (Critical) - Identity & Security (Run on every commit)

| Requirement              | Test Level  | Risk Link | Test Count | Notes                                                          |
| ------------------------ | ----------- | --------- | ---------- | -------------------------------------------------------------- |
| UUID v7 Monotonicity     | Unit (Prop) | R-002     | 1          | Verify 1000 sequential Note IDs are strictly increasing.       |
| Path Traversal Rejection | Unit        | R-004     | 5          | Rejection of `../`, `/`, `C:\`, `\\network\`, and empty paths. |
| Note ID Stability        | Unit        | R-002     | 1          | Verify ID remains unchanged when Note path is updated.         |
| Hexagonal Purity         | Unit        | -         | 1          | Automated check: `crates/domain` has zero I/O dependencies.    |

### P1 (High) - Subentity Logic (Run on PR)

| Requirement              | Test Level | Risk Link | Test Count | Notes                                                           |
| ------------------------ | ---------- | --------- | ---------- | --------------------------------------------------------------- |
| Hierarchical Tag Parsing | Unit       | -         | 6          | Valid: `#a/b/c`. Invalid: `#a//b`, `#a b`, `#a/`, `/a`.         |
| Wiki-link Parsing        | Unit       | R-009     | 4          | Support for aliases, headers, and position tracking.            |
| Frontmatter Best-Effort  | Unit       | -         | 5          | Date parsing (ISO/Moment), Number, Bool, fallback to String.    |
| Domain Event Emission    | Unit       | -         | 2          | `NoteCreated` and `NoteFrontmatterValidated` emitted correctly. |

### P2 (Medium) - Edge Cases (Run nightly)

| Requirement            | Test Level | Risk Link | Test Count | Notes                                               |
| ---------------------- | ---------- | --------- | ---------- | --------------------------------------------------- |
| Heading Level Limits   | Unit       | -         | 3          | Valid: 1-6. Invalid: 0, 7.                          |
| Task Status Variants   | Unit       | -         | 4          | `[ ]`, `[x]`, `[-]`, `[/]`, `[>]`.                  |
| Empty Note Composition | Unit       | -         | 1          | Construct Note with 0 tags, 0 links, 0 frontmatter. |
| Unicode Path Support   | Unit       | -         | 2          | Valid: `Vault/Журнал.md`.                           |

---

## Specific Test Scenarios (Gherkin-Style)

### Security: Path Traversal (R-004)

- **Scenario:** Rejects parent directory traversal
  - **Given** a note is being created
  - **When** the path is `../../etc/passwd`
  - **Then** construction fails with `DomainError::InvalidPath`
- **Scenario:** Rejects absolute paths
  - **Given** a note is being created
  - **When** the path is `/home/user/vault/note.md`
  - **Then** construction fails with `DomainError::InvalidPath`

### Identity: UUID v7 Stability (R-002)

- **Scenario:** Generates strictly monotonic IDs
  - **Given** I generate 10,000 Note IDs using the domain constructor
  - **Then** every ID must be greater than the previous ID
  - **And** the timestamp portion must match the current system time (virtual clock)

### Performance: Construction Latency (R-003)

- **Scenario:** Benchmarks typical note construction
  - **Given** a standard note template (5 tags, 10 links, 5 headings)
  - **When** the Note aggregate is constructed
  - **Then** execution time MUST be `< 100μs`

---

## Execution Order

### Smoke Tests (<1 min)

- [ ] Construct valid Note with minimum fields.
- [ ] Validate a standard Wiki-link with alias.
- [ ] Parse a simple hierarchical tag.

### P0 Suite (<5 min)

- [ ] Full Path Traversal suite (10 cases).
- [ ] UUID v7 Property-based test (1000 iterations).
- [ ] Domain Event emission check.

---

## Quality Gate Criteria

- **Pass Rate:** 100% P0, 100% Security tests.
- **Coverage:** >85% for `crates/domain/src/note/`.
- **Perf:** Must pass `mise run bench` with no regressions.

## Next Steps

1. **Developer:** Implement `Note::new()` with UUID v7 and Path validation first (RED phase).
2. **QA:** Run `*atdd` to generate initial failing tests for path traversal.
3. **Architect:** Verify `domain/Cargo.toml` remains dependency-pure.

---

**Generated by**: BMad TEA Agent - Murat
**Workflow**: `[TD] Test Design`
