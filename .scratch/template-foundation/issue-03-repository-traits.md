# Template Repository Traits

Status: ready-for-agent

## Parent

`.scratch/template-foundation/PRD.md`

## What to build

Define the Template repository port: segregated `ReadRepository`, `WriteRepository`, and unified `Repository` traits following the exact pattern established by the note and schema contexts.

The traits cover:
- CRUD operations for `Template` (find by id, find by path/name, save, delete)
- CRUD operations for `RawTemplateView` (find by path, save, delete)
- Batch raw-view operations for efficient multi-path discovery and atomic cache updates

No filesystem materialization belongs here — the traits are pure persistence ports. No implementation (redb adapter) is required in this slice; the traits and a test-double/in-memory implementation for use in other slices' tests are sufficient.

## Acceptance criteria

- [ ] `ReadRepository` trait defined with at minimum: find `Template` by `TemplateId`, find `Template` by `TemplateName`, list all `Template`s, find `RawTemplateView` by `PathKey`, batch find `RawTemplateView` by a set of `PathKey`s
- [ ] `WriteRepository` trait defined with at minimum: save `Template`, delete `Template` by `TemplateId`, save `RawTemplateView`, delete `RawTemplateView` by `PathKey`, batch save `RawTemplateView`s
- [ ] `Repository` is a unified blanket marker trait: `impl<T: ReadRepository + WriteRepository> Repository for T {}`
- [ ] No filesystem I/O or MiniJinja types appear in any trait method signature
- [ ] An in-memory test implementation is provided (or a testing module following `schema/storage/testing.rs` pattern) so downstream slices can write unit tests without a real redb adapter
- [ ] Repository contract tests cover: save and find round-trip for `Template`, missing-entity behavior (returns `None` or appropriate error), `RawTemplateView` save/find/delete, batch raw-view operations, find-by-name and find-by-id correctness

## Blocked by

- `issue-01-domain-models.md`
