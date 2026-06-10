# Architecture Tests: Template Context Isolation

Status: ready-for-agent

## Parent

`.scratch/template-foundation/PRD.md`

## What to build

Extend the existing architecture tests in `lithos-core/tests/architecture.rs` to enforce Template context isolation boundaries. The existing tests already cover note/schema cross-imports and FS isolation — this slice adds parallel coverage for the template context.

Rules to enforce:

1. **No cross-context imports** — `template` context must not import `crate::note` or `crate::schema`. The existing `contexts_must_not_import_each_other` test should be extended or a new test added to cover `template` as a participant.

2. **MiniJinja confined to adapter** — `minijinja` must not appear in template domain models, repository traits, service request/response types, or `TemplateError`. It is only permitted inside the `template/engine/` adapter module (or equivalent adapter boundary). A new architecture test should enforce this by scanning non-adapter template source files for `minijinja` imports.

3. **No raw `std::fs` in template use cases** — template service, processor, and artifact pipeline must not import `std::fs` directly. The existing `ports_must_not_import_std_fs` test may already cover this if template modules are included in its scope; verify and extend if not.

Prior art: the existing tests in `lithos-core/tests/architecture.rs` use source-file scanning (reading `.rs` files and checking for forbidden import patterns). Follow the same approach.

## Acceptance criteria

- [ ] `contexts_must_not_import_each_other` test (or a new companion test) covers `template` context: `template` does not import `crate::note` or `crate::schema`
- [ ] A test enforces that `minijinja` does not appear in template source files outside the designated adapter module
- [ ] The FS isolation test covers template modules (no raw `std::fs` in template service, processor, or artifact pipeline)
- [ ] All three tests pass after the implementation slices (1–8) are complete
- [ ] No false positives: the adapter module itself is correctly excluded from the MiniJinja confinement check

## Blocked by

- `issue-05-engine-port-adapter.md` (defines the adapter boundary that the MiniJinja confinement test must reference)
- `issue-07-template-service.md` (provides the full module tree the tests scan)
