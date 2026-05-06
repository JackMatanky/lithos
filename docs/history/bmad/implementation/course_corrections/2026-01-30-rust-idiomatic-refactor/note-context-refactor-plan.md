# Note Context Refactor Implementation Plan

**Date**: 2026-02-08
**Context**: Course correction for Note Bounded Context
**Specs**:
- `docs/design/004-note-models.md`
- `docs/design/005-note-cqrs.md`
- `docs/design/006-note-frontmatter.md`
- `docs/design/007-note-list-task.md`

## Overview

This plan aligns the `lithos-core/src/note` context with the reviewed design specifications. The goal is to enforce idiomatic Rust patterns, type-driven design, and strict rkyv compatibility, while enabling rich task/list modeling.

## Validation Status

- [x] **004-note-models.md**: Coherent. Defines core aggregate and value objects.
- [x] **005-note-cqrs.md**: Coherent. Defines split error types and zero-copy query strategy.
- [x] **006-note-frontmatter.md**: Coherent. Integrates `FieldValue` and strict/lenient extraction.
- [x] **007-note-list-task.md**: Coherent. Defines `List`, `Task`, `TaskMetadata` and usage of `FieldValue`.

## Implementation Phases

### Phase 1: Shared Primitives
**Goal**: Establish the `FieldValue` primitive used by Frontmatter and Tasks.
- [ ] Create `lithos-core/src/note/value.rs`.
- [ ] Implement `FieldValue` enum (String, Number, Boolean, Date, Array, Object).
- [ ] Implement `From` implementations or helper constructors.
- [ ] Add unit tests for `FieldValue`.

### Phase 2: Domain Value Objects
**Goal**: Implement the leaf node value objects.
- [ ] **Tag**: Update `lithos-core/src/note/tag.rs` (Tag, TagPath).
- [ ] **Link**: Update `lithos-core/src/note/link.rs` (Link, Target, Anchor).
- [ ] **Structure**: Update `lithos-core/src/note/structure.rs` (Heading, Section).
- [ ] **List**: Create `lithos-core/src/note/list.rs` (List, ListItem, ListType) per Spec 007.

### Phase 3: Task & Metadata
**Goal**: Implement Task entity and Metadata container.
- [ ] Create `lithos-core/src/note/task.rs` per Spec 007.
- [ ] Implement `TaskMetadata` using `FieldValue`.
- [ ] Implement `Task` struct (promoted entity).
- [ ] Implement `Task::from_checkbox` (logic stub or full impl depending on Config availability).

### Phase 4: Frontmatter
**Goal**: Refactor Frontmatter to use `FieldValue`.
- [ ] Update `lithos-core/src/note/frontmatter.rs` per Spec 006.
- [ ] Replace internal Map with `HashMap<String, FieldValue>`.
- [ ] Implement strict accessors (`try_get`, etc.).
- [ ] Implement leniency wrappers.

### Phase 5: Aggregate Root
**Goal**: Assemble the `Note` aggregate.
- [ ] Update `lithos-core/src/note/aggregate.rs` per Spec 004.
- [ ] Update fields to `lists: Vec<List>`, `tasks: Vec<Task>`.
- [ ] Ensure `NotePath` validation logic is present.
- [ ] Ensure rkyv derives are correct.

### Phase 6: CQRS Ports & Errors
**Goal**: Define the boundaries.
- [ ] Update `lithos-core/src/note/error.rs` to include `FrontmatterError`, etc.
- [ ] Update `lithos-core/src/note/ports.rs` with `Command` and `Query` traits.
- [ ] Define `NoteCommandError` and `NoteQueryError` in `lithos-core/src/note/error.rs` (or `ports.rs` if circular deps arise).

### Phase 7: Infrastructure (CQRS Impl)
**Goal**: Connect to DB.
- [ ] Update `lithos-core/src/note/command.rs`.
- [ ] Update `lithos-core/src/note/query.rs`.
- [ ] Implement zero-copy helpers (`with_archived_by_id`).

### Phase 8: Verification
- [ ] Run `cargo test -p lithos-core --lib note`.
- [ ] Verify no circular dependencies.
- [ ] Verify rkyv derives compile.

## Dependencies

- Requires `TaskConfig` from Config context (mockable for now if needed).
- Requires `pulldown-cmark` for parser adapters (separate from domain models).

## Rollout Strategy

1. Create git worktree `refactor/note-context`.
2. Execute phases sequentially.
3. Commit after each phase.
